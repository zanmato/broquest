use crate::{
    Block, Body, Diagnostic, Directive, Header, HttpFile, Line, LineKind, Request,
    ResponseRedirect, Script, Span, Text, Variable,
};

/// Parse a `.http` or `.rest` file. Never fails, see [`HttpFile::diagnostics`].
pub fn parse(source: &str) -> HttpFile {
    let lines = split_lines(source);

    // A block starts at every `###` line, and at the top of the file.
    let mut starts: Vec<usize> = (0..lines.len())
        .filter(|&i| content(source, &lines[i]).starts_with("###"))
        .collect();
    if starts.first() != Some(&0) {
        starts.insert(0, 0);
    }

    let mut diagnostics = Vec::new();
    let blocks = starts
        .iter()
        .enumerate()
        .filter_map(|(n, &first)| {
            let last = starts.get(n + 1).copied().unwrap_or(lines.len());
            (first < last)
                .then(|| BlockParser::new(source, &lines[first..last], &mut diagnostics).parse())
        })
        .collect();

    HttpFile {
        source: source.to_string(),
        blocks,
        diagnostics,
    }
}

/// A source line: content from `start` to `end`, terminator up to `next`.
#[derive(Debug, Clone, Copy)]
struct RawLine {
    start: usize,
    end: usize,
    next: usize,
}

fn split_lines(source: &str) -> Vec<RawLine> {
    let bytes = source.as_bytes();
    let mut lines = Vec::new();
    let mut start = 0;
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'\n' => {
                lines.push(RawLine {
                    start,
                    end: i,
                    next: i + 1,
                });
                i += 1;
                start = i;
            }
            b'\r' => {
                let next = if bytes.get(i + 1) == Some(&b'\n') {
                    i + 2
                } else {
                    i + 1
                };
                lines.push(RawLine {
                    start,
                    end: i,
                    next,
                });
                i = next;
                start = i;
            }
            _ => i += 1,
        }
    }
    if start < bytes.len() {
        lines.push(RawLine {
            start,
            end: bytes.len(),
            next: bytes.len(),
        });
    }
    lines
}

fn content<'a>(source: &'a str, line: &RawLine) -> &'a str {
    &source[line.start..line.end]
}

fn is_space(c: char) -> bool {
    matches!(c, ' ' | '\t' | '\x0c')
}

/// `start..end` with surrounding whitespace removed.
fn trim_span(source: &str, start: usize, end: usize) -> Span {
    let text = &source[start..end];
    let leading = text.len() - text.trim_start_matches(is_space).len();
    let trailing = text.len() - text.trim_end_matches(is_space).len();
    if leading == text.len() {
        return start..start;
    }
    start + leading..end - trailing
}

fn text(source: &str, span: Span) -> Text {
    Text {
        text: source[span.clone()].to_string(),
        span,
    }
}

/// Where parsing is within a block.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum State {
    /// Before the request line: comments, directives, variables and
    /// pre-request scripts.
    Pre,
    /// Right after the request line or a target continuation.
    Target,
    Headers,
    Body,
    /// After the body: response handlers, refs and redirects.
    Trailer,
    /// The rest of the block is not a request.
    Skip,
}

/// What a line in the body or trailer starts with.
enum TrailerStart {
    Handler,
    Ref,
    Redirect { force: bool },
}

fn trailer_start(line: &str) -> Option<(TrailerStart, usize)> {
    if let Some(rest) = line.strip_prefix(">>!") {
        return Some((
            TrailerStart::Redirect { force: true },
            line.len() - rest.len(),
        ));
    }
    if let Some(rest) = line.strip_prefix(">>") {
        return Some((
            TrailerStart::Redirect { force: false },
            line.len() - rest.len(),
        ));
    }
    if let Some(rest) = line.strip_prefix("<>")
        && rest.starts_with(is_space)
    {
        return Some((TrailerStart::Ref, 2));
    }
    if let Some(rest) = line.strip_prefix('>')
        && (rest.starts_with(is_space) || rest.starts_with("{%"))
    {
        return Some((TrailerStart::Handler, 1));
    }
    None
}

