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

    /// Resolve variables in a string.
    ///
    /// Variables are in the format `{{variable_name}}`. Resolution precedence
    /// (highest to lowest) mirrors Bruno: **runtime > collection > environment >
    /// secret**. Each placeholder is scanned in a single pass and resolved
    /// against the highest-precedence bucket that contains its name.
    pub fn resolve_string(
        &self,
        input: &str,
        runtime_vars: &HashMap<String, String>,
        collection_vars: &HashMap<String, String>,
        variables: &HashMap<String, String>,
        secrets: &HashMap<String, String>,
    ) -> String {
        resolve_placeholders(input, |name| {
            if name.starts_with('$') {
                // Dynamic {{$...}} variables are left untouched.
                return None;
            }
            runtime_vars
                .get(name)
                .or_else(|| collection_vars.get(name))
                .or_else(|| variables.get(name))
                .or_else(|| secrets.get(name))
                .cloned()
        })
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
        runtime_vars: &HashMap<String, String>,
        collection_vars: &HashMap<String, String>,
        variables: &HashMap<String, String>,
        secrets: &HashMap<String, String>,
    ) -> RequestData {
        // Resolve URL
        request_data.url = self.resolve_string(
            &request_data.url,
            runtime_vars,
            collection_vars,
            variables,
            secrets,
        );

        // Resolve headers
        for header in &mut request_data.headers {
            if header.enabled {
                header.key = self.resolve_string(
                    &header.key,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                );
                header.value = self.resolve_string(
                    &header.value,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                );
            }
        }

        // Resolve query parameters
        for param in &mut request_data.query_params {
            if param.enabled {
                param.key = self.resolve_string(
                    &param.key,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                );
                param.value = self.resolve_string(
                    &param.value,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                );
            }
        }

        // Resolve path parameters
        for param in &mut request_data.path_params {
            if param.enabled {
                param.key = self.resolve_string(
                    &param.key,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                );
                param.value = self.resolve_string(
                    &param.value,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                );
            }
        }

        // Resolve body
        request_data.body = self.resolve_string(
            &request_data.body,
            runtime_vars,
            collection_vars,
            variables,
            secrets,
        );

        // Resolve auth
        request_data.auth = self.resolve_auth(
            &request_data.auth,
            runtime_vars,
            collection_vars,
            variables,
            secrets,
        );

        request_data
    }

    /// Resolve variables in auth configuration
    pub fn resolve_auth(
        &self,
        auth: &crate::domain::AuthType,
        runtime_vars: &HashMap<String, String>,
        collection_vars: &HashMap<String, String>,
        variables: &HashMap<String, String>,
        secrets: &HashMap<String, String>,
    ) -> crate::domain::AuthType {
        use crate::domain::AuthType;

        match auth {
            AuthType::None | AuthType::Inherit | AuthType::Unsupported { .. } => auth.clone(),
            AuthType::Basic(basic) => AuthType::Basic(BasicAuth {
                username: self.resolve_string(
                    &basic.username,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                ),
                password: self.resolve_string(
                    &basic.password,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                ),
            }),
            AuthType::Digest(digest) => AuthType::Digest(DigestAuth {
                username: self.resolve_string(
                    &digest.username,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                ),
                password: self.resolve_string(
                    &digest.password,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                ),
            }),
            AuthType::Key(key) => AuthType::Key(KeyAuth {
                header: self.resolve_string(
                    &key.header,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                ),
                value: self.resolve_string(
                    &key.value,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                ),
            }),
            AuthType::OAuth2(oauth) => AuthType::OAuth2(OAuth2Auth {
                grant_type: oauth.grant_type.clone(),
                client_id: self.resolve_string(
                    &oauth.client_id,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                ),
                client_secret: self.resolve_string(
                    &oauth.client_secret,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                ),
                token_url: self.resolve_string(
                    &oauth.token_url,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                ),
                scope: oauth.scope.as_ref().map(|s| {
                    self.resolve_string(s, runtime_vars, collection_vars, variables, secrets)
                }),
                authorize_url: oauth.authorize_url.as_ref().map(|s| {
                    self.resolve_string(s, runtime_vars, collection_vars, variables, secrets)
                }),
                redirect_url: oauth.redirect_url.as_ref().map(|s| {
                    self.resolve_string(s, runtime_vars, collection_vars, variables, secrets)
                }),
                access_token: oauth.access_token.as_ref().map(|s| {
                    self.resolve_string(s, runtime_vars, collection_vars, variables, secrets)
                }),
                refresh_token: oauth.refresh_token.as_ref().map(|s| {
                    self.resolve_string(s, runtime_vars, collection_vars, variables, secrets)
                }),
                expires_at: oauth.expires_at,
            }),
            AuthType::Jwt(jwt) => AuthType::Jwt(JwtAuth {
                login_url: self.resolve_string(
                    &jwt.login_url,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                ),
                username_field: jwt.username_field.clone(),
                username: self.resolve_string(
                    &jwt.username,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                ),
                password_field: jwt.password_field.clone(),
                password: self.resolve_string(
                    &jwt.password,
                    runtime_vars,
                    collection_vars,
                    variables,
                    secrets,
                ),
                token_field: jwt.token_field.clone(),
                token_type_field: jwt.token_type_field.clone(),
                expiry_field: jwt.expiry_field.clone(),
                access_token: jwt.access_token.as_ref().map(|s| {
                    self.resolve_string(s, runtime_vars, collection_vars, variables, secrets)
                }),
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

/// Scan `input` for `{{ name }}` placeholders and replace each with the value
/// returned by `resolve(name)`, preserving the original placeholder text when
/// the resolver returns `None`. Whitespace inside the braces is trimmed. This
/// is a single-pass scan so resolved values are not re-scanned, giving correct
/// per-name precedence (higher-precedence buckets are consulted first by the
/// caller's resolver closure).
fn resolve_placeholders<F: Fn(&str) -> Option<String>>(input: &str, resolve: F) -> String {
    let bytes = input.as_bytes();
    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'{' && i + 1 < bytes.len() && bytes[i + 1] == b'{' {
            // Find the matching closing "}}".
            if let Some(rel_end) = input[i + 2..].find("}}") {
                let inner = &input[i + 2..i + 2 + rel_end];
                let name = inner.trim();
                match resolve(name) {
                    Some(value) => {
                        out.push_str(&value);
                    }
                    None => {
                        // Preserve the original placeholder verbatim
                        // (including its original surrounding whitespace).
                        out.push_str(&input[i..i + 2 + rel_end + 2]);
                    }
                }
                i = i + 2 + rel_end + 2;
                continue;
            }
        }
        // No placeholder start; copy this char as a UTF-8 boundary. The loop
        // condition guarantees `i < len`, but guard the slice defensively rather
        // than panicking if that ever changes.
        let Some(ch) = input[i..].chars().next() else {
            break;
        };
        out.push(ch);
        i += ch.len_utf8();
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn map(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn test_resolve_string_precedence() {
        let resolver = EnvironmentResolver::new();
        let runtime = map(&[("x", "runtime-x"), ("only_runtime", "rt")]);
        let collection = map(&[("x", "collection-x"), ("only_collection", "col")]);
        let env = map(&[("x", "env-x"), ("only_env", "env")]);
        let secrets = map(&[("secret", "shh")]);

        // runtime beats collection beats env
        let s = resolver.resolve_string("{{x}}", &runtime, &collection, &env, &secrets);
        assert_eq!(s, "runtime-x");

        // collection beats env when no runtime value
        let s =
            resolver.resolve_string("{{only_collection}}", &runtime, &collection, &env, &secrets);
        assert_eq!(s, "col");

        // env used when not in runtime/collection
        let s = resolver.resolve_string("{{only_env}}", &runtime, &collection, &env, &secrets);
        assert_eq!(s, "env");

        // secrets still resolve as the lowest tier
        let s = resolver.resolve_string("{{secret}}", &runtime, &collection, &env, &secrets);
        assert_eq!(s, "shh");
    }

    #[test]
    fn test_resolve_string_preserves_unknown_and_dynamic() {
        let resolver = EnvironmentResolver::new();
        let empty = HashMap::new();

        // Unknown names keep their placeholder.
        let s = resolver.resolve_string("a-{{missing}}-b", &empty, &empty, &empty, &empty);
        assert_eq!(s, "a-{{missing}}-b");

        // Dynamic {{$...}} vars are left untouched even if they share a name.
        let env = map(&[("guid", "should-not-win")]);
        let s = resolver.resolve_string("{{$guid}}", &empty, &empty, &env, &empty);
        assert_eq!(s, "{{$guid}}");
    }

    #[test]
    fn test_resolve_string_multiple_placeholders() {
        let resolver = EnvironmentResolver::new();
        let runtime = map(&[("a", "1")]);
        let collection = map(&[("b", "2")]);
        let env = map(&[("c", "3")]);
        let empty = HashMap::new();

        let s = resolver.resolve_string(
            "{{a}}/{{b}}/{{c}}/{{a}}{{b}}",
            &runtime,
            &collection,
            &env,
            &empty,
        );
        assert_eq!(s, "1/2/3/12");
    }
}
