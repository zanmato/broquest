//! End-to-end integration tests for auth application in HTTP requests
//!
//! These tests use wiremock to spin up a mock HTTP server and verify
//! that the correct authentication headers are sent with requests.

use crate::domain::{
    AuthType, BasicAuth, DigestAuth, HttpMethod, JwtAuth, KeyAuth, OAuth2Auth, OAuth2GrantType,
    RequestData,
};
use crate::http::HttpClientService;
use serde_json::json;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use wiremock::matchers::{body_string_contains, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn test_basic_auth_sends_correct_header() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .and(header("Authorization", "Basic dXNlcjpwYXNz"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/test", mock_server.uri()),
        auth: AuthType::Basic(BasicAuth {
            username: "user".to_string(),
            password: "pass".to_string(),
        }),
        ..Default::default()
    };

    let result = client.send_request(request_data, None, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_api_key_auth_sends_custom_header() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .and(header("X-Custom-Auth", "my-secret-key-123"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/test", mock_server.uri()),
        auth: AuthType::Key(KeyAuth {
            header: "X-Custom-Auth".to_string(),
            value: "my-secret-key-123".to_string(),
        }),
        ..Default::default()
    };

    let result = client.send_request(request_data, None, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_oauth2_sends_bearer_token() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .and(header("Authorization", "Bearer my-access-token-xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/test", mock_server.uri()),
        auth: AuthType::OAuth2(OAuth2Auth {
            grant_type: OAuth2GrantType::ClientCredentials,
            access_token: Some("my-access-token-xyz".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = client.send_request(request_data, None, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_oauth2_no_token_sends_no_auth_header() {
    let mock_server = MockServer::start().await;

    let guard = Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount_as_scoped(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/test", mock_server.uri()),
        auth: AuthType::OAuth2(OAuth2Auth {
            grant_type: OAuth2GrantType::ClientCredentials,
            access_token: None,
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = client.send_request(request_data, None, None).await;
    assert!(result.is_ok());

    let received = guard.received_requests().await;
    assert!(!received.is_empty());
    assert!(received.iter().all(|r| {
        !r.headers
            .iter()
            .any(|(name, _)| name.as_str().to_lowercase() == "authorization")
    }));
}

#[tokio::test]
async fn test_none_auth_sends_no_auth_header() {
    let mock_server = MockServer::start().await;

    let guard = Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount_as_scoped(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/test", mock_server.uri()),
        auth: AuthType::None,
        ..Default::default()
    };

    let result = client.send_request(request_data, None, None).await;
    assert!(result.is_ok());

    let received = guard.received_requests().await;
    assert!(!received.is_empty());
    assert!(received.iter().all(|r| {
        !r.headers
            .iter()
            .any(|(name, _)| name.as_str().to_lowercase() == "authorization")
    }));
}

#[tokio::test]
async fn test_inherit_auth_sends_no_auth_header() {
    let mock_server = MockServer::start().await;

    let guard = Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount_as_scoped(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/test", mock_server.uri()),
        auth: AuthType::Inherit,
        ..Default::default()
    };

    let result = client.send_request(request_data, None, None).await;
    assert!(result.is_ok());

    let received = guard.received_requests().await;
    assert!(!received.is_empty());
    assert!(received.iter().all(|r| {
        !r.headers
            .iter()
            .any(|(name, _)| name.as_str().to_lowercase() == "authorization")
    }));
}

#[tokio::test]
async fn test_digest_auth_challenge_response() {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use wiremock::Respond;

    struct DigestResponder {
        request_count: Arc<AtomicUsize>,
    }

    impl Respond for DigestResponder {
        fn respond(&self, request: &wiremock::Request) -> ResponseTemplate {
            let count = self.request_count.fetch_add(1, Ordering::SeqCst);

            if count == 0 {
                // First request: return 401 with Digest challenge
                ResponseTemplate::new(401)
                    .insert_header("WWW-Authenticate", "Digest realm=\"testrealm@host.com\", nonce=\"dcd98b7102dd2f0e8b11d0f600bfb0c093\", qop=\"auth\"")
            } else {
                // Second request: verify Authorization header exists and return 200
                let has_auth = request
                    .headers
                    .iter()
                    .any(|(name, _)| name.as_str().to_lowercase() == "authorization");
                if has_auth {
                    ResponseTemplate::new(200).set_body_string("OK")
                } else {
                    ResponseTemplate::new(401).set_body_string("Missing auth header")
                }
            }
        }
    }

    let mock_server = MockServer::start().await;
    let request_count = Arc::new(AtomicUsize::new(0));
    let responder = DigestResponder {
        request_count: request_count.clone(),
    };

    Mock::given(method("GET"))
        .and(path("/test"))
        .respond_with(responder)
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/test", mock_server.uri()),
        auth: AuthType::Digest(DigestAuth {
            username: "user".to_string(),
            password: "password".to_string(),
        }),
        ..Default::default()
    };

    let result = client.send_request(request_data, None, None).await;
    assert!(result.is_ok());

    let (response_data, _) = result.unwrap();
    assert_eq!(response_data.status_code, Some(200));
    assert_eq!(response_data.body, "OK");
}

#[tokio::test]
async fn test_digest_auth_no_challenge_returns_initial_response() {
    let mock_server = MockServer::start().await;

    // Server returns 200 without requiring auth
    Mock::given(method("GET"))
        .and(path("/public"))
        .respond_with(ResponseTemplate::new(200).set_body_string("Public content"))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/public", mock_server.uri()),
        auth: AuthType::Digest(DigestAuth {
            username: "user".to_string(),
            password: "password".to_string(),
        }),
        ..Default::default()
    };

    let result = client.send_request(request_data, None, None).await;
    assert!(result.is_ok());

    let (response_data, _) = result.unwrap();
    assert_eq!(response_data.status_code, Some(200));
    assert_eq!(response_data.body, "Public content");
}

#[tokio::test]
async fn test_auth_with_environment_variables() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .and(header("X-API-Key", "resolved-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/test", mock_server.uri()),
        auth: AuthType::Key(KeyAuth {
            header: "X-API-Key".to_string(),
            value: "{{api_key}}".to_string(),
        }),
        ..Default::default()
    };

    let mut variables = HashMap::new();
    variables.insert("api_key".to_string(), "resolved-api-key".to_string());

    let result = client
        .send_request(request_data, Some(variables), None)
        .await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_auth_with_secret_variables() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/test"))
        .and(header("Authorization", "Basic dXNlcjpzZWNyZXQtdG9rZW4="))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/test", mock_server.uri()),
        auth: AuthType::Basic(BasicAuth {
            username: "user".to_string(),
            password: "{{password}}".to_string(),
        }),
        ..Default::default()
    };

    let mut secrets = HashMap::new();
    secrets.insert("password".to_string(), "secret-token".to_string());

    let result = client.send_request(request_data, None, Some(secrets)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_post_request_with_auth() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/users"))
        .and(header("Authorization", "Bearer create-token"))
        .respond_with(ResponseTemplate::new(201).set_body_string("Created"))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Post,
        url: format!("{}/users", mock_server.uri()),
        auth: AuthType::OAuth2(OAuth2Auth {
            access_token: Some("create-token".to_string()),
            ..Default::default()
        }),
        body: r#"{"name":"test"}"#.to_string(),
        ..Default::default()
    };

    let result = client.send_request(request_data, None, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_oauth2_client_credentials_fetches_token() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .and(body_string_contains("grant_type=client_credentials"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "auto-fetched-token",
            "token_type": "Bearer",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let mut oauth = OAuth2Auth {
        grant_type: OAuth2GrantType::ClientCredentials,
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        token_url: format!("{}/oauth/token", mock_server.uri()),
        access_token: None,
        ..Default::default()
    };

    client
        .ensure_oauth2_token(&mut oauth, |_| {})
        .await
        .unwrap();
    assert_eq!(oauth.access_token, Some("auto-fetched-token".to_string()));
    assert!(oauth.expires_at.is_some());
}

#[tokio::test]
async fn test_oauth2_uses_existing_token() {
    let client = HttpClientService::new(30);

    let mut oauth = OAuth2Auth {
        access_token: Some("existing-token".to_string()),
        token_url: "http://should-not-be-called".to_string(),
        ..Default::default()
    };

    client
        .ensure_oauth2_token(&mut oauth, |_| {})
        .await
        .unwrap();
    assert_eq!(oauth.access_token, Some("existing-token".to_string()));
}

#[tokio::test]
async fn test_oauth2_token_error_returns_meaningful_message() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "invalid_client",
            "error_description": "Client authentication failed"
        })))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let mut oauth = OAuth2Auth {
        grant_type: OAuth2GrantType::ClientCredentials,
        client_id: "bad-client".to_string(),
        client_secret: "bad-secret".to_string(),
        token_url: format!("{}/oauth/token", mock_server.uri()),
        access_token: None,
        ..Default::default()
    };

    let result = client.ensure_oauth2_token(&mut oauth, |_| {}).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.details.contains("401") || err.details.contains("invalid_client"));
}

#[tokio::test]
async fn test_oauth2_refresh_token_stored() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/oauth/token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "new-access-token",
            "token_type": "Bearer",
            "refresh_token": "refresh-token-123",
            "expires_in": 7200
        })))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let mut oauth = OAuth2Auth {
        grant_type: OAuth2GrantType::ClientCredentials,
        client_id: "test-client".to_string(),
        client_secret: "test-secret".to_string(),
        token_url: format!("{}/oauth/token", mock_server.uri()),
        access_token: None,
        ..Default::default()
    };

    client
        .ensure_oauth2_token(&mut oauth, |_| {})
        .await
        .unwrap();
    assert_eq!(oauth.access_token, Some("new-access-token".to_string()));
    assert_eq!(oauth.refresh_token, Some("refresh-token-123".to_string()));
}

#[tokio::test]
async fn test_jwt_fetches_token_from_login_endpoint() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/login"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": "jwt-token-123",
            "token_type": "bearer",
            "expires_in": 3600
        })))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let mut jwt = JwtAuth {
        login_url: format!("{}/auth/login", mock_server.uri()),
        username_field: "username".to_string(),
        username: "testuser".to_string(),
        password_field: "password".to_string(),
        password: "testpass".to_string(),
        ..Default::default()
    };

    client.ensure_jwt_token(&mut jwt).await.unwrap();
    assert_eq!(jwt.access_token, Some("jwt-token-123".to_string()));
    assert_eq!(jwt.token_type, Some("bearer".to_string()));
    assert!(jwt.expires_at.is_some());
}

