use crate::collection_types::{EnvironmentToml, EnvironmentVariable};
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
                    // This is a secret - load from secure storage
                    if let Some(secret_value) = EnvironmentVariable::read_credential(
                        collection_name,
                        environment_name,
                        key,
                        cx,
                    )? {
                        secrets.insert(key.clone(), secret_value);
                    }
                } else {
                    // This is a regular variable
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
        mut request_data: crate::request_editor::RequestData,
        variables: &HashMap<String, String>,
        secrets: &HashMap<String, String>,
    ) -> crate::request_editor::RequestData {
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

        request_data
    }
}

impl Default for EnvironmentResolver {
    fn default() -> Self {
        Self::new()
    }
}
