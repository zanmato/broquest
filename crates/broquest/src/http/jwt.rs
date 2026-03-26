use crate::domain::JwtAuth;
use serde_json::Value;

use super::{HttpError, current_unix_timestamp, is_token_expired};

pub struct JwtTokenResponse {
    pub access_token: String,
    pub token_type: Option<String>,
    pub expires_at: Option<i64>,
}

/// Fetch JWT token from login endpoint
pub async fn fetch_jwt_token(
    client: &reqwest::Client,
    jwt: &JwtAuth,
) -> Result<JwtTokenResponse, HttpError> {
    let body = serde_json::json!({
        &jwt.username_field: &jwt.username,
        &jwt.password_field: &jwt.password,
    });

    let response = async_compat::Compat::new(client.post(&jwt.login_url).json(&body).send())
        .await
        .map_err(|e| HttpError::new("JWT login request failed", e.to_string()))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = async_compat::Compat::new(response.text())
            .await
            .unwrap_or_default();
        return Err(HttpError::new(
            "JWT login failed",
            format!("Status {}: {}", status, body),
        ));
    }

    let json: Value = async_compat::Compat::new(response.json())
        .await
        .map_err(|e| HttpError::new("Failed to parse JWT response", e.to_string()))?;

    let access_token = extract_string_field(&json, &jwt.token_field).ok_or_else(|| {
        HttpError::new(
            "JWT response missing token",
            format!("Field '{}' not found in response", jwt.token_field),
        )
    })?;

    let token_type = extract_string_field(&json, &jwt.token_type_field);

    let expires_at = extract_expiry(&json, &jwt.expiry_field);

    Ok(JwtTokenResponse {
        access_token,
        token_type,
        expires_at,
    })
}

fn extract_string_field(json: &Value, field: &str) -> Option<String> {
    json.get(field)
        .and_then(|v| v.as_str().map(|s| s.to_string()))
}

/// Extract and parse expiry, auto-detecting format:
/// - Relative seconds (e.g., 3599) -> converted to absolute timestamp
/// - Unix timestamp (e.g., 1709612400) -> used directly
/// - ISO 8601 string (e.g., "2024-03-05T10:00:00Z") -> parsed to timestamp
fn extract_expiry(json: &Value, field: &str) -> Option<i64> {
    let value = json.get(field)?;

    match value {
        Value::Number(n) => {
            let num = n.as_i64()?;
            // Heuristic: values < 100000 are likely relative seconds
            // (100000 seconds ≈ 27.7 hours, typical token lifetimes are 1-24 hours)
            if num < 100_000 {
                Some(current_unix_timestamp() + num)
            } else {
                Some(num)
            }
        }
        Value::String(s) => {
            // Try ISO 8601 parsing
            chrono::DateTime::parse_from_rfc3339(s)
                .ok()
                .map(|dt| dt.timestamp())
        }
        _ => None,
    }
}

/// Check if JWT token is expired or missing
pub fn is_jwt_token_expired(jwt: &JwtAuth) -> bool {
    is_token_expired(jwt.expires_at, jwt.access_token.is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_expiry_relative_seconds() {
        let json = serde_json::json!({"expires_in": 3600});
        let result = extract_expiry(&json, "expires_in");
        assert!(result.is_some());
        let expires_at = result.unwrap();
        let now = current_unix_timestamp();
        assert!(expires_at > now);
        assert!(expires_at < now + 3700);
    }

    #[test]
    fn test_extract_expiry_unix_timestamp() {
        let json = serde_json::json!({"exp": 1709612400});
        let result = extract_expiry(&json, "exp");
        assert_eq!(result, Some(1709612400));
    }

    #[test]
    fn test_extract_expiry_iso_string() {
        let json = serde_json::json!({"expires_at": "2024-03-05T10:00:00Z"});
        let result = extract_expiry(&json, "expires_at");
        assert_eq!(result, Some(1709632800));
    }

    #[test]
    fn test_is_jwt_token_expired_no_token() {
        let jwt = JwtAuth::default();
        assert!(is_jwt_token_expired(&jwt));
    }

    #[test]
    fn test_is_jwt_token_expired_valid_token() {
        let future = current_unix_timestamp() + 3600;
        let jwt = JwtAuth {
            access_token: Some("token".to_string()),
            expires_at: Some(future),
            ..Default::default()
        };
        assert!(!is_jwt_token_expired(&jwt));
    }

    #[test]
    fn test_is_jwt_token_expired_expired_token() {
        let past = current_unix_timestamp() - 100;
        let jwt = JwtAuth {
            access_token: Some("token".to_string()),
            expires_at: Some(past),
            ..Default::default()
        };
        assert!(is_jwt_token_expired(&jwt));
    }
}
