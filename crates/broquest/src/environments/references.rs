//! References to an earlier request's exchange, as in the VS Code REST
//! Client: `{{login.response.body.$.token}}`,
//! `{{login.response.headers.Location}}`, `{{login.response.body.*}}`, and
//! the same with `request` for what was sent.
//!
//! A reference resolves against the last response of the named request in
//! this session. Until that request has been sent, it keeps its placeholder.

use std::collections::HashMap;

use jsonpath_rust::JsonPath;

use crate::domain::ResponseData;

#[derive(Debug, PartialEq, Eq)]
enum Side {
    Request,
    Response,
}

#[derive(Debug, PartialEq, Eq)]
enum Field<'a> {
    /// `*` or a JSONPath such as `$.token`.
    Body(&'a str),
    Header(&'a str),
}

#[derive(Debug, PartialEq, Eq)]
struct Reference<'a> {
    request: &'a str,
    side: Side,
    field: Field<'a>,
}

fn parse(name: &str) -> Option<Reference<'_>> {
    let request_at = name.find(".request.");
    let response_at = name.find(".response.");
    let (at, side, marker) = match (request_at, response_at) {
        (Some(r), Some(s)) if r < s => (r, Side::Request, ".request."),
        (_, Some(s)) => (s, Side::Response, ".response."),
        (Some(r), None) => (r, Side::Request, ".request."),
        (None, None) => return None,
    };
    let request = &name[..at];
    let rest = &name[at + marker.len()..];
    if request.is_empty() {
        return None;
    }
    let field = if let Some(path) = rest.strip_prefix("body.") {
        Field::Body(path)
    } else if rest == "body" {
        Field::Body("*")
    } else {
        Field::Header(rest.strip_prefix("headers.")?)
    };
    Some(Reference {
        request,
        side,
        field,
    })
}

/// The `{{...}}` names in `text` that reference a request's exchange.
pub fn references_in(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find("{{") {
        let after = &rest[start + 2..];
        let Some(end) = after.find("}}") else {
            break;
        };
        let name = after[..end].trim();
        if parse(name).is_some() {
            found.push(name.to_string());
        }
        rest = &after[end + 2..];
    }
    found
}

/// The value of the reference `name`, from the last exchanges keyed by
/// request name. `None` when it is not a reference, the request has not
/// been sent, or the field is missing.
pub fn evaluate(name: &str, exchanges: &HashMap<String, ResponseData>) -> Option<String> {
    let reference = parse(name)?;
    let exchange = exchanges.get(reference.request)?;
    let (body, headers) = match reference.side {
        Side::Response => (&exchange.body, &exchange.headers),
        Side::Request => (&exchange.request_body, &exchange.request_headers),
    };
    match reference.field {
        Field::Header(header) => headers
            .iter()
            .find(|h| h.key.eq_ignore_ascii_case(header))
            .map(|h| h.value.clone()),
        Field::Body("*") => Some(body.clone()),
        Field::Body(path) if path.starts_with('$') => {
            let json: serde_json::Value = serde_json::from_str(body).ok()?;
            let matches = json.query(path).ok()?;
            match matches.as_slice() {
                [] => None,
                [serde_json::Value::String(text)] => Some(text.clone()),
                [value] => Some(value.to_string()),
                many => Some(
                    serde_json::Value::Array(many.iter().map(|v| (*v).clone()).collect())
                        .to_string(),
                ),
            }
        }
        // XPath for XML bodies is not supported.
        Field::Body(_) => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::KeyValuePair;

    fn header(key: &str, value: &str) -> KeyValuePair {
        KeyValuePair {
            key: key.to_string(),
            value: value.to_string(),
            enabled: true,
        }
    }

    fn exchanges() -> HashMap<String, ResponseData> {
        let login = ResponseData {
            body: r#"{"token": "abc", "user": {"id": 7}, "roles": ["a", "b"]}"#.to_string(),
            headers: vec![header("Location", "/users/7")],
            request_body: r#"{"name": "kestrel"}"#.to_string(),
            request_headers: vec![header("X-Client", "broquest")],
            ..Default::default()
        };
        HashMap::from([("login".to_string(), login)])
    }

    #[test]
    fn evaluates_body_and_header_references() {
        let exchanges = exchanges();
        let eval = |name: &str| evaluate(name, &exchanges);
        assert_eq!(eval("login.response.body.$.token").as_deref(), Some("abc"));
        assert_eq!(eval("login.response.body.$.user.id").as_deref(), Some("7"));
        assert_eq!(
            eval("login.response.body.$.user").as_deref(),
            Some(r#"{"id":7}"#)
        );
        assert_eq!(
            eval("login.response.body.$.roles[*]").as_deref(),
            Some(r#"["a","b"]"#)
        );
        assert!(eval("login.response.body.*").unwrap().contains("abc"));
        assert_eq!(
            eval("login.response.headers.location").as_deref(),
            Some("/users/7")
        );
        assert_eq!(
            eval("login.request.body.$.name").as_deref(),
            Some("kestrel")
        );
        assert_eq!(
            eval("login.request.headers.X-Client").as_deref(),
            Some("broquest")
        );
    }

    #[test]
    fn unknown_or_unsupported_references_resolve_to_nothing() {
        let exchanges = exchanges();
        let eval = |name: &str| evaluate(name, &exchanges);
        assert_eq!(eval("signup.response.body.$.token"), None);
        assert_eq!(eval("login.response.body.$.missing"), None);
        assert_eq!(eval("login.response.headers.X-Nope"), None);
        assert_eq!(eval("login.response.body.//token"), None);
        assert_eq!(eval("login.response.status"), None);
        assert_eq!(eval("host"), None);
    }

    #[test]
    fn finds_references_in_text() {
        let text = "Bearer {{login.response.body.$.token}} for {{ user }} at {{ login.response.headers.Location }}";
        assert_eq!(
            references_in(text),
            [
                "login.response.body.$.token",
                "login.response.headers.Location"
            ]
        );
    }
}
