//! Bruno OpenCollection (v1.0.0) format support.
//!
//! Spec: <https://spec.opencollection.com>, schema:
//! <https://schema.opencollection.com/opencollection/v1.0.0.json>.
//!
//! An OpenCollection is a YAML document. It can be *bundled* (the whole tree in
//! one `opencollection.yml`) or *non-bundled* (a directory tree of `.yml`
//! files). This module maps the modeled subset of the schema to/from broquest's
//! internal [`RequestData`]/[`EnvironmentToml`] model.
//!
//! Every struct carries a `#[serde(flatten)] extra` catch-all so that fields
//! broquest does not model (examples, settings, assertions, gRPC/websocket
//! items, unsupported auth/body, deeper folder nesting, `extensions`, …) are
//! preserved verbatim when a collection is read and written back.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_yaml_ng::{Mapping, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::collections::types::{EnvironmentToml, EnvironmentVariable};
use crate::domain::{
    AuthType, BasicAuth, ContentType, DigestAuth, HttpMethod, KeyAuth, KeyValuePair, RequestData,
};

/// Key used to losslessly preserve broquest-only auth (OAuth2/JWT) that has no
/// clean OpenCollection representation. Stored under the request's `http` block.
const BROQUEST_AUTH_KEY: &str = "x-broquest-auth";

/// Key used to preserve broquest collection-level variables (which the
/// OpenCollection schema does not model) on the root file's `extra` map, so
/// they survive read/write round-trips.
const BROQUEST_VARS_KEY: &str = "x-broquest-vars";

fn is_false(b: &bool) -> bool {
    !*b
}

