use serde::{Deserialize, Serialize};

use super::KeyValuePair;

/// Main auth type enum representing different authentication methods
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AuthType {
    #[default]
    None,
    Basic(BasicAuth),
    Digest(DigestAuth),
    Key(KeyAuth),
    Inherit,
    /// Saved as `oauth2`. Files from before 0.1.14 say `o_auth2`.
    #[serde(rename = "oauth2", alias = "o_auth2")]
    OAuth2(OAuth2Auth),
    Jwt(JwtAuth),
    /// An OpenCollection auth scheme broquest does not model (e.g. ntlm, awsv4,
    /// oauth1, wsse, api-key-in-query). The original definition is preserved in
    /// `raw` so it round-trips losslessly when the collection is saved back.
    Unsupported {
        kind: String,
        raw: serde_json::Value,
    },
}

impl AuthType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "No Auth",
            Self::Basic(_) => "Basic Auth",
            Self::Digest(_) => "Digest Auth",
            Self::Key(_) => "API Key",
            Self::Inherit => "Inherit from Collection",
            Self::OAuth2(_) => "OAuth2",
            Self::Jwt(_) => "JWT",
            Self::Unsupported { .. } => "Unsupported",
        }
    }

    pub fn has_secrets(&self) -> bool {
        matches!(
            self,
            Self::Basic(_) | Self::Digest(_) | Self::OAuth2(_) | Self::Jwt(_)
        )
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Self::None => "none",
            Self::Basic(_) => "basic",
            Self::Digest(_) => "digest",
            Self::Key(_) => "key",
            Self::Inherit => "inherit",
            Self::OAuth2(_) => "oauth2",
            Self::Jwt(_) => "jwt",
            Self::Unsupported { .. } => "unsupported",
        }
    }

    pub fn all_types() -> Vec<AuthType> {
        vec![
            AuthType::None,
            AuthType::Inherit,
            AuthType::Basic(BasicAuth::default()),
            AuthType::Digest(DigestAuth::default()),
            AuthType::Key(KeyAuth::default()),
            AuthType::OAuth2(OAuth2Auth::default()),
            AuthType::Jwt(JwtAuth::default()),
        ]
    }
}

/// Basic authentication credentials
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct BasicAuth {
    pub username: String,
    pub password: String,
}

/// Digest authentication credentials
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
pub struct DigestAuth {
    pub username: String,
    pub password: String,
}

/// API Key authentication
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KeyAuth {
    pub header: String,
    pub value: String,
}

impl Default for KeyAuth {
    fn default() -> Self {
        Self {
            header: "X-API-Key".to_string(),
            value: String::new(),
        }
    }
}

/// OAuth2 grant types
#[derive(Debug, Default, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuth2GrantType {
    #[default]
    ClientCredentials,
    AuthorizationCode,
    Password,
}

impl OAuth2GrantType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::ClientCredentials => "client_credentials",
            Self::AuthorizationCode => "authorization_code",
            Self::Password => "password",
        }
    }
}

/// How the client ID and secret reach the token endpoint.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuth2ClientAuth {
    /// `client_id` and `client_secret` as form fields.
    #[default]
    Body,
    /// An `Authorization: Basic` header (RFC 6749 section 2.3.1).
    BasicHeader,
}

/// Which token a request sends.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OAuth2TokenKind {
    #[default]
    Access,
    Id,
    /// The access token in the main header and the ID token in its own.
    Both,
}

/// OAuth2 configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuth2Auth {
    pub grant_type: OAuth2GrantType,
    pub client_id: String,
    pub client_secret: String,
    pub token_url: String,
    pub scope: Option<String>,
    pub authorize_url: Option<String>,
    /// Where the authorization server sends the browser back to. Must be a
    /// `http://localhost` or `http://127.0.0.1` address. Empty picks a free
    /// port.
    pub redirect_url: Option<String>,
    /// Where refresh tokens go. Empty uses the token URL.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refresh_url: Option<String>,
    #[serde(default)]
    pub client_auth: OAuth2ClientAuth,
    /// Extra fields for the token request, such as `audience`. They are also
    /// added to the authorization URL.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub extra_params: Vec<KeyValuePair>,
    /// The `state` of the authorization code flow. Empty makes a random one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub state: Option<String>,
    /// PKCE with an S256 code challenge for the authorization code flow.
    #[serde(default)]
    pub pkce: bool,
    /// Percent encode the values put on the authorization URL.
    #[serde(default = "yes")]
    pub encode_url: bool,
    #[serde(default)]
    pub token_to_send: OAuth2TokenKind,
    #[serde(default = "default_header_name")]
    pub header_name: String,
    /// Put before the token, as in `Bearer <token>`. Empty sends the token
    /// alone.
    #[serde(default = "default_header_prefix")]
    pub header_prefix: String,
    /// The header for the ID token when both tokens are sent.
    #[serde(default = "default_id_token_header")]
    pub id_token_header: String,
    /// Fetch a token when a request is sent without a valid one, and fetch a
    /// new one once when the server answers 401.
    #[serde(default = "yes")]
    pub auto_fetch: bool,
    /// Use the refresh token when the access token has expired.
    #[serde(default = "yes")]
    pub auto_refresh: bool,
    /// Requests with the same settings share a token. A token ID keeps a
    /// separate one.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub token_id: String,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_token: Option<String>,
}

impl Default for OAuth2Auth {
    fn default() -> Self {
        Self {
            grant_type: OAuth2GrantType::ClientCredentials,
            client_id: String::new(),
            client_secret: String::new(),
            token_url: String::new(),
            scope: None,
            authorize_url: None,
            redirect_url: None,
            refresh_url: None,
            client_auth: OAuth2ClientAuth::Body,
            extra_params: Vec::new(),
            state: None,
            pkce: false,
            encode_url: true,
            token_to_send: OAuth2TokenKind::Access,
            header_name: default_header_name(),
            header_prefix: default_header_prefix(),
            id_token_header: default_id_token_header(),
            auto_fetch: true,
            auto_refresh: true,
            token_id: String::new(),
            access_token: None,
            refresh_token: None,
            expires_at: None,
            id_token: None,
        }
    }
}

