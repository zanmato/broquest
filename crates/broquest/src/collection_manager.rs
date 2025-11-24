use crate::app_database::{AppDatabase, CollectionData};
use crate::collection_types::{CollectionToml, EnvironmentToml, EnvironmentVariable, RequestToml};
use crate::request_editor::RequestData;
use anyhow::{Context, Result};
use gpui::{App, Global};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
pub struct CollectionInfo {
    #[allow(dead_code)]
    pub id: i64,
    pub data: CollectionData,
    pub toml: CollectionToml,
    pub requests: HashMap<String, RequestData>, // file_path -> RequestData
}

pub struct CollectionManager {
    base_path: PathBuf,
    collections: HashMap<i64, CollectionInfo>,
    next_id: i64,
    path_to_id: HashMap<String, i64>, // path -> id mapping
}

impl CollectionManager {
    pub fn new(base_path: &str) -> Self {
        Self {
            base_path: PathBuf::from(base_path),
            collections: HashMap::new(),
            next_id: 1,
            path_to_id: HashMap::new(),
        }
    }

    /// Scan for collections in the specified directory and cache them
    pub fn scan_and_cache_collections(&mut self) -> Result<()> {
        self.collections.clear();
        self.path_to_id.clear();

        // Check if the base path exists
        if !self.base_path.exists() {
            tracing::warn!("Collection directory does not exist: {:?}", self.base_path);
            return Ok(());
        }

        // Look for collection.toml files in subdirectories
        for entry in fs::read_dir(&self.base_path)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Check if this directory contains a collection.toml file
                let collection_toml_path = path.join("collection.toml");
                if collection_toml_path.exists() {
                    match self.load_and_cache_collection(&path) {
                        Ok(_) => {}
                        Err(e) => {
                            tracing::error!("Failed to load collection from {:?}: {}", path, e)
                        }
                    }
                }
            }
        }

        tracing::info!("Found and cached {} collections", self.collections.len());
        Ok(())
    }

    /// Load and cache a single collection from a directory containing collection.toml
    fn load_and_cache_collection(&mut self, collection_dir: &Path) -> Result<i64> {
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

        // Load all requests in this collection
        let requests = self.load_collection_requests(collection_dir)?;

        tracing::info!(
            "Loaded collection '{}' with {} requests and {} environments from {:?}",
            collection_toml.collection.name,
            requests.len(),
            collection_toml.environments.len(),
            collection_dir
        );

        let id = self.next_id;
        self.next_id += 1;

        let path_string = collection_dir.to_string_lossy().to_string();

        let collection_info = CollectionInfo {
            id,
            data: CollectionData {
                id: Some(id),
                name: collection_toml.collection.name.clone(),
                path: path_string.clone(),
                position: 0,
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            toml: collection_toml,
            requests,
        };

        self.collections.insert(id, collection_info);
        self.path_to_id.insert(path_string, id);

        Ok(id)
    }

    /// Get collection by ID
    pub fn get_collection_by_id(&self, id: i64) -> Option<&CollectionInfo> {
        self.collections.get(&id)
    }

    /// Get collection ID by path
    pub fn get_collection_id(&self, collection_path: &str) -> Option<i64> {
        self.path_to_id.get(collection_path).copied()
    }

    /// Get all cached collections
    pub fn get_all_collections(&self) -> Vec<&CollectionInfo> {
        self.collections.values().collect()
    }

    /// Get collection environments
    pub fn get_collection_environments(&self, collection_id: i64) -> Option<Vec<EnvironmentToml>> {
        self.collections
            .get(&collection_id)
            .map(|info| info.toml.environments.clone())
    }

    /// Get collection ID by name
    #[allow(dead_code)]
    pub fn get_collection_id_by_name(&self, collection_name: &str) -> Option<i64> {
        self.collections
            .values()
            .find(|info| info.data.name == collection_name)
            .and_then(|info| info.data.id)
    }

    /// Get collection name by ID
    #[allow(dead_code)]
    pub fn get_collection_name_by_id(&self, collection_id: i64) -> Option<String> {
        self.collections
            .get(&collection_id)
            .map(|info| info.data.name.clone())
    }

    /// Load collections from the database and cache their TOML data from the file system
    pub fn load_saved(&mut self, cx: &mut App) -> Result<()> {
        let app_database = AppDatabase::global(cx).clone();

        // Load collections from database using blocking task
        let db_collections = async_std::task::block_on(async {
            match app_database.load_collections().await {
                Ok(collections) => {
                    tracing::info!("Loaded {} collections from database", collections.len());
                    collections
                }
                Err(e) => {
                    tracing::error!("Failed to load collections from database: {}", e);
                    Vec::new()
                }
            }
        });

        // Clear existing collections
        self.collections.clear();
        self.path_to_id.clear();

        // Load TOML data for each collection from the database
        for collection_data in db_collections {
            let collection_path = std::path::Path::new(&collection_data.path);

            if collection_path.exists() {
                match self.load_and_cache_collection(collection_path) {
                    Ok(_id) => {
                        tracing::info!(
                            "Loaded collection '{}' from database and file system",
                            collection_data.name
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to load collection '{}' from path '{}': {}",
                            collection_data.name,
                            collection_data.path,
                            e
                        );
                    }
                }
            } else {
                tracing::warn!("Collection path does not exist: {}", collection_data.path);
            }
        }

        // If no collections were loaded from database, fall back to file system scan
        if self.collections.is_empty() {
            tracing::info!("No collections found in database, scanning file system");
            self.scan_and_cache_collections()?;
        }

        tracing::info!("Total cached collections: {}", self.collections.len());
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
    pub fn save_collection(
        &mut self,
        collection_data: &CollectionToml,
        path: &str,
        database_id: Option<i64>,
    ) -> Result<()> {
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

        // Update or add the collection in the in-memory cache
        let collection_name = &collection_data.collection.name;

        // Find if this collection already exists in our cache by path
        let existing_collection = self
            .collections
            .values()
            .find(|info| info.data.path == path);

        if let Some(existing_info) = existing_collection {
            // Update existing collection
            if let Some(collection_id) = existing_info.data.id
                && let Some(info) = self.collections.get_mut(&collection_id)
            {
                info.toml = collection_data.clone();
                tracing::info!("Updated existing collection '{}' in cache", collection_name);
            }
        } else {
            // This is a new collection - use the database ID if available, otherwise assign a temporary ID
            let collection_id = if let Some(db_id) = database_id {
                db_id
            } else {
                // Use a negative temporary ID that will be replaced when the database assigns a proper ID
                let temp_id = -self.next_id;
                self.next_id += 1;
                temp_id
            };

            let collection_info = CollectionInfo {
                id: collection_id,
                data: crate::app_database::CollectionData {
                    id: Some(collection_id),
                    name: collection_name.clone(),
                    path: path.to_string(),
                    position: 0, // Default position
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                },
                toml: collection_data.clone(),
                requests: HashMap::new(), // Empty for new collections
            };

            self.collections.insert(collection_id, collection_info);
            if database_id.is_some() {
                tracing::info!(
                    "Added new collection '{}' to cache with database ID {}",
                    collection_name,
                    collection_id
                );
            } else {
                tracing::info!(
                    "Added new collection '{}' to cache with temporary ID {}",
                    collection_name,
                    collection_id
                );
            }
        }

        tracing::info!("Collection saved successfully to: {}", path);

        Ok(())
    }

    /// Remove a collection from the manager by ID
    pub fn remove_collection(&mut self, collection_id: i64) {
        if let Some(collection) = self.collections.remove(&collection_id) {
            // Remove from path_to_id mapping
            self.path_to_id.retain(|_, &mut id| id != collection_id);

            tracing::info!(
                "Removed collection '{}' (ID: {}) from manager",
                collection.data.name,
                collection_id
            );
        } else {
            tracing::warn!("Collection with ID {} not found in manager", collection_id);
        }
    }

    /// Save a request to a collection directory
    pub fn save_request(
        &mut self,
        collection_id: i64,
        request_data: &RequestData,
        request_name: &str,
    ) -> Result<()> {
        // Get collection info as mutable reference
        let collection_info = self
            .collections
            .get_mut(&collection_id)
            .ok_or_else(|| anyhow::anyhow!("Collection with ID {} not found", collection_id))?;

        // Create the full file path
        let collection_path = Path::new(&collection_info.data.path);
        let request_file_path = collection_path.join(format!("{}.toml", request_name));

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
    pub fn delete_request(&mut self, collection_id: i64, request_data: &RequestData) -> Result<()> {
        // Get collection info as mutable reference
        let collection_info = self
            .collections
            .get_mut(&collection_id)
            .ok_or_else(|| anyhow::anyhow!("Collection with ID {} not found", collection_id))?;

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
        collection_id: i64,
        environment_name: &str,
        dirty_vars: &HashMap<String, String>,
        cx: &mut App,
    ) -> Result<()> {
        // Get collection info
        let collection_info = self
            .collections
            .get_mut(&collection_id)
            .ok_or_else(|| anyhow::anyhow!("Collection with ID {} not found", collection_id))?;

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
