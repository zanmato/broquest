//! OAuth2 tokens: the client credentials and authorization code grants,
//! refresh tokens, PKCE and the cache that keeps tokens between requests.

use crate::domain::{OAuth2Auth, OAuth2ClientAuth, OAuth2GrantType};
use base64::Engine as _;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use sha2::{Digest, Sha256};
use smol::io::{AsyncReadExt, AsyncWriteExt};
use smol::net::{TcpListener, TcpStream};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use super::{HttpError, current_unix_timestamp, is_token_expired};

/// Where the authorization server sends the browser back to when no
/// redirect URL is set. A fixed port, so it can be registered with the
/// provider.
pub const DEFAULT_REDIRECT_URL: &str = "http://127.0.0.1:41983/callback";

/// How long the authorization code flow waits for the browser.
pub const AUTHORIZATION_TIMEOUT: Duration = Duration::from_secs(300);

/// A token endpoint's answer.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TokenResponse {
    pub access_token: String,
    pub token_type: Option<String>,
    pub expires_in: Option<u64>,
    pub refresh_token: Option<String>,
    pub id_token: Option<String>,
}

/// Read a token endpoint's answer, JSON or, as GitHub sends by default, form
/// encoded. Some servers answer 200 with an `error` field.
pub fn parse_token_response(body: &str) -> Result<TokenResponse, HttpError> {
    let fields: HashMap<String, serde_json::Value> =
        match serde_json::from_str::<serde_json::Value>(body) {
            Ok(serde_json::Value::Object(map)) => map.into_iter().collect(),
            _ => url::form_urlencoded::parse(body.trim().as_bytes())
                .map(|(k, v)| (k.into_owned(), serde_json::Value::String(v.into_owned())))
                .collect(),
        };
    let text = |name: &str| match fields.get(name) {
        Some(serde_json::Value::String(s)) if !s.is_empty() => Some(s.clone()),
        Some(serde_json::Value::Number(n)) => Some(n.to_string()),
        _ => None,
    };

    if let Some(error) = text("error") {
        return Err(HttpError::new(
            "OAuth2 token request failed",
            text("error_description").unwrap_or(error),
        ));
    }
    let access_token = text("access_token").ok_or_else(|| {
        HttpError::new(
            "Failed to parse token response",
            format!("No access_token in the response: {}", body.trim()),
        )
    })?;
    Ok(TokenResponse {
        access_token,
        token_type: text("token_type"),
        expires_in: text("expires_in").and_then(|s| s.parse::<f64>().ok().map(|n| n as u64)),
        refresh_token: text("refresh_token"),
        id_token: text("id_token"),
    })
}

/// A token kept between requests.
#[derive(Debug, Clone, PartialEq)]
pub struct CachedToken {
    pub access_token: String,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_at: Option<i64>,
}

impl CachedToken {
    /// The token a response holds. A refresh that sends no new refresh token
    /// keeps the old one.
    pub fn from_response(response: TokenResponse, previous_refresh: Option<String>) -> Self {
        Self {
            access_token: response.access_token,
            id_token: response.id_token,
            refresh_token: response.refresh_token.or(previous_refresh),
            expires_at: response.expires_in.map(calculate_expires_at),
        }
    }

    pub fn is_expired(&self) -> bool {
        is_token_expired(self.expires_at, true)
    }

    /// Put the token on a request's OAuth2 settings.
    pub fn apply(&self, oauth: &mut OAuth2Auth) {
        oauth.access_token = Some(self.access_token.clone());
        oauth.id_token = self.id_token.clone();
        oauth.refresh_token = self.refresh_token.clone();
        oauth.expires_at = self.expires_at;
    }
}

/// Tokens kept for the app's lifetime, by [`cache_key`].
#[derive(Debug, Clone, Default)]
pub struct TokenCache {
    tokens: Arc<Mutex<HashMap<String, CachedToken>>>,
}

impl TokenCache {
    pub fn get(&self, key: &str) -> Option<CachedToken> {
        self.tokens.lock().ok()?.get(key).cloned()
    }

    pub fn insert(&self, key: String, token: CachedToken) {
        if let Ok(mut tokens) = self.tokens.lock() {
            tokens.insert(key, token);
        }
    }

    pub fn remove(&self, key: &str) -> Option<CachedToken> {
        self.tokens.lock().ok()?.remove(key)
    }
}

