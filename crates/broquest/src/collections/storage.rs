//! Per-format persistence strategy for collections.
//!
//! [`CollectionManager`](super::manager::CollectionManager) owns the in-memory
//! state (the `collections` map, runtime vars) and delegates all disk I/O and
//! format-specific in-memory bookkeeping to a [`CollectionStorage`]
//! implementation chosen by [`CollectionInfo::format`]. This removes the
//! `if self.is_opencollection(..)` fork that previously duplicated every
//! mutating operation.

use anyhow::{Context as _, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use super::format::CollectionFormat;
use super::manager::{CollectionInfo, CollectionManager, GroupInfo, oc_request_key, sanitize_name};
use super::opencollection::{self, OcEnvironment, OcItem};
use super::types::RequestToml;
use crate::domain::RequestData;

/// A collection's on-disk persistence strategy. Implementations mutate the
/// passed-in [`CollectionInfo`] (in-memory model) and persist the change to
/// disk in their native format.
pub(crate) trait CollectionStorage {
    /// Persist the collection-level metadata (name, version, vars, docs,
    /// environments) currently held in `info`.
    fn save_collection(&self, info: &CollectionInfo) -> Result<()>;

    /// Save (create or overwrite) a request, optionally inside a group.
    fn save_request(
        &self,
        info: &mut CollectionInfo,
        request_data: &RequestData,
        request_name: &str,
        group_path: Option<&str>,
    ) -> Result<()>;

    /// Delete a request identified by name + method + url.
    fn delete_request(&self, info: &mut CollectionInfo, request_data: &RequestData) -> Result<()>;

    /// Move a request to a different group (or the root when `None`).
    fn move_request(
        &self,
        info: &mut CollectionInfo,
        request_data: &RequestData,
        target_group_path: Option<&str>,
    ) -> Result<()>;

    /// Create an empty group.
    fn create_group(&self, info: &mut CollectionInfo, group_name: &str) -> Result<()>;

    /// Rename a group, keeping its requests.
    fn rename_group(
        &self,
        info: &mut CollectionInfo,
        old_group_name: &str,
        new_group_name: &str,
    ) -> Result<()>;

    /// Delete a group and all its requests.
    fn delete_group(&self, info: &mut CollectionInfo, group_name: &str) -> Result<()>;
}

/// Return the storage strategy for a collection format.
pub(crate) fn storage_for(format: CollectionFormat) -> &'static dyn CollectionStorage {
    match format {
        CollectionFormat::Broquest => &BroquestStorage,
        CollectionFormat::OpenCollection => &OpenCollectionStorage,
    }
}

// -- Broquest native TOML strategy ------------------------------------------

/// One request/group per file under a collection directory, plus a
/// `collection.toml` for collection metadata.
pub(crate) struct BroquestStorage;

impl BroquestStorage {
    /// Re-read the on-disk structure of a native collection into `info`.
    fn reload(info: &mut CollectionInfo) -> Result<()> {
        let (requests, groups) =
            CollectionManager::load_collection_structure(Path::new(&info.data.path))?;
        info.requests = requests;
        info.groups = groups;
        Ok(())
    }
}

impl CollectionStorage for BroquestStorage {
    fn save_collection(&self, info: &CollectionInfo) -> Result<()> {
        let path = &info.data.path;
        fs::create_dir_all(path)?;
        let collection_file_path = Path::new(path).join("collection.toml");
        let toml_string = toml::to_string_pretty(&info.toml)
            .with_context(|| "Failed to serialize collection data to TOML")?;
        fs::write(collection_file_path, toml_string)
            .with_context(|| format!("Failed to write collection.toml to path: {}", path))?;
        Ok(())
    }