fn is_comment(line: &str) -> bool {
    let line = line.trim_start_matches(is_space);
    line.starts_with('#') || line.starts_with("//")
}

const METHODS: &[&str] = &[
    "GET",
    "HEAD",
    "POST",
    "PUT",
    "DELETE",
    "CONNECT",
    "PATCH",
    "OPTIONS",
    "TRACE",
    "GRAPHQL",
    "WEBSOCKET",
    "GRPC",
];

fn is_method(token: &str) -> bool {
    !token.is_empty()
        && token.bytes().all(|b| b.is_ascii_alphabetic())
        && (METHODS.iter().any(|m| m.eq_ignore_ascii_case(token))
            || (token.len() >= 3 && token.bytes().all(|b| b.is_ascii_uppercase())))
}

/// A header field name: an HTTP token, plus `{{variables}}`.
fn is_header_name(name: &str) -> bool {
    let without_variables = name.replace("{{", "").replace("}}", "");
    !name.is_empty()
        && without_variables
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"!#$%&'*+-.^_`|~".contains(&b))
}

fn is_http_version(token: &str) -> bool {
    let Some(version) = token.strip_prefix("HTTP/") else {
        return false;
    };
    let mut parts = version.splitn(2, '.');
    let major = parts.next().unwrap_or("");
    let minor_ok = parts
        .next()
        .is_none_or(|minor| !minor.is_empty() && minor.bytes().all(|b| b.is_ascii_digit()));
    !major.is_empty() && major.bytes().all(|b| b.is_ascii_digit()) && minor_ok
}

struct BlockParser<'a> {
    source: &'a str,
    raw: &'a [RawLine],
    diagnostics: &'a mut Vec<Diagnostic>,
    lines: Vec<Line>,
    variables: Vec<Variable>,
    directives: Vec<Directive>,
    pre_request_scripts: Vec<Script>,
    request: Option<Request>,
    /// End of the last line that belongs to the request.
    request_end: usize,
    /// Trimmed spans of the first and last non-blank body lines.
    body: Option<(Span, Span)>,
    state: State,
}

impl<'a> BlockParser<'a> {
    fn new(source: &'a str, raw: &'a [RawLine], diagnostics: &'a mut Vec<Diagnostic>) -> Self {
        Self {
            source,
            raw,
            diagnostics,
            lines: Vec::with_capacity(raw.len()),
            variables: Vec::new(),
            directives: Vec::new(),
            pre_request_scripts: Vec::new(),
            request: None,
            request_end: 0,
            body: None,
            state: State::Pre,
        }
    }

    fn parse(mut self) -> Block {
        let mut title = None;
        let mut i = 0;
        let first = content(self.source, &self.raw[0]);
        if first.starts_with("###") {
            let start = self.raw[0].start + (first.len() - first.trim_start_matches('#').len());
            let span = trim_span(self.source, start, self.raw[0].end);
            if !span.is_empty() {
                title = Some(text(self.source, span));
            }
            self.push(0, LineKind::Separator);
            i = 1;
        }

        while i < self.raw.len() {
            i = self.line(i);
        }

        if let Some(request) = self.request.as_mut() {
            request.span.end = self.request_end;
            request.pre_request_scripts = std::mem::take(&mut self.pre_request_scripts);
            if let Some((first, last)) = self.body.take() {
                request.body = Some(Body {
                    content: text(self.source, first.start..last.end),
                });
            }
        }

        let span = self.raw[0].start..self.raw[self.raw.len() - 1].next;
        Block {
            span,
            title,
            lines: self.lines,
            variables: self.variables,
            request: self.request,
        }
    }

    fn push(&mut self, i: usize, kind: LineKind) {
        let line = self.raw[i];
        self.lines.push(Line {
            span: line.start..line.next,
            kind,
        });
    }

