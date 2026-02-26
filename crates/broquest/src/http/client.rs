use gpui::Global;
use std::collections::HashMap;
use std::time::Duration;

use crate::domain::{HttpMethod, KeyValuePair, RequestData, ResponseData};
use crate::environments::EnvironmentResolver;
use crate::scripting::{ScriptExecutionService, VariableStore};

/// Well-defined error type for HTTP requests
#[derive(Debug, Clone)]
pub struct HttpError {
    pub summary: String,
    pub details: String,
}

impl std::error::Error for HttpError {}
impl std::fmt::Display for HttpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl HttpError {
    pub fn new(summary: impl Into<String>, details: impl Into<String>) -> Self {
        Self {
            summary: summary.into(),
            details: details.into(),
        }
    }

    fn from_reqwest_error(e: &reqwest::Error) -> Self {
        if e.is_timeout() {
            Self::new("Request timed out", format!("Request timed out - {}", e))
        } else if e.is_connect() {
            Self::new(
                "Request failed: couldn't connect",
                format!("Connection failed - {}", e),
            )
        } else if e.is_request() {
            Self::new("Request failed", format!("Request setup error - {}", e))
        } else if e.is_body() {
            Self::new("Request failed", format!("Request body error - {}", e))
        } else if e.is_decode() {
            Self::new("Request failed", format!("Response decode error - {}", e))
        } else {
            Self::new("Request failed", e.to_string())
        }
    }
}

/// Global HTTP client service for sending API requests
#[derive(Clone)]
#[allow(dead_code)]
pub struct HttpClientService {
    client: reqwest::Client,
    timeout: Duration,
    environment_resolver: EnvironmentResolver,
    script_execution_service: ScriptExecutionService,
}

impl Global for HttpClientService {}