#[tokio::test]
async fn test_jwt_uses_custom_field_names() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/login"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "token": "custom-token",
            "type": "Bearer",
            "exp": 1709612400
        })))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let mut jwt = JwtAuth {
        login_url: format!("{}/auth/login", mock_server.uri()),
        username_field: "email".to_string(),
        username: "user@example.com".to_string(),
        password_field: "pass".to_string(),
        password: "secret".to_string(),
        token_field: "token".to_string(),
        token_type_field: "type".to_string(),
        expiry_field: "exp".to_string(),
        ..Default::default()
    };

    client.ensure_jwt_token(&mut jwt).await.unwrap();
    assert_eq!(jwt.access_token, Some("custom-token".to_string()));
    assert_eq!(jwt.expires_at, Some(1709612400));
}

#[tokio::test]
async fn test_jwt_reuses_valid_token() {
    let client = HttpClientService::new(30);

    let future = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
        + 3600;

    let mut jwt = JwtAuth {
        access_token: Some("existing-token".to_string()),
        expires_at: Some(future),
        login_url: "http://should-not-be-called".to_string(),
        ..Default::default()
    };

    client.ensure_jwt_token(&mut jwt).await.unwrap();
    assert_eq!(jwt.access_token, Some("existing-token".to_string()));
}

