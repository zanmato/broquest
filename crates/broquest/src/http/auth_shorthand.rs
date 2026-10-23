//! The VS Code REST Client's `Authorization` shorthands, for `.http`
//! requests: `Basic user:password` and `Basic user password` are encoded,
//! and `Digest user password` becomes digest authentication.

use base64::{Engine as _, engine::general_purpose::STANDARD};

use crate::domain::{AuthType, DigestAuth, RequestData};

/// Rewrite a shorthand `Authorization` header. Run after variables are
/// resolved, so credentials can come from variables.
pub fn apply(request: &mut RequestData) {
    let Some(index) = request
        .headers
        .iter()
        .position(|h| h.enabled && h.key.eq_ignore_ascii_case("authorization"))
    else {
        return;
    };
    let value = request.headers[index].value.trim().to_string();
    let Some((scheme, rest)) = value.split_once(char::is_whitespace) else {
        return;
    };
    let rest = rest.trim();

    if scheme.eq_ignore_ascii_case("basic") {
        let credentials = if rest.contains(':') {
            Some(rest.to_string())
        } else {
            rest.split_once(char::is_whitespace)
                .map(|(user, password)| format!("{}:{}", user, password.trim()))
        };
        // Without a colon or a space it is already encoded.
        if let Some(credentials) = credentials {
            request.headers[index].value = format!("Basic {}", STANDARD.encode(credentials));
        }
    } else if scheme.eq_ignore_ascii_case("digest")
        && let Some((user, password)) = rest.split_once(char::is_whitespace)
        && !rest.contains('=')
    {
        request.auth = AuthType::Digest(DigestAuth {
            username: user.to_string(),
            password: password.trim().to_string(),
        });
        request.headers.remove(index);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::KeyValuePair;

    fn with_auth(value: &str) -> RequestData {
        RequestData {
            headers: vec![KeyValuePair {
                key: "Authorization".to_string(),
                value: value.to_string(),
                enabled: true,
            }],
            ..Default::default()
        }
    }

    fn after(value: &str) -> RequestData {
        let mut request = with_auth(value);
        apply(&mut request);
        request
    }

    #[test]
    fn basic_shorthands_are_encoded() {
        let encoded = format!("Basic {}", STANDARD.encode("kestrel:pa ss"));
        assert_eq!(after("Basic kestrel:pa ss").headers[0].value, encoded);
        let encoded = format!("Basic {}", STANDARD.encode("kestrel:secret"));
        assert_eq!(after("basic kestrel secret").headers[0].value, encoded);
        // Already encoded.
        assert_eq!(
            after("Basic a2VzdHJlbDpzZWNyZXQ=").headers[0].value,
            "Basic a2VzdHJlbDpzZWNyZXQ="
        );
    }

    #[test]
    fn digest_shorthand_becomes_digest_auth() {
        let request = after("Digest kestrel secret");
        assert!(request.headers.is_empty());
        assert_eq!(
            request.auth,
            AuthType::Digest(DigestAuth {
                username: "kestrel".to_string(),
                password: "secret".to_string(),
            })
        );
        // A full digest header is left alone.
        let full = r#"Digest username="k", realm="r", response="x""#;
        assert_eq!(after(full).headers[0].value, full);
    }

    #[test]
    fn other_headers_are_left_alone() {
        assert_eq!(after("Bearer abc").headers[0].value, "Bearer abc");
        assert_eq!(after("Basic").headers[0].value, "Basic");
    }
}
