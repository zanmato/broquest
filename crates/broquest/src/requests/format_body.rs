//! Prettify a JSON or XML request body.
//!
//! JSON is reindented token by token rather than parsed and printed again,
//! so only whitespace changes: keys keep their order, numbers keep how they
//! were written, and `{{variables}}` outside strings, which are not JSON,
//! stay where they are.

use crate::domain::ContentType;

/// Why a body was left as it was.
#[derive(Debug, PartialEq, Eq)]
pub enum FormatError {
    /// Not JSON or XML, so there is no way to format it.
    Unsupported,
    InvalidJson,
    InvalidXml,
}

impl FormatError {
    pub fn message(&self) -> &'static str {
        match self {
            Self::Unsupported => "Only JSON and XML bodies can be formatted.",
            Self::InvalidJson => "The body isn't valid JSON, so it was left as it is.",
            Self::InvalidXml => "The body isn't valid XML, so it was left as it is.",
        }
    }
}

/// The body formatted for its content type. A body of another type is
/// formatted when it looks like JSON or XML.
pub fn format_body(body: &str, content_type: Option<ContentType>) -> Result<String, FormatError> {
    let start = body.trim_start().chars().next();
    match (content_type, start) {
        (Some(ContentType::Json), _) | (_, Some('{' | '[')) => {
            format_json(body).ok_or(FormatError::InvalidJson)
        }
        (Some(ContentType::Xml), _) | (_, Some('<')) => {
            crate::http::format_xml(body).ok_or(FormatError::InvalidXml)
        }
        _ => Err(FormatError::Unsupported),
    }
}

/// A piece of a JSON body outside of whitespace.
#[derive(Debug, PartialEq)]
enum Token<'a> {
    Open(char),
    Close(char),
    Comma,
    Colon,
    /// A string, number, literal or `{{variable}}`, kept as written.
    Value(&'a str),
    /// A `{{variable}}` outside a string.
    Variable(&'a str),
}

/// Split a JSON body into tokens. `None` for an unterminated string or
/// variable, or a comment, which JSON has no place for.
fn tokenize(text: &str) -> Option<Vec<Token<'_>>> {
    let bytes = text.as_bytes();
    let mut tokens = Vec::new();
    let mut i = 0;
    while i < bytes.len() {
        let c = bytes[i];
        match c {
            b' ' | b'\t' | b'\n' | b'\r' => i += 1,
            b'{' if bytes.get(i + 1) == Some(&b'{') => {
                let end = text[i..].find("}}")? + i + 2;
                tokens.push(Token::Variable(&text[i..end]));
                i = end;
            }
            b'{' | b'[' => {
                tokens.push(Token::Open(c as char));
                i += 1;
            }
            b'}' | b']' => {
                tokens.push(Token::Close(c as char));
                i += 1;
            }
            b',' => {
                tokens.push(Token::Comma);
                i += 1;
            }
            b':' => {
                tokens.push(Token::Colon);
                i += 1;
            }
            b'"' => {
                let mut end = i + 1;
                loop {
                    match bytes.get(end)? {
                        b'\\' => end += 2,
                        b'"' => break,
                        _ => end += 1,
                    }
                }
                tokens.push(Token::Value(&text[i..=end]));
                i = end + 1;
            }
            b'/' => return None,
            _ => {
                let end = text[i..]
                    .find(|ch: char| {
                        ch.is_whitespace() || matches!(ch, '{' | '}' | '[' | ']' | ',' | ':' | '"')
                    })
                    .map_or(text.len(), |offset| i + offset);
                tokens.push(Token::Value(&text[i..end]));
                i = end;
            }
        }
    }
    Some(tokens)
}

/// Whether the tokens are valid JSON once each variable stands for a value.
/// A string stands in, as that is valid both as a value and as a key.
fn is_valid(tokens: &[Token]) -> bool {
    let probe: String = tokens
        .iter()
        .map(|token| match token {
            Token::Open(c) | Token::Close(c) => c.to_string(),
            Token::Comma => ",".to_string(),
            Token::Colon => ":".to_string(),
            Token::Value(text) => format!(" {text} "),
            Token::Variable(_) => "\"\"".to_string(),
        })
        .collect();
    serde_json::from_str::<serde::de::IgnoredAny>(&probe).is_ok()
}