#[tokio::test]
async fn test_jwt_sends_bearer_token() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/data"))
        .and(header("Authorization", "Bearer jwt-token-xyz"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/api/data", mock_server.uri()),
        auth: AuthType::Jwt(JwtAuth {
            access_token: Some("jwt-token-xyz".to_string()),
            token_type: Some("Bearer".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = client.send_request(request_data, None, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jwt_sends_custom_token_type() {
    let mock_server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/api/data"))
        .and(header("Authorization", "CustomType custom-token"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/api/data", mock_server.uri()),
        auth: AuthType::Jwt(JwtAuth {
            access_token: Some("custom-token".to_string()),
            token_type: Some("CustomType".to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = client.send_request(request_data, None, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_jwt_no_token_sends_no_auth_header() {
    let mock_server = MockServer::start().await;

    let guard = Mock::given(method("GET"))
        .and(path("/api/data"))
        .respond_with(ResponseTemplate::new(200).set_body_string("OK"))
        .mount_as_scoped(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let request_data = RequestData {
        method: HttpMethod::Get,
        url: format!("{}/api/data", mock_server.uri()),
        auth: AuthType::Jwt(JwtAuth {
            access_token: None,
            ..Default::default()
        }),
        ..Default::default()
    };

    let result = client.send_request(request_data, None, None).await;
    assert!(result.is_ok());

    let received = guard.received_requests().await;
    assert!(!received.is_empty());
    assert!(received.iter().all(|r| {
        !r.headers
            .iter()
            .any(|(name, _)| name.as_str().to_lowercase() == "authorization")
    }));
}

#[tokio::test]
async fn test_jwt_login_error_returns_meaningful_message() {
    let mock_server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/auth/login"))
        .respond_with(ResponseTemplate::new(401).set_body_json(json!({
            "error": "invalid_credentials",
            "message": "Invalid username or password"
        })))
        .mount(&mock_server)
        .await;

    let client = HttpClientService::new(30);

    let mut jwt = JwtAuth {
        login_url: format!("{}/auth/login", mock_server.uri()),
        username: "baduser".to_string(),
        password: "badpass".to_string(),
        ..Default::default()
    };

    let result = client.ensure_jwt_token(&mut jwt).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(err.details.contains("401") || err.details.contains("invalid_credentials"));
}