    fn diagnose(&mut self, span: Span, message: &str) {
        self.diagnostics.push(Diagnostic {
            span,
            message: message.to_string(),
        });
    }

    /// Mark line `i` as part of the request.
    fn extend_request(&mut self, i: usize) {
        self.request_end = self.raw[i].next;
    }

    /// Handle line `i` and return the index of the next unhandled line.
    fn line(&mut self, i: usize) -> usize {
        let raw = self.raw[i];
        let line = content(self.source, &raw);
        let blank = line.trim_matches(is_space).is_empty();

        match self.state {
            State::Skip => {
                self.push(
                    i,
                    if blank {
                        LineKind::Blank
                    } else {
                        LineKind::Unknown
                    },
                );
                i + 1
            }
            State::Pre => self.pre(i, line, blank),
            State::Target => {
                let continues = !blank
                    && (line.starts_with(is_space)
                        || line.trim_start_matches(is_space).starts_with(['?', '&']));
                if continues {
                    let mut span = trim_span(self.source, raw.start, raw.end);
                    let source = self.source;
                    if let Some(request) = self.request.as_mut() {
                        // The version may sit at the end of the last target line.
                        if request.version.is_none()
                            && let Some(space) = source[span.clone()].rfind(is_space)
                            && is_http_version(&source[span.start + space + 1..span.end])
                        {
                            request.version = Some(text(source, span.start + space + 1..span.end));
                            span = trim_span(source, span.start, span.start + space);
                        }
                        request.target_continuations.push(text(source, span));
                    }
                    self.push(i, LineKind::UrlContinuation);
                    self.extend_request(i);
                    return i + 1;
                }
                self.state = State::Headers;
                self.line(i)
            }
            State::Headers => self.headers(i, line, blank),
            State::Body => {
                if trailer_start(line).is_some() {
                    self.state = State::Trailer;
                    return self.line(i);
                }
                self.push(i, LineKind::Body);
                if !blank {
                    let span = trim_span(self.source, raw.start, raw.end);
                    self.body = Some(match self.body.take() {
                        Some((first, _)) => (first, span),
                        None => (span.clone(), span),
                    });
                    self.extend_request(i);
                }
                i + 1
            }
            State::Trailer => self.trailer(i, line, blank),
        }
    }

    fn pre(&mut self, i: usize, line: &str, blank: bool) -> usize {
        let raw = self.raw[i];
        if blank {
            self.push(i, LineKind::Blank);
            return i + 1;
        }
        if is_comment(line) {
            match self.directive(raw) {
                Some(directive) => {
                    self.directives.push(directive);
                    self.push(i, LineKind::Directive);
                }
                None => self.push(i, LineKind::Comment),
            }
            return i + 1;
        }
        let trimmed = line.trim_start_matches(is_space);
        if trimmed.starts_with('@') {
            match self.variable(raw) {
                Some(variable) => {
                    self.variables.push(variable);
                    self.push(i, LineKind::Variable);
                }
                None => {
                    self.push(i, LineKind::Unknown);
                    self.diagnose(raw.start..raw.end, "Expected a variable like @name = value");
                }
            }
            return i + 1;
        }
        if let Some(rest) = trimmed.strip_prefix('<')
            && rest.starts_with(is_space)
        {
            let marker = raw.start + (line.len() - rest.len());
            let (script, next) = self.script(i, marker, LineKind::PreRequestScript);
            self.pre_request_scripts.push(script);
            return next;
        }
        if let Some(rest) = trimmed.strip_prefix("HTTP/")
            && rest.starts_with(|c: char| c.is_ascii_digit())
        {
            self.diagnose(
                raw.start..raw.end,
                "This looks like a response rather than a request",
            );
            self.state = State::Skip;
            return self.line(i);
        }

        self.request = Some(self.request_line(raw));
        self.push(i, LineKind::RequestLine);
        self.extend_request(i);
        self.state = State::Target;
        i + 1
    }

