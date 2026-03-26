use serde::{Deserialize, Serialize};

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
    OAuth2(OAuth2Auth),
    Jwt(JwtAuth),
}

impl AuthType {
    pub fn name(&self) -> &'static str {
        match self {
            Self::None => "No Auth",
            Self::Basic(_) => "Basic Auth",
            Self::Digest(_) => "Digest Auth",
            Self::Key(_) => "API Key",
            Self::Inherit => "Inherit from Collection",
            Self::OAuth2(_) => "OAuth2 Client Credentials",
            Self::Jwt(_) => "JWT",
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

/// OAuth2 configuration
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OAuth2Auth {
    pub grant_type: OAuth2GrantType,
    pub client_id: String,
    pub client_secret: String,
    pub token_url: String,
    pub scope: Option<String>,
    pub authorize_url: Option<String>,
    pub redirect_url: Option<String>,
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    #[serde(default)]
    pub expires_at: Option<i64>,
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
            access_token: None,
            refresh_token: None,
            expires_at: None,
        }
    }
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
