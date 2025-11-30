use crate::app_database::{AppDatabase, CollectionData};
use crate::collection_types::{CollectionToml, EnvironmentToml, EnvironmentVariable, RequestToml};
use crate::request_editor::RequestData;
use anyhow::{Context, Result};
use gpui::{App, Global};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
pub struct CollectionInfo {
    pub data: CollectionData,
    pub toml: CollectionToml,
    pub requests: HashMap<String, RequestData>, // file_path -> RequestData
}

pub struct CollectionManager {
    collections: HashMap<String, CollectionInfo>, // collection_path -> CollectionInfo
}

impl CollectionManager {
    pub fn new() -> Self {
        Self {
            collections: HashMap::new(),
        }
    }

    /// Get collection by path
    pub fn get_collection_by_path(&self, collection_path: &str) -> Option<&CollectionInfo> {
        self.collections.get(collection_path)
    }

    /// Get collection environments by path
    pub fn get_collection_environments(
        &self,
        collection_path: &str,
    ) -> Option<Vec<EnvironmentToml>> {
        self.collections
            .get(collection_path)
            .map(|info| info.toml.environments.clone())
    }

    /// Get all cached collections
    pub fn get_all_collections(&self) -> Vec<&CollectionInfo> {
        self.collections.values().collect()
    }

    /// Load collections from the database and cache their TOML data from the file system
    pub fn load_saved(&mut self, cx: &mut App) -> Result<()> {
        let app_database = AppDatabase::global(cx).clone();

        // Load collections from the database
        let db_collections =
            async_std::task::block_on(async move { app_database.load_collections().await });

        match db_collections {
            Ok(collections) => {
                tracing::info!("Loaded {} collections from database", collections.len());

                for collection_data in collections {
                    // Try to load the TOML data from the file system path
                    let collection_path = Path::new(&collection_data.path);
                    if collection_path.exists() {
                        match self.load_collection_toml(collection_path) {
                            Ok(collection_toml) => {
                                tracing::info!(
                                    "Loaded TOML for collection '{}' from database path: {}",
                                    collection_data.name,
                                    collection_data.path
                                );

                                // Load all requests in this collection
                                let requests = self.load_collection_requests(collection_path)?;

                                let collection_info = CollectionInfo {
                                    data: collection_data,
                                    toml: collection_toml,
                                    requests,
                                };

                                self.collections
                                    .insert(collection_info.data.path.clone(), collection_info);
                            }
                            Err(e) => {
                                tracing::error!(
                                    "Failed to load TOML for collection '{}' from path {}: {}",
                                    collection_data.name,
                                    collection_data.path,
                                    e
                                );
                            }
                        }
                    } else {
                        tracing::warn!(
                            "Collection path from database does not exist: {}",
                            collection_data.path
                        );
                    }
                }

                tracing::info!("Total cached collections: {}", self.collections.len());
            }
            Err(e) => {
                tracing::error!("Failed to load collections from database: {}", e);
            }
        }

        Ok(())
    }

    /// Load collection data as CollectionToml from a collection directory
    pub fn load_collection_toml(&self, collection_dir: &Path) -> Result<CollectionToml> {
        let collection_toml_path = collection_dir.join("collection.toml");

        // Read and parse collection.toml
        let collection_content = fs::read_to_string(&collection_toml_path).with_context(|| {
            format!(
                "Failed to read collection.toml from {:?}",
                collection_toml_path
            )
        })?;

        let collection_toml: CollectionToml =
            toml::from_str(&collection_content).with_context(|| {
                format!(
                    "Failed to parse collection.toml from {:?}",
                    collection_toml_path
                )
            })?;

        Ok(collection_toml)
    }