fn yes() -> bool {
    true
}

fn default_header_name() -> String {
    "Authorization".to_string()
}

fn default_header_prefix() -> String {
    "Bearer".to_string()
}

fn default_id_token_header() -> String {
    "X-ID-Token".to_string()
}

fn default_token_type_field() -> String {
    "token_type".to_string()
}

fn default_expiry_field() -> String {
    "expires_in".to_string()
}

/// JWT authentication configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JwtAuth {
    pub login_url: String,
    pub username_field: String,
    pub username: String,
    pub password_field: String,
    pub password: String,
    pub token_field: String,
    #[serde(default = "default_token_type_field")]
    pub token_type_field: String,
    #[serde(default = "default_expiry_field")]
    pub expiry_field: String,
    pub access_token: Option<String>,
    pub token_type: Option<String>,
    pub expires_at: Option<i64>,
}

impl Default for JwtAuth {
    fn default() -> Self {
        Self {
            login_url: String::new(),
            username_field: "username".to_string(),
            username: String::new(),
            password_field: "password".to_string(),
            password: String::new(),
            token_field: "access_token".to_string(),
            token_type_field: default_token_type_field(),
            expiry_field: default_expiry_field(),
            access_token: None,
            token_type: None,
            expires_at: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_auth_type_serialization() {
        let auth = AuthType::Basic(BasicAuth {
            username: "test".to_string(),
            password: "secret".to_string(),
        });
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains(r#""type":"basic"#));
        assert!(json.contains(r#""username":"test"#));
    }

    #[test]
    fn test_auth_type_deserialization() {
        let json = r#"{"type":"basic","username":"test","password":"secret"}"#;
        let auth: AuthType = serde_json::from_str(json).unwrap();
        match auth {
            AuthType::Basic(basic) => {
                assert_eq!(basic.username, "test");
                assert_eq!(basic.password, "secret");
            }
            _ => panic!("Expected Basic auth"),
        }
    }

    #[test]
    fn oauth2_is_saved_as_oauth2_and_reads_the_old_name() {
        let json = serde_json::to_string(&AuthType::OAuth2(OAuth2Auth::default())).unwrap();
        assert!(json.contains(r#""type":"oauth2""#));

        let old: AuthType =
            serde_json::from_str(r#"{"type":"o_auth2","grant_type":"client_credentials","client_id":"c","client_secret":"","token_url":"t","scope":null,"authorize_url":null,"redirect_url":null,"access_token":null,"refresh_token":null}"#)
                .unwrap();
        assert!(matches!(old, AuthType::OAuth2(ref o) if o.client_id == "c"));

        let toml_old: AuthType =
            toml::from_str("type = \"o_auth2\"\ngrant_type = \"client_credentials\"\nclient_id = \"c\"\nclient_secret = \"\"\ntoken_url = \"t\"\n")
                .unwrap();
        assert!(matches!(toml_old, AuthType::OAuth2(_)));
    }

    #[test]
    fn test_auth_type_none_default() {
        let auth = AuthType::default();
        assert!(matches!(auth, AuthType::None));
    }

    #[test]
    fn test_jwt_auth_serialization() {
        let auth = AuthType::Jwt(JwtAuth {
            login_url: "https://api.example.com/login".to_string(),
            username_field: "email".to_string(),
            username: "user@example.com".to_string(),
            password_field: "pass".to_string(),
            password: "secret".to_string(),
            token_field: "token".to_string(),
            token_type_field: "type".to_string(),
            expiry_field: "exp".to_string(),
            access_token: Some("jwt-token-123".to_string()),
            token_type: Some("Bearer".to_string()),
            expires_at: Some(1709632800),
        });
        let json = serde_json::to_string(&auth).unwrap();
        assert!(json.contains(r#""type":"jwt"#));
        assert!(json.contains(r#""login_url":"https://api.example.com/login"#));
        assert!(json.contains(r#""username":"user@example.com"#));
    }

    #[test]
    fn test_jwt_auth_deserialization() {
        let json = r#"{"type":"jwt","login_url":"https://api.example.com/login","username_field":"email","username":"user@example.com","password_field":"pass","password":"secret","token_field":"token","token_type_field":"type","expiry_field":"exp","access_token":"jwt-token-123","token_type":"Bearer","expires_at":1709632800}"#;
        let auth: AuthType = serde_json::from_str(json).unwrap();
        match auth {
            AuthType::Jwt(jwt) => {
                assert_eq!(jwt.login_url, "https://api.example.com/login");
                assert_eq!(jwt.username_field, "email");
                assert_eq!(jwt.username, "user@example.com");
                assert_eq!(jwt.access_token, Some("jwt-token-123".to_string()));
            }
            _ => panic!("Expected Jwt auth"),
        }
    }

    #[test]
    fn test_jwt_auth_default_fields() {
        let json = r#"{"type":"jwt","login_url":"https://api.example.com/login","username_field":"username","username":"test","password_field":"password","password":"pass","token_field":"access_token"}"#;
        let auth: AuthType = serde_json::from_str(json).unwrap();
        match auth {
            AuthType::Jwt(jwt) => {
                assert_eq!(jwt.token_type_field, "token_type");
                assert_eq!(jwt.expiry_field, "expires_in");
                assert_eq!(jwt.access_token, None);
            }
            _ => panic!("Expected Jwt auth"),
        }
    }
}
