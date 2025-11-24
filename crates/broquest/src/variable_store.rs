use serde_json::Value;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Thread-safe storage for script environment variables
/// Environment variables are temporary during script execution but can be marked as dirty
/// to update the persistent environment storage
#[derive(Debug, Clone)]
pub struct VariableStore {
    env_vars: Arc<Mutex<HashMap<String, Value>>>,
    dirty_vars: Arc<Mutex<HashMap<String, bool>>>,
}

impl VariableStore {
    /// Create a new variable store
    pub fn new() -> Self {
        Self {
            env_vars: Arc::new(Mutex::new(HashMap::new())),
            dirty_vars: Arc::new(Mutex::new(HashMap::new())),
        }
    }

  
    /// Set an environment variable (temporary, script-scoped) and mark as dirty
    pub fn set_env_var(&self, name: &str, value: Value) {
        if let Ok(mut vars) = self.env_vars.lock() {
            vars.insert(name.to_string(), value.clone());
        }
        if let Ok(mut dirty) = self.dirty_vars.lock() {
            dirty.insert(name.to_string(), true);
        }
    }

    /// Get an environment variable
    pub fn get_env_var(&self, name: &str) -> Option<Value> {
        self.env_vars
            .lock()
            .ok()
            .and_then(|vars| vars.get(name).cloned())
    }

    /// Get all environment variables as a HashMap copy
    #[allow(dead_code)]
    pub fn get_all_env_vars(&self) -> HashMap<String, Value> {
        self.env_vars
            .lock()
            .map(|vars| vars.clone())
            .unwrap_or_default()
    }

    /// Get dirty environment variables that need to be persisted
    pub fn get_dirty_env_vars(&self) -> HashMap<String, String> {
        let mut dirty_vars = HashMap::new();
        if let Ok(env_vars) = self.env_vars.lock()
            && let Ok(dirty_flags) = self.dirty_vars.lock() {
                for (name, is_dirty) in dirty_flags.iter() {
                    if *is_dirty
                        && let Some(value) = env_vars.get(name) {
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

    /// Clear all environment variables
    pub fn clear_env_vars(&self) {
        if let Ok(mut vars) = self.env_vars.lock() {
            vars.clear();
        }
        if let Ok(mut dirty) = self.dirty_vars.lock() {
            dirty.clear();
        }
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
    pub fn initialize_with_env(&self, variables: &HashMap<String, String>, secrets: &HashMap<String, String>) {
        if let Ok(mut vars) = self.env_vars.lock() {
            // Add regular variables
            for (key, value) in variables {
                vars.insert(key.clone(), Value::String(value.clone()));
            }
            // Add secrets
            for (key, value) in secrets {
                vars.insert(key.clone(), Value::String(value.clone()));
            }
        }
    }
}

impl Default for VariableStore {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for VariableStore {
    fn drop(&mut self) {
        // Clean up when the store is dropped
        self.clear_env_vars();
    }
}
