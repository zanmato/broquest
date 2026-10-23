//! Lossless parser for `.http` and `.rest` request files, as used by the
//! JetBrains HTTP Client and the VS Code REST Client.
//!
//! Every byte of the source belongs to exactly one [`Line`] of exactly one
//! [`Block`], and every parsed piece records the byte range it came from.
//! Edits can then rewrite only the bytes they change, keeping comments,
//! blank lines and layout as the user wrote them.
//!
//! The parser never fails. Text it cannot place becomes a
//! [`LineKind::Unknown`] line with a [`Diagnostic`], and parsing goes on.

use std::ops::Range;

mod parser;

pub use parser::parse;

/// A byte range into the parsed source.
pub type Span = Range<usize>;

/// A piece of the source and the bytes it came from. `text` always equals
/// the source sliced by `span`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    pub span: Span,
    pub text: String,
}

impl Text {
    pub fn as_str(&self) -> &str {
        &self.text
    }
}

/// A parsed file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HttpFile {
    pub source: String,
    /// The file split at its `###` separators. Together the blocks cover the
    /// whole source, in order, without gaps.
    pub blocks: Vec<Block>,
    pub diagnostics: Vec<Diagnostic>,
}

impl HttpFile {
    /// The blocks that hold a request.
    pub fn requests(&self) -> impl Iterator<Item = &Request> {
        self.blocks
            .iter()
            .filter_map(|block| block.request.as_ref())
    }

    /// Every `@name = value` variable in the file, in order.
    pub fn variables(&self) -> impl Iterator<Item = &Variable> {
        self.blocks.iter().flat_map(|block| block.variables.iter())
    }
}

/// Something in the file that could not be understood.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    pub span: Span,
    pub message: String,
}

/// The text from one `###` separator up to the next, or the text before the
/// first separator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub span: Span,
    /// The text after `###` on the separator line, when there is any.
    pub title: Option<Text>,
    /// Every line of the block, in order, covering its whole span.
    pub lines: Vec<Line>,
    /// `@name = value` lines in this block.
    pub variables: Vec<Variable>,
    /// The request, when the block has a request line.
    pub request: Option<Request>,
}

/// One line of the source, including its line terminator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Line {
    pub span: Span,
    pub kind: LineKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    /// `###`, with an optional title.
    Separator,
    Blank,
    /// `# ...` or `// ...` outside a body.
    Comment,
    /// `# @name value`, `// @no-log` and the like.
    Directive,
    /// `@name = value`.
    Variable,
    /// `< {% ... %}` or `< script.js` before the request line.
    PreRequestScript,
    RequestLine,
    /// An indented line continuing the request target, or a VS Code style
    /// `?query` / `&query` line.
    UrlContinuation,
    Header,
    /// An indented line continuing the previous header's value.
    HeaderContinuation,
    /// A line of the message body, blank lines and comments within it
    /// included.
    Body,
    /// `> {% ... %}` or `> handler.js` after the body.
    ResponseHandler,
    /// `<> previous-response.json`.
    ResponseRef,
    /// `>> file` or `>>! file`.
    ResponseRedirect,
    /// A line inside a multi-line `{% ... %}` script, after its first line.
    Script,
    /// A line that could not be placed. A diagnostic explains why.
    Unknown,
}

/// `@name = value`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Variable {
    /// The whole line, without its terminator.
    pub span: Span,
    pub name: Text,
    pub value: Text,
}

/// `# @name value` or `# @name=value`, before the request line.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Directive {
    /// The whole line, without its terminator.
    pub span: Span,
    pub name: Text,
    pub value: Option<Text>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Request {
    /// From the start of the request line to the end of its last line,
    /// terminator included. Trailing blank lines and comments are left out.
    pub span: Span,
    pub directives: Vec<Directive>,
    /// `None` when the request line has no method, which means GET.
    pub method: Option<Text>,
    /// The request target on the request line.
    pub target: Text,
    /// Further parts of the target on the lines after it, trimmed.
    pub target_continuations: Vec<Text>,
    pub version: Option<Text>,
    pub headers: Vec<Header>,
    pub body: Option<Body>,
    pub pre_request_scripts: Vec<Script>,
    pub response_handlers: Vec<Script>,
    /// Paths of `<>` references to earlier responses.
    pub response_refs: Vec<Text>,
    pub response_redirects: Vec<ResponseRedirect>,
}

