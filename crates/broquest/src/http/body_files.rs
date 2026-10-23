//! `< file` and `<@ file` lines in the bodies of `.http` requests.
//!
//! `< path` sends the file's bytes in place of the line. `<@ path` (VS Code)
//! puts the file's text in, with its `{{variables}}` resolved like the rest
//! of the request. `<@latin1 path` names the file's encoding. Paths are
//! relative to the `.http` file's folder.

use std::fs;
use std::path::Path;

/// The path of a `< path` line.
fn file_ref(line: &str) -> Option<&str> {
    let path = line.strip_prefix('<')?;
    path.starts_with([' ', '\t']).then(|| path.trim())
}

/// The encoding and path of a `<@ path` or `<@encoding path` line.
fn text_ref(line: &str) -> Option<(Option<&str>, &str)> {
    let rest = line.strip_prefix("<@")?;
    if rest.starts_with([' ', '\t']) {
        return Some((None, rest.trim()));
    }
    let (encoding, path) = rest.split_once([' ', '\t'])?;
    Some((Some(encoding), path.trim()))
}

/// Replace `<@ path` lines with the text of their files.
pub fn expand_text_refs(body: &str, dir: &Path) -> String {
    if !body.lines().any(|line| text_ref(line).is_some()) {
        return body.to_string();
    }
    body.lines()
        .map(|line| match text_ref(line) {
            Some((encoding, path)) => {
                read_text(&dir.join(path), encoding).unwrap_or_else(|| line.to_string())
            }
            None => line.to_string(),
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn read_text(path: &Path, encoding: Option<&str>) -> Option<String> {
    let bytes = fs::read(path)
        .map_err(|e| tracing::warn!("Could not read body file {}: {}", path.display(), e))
        .ok()?;
    let encoding = encoding
        .and_then(|label| encoding_rs::Encoding::for_label(label.as_bytes()))
        .unwrap_or(encoding_rs::UTF_8);
    Some(encoding.decode(&bytes).0.into_owned())
}

/// The body to send: `< path` lines become the files' bytes. Multipart
/// bodies get CRLF line endings, as the multipart format requires.
pub fn body_bytes(body: &str, dir: &Path, multipart: bool) -> Vec<u8> {
    let has_refs = body.lines().any(|line| file_ref(line).is_some());
    if !has_refs && !multipart {
        return body.as_bytes().to_vec();
    }
    let separator: &[u8] = if multipart { b"\r\n" } else { b"\n" };
    let mut out = Vec::with_capacity(body.len());
    for (n, line) in body.lines().enumerate() {
        if n > 0 {
            out.extend_from_slice(separator);
        }
        match file_ref(line).map(|path| (path, fs::read(dir.join(path)))) {
            Some((_, Ok(bytes))) => out.extend_from_slice(&bytes),
            Some((path, Err(error))) => {
                tracing::warn!("Could not read body file {}: {}", path, error);
                out.extend_from_slice(line.as_bytes());
            }
            None => out.extend_from_slice(line.as_bytes()),
        }
    }
    if multipart {
        out.extend_from_slice(separator);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir_with(files: &[(&str, &[u8])]) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "broquest-body-files-{}-{}",
            std::process::id(),
            fastrand::u64(..)
        ));
        fs::create_dir_all(&dir).unwrap();
        for (name, bytes) in files {
            fs::write(dir.join(name), bytes).unwrap();
        }
        dir
    }

    #[test]
    fn whole_body_and_inline_files() {
        let dir = dir_with(&[("a.bin", &[0, 159, 146, 150]), ("b.json", b"{\"b\": 1}")]);
        assert_eq!(body_bytes("< ./a.bin", &dir, false), vec![0, 159, 146, 150]);
        assert_eq!(
            body_bytes("x\n< b.json\ny", &dir, false),
            b"x\n{\"b\": 1}\ny"
        );
        // A missing file keeps its line.
        assert_eq!(body_bytes("< nope.txt", &dir, false), b"< nope.txt");
        // `<>` and `<@` lines are not byte references.
        assert_eq!(body_bytes("<> ref.json", &dir, false), b"<> ref.json");
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn multipart_bodies_use_crlf() {
        let dir = dir_with(&[("label.txt", b"label")]);
        let body = "--b\nContent-Disposition: form-data; name=\"f\"; filename=\"label.txt\"\n\n< ./label.txt\n--b--";
        assert_eq!(
            body_bytes(body, &dir, true),
            b"--b\r\nContent-Disposition: form-data; name=\"f\"; filename=\"label.txt\"\r\n\r\nlabel\r\n--b--\r\n"
        );
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn text_refs_bring_in_file_text() {
        let dir = dir_with(&[
            ("t.json", "{\"user\": \"{{user}}\"}".as_bytes()),
            ("l.txt", b"caf\xe9"),
        ]);
        assert_eq!(
            expand_text_refs("<@ ./t.json", &dir),
            "{\"user\": \"{{user}}\"}"
        );
        assert_eq!(expand_text_refs("<@latin1 l.txt", &dir), "café");
        assert_eq!(expand_text_refs("plain", &dir), "plain");
        assert_eq!(expand_text_refs("<@ missing.json", &dir), "<@ missing.json");
        fs::remove_dir_all(dir).ok();
    }
}