/// The JSON body indented by two spaces, or `None` when it is not valid.
fn format_json(text: &str) -> Option<String> {
    let tokens = tokenize(text)?;
    if !is_valid(&tokens) {
        return None;
    }
    let mut out = String::with_capacity(text.len() * 2);
    let mut depth = 0usize;
    let newline = |out: &mut String, depth: usize| {
        out.push('\n');
        out.push_str(&"  ".repeat(depth));
    };
    let mut i = 0;
    while i < tokens.len() {
        match tokens[i] {
            Token::Open(c) => {
                // An empty object or array stays on one line.
                if let Some(Token::Close(close)) = tokens.get(i + 1) {
                    out.push(c);
                    out.push(*close);
                    i += 1;
                } else {
                    out.push(c);
                    depth += 1;
                    newline(&mut out, depth);
                }
            }
            Token::Close(c) => {
                depth = depth.saturating_sub(1);
                newline(&mut out, depth);
                out.push(c);
            }
            Token::Comma => {
                out.push(',');
                newline(&mut out, depth);
            }
            Token::Colon => out.push_str(": "),
            Token::Value(text) | Token::Variable(text) => out.push_str(text),
        }
        i += 1;
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn json_is_indented_keeping_order_and_numbers() {
        let body = r#"{"z":1.50,"a":[1,2,{"b":null}],"e":{},"f":[],"big":12345678901234567890}"#;
        assert_eq!(
            format_body(body, Some(ContentType::Json)).unwrap(),
            r#"{
  "z": 1.50,
  "a": [
    1,
    2,
    {
      "b": null
    }
  ],
  "e": {},
  "f": [],
  "big": 12345678901234567890
}"#
        );
    }

    #[test]
    fn variables_stay_as_written() {
        let body = r#"{"id":{{id}},"{{key}}":"x {{name}}, \"quoted\" {", "list": [{{a}}, {{b}}]}"#;
        assert_eq!(
            format_body(body, Some(ContentType::Json)).unwrap(),
            r#"{
  "id": {{id}},
  "{{key}}": "x {{name}}, \"quoted\" {",
  "list": [
    {{a}},
    {{b}}
  ]
}"#
        );
    }

    #[test]
    fn text_outside_ascii_is_kept() {
        let body = r#"{"namn":"Åsa é \u00e9","emoji":"🦀"}"#;
        let expected = "{\n  \"namn\": \"Åsa é \\u00e9\",\n  \"emoji\": \"🦀\"\n}";
        assert_eq!(format_body(body, None).unwrap(), expected);
    }

    #[test]
    fn formatting_twice_changes_nothing() {
        let once = format_body(r#"[{"a":1},{"b":[true,false]}]"#, None).unwrap();
        assert_eq!(format_body(&once, None).unwrap(), once);
    }

    #[test]
    fn invalid_json_is_refused() {
        for body in [
            r#"{"a":1"#,
            r#"{"a" 1}"#,
            r#"{"a":"unterminated}"#,
            r#"{"a":1} // comment"#,
            r#"{"a":{{id}"#,
            r#"{"a":1,}"#,
        ] {
            assert_eq!(
                format_body(body, Some(ContentType::Json)),
                Err(FormatError::InvalidJson),
                "{body}"
            );
        }
    }

    #[test]
    fn xml_is_indented() {
        let body = r#"<?xml version="1.0"?><a><b id="{{id}}">{{name}}</b><c/></a>"#;
        assert_eq!(
            format_body(body, Some(ContentType::Xml)).unwrap(),
            "<?xml version=\"1.0\"?>\n<a>\n  <b id=\"{{id}}\">{{name}}</b>\n  <c/>\n</a>"
        );
        assert_eq!(
            format_body("<a><b></a>", Some(ContentType::Xml)),
            Err(FormatError::InvalidXml)
        );
    }

    #[test]
    fn other_bodies_are_formatted_when_they_look_like_json_or_xml() {
        assert_eq!(
            format_body("[1,2]", Some(ContentType::Text)).unwrap(),
            "[\n  1,\n  2\n]"
        );
        assert!(format_body("<a/>", Some(ContentType::Text)).is_ok());
        assert_eq!(
            format_body("name=value", Some(ContentType::Text)),
            Err(FormatError::Unsupported)
        );
    }
}