impl Request {
    /// The `@name` directive's value.
    pub fn name(&self) -> Option<&str> {
        self.directive("name")
            .and_then(|directive| directive.value.as_ref())
            .map(Text::as_str)
    }

    /// The first directive called `name`, matched without regard to case.
    pub fn directive(&self, name: &str) -> Option<&Directive> {
        self.directives
            .iter()
            .find(|directive| directive.name.text.eq_ignore_ascii_case(name))
    }

    /// The method, or GET when the request line has none.
    pub fn method(&self) -> &str {
        self.method.as_ref().map_or("GET", Text::as_str)
    }

    /// The request target with its continuation lines joined on.
    pub fn url(&self) -> String {
        let mut url = self.target.text.clone();
        for part in &self.target_continuations {
            url.push_str(&part.text);
        }
        url
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Header {
    /// The header's lines, continuations included, without the final
    /// terminator.
    pub span: Span,
    pub name: Text,
    /// The value on the header's first line, trimmed.
    pub value: Text,
    /// Further parts of the value on indented lines, trimmed.
    pub continuations: Vec<Text>,
}

impl Header {
    /// The value with any continuation lines joined on with spaces.
    pub fn full_value(&self) -> String {
        let mut value = self.value.text.clone();
        for part in &self.continuations {
            if !value.is_empty() {
                value.push(' ');
            }
            value.push_str(&part.text);
        }
        value
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Body {
    /// The body with surrounding whitespace trimmed, which is what is sent.
    pub content: Text,
}

impl Body {
    /// The path when the whole body is a single `< path` file reference.
    pub fn file_ref(&self) -> Option<&str> {
        let text = self.content.text.as_str();
        if text.contains(['\n', '\r']) {
            return None;
        }
        let path = text.strip_prefix('<')?;
        path.starts_with([' ', '\t']).then(|| path.trim())
    }
}

/// A pre-request script or response handler.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Script {
    /// `{% ... %}`. `code` is what is between the markers.
    Inline { span: Span, code: Text },
    /// A path to a script file.
    File { span: Span, path: Text },
}

/// `>> file` writes the response to a file. `>>! file` overwrites it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResponseRedirect {
    pub span: Span,
    pub force: bool,
    pub path: Text,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(source: &str) -> Request {
        parse(source).requests().next().cloned().expect("a request")
    }

    #[test]
    fn request_helpers() {
        let request = request(
            "# @Name=Create label\nhttps://example.com/labels\n  ?size=a6\nAccept: text/plain,\n  application/pdf\n",
        );
        assert_eq!(request.name(), Some("Create label"));
        assert_eq!(request.method(), "GET");
        assert_eq!(request.url(), "https://example.com/labels?size=a6");
        assert_eq!(
            request.headers[0].full_value(),
            "text/plain, application/pdf"
        );
    }

    #[test]
    fn body_file_refs() {
        let body = |source: &str| request(source).body.expect("a body");
        assert_eq!(
            body("POST /a\n\n< ./in.json\n").file_ref(),
            Some("./in.json")
        );
        assert_eq!(body("POST /a\n\n<./in.json\n").file_ref(), None);
        assert_eq!(body("POST /a\n\n< a.txt\n< b.txt\n").file_ref(), None);
    }

    #[test]
    fn spans_point_into_crlf_sources() {
        let source = "### One\r\nPOST https://example.com/a HTTP/1.1\r\nX-Id: 7\r\n\r\nbody\r\n";
        let file = parse(source);
        let request = file.requests().next().unwrap();
        assert_eq!(&source[request.headers[0].value.span.clone()], "7");
        assert_eq!(
            &source[request.body.as_ref().unwrap().content.span.clone()],
            "body"
        );
        assert_eq!(
            &source[request.version.as_ref().unwrap().span.clone()],
            "HTTP/1.1"
        );
        assert_eq!(request.span.end, source.len());
    }

    #[test]
    fn variables_are_collected_across_blocks() {
        let file = parse("@a = 1\n###\n@b=2\nGET /x\n");
        let names: Vec<&str> = file.variables().map(|v| v.name.as_str()).collect();
        assert_eq!(names, ["a", "b"]);
    }
}