impl HttpClientService {
    pub fn new() -> Self {
        let client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .user_agent(format!("broquest/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        let script_execution_service =
            ScriptExecutionService::new().expect("Failed to create script execution service");

        Self {
            client,
            timeout: Duration::from_secs(30),
            environment_resolver: EnvironmentResolver::new(),
            script_execution_service,
        }
    }

    /// Get the global HTTP client instance
    pub fn global(cx: &gpui::App) -> &Self {
        cx.global::<Self>()
    }

    pub async fn send_request(
        &self,
        request_data: RequestData,
        variables: Option<HashMap<String, String>>,
        secrets: Option<HashMap<String, String>>,
    ) -> std::result::Result<(ResponseData, VariableStore), HttpError> {
        self.send_request_internal(request_data, variables, secrets)
            .await
    }

    async fn send_request_internal(
        &self,
        mut request_data: RequestData,
        variables: Option<HashMap<String, String>>,
        secrets: Option<HashMap<String, String>>,
    ) -> std::result::Result<(ResponseData, VariableStore), HttpError> {
        let start_time = std::time::Instant::now();

        // Create variable store for this request
        let variable_store = VariableStore::new();

        // Initialize variable store with environment data if provided
        if let (Some(variables), Some(secrets)) = (variables, secrets) {
            tracing::info!(
                "Loaded {} variables and {} secrets for request",
                variables.len(),
                secrets.len()
            );

            // Initialize variable store with environment data
            variable_store.initialize_with_env(&variables, &secrets);

            // Resolve variables in request data using EnvironmentResolver
            request_data =
                self.environment_resolver
                    .resolve_request_data(request_data, &variables, &secrets);

            tracing::info!("URL after environment substitution: {}", request_data.url);
        }

        tracing::info!(
            "Sending {} request to {}",
            request_data.method.as_str(),
            request_data.url
        );

        // Execute pre-request script if present
        if let Some(pre_request_script) = request_data.pre_request_script.clone() {
            tracing::info!("Executing pre-request script");
            if let Err(e) = self.script_execution_service.execute_pre_request_script(
                &pre_request_script,
                &mut request_data,
                &variable_store,
            ) {
                tracing::error!("Failed to execute pre-request script: {}", e);
                return Err(HttpError::new(
                    "Pre-request script execution failed",
                    format!("Pre-request script execution failed: {}", e),
                ));
            }
        }

        // Apply query parameters to URL for proper encoding after variable substitution
        let url = Self::apply_query_parameters(&request_data.url, &request_data.query_params);
        let mut request = self
            .client
            .request(map_http_method(request_data.method), &url);

        // Add headers
        for header in &request_data.headers {
            if header.enabled {
                request = request.header(&header.key, &header.value);
            }
        }

        // Add body for POST, PUT, PATCH requests
        if matches!(
            request_data.method,
            HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch
        ) && !request_data.body.is_empty()
        {
            // Check if this is form data with file uploads
            if request_data.headers.iter().any(|h| {
                h.key.to_lowercase() == "content-type"
                    && h.value == "application/x-www-form-urlencoded"
            }) {
                // Check if body contains file references (paths starting with @)
                if request_data.body.contains('@') {
                    // Parse form data and create multipart form
                    let mut form = reqwest::multipart::Form::new();

                    for pair in request_data.body.split('&') {
                        if let Some(eq_pos) = pair.find('=') {
                            let key = urlencoding::decode(&pair[..eq_pos]).unwrap_or_default();
                            let value =
                                urlencoding::decode(&pair[eq_pos + 1..]).unwrap_or_default();

                            if let Some(value_str) = value.strip_prefix('@') {
                                // This is a file upload
                                match std::fs::read(value_str) {
                                    Ok(file_contents) => {
                                        let file_name = std::path::Path::new(value_str)
                                            .file_name()
                                            .and_then(|n| n.to_str())
                                            .unwrap_or("file");
                                        let part = reqwest::multipart::Part::bytes(file_contents)
                                            .file_name(file_name.to_string());
                                        form = form.part(key.to_string(), part);
                                    }
                                    Err(e) => {
                                        tracing::error!(
                                            "Failed to read file '{}': {}",
                                            value_str,
                                            e
                                        );
                                        // Fall back to regular form field
                                        form = form.text(key.to_string(), value.to_string());
                                    }
                                }
                            } else {
                                // Regular form field
                                form = form.text(key.to_string(), value.to_string());
                            }
                        }
                    }

                    request = request.multipart(form);
                } else {
                    // Regular form data without files
                    request = request.body(request_data.body.clone());
                }
            } else {
                // Regular body
                request = request.body(request_data.body.clone());
            }
        }

        // Execute the request with async-compat
        let response = async_compat::Compat::new(request.send())
            .await
            .map_err(|e| HttpError::from_reqwest_error(&e))?;

        let status = response.status();
        let status_code = status.as_u16();
        let status_text = status.canonical_reason().map(|s| s.to_string());

        // Get response headers
        let response_headers = response
            .headers()
            .iter()
            .filter_map(|(name, value)| {
                value.to_str().ok().map(|v| KeyValuePair {
                    key: name.to_string(),
                    value: v.to_string(),
                    enabled: true,
                })
            })
            .collect::<Vec<_>>();

        // Get response body with async-compat
        let response_body = async_compat::Compat::new(response.text())
            .await
            .map_err(|e| {
                HttpError::new(
                    "Failed to read response body",
                    format!("Failed to read response body: {}", e),
                )
            })?;

        let latency = start_time.elapsed();
        let response_size = response_body.len();

        let response_data = ResponseData {
            status_code: Some(status_code),
            status_text: status_text.clone(),
            latency: Some(latency),
            size: Some(response_size),
            headers: response_headers,
            body: response_body,
            url: Some(request_data.url.clone()),
        };

        // Execute post-response script if present
        if let Some(post_response_script) = &request_data.post_response_script {
            tracing::info!("Executing post-response script");
            if let Err(e) = self.script_execution_service.execute_post_response_script(
                post_response_script,
                &request_data,
                &response_data,
                &variable_store,
            ) {
                tracing::error!("Failed to execute post-response script: {}", e);
                return Err(HttpError::new(
                    "Post-response script execution failed",
                    format!("Post-response script execution failed: {}", e),
                ));
            }
        }

        tracing::info!(
            "Request completed: {} {} ({} bytes, {}ms)",
            status_code,
            status_text.unwrap_or_else(|| "Unknown".to_string()),
            response_size,
            latency.as_millis()
        );

        // Check for dirty environment variables
        let dirty_vars = variable_store.get_dirty_env_vars();
        if !dirty_vars.is_empty() {
            tracing::info!(
                "Environment variables modified by scripts: {:?}",
                dirty_vars.keys().collect::<Vec<_>>()
            );
        }

        Ok((response_data, variable_store))
    }
    /// Apply query parameters to a URL, handling URL encoding
    fn apply_query_parameters(url: &str, params: &[KeyValuePair]) -> String {
        let mut result = url.to_string();

        // Filter enabled parameters with non-empty keys
        let enabled_params: Vec<_> = params
            .iter()
            .filter(|p| p.enabled && !p.key.is_empty())
            .collect();

        if enabled_params.is_empty() {
            return result;
        }

        // Remove existing query string and fragment if present
        if let Some(query_start) = result.find('?') {
            if let Some(fragment_start) = result.find('#') {
                // Fragment comes after query
                if fragment_start > query_start {
                    result.truncate(fragment_start);
                }
            } else {
                // No fragment, just truncate at query
                result.truncate(query_start);
            }
        }

        // Build query string
        let query_string = enabled_params
            .iter()
            .map(|p| {
                format!(
                    "{}={}",
                    urlencoding::encode(&p.key),
                    urlencoding::encode(&p.value)
                )
            })
            .collect::<Vec<_>>()
            .join("&");

        // Append query string
        result.push('?');
        result.push_str(&query_string);

        result
    }
}

fn map_http_method(method: HttpMethod) -> reqwest::Method {
    match method {
        HttpMethod::Get => reqwest::Method::GET,
        HttpMethod::Post => reqwest::Method::POST,
        HttpMethod::Put => reqwest::Method::PUT,
        HttpMethod::Delete => reqwest::Method::DELETE,
        HttpMethod::Patch => reqwest::Method::PATCH,
        HttpMethod::Head => reqwest::Method::HEAD,
        HttpMethod::Options => reqwest::Method::OPTIONS,
    }
}

/// Content type detection for response formatting
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseFormat {
    Json,
    Xml,
    Html,
    PlainText,
    Binary,
    Unknown,
}

impl ResponseFormat {
    pub fn from_content_type(content_type: &str) -> Self {
        let content_type = content_type.to_lowercase();

        if content_type.contains("json") {
            ResponseFormat::Json
        } else if content_type.contains("xml") {
            ResponseFormat::Xml
        } else if content_type.contains("html") {
            ResponseFormat::Html
        } else if content_type.contains("text") || content_type.contains("plain") {
            ResponseFormat::PlainText
        } else if content_type.contains("application/octet-stream")
            || content_type.contains("image/")
            || content_type.contains("video/")
            || content_type.contains("audio/")
        {
            ResponseFormat::Binary
        } else {
            ResponseFormat::Unknown
        }
    }

    pub fn detect_from_content(content: &str, headers: &[KeyValuePair]) -> Self {
        // Try to detect from content-type header first
        if let Some(content_type_header) = headers
            .iter()
            .find(|h| h.key.to_lowercase() == "content-type")
        {
            return Self::from_content_type(&content_type_header.value);
        }

        // Try to detect from content
        let content_trimmed = content.trim();

        // Check if it looks like JSON
        if (content_trimmed.starts_with('{') && content_trimmed.ends_with('}'))
            || (content_trimmed.starts_with('[') && content_trimmed.ends_with(']'))
        {
            return ResponseFormat::Json;
        }

        // Check if it looks like XML
        if content_trimmed.starts_with('<') && content_trimmed.ends_with('>') {
            return ResponseFormat::Xml;
        }

        // Check if it looks like HTML
        if content_trimmed.to_lowercase().starts_with("<html")
            || content_trimmed.to_lowercase().starts_with("<!doctype html")
        {
            return ResponseFormat::Html;
        }

        ResponseFormat::Unknown
    }

    pub fn language_string(&self) -> &'static str {
        match self {
            ResponseFormat::Json => "json",
            ResponseFormat::Xml => "xml",
            ResponseFormat::Html => "html",
            ResponseFormat::PlainText => "text",
            ResponseFormat::Binary => "text",
            ResponseFormat::Unknown => "text",
        }
    }

    /// Format content with pretty printing if it's JSON
    pub fn format_content(&self, content: &str) -> String {
        match self {
            ResponseFormat::Json => {
                // Try to parse as JSON and pretty print
                match serde_json::from_str::<serde_json::Value>(content) {
                    Ok(value) => {
                        serde_json::to_string_pretty(&value).unwrap_or_else(|_| content.to_string())
                    }
                    Err(_) => {
                        // If parsing fails, return the original content
                        content.to_string()
                    }
                }
            }
            _ => content.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_type_detection() {
        assert_eq!(
            ResponseFormat::from_content_type("application/json"),
            ResponseFormat::Json
        );
        assert_eq!(
            ResponseFormat::from_content_type("text/html"),
            ResponseFormat::Html
        );
        assert_eq!(
            ResponseFormat::from_content_type("application/xml"),
            ResponseFormat::Xml
        );
        assert_eq!(
            ResponseFormat::from_content_type("text/plain"),
            ResponseFormat::PlainText
        );
        assert_eq!(
            ResponseFormat::from_content_type("application/octet-stream"),
            ResponseFormat::Binary
        );
    }

    #[test]
    fn test_content_detection() {
        let json_content = r#"{"key": "value"}"#;
        let headers = vec![KeyValuePair {
            key: "content-type".to_string(),
            value: "application/json".to_string(),
            enabled: true,
        }];

        assert_eq!(
            ResponseFormat::detect_from_content(json_content, &headers),
            ResponseFormat::Json
        );
    }

    #[test]
    fn test_json_formatting() {
        let format = ResponseFormat::Json;

        // Test with valid JSON
        let compact_json = r#"{"name":"John","age":30,"city":"New York"}"#;
        let expected_pretty = r#"{
  "name": "John",
  "age": 30,
  "city": "New York"
}"#;
        assert_eq!(format.format_content(compact_json), expected_pretty);

        // Test with invalid JSON - should return original
        let invalid_json = r#"{"name":"John","age":30,"city":"New York""#;
        assert_eq!(format.format_content(invalid_json), invalid_json);
    }

    #[test]
    fn test_non_json_formatting() {
        let format = ResponseFormat::PlainText;
        let text_content = "Just some plain text";
        assert_eq!(format.format_content(text_content), text_content);

        let format = ResponseFormat::Xml;
        let xml_content = "<root><item>value</item></root>";
        assert_eq!(format.format_content(xml_content), xml_content);
    }
}
