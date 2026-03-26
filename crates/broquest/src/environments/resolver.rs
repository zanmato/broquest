use crate::collections::{EnvironmentToml, EnvironmentVariable};
use crate::domain::{BasicAuth, DigestAuth, JwtAuth, KeyAuth, OAuth2Auth, RequestData};
use std::collections::HashMap;

/// Environment variable resolver for HTTP requests
#[derive(Clone)]
pub struct EnvironmentResolver {
    // No credential manager needed anymore
}

impl EnvironmentResolver {
    pub fn new() -> Self {
        Self {}
    }

    /// Resolve variables in a string using environment data
    /// Variables are in the format {{variable_name}} or {{secret_name}}
    pub fn resolve_string(
        &self,
        input: &str,
        variables: &HashMap<String, String>,
        secrets: &HashMap<String, String>,
    ) -> String {
        let mut result = input.to_string();

        // First resolve regular variables
        for (key, value) in variables {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        // Then resolve secrets
        for (key, value) in secrets {
            let placeholder = format!("{{{{{}}}}}", key);
            result = result.replace(&placeholder, value);
        }

        result
    }

    /// Load variables and secrets for a collection and environment
    #[allow(clippy::type_complexity)]
    pub fn load_environment_data(
        &self,
        collection_name: &str,
        environment_name: &str,
        environments: &[EnvironmentToml],
        cx: &gpui::App,
    ) -> Result<(HashMap<String, String>, HashMap<String, String>), Box<dyn std::error::Error>>
    {
        let mut variables = HashMap::new();
        let mut secrets = HashMap::new();

        // Find the specified environment
        if let Some(env) = environments.iter().find(|e| e.name == environment_name) {
            // Load variables and secrets from the unified variables map
            for (key, env_var) in &env.variables {
                // Skip temporary variables as they shouldn't be persisted
                if env_var.temporary {
                    continue;
                }

                if env_var.secret {
                    if let Some(secret_value) = EnvironmentVariable::read_credential(
                        collection_name,
                        environment_name,
                        key,
                        cx,
                    )? {
                        secrets.insert(key.clone(), secret_value);
                    }
                } else {
                    variables.insert(key.clone(), env_var.value.clone());
                }
            }
        } else {
            tracing::warn!(
                "Environment '{}' not found in collection '{}'",
                environment_name,
                collection_name
            );
        }

        Ok((variables, secrets))
    }

    /// Resolve all variables in a request data
    pub fn resolve_request_data(
        &self,
        mut request_data: RequestData,
        variables: &HashMap<String, String>,
        secrets: &HashMap<String, String>,
    ) -> RequestData {
        // Resolve URL
        request_data.url = self.resolve_string(&request_data.url, variables, secrets);

        // Resolve headers
        for header in &mut request_data.headers {
            if header.enabled {
                header.key = self.resolve_string(&header.key, variables, secrets);
                header.value = self.resolve_string(&header.value, variables, secrets);
            }
        }

        // Resolve query parameters
        for param in &mut request_data.query_params {
            if param.enabled {
                param.key = self.resolve_string(&param.key, variables, secrets);
                param.value = self.resolve_string(&param.value, variables, secrets);
            }
        }

        // Resolve path parameters
        for param in &mut request_data.path_params {
            if param.enabled {
                param.key = self.resolve_string(&param.key, variables, secrets);
                param.value = self.resolve_string(&param.value, variables, secrets);
            }
        }

        // Resolve body
        request_data.body = self.resolve_string(&request_data.body, variables, secrets);

        // Resolve auth
        request_data.auth = self.resolve_auth(&request_data.auth, variables, secrets);

        request_data
    }

    /// Resolve variables in auth configuration
    pub fn resolve_auth(
        &self,
        auth: &crate::domain::AuthType,
        variables: &HashMap<String, String>,
        secrets: &HashMap<String, String>,
    ) -> crate::domain::AuthType {
        use crate::domain::AuthType;

        match auth {
            AuthType::None | AuthType::Inherit => auth.clone(),
            AuthType::Basic(basic) => AuthType::Basic(BasicAuth {
                username: self.resolve_string(&basic.username, variables, secrets),
                password: self.resolve_string(&basic.password, variables, secrets),
            }),
            AuthType::Digest(digest) => AuthType::Digest(DigestAuth {
                username: self.resolve_string(&digest.username, variables, secrets),
                password: self.resolve_string(&digest.password, variables, secrets),
            }),
            AuthType::Key(key) => AuthType::Key(KeyAuth {
                header: self.resolve_string(&key.header, variables, secrets),
                value: self.resolve_string(&key.value, variables, secrets),
            }),
            AuthType::OAuth2(oauth) => AuthType::OAuth2(OAuth2Auth {
                grant_type: oauth.grant_type.clone(),
                client_id: self.resolve_string(&oauth.client_id, variables, secrets),
                client_secret: self.resolve_string(&oauth.client_secret, variables, secrets),
                token_url: self.resolve_string(&oauth.token_url, variables, secrets),
                scope: oauth
                    .scope
                    .as_ref()
                    .map(|s| self.resolve_string(s, variables, secrets)),
                authorize_url: oauth
                    .authorize_url
                    .as_ref()
                    .map(|s| self.resolve_string(s, variables, secrets)),
                redirect_url: oauth
                    .redirect_url
                    .as_ref()
                    .map(|s| self.resolve_string(s, variables, secrets)),
                access_token: oauth
                    .access_token
                    .as_ref()
                    .map(|s| self.resolve_string(s, variables, secrets)),
                refresh_token: oauth
                    .refresh_token
                    .as_ref()
                    .map(|s| self.resolve_string(s, variables, secrets)),
                expires_at: oauth.expires_at,
            }),
            AuthType::Jwt(jwt) => AuthType::Jwt(JwtAuth {
                login_url: self.resolve_string(&jwt.login_url, variables, secrets),
                username_field: jwt.username_field.clone(),
                username: self.resolve_string(&jwt.username, variables, secrets),
                password_field: jwt.password_field.clone(),
                password: self.resolve_string(&jwt.password, variables, secrets),
                token_field: jwt.token_field.clone(),
                token_type_field: jwt.token_type_field.clone(),
                expiry_field: jwt.expiry_field.clone(),
                access_token: jwt
                    .access_token
                    .as_ref()
                    .map(|s| self.resolve_string(s, variables, secrets)),
                token_type: jwt.token_type.clone(),
                expires_at: jwt.expires_at,
            }),
        }
    }
}

impl Default for EnvironmentResolver {
    fn default() -> Self {
        Self::new()
    }
}
