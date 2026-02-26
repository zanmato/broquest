use super::types::{CollectionToml, EnvironmentToml, EnvironmentVariable, RequestToml};
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
                    // Try to load the TOML data from the file system path
                    let collection_path = Path::new(&collection_data.path);
                    if collection_path.exists() {
                        match self.read_collection_toml(collection_path) {
                            Ok(collection_toml) => {
                                tracing::info!(
                                    "Loaded TOML for collection '{}' from database path: {}",
                                    collection_data.name,
                                    collection_data.path
                                );

                                // Load all requests and groups in this collection
                                let (requests, groups) =
                                    self.load_collection_structure(collection_path)?;

                                let collection_info = CollectionInfo {
                                    data: collection_data,
                                    toml: collection_toml,
                                    requests,
                                    groups,
                                };

                                local_collections
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
                created_at: chrono::Utc::now(),
                updated_at: chrono::Utc::now(),
            },
            toml: collection_toml.clone(),
            requests,
            groups,
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
        let (existing_requests, existing_groups, existing_created_at) =
            if let Some(existing_collection) = self.collections.get(path) {
                (
                    existing_collection.requests.clone(),
                    existing_collection.groups.clone(),
                    existing_collection.data.created_at,
                )
            } else {
                (HashMap::new(), HashMap::new(), chrono::Utc::now())
            };

        let collection_info = CollectionInfo {
            data: CollectionData {
                id: None, // We don't use IDs anymore
                name: collection_name.clone(),
                path: path.to_string(),
                position: 0,
                created_at: existing_created_at,
                updated_at: chrono::Utc::now(),
            },
            toml: collection_data.clone(),
            requests: existing_requests,
            groups: existing_groups,
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
        if let Some(collection_info) = self.collections.get_mut(collection_path) {
            collection_info.toml.environments.push(environment);

            // Save the updated collection
            let collection_data = collection_info.toml.clone();
            let _ = collection_info;
            self.save_collection(&collection_data, collection_path)?;
            Ok(())
        } else {
            Err(anyhow::anyhow!("Collection not found: {}", collection_path))
        }
    }

    pub fn global(cx: &App) -> &Self {
        cx.global::<Self>()
    }
}

impl Global for CollectionManager {}
