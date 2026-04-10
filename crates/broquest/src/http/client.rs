use gpui::Global;
use std::collections::HashMap;
use std::time::Duration;

use crate::domain::{
    AuthType, HttpMethod, JwtAuth, KeyValuePair, OAuth2Auth, OAuth2GrantType, RequestData,
    ResponseData,
};
use crate::environments::EnvironmentResolver;
use crate::scripting::{ScriptExecutionService, VariableStore};

use super::jwt;
use super::oauth2::{self, calculate_expires_at, is_oauth_token_expired};

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
        let timeout_duration = Duration::from_secs(300); // 5 minutes
        let client = reqwest::Client::builder()
            .timeout(timeout_duration)
            .user_agent(format!("broquest/{}", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("Failed to create HTTP client");

        let script_execution_service =
            ScriptExecutionService::new().expect("Failed to create script execution service");

        Self {
            client,
            timeout: timeout_duration,
            environment_resolver: EnvironmentResolver::new(),
            script_execution_service,
        }
    }

    /// Get the global HTTP client instance
    pub fn global(cx: &gpui::App) -> &Self {
        cx.global::<Self>()
    }

    /// Ensure OAuth2 has a valid access token, fetching if necessary
    pub async fn ensure_oauth2_token(
        &self,
        oauth: &mut OAuth2Auth,
        open_url: impl Fn(&str) + Send + 'static,
    ) -> Result<(), HttpError> {
        if !is_oauth_token_expired(oauth) {
            return Ok(());
        }

        match oauth.grant_type {
            OAuth2GrantType::ClientCredentials => {
                let token = oauth2::fetch_client_credentials_token(&self.client, oauth).await?;
                oauth.access_token = Some(token.access_token);
                oauth.refresh_token = token.refresh_token;
                if let Some(expires_in) = token.expires_in {
                    oauth.expires_at = Some(calculate_expires_at(expires_in));
                }
            }
            OAuth2GrantType::AuthorizationCode => {
                let authorize_url = oauth
                    .authorize_url
                    .as_ref()
                    .ok_or_else(|| {
                        HttpError::new(
                            "OAuth2 authorization URL missing",
                            "Authorization URL is required for authorization code flow",
                        )
                    })?
                    .clone();

                let (addr, callback_future) = oauth2::spawn_callback_server(None).await?;
                let redirect_uri = format!("http://{}", addr);

                let auth_url =
                    oauth2::build_authorization_url_with_url(&authorize_url, oauth, &redirect_uri);

                open_url(&auth_url);

                let result = callback_future.await?;

                let token =
                    oauth2::exchange_auth_code(&self.client, oauth, &result.code, &redirect_uri)
                        .await?;
                oauth.access_token = Some(token.access_token);
                oauth.refresh_token = token.refresh_token;
                if let Some(expires_in) = token.expires_in {
                    oauth.expires_at = Some(calculate_expires_at(expires_in));
                }
            }
            OAuth2GrantType::Password => {
                return Err(HttpError::new(
                    "Password grant not supported",
                    "Use client credentials or authorization code flow",
                ));
            }
        }

        Ok(())
    }

    /// Ensure JWT has a valid access token, fetching if necessary
    pub async fn ensure_jwt_token(&self, jwt: &mut JwtAuth) -> Result<(), HttpError> {
        if !jwt::is_jwt_token_expired(jwt) {
            return Ok(());
        }

        let token = jwt::fetch_jwt_token(&self.client, jwt).await?;
        jwt.access_token = Some(token.access_token);
        jwt.token_type = token.token_type;
        jwt.expires_at = token.expires_at;

        Ok(())
    }

    /// Ensure all auth tokens are valid, fetching/refreshing if necessary
    pub async fn ensure_auth_tokens(
        &self,
        auth: &mut crate::domain::AuthType,
    ) -> Result<(), HttpError> {
        use crate::domain::AuthType;

        match auth {
            AuthType::OAuth2(oauth) => {
                // Only auto-fetch for client credentials flow when token_url is configured
                if matches!(oauth.grant_type, OAuth2GrantType::ClientCredentials)
                    && !oauth.token_url.is_empty()
                {
                    self.ensure_oauth2_token(oauth, |_| {}).await?;
                }
            }
            AuthType::Jwt(jwt) => {
                // Only try to fetch if login_url is configured
                if !jwt.login_url.is_empty() {
                    self.ensure_jwt_token(jwt).await?;
                }
            }
            _ => {}
        }

        Ok(())
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

        // Ensure auth tokens are valid (fetch/refresh if needed)
        self.ensure_auth_tokens(&mut request_data.auth).await?;

        // For digest auth, use challenge-response flow
        if matches!(&request_data.auth, AuthType::Digest(_)) {
            return self
                .send_with_digest_auth(request_data, start_time, variable_store)
                .await;
        }

        // Standard request flow for non-digest auth
        let (request_builder, request_headers) = self.build_request_builder(&request_data, None);
        let response = self.execute_request(request_builder).await?;

        self.process_response(
            response,
            request_data,
            request_headers,
            start_time,
            variable_store,
        )
        .await
    }

    /// Send request with digest authentication (RFC 2617 challenge-response flow)
    async fn send_with_digest_auth(
        &self,
        request_data: RequestData,
        start_time: std::time::Instant,
        variable_store: VariableStore,
    ) -> std::result::Result<(ResponseData, VariableStore), HttpError> {
        let (username, password) = match &request_data.auth {
            AuthType::Digest(d) => (d.username.clone(), d.password.clone()),
            _ => unreachable!("send_with_digest_auth called with non-digest auth"),
        };

        // Send initial request without auth to get the challenge
        let (request_builder, initial_headers) = self.build_request_builder(&request_data, None);
        let response = self.execute_request(request_builder).await?;

        // Check for 401 with Digest challenge
        if response.status() == 401
            && let Some(www_authenticate) = response.headers().get("www-authenticate")
        {
            let www_auth_str = www_authenticate.to_str().unwrap_or("");
            if www_auth_str.starts_with("Digest") {
                tracing::info!("Received Digest challenge, computing auth response");

                // Parse the challenge and compute the digest response
                let mut prompt = match digest_auth::parse(www_auth_str) {
                    Ok(p) => p,
                    Err(e) => {
                        return Err(HttpError::new(
                            "Failed to parse Digest challenge",
                            format!("Failed to parse WWW-Authenticate header: {}", e),
                        ));
                    }
                };

                // Extract URI path for the auth context
                let uri = request_data
                    .url
                    .find("://")
                    .and_then(|scheme_end| {
                        let rest = &request_data.url[scheme_end + 3..];
                        rest.find('/')
                            .map(|path_start| rest[path_start..].to_string())
                    })
                    .unwrap_or_else(|| "/".to_string());

                let context = digest_auth::AuthContext::new(&username, &password, &uri);

                let auth_header = match prompt.respond(&context) {
                    Ok(header) => header.to_string(),
                    Err(e) => {
                        return Err(HttpError::new(
                            "Failed to compute Digest response",
                            format!("Failed to compute digest auth response: {}", e),
                        ));
                    }
                };

                tracing::info!("Retrying request with Digest authentication");

                // Retry with the computed auth header
                let (request_builder, mut request_headers) =
                    self.build_request_builder(&request_data, Some(auth_header.clone()));
                request_headers.push(KeyValuePair {
                    key: "Authorization".to_string(),
                    value: auth_header,
                    enabled: true,
                });

                let response = self.execute_request(request_builder).await?;

                return self
                    .process_response(
                        response,
                        request_data,
                        request_headers,
                        start_time,
                        variable_store,
                    )
                    .await;
            }
        }

        // Return initial response (either success or non-digest 401)
        self.process_response(
            response,
            request_data,
            initial_headers,
            start_time,
            variable_store,
        )
        .await
    }

    /// Build a request builder with optional extra auth header (for digest retry)
    fn build_request_builder(
        &self,
        request_data: &RequestData,
        extra_auth_header: Option<String>,
    ) -> (reqwest::RequestBuilder, Vec<KeyValuePair>) {
        let url = Self::apply_query_parameters(&request_data.url, &request_data.query_params);
        let mut request = self
            .client
            .request(map_http_method(request_data.method), &url);

        let mut request_headers: Vec<KeyValuePair> = Vec::new();

        // Add headers
        for header in &request_data.headers {
            if header.enabled {
                request = request.header(&header.key, &header.value);
                request_headers.push(header.clone());
            }
        }

        // Add extra auth header if provided (for digest retry)
        if let Some(auth_header) = extra_auth_header {
            request = request.header("Authorization", &auth_header);
        } else {
            // Apply standard auth
            let (req, auth_headers) = self.apply_auth_with_tracking(request, &request_data.auth);
            request = req;
            request_headers.extend(auth_headers);
        }

        // Add body for POST, PUT, PATCH requests
        if matches!(
            request_data.method,
            HttpMethod::Post | HttpMethod::Put | HttpMethod::Patch
        ) && !request_data.body.is_empty()
        {
            request = self.add_request_body(request, request_data);
        }

        (request, request_headers)
    }

    /// Add body to request, handling form data and file uploads
    fn add_request_body(
        &self,
        mut request: reqwest::RequestBuilder,
        request_data: &RequestData,
    ) -> reqwest::RequestBuilder {
        // Check if this is form data with file uploads
        if request_data.headers.iter().any(|h| {
            h.key.to_lowercase() == "content-type" && h.value == "application/x-www-form-urlencoded"
        }) {
            // Check if body contains file references (paths starting with @)
            if request_data.body.contains('@') {
                let mut form = reqwest::multipart::Form::new();

                for pair in request_data.body.split('&') {
                    if let Some(eq_pos) = pair.find('=') {
                        let key = urlencoding::decode(&pair[..eq_pos]).unwrap_or_default();
                        let value = urlencoding::decode(&pair[eq_pos + 1..]).unwrap_or_default();

                        if let Some(value_str) = value.strip_prefix('@') {
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
                                    tracing::error!("Failed to read file '{}': {}", value_str, e);
                                    form = form.text(key.to_string(), value.to_string());
                                }
                            }
                        } else {
                            form = form.text(key.to_string(), value.to_string());
                        }
                    }
                }

                request = request.multipart(form);
            } else {
                request = request.body(request_data.body.clone());
            }
        } else {
            request = request.body(request_data.body.clone());
        }

        request
    }

    /// Execute a request and return the response
    async fn execute_request(
        &self,
        request: reqwest::RequestBuilder,
    ) -> std::result::Result<reqwest::Response, HttpError> {
        async_compat::Compat::new(request.send())
            .await
            .map_err(|e| HttpError::from_reqwest_error(&e))
    }

    /// Process response into ResponseData and execute post-response scripts
    async fn process_response(
        &self,
        response: reqwest::Response,
        request_data: RequestData,
        request_headers: Vec<KeyValuePair>,
        start_time: std::time::Instant,
        variable_store: VariableStore,
    ) -> std::result::Result<(ResponseData, VariableStore), HttpError> {
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
            request_headers,
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

    /// Apply authentication to a request builder and track the headers added
    fn apply_auth_with_tracking(
        &self,
        request: reqwest::RequestBuilder,
        auth: &AuthType,
    ) -> (reqwest::RequestBuilder, Vec<KeyValuePair>) {
        use base64::{Engine as _, engine::general_purpose::STANDARD};
        let mut headers = Vec::new();
        let request = match auth {
            AuthType::None | AuthType::Inherit | AuthType::Digest(_) => request,
            AuthType::Basic(basic) => {
                let encoded = STANDARD.encode(format!("{}:{}", basic.username, basic.password));
                headers.push(KeyValuePair {
                    key: "Authorization".to_string(),
                    value: format!("Basic {}", encoded),
                    enabled: true,
                });
                request.header("Authorization", format!("Basic {}", encoded))
            }
            AuthType::Key(key) => {
                headers.push(KeyValuePair {
                    key: key.header.clone(),
                    value: key.value.clone(),
                    enabled: true,
                });
                request.header(&key.header, &key.value)
            }
            AuthType::OAuth2(oauth) => {
                if let Some(token) = &oauth.access_token {
                    headers.push(KeyValuePair {
                        key: "Authorization".to_string(),
                        value: format!("Bearer {}", token),
                        enabled: true,
                    });
                    request.bearer_auth(token)
                } else {
                    request
                }
            }
            AuthType::Jwt(jwt) => {
                if let Some(token) = &jwt.access_token {
                    let token_type = jwt.token_type.as_deref().unwrap_or("Bearer");
                    let header_value = format!("{} {}", token_type, token);
                    headers.push(KeyValuePair {
                        key: "Authorization".to_string(),
                        value: header_value.clone(),
                        enabled: true,
                    });
                    request.header("Authorization", &header_value)
                } else {
                    request
                }
            }
        };

        (request, headers)
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
            ResponseFormat::Xml => format_xml(content).unwrap_or_else(|| content.to_string()),
            _ => content.to_string(),
        }
    }
}

fn format_xml(content: &str) -> Option<String> {
    use quick_xml::events::Event;
    use quick_xml::reader::Reader;
    use quick_xml::writer::Writer;
    use std::io::Cursor;

    let mut reader = Reader::from_str(content);
    reader.config_mut().trim_text(true);

    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    loop {
        match reader.read_event().ok()? {
            Event::Eof => break,
            e => writer.write_event(e).ok()?,
        }
    }

    String::from_utf8(writer.into_inner().into_inner()).ok()
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
        assert_eq!(
            format.format_content(xml_content),
            "<root>\n  <item>value</item>\n</root>"
        );
    }
}
