//! OAuth2 token acquisition logic
//!
//! Supports Client Credentials and Authorization Code grant types.

use crate::domain::OAuth2Auth;
use smol::io::{AsyncReadExt, AsyncWriteExt};
use smol::net::TcpListener;
use std::collections::HashMap;
use std::net::SocketAddr;

use super::{HttpError, current_unix_timestamp, is_token_expired};

/// OAuth2 token response from authorization server
#[derive(Debug, Clone, serde::Deserialize)]
#[allow(dead_code)]
pub struct TokenResponse {
    pub access_token: String,
    #[serde(default)]
    pub token_type: String,
    #[serde(default)]
    pub expires_in: Option<u64>,
    #[serde(default)]
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub scope: Option<String>,
}

/// Result of authorization code callback
#[allow(dead_code)]
pub struct AuthCodeResult {
    pub code: String,
    pub state: Option<String>,
}

/// Fetch a new access token using client credentials grant
pub async fn fetch_client_credentials_token(
    client: &reqwest::Client,
    oauth: &OAuth2Auth,
) -> Result<TokenResponse, HttpError> {
    let mut params = vec![
        ("grant_type", "client_credentials".to_string()),
        ("client_id", oauth.client_id.clone()),
        ("client_secret", oauth.client_secret.clone()),
    ];

    if let Some(scope) = &oauth.scope {
        params.push(("scope", scope.clone()));
    }

    let response = async_compat::Compat::new(client.post(&oauth.token_url).form(&params).send())
        .await
        .map_err(|e| HttpError::new("OAuth2 token request failed", e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = async_compat::Compat::new(response.text())
            .await
            .unwrap_or_default();
        return Err(HttpError::new(
            "OAuth2 token request failed",
            format!("Status {}: {}", status, body),
        ));
    }

    async_compat::Compat::new(response.json::<TokenResponse>())
        .await
        .map_err(|e| HttpError::new("Failed to parse token response", e.to_string()))
}

/// Spawn a local HTTP server to receive OAuth2 callback
/// Returns the server address and a future that resolves when callback is received
pub async fn spawn_callback_server(
    port: Option<u16>,
) -> Result<
    (
        SocketAddr,
        impl std::future::Future<Output = Result<AuthCodeResult, HttpError>>,
    ),
    HttpError,
> {
    let listener = TcpListener::bind(("127.0.0.1", port.unwrap_or(0)))
        .await
        .map_err(|e| HttpError::new("Failed to start callback server", e.to_string()))?;

    let addr = listener
        .local_addr()
        .map_err(|e| HttpError::new("Failed to get server address", e.to_string()))?;

    let future = async move {
        let (mut stream, _) = listener
            .accept()
            .await
            .map_err(|e| HttpError::new("Failed to accept connection", e.to_string()))?;

        let mut buffer = vec![0u8; 4096];
        let n = stream
            .read(&mut buffer)
            .await
            .map_err(|e| HttpError::new("Failed to read request", e.to_string()))?;

        let request = String::from_utf8_lossy(&buffer[..n]);
        let path = extract_path(&request)?;

        let query = path.split('?').nth(1).unwrap_or("");
        let params: HashMap<&str, &str> = query
            .split('&')
            .filter_map(|p| {
                let mut parts = p.splitn(2, '=');
                Some((parts.next()?, parts.next().unwrap_or("")))
            })
            .collect();

        if let Some(error) = params.get("error") {
            return Err(HttpError::new(
                "OAuth2 authorization failed",
                params
                    .get("error_description")
                    .map(|d| d.to_string())
                    .unwrap_or_else(|| error.to_string()),
            ));
        }

        let code = params
            .get("code")
            .ok_or_else(|| {
                HttpError::new(
                    "OAuth2 callback missing code",
                    "No authorization code in callback",
                )
            })?
            .to_string();

        let state = params.get("state").map(|s| s.to_string());

        let response = "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\n\r\n\
            <html><body><h1>Authorization successful!</h1>\
            <p>You can close this window now.</p></body></html>";
        if let Err(e) = stream.write_all(response.as_bytes()).await {
            tracing::error!("Failed to write OAuth callback response: {}", e);
        }

        Ok(AuthCodeResult { code, state })
    };

    Ok((addr, future))
}

fn extract_path(request: &str) -> Result<String, HttpError> {
    let first_line = request
        .lines()
        .next()
        .ok_or_else(|| HttpError::new("Invalid HTTP request", "Empty request"))?;
    let path = first_line
        .split(' ')
        .nth(1)
        .ok_or_else(|| HttpError::new("Invalid HTTP request", "No path in request"))?;
    Ok(path.to_string())
}

/// Exchange authorization code for access token
pub async fn exchange_auth_code(
    client: &reqwest::Client,
    oauth: &OAuth2Auth,
    code: &str,
    redirect_uri: &str,
) -> Result<TokenResponse, HttpError> {
    let params = vec![
        ("grant_type", "authorization_code".to_string()),
        ("code", code.to_string()),
        ("redirect_uri", redirect_uri.to_string()),
        ("client_id", oauth.client_id.clone()),
        ("client_secret", oauth.client_secret.clone()),
    ];

    let response = async_compat::Compat::new(client.post(&oauth.token_url).form(&params).send())
        .await
        .map_err(|e| HttpError::new("OAuth2 token exchange failed", e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = async_compat::Compat::new(response.text())
            .await
            .unwrap_or_default();
        return Err(HttpError::new(
            "OAuth2 token exchange failed",
            format!("Status {}: {}", status, body),
        ));
    }

    async_compat::Compat::new(response.json::<TokenResponse>())
        .await
        .map_err(|e| HttpError::new("Failed to parse token response", e.to_string()))
}

/// Build the authorization URL with a provided authorize_url
pub fn build_authorization_url_with_url(
    authorize_url: &str,
    oauth: &OAuth2Auth,
    redirect_uri: &str,
) -> String {
    let mut url = authorize_url.to_string();
    url.push_str("?response_type=code");
    url.push_str("&client_id=");
    url.push_str(&urlencoding::encode(&oauth.client_id));
    url.push_str("&redirect_uri=");
    url.push_str(&urlencoding::encode(redirect_uri));

    if let Some(scope) = &oauth.scope {
        url.push_str("&scope=");
        url.push_str(&urlencoding::encode(scope));
    }

    url
}

/// Calculate expiration timestamp from expires_in seconds
pub fn calculate_expires_at(expires_in: u64) -> i64 {
    current_unix_timestamp() + expires_in as i64
}

/// Check if the token has expired or will expire soon
pub fn is_oauth_token_expired(oauth: &OAuth2Auth) -> bool {
    is_token_expired(oauth.expires_at, oauth.access_token.is_some())
}
