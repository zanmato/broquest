//! Editing a parsed file by changing only the bytes that differ.
//!
//! Each function returns [`Change`]s against [`HttpFile::source`]. Apply them
//! together with [`apply`]. Everything a change does not cover stays as it
//! was, comments, blank lines and layout included. New text uses the file's
//! own line endings.

use crate::{Block, HttpFile, Line, Request, Script, Span};

/// Replace `span` of the source with `text`. An empty span inserts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub span: Span,
    pub text: String,
}

/// Apply changes that do not overlap. Changes at the same offset apply in
/// the order given.
pub fn apply(source: &str, changes: &[Change]) -> String {
    let mut order: Vec<usize> = (0..changes.len()).collect();
    order.sort_by_key(|&i| (changes[i].span.start, changes[i].span.end));
    let mut out = String::with_capacity(source.len());
    let mut at = 0;
    for i in order {
        let change = &changes[i];
        assert!(
            change.span.start >= at,
            "changes overlap at {}",
            change.span.start
        );
        out.push_str(&source[at..change.span.start]);
        out.push_str(&change.text);
        at = change.span.end;
    }
    out.push_str(&source[at..]);
    out
}

impl HttpFile {
    /// The file's line ending, `\r\n` when it uses them, else `\n`.
    pub fn line_ending(&self) -> &'static str {
        if self.source.contains("\r\n") {
            "\r\n"
        } else {
            "\n"
        }
    }
}

/// What to change in a request. `None` leaves that part as it is.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RequestEdit {
    /// Name the request: sets its `@name`, else its `###` title, else adds a
    /// `# @name` line.
    pub name: Option<String>,
    pub method: Option<String>,
    /// The whole target, query included.
    pub url: Option<String>,
    /// Header names and values. Replaces all headers.
    pub headers: Option<Vec<(String, String)>>,
    /// The body. An empty body removes it.
    pub body: Option<String>,
    /// `Some(None)` removes the pre-request scripts, `Some(Some(code))`
    /// makes them one inline script.
    pub pre_request_script: Option<Option<String>>,
    /// As `pre_request_script`, for response handlers.
    pub response_handler: Option<Option<String>>,
}

/// A request to write as a new block.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NewRequest {
    pub name: Option<String>,
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub pre_request_script: Option<String>,
    pub response_handler: Option<String>,
}

/// `text` with its line breaks in the file's style.
fn with_line_ending(text: &str, le: &str) -> String {
    let unified = text.replace("\r\n", "\n");
    if le == "\n" {
        unified
    } else {
        unified.replace('\n', le)
    }
}

fn inline_script(marker: char, code: &str, le: &str) -> String {
    format!(
        "{marker} {{%{le}{}{le}%}}",
        with_line_ending(code.trim_matches('\n'), le)
    )
}

/// The text of a new block, ending with a line ending.
pub fn render(request: &NewRequest, le: &str) -> String {
    let mut out = String::new();
    match &request.name {
        Some(name) => out.push_str(&format!("### {name}{le}")),
        None => out.push_str(&format!("###{le}")),
    }
    if let Some(code) = &request.pre_request_script {
        out.push_str(&inline_script('<', code, le));
        out.push_str(le);
    }
    out.push_str(&format!("{} {}{le}", request.method, request.url));
    for (name, value) in &request.headers {
        out.push_str(&format!("{name}: {value}{le}"));
    }
    if !request.body.trim().is_empty() {
        out.push_str(le);
        out.push_str(&with_line_ending(request.body.trim(), le));
        out.push_str(le);
    }
    if let Some(code) = &request.response_handler {
        out.push_str(le);
        out.push_str(&inline_script('>', code, le));
        out.push_str(le);
    }
    out
}

/// Add `request` at the end of the file, after a blank line.
pub fn append_request(file: &HttpFile, request: &NewRequest) -> Vec<Change> {
    let le = file.line_ending();
    let source = &file.source;
    let mut prefix = String::new();
    if !source.is_empty() {
        if !source.ends_with(['\n', '\r']) {
            prefix.push_str(le);
        }
        if !source.trim_end_matches(['\n', '\r']).is_empty() && !ends_with_blank_line(source) {
            prefix.push_str(le);
        }
    }
    let end = source.len();
    vec![Change {
        span: end..end,
        text: format!("{prefix}{}", render(request, le)),
    }]
}

fn ends_with_blank_line(source: &str) -> bool {
    let trimmed = source.trim_end_matches([' ', '\t']);
    trimmed.ends_with("\n\n") || trimmed.ends_with("\r\n\r\n") || trimmed.ends_with("\r\r")
}

/// Where a request's own lines start: its first directive, pre-request
/// script or request line.
fn request_start(request: &Request) -> usize {
    request
        .directives
        .iter()
        .map(|d| d.span.start)
        .chain(
            request
                .pre_request_scripts
                .iter()
                .map(|s| script_span(s).start),
        )
        .chain([request.span.start])
        .min()
        .unwrap_or(request.span.start)
}

