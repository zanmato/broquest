use crate::domain::{HttpMethod, KeyValuePair, RequestData};

/// Parsed result from a cURL command
pub struct ParsedCurl {
    pub method: HttpMethod,
    pub url: String,
    pub headers: Vec<KeyValuePair>,
    pub body: Option<String>,
    pub basic_auth: Option<(String, String)>,
    pub is_multipart: bool,
}

/// Attempt to parse a string as a cURL command.
/// Returns `None` if the input does not look like a cURL command.
pub fn parse_curl(input: &str) -> Option<ParsedCurl> {
    let trimmed = input.trim();
    if !trimmed.starts_with("curl") {
        return None;
    }

    let tokens = tokenize_curl(trimmed)?;
    if tokens.is_empty() {
        return None;
    }

    let mut method: Option<HttpMethod> = None;
    let mut url: Option<String> = None;
    let mut headers: Vec<KeyValuePair> = Vec::new();
    let mut body: Option<String> = None;
    let mut basic_auth: Option<(String, String)> = None;
    let mut is_data_get = false;
    let mut is_multipart = false;
    let mut tokens = tokens.into_iter().peekable();

    // Skip "curl" itself
    tokens.next();

    while let Some(token) = tokens.next() {
        match token.as_str() {
            "-X" | "--request" => {
                if let Some(value) = tokens.next() {
                    method = Some(parse_http_method(&value));
                }
            }
            "-H" | "--header" => {
                if let Some(value) = tokens.next()
                    && let Some((key, val)) = parse_header(&value)
                {
                    headers.push(KeyValuePair {
                        key,
                        value: val,
                        enabled: true,
                    });
                }
            }
            "-d" | "--data" | "--data-raw" | "--data-binary" => {
                if let Some(value) = tokens.next() {
                    body = Some(value);
                    if method.is_none() {
                        method = Some(HttpMethod::Post);
                    }
                }
            }
            "-F" | "--form" => {
                if let Some(value) = tokens.next() {
                    body = Some(value);
                    is_multipart = true;
                    if method.is_none() {
                        method = Some(HttpMethod::Post);
                    }
                    if !headers
                        .iter()
                        .any(|h| h.key.to_lowercase() == "content-type")
                    {
                        headers.push(KeyValuePair {
                            key: "Content-Type".to_string(),
                            value: "multipart/form-data".to_string(),
                            enabled: true,
                        });
                    }
                }
            }
            "-u" | "--user" => {
                if let Some(value) = tokens.next() {
                    if let Some((user, pass)) = value.split_once(':') {
                        basic_auth = Some((user.to_string(), pass.to_string()));
                    } else {
                        basic_auth = Some((value, String::new()));
                    }
                }
            }
            "--url" => {
                if let Some(value) = tokens.next() {
                    url = Some(value);
                }
            }
            "-G" | "--get" => {
                is_data_get = true;
            }
            "-k" | "--insecure" | "--compressed" | "-s" | "--silent" | "-S" | "--show-error"
            | "-v" | "--verbose" | "-L" | "--location" | "-i" | "--include" | "-I" | "--head" => {}
            _ => {
                // Positional argument: treat as URL if we haven't seen one
                if !token.starts_with('-') && url.is_none() {
                    url = Some(token);
                }
            }
        }
    }

    let url = url?;

    let method = if is_data_get {
        HttpMethod::Get
    } else {
        method.unwrap_or(HttpMethod::Get)
    };

    Some(ParsedCurl {
        method,
        url,
        headers,
        body,
        basic_auth,
        is_multipart,
    })
}

/// Serialize a RequestData into a cURL command string
pub fn to_curl(request_data: &RequestData) -> String {
    let mut parts = Vec::new();
    parts.push("curl".to_string());
    parts.push(format!("-X {}", request_data.method.as_str()));

    // Add headers (only enabled ones)
    for header in &request_data.headers {
        if header.enabled && !header.key.is_empty() {
            parts.push(format!(
                "-H '{}: {}'",
                escape_single_quotes(&header.key),
                escape_single_quotes(&header.value)
            ));
        }
    }

    // Add body
    if !request_data.body.is_empty()
        && matches!(
            request_data.method,
            HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch
        )
    {
        parts.push(format!("-d '{}'", escape_single_quotes(&request_data.body)));
    }

    // Add URL last
    parts.push(format!("'{}'", escape_single_quotes(&request_data.url)));

    parts.join(" \\\n  ")
}

fn parse_http_method(s: &str) -> HttpMethod {
    match s.to_uppercase().as_str() {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "DELETE" => HttpMethod::Delete,
        "PATCH" => HttpMethod::Patch,
        "HEAD" => HttpMethod::Head,
        "OPTIONS" => HttpMethod::Options,
        _ => HttpMethod::Get,
    }
}

fn parse_header(value: &str) -> Option<(String, String)> {
    let (key, val) = value.split_once(':')?;
    Some((key.trim().to_string(), val.trim().to_string()))
}

fn escape_single_quotes(s: &str) -> String {
    s.replace('\'', "'\\''")
}