/// Which cached token a request uses. Requests in the same collection and
/// environment with the same token settings share one. The headers a token
/// is sent in don't count, and neither does a token ID left empty.
pub fn cache_key(scope: &str, oauth: &OAuth2Auth) -> String {
    let mut hasher = Sha256::new();
    let mut add = |part: &str| {
        hasher.update(part.as_bytes());
        hasher.update([0]);
    };
    add(scope);
    add(&oauth.token_id);
    add(oauth.grant_type.as_str());
    add(&oauth.token_url);
    add(&oauth.client_id);
    add(&oauth.client_secret);
    add(oauth.scope.as_deref().unwrap_or(""));
    add(oauth.authorize_url.as_deref().unwrap_or(""));
    add(match oauth.client_auth {
        OAuth2ClientAuth::Body => "body",
        OAuth2ClientAuth::BasicHeader => "basic",
    });
    for param in oauth.extra_params.iter().filter(|p| p.enabled) {
        add(&param.key);
        add(&param.value);
    }
    hex::encode(hasher.finalize())
}

/// A PKCE code verifier and its S256 challenge (RFC 7636).
pub struct Pkce {
    pub verifier: String,
    pub challenge: String,
}

impl Pkce {
    pub fn new() -> Self {
        Self::from_verifier(random_token())
    }

    pub fn from_verifier(verifier: String) -> Self {
        let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
        Self {
            verifier,
            challenge,
        }
    }
}

