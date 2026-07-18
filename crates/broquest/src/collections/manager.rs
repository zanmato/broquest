use super::format::CollectionFormat;
use super::opencollection::{self, LoadedCollection, OcEnvironment, OcItem, OpenCollectionFile};
use super::storage::storage_for;
use super::types::{
    CollectionMeta, CollectionToml, EnvironmentToml, EnvironmentVariable, RequestToml,
};
use crate::app_database::{AppDatabase, CollectionData};
use crate::domain::RequestData;
use anyhow::{Context as _, Result};
use gpui::{App, Context, Entity, EventEmitter, Global, SharedString};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Typed change notifications emitted by [`CollectionManager`]. Views subscribe
/// to the manager entity and react to the specific change that concerns them.
#[derive(Clone, Debug)]
#[allow(clippy::enum_variant_names)]
pub enum CollectionManagerEvent {
    /// A collection's environment set or environment variables changed.
    EnvironmentsChanged { collection_path: SharedString },
    /// The set of collections or their structure (groups) changed.
    CollectionsChanged,
    /// A collection's requests changed (saved, deleted, moved).
    RequestsChanged { collection_path: SharedString },
    /// A collection's in-memory runtime variables changed.
    RuntimeVarsChanged { collection_path: SharedString },
}

/// Handle to the global [`CollectionManager`] entity, following the Zed pattern
/// for entity-backed globals.
struct GlobalCollectionManager(Entity<CollectionManager>);