// ---------------------------------------------------------------------------
// Serde structs (modeled subset of the v1.0.0 schema)
// ---------------------------------------------------------------------------

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenCollectionFile {
    /// Spec version string, e.g. "1.0.0".
    pub opencollection: String,
    pub info: OcInfo,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config: Option<OcConfig>,
    /// Collection-level request defaults; preserved opaque.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub request: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub items: Vec<OcItem>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bundled: Option<bool>,
    #[serde(flatten)]
    pub extra: Mapping,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OcInfo {
    #[serde(default)]
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(flatten)]
    pub extra: Mapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcItem {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub info: Option<OcItemInfo>,
    /// Top-level `type` for a `ScriptFile` item (which has no `info` wrapper).
    #[serde(default, skip_serializing_if = "Option::is_none", rename = "type")]
    pub script_file_type: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub http: Option<OcHttp>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graphql: Option<Value>,
    /// Folder children.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub items: Option<Vec<OcItem>>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<OcRuntime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub settings: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub examples: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub docs: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(flatten)]
    pub extra: Mapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcItemInfo {
    #[serde(default)]
    pub name: String,
    #[serde(rename = "type")]
    pub item_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seq: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Value>,
    #[serde(flatten)]
    pub extra: Mapping,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OcHttp {
    #[serde(default)]
    pub method: String,
    #[serde(default)]
    pub url: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub headers: Vec<OcHeader>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<OcParam>,
    /// Single body object or an array of variants; kept opaque so multipart /
    /// file bodies survive round-trips.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<Value>,
    /// Typed auth object or the literal string "inherit".
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub auth: Option<Value>,
    #[serde(flatten)]
    pub extra: Mapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcHeader {
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Value>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
    #[serde(flatten)]
    pub extra: Mapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcParam {
    pub name: String,
    #[serde(default)]
    pub value: String,
    #[serde(rename = "type")]
    pub param_type: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Value>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub disabled: bool,
    #[serde(flatten)]
    pub extra: Mapping,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OcRuntime {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub scripts: Vec<OcScript>,
    /// Request-level variables (`Variable` objects: name/value/description/disabled).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub variables: Option<Vec<Value>>,
    /// assertions / actions preserved opaque.
    #[serde(flatten)]
    pub extra: Mapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcScript {
    #[serde(rename = "type")]
    pub script_type: String,
    #[serde(default)]
    pub code: String,
    #[serde(flatten)]
    pub extra: Mapping,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OcConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub environments: Vec<OcEnvironment>,
    #[serde(flatten)]
    pub extra: Mapping,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OcEnvironment {
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// Each entry is a `Variable` or `SecretVariable`; interpreted manually.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub variables: Vec<Value>,
    #[serde(flatten)]
    pub extra: Mapping,
}

// ---------------------------------------------------------------------------
// Loaded representation handed to the CollectionManager
// ---------------------------------------------------------------------------

/// A request paired with the original item it was parsed from, so fields
/// broquest does not model (examples, settings, tags, param descriptions, …)
/// can be re-attached when the collection is written back.
pub struct LoadedRequest {
    pub request: RequestData,
    pub source: OcItem,
}

/// A top-level folder and its requests.
pub struct LoadedGroup {
    pub name: String,
    /// The folder's own config (`folder.yml` for non-bundled, or the folder
    /// item for bundled), retained for preservation.
    pub source: Option<OcItem>,
    pub requests: Vec<LoadedRequest>,
}

/// An environment paired with its parsed source for preservation.
pub struct LoadedEnv {
    pub toml: EnvironmentToml,
    pub source: OcEnvironment,
}

/// The result of reading an OpenCollection, decomposed into broquest's model.
pub struct LoadedCollection {
    /// The fully-parsed root file, retained for high-fidelity write-back.
    pub file: OpenCollectionFile,
    pub name: String,
    pub version: String,
    /// Collection documentation as markdown (`None` when the OC file has no
    /// `docs`, or a block shape broquest can't model).
    pub docs: Option<String>,
    pub environments: Vec<LoadedEnv>,
    /// Requests at the collection root, in document order.
    pub root_requests: Vec<LoadedRequest>,
    /// Top-level folders.
    pub groups: Vec<LoadedGroup>,
    /// broquest collection-level variables (preserved via `x-broquest-vars`).
    pub vars: Vec<KeyValuePair>,
}

// ---------------------------------------------------------------------------
// Parsing / loading
// ---------------------------------------------------------------------------

/// Parse OpenCollection YAML text into the typed representation.
pub fn parse_opencollection(content: &str) -> Result<OpenCollectionFile> {
    serde_yaml_ng::from_str(content).context("Failed to parse OpenCollection YAML")
}

/// Locate the `opencollection.yml` / `.yaml` marker file inside `dir`.
pub fn find_opencollection_file(dir: &Path) -> Option<PathBuf> {
    for name in ["opencollection.yml", "opencollection.yaml"] {
        let candidate = dir.join(name);
        if candidate.exists() {
            return Some(candidate);
        }
    }
    None
}

/// Read and decompose an OpenCollection directory, handling both the bundled
/// (single-file) and non-bundled (directory tree) layouts.
pub fn load_opencollection(dir: &Path) -> Result<LoadedCollection> {
    let path = find_opencollection_file(dir)
        .ok_or_else(|| anyhow::anyhow!("No opencollection.yml found in {:?}", dir))?;
    let content =
        fs::read_to_string(&path).with_context(|| format!("Failed to read {:?}", path))?;
    let file = parse_opencollection(&content)?;
    if file.bundled.unwrap_or(true) {
        Ok(decompose_bundled(file))
    } else {
        decompose_unbundled(dir, file)
    }
}

/// Extract `(name, version, docs)` from the root file.
///
/// `docs` is best-effort: a YAML string scalar becomes a markdown `Some(s)`;
/// an array of content blocks is joined into markdown from any string-valued
/// blocks (unsupported block shapes are skipped with a warning); anything else
/// yields `None` (the raw value is still preserved in `file`/`oc_source`).
fn collection_common(file: &OpenCollectionFile) -> (String, String, Option<String>) {
    let name = file.info.name.clone();
    let version = file
        .info
        .version
        .clone()
        .unwrap_or_else(|| "1.0.0".to_string());
    let docs = file.docs.as_ref().and_then(docs_value_to_markdown);
    (name, version, docs)
}

/// Convert an OpenCollection `docs` value to a markdown string.
fn docs_value_to_markdown(value: &Value) -> Option<String> {
    if let Some(s) = value.as_str() {
        return Some(s.to_string());
    }
    // Array of content blocks: join the text of string-valued blocks.
    if let Some(seq) = value.as_sequence() {
        let mut parts = Vec::new();
        for block in seq {
            if let Some(s) = block.as_str() {
                parts.push(s.to_string());
            } else if let Some(map) = block.as_mapping()
                && let Some(text) = map
                    .get(Value::String("text".into()))
                    .and_then(|t| t.as_str())
            {
                parts.push(text.to_string());
            } else {
                tracing::warn!(
                    "Skipping unsupported OpenCollection docs block: {:?}",
                    block
                );
            }
        }
        if parts.is_empty() {
            return None;
        }
        return Some(parts.join("\n\n"));
    }
    None
}

/// Read broquest collection-level vars preserved under `x-broquest-vars` on the
/// root file's `extra` map. OpenCollection does not model collection vars, so
/// absent or malformed data yields an empty list (rather than an error).
fn collection_vars(file: &OpenCollectionFile) -> Vec<KeyValuePair> {
    let Some(raw) = file.extra.get(Value::String(BROQUEST_VARS_KEY.into())) else {
        return Vec::new();
    };
    let json = serde_json::to_value(raw).unwrap_or(serde_json::Value::Null);
    serde_json::from_value::<Vec<KeyValuePair>>(json).unwrap_or_default()
}

/// Serialize collection vars into a YAML value for `x-broquest-vars`.
fn vars_to_yaml(vars: &[KeyValuePair]) -> Option<Value> {
    if vars.is_empty() {
        return None;
    }
    let json = serde_json::to_value(vars).ok()?;
    Some(json_to_yaml(&json))
}

fn loaded_request(item: &OcItem) -> Option<LoadedRequest> {
    oc_item_to_request(item).map(|request| LoadedRequest {
        request,
        source: item.clone(),
    })
}

/// Bundled: walk the in-file `items` tree.
fn decompose_bundled(file: OpenCollectionFile) -> LoadedCollection {
    let (name, version, docs) = collection_common(&file);
    let vars = collection_vars(&file);

    let environments = file
        .config
        .as_ref()
        .map(|c| c.environments.iter().map(loaded_env).collect())
        .unwrap_or_default();

    let mut root_requests = Vec::new();
    let mut groups = Vec::new();

    for item in &file.items {
        match item_type(item) {
            ItemKind::Http => {
                if let Some(loaded) = loaded_request(item) {
                    root_requests.push(loaded);
                }
            }
            ItemKind::Folder => {
                let group_name = item
                    .info
                    .as_ref()
                    .map(|i| i.name.clone())
                    .unwrap_or_default();
                let mut requests = Vec::new();
                collect_folder_requests(item, &mut requests);
                groups.push(LoadedGroup {
                    name: group_name,
                    source: Some(item.clone()),
                    requests,
                });
            }
            ItemKind::Other => {
                tracing::warn!("Skipping unsupported OpenCollection item type (preserved on save)");
            }
        }
    }

    LoadedCollection {
        file,
        name,
        version,
        docs,
        environments,
        root_requests,
        groups,
        vars,
    }
}

/// Non-bundled: scan the directory tree (root `.yml` requests, `environments/`,
/// and one directory per folder containing `folder.yml` + request `.yml` files).
fn decompose_unbundled(dir: &Path, file: OpenCollectionFile) -> Result<LoadedCollection> {
    let (name, version, docs) = collection_common(&file);
    let vars = collection_vars(&file);

    // Environments live under environments/<name>.yml.
    let mut environments = Vec::new();
    let env_dir = dir.join("environments");
    if env_dir.is_dir() {
        for path in sorted_yaml_files(&env_dir) {
            match parse_file::<OcEnvironment>(&path) {
                Ok(env) => environments.push(loaded_env(&env)),
                Err(e) => tracing::error!("Failed to parse environment {:?}: {}", path, e),
            }
        }
    }

    let mut root_requests = Vec::new();
    let mut groups = Vec::new();

    let mut entries: Vec<PathBuf> = fs::read_dir(dir)
        .with_context(|| format!("Failed to read collection dir {:?}", dir))?
        .filter_map(|e| e.ok().map(|e| e.path()))
        .collect();
    entries.sort();

    for path in entries {
        if path.is_dir() {
            if path.file_name().and_then(|n| n.to_str()) == Some("environments") {
                continue;
            }
            if let Some(group) = load_unbundled_folder(&path) {
                groups.push(group);
            }
        } else if is_request_file(&path) {
            match parse_file::<OcItem>(&path) {
                Ok(item) => {
                    if let Some(loaded) = loaded_request(&item) {
                        root_requests.push(loaded);
                    }
                }
                Err(e) => tracing::error!("Failed to parse request {:?}: {}", path, e),
            }
        }
    }

    Ok(LoadedCollection {
        file,
        name,
        version,
        docs,
        environments,
        root_requests,
        groups,
        vars,
    })
}

/// Load a folder directory: `folder.yml` config + sibling request `.yml` files.
fn load_unbundled_folder(dir: &Path) -> Option<LoadedGroup> {
    let folder_yml = dir.join("folder.yml");
    let source = parse_file::<OcItem>(&folder_yml).ok();
    let name = source
        .as_ref()
        .and_then(|s| s.info.as_ref())
        .map(|i| i.name.clone())
        .unwrap_or_else(|| {
            dir.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or_default()
                .to_string()
        });

    let mut requests = Vec::new();
    for path in sorted_yaml_files(dir) {
        if path.file_name().and_then(|n| n.to_str()) == Some("folder.yml") {
            continue;
        }
        match parse_file::<OcItem>(&path) {
            Ok(item) => {
                if let Some(loaded) = loaded_request(&item) {
                    requests.push(loaded);
                }
            }
            Err(e) => tracing::error!("Failed to parse request {:?}: {}", path, e),
        }
    }

    Some(LoadedGroup {
        name,
        source,
        requests,
    })
}

fn is_request_file(path: &Path) -> bool {
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

fn sorted_yaml_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .map(|rd| {
            rd.filter_map(|e| e.ok().map(|e| e.path()))
                .filter(|p| {
                    p.is_file()
                        && p.extension()
                            .map(|x| x == "yml" || x == "yaml")
                            .unwrap_or(false)
                })
                .collect()
        })
        .unwrap_or_default();
    files.sort();
    files
}

fn parse_file<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path).with_context(|| format!("Failed to read {:?}", path))?;
    serde_yaml_ng::from_str(&content).with_context(|| format!("Failed to parse {:?}", path))
}

enum ItemKind {
    Http,
    Folder,
    Other,
}

fn item_type(item: &OcItem) -> ItemKind {
    let t = item
        .info
        .as_ref()
        .map(|i| i.item_type.as_str())
        .or(item.script_file_type.as_deref())
        .unwrap_or("");
    match t {
        "http" => ItemKind::Http,
        "folder" => ItemKind::Folder,
        _ => ItemKind::Other,
    }
}

/// Recursively collect HTTP requests under a folder. broquest supports a single
/// nesting level, so requests from nested folders are flattened into the parent
/// group (their structure is preserved in the retained source file).
fn collect_folder_requests(folder: &OcItem, out: &mut Vec<LoadedRequest>) {
    let Some(children) = &folder.items else {
        return;
    };
    for child in children {
        match item_type(child) {
            ItemKind::Http => {
                if let Some(loaded) = loaded_request(child) {
                    out.push(loaded);
                }
            }
            ItemKind::Folder => collect_folder_requests(child, out),
            ItemKind::Other => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Item <-> RequestData mapping
// ---------------------------------------------------------------------------

fn oc_item_to_request(item: &OcItem) -> Option<RequestData> {
    let http = item.http.as_ref()?;
    let name = item
        .info
        .as_ref()
        .map(|i| i.name.clone())
        .unwrap_or_default();

    let mut headers: Vec<KeyValuePair> = http
        .headers
        .iter()
        .map(|h| KeyValuePair {
            key: h.name.clone(),
            value: h.value.clone(),
            enabled: !h.disabled,
        })
        .collect();

    let mut query_params = Vec::new();
    let mut path_params = Vec::new();
    for p in &http.params {
        let pair = KeyValuePair {
            key: p.name.clone(),
            value: p.value.clone(),
            enabled: !p.disabled,
        };
        if p.param_type == "path" {
            path_params.push(pair);
        } else {
            query_params.push(pair);
        }
    }

    let body = http
        .body
        .as_ref()
        .map(|b| oc_body_to_string(b, &mut headers))
        .unwrap_or_default();

    let (pre_request_script, post_response_script) = match &item.runtime {
        Some(rt) => scripts_from_runtime(rt),
        None => (None, None),
    };

    // Prefer a losslessly-preserved broquest auth (OAuth2/JWT) if present.
    let auth = if let Some(raw) = http.extra.get(Value::String(BROQUEST_AUTH_KEY.into())) {
        serde_json::to_value(raw)
            .ok()
            .and_then(|j| serde_json::from_value::<AuthType>(j).ok())
            .unwrap_or(AuthType::None)
    } else {
        http.auth
            .as_ref()
            .map(oc_auth_to_authtype)
            .unwrap_or(AuthType::None)
    };

    Some(RequestData {
        name,
        method: parse_method(&http.method),
        url: http.url.clone(),
        path_params,
        query_params,
        headers,
        body,
        pre_request_script,
        post_response_script,
        auth,
        vars: vars_from_runtime(item.runtime.as_ref()),
    })
}

/// Extract request-level variables from an item's `runtime.variables`.
fn vars_from_runtime(runtime: Option<&OcRuntime>) -> Vec<KeyValuePair> {
    let Some(vars) = runtime.and_then(|rt| rt.variables.as_ref()) else {
        return Vec::new();
    };
    vars.iter()
        .filter_map(|v| {
            let map = v.as_mapping()?;
            let name = map.get("name")?.as_str()?.to_string();
            let value = map
                .get("value")
                .and_then(|x| x.as_str())
                .unwrap_or("")
                .to_string();
            let disabled = map
                .get("disabled")
                .and_then(|x| x.as_bool())
                .unwrap_or(false);
            Some(KeyValuePair {
                key: name,
                value,
                enabled: !disabled,
            })
        })
        .collect()
}

/// Build an OpenCollection HTTP item from a broquest request.
/// Build an OpenCollection HTTP item from a broquest request, merging the
/// modeled fields into `source` (the item this request was parsed from) so that
/// unmodeled fields — examples, settings, tags, param descriptions, unsupported
/// bodies — are preserved. When the request is unchanged from its source, the
/// source is returned verbatim (guaranteeing a lossless open→save round-trip).
pub fn merge_request_into_item(source: Option<&OcItem>, req: &RequestData) -> OcItem {
    if let Some(src) = source
        && oc_item_to_request(src).as_ref() == Some(req)
    {
        return src.clone();
    }

    let mut item = source.cloned().unwrap_or_else(new_http_item);

    let info = item.info.get_or_insert_with(|| OcItemInfo {
        name: String::new(),
        item_type: "http".to_string(),
        description: None,
        seq: None,
        tags: None,
        extra: Mapping::new(),
    });
    info.name = req.name.clone();
    info.item_type = "http".to_string();

    let mut http = item.http.take().unwrap_or_default();
    http.method = req.method.as_str().to_string();
    http.url = req.url.clone();
    http.headers = merge_headers(&http.headers, &req.headers);
    http.params = merge_params(&http.params, &req.query_params, &req.path_params);
    // Only overwrite the body when the request carries one, so preserved
    // multipart/file bodies survive when broquest leaves the body empty.
    if !req.body.is_empty() {
        http.body = request_to_oc_body(req);
    }
    http.auth = None;
    http.extra.remove(Value::from(BROQUEST_AUTH_KEY));
    apply_auth_to_http(&mut http, &req.auth);
    item.http = Some(http);

    item.runtime = merge_runtime(
        item.runtime.take(),
        &req.pre_request_script,
        &req.post_response_script,
        &req.vars,
    );

    item
}

fn new_http_item() -> OcItem {
    OcItem {
        info: None,
        script_file_type: None,
        http: None,
        graphql: None,
        items: None,
        runtime: None,
        settings: None,
        examples: None,
        docs: None,
        script: None,
        extra: Mapping::new(),
    }
}

fn merge_headers(source: &[OcHeader], headers: &[KeyValuePair]) -> Vec<OcHeader> {
    headers
        .iter()
        .filter(|h| !h.key.is_empty())
        .map(|h| {
            let mut base = source
                .iter()
                .find(|s| s.name == h.key)
                .cloned()
                .unwrap_or_else(|| OcHeader {
                    name: h.key.clone(),
                    value: String::new(),
                    description: None,
                    disabled: false,
                    extra: Mapping::new(),
                });
            base.name = h.key.clone();
            base.value = h.value.clone();
            base.disabled = !h.enabled;
            base
        })
        .collect()
}

fn merge_params(
    source: &[OcParam],
    query_params: &[KeyValuePair],
    path_params: &[KeyValuePair],
) -> Vec<OcParam> {
    let mut out = Vec::new();
    let mut push = |pair: &KeyValuePair, param_type: &str| {
        if pair.key.is_empty() {
            return;
        }
        let mut base = source
            .iter()
            .find(|s| s.name == pair.key && s.param_type == param_type)
            .cloned()
            .unwrap_or_else(|| OcParam {
                name: pair.key.clone(),
                value: String::new(),
                param_type: param_type.to_string(),
                description: None,
                disabled: false,
                extra: Mapping::new(),
            });
        base.name = pair.key.clone();
        base.value = pair.value.clone();
        base.param_type = param_type.to_string();
        base.disabled = !pair.enabled;
        out.push(base);
    };
    for p in query_params {
        push(p, "query");
    }
    for p in path_params {
        push(p, "path");
    }
    out
}

fn merge_runtime(
    source: Option<OcRuntime>,
    pre: &Option<String>,
    post: &Option<String>,
    vars: &[KeyValuePair],
) -> Option<OcRuntime> {
    let mut rt = source.unwrap_or_default();
    // Replace the before-request / after-response scripts, preserving any
    // tests / hooks and unmodeled runtime fields (assertions, actions, …).
    rt.scripts
        .retain(|s| s.script_type != "before-request" && s.script_type != "after-response");
    if let Some(code) = pre {
        rt.scripts.push(OcScript {
            script_type: "before-request".to_string(),
            code: code.clone(),
            extra: Mapping::new(),
        });
    }
    if let Some(code) = post {
        rt.scripts.push(OcScript {
            script_type: "after-response".to_string(),
            code: code.clone(),
            extra: Mapping::new(),
        });
    }

    // Request variables, preserving each source variable's unmodeled fields
    // (description, etc.) by matching on name.
    let named: Vec<&KeyValuePair> = vars.iter().filter(|v| !v.key.is_empty()).collect();
    if named.is_empty() {
        rt.variables = None;
    } else {
        let source_vars = rt.variables.take().unwrap_or_default();
        rt.variables = Some(
            named
                .iter()
                .map(|v| var_to_oc_value(&source_vars, v))
                .collect(),
        );
    }

    if rt.scripts.is_empty() && rt.variables.is_none() && rt.extra.is_empty() {
        None
    } else {
        Some(rt)
    }
}

/// Build an OpenCollection `Variable` value from a broquest var, carrying over
/// any unmodeled fields (e.g. `description`) from the matching source variable.
fn var_to_oc_value(source: &[Value], var: &KeyValuePair) -> Value {
    let mut map = source
        .iter()
        .find(|v| {
            v.as_mapping()
                .and_then(|m| m.get("name"))
                .and_then(|n| n.as_str())
                == Some(&var.key)
        })
        .and_then(|v| v.as_mapping().cloned())
        .unwrap_or_default();
    map.insert(Value::from("name"), Value::from(var.key.clone()));
    map.insert(Value::from("value"), Value::from(var.value.clone()));
    if var.enabled {
        map.remove(Value::from("disabled"));
    } else {
        map.insert(Value::from("disabled"), Value::from(true));
    }
    Value::Mapping(map)
}

fn parse_method(s: &str) -> HttpMethod {
    match s.to_uppercase().as_str() {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "DELETE" => HttpMethod::Delete,
        "PATCH" => HttpMethod::Patch,
        "HEAD" => HttpMethod::Head,
        "OPTIONS" => HttpMethod::Options,
        other => {
            tracing::warn!("Unknown HTTP method '{}', defaulting to GET", other);
            HttpMethod::Get
        }
    }
}

// ---------------------------------------------------------------------------
// Body mapping
// ---------------------------------------------------------------------------

/// Convert an OpenCollection body value to broquest's flat body string,
/// ensuring a matching `Content-Type` header exists so broquest's body-type
/// inference stays consistent.
fn oc_body_to_string(body: &Value, headers: &mut Vec<KeyValuePair>) -> String {
    let Some(map) = body.as_mapping() else {
        // Array of variants (multipart/file) or unexpected shape: preserved in
        // the source file, not represented in the flat body.
        return String::new();
    };

    // GraphQL body has `query`/`variables` and no `type`.
    if map.get("query").is_some() || map.get("variables").is_some() {
        let query = map.get("query").and_then(|v| v.as_str()).unwrap_or("");
        let variables = map
            .get("variables")
            .and_then(|v| v.as_str())
            .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
            .unwrap_or(serde_json::Value::Object(serde_json::Map::new()));
        return serde_json::json!({ "query": query, "variables": variables }).to_string();
    }

    let body_type = map.get("type").and_then(|v| v.as_str()).unwrap_or("");
    match body_type {
        "json" => {
            ensure_content_type(headers, "application/json");
            map.get("data")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        }
        "xml" | "sparql" => {
            ensure_content_type(headers, "application/xml");
            map.get("data")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        }
        "text" => {
            ensure_content_type(headers, "text/plain");
            map.get("data")
                .and_then(|v| v.as_str())
                .unwrap_or("")
                .to_string()
        }
        "form-urlencoded" => {
            ensure_content_type(headers, "application/x-www-form-urlencoded");
            map.get("data")
                .and_then(|v| v.as_sequence())
                .map(|seq| {
                    seq.iter()
                        .filter_map(|e| e.as_mapping())
                        .filter(|m| !m.get("disabled").and_then(|d| d.as_bool()).unwrap_or(false))
                        .map(|m| {
                            let k = m.get("name").and_then(|v| v.as_str()).unwrap_or("");
                            let v = m.get("value").and_then(|v| v.as_str()).unwrap_or("");
                            format!("{}={}", urlencoding::encode(k), urlencoding::encode(v))
                        })
                        .collect::<Vec<_>>()
                        .join("&")
                })
                .unwrap_or_default()
        }
        // multipart-form / file / unknown: preserved in source, no flat body.
        _ => String::new(),
    }
}

fn ensure_content_type(headers: &mut Vec<KeyValuePair>, value: &str) {
    let has = headers
        .iter()
        .any(|h| h.key.eq_ignore_ascii_case("content-type") && h.enabled);
    if !has {
        headers.push(KeyValuePair {
            key: "Content-Type".to_string(),
            value: value.to_string(),
            enabled: true,
        });
    }
}

/// Build an OpenCollection body from a broquest request (body type inferred
/// from the Content-Type header, matching the native TOML behavior).
fn request_to_oc_body(req: &RequestData) -> Option<Value> {
    if req.body.is_empty() {
        return None;
    }
    let body_type = req
        .headers
        .iter()
        .find(|h| h.key.eq_ignore_ascii_case("content-type") && h.enabled)
        .map(|h| ContentType::from_header(&h.value).body_type())
        .unwrap_or("json");

    let mut map = Mapping::new();
    match body_type {
        "xml" => {
            map.insert(Value::from("type"), Value::from("xml"));
            map.insert(Value::from("data"), Value::from(req.body.clone()));
        }
        "text" | "html" => {
            map.insert(Value::from("type"), Value::from("text"));
            map.insert(Value::from("data"), Value::from(req.body.clone()));
        }
        "form" => {
            map.insert(Value::from("type"), Value::from("form-urlencoded"));
            let data: Vec<Value> = req
                .body
                .split('&')
                .filter_map(|pair| pair.split_once('='))
                .map(|(k, v)| {
                    let k = urlencoding::decode(k)
                        .map(|c| c.into_owned())
                        .unwrap_or_else(|_| k.to_string());
                    let v = urlencoding::decode(v)
                        .map(|c| c.into_owned())
                        .unwrap_or_else(|_| v.to_string());
                    let mut field = Mapping::new();
                    field.insert(Value::from("name"), Value::from(k));
                    field.insert(Value::from("value"), Value::from(v));
                    Value::Mapping(field)
                })
                .collect();
            map.insert(Value::from("data"), Value::Sequence(data));
        }
        _ => {
            map.insert(Value::from("type"), Value::from("json"));
            map.insert(Value::from("data"), Value::from(req.body.clone()));
        }
    }
    Some(Value::Mapping(map))
}

// ---------------------------------------------------------------------------
// Auth mapping
// ---------------------------------------------------------------------------

fn oc_auth_to_authtype(v: &Value) -> AuthType {
    if let Some(s) = v.as_str() {
        return if s == "inherit" {
            AuthType::Inherit
        } else {
            AuthType::Unsupported {
                kind: s.to_string(),
                raw: yaml_to_json(v),
            }
        };
    }
    let Some(map) = v.as_mapping() else {
        return AuthType::Unsupported {
            kind: "unknown".to_string(),
            raw: yaml_to_json(v),
        };
    };
    let kind = map
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();
    let field = |k: &str| {
        map.get(k)
            .and_then(|x| x.as_str())
            .unwrap_or("")
            .to_string()
    };

    match kind.as_str() {
        "basic" => AuthType::Basic(BasicAuth {
            username: field("username"),
            password: field("password"),
        }),
        "digest" => AuthType::Digest(DigestAuth {
            username: field("username"),
            password: field("password"),
        }),
        "bearer" => AuthType::Key(KeyAuth {
            header: "Authorization".to_string(),
            value: format!("Bearer {}", field("token")),
        }),
        "apikey" => {
            let placement = map
                .get("placement")
                .and_then(|x| x.as_str())
                .unwrap_or("header");
            if placement == "header" {
                let header = field("key");
                AuthType::Key(KeyAuth {
                    header: if header.is_empty() {
                        "X-API-Key".to_string()
                    } else {
                        header
                    },
                    value: field("value"),
                })
            } else {
                AuthType::Unsupported {
                    kind: "apikey".to_string(),
                    raw: yaml_to_json(v),
                }
            }
        }
        other => AuthType::Unsupported {
            kind: if other.is_empty() {
                "unknown".to_string()
            } else {
                other.to_string()
            },
            raw: yaml_to_json(v),
        },
    }
}

fn apply_auth_to_http(http: &mut OcHttp, auth: &AuthType) {
    match auth {
        AuthType::None => {}
        AuthType::Inherit => {
            http.auth = Some(Value::from("inherit"));
        }
        AuthType::Basic(b) => {
            http.auth = Some(auth_object(
                "basic",
                &[("username", &b.username), ("password", &b.password)],
            ));
        }
        AuthType::Digest(d) => {
            http.auth = Some(auth_object(
                "digest",
                &[("username", &d.username), ("password", &d.password)],
            ));
        }
        AuthType::Key(k) => {
            http.auth = Some(auth_object(
                "apikey",
                &[
                    ("key", &k.header),
                    ("value", &k.value),
                    ("placement", "header"),
                ],
            ));
        }
        AuthType::Unsupported { raw, .. } => {
            http.auth = Some(json_to_yaml(raw));
        }
        AuthType::OAuth2(_) | AuthType::Jwt(_) => {
            // No clean OpenCollection representation; preserve the full broquest
            // auth losslessly under a namespaced key so it round-trips.
            if let Ok(j) = serde_json::to_value(auth) {
                http.extra
                    .insert(Value::from(BROQUEST_AUTH_KEY), json_to_yaml(&j));
            }
        }
    }
}

fn auth_object(kind: &str, fields: &[(&str, &str)]) -> Value {
    let mut map = Mapping::new();
    map.insert(Value::from("type"), Value::from(kind));
    for (k, v) in fields {
        map.insert(Value::from(*k), Value::from(*v));
    }
    Value::Mapping(map)
}

// ---------------------------------------------------------------------------
// Script mapping
// ---------------------------------------------------------------------------

fn scripts_from_runtime(rt: &OcRuntime) -> (Option<String>, Option<String>) {
    let mut pre = None;
    let mut post = None;
    for script in &rt.scripts {
        match script.script_type.as_str() {
            "before-request" => pre = Some(script.code.clone()),
            "after-response" => post = Some(script.code.clone()),
            _ => {}
        }
    }
    (pre, post)
}

// ---------------------------------------------------------------------------
// Environment mapping
// ---------------------------------------------------------------------------

fn oc_environment_to_toml(env: &OcEnvironment) -> EnvironmentToml {
    let mut variables = HashMap::new();
    for var in &env.variables {
        let Some(map) = var.as_mapping() else {
            continue;
        };
        let Some(name) = map.get("name").and_then(|v| v.as_str()) else {
            continue;
        };
        let secret = map.get("secret").and_then(|v| v.as_bool()).unwrap_or(false);
        // Secret values live in the OS credential store, never the YAML.
        let value = if secret {
            String::new()
        } else {
            map.get("value").map(oc_variable_value).unwrap_or_default()
        };
        variables.insert(
            name.to_string(),
            EnvironmentVariable {
                value,
                secret,
                temporary: false,
            },
        );
    }
    EnvironmentToml {
        name: env.name.clone(),
        variables,
    }
}

fn loaded_env(env: &OcEnvironment) -> LoadedEnv {
    LoadedEnv {
        toml: oc_environment_to_toml(env),
        source: env.clone(),
    }
}

fn env_toml_to_oc(env: &EnvironmentToml) -> OcEnvironment {
    let variables = env
        .variables
        .iter()
        .map(|(name, var)| {
            let mut map = Mapping::new();
            if var.secret {
                map.insert(Value::from("secret"), Value::from(true));
                map.insert(Value::from("name"), Value::from(name.clone()));
                map.insert(Value::from("type"), Value::from("string"));
            } else {
                map.insert(Value::from("name"), Value::from(name.clone()));
                map.insert(Value::from("value"), Value::from(var.value.clone()));
            }
            Value::Mapping(map)
        })
        .collect();
    OcEnvironment {
        name: env.name.clone(),
        color: None,
        variables,
        extra: Mapping::new(),
    }
}

/// Extract a string from a `VariableValue` (string, or `{type, data}` object).
fn oc_variable_value(v: &Value) -> String {
    if let Some(s) = v.as_str() {
        return s.to_string();
    }
    if let Some(map) = v.as_mapping()
        && let Some(data) = map.get("data").and_then(|d| d.as_str())
    {
        return data.to_string();
    }
    String::new()
}

// ---------------------------------------------------------------------------
// Building files for writing
// ---------------------------------------------------------------------------

fn default_file() -> OpenCollectionFile {
    OpenCollectionFile {
        opencollection: "1.0.0".to_string(),
        info: OcInfo::default(),
        config: None,
        request: None,
        items: Vec::new(),
        docs: None,
        bundled: None,
        extra: Mapping::new(),
    }
}

/// Assemble a bundled `OpenCollectionFile` from already-built items, starting
/// from `base` so unmodeled top-level fields (extensions, request defaults) are
/// preserved.
pub fn assemble_bundled_file(
    base: Option<&OpenCollectionFile>,
    name: &str,
    version: &str,
    docs: Option<&str>,
    items: Vec<OcItem>,
    environments: Vec<OcEnvironment>,
    vars: &[KeyValuePair],
) -> OpenCollectionFile {
    let mut file = base.cloned().unwrap_or_else(default_file);
    file.info.name = name.to_string();
    if file.info.version.is_none() {
        file.info.version = Some(version.to_string());
    }
    apply_docs(&mut file, docs);
    file.bundled = Some(true);
    file.items = items;
    let config = file.config.get_or_insert_with(OcConfig::default);
    config.environments = environments;
    apply_collection_vars(&mut file, vars);
    file
}

/// Build the root `opencollection.yml` for a non-bundled collection: no items
/// (they live on disk) and no inline environments (they live in files).
pub fn build_unbundled_root(
    base: Option<&OpenCollectionFile>,
    name: &str,
    version: &str,
    docs: Option<&str>,
    vars: &[KeyValuePair],
) -> OpenCollectionFile {
    let mut file = base.cloned().unwrap_or_else(default_file);
    file.info.name = name.to_string();
    if file.info.version.is_none() {
        file.info.version = Some(version.to_string());
    }
    apply_docs(&mut file, docs);
    file.bundled = Some(false);
    file.items = Vec::new();
    if let Some(cfg) = file.config.as_mut() {
        cfg.environments.clear();
    }
    apply_collection_vars(&mut file, vars);
    file
}

/// Write the collection docs onto `file.docs` as a string scalar when broquest
/// has markdown content. When `None`, leave any pre-existing `docs` (e.g. an
/// opaque block shape from the original file) untouched so it round-trips.
fn apply_docs(file: &mut OpenCollectionFile, docs: Option<&str>) {
    if let Some(text) = docs
        && !text.is_empty()
    {
        file.docs = Some(Value::from(text));
    }
}

/// Write collection vars onto the file's `extra` map under `x-broquest-vars`,
/// removing the key entirely when there are none so the output stays clean.
fn apply_collection_vars(file: &mut OpenCollectionFile, vars: &[KeyValuePair]) {
    match vars_to_yaml(vars) {
        Some(v) => {
            file.extra
                .insert(Value::String(BROQUEST_VARS_KEY.into()), v);
        }
        None => {
            file.extra.remove(Value::String(BROQUEST_VARS_KEY.into()));
        }
    }
}

/// Build a folder item (bundled): folder config plus its child items.
pub fn build_folder_item(source: Option<&OcItem>, name: &str, children: Vec<OcItem>) -> OcItem {
    let mut item = build_folder_config(source, name);
    item.items = Some(children);
    item
}

/// Build a folder config item (used as `folder.yml` for non-bundled: no
/// children, since requests live in sibling files).
pub fn build_folder_config(source: Option<&OcItem>, name: &str) -> OcItem {
    let mut item = source.cloned().unwrap_or_else(new_folder_item);
    let info = item.info.get_or_insert_with(|| OcItemInfo {
        name: String::new(),
        item_type: "folder".to_string(),
        description: None,
        seq: None,
        tags: None,
        extra: Mapping::new(),
    });
    info.name = name.to_string();
    info.item_type = "folder".to_string();
    item.items = None;
    item
}

fn new_folder_item() -> OcItem {
    OcItem {
        info: None,
        script_file_type: None,
        http: None,
        graphql: None,
        items: None,
        runtime: None,
        settings: None,
        examples: None,
        docs: None,
        script: None,
        extra: Mapping::new(),
    }
}

/// Merge modeled environment fields into `source`; if unchanged, return the
/// source verbatim so an open→save round-trip is lossless.
pub fn merge_env(source: Option<&OcEnvironment>, toml: &EnvironmentToml) -> OcEnvironment {
    if let Some(src) = source
        && &oc_environment_to_toml(src) == toml
    {
        return src.clone();
    }
    env_toml_to_oc(toml)
}

/// Serialize a full file to YAML text.
pub fn to_yaml_string(file: &OpenCollectionFile) -> Result<String> {
    serde_yaml_ng::to_string(file).context("Failed to serialize OpenCollection to YAML")
}

/// Serialize a single item (request or folder config) to YAML text.
pub fn item_to_yaml(item: &OcItem) -> Result<String> {
    serde_yaml_ng::to_string(item).context("Failed to serialize OpenCollection item to YAML")
}

/// Serialize a single environment to YAML text.
pub fn environment_to_yaml(env: &OcEnvironment) -> Result<String> {
    serde_yaml_ng::to_string(env).context("Failed to serialize OpenCollection environment to YAML")
}

// ---------------------------------------------------------------------------
// Value conversion helpers
// ---------------------------------------------------------------------------

fn yaml_to_json(v: &Value) -> serde_json::Value {
    serde_json::to_value(v).unwrap_or(serde_json::Value::Null)
}

fn json_to_yaml(v: &serde_json::Value) -> Value {
    serde_yaml_ng::to_value(v).unwrap_or(Value::Null)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FIXTURE: &str = r#"
opencollection: "1.0.0"
info:
  name: Sample API
  version: "2.0.0"
docs: A sample collection
bundled: true
config:
  environments:
    - name: Development
      variables:
        - name: BASE_URL
          value: https://api.example.com
        - secret: true
          name: API_KEY
          type: string
items:
  - info:
      name: Login
      type: http
    http:
      method: POST
      url: https://api.example.com/login
      headers:
        - name: Content-Type
          value: application/json
        - name: X-Debug
          value: "1"
          disabled: true
      params:
        - name: verbose
          value: "true"
          type: query
        - name: id
          value: "42"
          type: path
      body:
        type: json
        data: '{"user":"a"}'
      auth:
        type: basic
        username: admin
        password: secret
    runtime:
      scripts:
        - type: before-request
          code: "console.log('pre')"
        - type: after-response
          code: "console.log('post')"
  - info:
      name: Users
      type: folder
    items:
      - info:
          name: Get User
          type: http
        http:
          method: GET
          url: https://api.example.com/users/1
          auth:
            type: ntlm
            username: u
            password: p
            domain: d
extensions:
  vendor: custom-data
"#;

    /// Rebuild a bundled file from a loaded collection, preserving per-item
    /// sources (mirrors what the CollectionManager does on save).
    fn rebuild_bundled(loaded: &LoadedCollection) -> OpenCollectionFile {
        let mut items: Vec<OcItem> = loaded
            .root_requests
            .iter()
            .map(|lr| merge_request_into_item(Some(&lr.source), &lr.request))
            .collect();
        for group in &loaded.groups {
            let children = group
                .requests
                .iter()
                .map(|lr| merge_request_into_item(Some(&lr.source), &lr.request))
                .collect();
            items.push(build_folder_item(
                group.source.as_ref(),
                &group.name,
                children,
            ));
        }
        let environments = loaded
            .environments
            .iter()
            .map(|e| merge_env(Some(&e.source), &e.toml))
            .collect();
        assemble_bundled_file(
            Some(&loaded.file),
            &loaded.name,
            &loaded.version,
            loaded.docs.as_deref(),
            items,
            environments,
            &loaded.vars,
        )
    }

    #[test]
    fn test_parse_bundled_fixture() {
        let loaded = decompose_bundled(parse_opencollection(FIXTURE).expect("parse"));
        assert_eq!(loaded.name, "Sample API");
        assert_eq!(loaded.version, "2.0.0");
        assert_eq!(loaded.docs.as_deref(), Some("A sample collection"));
        assert_eq!(loaded.file.bundled, Some(true));
        assert_eq!(loaded.root_requests.len(), 1);
        assert_eq!(loaded.groups.len(), 1);
        assert_eq!(loaded.groups[0].name, "Users");
        assert_eq!(loaded.groups[0].requests.len(), 1);
        assert_eq!(loaded.environments.len(), 1);
        let env = &loaded.environments[0].toml;
        assert_eq!(env.name, "Development");
        assert_eq!(
            env.variables.get("BASE_URL").unwrap().value,
            "https://api.example.com"
        );
        assert!(env.variables.get("API_KEY").unwrap().secret);
        assert!(env.variables.get("API_KEY").unwrap().value.is_empty());
    }

    #[test]
    fn test_request_mapping() {
        let loaded = decompose_bundled(parse_opencollection(FIXTURE).expect("parse"));
        let login = &loaded.root_requests[0].request;
        assert_eq!(login.name, "Login");
        assert_eq!(login.method, HttpMethod::Post);
        assert_eq!(login.url, "https://api.example.com/login");
        assert_eq!(login.body, r#"{"user":"a"}"#);
        // header disabled -> enabled=false
        assert!(
            login
                .headers
                .iter()
                .any(|h| h.key == "X-Debug" && !h.enabled)
        );
        // params split by type
        assert!(login.query_params.iter().any(|p| p.key == "verbose"));
        assert!(login.path_params.iter().any(|p| p.key == "id"));
        assert!(
            matches!(login.auth, AuthType::Basic(ref b) if b.username == "admin" && b.password == "secret")
        );
        assert_eq!(
            login.pre_request_script.as_deref(),
            Some("console.log('pre')")
        );
        assert_eq!(
            login.post_response_script.as_deref(),
            Some("console.log('post')")
        );

        // Unsupported auth (ntlm) preserved as Unsupported.
        let get_user = &loaded.groups[0].requests[0].request;
        assert!(matches!(get_user.auth, AuthType::Unsupported { ref kind, .. } if kind == "ntlm"));
    }

    #[test]
    fn test_roundtrip_modeled_fields() {
        let loaded = decompose_bundled(parse_opencollection(FIXTURE).expect("parse"));
        let yaml = to_yaml_string(&rebuild_bundled(&loaded)).expect("serialize");
        let reloaded = decompose_bundled(parse_opencollection(&yaml).expect("reparse"));

        assert_eq!(reloaded.name, loaded.name);
        assert_eq!(reloaded.root_requests.len(), loaded.root_requests.len());
        assert_eq!(reloaded.groups.len(), loaded.groups.len());

        let a = &loaded.root_requests[0].request;
        let b = &reloaded.root_requests[0].request;
        assert_eq!(a, b, "request should survive the round-trip unchanged");

        // Unsupported auth survives the round-trip.
        assert_eq!(
            loaded.groups[0].requests[0].request.auth,
            reloaded.groups[0].requests[0].request.auth
        );
    }

    #[test]
    fn test_preserve_unknown_top_level_fields() {
        let loaded = decompose_bundled(parse_opencollection(FIXTURE).expect("parse"));
        let yaml = to_yaml_string(&rebuild_bundled(&loaded)).expect("serialize");
        // The unknown top-level `extensions` block must survive.
        assert!(yaml.contains("extensions"));
        assert!(yaml.contains("custom-data"));
    }

    #[test]
    fn test_roundtrip_oauth2_via_extension() {
        use crate::domain::OAuth2Auth;
        let mut req = RequestData {
            name: "OAuth".to_string(),
            method: HttpMethod::Get,
            url: "https://api.example.com".to_string(),
            auth: AuthType::OAuth2(OAuth2Auth {
                client_id: "cid".to_string(),
                client_secret: "csecret".to_string(),
                token_url: "https://auth/token".to_string(),
                ..Default::default()
            }),
            ..Default::default()
        };
        req.headers.clear();

        let item = merge_request_into_item(None, &req);
        let back = oc_item_to_request(&item).expect("request");
        match back.auth {
            AuthType::OAuth2(o) => {
                assert_eq!(o.client_id, "cid");
                assert_eq!(o.client_secret, "csecret");
                assert_eq!(o.token_url, "https://auth/token");
            }
            other => panic!("expected OAuth2, got {:?}", other),
        }
    }

    #[test]
    fn test_request_vars_roundtrip() {
        let req = RequestData {
            name: "With Vars".to_string(),
            method: HttpMethod::Get,
            url: "https://api.example.com".to_string(),
            vars: vec![
                KeyValuePair {
                    key: "token".to_string(),
                    value: "abc".to_string(),
                    enabled: true,
                },
                KeyValuePair {
                    key: "region".to_string(),
                    value: "eu".to_string(),
                    enabled: false,
                },
            ],
            ..Default::default()
        };

        // Vars are written into runtime.variables and read back.
        let item = merge_request_into_item(None, &req);
        assert!(
            item.runtime
                .as_ref()
                .and_then(|r| r.variables.as_ref())
                .is_some(),
            "vars should be written to runtime.variables"
        );
        let back = oc_item_to_request(&item).expect("request");
        assert!(KeyValuePair::vec_equals(&req.vars, &back.vars));

        // Serialize -> parse -> map: still intact.
        let yaml = item_to_yaml(&item).expect("serialize");
        let reparsed: OcItem = serde_yaml_ng::from_str(&yaml).expect("reparse");
        let back2 = oc_item_to_request(&reparsed).expect("request2");
        assert!(KeyValuePair::vec_equals(&req.vars, &back2.vars));
    }

    #[test]
    fn test_collection_vars_roundtrip() {
        use crate::domain::KeyValuePair;
        // Start from the standard fixture (no vars) and attach collection vars.
        let mut loaded = decompose_bundled(parse_opencollection(FIXTURE).expect("parse"));
        loaded.vars = vec![
            KeyValuePair {
                key: "namespace".to_string(),
                value: "production".to_string(),
                enabled: true,
            },
            KeyValuePair {
                key: "feature.flag".to_string(),
                value: "on".to_string(),
                enabled: false,
            },
        ];

        let yaml = to_yaml_string(&rebuild_bundled(&loaded)).expect("serialize");
        // The broquest-only key must be present in the serialized output.
        assert!(yaml.contains(BROQUEST_VARS_KEY), "vars key missing in YAML");

        let reloaded = decompose_bundled(parse_opencollection(&yaml).expect("reparse"));
        assert_eq!(reloaded.vars.len(), 2);
        assert_eq!(reloaded.vars[0].key, "namespace");
        assert_eq!(reloaded.vars[0].value, "production");
        assert!(reloaded.vars[0].enabled);
        assert_eq!(reloaded.vars[1].key, "feature.flag");
        assert!(!reloaded.vars[1].enabled);

        // A file with no vars must round-trip to no vars and NOT emit the key.
        let no_vars = decompose_bundled(parse_opencollection(FIXTURE).expect("parse"));
        let yaml_none = to_yaml_string(&rebuild_bundled(&no_vars)).expect("serialize");
        assert!(
            !yaml_none.contains(BROQUEST_VARS_KEY),
            "vars key should be absent when there are no vars"
        );
        let reloaded_none = decompose_bundled(parse_opencollection(&yaml_none).expect("reparse"));
        assert!(reloaded_none.vars.is_empty());
    }

    #[test]
    fn test_docs_roundtrip() {
        // String-scalar docs survive a bundled read/write round-trip.
        let mut loaded = decompose_bundled(parse_opencollection(FIXTURE).expect("parse"));
        assert_eq!(loaded.docs.as_deref(), Some("A sample collection"));

        let edited = "# My Collection\n\nSome **markdown** docs.\n".to_string();
        loaded.docs = Some(edited.clone());
        let yaml = to_yaml_string(&rebuild_bundled(&loaded)).expect("serialize");
        assert!(yaml.contains("docs:"));

        let reloaded = decompose_bundled(parse_opencollection(&yaml).expect("reparse"));
        assert_eq!(reloaded.docs.as_deref(), Some(edited.as_str()));
    }

    #[test]
    fn test_docs_array_blocks_extracted_best_effort() {
        // An array of content blocks is joined into markdown from text blocks;
        // unsupported block shapes are skipped, not fatal.
        let yaml = r#"
opencollection: "1.0.0"
info:
  name: Blocked
docs:
  - "First paragraph."
  - text: "Second paragraph."
  - type: image
    src: "ignored.png"
items: []
"#;
        let loaded = decompose_bundled(parse_opencollection(yaml).expect("parse"));
        let docs = loaded.docs.expect("docs should be extracted");
        assert!(docs.contains("First paragraph."));
        assert!(docs.contains("Second paragraph."));
    }
}