/// 43 URL safe characters from 32 random bytes.
fn random_token() -> String {
    let mut bytes = [0u8; 32];
    bytes[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    bytes[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    URL_SAFE_NO_PAD.encode(bytes)
}

/// The `state` to send: the configured one, else a random one.
pub fn state_for(oauth: &OAuth2Auth) -> String {
    oauth
        .state
        .clone()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(random_token)
}

fn form_encode(value: &str) -> String {
    url::form_urlencoded::byte_serialize(value.as_bytes()).collect()
}

/// POST a token request, adding the client credentials and extra fields.
async fn token_request(
    client: &reqwest::Client,
    url: &str,
    oauth: &OAuth2Auth,
    mut params: Vec<(String, String)>,
) -> Result<TokenResponse, HttpError> {
    if url.trim().is_empty() {
        return Err(HttpError::new(
            "OAuth2 token URL missing",
            "Set the token URL to fetch a token",
        ));
    }
    if oauth.client_auth == OAuth2ClientAuth::Body {
        if !oauth.client_id.is_empty() {
            params.push(("client_id".into(), oauth.client_id.clone()));
        }
        if !oauth.client_secret.is_empty() {
            params.push(("client_secret".into(), oauth.client_secret.clone()));
        }
    }
    for param in &oauth.extra_params {
        if param.enabled && !param.key.is_empty() {
            params.push((param.key.clone(), param.value.clone()));
        }
    }

    let mut request = client
        .post(url)
        .header(reqwest::header::ACCEPT, "application/json")
        .form(&params);
    if oauth.client_auth == OAuth2ClientAuth::BasicHeader {
        request = request.basic_auth(
            form_encode(&oauth.client_id),
            Some(form_encode(&oauth.client_secret)),
        );
    }

    let response = async_compat::Compat::new(request.send())
        .await
        .map_err(|e| HttpError::new("OAuth2 token request failed", e.to_string()))?;
    let status = response.status();
    let body = async_compat::Compat::new(response.text())
        .await
        .unwrap_or_default();
    if !status.is_success() {
        return Err(HttpError::new(
            "OAuth2 token request failed",
            format!("Status {}: {}", status, body),
        ));
    }
    parse_token_response(&body)
}

fn scope_param(oauth: &OAuth2Auth) -> Option<(String, String)> {
    oauth
        .scope
        .as_ref()
        .filter(|s| !s.is_empty())
        .map(|s| ("scope".to_string(), s.clone()))
}

/// Fetch a new access token using client credentials grant
pub async fn fetch_client_credentials_token(
    client: &reqwest::Client,
    oauth: &OAuth2Auth,
) -> Result<TokenResponse, HttpError> {
    let mut params = vec![("grant_type".to_string(), "client_credentials".to_string())];
    params.extend(scope_param(oauth));
    token_request(client, &oauth.token_url, oauth, params).await
}

/// Exchange authorization code for access token
pub async fn exchange_auth_code(
    client: &reqwest::Client,
    oauth: &OAuth2Auth,
    code: &str,
    redirect_uri: &str,
    code_verifier: Option<&str>,
) -> Result<TokenResponse, HttpError> {
    let mut params = vec![
        ("grant_type".to_string(), "authorization_code".to_string()),
        ("code".to_string(), code.to_string()),
        ("redirect_uri".to_string(), redirect_uri.to_string()),
    ];
    if let Some(verifier) = code_verifier {
        params.push(("code_verifier".to_string(), verifier.to_string()));
    }
    token_request(client, &oauth.token_url, oauth, params).await
}

/// Trade a refresh token for a new access token, at the refresh URL or else
/// the token URL.
pub async fn refresh_token(
    client: &reqwest::Client,
    oauth: &OAuth2Auth,
    refresh_token: &str,
) -> Result<TokenResponse, HttpError> {
    let url = oauth
        .refresh_url
        .as_deref()
        .filter(|u| !u.trim().is_empty())
        .unwrap_or(&oauth.token_url);
    let mut params = vec![
        ("grant_type".to_string(), "refresh_token".to_string()),
        ("refresh_token".to_string(), refresh_token.to_string()),
    ];
    params.extend(scope_param(oauth));
    token_request(client, url, oauth, params).await
}

/// Where the browser goes to sign in, with `state`, the PKCE challenge and
/// the extra fields. With `encode_url` off the values go on as written.
pub fn build_authorization_url(
    authorize_url: &str,
    oauth: &OAuth2Auth,
    redirect_uri: &str,
    state: &str,
    code_challenge: Option<&str>,
) -> String {
    let mut params: Vec<(&str, &str)> = vec![
        ("response_type", "code"),
        ("client_id", &oauth.client_id),
        ("redirect_uri", redirect_uri),
    ];
    if let Some(scope) = oauth.scope.as_deref().filter(|s| !s.is_empty()) {
        params.push(("scope", scope));
    }
    params.push(("state", state));
    if let Some(challenge) = code_challenge {
        params.push(("code_challenge", challenge));
        params.push(("code_challenge_method", "S256"));
    }
    for param in &oauth.extra_params {
        if param.enabled && !param.key.is_empty() {
            params.push((&param.key, &param.value));
        }
    }

    let encode = |s: &str| {
        if oauth.encode_url {
            urlencoding::encode(s).into_owned()
        } else {
            s.to_string()
        }
    };
    let query = params
        .iter()
        .map(|(k, v)| format!("{}={}", encode(k), encode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let separator = match authorize_url.find('?') {
        None => "?",
        Some(_) if authorize_url.ends_with(['?', '&']) => "",
        Some(_) => "&",
    };
    format!("{authorize_url}{separator}{query}")
}

/// A local listener for the authorization server's redirect.
pub struct CallbackServer {
    listener: TcpListener,
    path: String,
    /// The `redirect_uri` to send, the configured one as written.
    pub redirect_uri: String,
}

impl CallbackServer {
    /// Listen where the redirect URL points, [`DEFAULT_REDIRECT_URL`] when
    /// it is empty. Port 0 picks a free port, for providers that allow any
    /// loopback port.
    pub async fn bind(redirect_url: Option<&str>) -> Result<Self, HttpError> {
        let invalid = |details: String| HttpError::new("Invalid OAuth2 redirect URL", details);
        let url = redirect_url
            .map(str::trim)
            .filter(|u| !u.is_empty())
            .unwrap_or(DEFAULT_REDIRECT_URL);

        let mut parsed = url::Url::parse(url).map_err(|e| invalid(e.to_string()))?;
        let host = parsed.host_str().unwrap_or_default().to_string();
        if parsed.scheme() != "http"
            || !matches!(host.as_str(), "localhost" | "127.0.0.1" | "[::1]")
        {
            return Err(invalid(format!(
                "{url} must be a http://localhost or http://127.0.0.1 address so Broquest can receive the callback"
            )));
        }
        let port = parsed.port_or_known_default().unwrap_or(80);
        let bind_host = match host.as_str() {
            "[::1]" => "::1",
            "localhost" => "127.0.0.1",
            other => other,
        };

        let listener = TcpListener::bind((bind_host, port)).await.map_err(|e| {
            HttpError::new(
                "Failed to start callback server",
                format!("Could not listen on {bind_host}:{port} for the redirect ({e}). Set another redirect URL, or one with port 0 if the provider allows any port."),
            )
        })?;
        let addr = listener
            .local_addr()
            .map_err(|e| HttpError::new("Failed to get server address", e.to_string()))?;

        // The URL as written, unless the port was left to the system.
        let redirect_uri = if port == 0 {
            parsed.set_port(Some(addr.port())).ok();
            parsed.to_string()
        } else {
            url.to_string()
        };
        Ok(Self {
            listener,
            path: parsed.path().to_string(),
            redirect_uri,
        })
    }

    /// Wait for the redirect and return its authorization code. Other
    /// requests to the listener, such as for a favicon, are turned away.
    pub async fn wait_for_code(
        self,
        expected_state: &str,
        timeout: Duration,
    ) -> Result<String, HttpError> {
        let wait = async {
            loop {
                let (mut stream, _) =
                    self.listener.accept().await.map_err(|e| {
                        HttpError::new("Failed to accept connection", e.to_string())
                    })?;
                let Some(target) = read_request_target(&mut stream).await else {
                    continue;
                };
                let (path, query) = target.split_once('?').unwrap_or((&target, ""));
                if path != self.path {
                    respond(&mut stream, "404 Not Found", "Not found").await;
                    continue;
                }
                let params: HashMap<String, String> = url::form_urlencoded::parse(query.as_bytes())
                    .into_owned()
                    .collect();

                if let Some(error) = params.get("error") {
                    respond(&mut stream, "200 OK", "Authorization failed").await;
                    return Err(HttpError::new(
                        "OAuth2 authorization failed",
                        params
                            .get("error_description")
                            .cloned()
                            .unwrap_or_else(|| error.clone()),
                    ));
                }
                let Some(code) = params.get("code") else {
                    respond(&mut stream, "400 Bad Request", "No authorization code").await;
                    continue;
                };
                if params.get("state").map(String::as_str) != Some(expected_state) {
                    respond(&mut stream, "400 Bad Request", "Authorization failed").await;
                    return Err(HttpError::new(
                        "OAuth2 authorization failed",
                        "The state in the callback does not match the one sent",
                    ));
                }
                respond(
                    &mut stream,
                    "200 OK",
                    "Authorization successful. You can close this window now.",
                )
                .await;
                return Ok(code.clone());
            }
        };
        let expire = async {
            smol::Timer::after(timeout).await;
            Err(HttpError::new(
                "OAuth2 authorization timed out",
                format!(
                    "No callback within {} minutes",
                    timeout.as_secs().div_ceil(60)
                ),
            ))
        };
        smol::future::or(wait, expire).await
    }
}

/// The request target of an HTTP request, such as `/callback?code=1`. None
/// for a connection that sends nothing, as browsers open ahead of time.
async fn read_request_target(stream: &mut TcpStream) -> Option<String> {
    let mut buffer = Vec::with_capacity(1024);
    let mut chunk = [0u8; 1024];
    while !buffer.windows(4).any(|w| w == b"\r\n\r\n") && buffer.len() < 16 * 1024 {
        let n = stream.read(&mut chunk).await.ok()?;
        if n == 0 {
            break;
        }
        buffer.extend_from_slice(&chunk[..n]);
    }
    let request = String::from_utf8_lossy(&buffer);
    let target = request.lines().next()?.split(' ').nth(1)?;
    Some(target.to_string())
}

async fn respond(stream: &mut TcpStream, status: &str, message: &str) {
    let body = format!(
        "<!doctype html><html><head><meta charset=\"utf-8\"><title>Broquest</title></head>\
         <body style=\"font-family:sans-serif;padding:2rem\"><p>{message}</p></body></html>"
    );
    let response = format!(
        "HTTP/1.1 {status}\r\nContent-Type: text/html; charset=utf-8\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );
    if let Err(e) = stream.write_all(response.as_bytes()).await {
        tracing::error!("Failed to write OAuth callback response: {}", e);
    }
    stream.flush().await.ok();
}

/// Calculate expiration timestamp from expires_in seconds
pub fn calculate_expires_at(expires_in: u64) -> i64 {
    current_unix_timestamp() + expires_in as i64
}

/// Whether broquest can fetch a token for these settings by itself.
pub fn can_fetch(oauth: &OAuth2Auth) -> bool {
    !oauth.token_url.trim().is_empty()
        && matches!(
            oauth.grant_type,
            OAuth2GrantType::ClientCredentials | OAuth2GrantType::AuthorizationCode
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::KeyValuePair;

    #[test]
    fn pkce_challenge_matches_rfc_7636_example() {
        let pkce = Pkce::from_verifier("dBjftJeZ4CVP-mB92K27uhbUJU1p1r_wW1gFWFOEjXk".to_string());
        assert_eq!(
            pkce.challenge,
            "E9Melhoa2OwvFrEMTJguCHaoeK1t8URWbuGJSstw-cM"
        );
    }

    #[test]
    fn random_verifiers_are_long_enough_and_differ() {
        let a = Pkce::new();
        let b = Pkce::new();
        assert_eq!(a.verifier.len(), 43);
        assert_ne!(a.verifier, b.verifier);
    }

    #[test]
    fn authorization_url_has_state_challenge_and_extra_fields() {
        let oauth = OAuth2Auth {
            client_id: "my app".into(),
            scope: Some("openid profile".into()),
            extra_params: vec![KeyValuePair {
                key: "audience".into(),
                value: "https://api".into(),
                enabled: true,
            }],
            ..Default::default()
        };
        let url = build_authorization_url(
            "https://auth/authorize?tenant=1",
            &oauth,
            "http://127.0.0.1:1/cb",
            "xyz",
            Some("abc"),
        );
        assert_eq!(
            url,
            "https://auth/authorize?tenant=1&response_type=code&client_id=my%20app\
             &redirect_uri=http%3A%2F%2F127.0.0.1%3A1%2Fcb&scope=openid%20profile&state=xyz\
             &code_challenge=abc&code_challenge_method=S256&audience=https%3A%2F%2Fapi"
        );
    }

    #[test]
    fn authorization_url_can_skip_encoding() {
        let oauth = OAuth2Auth {
            client_id: "app".into(),
            scope: Some("a b".into()),
            encode_url: false,
            ..Default::default()
        };
        let url =
            build_authorization_url("https://auth/authorize", &oauth, "http://x/cb", "s", None);
        assert_eq!(
            url,
            "https://auth/authorize?response_type=code&client_id=app&redirect_uri=http://x/cb&scope=a b&state=s"
        );
    }

    #[test]
    fn token_response_reads_json_form_and_errors() {
        let json = parse_token_response(
            r#"{"access_token":"a","expires_in":"3600","id_token":"i","refresh_token":"r"}"#,
        )
        .unwrap();
        assert_eq!(json.access_token, "a");
        assert_eq!(json.expires_in, Some(3600));
        assert_eq!(json.id_token.as_deref(), Some("i"));

        let form = parse_token_response("access_token=gho_1&token_type=bearer&scope=").unwrap();
        assert_eq!(form.access_token, "gho_1");

        let error =
            parse_token_response(r#"{"error":"invalid_grant","error_description":"Expired"}"#)
                .unwrap_err();
        assert_eq!(error.details, "Expired");
    }

    #[test]
    fn cache_key_tells_environments_and_token_ids_apart() {
        let oauth = OAuth2Auth {
            token_url: "https://auth/token".into(),
            ..Default::default()
        };
        let named = OAuth2Auth {
            token_id: "admin".into(),
            ..oauth.clone()
        };
        let other_header = OAuth2Auth {
            header_name: "X-Token".into(),
            ..oauth.clone()
        };
        assert_ne!(cache_key("c\ndev", &oauth), cache_key("c\nprod", &oauth));
        assert_ne!(cache_key("c\ndev", &oauth), cache_key("c\ndev", &named));
        assert_eq!(
            cache_key("c\ndev", &oauth),
            cache_key("c\ndev", &other_header)
        );
    }

    #[test]
    fn empty_redirect_url_uses_the_fixed_default() {
        smol::block_on(async {
            match CallbackServer::bind(None).await {
                Ok(server) => assert_eq!(server.redirect_uri, DEFAULT_REDIRECT_URL),
                // Something else on this machine has the port.
                Err(error) => assert!(error.details.contains("41983")),
            }
        });
    }

    #[test]
    fn redirect_url_must_be_local() {
        let error = smol::block_on(CallbackServer::bind(Some("https://example.com/cb")))
            .err()
            .unwrap();
        assert_eq!(error.summary, "Invalid OAuth2 redirect URL");
    }

    #[test]
    fn callback_server_turns_away_other_paths_and_checks_state() {
        smol::block_on(async {
            let server = CallbackServer::bind(Some("http://127.0.0.1:0/callback"))
                .await
                .unwrap();
            let uri = server.redirect_uri.clone();
            let base = uri.trim_end_matches("/callback").to_string();
            let wait = smol::spawn(server.wait_for_code("good", Duration::from_secs(5)));

            let get = |target: String| async move {
                let host = target.trim_start_matches("http://");
                let (authority, path) = host.split_once('/').unwrap();
                let mut stream = TcpStream::connect(authority).await.unwrap();
                let request = format!("GET /{path} HTTP/1.1\r\nHost: {authority}\r\n\r\n");
                stream.write_all(request.as_bytes()).await.unwrap();
                let mut response = String::new();
                stream.read_to_string(&mut response).await.ok();
                response
            };
            assert!(get(format!("{base}/favicon.ico")).await.contains("404"));
            assert!(
                get(format!("{uri}?code=a%2Fb&state=good"))
                    .await
                    .contains("200 OK")
            );
            assert_eq!(wait.await.unwrap(), "a/b");
        });
    }
}
