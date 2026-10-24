use crate::collections::{EnvironmentToml, EnvironmentVariable};
use crate::domain::{
    BasicAuth, DigestAuth, JwtAuth, KeyAuth, KeyValuePair, OAuth2Auth, RequestData,
};
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
            if let Some(expr) = name.strip_prefix('$') {
                // Built in {{$...}} variables. Unknown ones keep their placeholder.
                return super::dynamic::resolve(expr);
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
            AuthType::OAuth2(oauth) => {
                let r = |s: &String| {
                    self.resolve_string(s, runtime_vars, collection_vars, variables, secrets)
                };
                let ro = |s: &Option<String>| s.as_ref().map(r);
                AuthType::OAuth2(OAuth2Auth {
                    client_id: r(&oauth.client_id),
                    client_secret: r(&oauth.client_secret),
                    token_url: r(&oauth.token_url),
                    scope: ro(&oauth.scope),
                    authorize_url: ro(&oauth.authorize_url),
                    redirect_url: ro(&oauth.redirect_url),
                    refresh_url: ro(&oauth.refresh_url),
                    extra_params: oauth
                        .extra_params
                        .iter()
                        .map(|p| KeyValuePair {
                            key: r(&p.key),
                            value: r(&p.value),
                            enabled: p.enabled,
                        })
                        .collect(),
                    state: ro(&oauth.state),
                    header_name: r(&oauth.header_name),
                    header_prefix: r(&oauth.header_prefix),
                    id_token_header: r(&oauth.id_token_header),
                    token_id: r(&oauth.token_id),
                    access_token: ro(&oauth.access_token),
                    refresh_token: ro(&oauth.refresh_token),
                    ..oauth.clone()
                })
            }
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

/// How deep variables may refer to other variables before resolving stops.
const MAX_DEPTH: usize = 10;

/// Replace each `{{ name }}` in `input` with `resolve(name)`, keeping the
/// placeholder as written when `resolve` returns `None`.
///
/// Resolved values are resolved in turn, so variables can refer to other
/// variables, up to [`MAX_DEPTH`] deep. A variable that refers back to
/// itself, directly or through others, keeps its placeholder. Names can be
/// built from other variables, as in `{{server_{{env}}_url}}`: the inner
/// placeholder is resolved first. Whitespace inside the braces is trimmed.
fn resolve_placeholders<F: Fn(&str) -> Option<String>>(input: &str, resolve: F) -> String {
    expand(input, &resolve, &mut Vec::new())
}

fn expand(
    input: &str,
    resolve: &dyn Fn(&str) -> Option<String>,
    stack: &mut Vec<String>,
) -> String {
    let bytes = input.as_bytes();
    let opens_at = |i: usize| bytes.get(i) == Some(&b'{') && bytes.get(i + 1) == Some(&b'{');
    let closes_at = |i: usize| bytes.get(i) == Some(&b'}') && bytes.get(i + 1) == Some(&b'}');

    let mut out = String::with_capacity(input.len());
    let mut i = 0;
    while i < bytes.len() {
        if opens_at(i) {
            // Find the matching "}}", counting nested "{{".
            let mut depth = 1;
            let mut j = i + 2;
            let mut close = None;
            while j < bytes.len() {
                if opens_at(j) {
                    depth += 1;
                    j += 2;
                } else if closes_at(j) {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(j);
                        break;
                    }
                    j += 2;
                } else {
                    j += 1;
                }
            }
            if let Some(close) = close {
                let placeholder = &input[i..close + 2];
                let inner = &input[i + 2..close];
                let name = if inner.contains("{{") {
                    expand(inner, resolve, stack)
                } else {
                    inner.to_string()
                };
                let name = name.trim();

                let value = (stack.len() < MAX_DEPTH && !stack.iter().any(|n| n == name))
                    .then(|| resolve(name))
                    .flatten();
                match value {
                    Some(value) => {
                        stack.push(name.to_string());
                        out.push_str(&expand(&value, resolve, stack));
                        stack.pop();
                    }
                    None => out.push_str(placeholder),
                }
                i = close + 2;
                continue;
            }
        }
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
    fn test_resolve_string_unknown_and_dynamic() {
        let resolver = EnvironmentResolver::new();
        let empty = HashMap::new();

        // Unknown names keep their placeholder.
        let s = resolver.resolve_string("a-{{missing}}-b", &empty, &empty, &empty, &empty);
        assert_eq!(s, "a-{{missing}}-b");

        // Built in {{$...}} vars are generated, and win over a plain variable
        // of the same name.
        let env = map(&[("guid", "should-not-win")]);
        let s = resolver.resolve_string("{{ $guid }}", &empty, &empty, &env, &empty);
        assert!(uuid::Uuid::parse_str(&s).is_ok(), "{s}");

        // Unknown built ins keep their placeholder.
        let s = resolver.resolve_string("{{$nope}}", &empty, &empty, &empty, &empty);
        assert_eq!(s, "{{$nope}}");
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

    #[test]
    fn test_resolve_string_nested() {
        let resolver = EnvironmentResolver::new();
        let empty = HashMap::new();
        let env = map(&[
            ("host", "api.example.com"),
            ("baseUrl", "https://{{host}}/v1"),
            ("usersUrl", "{{baseUrl}}/users"),
            ("env", "prod"),
            ("server_prod_url", "https://prod.example.com"),
            ("a", "{{b}}"),
            ("b", "{{a}}"),
            ("self", "x{{self}}"),
        ]);
        let secrets = map(&[("token", "s3cret")]);
        let collection = map(&[("auth", "Bearer {{token}}")]);
        let resolve =
            |input: &str| resolver.resolve_string(input, &empty, &collection, &env, &secrets);

        // Values refer to other values, across buckets too.
        assert_eq!(
            resolve("{{usersUrl}}?q=1"),
            "https://api.example.com/v1/users?q=1"
        );
        assert_eq!(resolve("{{auth}}"), "Bearer s3cret");
        // Names built from other variables.
        assert_eq!(
            resolve("{{server_{{env}}_url}}/health"),
            "https://prod.example.com/health"
        );
        assert_eq!(
            resolve("{{ server_{{ env }}_url }}"),
            "https://prod.example.com"
        );
        // Loops stop and keep the placeholder where they close.
        assert_eq!(resolve("{{a}}"), "{{a}}");
        assert_eq!(resolve("{{self}}"), "x{{self}}");
        // Unknown names inside a built name keep everything.
        assert_eq!(
            resolve("{{server_{{nope}}_url}}"),
            "{{server_{{nope}}_url}}"
        );
        // Unmatched braces are left alone.
        assert_eq!(resolve("{{host"), "{{host");
        assert_eq!(resolve("{{{host}}}"), "{{{host}}}");
    }

    #[test]
    fn test_resolve_string_depth_limit() {
        let resolver = EnvironmentResolver::new();
        let empty = HashMap::new();
        // v0 -> v1 -> ... -> v20, deeper than the limit.
        let pairs: Vec<(String, String)> = (0..20)
            .map(|n| (format!("v{n}"), format!("{{{{v{}}}}}", n + 1)))
            .chain([("v20".to_string(), "end".to_string())])
            .collect();
        let env: HashMap<String, String> = pairs.into_iter().collect();
        let out = resolver.resolve_string("{{v0}}", &empty, &empty, &env, &empty);
        assert_eq!(out, format!("{{{{v{MAX_DEPTH}}}}}"));
    }
}