fn script_span(script: &Script) -> &Span {
    match script {
        Script::Inline { span, .. } | Script::File { span, .. } => span,
    }
}

/// Remove a block. A block without a `###` separator keeps what comes before
/// its request, such as file variables.
pub fn remove_block(file: &HttpFile, block: usize) -> Vec<Change> {
    let block = &file.blocks[block];
    let has_separator = block
        .lines
        .first()
        .is_some_and(|line| line.kind == crate::LineKind::Separator);
    let start = match (&block.request, has_separator) {
        (Some(request), false) => request_start(request),
        _ => block.span.start,
    };
    vec![Change {
        span: start..block.span.end,
        text: String::new(),
    }]
}

/// The line holding byte `offset` of the block.
fn line_at(block: &Block, offset: usize) -> Option<&Line> {
    block
        .lines
        .iter()
        .find(|line| line.span.start <= offset && offset < line.span.end)
        .or_else(|| {
            block
                .lines
                .iter()
                .rev()
                .find(|line| line.span.end == offset)
        })
}

/// From the start of the line holding `span.start` to the end of the line
/// holding the end of `span`, line ending included.
fn whole_lines(block: &Block, span: &Span) -> Span {
    let start = line_at(block, span.start).map_or(span.start, |line| line.span.start);
    let last = span.end.saturating_sub(1).max(span.start);
    let end = line_at(block, last).map_or(span.end, |line| line.span.end);
    start..end
}

/// Changes that edit the request of `block` as `edit` asks.
pub fn update_request(file: &HttpFile, block_index: usize, edit: &RequestEdit) -> Vec<Change> {
    let le = file.line_ending();
    let block = &file.blocks[block_index];
    let Some(request) = &block.request else {
        return Vec::new();
    };
    let source = &file.source;
    let mut changes = Vec::new();
    let insert = |at: usize, text: String| Change { span: at..at, text };
    // Text inserted after a line that may lack its line ending (end of file).
    let after_line = |line_end: usize, text: String| {
        if source[..line_end].ends_with(['\n', '\r']) || line_end == 0 {
            text
        } else {
            format!("{le}{text}")
        }
    };

    // Name.
    if let Some(name) = &edit.name {
        if let Some(directive) = request.directive("name") {
            match &directive.value {
                Some(value) if value.text != *name => changes.push(Change {
                    span: value.span.clone(),
                    text: name.clone(),
                }),
                Some(_) => {}
                None => changes.push(Change {
                    span: directive.span.clone(),
                    text: format!("# @name {name}"),
                }),
            }
        } else if let Some(title) = &block.title {
            if title.text != *name {
                changes.push(Change {
                    span: title.span.clone(),
                    text: name.clone(),
                });
            }
        } else {
            changes.push(insert(
                request_start(request),
                format!("# @name {name}{le}"),
            ));
        }
    }

    // Pre-request scripts.
    if let Some(script) = &edit.pre_request_script {
        changes.extend(replace_scripts(
            block,
            &request.pre_request_scripts,
            script.as_deref(),
            '<',
            request.span.start,
            le,
            source,
        ));
    }

    // Method and target.
    let target_end = request
        .target_continuations
        .last()
        .map_or(request.target.span.end, |part| part.span.end);
    if let Some(method) = &edit.method
        && !method.eq_ignore_ascii_case(request.method())
    {
        match &request.method {
            Some(old) => changes.push(Change {
                span: old.span.clone(),
                text: method.clone(),
            }),
            None => changes.push(insert(request.target.span.start, format!("{method} "))),
        }
    }
    if let Some(url) = &edit.url
        && *url != request.url()
    {
        changes.push(Change {
            span: request.target.span.start..target_end,
            text: url.clone(),
        });
    }
    let target_line_end = line_at(block, target_end).map_or(target_end, |line| line.span.end);

    // Headers.
    let header_section_end = request
        .headers
        .last()
        .and_then(|header| line_at(block, header.span.end.saturating_sub(1)))
        .map_or(target_line_end, |line| line.span.end);
    if let Some(headers) = &edit.headers {
        let current: Vec<(String, String)> = request
            .headers
            .iter()
            .map(|h| (h.name.text.clone(), h.full_value()))
            .collect();
        if current != *headers {
            let same_names = current.len() == headers.len()
                && current.iter().zip(headers).all(|(old, new)| old.0 == new.0);
            if same_names {
                for (header, (name, value)) in request.headers.iter().zip(headers) {
                    if header.full_value() == *value {
                        continue;
                    }
                    if header.continuations.is_empty() {
                        changes.push(Change {
                            span: header.value.span.clone(),
                            text: value.clone(),
                        });
                    } else {
                        changes.push(Change {
                            span: header.span.clone(),
                            text: format!("{name}: {value}"),
                        });
                    }
                }
            } else if let (Some(first), Some(_)) = (request.headers.first(), request.headers.last())
            {
                let lines: String = headers
                    .iter()
                    .map(|(name, value)| format!("{name}: {value}{le}"))
                    .collect();
                let region = whole_lines(block, &(first.span.start..header_section_end));
                changes.push(Change {
                    span: region.start..header_section_end,
                    text: after_line(region.start, lines),
                });
            } else {
                let lines: String = headers
                    .iter()
                    .map(|(name, value)| format!("{name}: {value}{le}"))
                    .collect();
                changes.push(insert(target_line_end, after_line(target_line_end, lines)));
            }
        }
    }

    // Body.
    if let Some(body) = &edit.body {
        let new_body = body.trim();
        let current = request.body.as_ref().map(|b| b.content.text.as_str());
        if current.unwrap_or("") != new_body {
            match (&request.body, new_body.is_empty()) {
                (Some(old), false) => changes.push(Change {
                    span: old.content.span.clone(),
                    text: with_line_ending(new_body, le),
                }),
                (Some(old), true) => {
                    // The blank line that opens the body goes with it.
                    let body_lines = whole_lines(block, &old.content.span);
                    changes.push(Change {
                        span: header_section_end.max(target_line_end)..body_lines.end,
                        text: String::new(),
                    });
                }
                (None, false) => changes.push(insert(
                    header_section_end,
                    after_line(
                        header_section_end,
                        format!("{le}{}{le}", with_line_ending(new_body, le)),
                    ),
                )),
                (None, true) => {}
            }
        }
    }

    // Response handlers.
    if let Some(script) = &edit.response_handler {
        changes.extend(replace_scripts(
            block,
            &request.response_handlers,
            script.as_deref(),
            '>',
            request.span.end,
            le,
            source,
        ));
    }

    changes
}