/// Tokenize a cURL command, handling single quotes, double quotes,
/// and backslash-escaped newlines.
fn tokenize_curl(input: &str) -> Option<Vec<String>> {
    let input = input.replace("\\\n", " ").replace("\\\r\n", " ");
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
                chars.next();
            }
            '\'' => {
                chars.next();
                while let Some(&c) = chars.peek() {
                    if c == '\'' {
                        chars.next();
                        break;
                    }
                    current.push(chars.next()?);
                }
            }
            '"' => {
                chars.next();
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next();
                        break;
                    }
                    if c == '\\' {
                        chars.next();
                        let escaped = chars.next()?;
                        current.push(match escaped {
                            'n' => '\n',
                            't' => '\t',
                            'r' => '\r',
                            _ => escaped,
                        });
                    } else {
                        current.push(chars.next()?);
                    }
                }
            }
            '$' => {
                // Variable reference: capture until whitespace
                current.push(chars.next()?);
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric()
                        || c == '_'
                        || c == '{'
                        || c == '}'
                        || c == '('
                        || c == ')'
                    {
                        current.push(chars.next()?);
                    } else {
                        break;
                    }
                }
            }
            _ => {
                current.push(chars.next()?);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    Some(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_get() {
        let result = parse_curl("curl https://example.com").unwrap();
        assert_eq!(result.method, HttpMethod::Get);
        assert_eq!(result.url, "https://example.com");
        assert!(result.headers.is_empty());
        assert!(result.body.is_none());
    }

    #[test]
    fn test_parse_post_with_headers_and_body() {
        let result = parse_curl(
            r#"curl -X POST https://api.example.com/users \
               -H 'Content-Type: application/json' \
               -H 'Authorization: Bearer token123' \
               -d '{"name": "John"}'"#,
        )
        .unwrap();
        assert_eq!(result.method, HttpMethod::Post);
        assert_eq!(result.url, "https://api.example.com/users");
        assert_eq!(result.headers.len(), 2);
        assert_eq!(result.headers[0].key, "Content-Type");
        assert_eq!(result.headers[0].value, "application/json");
        assert_eq!(result.body.unwrap(), r#"{"name": "John"}"#);
    }

    #[test]
    fn test_parse_basic_auth() {
        let result = parse_curl("curl -u user:pass https://example.com").unwrap();
        assert_eq!(
            result.basic_auth,
            Some(("user".to_string(), "pass".to_string()))
        );
    }

    #[test]
    fn test_parse_patch() {
        let result = parse_curl("curl -X PATCH https://example.com/resource -d 'updated'").unwrap();
        assert_eq!(result.method, HttpMethod::Patch);
        assert_eq!(result.body.unwrap(), "updated");
    }

    #[test]
    fn test_parse_with_double_quotes() {
        let result = parse_curl(
            r#"curl -X POST "https://example.com" -H "Content-Type: application/json" -d "{\"key\": \"value\""}"#,
        ).unwrap();
        assert_eq!(result.method, HttpMethod::Post);
        assert_eq!(result.url, "https://example.com");
        assert_eq!(result.body.unwrap(), r#"{"key": "value"}"#);
    }

    #[test]
    fn test_parse_implicit_post_with_data() {
        let result = parse_curl("curl -d 'hello' https://example.com").unwrap();
        assert_eq!(result.method, HttpMethod::Post);
        assert_eq!(result.body.unwrap(), "hello");
    }

    #[test]
    fn test_parse_get_with_data_flag() {
        let result = parse_curl("curl -G -d 'q=test' https://example.com/search").unwrap();
        assert_eq!(result.method, HttpMethod::Get);
        assert_eq!(result.body.unwrap(), "q=test");
    }

    #[test]
    fn test_parse_ignores_unknown_flags() {
        let result = parse_curl("curl -s -k -L https://example.com").unwrap();
        assert_eq!(result.method, HttpMethod::Get);
        assert_eq!(result.url, "https://example.com");
    }

    #[test]
    fn test_not_curl_returns_none() {
        assert!(parse_curl("https://example.com").is_none());
        assert!(parse_curl("").is_none());
    }

    #[test]
    fn test_to_curl_simple() {
        let request = RequestData {
            name: "Test".to_string(),
            method: HttpMethod::Post,
            url: "https://api.example.com/users".to_string(),
            headers: vec![KeyValuePair {
                key: "Content-Type".to_string(),
                value: "application/json".to_string(),
                enabled: true,
            }],
            body: r#"{"name": "John"}"#.to_string(),
            ..Default::default()
        };
        let curl = to_curl(&request);
        assert!(curl.contains("curl"));
        assert!(curl.contains("-X POST"));
        assert!(curl.contains("-H 'Content-Type: application/json'"));
        assert!(curl.contains("-d '{\"name\": \"John\"}'"));
        assert!(curl.contains("'https://api.example.com/users'"));
    }

    #[test]
    fn test_to_curl_skips_disabled_headers() {
        let request = RequestData {
            name: "Test".to_string(),
            method: HttpMethod::Get,
            url: "https://example.com".to_string(),
            headers: vec![
                KeyValuePair {
                    key: "Accept".to_string(),
                    value: "application/json".to_string(),
                    enabled: true,
                },
                KeyValuePair {
                    key: "X-Disabled".to_string(),
                    value: "true".to_string(),
                    enabled: false,
                },
            ],
            ..Default::default()
        };
        let curl = to_curl(&request);
        assert!(curl.contains("-H 'Accept: application/json'"));
        assert!(!curl.contains("X-Disabled"));
    }

    #[test]
    fn test_roundtrip() {
        let original = "curl -X PUT https://api.example.com/items/1 -H 'Content-Type: application/json' -d '{\"name\": \"test\"}'";
        let parsed = parse_curl(original).unwrap();
        let request = RequestData {
            name: String::new(),
            method: parsed.method,
            url: parsed.url,
            headers: parsed.headers,
            body: parsed.body.unwrap_or_default(),
            ..Default::default()
        };
        let curl_output = to_curl(&request);
        let reparsed = parse_curl(&curl_output).unwrap();
        assert_eq!(reparsed.method, HttpMethod::Put);
        assert_eq!(reparsed.url, "https://api.example.com/items/1");
    }
}
