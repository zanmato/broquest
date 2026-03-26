use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Thread-safe storage for script environment variables
/// Environment variables are temporary during script execution but can be marked as dirty
/// to update the persistent environment storage
#[derive(Debug, Clone)]
pub struct VariableStore {
    /// Single mutex protecting both env_vars and dirty_flags to prevent race conditions
    data: Arc<Mutex<VariableStoreData>>,
}

#[derive(Debug, Default)]
struct VariableStoreData {
    env_vars: HashMap<String, Value>,
    dirty_flags: HashMap<String, bool>,
}

impl VariableStore {
    /// Create a new variable store
    pub fn new() -> Self {
        Self {
            data: Arc::new(Mutex::new(VariableStoreData::default())),
        }
    }

    /// Set an environment variable (temporary, script-scoped) and mark as dirty
    pub fn set_env_var(&self, name: &str, value: Value) {
        if let Ok(mut data) = self.data.lock() {
            data.env_vars.insert(name.to_string(), value);
            data.dirty_flags.insert(name.to_string(), true);
        }
    }

    /// Get an environment variable
    pub fn get_env_var(&self, name: &str) -> Option<Value> {
        self.data
            .lock()
            .ok()
            .and_then(|data| data.env_vars.get(name).cloned())
    }

    /// Get all environment variables as a HashMap copy
    #[allow(dead_code)]
    pub fn get_all_env_vars(&self) -> HashMap<String, Value> {
        self.data
            .lock()
            .map(|data| data.env_vars.clone())
            .unwrap_or_default()
    }

    /// Get dirty environment variables that need to be persisted
    pub fn get_dirty_env_vars(&self) -> HashMap<String, String> {
        let mut dirty_vars = HashMap::new();
        if let Ok(data) = self.data.lock() {
            for (name, is_dirty) in data.dirty_flags.iter() {
                if *is_dirty && let Some(value) = data.env_vars.get(name) {
                    if let Some(str_value) = value.as_str() {
                        dirty_vars.insert(name.clone(), str_value.to_string());
                    } else {
                        dirty_vars.insert(name.clone(), value.to_string());
                    }
                }
            }
        }
        dirty_vars
    }

    /// Set an environment variable using a string (convenience method for JavaScript integration) and mark as dirty
    pub fn set_env_var_str(&self, name: &str, value: &str) {
        self.set_env_var(name, Value::String(value.to_string()));
    }

    /// Get an environment variable as a string (convenience method for JavaScript integration)
    pub fn get_env_var_str(&self, name: &str) -> Option<String> {
        self.get_env_var(name)
            .and_then(|v| v.as_str().map(|s| s.to_string()))
    }

    /// Initialize the variable store with environment data (not marked as dirty)
    pub fn initialize_with_env(
        &self,
        variables: &HashMap<String, String>,
        secrets: &HashMap<String, String>,
    ) {
        if let Ok(mut data) = self.data.lock() {
            // Add regular variables
            for (key, value) in variables {
                data.env_vars
                    .insert(key.clone(), Value::String(value.clone()));
            }
            // Add secrets
            for (key, value) in secrets {
                data.env_vars
                    .insert(key.clone(), Value::String(value.clone()));
            }
        }
    }
}

impl Default for VariableStore {
    fn default() -> Self {
        Self::new()
    }
}