    /// Load all requests from a collection directory
    pub fn load_collection_requests(
        &self,
        collection_dir: &Path,
    ) -> Result<std::collections::HashMap<String, RequestData>> {
        let mut requests = std::collections::HashMap::new();

        for entry in fs::read_dir(collection_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Load .toml files that aren't collection.toml or in environments directory
            if let Some(filename) = path.file_name()
                && let Some(filename_str) = filename.to_str()
                && filename_str.ends_with(".toml")
                && filename_str != "collection.toml"
                && !path.parent().unwrap().ends_with("environments")
            {
                match self.load_request_file(&path) {
                    Ok(request) => {
                        let path_str = path.to_string_lossy().to_string();
                        requests.insert(path_str, request);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load request from {:?}: {}", path, e)
                    }
                }
            }
        }

        Ok(requests)
    }

    /// Save collection data to the specified path and update the in-memory cache
    pub fn save_collection(&mut self, collection_data: &CollectionToml, path: &str) -> Result<()> {
        use std::path::Path;

        // Create directory if it doesn't exist
        fs::create_dir_all(path)?;

        // Create the full path to collection.toml
        let collection_file_path = Path::new(path).join("collection.toml");

        // Serialize collection data to TOML string
        let toml_string = toml::to_string_pretty(collection_data)
            .with_context(|| "Failed to serialize collection data to TOML")?;

        // Write to file
        fs::write(collection_file_path, toml_string)
            .with_context(|| format!("Failed to write collection.toml to path: {}", path))?;

        // Update or add the collection in the in-memory cache using path as key
        let collection_name = &collection_data.collection.name;

        let collection_info = CollectionInfo {
            data: crate::app_database::CollectionData {
                id: None, // We don't use IDs anymore
                name: collection_name.clone(),
                path: path.to_string(),
                position: 0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            toml: collection_data.clone(),
            requests: HashMap::new(), // This will be populated when requests are loaded
        };

        self.collections.insert(path.to_string(), collection_info);
        tracing::info!(
            "Collection '{}' saved and cached at path: {}",
            collection_name,
            path
        );

        Ok(())
    }

    /// Remove a collection from the manager by path
    pub fn remove_collection(&mut self, collection_path: &str) {
        if let Some(collection) = self.collections.remove(collection_path) {
            tracing::info!(
                "Removed collection '{}' (path: {}) from manager",
                collection.data.name,
                collection_path
            );
        } else {
            tracing::warn!(
                "Collection with path {} not found in manager",
                collection_path
            );
        }
    }

    /// Save a request to a collection directory
    pub fn save_request(
        &mut self,
        collection_path: &str,
        request_data: &RequestData,
        request_name: &str,
    ) -> Result<()> {
        // Get collection info as mutable reference
        let collection_info = self
            .collections
            .get_mut(collection_path)
            .ok_or_else(|| anyhow::anyhow!("Collection with path {} not found", collection_path))?;

        // Create the full file path
        let collection_dir_path = Path::new(&collection_info.data.path);
        let request_file_path = collection_dir_path.join(format!("{}.toml", request_name));

        // Convert RequestData to RequestToml
        let request_toml: RequestToml = request_data.clone().into();

        // Serialize to TOML string
        let toml_string = toml::to_string_pretty(&request_toml)
            .with_context(|| "Failed to serialize request data to TOML")?;

        // Write to file
        fs::write(&request_file_path, toml_string)
            .with_context(|| format!("Failed to write request file to {:?}", request_file_path))?;

        // Check if request already exists by path and update, otherwise insert
        let request_path_str = request_file_path.to_string_lossy().to_string();
        let is_update = collection_info.requests.contains_key(&request_path_str);

        collection_info
            .requests
            .insert(request_path_str.clone(), request_data.clone());

        tracing::info!(
            "Request '{}' {} successfully to collection '{}'",
            request_name,
            if is_update { "updated" } else { "saved" },
            collection_info.data.name
        );

        Ok(())
    }

    /// Delete a request from a collection
    pub fn delete_request(
        &mut self,
        collection_path: &str,
        request_data: &RequestData,
    ) -> Result<()> {
        // Get collection info as mutable reference
        let collection_info = self
            .collections
            .get_mut(collection_path)
            .ok_or_else(|| anyhow::anyhow!("Collection with path {} not found", collection_path))?;

        // Find the file path for this request
        let request_file_path = collection_info
            .requests
            .iter()
            .find(|(_, stored_request)| {
                stored_request.name == request_data.name
                    && stored_request.method == request_data.method
                    && stored_request.url == request_data.url
            })
            .map(|(path, _)| path.clone());

        let Some(request_file_path) = request_file_path else {
            return Err(anyhow::anyhow!("Request file not found in collection"));
        };

        // Delete the file from disk
        fs::remove_file(&request_file_path)
            .with_context(|| format!("Failed to delete request file {:?}", request_file_path))?;

        // Remove from in-memory collection info
        collection_info.requests.remove(&request_file_path);

        tracing::info!(
            "Request '{}' deleted successfully from collection '{}'",
            request_data.name,
            collection_info.data.name
        );

        Ok(())
    }

    /// Load a single request file
    fn load_request_file(&self, file_path: &Path) -> Result<RequestData> {
        let content = fs::read_to_string(file_path)
            .with_context(|| format!("Failed to read request file {:?}", file_path))?;

        let mut request_toml: RequestToml = toml::from_str(&content)
            .with_context(|| format!("Failed to parse request file {:?}", file_path))?;

        // If the request name is empty or missing, use the file name without extension
        if request_toml.meta.name.trim().is_empty()
            && let Some(file_stem) = file_path.file_stem()
            && let Some(file_name) = file_stem.to_str()
        {
            request_toml.meta.name = file_name.to_string();
            tracing::info!(
                "Request name was empty, using file name: '{}' for file: {:?}",
                file_name,
                file_path
            );
        }

        // Convert TOML to RequestData using the existing From impl
        Ok(request_toml.into())
    }

    /// Update environment variables in a collection environment
    pub fn update_environment_variables(
        &mut self,
        collection_path: &str,
        environment_name: &str,
        dirty_vars: &HashMap<String, String>,
        cx: &mut App,
    ) -> Result<()> {
        // Get collection info
        let collection_info = self
            .collections
            .get_mut(collection_path)
            .ok_or_else(|| anyhow::anyhow!("Collection with path {} not found", collection_path))?;

        // Find the environment
        let environment = collection_info
            .toml
            .environments
            .iter_mut()
            .find(|env| env.name == environment_name)
            .ok_or_else(|| {
                anyhow::anyhow!("Environment '{}' not found in collection", environment_name)
            })?;

        // Update dirty variables
        for (var_name, var_value) in dirty_vars {
            if let Some(env_var) = environment.variables.get_mut(var_name) {
                // Update existing variable value
                env_var.value = var_value.clone();

                // If it's a secret, also update secure storage
                if env_var.secret
                    && let Err(e) = EnvironmentVariable::write_credential(
                        &collection_info.data.name,
                        environment_name,
                        var_name,
                        var_value,
                        cx,
                    )
                {
                    tracing::error!(
                        "Failed to update secret '{}' in secure storage: {}",
                        var_name,
                        e
                    );
                    return Err(anyhow::anyhow!("Failed to update secret storage: {}", e));
                }

                tracing::info!(
                    "Updated environment variable '{}' to value '{}'",
                    var_name,
                    var_value
                );
            } else {
                tracing::warn!(
                    "Environment variable '{}' not found in environment '{}'",
                    var_name,
                    environment_name
                );
            }
        }

        // Save the updated collection.toml file
        let collection_path = Path::new(&collection_info.data.path);
        let collection_toml_path = collection_path.join("collection.toml");

        // Serialize to TOML string
        let toml_string = toml::to_string_pretty(&collection_info.toml)
            .with_context(|| "Failed to serialize collection data to TOML")?;

        // Write to file
        fs::write(&collection_toml_path, toml_string).with_context(|| {
            format!(
                "Failed to write collection.toml to {:?}",
                collection_toml_path
            )
        })?;

        tracing::info!(
            "Environment variables updated and saved for collection '{}', environment '{}'",
            collection_info.data.name,
            environment_name
        );

        Ok(())
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }
}

impl Global for CollectionManager {}
