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
    /// Which API the request's scripts are written against.
    #[serde(skip)]
    pub script_dialect: ScriptDialect,
    /// Set for requests that come from a `.http` file.
    #[serde(skip)]
    pub http_file: Option<HttpFileOptions>,
    /// Where the request's OAuth2 token is cached: its collection and
    /// environment, so each environment keeps its own token.
    #[serde(skip)]
    pub token_scope: String,
}

/// What a request from a `.http` file needs beyond the request itself.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HttpFileOptions {
    /// The folder of the `.http` file, which `< file` paths are relative to.
    pub dir: std::path::PathBuf,
    /// `# @no-redirect`: do not follow redirects.
    pub no_redirect: bool,
    /// `# @no-log`: leave the request out of the history.
    pub no_log: bool,
    /// `# @timeout 5s`.
    pub timeout: Option<std::time::Duration>,
    /// `# @note`: ask before sending. Holds the note's text, which may be
    /// empty.
    pub note: Option<String>,
    /// `# @prompt name description`: values to ask for before sending.
    pub prompts: Vec<Prompt>,
    /// `>> path` and `>>! path`: files to save the response body to.
    pub response_files: Vec<ResponseFile>,
    /// Where the request is stored. `None` for a request not saved yet, or a
    /// duplicate, which is saved as a new request.
    pub source: Option<HttpSource>,
}

/// Where a request of a `.http` file is stored.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct HttpSource {
    /// The file, relative to the collection folder.
    pub file: String,
    /// The request's name in that file when it was loaded.
    pub name: String,
}

/// `>> path` saves the response body next to an existing file of the same
/// name, `>>! path` overwrites it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResponseFile {
    pub path: String,
    pub overwrite: bool,
}

/// A variable to ask for before sending, from `# @prompt`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Prompt {
    pub name: String,
    pub description: Option<String>,
}

impl Prompt {
    /// Whether to hide what is typed, as the VS Code REST Client does for
    /// names like `password`.
    pub fn is_secret(&self) -> bool {
        let name = self.name.to_ascii_lowercase();
        name.contains("password") || name.contains("passwd") || name == "pass"
    }
}

/// The API a request's scripts use.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ScriptDialect {
    /// Broquest's `bro`, `req` and `res`, with Bruno's `bru` API on top.
    #[default]
    Broquest,
    /// The JetBrains HTTP Client's `client`, `request` and `response`, as in
    /// `.http` files.
    JetBrains,
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
            script_dialect: ScriptDialect::default(),
            http_file: None,
            token_scope: String::new(),
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
    /// The request body as sent, variables resolved.
    pub request_body: String,
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