/// Make `scripts` one inline script with `code`, or remove them when `code`
/// is `None`. New scripts go at `insert_at`.
fn replace_scripts(
    block: &Block,
    scripts: &[Script],
    code: Option<&str>,
    marker: char,
    insert_at: usize,
    le: &str,
    source: &str,
) -> Vec<Change> {
    let current = match scripts {
        [Script::Inline { code, .. }] => Some(code.text.trim_matches(['\n', '\r']).to_string()),
        _ => None,
    };
    if scripts.len() == 1 && current.as_deref() == code {
        return Vec::new();
    }
    match (scripts.first(), scripts.last(), code) {
        (None, _, None) => Vec::new(),
        (None, _, Some(code)) => {
            let text = inline_script(marker, code, le);
            let text = if marker == '<' {
                format!("{text}{le}")
            } else {
                let lead = if insert_at > 0 && !source[..insert_at].ends_with(['\n', '\r']) {
                    le
                } else {
                    ""
                };
                format!("{lead}{le}{text}{le}")
            };
            vec![Change {
                span: insert_at..insert_at,
                text,
            }]
        }
        (Some(first), Some(last), code) => {
            let mut region = whole_lines(block, &(script_span(first).start..script_span(last).end));
            // A removed response handler takes the blank lines before it along.
            if code.is_none() && marker == '>' {
                for line in block.lines.iter().rev() {
                    if line.span.end == region.start && source[line.span.clone()].trim().is_empty()
                    {
                        region.start = line.span.start;
                    }
                }
            }
            let text = match code {
                Some(code) => format!("{}{le}", inline_script(marker, code, le)),
                None => String::new(),
            };
            vec![Change { span: region, text }]
        }
        _ => Vec::new(),
    }
}

/// Changes that make the file's `@name = value` variables `vars`, in place
/// where they already are. New ones go after the last variable, or at the
/// top of the file.
pub fn set_variables(file: &HttpFile, vars: &[(String, String)]) -> Vec<Change> {
    let le = file.line_ending();
    let mut changes = Vec::new();
    let mut seen: Vec<&str> = Vec::new();
    let mut last_end = None;
    for block in &file.blocks {
        for variable in &block.variables {
            let name = variable.name.text.as_str();
            let lines = whole_lines(block, &variable.span);
            last_end = Some(lines.end);
            if seen.contains(&name) {
                continue;
            }
            seen.push(name);
            match vars.iter().find(|(n, _)| n == name) {
                Some((_, value)) if *value != variable.value.text => changes.push(Change {
                    span: variable.value.span.clone(),
                    text: value.clone(),
                }),
                Some(_) => {}
                None => changes.push(Change {
                    span: lines,
                    text: String::new(),
                }),
            }
        }
    }

    let new: String = vars
        .iter()
        .filter(|(name, _)| !seen.contains(&name.as_str()))
        .map(|(name, value)| format!("@{name} = {value}{le}"))
        .collect();
    if !new.is_empty() {
        match last_end {
            Some(end) => {
                let lead = if file.source[..end].ends_with(['\n', '\r']) {
                    ""
                } else {
                    le
                };
                changes.push(Change {
                    span: end..end,
                    text: format!("{lead}{new}"),
                });
            }
            None => {
                let gap = if file.source.is_empty() { "" } else { le };
                changes.push(Change {
                    span: 0..0,
                    text: format!("{new}{gap}"),
                });
            }
        }
    }
    changes
}