    fn save_request(
        &self,
        info: &mut CollectionInfo,
        request_data: &RequestData,
        request_name: &str,
        group_path: Option<&str>,
    ) -> Result<()> {
        // Determine the target directory
        let collection_dir_path = Path::new(&info.data.path);
        let target_dir = if let Some(group_path) = group_path {
            collection_dir_path.join(group_path)
        } else {
            collection_dir_path.to_path_buf()
        };

        // Ensure directory exists (ignore error if it already exists)
        if let Err(e) = fs::create_dir_all(&target_dir)
            && e.kind() != std::io::ErrorKind::AlreadyExists
        {
            return Err(e).with_context(|| format!("Failed to create directory {:?}", target_dir));
        }

        // Create the full file path
        let request_file_path = target_dir.join(format!("{}.toml", request_name));

        // Convert RequestData to RequestToml and serialize
        let request_toml: RequestToml = request_data.clone().into();
        let toml_string = toml::to_string_pretty(&request_toml)
            .with_context(|| "Failed to serialize request data to TOML")?;
        fs::write(&request_file_path, toml_string)
            .with_context(|| format!("Failed to write request file to {:?}", request_file_path))?;

        let request_path_str = request_file_path.to_string_lossy().to_string();

        // Determine if this is a group request or root request
        let is_update = if let Some(group_path) = group_path {
            let group_name = Path::new(group_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or(group_path);

            if let Some(group_info) = info.groups.get_mut(group_name) {
                let is_update = group_info.requests.contains_key(&request_path_str);
                group_info
                    .requests
                    .insert(request_path_str.clone(), request_data.clone());
                is_update
            } else {
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
                info.groups.insert(group_name.to_string(), new_group);
                false
            }
        } else {
            let is_update = info.requests.contains_key(&request_path_str);
            info.requests
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
            info.data.name
        );

        Ok(())
    }

    fn delete_request(&self, info: &mut CollectionInfo, request_data: &RequestData) -> Result<()> {
        let request_file_path = info
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

        fs::remove_file(&request_file_path)
            .with_context(|| format!("Failed to delete request file {:?}", request_file_path))?;
        info.requests.remove(&request_file_path);

        tracing::info!(
            "Request '{}' deleted successfully from collection '{}'",
            request_data.name,
            info.data.name
        );
        Ok(())
    }

    fn move_request(
        &self,
        info: &mut CollectionInfo,
        request_data: &RequestData,
        target_group_path: Option<&str>,
    ) -> Result<()> {
        // Find the current location of the request.
        let matches = |stored: &RequestData| {
            stored.name == request_data.name
                && stored.method == request_data.method
                && stored.url == request_data.url
        };
        let mut current_location = info
            .requests
            .iter()
            .find(|(_, r)| matches(r))
            .map(|(path, _)| path.clone());
        if current_location.is_none() {
            'outer: for group_info in info.groups.values() {
                for (path, stored_request) in &group_info.requests {
                    if matches(stored_request) {
                        current_location = Some(path.clone());
                        break 'outer;
                    }
                }
            }
        }
        let current_location = current_location.ok_or_else(|| {
            anyhow::anyhow!("Request '{}' not found in collection", request_data.name)
        })?;

        // Remove from current location (in-memory).
        if info.requests.remove(&current_location).is_none() {
            for group_info in info.groups.values_mut() {
                if group_info.requests.remove(&current_location).is_some() {
                    break;
                }
            }
        }

        // Delete the old file from disk.
        fs::remove_file(&current_location)
            .with_context(|| format!("Failed to delete old request file {:?}", current_location))?;

        // Save to the new location (updates disk and in-memory).
        self.save_request(info, request_data, &request_data.name, target_group_path)?;

        tracing::info!(
            "Request '{}' moved to {:?}",
            request_data.name,
            target_group_path.unwrap_or("root level")
        );
        Ok(())
    }

    fn create_group(&self, info: &mut CollectionInfo, group_name: &str) -> Result<()> {
        let sanitized_name = sanitize_name(group_name);
        let collection_dir = Path::new(&info.data.path);
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
            info.data.name,
            group_dir
        );
        Self::reload(info)
    }

    fn rename_group(
        &self,
        info: &mut CollectionInfo,
        old_group_name: &str,
        new_group_name: &str,
    ) -> Result<()> {
        let sanitized_new_name = sanitize_name(new_group_name);

        let old_group_path = info
            .groups
            .get(old_group_name)
            .ok_or_else(|| anyhow::anyhow!("Group '{}' not found in collection", old_group_name))?
            .path
            .clone();

        let collection_dir = Path::new(&info.data.path);
        let old_group_dir = collection_dir.join(&old_group_path);
        let new_group_dir = collection_dir.join(&sanitized_new_name);

        if new_group_dir.exists() && sanitized_new_name != old_group_path {
            return Err(anyhow::anyhow!(
                "Group '{}' already exists in collection",
                sanitized_new_name
            ));
        }

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
            info.data.name
        );
        Self::reload(info)
    }

    fn delete_group(&self, info: &mut CollectionInfo, group_name: &str) -> Result<()> {
        let group_path = info
            .groups
            .get(group_name)
            .ok_or_else(|| anyhow::anyhow!("Group '{}' not found in collection", group_name))?
            .path
            .clone();

        let collection_dir = Path::new(&info.data.path);
        let full_group_path = collection_dir.join(&group_path);

        if full_group_path.exists() {
            fs::remove_dir_all(&full_group_path).with_context(|| {
                format!("Failed to delete group directory {:?}", full_group_path)
            })?;
        }

        tracing::info!(
            "Group '{}' deleted from collection '{}' (removed directory: {:?})",
            group_name,
            info.data.name,
            full_group_path
        );
        Self::reload(info)
    }
}

