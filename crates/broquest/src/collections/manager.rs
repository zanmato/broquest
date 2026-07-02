use super::format::CollectionFormat;
use super::opencollection::{self, LoadedCollection, OcEnvironment, OcItem, OpenCollectionFile};
use super::types::{
    CollectionMeta, CollectionToml, EnvironmentToml, EnvironmentVariable, RequestToml,
};
use crate::app_database::{AppDatabase, CollectionData};
use crate::domain::RequestData;
use anyhow::{Context, Result};
use gpui::{App, Global};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

#[derive(Clone, Debug)]
#[allow(dead_code)]
pub struct GroupInfo {
    pub name: String,
    pub requests: HashMap<String, RequestData>, // file_path -> RequestData within group
    pub path: String,                           // Relative path from collection root
}

#[derive(Clone, Debug)]
pub struct CollectionInfo {
    pub data: CollectionData,
    pub toml: CollectionToml,
    pub requests: HashMap<String, RequestData>, // file_path -> RequestData
    pub groups: HashMap<String, GroupInfo>,     // group_name -> GroupInfo
    /// On-disk format of this collection.
    pub format: CollectionFormat,
    /// For OpenCollection collections: the fully-parsed root file, retained so
    /// top-level fields broquest does not model survive when saving back.
    pub oc_source: Option<OpenCollectionFile>,
    /// For OpenCollection collections: the parsed source item for each request,
    /// keyed like `requests`, so per-request unmodeled fields (examples,
    /// settings, tags, …) are preserved on save.
    pub oc_items: HashMap<String, OcItem>,
    /// Parsed `folder.yml` / folder-item source per group name.
    pub oc_groups: HashMap<String, OcItem>,
    /// Parsed environment source per environment name.
    pub oc_envs: HashMap<String, OcEnvironment>,
    /// In-memory runtime variables (Bruno `bru.setVar`/`getVar`), session-scoped
    /// per collection. Not persisted to disk.
    pub runtime_vars: HashMap<String, serde_json::Value>,
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
        let db_collections = smol::block_on(async move { app_database.load_collections().await });

        match db_collections {
            Ok(collections) => {
                tracing::info!("Loaded {} collections from database", collections.len());

                // Create a local HashMap to store collections
                let mut local_collections: HashMap<String, CollectionInfo> = HashMap::new();

                for collection_data in collections {
                    // Try to load from the file system path
                    let collection_path = Path::new(&collection_data.path).to_path_buf();
                    if !collection_path.exists() {
                        tracing::warn!(
                            "Collection path from database does not exist: {}",
                            collection_data.path
                        );
                        continue;
                    }

                    let format = Self::detect_collection_format(&collection_data, &collection_path);
                    let result = match format {
                        CollectionFormat::OpenCollection => self
                            .build_opencollection_info(&collection_path, collection_data.clone()),
                        CollectionFormat::Broquest => {
                            self.build_broquest_info(&collection_path, collection_data.clone())
                        }
                    };

                    match result {
                        Ok(info) => {
                            local_collections.insert(info.data.path.clone(), info);
                        }
                        Err(e) => {
                            tracing::error!(
                                "Failed to load collection '{}' from path {}: {}",
                                collection_data.name,
                                collection_data.path,
                                e
                            );
                        }
                    }
                }

                tracing::info!("Total cached collections: {}", local_collections.len());

                // Replace the contents of self.collections with the local collections
                self.collections = local_collections;
            }
            Err(e) => {
                tracing::error!("Failed to load collections from database: {}", e);
            }
        }