    fn headers(&mut self, i: usize, line: &str, blank: bool) -> usize {
        let raw = self.raw[i];
        if blank {
            self.push(i, LineKind::Blank);
            self.state = State::Body;
            return i + 1;
        }
        if trailer_start(line).is_some() {
            self.state = State::Trailer;
            return self.line(i);
        }
        let previous_is_header = matches!(
            self.lines.last().map(|line| line.kind),
            Some(LineKind::Header | LineKind::HeaderContinuation)
        );
        if line.starts_with(is_space) && previous_is_header {
            let span = trim_span(self.source, raw.start, raw.end);
            if let Some(header) = self.request.as_mut().and_then(|r| r.headers.last_mut()) {
                header.continuations.push(text(self.source, span));
                header.span.end = raw.end;
            }
            self.push(i, LineKind::HeaderContinuation);
            self.extend_request(i);
            return i + 1;
        }
        if is_comment(line) {
            self.push(i, LineKind::Comment);
            return i + 1;
        }
        if let Some(colon) = line.find(':') {
            let name = &line[..colon];
            if is_header_name(name) {
                let value = trim_span(self.source, raw.start + colon + 1, raw.end);
                let header = Header {
                    span: raw.start..raw.end,
                    name: text(self.source, raw.start..raw.start + colon),
                    value: text(self.source, value),
                    continuations: Vec::new(),
                };
                if let Some(request) = self.request.as_mut() {
                    request.headers.push(header);
                }
                self.push(i, LineKind::Header);
                self.extend_request(i);
                return i + 1;
            }
        }
        self.diagnose(
            raw.start..raw.end,
            "Expected a header, or a blank line before the body",
        );
        self.state = State::Body;
        self.line(i)
    }

    fn trailer(&mut self, i: usize, line: &str, blank: bool) -> usize {
        let raw = self.raw[i];
        if blank {
            self.push(i, LineKind::Blank);
            return i + 1;
        }
        if is_comment(line) {
            self.push(i, LineKind::Comment);
            return i + 1;
        }
        let Some((start, marker_len)) = trailer_start(line) else {
            self.push(i, LineKind::Unknown);
            self.diagnose(
                raw.start..raw.end,
                "Expected a response handler, a >> redirect or a <> reference",
            );
            return i + 1;
        };
        let after_marker = raw.start + marker_len;
        match start {
            TrailerStart::Handler => {
                let (script, next) = self.script(i, after_marker, LineKind::ResponseHandler);
                if let Some(request) = self.request.as_mut() {
                    request.response_handlers.push(script);
                }
                self.extend_request(next - 1);
                next
            }
            TrailerStart::Ref => {
                let path = text(self.source, trim_span(self.source, after_marker, raw.end));
                if let Some(request) = self.request.as_mut() {
                    request.response_refs.push(path);
                }
                self.push(i, LineKind::ResponseRef);
                self.extend_request(i);
                i + 1
            }
            TrailerStart::Redirect { force } => {
                let path = text(self.source, trim_span(self.source, after_marker, raw.end));
                if let Some(request) = self.request.as_mut() {
                    request.response_redirects.push(ResponseRedirect {
                        span: raw.start..raw.end,
                        force,
                        path,
                    });
                }
                self.push(i, LineKind::ResponseRedirect);
                self.extend_request(i);
                i + 1
            }
        }
    }

    /// A script starting on line `i`, after its `<` or `>` marker at byte
    /// `from`: an inline `{% ... %}` that may span lines, or a file path.
    /// Returns the script and the index of the line after it.
    fn script(&mut self, i: usize, from: usize, kind: LineKind) -> (Script, usize) {
        let raw = self.raw[i];
        let rest = &self.source[from..raw.end];
        let Some(open) = rest.find("{%") else {
            let path = text(self.source, trim_span(self.source, from, raw.end));
            self.push(i, kind);
            let span = raw.start..raw.end;
            return (Script::File { span, path }, i + 1);
        };

        let code_start = from + open + 2;
        self.push(i, kind);
        let mut j = i;
        let mut search_from = code_start;
        loop {
            let line = self.raw[j];
            if let Some(close) = self.source[search_from..line.end].find("%}") {
                let code_end = search_from + close;
                let script = Script::Inline {
                    span: raw.start..line.end,
                    code: text(self.source, code_start..code_end),
                };
                return (script, j + 1);
            }
            if j + 1 == self.raw.len() {
                let end = line.end;
                self.diagnose(raw.start..end, "This script is missing its closing %}");
                let script = Script::Inline {
                    span: raw.start..end,
                    code: text(self.source, code_start..end),
                };
                return (script, j + 1);
            }
            j += 1;
            search_from = self.raw[j].start;
            self.push(j, LineKind::Script);
        }
    }

