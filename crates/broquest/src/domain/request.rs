//! Core request and response data types

use serde::{Deserialize, Serialize};
use std::time::Duration;

use super::{AuthType, HttpMethod};

/// A key-value pair with an enabled flag, used for headers, query params, etc.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

impl Default for KeyValuePair {
    fn default() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
            enabled: true,
        }
    }
}

impl KeyValuePair {
    /// Check if two KeyValuePair vectors are equal (order-independent comparison).
    /// Filters out entries with empty keys and compares the remaining entries.
    pub fn vec_equals(left: &[KeyValuePair], right: &[KeyValuePair]) -> bool {
        // Filter out entries with empty keys (they don't represent real data)
        let left_filtered: Vec<_> = left.iter().filter(|p| !p.key.trim().is_empty()).collect();
        let right_filtered: Vec<_> = right.iter().filter(|p| !p.key.trim().is_empty()).collect();

        if left_filtered.len() != right_filtered.len() {
            return false;
        }

        // For each item in left, find a matching item in right
        for l in &left_filtered {
            if !right_filtered
                .iter()
                .any(|r| r.key == l.key && r.value == l.value && r.enabled == l.enabled)
            {
                return false;
            }
        }
        true
    }
}

/// Request data for HTTP requests
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestData {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub path_params: Vec<KeyValuePair>,
    pub query_params: Vec<KeyValuePair>,
    pub headers: Vec<KeyValuePair>,
    pub body: String,
    pub pre_request_script: Option<String>,
    pub post_response_script: Option<String>,
    #[serde(default)]
    pub auth: AuthType,
    /// Request-level variables (Bruno `runtime.variables` / `bru.getRequestVar`),
    /// resolvable in this request via `{{name}}`.
    #[serde(default)]
    pub vars: Vec<KeyValuePair>,
}

impl Default for RequestData {
    fn default() -> Self {
        Self {
            name: "New Request".to_string(),
            method: HttpMethod::Get,
            url: String::new(),
            path_params: Vec::new(),
            query_params: Vec::new(),
            headers: Vec::new(),
            body: String::new(),
            pre_request_script: None,
            post_response_script: None,
            auth: AuthType::None,
            vars: Vec::new(),
        }
    }
}

/// Response data from HTTP requests
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResponseData {
    pub status_code: Option<u16>,
    pub status_text: Option<String>,
    pub latency: Option<Duration>,
    pub size: Option<usize>,
    pub headers: Vec<KeyValuePair>,
    pub request_headers: Vec<KeyValuePair>,
    pub body: String,
    pub body_bytes: Option<Vec<u8>>,
    pub url: Option<String>,
    /// How the body arrived when it was compressed. `size` above is the
    /// decoded size.
    pub encoded: Option<EncodedBody>,
}

/// A compressed response body as it arrived.
#[derive(Debug, Clone, PartialEq)]
pub struct EncodedBody {
    /// The `Content-Encoding` header, such as `gzip` or `br`.
    pub encoding: String,
    /// Size of the body as sent, in bytes.
    pub size: usize,
}

/// Split a URL into the part before its query and the decoded query
/// parameters. A `#fragment` is dropped.
pub fn split_query(url: &str) -> (String, Vec<KeyValuePair>) {
    let mut base_url = url.to_string();
    let mut query_params = Vec::new();

    // Check if URL has query parameters
    if let Some(query_start) = url.find('?') {
        // Extract the base URL (before ?)
        base_url = url[..query_start].to_string();

        // Extract query string (after ?, before # if present)
        let query_part = if let Some(fragment_start) = url.find('#') {
            // Fragment comes after query
            if fragment_start > query_start {
                &url[query_start + 1..fragment_start]
            } else {
                &url[query_start + 1..]
            }
        } else {
            &url[query_start + 1..]
        };

        // Parse query parameters
        for pair in query_part.split('&') {
            if let Some(eq_pos) = pair.find('=') {
                let key = &pair[..eq_pos];
                let value = &pair[eq_pos + 1..];

                if !key.is_empty() {
                    query_params.push(KeyValuePair {
                        key: urlencoding::decode(key)
                            .unwrap_or_else(|_| key.to_string().into())
                            .to_string(),
                        value: urlencoding::decode(value)
                            .unwrap_or_else(|_| value.to_string().into())
                            .to_string(),
                        enabled: true,
                    });
                }
            } else if !pair.is_empty() {
                // Parameter without value
                query_params.push(KeyValuePair {
                    key: urlencoding::decode(pair)
                        .unwrap_or_else(|_| pair.to_string().into())
                        .to_string(),
                    value: String::new(),
                    enabled: true,
                });
            }
        }
    }

    (base_url, query_params)
}