impl Global for GlobalCollectionManager {}

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
    pub fn load_saved(&mut self, cx: &mut Context<Self>) -> Result<()> {
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

        cx.emit(CollectionManagerEvent::CollectionsChanged);
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
        let (requests, groups) = Self::load_collection_structure(dir)?;
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
        cx: &mut Context<Self>,
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
        let info = self
            .collections
            .get(path)
            .expect("collection was just inserted");
        storage_for(info.format).save_collection(info)?;
        cx.emit(CollectionManagerEvent::CollectionsChanged);
        Ok(())
    }

    /// Load an OpenCollection directory into the manager and return the
    /// synthesized collection metadata (mirrors [`Self::load_collection_toml`]).
    pub fn load_opencollection_dir(
        &mut self,
        dir: &Path,
        cx: &mut Context<Self>,
    ) -> Result<CollectionToml> {
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
        cx.emit(CollectionManagerEvent::CollectionsChanged);
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
    pub fn load_collection_toml(
        &mut self,
        collection_dir: &Path,
        cx: &mut Context<Self>,
    ) -> Result<CollectionToml> {
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
        let (requests, groups) = Self::load_collection_structure(collection_dir)?;

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

        cx.emit(CollectionManagerEvent::CollectionsChanged);
        Ok(collection_toml)
    }

    /// Load all requests and groups from a collection directory
    pub(crate) fn load_collection_structure(
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

                let group_requests = Self::load_group_requests(&path)?;
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
                match Self::load_request_file(&path) {
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
    pub(crate) fn load_group_requests(group_dir: &Path) -> Result<HashMap<String, RequestData>> {
        let mut requests = HashMap::new();

        for entry in fs::read_dir(group_dir)? {
            let entry = entry?;
            let path = entry.path();

            if let Some(filename) = path.file_name()
                && let Some(filename_str) = filename.to_str()
                && filename_str.ends_with(".toml")
            {
                match Self::load_request_file(&path) {
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
    pub fn save_collection(
        &mut self,
        collection_data: &CollectionToml,
        path: &str,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let now = chrono::Utc::now();

        // Update the in-memory model in place (preserving requests, groups,
        // runtime vars, and OpenCollection sources) or create a fresh native
        // collection. OpenCollection collections are created via
        // [`Self::create_opencollection`], so a not-yet-cached collection here
        // is always native TOML.
        match self.collections.get_mut(path) {
            Some(info) => {
                info.toml = collection_data.clone();
                info.data.name = collection_data.collection.name.clone();
                info.data.updated_at = now;
            }
            None => {
                let info = CollectionInfo {
                    data: CollectionData {
                        id: None,
                        name: collection_data.collection.name.clone(),
                        path: path.to_string(),
                        position: 0,
                        format: CollectionFormat::Broquest.as_db_str().to_string(),
                        created_at: now,
                        updated_at: now,
                    },
                    toml: collection_data.clone(),
                    requests: HashMap::new(),
                    groups: HashMap::new(),
                    format: CollectionFormat::Broquest,
                    oc_source: None,
                    oc_items: HashMap::new(),
                    oc_groups: HashMap::new(),
                    oc_envs: HashMap::new(),
                    runtime_vars: HashMap::new(),
                };
                self.collections.insert(path.to_string(), info);
            }
        }

        let info = self
            .collections
            .get(path)
            .expect("collection was just inserted or updated");
        storage_for(info.format).save_collection(info)?;

        tracing::info!(
            "Collection '{}' saved and cached at path: {}",
            collection_data.collection.name,
            path
        );

        self.emit_collection_saved(path, cx);
        Ok(())
    }

    /// Emit the change events that follow a collection save: environments and
    /// collection-level vars may both have changed, so notify both audiences.
    fn emit_collection_saved(&self, path: &str, cx: &mut Context<Self>) {
        cx.emit(CollectionManagerEvent::EnvironmentsChanged {
            collection_path: path.to_string().into(),
        });
        cx.emit(CollectionManagerEvent::CollectionsChanged);
    }

    /// Remove a collection from the manager by path
    pub fn remove_collection(&mut self, collection_path: &str, cx: &mut Context<Self>) {
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
        cx.emit(CollectionManagerEvent::CollectionsChanged);
    }

    /// Save a request to a collection directory or group
    pub fn save_request(
        &mut self,
        collection_path: &str,
        request_data: &RequestData,
        request_name: &str,
        group_path: Option<&str>,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let info = self
            .collections
            .get_mut(collection_path)
            .ok_or_else(|| anyhow::anyhow!("Collection with path {} not found", collection_path))?;
        storage_for(info.format).save_request(info, request_data, request_name, group_path)?;
        cx.emit(CollectionManagerEvent::RequestsChanged {
            collection_path: collection_path.to_string().into(),
        });
        Ok(())
    }

    /// Delete a request from a collection
    pub fn delete_request(
        &mut self,
        collection_path: &str,
        request_data: &RequestData,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let info = self
            .collections
            .get_mut(collection_path)
            .ok_or_else(|| anyhow::anyhow!("Collection with path {} not found", collection_path))?;
        storage_for(info.format).delete_request(info, request_data)?;
        cx.emit(CollectionManagerEvent::RequestsChanged {
            collection_path: collection_path.to_string().into(),
        });
        Ok(())
    }

    /// Load a single request file
    pub(crate) fn load_request_file(file_path: &Path) -> Result<RequestData> {
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
        cx: &mut Context<Self>,
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

        // Persist the updated environments in the collection's on-disk format.
        let collection_name = collection_info.data.name.clone();
        storage_for(collection_info.format).save_collection(collection_info)?;

        tracing::info!(
            "Environment variables updated and saved for collection '{}', environment '{}'",
            collection_name,
            environment_name
        );

        cx.emit(CollectionManagerEvent::EnvironmentsChanged {
            collection_path: collection_path.to_string().into(),
        });
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
        cx: &mut Context<Self>,
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
        cx.emit(CollectionManagerEvent::RuntimeVarsChanged {
            collection_path: collection_path.to_string().into(),
        });
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
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let info = self
            .collections
            .get_mut(collection_path)
            .ok_or_else(|| anyhow::anyhow!("Collection with path {} not found", collection_path))?;
        storage_for(info.format).move_request(info, request_data, target_group_path)?;
        cx.emit(CollectionManagerEvent::RequestsChanged {
            collection_path: collection_path.to_string().into(),
        });
        Ok(())
    }

    /// Create a new group in a collection
    pub fn create_group(
        &mut self,
        collection_path: &str,
        group_name: &str,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        // Validate group name
        if group_name.is_empty() {
            return Err(anyhow::anyhow!("Group name cannot be empty"));
        }

        let info = self.collections.get_mut(collection_path).ok_or_else(|| {
            anyhow::anyhow!("Collection with path '{}' not found", collection_path)
        })?;
        storage_for(info.format).create_group(info, group_name)?;
        cx.emit(CollectionManagerEvent::CollectionsChanged);
        Ok(())
    }

    /// Rename an existing group in a collection
    pub fn rename_group(
        &mut self,
        collection_path: &str,
        old_group_name: &str,
        new_group_name: &str,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        // Validate new group name
        if new_group_name.is_empty() {
            return Err(anyhow::anyhow!("Group name cannot be empty"));
        }

        let info = self.collections.get_mut(collection_path).ok_or_else(|| {
            anyhow::anyhow!("Collection with path '{}' not found", collection_path)
        })?;
        storage_for(info.format).rename_group(info, old_group_name, new_group_name)?;
        cx.emit(CollectionManagerEvent::CollectionsChanged);
        Ok(())
    }

    /// Delete a group from a collection (including all requests inside)
    pub fn delete_group(
        &mut self,
        collection_path: &str,
        group_name: &str,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let info = self.collections.get_mut(collection_path).ok_or_else(|| {
            anyhow::anyhow!("Collection with path '{}' not found", collection_path)
        })?;
        storage_for(info.format).delete_group(info, group_name)?;
        cx.emit(CollectionManagerEvent::CollectionsChanged);
        Ok(())
    }

    /// Add an environment to an existing collection
    pub fn add_environment_to_collection(
        &mut self,
        collection_path: &str,
        environment: EnvironmentToml,
        cx: &mut Context<Self>,
    ) -> Result<()> {
        let collection_data = {
            let Some(collection_info) = self.collections.get_mut(collection_path) else {
                return Err(anyhow::anyhow!("Collection not found: {}", collection_path));
            };
            collection_info.toml.environments.push(environment);
            collection_info.toml.clone()
        };
        // save_collection emits EnvironmentsChanged + CollectionsChanged.
        self.save_collection(&collection_data, collection_path, cx)?;
        Ok(())
    }

    /// Return the global manager entity handle (a cheap clone).
    pub fn global(cx: &App) -> Entity<Self> {
        cx.global::<GlobalCollectionManager>().0.clone()
    }

    /// Install `entity` as the global manager.
    pub fn set_global(entity: Entity<Self>, cx: &mut App) {
        cx.set_global(GlobalCollectionManager(entity));
    }
}

impl EventEmitter<CollectionManagerEvent> for CollectionManager {}

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
    use gpui::AppContext as _;
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

    #[gpui::test]
    fn test_save_and_reload_collection_with_environments(cx: &mut gpui::TestAppContext) {
        let temp_dir = std::env::temp_dir().join("broquest_test_save_reload");
        let _ = fs::remove_dir_all(&temp_dir);
        let collection_path = temp_dir.to_string_lossy().to_string();

        let collection_data = make_collection_with_environments();

        let manager = cx.new(|_| CollectionManager::new());
        manager
            .update(cx, |manager, cx| {
                manager.save_collection(&collection_data, &collection_path, cx)
            })
            .expect("save_collection should succeed");

        // Verify the file was written
        let toml_path = temp_dir.join("collection.toml");
        assert!(toml_path.exists(), "collection.toml should exist on disk");

        // Read back from disk
        let loaded = manager
            .read_with(cx, |manager, _| manager.read_collection_toml(&temp_dir))
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
            .read_with(cx, |manager, _| {
                manager.get_collection_environments(&collection_path)
            })
            .expect("cached environments should exist");
        assert_eq!(cached.len(), 2);

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[gpui::test]
    fn test_save_collection_twice_preserves_environments(cx: &mut gpui::TestAppContext) {
        let temp_dir = std::env::temp_dir().join("broquest_test_save_twice");
        let _ = fs::remove_dir_all(&temp_dir);
        let collection_path = temp_dir.to_string_lossy().to_string();

        let manager = cx.new(|_| CollectionManager::new());

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
            .update(cx, |manager, cx| {
                manager.save_collection(&empty_collection, &collection_path, cx)
            })
            .expect("first save should succeed");

        let loaded = manager
            .read_with(cx, |manager, _| manager.read_collection_toml(&temp_dir))
            .expect("should read after first save");
        assert_eq!(
            loaded.environments.len(),
            0,
            "should have no environments after first save"
        );

        // Second save: with environments (simulates user adding environments then saving)
        let collection_with_envs = make_collection_with_environments();
        manager
            .update(cx, |manager, cx| {
                manager.save_collection(&collection_with_envs, &collection_path, cx)
            })
            .expect("second save should succeed");

        let loaded = manager
            .read_with(cx, |manager, _| manager.read_collection_toml(&temp_dir))
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

    #[gpui::test]
    fn test_opencollection_save_request_rewrites_yaml(cx: &mut gpui::TestAppContext) {
        let temp_dir = write_oc_fixture("broquest_test_oc_save");
        let dir_str = temp_dir.to_string_lossy().to_string();

        let manager = cx.new(|_| CollectionManager::new());
        manager
            .update(cx, |manager, cx| {
                manager.load_opencollection_dir(&temp_dir, cx)
            })
            .expect("load oc");

        let req = RequestData {
            name: "NewReq".to_string(),
            url: "https://example.com/new".to_string(),
            ..Default::default()
        };
        manager
            .update(cx, |manager, cx| {
                manager.save_request(&dir_str, &req, "NewReq", None, cx)
            })
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
        let manager2 = cx.new(|_| CollectionManager::new());
        manager2
            .update(cx, |manager, cx| {
                manager.load_opencollection_dir(&temp_dir, cx)
            })
            .expect("reload oc");
        manager2.read_with(cx, |manager, _| {
            let info = manager.get_collection_by_path(&dir_str).unwrap();
            assert_eq!(info.format, CollectionFormat::OpenCollection);
            assert!(info.requests.values().any(|r| r.name == "NewReq"));
            assert!(info.requests.values().any(|r| r.name == "Existing"));
        });

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[gpui::test]
    fn test_opencollection_group_ops_no_disk_rescan(cx: &mut gpui::TestAppContext) {
        let temp_dir = write_oc_fixture("broquest_test_oc_groups");
        let dir_str = temp_dir.to_string_lossy().to_string();

        let manager = cx.new(|_| CollectionManager::new());
        manager
            .update(cx, |manager, cx| {
                manager.load_opencollection_dir(&temp_dir, cx)
            })
            .expect("load oc");

        // Create a group and add a request into it.
        manager
            .update(cx, |manager, cx| {
                manager.create_group(&dir_str, "Users", cx)
            })
            .expect("create group");
        let req = RequestData {
            name: "GetUser".to_string(),
            url: "https://example.com/users/1".to_string(),
            ..Default::default()
        };
        manager
            .update(cx, |manager, cx| {
                manager.save_request(&dir_str, &req, "GetUser", Some("Users"), cx)
            })
            .expect("save request in group");

        // Rename the group; the child request must move with it.
        manager
            .update(cx, |manager, cx| {
                manager.rename_group(&dir_str, "Users", "People", cx)
            })
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
            .update(cx, |manager, cx| {
                manager.delete_group(&dir_str, "People", cx)
            })
            .expect("delete group");
        let content = fs::read_to_string(temp_dir.join("opencollection.yml")).unwrap();
        assert!(
            !content.contains("GetUser"),
            "deleted group's request should be gone"
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }

    #[gpui::test]
    fn test_create_opencollection_non_bundled(cx: &mut gpui::TestAppContext) {
        let temp_dir = std::env::temp_dir().join("broquest_test_oc_create");
        let _ = fs::remove_dir_all(&temp_dir);
        let dir_str = temp_dir.to_string_lossy().to_string();

        let manager = cx.new(|_| CollectionManager::new());
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
            .update(cx, |manager, cx| {
                manager.create_opencollection(&toml, &dir_str, cx)
            })
            .expect("create opencollection");

        let root = fs::read_to_string(temp_dir.join("opencollection.yml")).unwrap();
        assert!(
            root.contains("bundled: false"),
            "new OC defaults to non-bundled"
        );
        assert!(root.contains("New OC"));

        // Reloading picks it up as OpenCollection.
        let manager2 = cx.new(|_| CollectionManager::new());
        manager2
            .update(cx, |manager, cx| {
                manager.load_opencollection_dir(&temp_dir, cx)
            })
            .expect("reload");
        manager2.read_with(cx, |manager, _| {
            assert_eq!(
                manager.get_collection_by_path(&dir_str).unwrap().format,
                CollectionFormat::OpenCollection
            );
        });

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
    #[gpui::test]
    fn test_petstore_nonbundled_roundtrip_lossless(cx: &mut gpui::TestAppContext) {
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

        let manager = cx.new(|_| CollectionManager::new());
        manager
            .update(cx, |manager, cx| {
                manager.load_opencollection_dir(&temp_dir, cx)
            })
            .expect("load petstore");

        let list_path = temp_dir.join("pets").join("List all pets.yml");
        let original: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(&fs::read_to_string(&list_path).unwrap()).unwrap();

        let (req, names_before) = manager.read_with(cx, |manager, _| {
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
        });

        // Saving an unchanged request must rewrite the tree losslessly.
        manager
            .update(cx, |manager, cx| {
                manager.save_request(&dir_str, &req, "List all pets", Some("pets"), cx)
            })
            .expect("save request");

        let after: serde_yaml_ng::Value =
            serde_yaml_ng::from_str(&fs::read_to_string(&list_path).unwrap()).unwrap();
        assert_eq!(
            original, after,
            "an unchanged request must round-trip byte-for-byte (semantically)"
        );

        // Reload from disk and verify the structure is intact.
        let manager2 = cx.new(|_| CollectionManager::new());
        manager2
            .update(cx, |manager, cx| {
                manager.load_opencollection_dir(&temp_dir, cx)
            })
            .expect("reload");
        manager2.read_with(cx, |manager, _| {
            let info2 = manager.get_collection_by_path(&dir_str).unwrap();
            let group2 = info2.groups.get("pets").expect("pets group after reload");
            let names_after: std::collections::BTreeSet<String> =
                group2.requests.values().map(|r| r.name.clone()).collect();
            assert_eq!(names_before, names_after, "request set must be preserved");
        });

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

    #[gpui::test]
    fn test_runtime_vars_survive_save_collection(cx: &mut gpui::TestAppContext) {
        // Runtime vars are in-memory only; they must be preserved when a native
        // collection is re-saved (which rebuilds CollectionInfo from scratch).
        let temp_dir = std::env::temp_dir().join("broquest_test_runtime_vars");
        let _ = fs::remove_dir_all(&temp_dir);
        let collection_path = temp_dir.to_string_lossy().to_string();

        let collection_data = make_collection_with_environments();
        let manager = cx.new(|_| CollectionManager::new());
        manager
            .update(cx, |manager, cx| {
                manager.save_collection(&collection_data, &collection_path, cx)
            })
            .expect("save_collection should succeed");

        // Set a runtime var via the manager.
        let mut vars = HashMap::new();
        vars.insert(
            "userId".to_string(),
            serde_json::Value::String("usr_123".to_string()),
        );
        manager
            .update(cx, |manager, cx| {
                manager.update_runtime_vars(&collection_path, vars, cx)
            })
            .expect("update_runtime_vars should succeed");
        assert_eq!(
            manager.read_with(cx, |manager, _| manager
                .get_collection_by_path(&collection_path)
                .unwrap()
                .runtime_vars
                .get("userId")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())),
            Some("usr_123".to_string())
        );

        // Re-save the collection; runtime vars must survive.
        manager
            .update(cx, |manager, cx| {
                manager.save_collection(&collection_data, &collection_path, cx)
            })
            .expect("second save should succeed");
        assert_eq!(
            manager.read_with(cx, |manager, _| manager
                .get_collection_by_path(&collection_path)
                .unwrap()
                .runtime_vars
                .len()),
            1,
            "runtime vars must be preserved across save_collection"
        );

        let _ = fs::remove_dir_all(&temp_dir);
    }
}