    /// `# @name value`, `// @name=value` or `# @flag`.
    fn directive(&self, raw: RawLine) -> Option<Directive> {
        let line = content(self.source, &raw);
        let body = line.trim_start_matches(is_space);
        let body = if body.starts_with('#') {
            body.trim_start_matches('#')
        } else {
            body.trim_start_matches('/')
        };
        let body = body.trim_start_matches(is_space);
        let after_at = body.strip_prefix('@')?;
        let name_len = after_at
            .find(|c: char| !(c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.')))
            .unwrap_or(after_at.len());
        if name_len == 0 {
            return None;
        }
        let rest = &after_at[name_len..];
        if !(rest.is_empty() || rest.starts_with(is_space) || rest.starts_with('=')) {
            return None;
        }
        let name_start = raw.end - after_at.len();
        let mut value_start = name_start + name_len;
        let after_name = rest.trim_start_matches(is_space);
        if let Some(after_eq) = after_name.strip_prefix('=') {
            value_start = raw.end - after_eq.len();
        }
        let value = trim_span(self.source, value_start, raw.end);
        Some(Directive {
            span: raw.start..raw.end,
            name: text(self.source, name_start..name_start + name_len),
            value: (!value.is_empty()).then(|| text(self.source, value)),
        })
    }

    /// `@name = value`.
    fn variable(&self, raw: RawLine) -> Option<Variable> {
        let line = content(self.source, &raw);
        let after_at = line.trim_start_matches(is_space).strip_prefix('@')?;
        let name_len = after_at
            .find(|c: char| is_space(c) || c == '=')
            .unwrap_or(after_at.len());
        if name_len == 0 {
            return None;
        }
        let name_start = raw.end - after_at.len();
        let after_eq = after_at[name_len..]
            .trim_start_matches(is_space)
            .strip_prefix('=')?;
        let value = trim_span(self.source, raw.end - after_eq.len(), raw.end);
        Some(Variable {
            span: raw.start..raw.end,
            name: text(self.source, name_start..name_start + name_len),
            value: text(self.source, value),
        })
    }

    /// `[METHOD] target [HTTP/version]`.
    fn request_line(&self, raw: RawLine) -> Request {
        let span = trim_span(self.source, raw.start, raw.end);
        let line = &self.source[span.clone()];

        let mut target = span.clone();
        let mut method = None;
        if let Some(space) = line.find(is_space) {
            let token = &line[..space];
            if is_method(token) {
                method = Some(text(self.source, span.start..span.start + space));
                target = trim_span(self.source, span.start + space, span.end);
            }
        }

        let mut version = None;
        let target_text = &self.source[target.clone()];
        if let Some(space) = target_text.rfind(is_space) {
            let token = &target_text[space + 1..];
            if is_http_version(token) {
                version = Some(text(self.source, target.start + space + 1..target.end));
                target = trim_span(self.source, target.start, target.start + space);
            }
        }

        Request {
            span: raw.start..raw.next,
            directives: self.directives.clone(),
            method,
            target: text(self.source, target),
            target_continuations: Vec::new(),
            version,
            headers: Vec::new(),
            body: None,
            pre_request_scripts: Vec::new(),
            response_handlers: Vec::new(),
            response_refs: Vec::new(),
            response_redirects: Vec::new(),
        }
    }
}