        Ok(())
    }

    /// Determine a collection's format from the database record, falling back to
    /// file presence (a directory with an OpenCollection marker but no
    /// `collection.toml` is treated as OpenCollection).
    fn detect_collection_format(data: &CollectionData, dir: &Path) -> CollectionFormat {
        if CollectionFormat::from_db_str(&data.format) == CollectionFormat::OpenCollection {
            return CollectionFormat::OpenCollection;
        }
        if !dir.join("collection.toml").exists()
            && opencollection::find_opencollection_file(dir).is_some()
        {
            return CollectionFormat::OpenCollection;
        }
        CollectionFormat::Broquest
    }

    /// Build a native (TOML) `CollectionInfo` from a directory + database record.
    fn build_broquest_info(&self, dir: &Path, mut data: CollectionData) -> Result<CollectionInfo> {
        let toml = self.read_collection_toml(dir)?;
        let (requests, groups) = self.load_collection_structure(dir)?;
        data.format = CollectionFormat::Broquest.as_db_str().to_string();
        Ok(CollectionInfo {
            data,
            toml,
            requests,
            groups,
            format: CollectionFormat::Broquest,
            oc_source: None,
            oc_items: HashMap::new(),
            oc_groups: HashMap::new(),
            oc_envs: HashMap::new(),
            runtime_vars: HashMap::new(),
        })
    }

    /// Build an OpenCollection `CollectionInfo` from a directory + database
    /// record, decomposing the YAML tree into broquest's request/group model.
    fn build_opencollection_info(
        &self,
        dir: &Path,
        mut data: CollectionData,
    ) -> Result<CollectionInfo> {
        let loaded = opencollection::load_opencollection(dir)?;
        data.name = loaded.name.clone();
        data.format = CollectionFormat::OpenCollection.as_db_str().to_string();
        Ok(assemble_opencollection_info(dir, data, loaded))
    }

    /// Create a new, empty OpenCollection collection at `path` (non-bundled by
    /// default — Bruno's convention and the most git-friendly layout) and write
    /// it to disk. Used by the "Use OpenCollection format" toggle.
    pub fn create_opencollection(
        &mut self,
        collection_data: &CollectionToml,
        path: &str,
    ) -> Result<()> {
        let now = chrono::Utc::now();
        let base = opencollection::build_unbundled_root(
            None,
            &collection_data.collection.name,
            &collection_data.collection.version,
            collection_data.collection.docs.as_deref(),
            &collection_data.collection.vars,
        );
        let info = CollectionInfo {
            data: CollectionData {
                id: None,
                name: collection_data.collection.name.clone(),
                path: path.to_string(),
                position: 0,
                format: CollectionFormat::OpenCollection.as_db_str().to_string(),
                created_at: now,
                updated_at: now,
            },
            toml: collection_data.clone(),
            requests: HashMap::new(),
            groups: HashMap::new(),
            format: CollectionFormat::OpenCollection,
            oc_source: Some(base),
            oc_items: HashMap::new(),
            oc_groups: HashMap::new(),
            oc_envs: HashMap::new(),
            runtime_vars: HashMap::new(),
        };
        self.collections.insert(path.to_string(), info);
        self.persist_opencollection(path)
    }

    /// Load an OpenCollection directory into the manager and return the
    /// synthesized collection metadata (mirrors [`Self::load_collection_toml`]).
    pub fn load_opencollection_dir(&mut self, dir: &Path) -> Result<CollectionToml> {
        let data = CollectionData {
            id: None,
            name: String::new(),
            path: dir.to_string_lossy().to_string(),
            position: 0,
            format: CollectionFormat::OpenCollection.as_db_str().to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let info = self.build_opencollection_info(dir, data)?;
        let toml = info.toml.clone();
        self.collections.insert(info.data.path.clone(), info);
        Ok(toml)
    }

    /// Read collection data as CollectionToml from a collection directory
    pub fn read_collection_toml(&self, collection_dir: &Path) -> Result<CollectionToml> {
        let collection_path = collection_dir.join("collection.toml");

        // Read and parse collection.toml
        let collection_content = fs::read_to_string(&collection_path).with_context(|| {
            format!("Failed to read collection.toml from {:?}", collection_path)
        })?;

        let collection_toml: CollectionToml =
            toml::from_str(&collection_content).with_context(|| {
                format!("Failed to parse collection.toml from {:?}", collection_path)
            })?;

        Ok(collection_toml)
    }

    /// Load collection data as CollectionToml from a collection directory
    pub fn load_collection_toml(&mut self, collection_dir: &Path) -> Result<CollectionToml> {
        let collection_path = collection_dir.join("collection.toml");

        // Read and parse collection.toml
        let collection_content = fs::read_to_string(&collection_path).with_context(|| {
            format!("Failed to read collection.toml from {:?}", collection_path)
        })?;

        let collection_toml: CollectionToml =
            toml::from_str(&collection_content).with_context(|| {
                format!("Failed to parse collection.toml from {:?}", collection_path)
            })?;

        // Load all requests and groups in this collection
        let (requests, groups) = self.load_collection_structure(collection_dir)?;

        let collection_name = collection_toml.collection.name.clone();
        let collection_info = CollectionInfo {
            data: CollectionData {
                id: None,
                name: collection_name,
                path: collection_path.to_string_lossy().to_string(),
                position: 0,
                format: CollectionFormat::Broquest.as_db_str().to_string(),
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            toml: collection_toml.clone(),
            requests,
            groups,
            format: CollectionFormat::Broquest,
            oc_source: None,
            oc_items: HashMap::new(),
            oc_groups: HashMap::new(),
            oc_envs: HashMap::new(),
            runtime_vars: HashMap::new(),
        };

        self.collections
            .insert(collection_info.data.path.clone(), collection_info);

        Ok(collection_toml)
    }

    /// Load all requests and groups from a collection directory
    pub fn load_collection_structure(
        &self,
        collection_dir: &Path,
    ) -> Result<(HashMap<String, RequestData>, HashMap<String, GroupInfo>)> {
        let mut requests = HashMap::new();
        let mut groups = HashMap::new();

        for entry in fs::read_dir(collection_dir)? {
            let entry = entry?;
            let path = entry.path();

            // Skip collection.toml and environments directory
            if let Some(filename) = path.file_name()
                && let Some(filename_str) = filename.to_str()
                && (filename_str == "collection.toml" || filename_str == "environments")
            {
                continue;
            }

            // Check if this is a directory (potential group)
            if path.is_dir() {
                // Handle group directory
                let group_name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .ok_or_else(|| anyhow::anyhow!("Invalid group directory name"))?;

                let group_requests = self.load_group_requests(&path)?;
                let group_info = GroupInfo {
                    name: group_name.to_string(),
                    requests: group_requests,
                    path: path
                        .strip_prefix(collection_dir)
                        .map_err(|_| anyhow::anyhow!("Failed to get relative group path"))?
                        .to_string_lossy()
                        .to_string(),
                };
                groups.insert(group_name.to_string(), group_info);
            }
            // Handle individual request files at collection root
            else if let Some(filename) = path.file_name()
                && let Some(filename_str) = filename.to_str()
                && filename_str.ends_with(".toml")
            {
                match self.load_request_file(&path) {
                    Ok(request) => {
                        let path_str = path.to_string_lossy().to_string();
                        requests.insert(path_str, request);
                    }
                    Err(e) => {
                        tracing::error!("Failed to load request from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok((requests, groups))
    }

    /// Load requests from a group directory
    fn load_group_requests(&self, group_dir: &Path) -> Result<HashMap<String, RequestData>> {
        let mut requests = HashMap::new();

        for entry in fs::read_dir(group_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(filename) = path.file_name()
                && let Some(filename_str) = filename.to_str()
                && filename_str.ends_with(".toml")
            {
                match self.load_request_file(&path) {
                    Ok(request) => {
                        let path_str = path.to_string_lossy().to_string();
                        requests.insert(path_str, request);
                    }
                    Err(e) => {
                        tracing::error!(
                            "Failed to load request from group directory {:?}: {}",
                            path,
                            e
                        );
                    }
                }
            }
        }

        Ok(requests)
    }

    /// Save collection data to the specified path and update the in-memory cache
    pub fn save_collection(&mut self, collection_data: &CollectionToml, path: &str) -> Result<()> {
        use std::path::Path;

        // OpenCollection collections don't have a collection.toml; update the
        // in-memory model and re-serialize the YAML instead.
        if self.is_opencollection(path) {
            if let Some(info) = self.collections.get_mut(path) {
                info.toml = collection_data.clone();
                info.data.name = collection_data.collection.name.clone();
                info.data.updated_at = chrono::Utc::now();
            }
            return self.persist_opencollection(path);
        }

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

        // Check if collection already exists in cache to preserve existing requests and groups
        let (
            existing_requests,
            existing_groups,
            existing_created_at,
            existing_format,
            existing_oc,
            existing_runtime_vars,
        ) = if let Some(existing_collection) = self.collections.get(path) {
            (
                existing_collection.requests.clone(),
                existing_collection.groups.clone(),
                existing_collection.data.created_at,
                existing_collection.format,
                existing_collection.oc_source.clone(),
                existing_collection.runtime_vars.clone(),
            )
        } else {
            (
                HashMap::new(),
                HashMap::new(),
                chrono::Utc::now(),
                CollectionFormat::Broquest,
                None,
                HashMap::new(),
            )
        };

        let collection_info = CollectionInfo {
            data: CollectionData {
                id: None, // We don't use IDs anymore
                name: collection_name.clone(),
                path: path.to_string(),
                position: 0,
                format: existing_format.as_db_str().to_string(),
                created_at: existing_created_at,
                updated_at: chrono::Utc::now(),
            },
            toml: collection_data.clone(),
            requests: existing_requests,
            groups: existing_groups,
            format: existing_format,
            oc_source: existing_oc,
            // This path is only reached for native collections (OpenCollection
            // saves return earlier), so no OpenCollection sources are needed.
            oc_items: HashMap::new(),
            oc_groups: HashMap::new(),
            oc_envs: HashMap::new(),
            runtime_vars: existing_runtime_vars,
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

    /// Save a request to a collection directory or group
    pub fn save_request(
        &mut self,
        collection_path: &str,
        request_data: &RequestData,
        request_name: &str,
        group_path: Option<&str>,
    ) -> Result<()> {
        if self.is_opencollection(collection_path) {
            return self.save_request_opencollection(
                collection_path,
                request_data,
                request_name,
                group_path,
            );
        }

        // Get collection info as mutable reference
        let collection_info = self
            .collections
            .get_mut(collection_path)
            .ok_or_else(|| anyhow::anyhow!("Collection with path {} not found", collection_path))?;

        // Determine the target directory
        let collection_dir_path = Path::new(&collection_info.data.path);
        let target_dir = if let Some(group_path) = group_path {
            collection_dir_path.join(group_path)
        } else {
            collection_dir_path.to_path_buf()
        };

        // Ensure directory exists (ignore error if it already exists)
        if let Err(e) = fs::create_dir_all(&target_dir) {
            // Only re-raise the error if it's not "already exists"
            if e.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(e)
                    .with_context(|| format!("Failed to create directory {:?}", target_dir));
            }
        }

        // Create the full file path
        let request_file_path = target_dir.join(format!("{}.toml", request_name));

        // Convert RequestData to RequestToml
        let request_toml: RequestToml = request_data.clone().into();

        // Serialize to TOML string
        let toml_string = toml::to_string_pretty(&request_toml)
            .with_context(|| "Failed to serialize request data to TOML")?;

        // Overwrite the file with new content
        fs::write(&request_file_path, toml_string)
            .with_context(|| format!("Failed to write request file to {:?}", request_file_path))?;

        // Check if request already exists by path and update, otherwise insert
        let request_path_str = request_file_path.to_string_lossy().to_string();

        // Determine if this is a group request or root request
        let is_update = if let Some(group_path) = group_path {
            // For group requests, store in the appropriate group
            let group_name = Path::new(group_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(group_path);

            if let Some(group_info) = collection_info.groups.get_mut(group_name) {
                let is_update = group_info.requests.contains_key(&request_path_str);
                group_info
                    .requests
                    .insert(request_path_str.clone(), request_data.clone());
                is_update
            } else {
                // Group doesn't exist, create it
                tracing::warn!(
                    "Group '{}' not found in collection, creating new group",
                    group_name
                );
                let mut new_group_requests = HashMap::new();
                new_group_requests.insert(request_path_str.clone(), request_data.clone());

                let new_group = GroupInfo {
                    name: group_name.to_string(),
                    requests: new_group_requests,
                    path: group_path.to_string(),
                };
                collection_info
                    .groups
                    .insert(group_name.to_string(), new_group);
                false // New group, so this is a new request
            }
        } else {
            // Root level request
            let is_update = collection_info.requests.contains_key(&request_path_str);
            collection_info
                .requests
                .insert(request_path_str.clone(), request_data.clone());
            is_update
        };

        let location_info = if let Some(group_path) = group_path {
            format!("group '{}' in collection", group_path)
        } else {
            "collection".to_string()
        };

        tracing::info!(
            "Request '{}' {} successfully to {} '{}'",
            request_name,
            if is_update { "updated" } else { "saved" },
            location_info,
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
        if self.is_opencollection(collection_path) {
            return self.delete_request_opencollection(collection_path, request_data);
        }

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

        // Persist the updated environments in the collection's format.
        let format = collection_info.format;
        let collection_dir = collection_info.data.path.clone();
        let collection_name = collection_info.data.name.clone();
        let toml_snapshot = collection_info.toml.clone();

        match format {
            CollectionFormat::OpenCollection => {
                self.persist_opencollection(&collection_dir)?;
            }
            CollectionFormat::Broquest => {
                let collection_toml_path = Path::new(&collection_dir).join("collection.toml");
                let toml_string = toml::to_string_pretty(&toml_snapshot)
                    .with_context(|| "Failed to serialize collection data to TOML")?;
                fs::write(&collection_toml_path, toml_string).with_context(|| {
                    format!(
                        "Failed to write collection.toml to {:?}",
                        collection_toml_path
                    )
                })?;
            }
        }

        tracing::info!(
            "Environment variables updated and saved for collection '{}', environment '{}'",
            collection_name,
            environment_name
        );

        Ok(())
    }

    /// Replace the in-memory runtime variables for a collection with `vars`.
    ///
    /// Runtime vars are set by scripts via `bru.setVar` and live only for the
    /// session (never persisted to disk). Replacing (rather than merging) mirrors
    /// Bruno, where the store reflects the latest state after each request run.
    /// Triggers `CollectionManager` observers so Vars views refresh.
    pub fn update_runtime_vars(
        &mut self,
        collection_path: &str,
        vars: HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        let info = self
            .collections
            .get_mut(collection_path)
            .ok_or_else(|| anyhow::anyhow!("Collection with path {} not found", collection_path))?;
        info.runtime_vars = vars;
        tracing::info!(
            "Updated runtime variables for collection '{}'",
            info.data.name
        );
        Ok(())
    }

    /// Move a request to a different location in the same collection.
    ///
    /// - `collection_path`: The collection containing the request
    /// - `request_data`: The request to move
    /// - `target_group_path`: Optional path to target group (None = root level)
    pub fn move_request(
        &mut self,
        collection_path: &str,
        request_data: &RequestData,
        target_group_path: Option<&str>,
    ) -> Result<()> {
        if self.is_opencollection(collection_path) {
            return self.move_request_opencollection(
                collection_path,
                request_data,
                target_group_path,
            );
        }

        // First, find the current location of the request
        let current_location = {
            let collection_info = self.collections.get(collection_path).ok_or_else(|| {
                anyhow::anyhow!("Collection with path {} not found", collection_path)
            })?;

            // First check root level requests
            let mut found = None;
            for (path, stored_request) in &collection_info.requests {
                if stored_request.name == request_data.name
                    && stored_request.method == request_data.method
                    && stored_request.url == request_data.url
                {
                    found = Some(path.clone());
                    break;
                }
            }

            // Then check group requests
            if found.is_none() {
                for group_info in collection_info.groups.values() {
                    for (path, stored_request) in &group_info.requests {
                        if stored_request.name == request_data.name
                            && stored_request.method == request_data.method
                            && stored_request.url == request_data.url
                        {
                            found = Some(path.clone());
                            break;
                        }
                    }
                    if found.is_some() {
                        break;
                    }
                }
            }

            found.ok_or_else(|| {
                anyhow::anyhow!("Request '{}' not found in collection", request_data.name)
            })?
        };

        // Remove from current location (in-memory)
        let collection_info = self
            .collections
            .get_mut(collection_path)
            .ok_or_else(|| anyhow::anyhow!("Collection with path {} not found", collection_path))?;

        // Try to remove from root level requests
        let removed_from_root = collection_info.requests.remove(&current_location);

        // If not in root, try to remove from groups
        if removed_from_root.is_none() {
            for group_info in collection_info.groups.values_mut() {
                if group_info.requests.remove(&current_location).is_some() {
                    break;
                }
            }
        }

        // Delete the old file from disk
        fs::remove_file(&current_location)
            .with_context(|| format!("Failed to delete old request file {:?}", current_location))?;

        // Save to the new location (this will update both disk and in-memory)
        self.save_request(
            collection_path,
            request_data,
            &request_data.name,
            target_group_path,
        )?;

        tracing::info!(
            "Request '{}' moved to {:?}",
            request_data.name,
            target_group_path.unwrap_or("root level")
        );

        Ok(())
    }

    /// Create a new group in a collection
    pub fn create_group(&mut self, collection_path: &str, group_name: &str) -> Result<()> {
        // Validate group name
        if group_name.is_empty() {
            return Err(anyhow::anyhow!("Group name cannot be empty"));
        }

        if self.is_opencollection(collection_path) {
            return self.create_group_opencollection(collection_path, group_name);
        }

        // Sanitize group name for filesystem
        let sanitized_name = group_name
            .chars()
            .map(|c| match c {
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
                _ => c,
            })
            .collect::<String>();

        // Get collection info
        let collection_info = self.collections.get(collection_path).ok_or_else(|| {
            anyhow::anyhow!("Collection with path '{}' not found", collection_path)
        })?;

        // Create the group directory
        let collection_dir = Path::new(&collection_info.data.path);
        let group_dir = collection_dir.join(&sanitized_name);

        if group_dir.exists() {
            return Err(anyhow::anyhow!(
                "Group '{}' already exists in collection",
                sanitized_name
            ));
        }

        fs::create_dir_all(&group_dir)
            .with_context(|| format!("Failed to create group directory {:?}", group_dir))?;

        tracing::info!(
            "Group '{}' created in collection '{}' at {:?}",
            sanitized_name,
            collection_info.data.name,
            group_dir
        );

        // Reload the collection to pick up the new group (this modifies state and triggers observers)
        self.reload_collection(collection_path)?;

        Ok(())
    }

    /// Reload a single collection (used when groups are added/removed)
    fn reload_collection(&mut self, collection_path: &str) -> Result<()> {
        // Get the collection info to get the file path
        let collection_dir = Path::new(collection_path);

        // Load the structure
        let (requests, groups) = self.load_collection_structure(collection_dir)?;

        // Update the collection's requests and groups
        if let Some(collection_info) = self.collections.get_mut(collection_path) {
            collection_info.requests = requests;
            collection_info.groups = groups;
        }

        Ok(())
    }

    /// Rename an existing group in a collection
    pub fn rename_group(
        &mut self,
        collection_path: &str,
        old_group_name: &str,
        new_group_name: &str,
    ) -> Result<()> {
        // Validate new group name
        if new_group_name.is_empty() {
            return Err(anyhow::anyhow!("Group name cannot be empty"));
        }

        if self.is_opencollection(collection_path) {
            return self.rename_group_opencollection(
                collection_path,
                old_group_name,
                new_group_name,
            );
        }

        // Sanitize new group name for filesystem
        let sanitized_new_name = new_group_name
            .chars()
            .map(|c| match c {
                '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
                _ => c,
            })
            .collect::<String>();

        // Get collection info
        let collection_info = self.collections.get(collection_path).ok_or_else(|| {
            anyhow::anyhow!("Collection with path '{}' not found", collection_path)
        })?;

        // Get the old group path
        let old_group_path = collection_info
            .groups
            .get(old_group_name)
            .ok_or_else(|| anyhow::anyhow!("Group '{}' not found in collection", old_group_name))?
            .path
            .clone();

        let collection_dir = Path::new(&collection_info.data.path);
        let old_group_dir = collection_dir.join(&old_group_path);
        let new_group_dir = collection_dir.join(&sanitized_new_name);

        // Check if new name already exists (and it's not the same as old name)
        if new_group_dir.exists() && sanitized_new_name != old_group_path {
            return Err(anyhow::anyhow!(
                "Group '{}' already exists in collection",
                sanitized_new_name
            ));
        }

        // Rename the directory
        fs::rename(&old_group_dir, &new_group_dir).with_context(|| {
            format!(
                "Failed to rename group directory from {:?} to {:?}",
                old_group_dir, new_group_dir
            )
        })?;

        tracing::info!(
            "Group '{}' renamed to '{}' in collection '{}'",
            old_group_name,
            sanitized_new_name,
            collection_info.data.name
        );

        // Reload the collection to pick up the renamed group
        self.reload_collection(collection_path)?;

        Ok(())
    }

    /// Delete a group from a collection (including all requests inside)
    pub fn delete_group(&mut self, collection_path: &str, group_name: &str) -> Result<()> {
        if self.is_opencollection(collection_path) {
            return self.delete_group_opencollection(collection_path, group_name);
        }

        // Get collection info
        let collection_info = self.collections.get(collection_path).ok_or_else(|| {
            anyhow::anyhow!("Collection with path '{}' not found", collection_path)
        })?;

        // Get the group path from the collection
        let group_path = collection_info
            .groups
            .get(group_name)
            .ok_or_else(|| anyhow::anyhow!("Group '{}' not found in collection", group_name))?
            .path
            .clone();

        // Get the full path to the group directory
        let collection_dir = Path::new(&collection_info.data.path);
        let full_group_path = collection_dir.join(&group_path);

        // Delete the entire group directory
        if full_group_path.exists() {
            fs::remove_dir_all(&full_group_path).with_context(|| {
                format!("Failed to delete group directory {:?}", full_group_path)
            })?;
        }

        tracing::info!(
            "Group '{}' deleted from collection '{}' (removed directory: {:?})",
            group_name,
            collection_info.data.name,
            full_group_path
        );

        // Reload the collection to update state and trigger observers
        self.reload_collection(collection_path)?;

        Ok(())
    }

    /// Add an environment to an existing collection
    pub fn add_environment_to_collection(
        &mut self,
        collection_path: &str,
        environment: EnvironmentToml,
    ) -> Result<()> {
        let collection_data = {
            let Some(collection_info) = self.collections.get_mut(collection_path) else {
                return Err(anyhow::anyhow!("Collection not found: {}", collection_path));
            };
            collection_info.toml.environments.push(environment);
            collection_info.toml.clone()
        };
        self.save_collection(&collection_data, collection_path)?;
        Ok(())
    }

    /// True if the collection at `collection_path` uses the OpenCollection format.
    fn is_opencollection(&self, collection_path: &str) -> bool {
        self.collections
            .get(collection_path)
            .map(|c| c.format == CollectionFormat::OpenCollection)
            .unwrap_or(false)
    }

    /// Re-serialize an OpenCollection collection to disk from its in-memory
    /// model. Bundled collections are written as a single `opencollection.yml`;
    /// non-bundled collections are written as a directory tree. Each request is
    /// merged into its retained source item so unmodeled fields are preserved.
    fn persist_opencollection(&self, collection_path: &str) -> Result<()> {
        let info = self
            .collections
            .get(collection_path)
            .ok_or_else(|| anyhow::anyhow!("Collection with path {} not found", collection_path))?;
        let dir = Path::new(&info.data.path).to_path_buf();

        let bundled = info
            .oc_source
            .as_ref()
            .and_then(|f| f.bundled)
            .unwrap_or(true);

        if !bundled {
            return self.persist_opencollection_unbundled(&dir, info);
        }

        // Root items, sorted by name for deterministic output.
        let root_items: Vec<OcItem> = sorted_request_items(&info.requests, &info.oc_items);

        // Folder items, sorted by group name.
        let mut group_names: Vec<&String> = info.groups.keys().collect();
        group_names.sort();
        let mut group_items = Vec::new();
        for gname in group_names {
            let group = &info.groups[gname];
            let children = sorted_request_items(&group.requests, &info.oc_items);
            group_items.push(opencollection::build_folder_item(
                info.oc_groups.get(gname),
                &group.name,
                children,
            ));
        }

        let mut items = root_items;
        items.extend(group_items);

        let environments: Vec<OcEnvironment> = info
            .toml
            .environments
            .iter()
            .map(|e| opencollection::merge_env(info.oc_envs.get(&e.name), e))
            .collect();

        let file = opencollection::assemble_bundled_file(
            info.oc_source.as_ref(),
            &info.toml.collection.name,
            &info.toml.collection.version,
            info.toml.collection.docs.as_deref(),
            items,
            environments,
            &info.toml.collection.vars,
        );
        let yaml = opencollection::to_yaml_string(&file)?;
        let target = opencollection::find_opencollection_file(&dir)
            .unwrap_or_else(|| dir.join("opencollection.yml"));
        fs::create_dir_all(&dir).ok();
        fs::write(&target, yaml)
            .with_context(|| format!("Failed to write OpenCollection to {:?}", target))?;
        Ok(())
    }

    /// Write a non-bundled OpenCollection as a directory tree: root
    /// `opencollection.yml`, `environments/<name>.yml`, and one directory per
    /// folder (`folder.yml` + a `.yml` per request). Stray files/dirs left by
    /// deletions/renames are reconciled away.
    fn persist_opencollection_unbundled(&self, dir: &Path, info: &CollectionInfo) -> Result<()> {
        fs::create_dir_all(dir).ok();

        // 1. Root opencollection.yml (structure lives on disk, not inline).
        let root = opencollection::build_unbundled_root(
            info.oc_source.as_ref(),
            &info.toml.collection.name,
            &info.toml.collection.version,
            info.toml.collection.docs.as_deref(),
            &info.toml.collection.vars,
        );
        let target = opencollection::find_opencollection_file(dir)
            .unwrap_or_else(|| dir.join("opencollection.yml"));
        fs::write(&target, opencollection::to_yaml_string(&root)?)
            .with_context(|| format!("Failed to write {:?}", target))?;

        // 2. Environments.
        let env_dir = dir.join("environments");
        let mut desired_env_files = std::collections::HashSet::new();
        if !info.toml.environments.is_empty() {
            fs::create_dir_all(&env_dir).ok();
        }
        for env in &info.toml.environments {
            let oc_env = opencollection::merge_env(info.oc_envs.get(&env.name), env);
            let fname = format!("{}.yml", sanitize_name(&env.name));
            fs::write(
                env_dir.join(&fname),
                opencollection::environment_to_yaml(&oc_env)?,
            )
            .with_context(|| format!("Failed to write environment {}", fname))?;
            desired_env_files.insert(fname);
        }
        remove_stray_yaml(&env_dir, &desired_env_files, &[]);

        // 3. Root request files.
        let mut desired_root_files = std::collections::HashSet::new();
        for (key, req) in &info.requests {
            let item = opencollection::merge_request_into_item(info.oc_items.get(key), req);
            let fname = format!("{}.yml", sanitize_name(&req.name));
            fs::write(dir.join(&fname), opencollection::item_to_yaml(&item)?)
                .with_context(|| format!("Failed to write request {}", fname))?;
            desired_root_files.insert(fname);
        }

        // 4. Folder directories.
        let mut desired_group_dirs = std::collections::HashSet::new();
        for (gname, group) in &info.groups {
            let group_dir_name = sanitize_name(gname);
            let group_dir = dir.join(&group_dir_name);
            fs::create_dir_all(&group_dir).ok();
            desired_group_dirs.insert(group_dir_name);

            let folder =
                opencollection::build_folder_config(info.oc_groups.get(gname), &group.name);
            fs::write(
                group_dir.join("folder.yml"),
                opencollection::item_to_yaml(&folder)?,
            )
            .with_context(|| "Failed to write folder.yml")?;

            let mut desired = std::collections::HashSet::new();
            for (key, req) in &group.requests {
                let item = opencollection::merge_request_into_item(info.oc_items.get(key), req);
                let fname = format!("{}.yml", sanitize_name(&req.name));
                fs::write(group_dir.join(&fname), opencollection::item_to_yaml(&item)?)
                    .with_context(|| format!("Failed to write request {}", fname))?;
                desired.insert(fname);
            }
            remove_stray_yaml(&group_dir, &desired, &["folder.yml"]);
        }

        // 5. Reconcile stray root request files and removed group directories.
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                let name = path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("")
                    .to_string();
                if path.is_dir() {
                    if name != "environments" && !desired_group_dirs.contains(&name) {
                        let _ = fs::remove_dir_all(&path);
                    }
                } else if is_request_yaml(&path) && !desired_root_files.contains(&name) {
                    let _ = fs::remove_file(&path);
                }
            }
        }

        Ok(())
    }

    // -- OpenCollection mutation variants (in-memory edits + full re-serialize) --

    fn save_request_opencollection(
        &mut self,
        collection_path: &str,
        request_data: &RequestData,
        request_name: &str,
        group_path: Option<&str>,
    ) -> Result<()> {
        {
            let info = self.collections.get_mut(collection_path).ok_or_else(|| {
                anyhow::anyhow!("Collection with path {} not found", collection_path)
            })?;
            let dir = Path::new(&info.data.path).to_path_buf();
            let key = oc_request_key(&dir, group_path, request_name);
            if let Some(group_path) = group_path {
                let group_name = group_name_from_path(group_path);
                let group = info
                    .groups
                    .entry(group_name.clone())
                    .or_insert_with(|| GroupInfo {
                        name: group_name.clone(),
                        requests: HashMap::new(),
                        path: sanitize_name(&group_name),
                    });
                group.requests.insert(key, request_data.clone());
            } else {
                info.requests.insert(key, request_data.clone());
            }
        }
        self.persist_opencollection(collection_path)
    }

    fn delete_request_opencollection(
        &mut self,
        collection_path: &str,
        request_data: &RequestData,
    ) -> Result<()> {
        {
            let info = self.collections.get_mut(collection_path).ok_or_else(|| {
                anyhow::anyhow!("Collection with path {} not found", collection_path)
            })?;
            let key = find_request_key(info, request_data)
                .ok_or_else(|| anyhow::anyhow!("Request not found in collection"))?;
            if info.requests.remove(&key).is_none() {
                for group in info.groups.values_mut() {
                    if group.requests.remove(&key).is_some() {
                        break;
                    }
                }
            }
        }
        self.persist_opencollection(collection_path)
    }

    fn move_request_opencollection(
        &mut self,
        collection_path: &str,
        request_data: &RequestData,
        target_group_path: Option<&str>,
    ) -> Result<()> {
        {
            let info = self.collections.get_mut(collection_path).ok_or_else(|| {
                anyhow::anyhow!("Collection with path {} not found", collection_path)
            })?;
            let dir = Path::new(&info.data.path).to_path_buf();

            if let Some(key) = find_request_key(info, request_data)
                && info.requests.remove(&key).is_none()
            {
                for group in info.groups.values_mut() {
                    if group.requests.remove(&key).is_some() {
                        break;
                    }
                }
            }

            let new_key = oc_request_key(&dir, target_group_path, &request_data.name);
            if let Some(group_path) = target_group_path {
                let group_name = group_name_from_path(group_path);
                let group = info
                    .groups
                    .entry(group_name.clone())
                    .or_insert_with(|| GroupInfo {
                        name: group_name.clone(),
                        requests: HashMap::new(),
                        path: sanitize_name(&group_name),
                    });
                group.requests.insert(new_key, request_data.clone());
            } else {
                info.requests.insert(new_key, request_data.clone());
            }
        }
        self.persist_opencollection(collection_path)
    }

    fn create_group_opencollection(
        &mut self,
        collection_path: &str,
        group_name: &str,
    ) -> Result<()> {
        {
            let info = self.collections.get_mut(collection_path).ok_or_else(|| {
                anyhow::anyhow!("Collection with path '{}' not found", collection_path)
            })?;
            if info.groups.contains_key(group_name) {
                return Err(anyhow::anyhow!(
                    "Group '{}' already exists in collection",
                    group_name
                ));
            }
            info.groups.insert(
                group_name.to_string(),
                GroupInfo {
                    name: group_name.to_string(),
                    requests: HashMap::new(),
                    path: sanitize_name(group_name),
                },
            );
        }
        self.persist_opencollection(collection_path)
    }

    fn rename_group_opencollection(
        &mut self,
        collection_path: &str,
        old_group_name: &str,
        new_group_name: &str,
    ) -> Result<()> {
        {
            let info = self.collections.get_mut(collection_path).ok_or_else(|| {
                anyhow::anyhow!("Collection with path '{}' not found", collection_path)
            })?;
            let mut group = info
                .groups
                .remove(old_group_name)
                .ok_or_else(|| anyhow::anyhow!("Group '{}' not found", old_group_name))?;
            let dir = Path::new(&info.data.path).to_path_buf();
            group.name = new_group_name.to_string();
            group.path = sanitize_name(new_group_name);
            // Rewrite child keys so they reflect the new group segment.
            let rekeyed: HashMap<String, RequestData> = group
                .requests
                .drain()
                .map(|(_, req)| (oc_request_key(&dir, Some(new_group_name), &req.name), req))
                .collect();
            group.requests = rekeyed;
            info.groups.insert(new_group_name.to_string(), group);
        }
        self.persist_opencollection(collection_path)
    }

    fn delete_group_opencollection(
        &mut self,
        collection_path: &str,
        group_name: &str,
    ) -> Result<()> {
        {
            let info = self.collections.get_mut(collection_path).ok_or_else(|| {
                anyhow::anyhow!("Collection with path '{}' not found", collection_path)
            })?;
            info.groups.remove(group_name);
        }
        self.persist_opencollection(collection_path)
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }
}

/// Locate the in-memory key of a request within a collection, matched by
/// name + method + url (the same identity used by the native mutation methods).
fn find_request_key(info: &CollectionInfo, request_data: &RequestData) -> Option<String> {
    let matches = |stored: &RequestData| {
        stored.name == request_data.name
            && stored.method == request_data.method
            && stored.url == request_data.url
    };
    if let Some((key, _)) = info.requests.iter().find(|(_, r)| matches(r)) {
        return Some(key.clone());
    }
    for group in info.groups.values() {
        if let Some((key, _)) = group.requests.iter().find(|(_, r)| matches(r)) {
            return Some(key.clone());
        }
    }
    None
}

/// Derive the group name from the relative group path passed by the UI.
fn group_name_from_path(group_path: &str) -> String {
    Path::new(group_path)
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or(group_path)
        .to_string()
}

/// Build the OpenCollection items for a set of requests, merged with their
/// retained sources and sorted by request name for deterministic output.
fn sorted_request_items(
    requests: &HashMap<String, RequestData>,
    sources: &HashMap<String, OcItem>,
) -> Vec<OcItem> {
    let mut entries: Vec<(&String, &RequestData)> = requests.iter().collect();
    entries.sort_by(|a, b| a.1.name.cmp(&b.1.name));
    entries
        .into_iter()
        .map(|(key, req)| opencollection::merge_request_into_item(sources.get(key), req))
        .collect()
}

/// A `.yml`/`.yaml` file that is not the collection marker file.
fn is_request_yaml(path: &Path) -> bool {
    if !path.is_file() {
        return false;
    }
    let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
        return false;
    };
    name != "opencollection.yml"
        && name != "opencollection.yaml"
        && (name.ends_with(".yml") || name.ends_with(".yaml"))
}

/// Remove `.yml`/`.yaml` files in `dir` whose names are not in `keep`,
/// ignoring the names listed in `always_keep`.
fn remove_stray_yaml(dir: &Path, keep: &std::collections::HashSet<String>, always_keep: &[&str]) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();
        let is_yaml = name.ends_with(".yml") || name.ends_with(".yaml");
        if is_yaml && !keep.contains(&name) && !always_keep.contains(&name.as_str()) {
            let _ = fs::remove_file(&path);
        }
    }
}

impl Global for CollectionManager {}

/// Sanitize a name for use as a filesystem path segment.
pub(crate) fn sanitize_name(name: &str) -> String {
    name.chars()
        .map(|c| match c {
            '<' | '>' | ':' | '"' | '/' | '\\' | '|' | '?' | '*' => '_',
            _ => c,
        })
        .collect()
}

/// Build the stable in-memory key for an OpenCollection request. For bundled
/// collections this key is virtual (no file is written at it); for non-bundled
/// collections it is the real `.yml` path. Either way it is the identity used by
/// the mutation methods, so it must be derived consistently.
pub(crate) fn oc_request_key(dir: &Path, group: Option<&str>, request_name: &str) -> String {
    let mut path = dir.to_path_buf();
    if let Some(group) = group {
        path = path.join(sanitize_name(group));
    }
    path.join(format!("{}.yml", sanitize_name(request_name)))
        .to_string_lossy()
        .to_string()
}

/// Assemble a `CollectionInfo` from a decomposed OpenCollection.
fn assemble_opencollection_info(
    dir: &Path,
    data: CollectionData,
    loaded: LoadedCollection,
) -> CollectionInfo {
    let toml = CollectionToml {
        collection: CollectionMeta {
            name: loaded.name.clone(),
            version: loaded.version.clone(),
            collection_type: "collection".to_string(),
            docs: loaded.docs.clone(),
            ignore: Vec::new(),
            auth: None,
            vars: loaded.vars.clone(),
        },
        environments: loaded.environments.iter().map(|e| e.toml.clone()).collect(),
    };

    let mut oc_envs = HashMap::new();
    for env in &loaded.environments {
        oc_envs.insert(env.toml.name.clone(), env.source.clone());
    }

    let mut requests = HashMap::new();
    let mut oc_items = HashMap::new();
    for loaded_req in &loaded.root_requests {
        let key = oc_request_key(dir, None, &loaded_req.request.name);
        requests.insert(key.clone(), loaded_req.request.clone());
        oc_items.insert(key, loaded_req.source.clone());
    }

    let mut groups = HashMap::new();
    let mut oc_groups = HashMap::new();
    for group in &loaded.groups {
        let mut group_requests = HashMap::new();
        for loaded_req in &group.requests {
            let key = oc_request_key(dir, Some(&group.name), &loaded_req.request.name);
            group_requests.insert(key.clone(), loaded_req.request.clone());
            oc_items.insert(key, loaded_req.source.clone());
        }
        groups.insert(
            group.name.clone(),
            GroupInfo {
                name: group.name.clone(),
                requests: group_requests,
                path: sanitize_name(&group.name),
            },
        );
        if let Some(source) = &group.source {
            oc_groups.insert(group.name.clone(), source.clone());
        }
    }

    CollectionInfo {
        data,
        toml,
        requests,
        groups,
        format: CollectionFormat::OpenCollection,
        oc_source: Some(loaded.file),
        oc_items,
        oc_groups,
        oc_envs,
        runtime_vars: HashMap::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    use crate::collections::types::CollectionMeta;

    fn make_collection_with_environments() -> CollectionToml {
        let mut variables = HashMap::new();
        variables.insert(
            "BASE_URL".to_string(),
            EnvironmentVariable {
                value: "https://api.example.com".to_string(),
                secret: false,
                temporary: false,
            },
        );
        variables.insert(
            "API_KEY".to_string(),
            EnvironmentVariable {
                value: "".to_string(),
                secret: true,
                temporary: false,
            },
        );

        CollectionToml {
            collection: CollectionMeta {
                name: "Test Collection".to_string(),
                version: "1.0.0".to_string(),
                collection_type: "collection".to_string(),
                docs: Some("A test collection".to_string()),
                ignore: Vec::new(),
                auth: None,
                vars: Vec::new(),
            },
            environments: vec![
                EnvironmentToml {
                    name: "Development".to_string(),
                    variables: variables.clone(),
                },
                EnvironmentToml {
                    name: "Production".to_string(),
                    variables,
                },
            ],
        }
    }

    #[test]
    fn test_save_and_reload_collection_with_environments() {
        let temp_dir = std::env::temp_dir().join("broquest_test_save_reload");
        let _ = fs::remove_dir_all(&temp_dir);
        let collection_path = temp_dir.to_string_lossy().to_string();

        let collection_data = make_collection_with_environments();

        let mut manager = CollectionManager::new();
        manager
            .save_collection(&collection_data, &collection_path)
            .expect("save_collection should succeed");

        // Verify the file was written
        let toml_path = temp_dir.join("collection.toml");
        assert!(toml_path.exists(), "collection.toml should exist on disk");

        // Read back from disk
        let loaded = manager
            .read_collection_toml(&temp_dir)
            .expect("read_collection_toml should succeed");

        assert_eq!(loaded.environments.len(), 2);
        assert_eq!(
            loaded.environments[0].name,
            collection_data.environments[0].name
        );
        assert_eq!(
            loaded.environments[1].name,
            collection_data.environments[1].name
        );

        // Verify variables survived the roundtrip
        for (idx, env) in loaded.environments.iter().enumerate() {
            let original = &collection_data.environments[idx];
            assert_eq!(
                env.variables.len(),
                original.variables.len(),
                "Environment '{}' should have the same number of variables",
                env.name
            );
            for (key, var) in &env.variables {
                let original_var = original
                    .variables
                    .get(key)
                    .unwrap_or_else(|| panic!("Variable '{}' should exist in original", key));
                assert_eq!(
                    var.value, original_var.value,
                    "Value mismatch for '{}'",
                    key
                );
                assert_eq!(
                    var.secret, original_var.secret,
                    "Secret flag mismatch for '{}'",
                    key
                );
            }
        }

        // Verify the in-memory cache was also updated
        let cached = manager
            .get_collection_environments(&collection_path)
            .expect("cached environments should exist");
        assert_eq!(cached.len(), 2);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_save_collection_twice_preserves_environments() {
        let temp_dir = std::env::temp_dir().join("broquest_test_save_twice");
        let _ = fs::remove_dir_all(&temp_dir);
        let collection_path = temp_dir.to_string_lossy().to_string();

        let mut manager = CollectionManager::new();

        // First save: empty environments (simulates creating a new collection)
        let empty_collection = CollectionToml {
            collection: CollectionMeta {
                name: "My Collection".to_string(),
                version: "1.0.0".to_string(),
                collection_type: "collection".to_string(),
                docs: None,
                ignore: Vec::new(),
                auth: None,
                vars: Vec::new(),
            },
            environments: vec![],
        };
        manager
            .save_collection(&empty_collection, &collection_path)
            .expect("first save should succeed");

        let loaded = manager
            .read_collection_toml(&temp_dir)
            .expect("should read after first save");
        assert_eq!(
            loaded.environments.len(),
            0,
            "should have no environments after first save"
        );

        // Second save: with environments (simulates user adding environments then saving)
        let collection_with_envs = make_collection_with_environments();
        manager
            .save_collection(&collection_with_envs, &collection_path)
            .expect("second save should succeed");

        let loaded = manager
            .read_collection_toml(&temp_dir)
            .expect("should read after second save");
        assert_eq!(
            loaded.environments.len(),
            2,
            "should have 2 environments after second save"
        );

        // Verify secrets are present in the TOML
        let has_secret = loaded
            .environments
            .iter()
            .any(|env| env.variables.values().any(|var| var.secret));
        assert!(
            has_secret,
            "at least one secret variable should be persisted in the TOML"
        );

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    const OC_YAML: &str = r#"opencollection: "1.0.0"
info:
  name: OC Test
bundled: true
items:
  - info:
      name: Existing
      type: http
    http:
      method: GET
      url: https://example.com
"#;

    fn write_oc_fixture(dir_name: &str) -> std::path::PathBuf {
        let temp_dir = std::env::temp_dir().join(dir_name);
        let _ = fs::remove_dir_all(&temp_dir);
        fs::create_dir_all(&temp_dir).unwrap();
        fs::write(temp_dir.join("opencollection.yml"), OC_YAML).unwrap();
        temp_dir
    }

    #[test]
    fn test_opencollection_save_request_rewrites_yaml() {
        let temp_dir = write_oc_fixture("broquest_test_oc_save");
        let dir_str = temp_dir.to_string_lossy().to_string();

        let mut manager = CollectionManager::new();
        manager.load_opencollection_dir(&temp_dir).expect("load oc");

        let req = RequestData {
            name: "NewReq".to_string(),
            url: "https://example.com/new".to_string(),
            ..Default::default()
        };
        manager
            .save_request(&dir_str, &req, "NewReq", None)
            .expect("save request");

        // The YAML file was rewritten with both requests.
        let content = fs::read_to_string(temp_dir.join("opencollection.yml")).unwrap();
        assert!(
            content.contains("NewReq"),
            "new request should be persisted"
        );
        assert!(
            content.contains("Existing"),
            "existing request should remain"
        );

        // No stray native .toml files were created.
        let toml_count = fs::read_dir(&temp_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().extension().map(|x| x == "toml").unwrap_or(false))
            .count();
        assert_eq!(
            toml_count, 0,
            "OpenCollection save must not write .toml files"
        );

        // Reloading from disk shows the new request and preserves the format.
        let mut manager2 = CollectionManager::new();
        manager2
            .load_opencollection_dir(&temp_dir)
            .expect("reload oc");
        let info = manager2.get_collection_by_path(&dir_str).unwrap();
        assert_eq!(info.format, CollectionFormat::OpenCollection);
        assert!(info.requests.values().any(|r| r.name == "NewReq"));
        assert!(info.requests.values().any(|r| r.name == "Existing"));

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_opencollection_group_ops_no_disk_rescan() {
        let temp_dir = write_oc_fixture("broquest_test_oc_groups");
        let dir_str = temp_dir.to_string_lossy().to_string();

        let mut manager = CollectionManager::new();
        manager.load_opencollection_dir(&temp_dir).expect("load oc");

        // Create a group and add a request into it.
        manager
            .create_group(&dir_str, "Users")
            .expect("create group");
        let req = RequestData {
            name: "GetUser".to_string(),
            url: "https://example.com/users/1".to_string(),
            ..Default::default()
        };
        manager
            .save_request(&dir_str, &req, "GetUser", Some("Users"))
            .expect("save request in group");

        // Rename the group; the child request must move with it.
        manager
            .rename_group(&dir_str, "Users", "People")
            .expect("rename group");

        let content = fs::read_to_string(temp_dir.join("opencollection.yml")).unwrap();
        assert!(
            content.contains("People"),
            "renamed group should be present"
        );
        assert!(
            !content.contains("name: Users"),
            "old group name should be gone"
        );
        assert!(
            content.contains("GetUser"),
            "request should follow the group"
        );

        // No group directories were created on disk.
        assert!(!temp_dir.join("Users").exists());
        assert!(!temp_dir.join("People").exists());

        // Delete the group.
        manager
            .delete_group(&dir_str, "People")
            .expect("delete group");
        let content = fs::read_to_string(temp_dir.join("opencollection.yml")).unwrap();
        assert!(
            !content.contains("GetUser"),
            "deleted group's request should be gone"
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_create_opencollection_non_bundled() {
        let temp_dir = std::env::temp_dir().join("broquest_test_oc_create");
        let _ = fs::remove_dir_all(&temp_dir);
        let dir_str = temp_dir.to_string_lossy().to_string();

        let mut manager = CollectionManager::new();
        let toml = CollectionToml {
            collection: CollectionMeta {
                name: "New OC".to_string(),
                version: "1.0.0".to_string(),
                collection_type: "collection".to_string(),
                docs: None,
                ignore: Vec::new(),
                auth: None,
                vars: Vec::new(),
            },
            environments: Vec::new(),
        };
        manager
            .create_opencollection(&toml, &dir_str)
            .expect("create opencollection");

        let root = fs::read_to_string(temp_dir.join("opencollection.yml")).unwrap();
        assert!(
            root.contains("bundled: false"),
            "new OC defaults to non-bundled"
        );
        assert!(root.contains("New OC"));

        // Reloading picks it up as OpenCollection.
        let mut manager2 = CollectionManager::new();
        manager2.load_opencollection_dir(&temp_dir).expect("reload");
        assert_eq!(
            manager2.get_collection_by_path(&dir_str).unwrap().format,
            CollectionFormat::OpenCollection
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    fn copy_dir_all(src: &Path, dst: &Path) {
        fs::create_dir_all(dst).unwrap();
        for entry in fs::read_dir(src).unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let dest = dst.join(entry.file_name());
            if path.is_dir() {
                copy_dir_all(&path, &dest);
            } else {
                fs::copy(&path, &dest).unwrap();
            }
        }
    }

    /// Load the real Bruno-exported "Swagger Petstore" (non-bundled) collection,
    /// save it back, and assert nothing is lost.
    #[test]
    fn test_petstore_nonbundled_roundtrip_lossless() {
        let src = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("resources")
            .join("Swagger Petstore");
        if !src.exists() {
            eprintln!("Petstore fixture missing; skipping");
            return;
        }
        let temp_dir = std::env::temp_dir().join("broquest_test_petstore");
        let _ = fs::remove_dir_all(&temp_dir);
        copy_dir_all(&src, &temp_dir);
        let dir_str = temp_dir.to_string_lossy().to_string();

        let mut manager = CollectionManager::new();
        manager
            .load_opencollection_dir(&temp_dir)
            .expect("load petstore");

        let list_path = temp_dir.join("pets").join("List all pets.yml");
        let original: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(&fs::read_to_string(&list_path).unwrap()).unwrap();

        let (req, names_before) = {
            let info = manager.get_collection_by_path(&dir_str).unwrap();
            assert_eq!(info.format, CollectionFormat::OpenCollection);
            let group = info.groups.get("pets").expect("pets group");
            assert_eq!(
                group.requests.len(),
                3,
                "petstore has 3 requests under pets"
            );
            let names: std::collections::BTreeSet<String> =
                group.requests.values().map(|r| r.name.clone()).collect();
            let req = group
                .requests
                .values()
                .find(|r| r.name == "List all pets")
                .expect("List all pets request")
                .clone();
            (req, names)
        };

        // Saving an unchanged request must rewrite the tree losslessly.
        manager
            .save_request(&dir_str, &req, "List all pets", Some("pets"))
            .expect("save request");

        let after: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(&fs::read_to_string(&list_path).unwrap()).unwrap();
        assert_eq!(
            original, after,
            "an unchanged request must round-trip byte-for-byte (semantically)"
        );

        // Reload from disk and verify the structure is intact.
        let mut manager2 = CollectionManager::new();
        manager2.load_opencollection_dir(&temp_dir).expect("reload");
        let info2 = manager2.get_collection_by_path(&dir_str).unwrap();
        let group2 = info2.groups.get("pets").expect("pets group after reload");
        let names_after: std::collections::BTreeSet<String> =
            group2.requests.values().map(|r| r.name.clone()).collect();
        assert_eq!(names_before, names_after, "request set must be preserved");

        // Environment file preserved with its variable.
        let env_path = temp_dir.join("environments").join("Environment 1.yml");
        assert!(env_path.exists(), "environment file must survive");
        let env_content = fs::read_to_string(&env_path).unwrap();
        assert!(env_content.contains("baseUrl"), "env variable must survive");

        // Root file remains non-bundled and keeps the bruno extensions block.
        let root = fs::read_to_string(temp_dir.join("opencollection.yml")).unwrap();
        assert!(root.contains("bundled: false"));
        assert!(root.contains("bruno"), "extensions.bruno must be preserved");

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[test]
    fn test_runtime_vars_survive_save_collection() {
        // Runtime vars are in-memory only; they must be preserved when a native
        // collection is re-saved (which rebuilds CollectionInfo from scratch).
        let temp_dir = std::env::temp_dir().join("broquest_test_runtime_vars");
        let _ = fs::remove_dir_all(&temp_dir);
        let collection_path = temp_dir.to_string_lossy().to_string();

        let collection_data = make_collection_with_environments();
        let mut manager = CollectionManager::new();
        manager
            .save_collection(&collection_data, &collection_path)
            .expect("save_collection should succeed");

        // Set a runtime var via the manager.
        let mut vars = HashMap::new();
        vars.insert(
            "userId".to_string(),
            serde_json::Value::String("usr_123".to_string()),
        );
        manager
            .update_runtime_vars(&collection_path, vars)
            .expect("update_runtime_vars should succeed");
        assert_eq!(
            manager
                .get_collection_by_path(&collection_path)
                .unwrap()
                .runtime_vars
                .get("userId")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string()),
            Some("usr_123".to_string())
        );

        // Re-save the collection; runtime vars must survive.
        manager
            .save_collection(&collection_data, &collection_path)
            .expect("second save should succeed");
        assert_eq!(
            manager
                .get_collection_by_path(&collection_path)
                .unwrap()
                .runtime_vars
                .len(),
            1,
            "runtime vars must be preserved across save_collection"
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
