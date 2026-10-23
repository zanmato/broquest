//! Runs the parser over every file in `tests/corpus` and checks:
//!
//! - the blocks and their lines cover the file exactly, so no byte is lost,
//! - every parsed piece's text is what its span points at,
//! - the parse matches the reviewed summary in the `.parsed` file beside it.
//!
//! Run with `BLESS=1` to rewrite the `.parsed` files after a deliberate
//! change, then review the diff.

use std::fmt::Write as _;
use std::fs;
use std::path::{Path, PathBuf};

use http_file::{HttpFile, LineKind, Script, Text, parse};

fn corpus_files() -> Vec<PathBuf> {
    fn walk(dir: &Path, files: &mut Vec<PathBuf>) {
        for entry in fs::read_dir(dir).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                walk(&path, files);
            } else if matches!(
                path.extension().and_then(|e| e.to_str()),
                Some("http" | "rest")
            ) {
                files.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/corpus"),
        &mut files,
    );
    files.sort();
    files
}

/// Every [`Text`] in the parse.
fn texts(file: &HttpFile) -> Vec<&Text> {
    let mut texts = Vec::new();
    for block in &file.blocks {
        texts.extend(&block.title);
        for variable in &block.variables {
            texts.extend([&variable.name, &variable.value]);
        }
        let Some(request) = &block.request else {
            continue;
        };
        for directive in &request.directives {
            texts.push(&directive.name);
            texts.extend(&directive.value);
        }
        texts.extend(&request.method);
        texts.push(&request.target);
        texts.extend(&request.target_continuations);
        texts.extend(&request.version);
        for header in &request.headers {
            texts.extend([&header.name, &header.value]);
            texts.extend(&header.continuations);
        }
        texts.extend(request.body.as_ref().map(|body| &body.content));
        for script in request
            .pre_request_scripts
            .iter()
            .chain(&request.response_handlers)
        {
            texts.push(match script {
                Script::Inline { code, .. } => code,
                Script::File { path, .. } => path,
            });
        }
        texts.extend(&request.response_refs);
        texts.extend(request.response_redirects.iter().map(|r| &r.path));
    }
    texts
}

fn check_invariants(path: &Path, file: &HttpFile) {
    let name = path.display();
    let mut at = 0;
    for block in &file.blocks {
        assert_eq!(block.span.start, at, "{name}: gap before block at {at}");
        let mut line_at = block.span.start;
        for line in &block.lines {
            assert_eq!(
                line.span.start, line_at,
                "{name}: gap before line at {line_at}"
            );
            assert!(
                line.span.end > line.span.start,
                "{name}: empty line at {line_at}"
            );
            line_at = line.span.end;
        }
        assert_eq!(
            line_at, block.span.end,
            "{name}: lines end before their block"
        );
        if let Some(request) = &block.request {
            assert!(
                block.span.start <= request.span.start && request.span.end <= block.span.end,
                "{name}: request outside its block"
            );
        }
        at = block.span.end;
    }
    assert_eq!(
        at,
        file.source.len(),
        "{name}: blocks end before the file does"
    );

    for text in texts(file) {
        assert_eq!(
            &file.source[text.span.clone()],
            text.text,
            "{name}: text does not match its span {:?}",
            text.span
        );
    }
}

/// A readable account of the parse, for review.
fn summary(file: &HttpFile) -> String {
    let mut out = String::new();
    for (n, block) in file.blocks.iter().enumerate() {
        write!(out, "block {}", n + 1).unwrap();
        if let Some(title) = &block.title {
            write!(out, " {:?}", title.text).unwrap();
        }
        out.push('\n');

        let mut kinds: Vec<(LineKind, usize)> = Vec::new();
        for line in &block.lines {
            match kinds.last_mut() {
                Some((kind, count)) if *kind == line.kind => *count += 1,
                _ => kinds.push((line.kind, 1)),
            }
        }
        let kinds: Vec<String> = kinds
            .iter()
            .map(|(kind, count)| match count {
                1 => format!("{kind:?}"),
                _ => format!("{kind:?} x{count}"),
            })
            .collect();
        writeln!(out, "  lines: {}", kinds.join(", ")).unwrap();

        for variable in &block.variables {
            writeln!(
                out,
                "  variable {} = {:?}",
                variable.name.text, variable.value.text
            )
            .unwrap();
        }
        let Some(request) = &block.request else {
            continue;
        };
        write!(out, "  request {} {:?}", request.method(), request.url()).unwrap();
        if request.method.is_none() {
            out.push_str(" (no method)");
        }
        if let Some(version) = &request.version {
            write!(out, " {}", version.text).unwrap();
        }
        out.push('\n');
        for directive in &request.directives {
            match &directive.value {
                Some(value) => writeln!(out, "    @{} {:?}", directive.name.text, value.text),
                None => writeln!(out, "    @{}", directive.name.text),
            }
            .unwrap();
        }
        for header in &request.headers {
            writeln!(
                out,
                "    header {}: {:?}",
                header.name.text,
                header.full_value()
            )
            .unwrap();
        }
        if let Some(body) = &request.body {
            match body.file_ref() {
                Some(path) => writeln!(out, "    body from file {path:?}"),
                None => writeln!(out, "    body {:?}", body.content.text),
            }
            .unwrap();
        }
        for (label, scripts) in [
            ("pre-request script", &request.pre_request_scripts),
            ("response handler", &request.response_handlers),
        ] {
            for script in scripts {
                match script {
                    Script::Inline { code, .. } => {
                        writeln!(out, "    {label} {:?}", code.text.trim())
                    }
                    Script::File { path, .. } => writeln!(out, "    {label} file {:?}", path.text),
                }
                .unwrap();
            }
        }
        for reference in &request.response_refs {
            writeln!(out, "    response ref {:?}", reference.text).unwrap();
        }
        for redirect in &request.response_redirects {
            let arrow = if redirect.force { ">>!" } else { ">>" };
            writeln!(out, "    {arrow} {:?}", redirect.path.text).unwrap();
        }
    }
    for diagnostic in &file.diagnostics {
        writeln!(
            out,
            "diagnostic {:?}: {}",
            &file.source[diagnostic.span.clone()],
            diagnostic.message
        )
        .unwrap();
    }
    out
}

#[test]
fn corpus() {
    let bless = std::env::var_os("BLESS").is_some();
    let files = corpus_files();
    assert!(files.len() > 50, "corpus went missing");

    let mut mismatches = Vec::new();
    for path in &files {
        let source = fs::read_to_string(path).unwrap();
        let file = parse(&source);
        check_invariants(path, &file);

        let parsed = summary(&file);
        let expected_path = path.with_extension(format!(
            "{}.parsed",
            path.extension().unwrap().to_str().unwrap()
        ));
        if bless {
            fs::write(&expected_path, &parsed).unwrap();
        } else if fs::read_to_string(&expected_path).ok().as_deref() != Some(parsed.as_str()) {
            mismatches.push(expected_path.display().to_string());
        }
    }
    assert!(
        mismatches.is_empty(),
        "parse summaries differ, run with BLESS=1 and review the diff:\n{}",
        mismatches.join("\n")
    );
}