// -- OpenCollection YAML strategy -------------------------------------------

/// Bruno's OpenCollection format. Mutations edit the in-memory model, then the
/// whole collection is re-serialized to disk so unmodeled fields survive.
pub(crate) struct OpenCollectionStorage;

impl OpenCollectionStorage {
    /// Re-serialize an OpenCollection collection to disk from its in-memory
    /// model. Bundled collections are written as a single `opencollection.yml`;
    /// non-bundled collections are written as a directory tree. Each request is
    /// merged into its retained source item so unmodeled fields are preserved.
    fn persist(info: &CollectionInfo) -> Result<()> {
        let dir = Path::new(&info.data.path).to_path_buf();

        let bundled = info
            .oc_source
            .as_ref()
            .and_then(|f| f.bundled)
            .unwrap_or(true);

        if !bundled {
            return Self::persist_unbundled(&dir, info);
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
    fn persist_unbundled(dir: &Path, info: &CollectionInfo) -> Result<()> {
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
        let mut desired_env_files = HashSet::new();
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
        let mut desired_root_files = HashSet::new();
        for (key, req) in &info.requests {
            let item = opencollection::merge_request_into_item(info.oc_items.get(key), req);
            let fname = format!("{}.yml", sanitize_name(&req.name));
            fs::write(dir.join(&fname), opencollection::item_to_yaml(&item)?)
                .with_context(|| format!("Failed to write request {}", fname))?;
            desired_root_files.insert(fname);
        }

        // 4. Folder directories.
        let mut desired_group_dirs = HashSet::new();
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

            let mut desired = HashSet::new();
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
}

impl CollectionStorage for OpenCollectionStorage {
    fn save_collection(&self, info: &CollectionInfo) -> Result<()> {
        Self::persist(info)
    }

    fn save_request(
        &self,
        info: &mut CollectionInfo,
        request_data: &RequestData,
        request_name: &str,
        group_path: Option<&str>,
    ) -> Result<()> {
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
        Self::persist(info)
    }

    fn delete_request(&self, info: &mut CollectionInfo, request_data: &RequestData) -> Result<()> {
        let key = find_request_key(info, request_data)
            .ok_or_else(|| anyhow::anyhow!("Request not found in collection"))?;
        if info.requests.remove(&key).is_none() {
            for group in info.groups.values_mut() {
                if group.requests.remove(&key).is_some() {
                    break;
                }
            }
        }
        Self::persist(info)
    }

    fn move_request(
        &self,
        info: &mut CollectionInfo,
        request_data: &RequestData,
        target_group_path: Option<&str>,
    ) -> Result<()> {
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
        Self::persist(info)
    }

    fn create_group(&self, info: &mut CollectionInfo, group_name: &str) -> Result<()> {
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
        Self::persist(info)
    }

    fn rename_group(
        &self,
        info: &mut CollectionInfo,
        old_group_name: &str,
        new_group_name: &str,
    ) -> Result<()> {
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
        Self::persist(info)
    }

    fn delete_group(&self, info: &mut CollectionInfo, group_name: &str) -> Result<()> {
        info.groups.remove(group_name);
        Self::persist(info)
    }
}

// -- Shared OpenCollection helpers ------------------------------------------

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
fn remove_stray_yaml(dir: &Path, keep: &HashSet<String>, always_keep: &[&str]) {
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
