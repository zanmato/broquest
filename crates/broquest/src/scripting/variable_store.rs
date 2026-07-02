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
    /// Runtime variables (Bruno `bru.setVar`/`getVar`): in-memory, not persisted
    /// to the environment file.
    runtime_vars: HashMap<String, Value>,
    /// Collection-level variables (Bruno `bru.getCollectionVar`): declared in
    /// the collection, read-only from scripts, resolvable via `{{name}}`.
    collection_vars: HashMap<String, Value>,
    /// Request-level variables (Bruno `bru.getRequestVar`): declared on the
    /// request, read-only from scripts, resolvable via `{{name}}`.
    request_vars: HashMap<String, Value>,
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

    /// Check whether an environment variable exists.
    pub fn has_env_var(&self, name: &str) -> bool {
        self.data
            .lock()
            .map(|data| data.env_vars.contains_key(name))
            .unwrap_or(false)
    }

    /// Delete an environment variable (and mark the deletion dirty).
    pub fn delete_env_var(&self, name: &str) {
        if let Ok(mut data) = self.data.lock() {
            data.env_vars.remove(name);
            data.dirty_flags.insert(name.to_string(), true);
        }
    }

    /// Set a runtime variable (Bruno `bru.setVar`).
    pub fn set_var(&self, name: &str, value: Value) {
        if let Ok(mut data) = self.data.lock() {
            data.runtime_vars.insert(name.to_string(), value);
        }
    }

    /// Get a runtime variable (Bruno `bru.getVar`).
    pub fn get_var(&self, name: &str) -> Option<Value> {
        self.data
            .lock()
            .ok()
            .and_then(|data| data.runtime_vars.get(name).cloned())
    }

    /// Check whether a runtime variable exists.
    pub fn has_var(&self, name: &str) -> bool {
        self.data
            .lock()
            .map(|data| data.runtime_vars.contains_key(name))
            .unwrap_or(false)
    }

    /// Delete a runtime variable.
    pub fn delete_var(&self, name: &str) {
        if let Ok(mut data) = self.data.lock() {
            data.runtime_vars.remove(name);
        }
    }

    /// Get all runtime variables as a HashMap copy.
    pub fn get_all_vars(&self) -> HashMap<String, Value> {
        self.data
            .lock()
            .map(|data| data.runtime_vars.clone())
            .unwrap_or_default()
    }

    /// Seed the read-only collection variable bucket (Bruno `bru.getCollectionVar`).
    /// Replaces any prior collection vars. Called once per request before scripts run.
    pub fn set_collection_vars(&self, vars: HashMap<String, Value>) {
        if let Ok(mut data) = self.data.lock() {
            data.collection_vars = vars;
        }
    }

    /// Get a collection-level variable (Bruno `bru.getCollectionVar`).
    pub fn get_collection_var(&self, name: &str) -> Option<Value> {
        self.data
            .lock()
            .ok()
            .and_then(|data| data.collection_vars.get(name).cloned())
    }

    /// Check whether a collection-level variable exists.
    pub fn has_collection_var(&self, name: &str) -> bool {
        self.data
            .lock()
            .map(|data| data.collection_vars.contains_key(name))
            .unwrap_or(false)
    }

    /// Seed the read-only request variable bucket (Bruno `bru.getRequestVar`).
    /// Replaces any prior request vars. Called once per request before scripts run.
    pub fn set_request_vars(&self, vars: HashMap<String, Value>) {
        if let Ok(mut data) = self.data.lock() {
            data.request_vars = vars;
        }
    }

    /// Get a request-level variable (Bruno `bru.getRequestVar`).
    pub fn get_request_var(&self, name: &str) -> Option<Value> {
        self.data
            .lock()
            .ok()
            .and_then(|data| data.request_vars.get(name).cloned())
    }

    /// Check whether a request-level variable exists.
    pub fn has_request_var(&self, name: &str) -> bool {
        self.data
            .lock()
            .map(|data| data.request_vars.contains_key(name))
            .unwrap_or(false)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_runtime_vars_set_get_delete() {
        let store = VariableStore::new();
        assert!(!store.has_var("token"));
        assert_eq!(store.get_var("token"), None);

        store.set_var("token", Value::String("abc".into()));
        assert!(store.has_var("token"));
        assert_eq!(store.get_var("token"), Some(Value::String("abc".into())));

        store.delete_var("token");
        assert!(!store.has_var("token"));
    }

    #[test]
    fn test_collection_vars_readonly_separate_bucket() {
        let store = VariableStore::new();

        // Collection vars are seeded in bulk and distinct from runtime vars.
        let mut cv = HashMap::new();
        cv.insert("namespace".to_string(), Value::String("prod".into()));
        store.set_collection_vars(cv);
        assert!(store.has_collection_var("namespace"));
        assert_eq!(
            store.get_collection_var("namespace"),
            Some(Value::String("prod".into()))
        );

        // A collection var is not visible as a runtime var and vice versa.
        store.set_var("runtime_only", Value::Number(42.into()));
        assert!(!store.has_collection_var("runtime_only"));
        assert!(store.get_var("namespace").is_none());

        // Re-seeding collection vars replaces the bucket entirely.
        let empty: HashMap<String, Value> = HashMap::new();
        store.set_collection_vars(empty);
        assert!(!store.has_collection_var("namespace"));
        // Runtime vars are untouched by collection re-seed.
        assert!(store.has_var("runtime_only"));
    }
}

impl Default for VariableStore {
    fn default() -> Self {
        Self::new()
    }
}
