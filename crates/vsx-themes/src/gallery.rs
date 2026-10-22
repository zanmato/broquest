//! An extension gallery that speaks VS Code's gallery API: Open VSX's VS Code
//! adapter (https://open-vsx.org/swagger-ui/index.html?urls.primaryName=VSCode%20Adapter)
//! by default, or a self-hosted Open VSX at its `/vscode/gallery` path.
//!
//! The calls block, so callers run them on a background thread.

use std::collections::HashMap;
use std::io::Read as _;
use std::time::Duration;

use anyhow::{Context as _, bail};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::package::Manifest;

pub const OPEN_VSX: &str = "https://open-vsx.org/vscode/gallery";
/// A theme `.vsix` is a few hundred kilobytes. This only stops a runaway.
const MAX_DOWNLOAD_BYTES: u64 = 64 * 1024 * 1024;
/// The longest a rate limited request waits before its one retry.
const MAX_RETRY_WAIT: Duration = Duration::from_secs(10);

// Filter types, sort keys and flags of the gallery API.
const FILTER_CATEGORY: u32 = 5;
const FILTER_EXTENSION_NAME: u32 = 7;
const FILTER_TARGET: u32 = 8;
const FILTER_SEARCH_TEXT: u32 = 10;
const FILTER_EXCLUDE_WITH_FLAGS: u32 = 12;
const SORT_BY_RELEVANCE: u32 = 0;
const SORT_BY_INSTALLS: u32 = 4;
const FLAG_INCLUDE_FILES: u32 = 0x2;
const FLAG_INCLUDE_CATEGORY_AND_TAGS: u32 = 0x4;
const FLAG_INCLUDE_VERSION_PROPERTIES: u32 = 0x10;
const FLAG_INCLUDE_ASSET_URI: u32 = 0x80;
const FLAG_INCLUDE_STATISTICS: u32 = 0x100;
const FLAG_LATEST_VERSION_ONLY: u32 = 0x200;
/// Extensions flagged unpublished.
const FLAG_UNPUBLISHED: u32 = 0x1000;

const ASSET_MANIFEST: &str = "Microsoft.VisualStudio.Code.Manifest";
const ASSET_ICON: &str = "Microsoft.VisualStudio.Services.Icons.Small";
const ASSET_ICON_LARGE: &str = "Microsoft.VisualStudio.Services.Icons.Default";
const ASSET_PACKAGE: &str = "Microsoft.VisualStudio.Services.VSIXPackage";

/// An extension as the search lists it.
#[derive(Debug, Clone)]
pub struct Extension {
    pub publisher: String,
    pub publisher_display_name: String,
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub installs: u64,
    pub rating: Option<f32>,
    tags: Vec<String>,
    /// Asset URLs by asset type.
    assets: HashMap<String, String>,
}

impl Extension {
    /// `publisher.name`, the extension's id everywhere in VS Code.
    pub fn id(&self) -> String {
        format!("{}.{}", self.publisher, self.name)
    }

    /// Whether the extension has color themes, as far as its tags tell.
    /// `vsce` tags a package by what it contributes, so `color-theme` is
    /// reliable, and an icon theme tag without it means icons only. Packages
    /// made by older tools have neither and need their manifest read.
    pub fn has_color_themes_by_tags(&self) -> Option<bool> {
        let tagged = |tag: &str| self.tags.iter().any(|candidate| candidate == tag);
        if tagged("color-theme") {
            Some(true)
        } else if tagged("icon-theme") || tagged("product-icon-theme") {
            Some(false)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct SearchPage {
    pub extensions: Vec<Extension>,
    /// Matches across all pages.
    pub total: u64,
}

pub struct Client {
    http: reqwest::blocking::Client,
    gallery: String,
}

impl Client {
    /// A client for the gallery at `gallery`, such as [`OPEN_VSX`].
    pub fn new(gallery: &str) -> anyhow::Result<Self> {
        let http = reqwest::blocking::Client::builder()
            .user_agent(concat!("vsx-themes/", env!("CARGO_PKG_VERSION")))
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(60))
            .build()?;
        Ok(Self {
            http,
            gallery: gallery.trim_end_matches('/').to_string(),
        })
    }

    /// Theme extensions matching `query`, best match first, or the most
    /// installed themes for an empty query. `page` counts from 1.
    pub fn search(&self, query: &str, page: u32, page_size: u32) -> anyhow::Result<SearchPage> {
        let mut criteria = vec![json!({"filterType": FILTER_CATEGORY, "value": "Themes"})];
        let query = query.trim();
        let sort_by = if query.is_empty() {
            SORT_BY_INSTALLS
        } else {
            criteria.push(json!({"filterType": FILTER_SEARCH_TEXT, "value": query}));
            SORT_BY_RELEVANCE
        };
        self.query(criteria, sort_by, page, page_size)
    }

    /// The extension with the id `publisher.name`.
    pub fn find(&self, id: &str) -> anyhow::Result<Extension> {
        let criteria = vec![json!({"filterType": FILTER_EXTENSION_NAME, "value": id})];
        self.query(criteria, SORT_BY_RELEVANCE, 1, 1)?
            .extensions
            .into_iter()
            .next()
            .with_context(|| format!("{id} is not in the gallery"))
    }

    fn query(
        &self,
        mut criteria: Vec<Value>,
        sort_by: u32,
        page: u32,
        page_size: u32,
    ) -> anyhow::Result<SearchPage> {
        criteria.push(json!({"filterType": FILTER_TARGET, "value": "Microsoft.VisualStudio.Code"}));
        criteria.push(
            json!({"filterType": FILTER_EXCLUDE_WITH_FLAGS, "value": FLAG_UNPUBLISHED.to_string()}),
        );
        let body = json!({
            "filters": [{
                "criteria": criteria,
                "pageNumber": page,
                "pageSize": page_size,
                "sortBy": sort_by,
                "sortOrder": 0,
            }],
            "assetTypes": [],
            "flags": FLAG_INCLUDE_FILES
                | FLAG_INCLUDE_CATEGORY_AND_TAGS
                | FLAG_INCLUDE_VERSION_PROPERTIES
                | FLAG_INCLUDE_ASSET_URI
                | FLAG_INCLUDE_STATISTICS
                | FLAG_LATEST_VERSION_ONLY,
        });
        let url = format!("{}/extensionquery", self.gallery);
        let response: QueryResponse = self
            .send(|| {
                self.http
                    .post(&url)
                    .header(
                        reqwest::header::ACCEPT,
                        "application/json;api-version=7.2-preview.1",
                    )
                    .json(&body)
            })
            .context("Gallery search failed")?
            .json()
            .context("The gallery answered the search with something unexpected")?;
        Ok(parse_query(response))
    }

    /// The extension's `package.json`, which says whether it has color
    /// themes at all. Icon themes are listed under Themes as well.
    pub fn manifest(&self, extension: &Extension) -> anyhow::Result<Manifest> {
        let bytes = self.asset(extension, ASSET_MANIFEST)?;
        Manifest::parse(&String::from_utf8_lossy(&bytes))
    }

    /// `None` when the extension has no icon.
    pub fn icon(&self, extension: &Extension) -> anyhow::Result<Option<Vec<u8>>> {
        [ASSET_ICON, ASSET_ICON_LARGE]
            .into_iter()
            .find(|asset_type| extension.assets.contains_key(*asset_type))
            .map(|asset_type| self.asset(extension, asset_type))
            .transpose()
    }

    /// The extension's `.vsix`.
    pub fn download(&self, extension: &Extension) -> anyhow::Result<Vec<u8>> {
        self.asset(extension, ASSET_PACKAGE)
    }

    fn asset(&self, extension: &Extension, asset_type: &str) -> anyhow::Result<Vec<u8>> {
        let Some(url) = extension.assets.get(asset_type) else {
            bail!("{} has no {asset_type}", extension.id());
        };
        let response = self
            .send(|| self.http.get(url))
            .with_context(|| format!("downloading {asset_type} of {}", extension.id()))?;
        let mut bytes = Vec::new();
        response
            .take(MAX_DOWNLOAD_BYTES + 1)
            .read_to_end(&mut bytes)?;
        if bytes.len() as u64 > MAX_DOWNLOAD_BYTES {
            bail!("{} is too large", extension.id());
        }
        Ok(bytes)
    }

    /// Send the request `build` makes. Open VSX limits the request rate and
    /// answers 429 with how long to wait, so that is waited out once.
    fn send(
        &self,
        build: impl Fn() -> reqwest::blocking::RequestBuilder,
    ) -> anyhow::Result<reqwest::blocking::Response> {
        let response = build().send()?;
        if response.status() != reqwest::StatusCode::TOO_MANY_REQUESTS {
            return Ok(response.error_for_status()?);
        }
        let wait = response
            .headers()
            .get(reqwest::header::RETRY_AFTER)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.trim().parse::<u64>().ok())
            .map(Duration::from_secs)
            .unwrap_or(Duration::from_secs(1));
        if wait > MAX_RETRY_WAIT {
            bail!(
                "the gallery is rate limiting requests, try again in {} seconds",
                wait.as_secs()
            );
        }
        std::thread::sleep(wait);
        Ok(build().send()?.error_for_status()?)
    }
}

fn parse_query(response: QueryResponse) -> SearchPage {
    let Some(result) = response.results.into_iter().next() else {
        return SearchPage {
            extensions: Vec::new(),
            total: 0,
        };
    };
    let total = result
        .result_metadata
        .iter()
        .filter(|metadata| metadata.metadata_type == "ResultCount")
        .flat_map(|metadata| &metadata.metadata_items)
        .find(|item| item.name == "TotalCount")
        .map(|item| item.count)
        .unwrap_or(0);
    SearchPage {
        extensions: result
            .extensions
            .into_iter()
            .filter_map(RawExtension::into_extension)
            .collect(),
        total,
    }
}

#[derive(Deserialize)]
struct QueryResponse {
    #[serde(default)]
    results: Vec<QueryResult>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct QueryResult {
    #[serde(default)]
    extensions: Vec<RawExtension>,
    #[serde(default)]
    result_metadata: Vec<ResultMetadata>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ResultMetadata {
    metadata_type: String,
    #[serde(default)]
    metadata_items: Vec<MetadataItem>,
}

#[derive(Deserialize)]
struct MetadataItem {
    name: String,
    count: u64,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawExtension {
    extension_name: String,
    #[serde(default)]
    display_name: String,
    #[serde(default)]
    short_description: String,
    publisher: RawPublisher,
    #[serde(default)]
    versions: Vec<RawVersion>,
    #[serde(default)]
    statistics: Vec<RawStatistic>,
    #[serde(default)]
    tags: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPublisher {
    publisher_name: String,
    #[serde(default)]
    display_name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawVersion {
    version: String,
    #[serde(default)]
    files: Vec<RawFile>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawFile {
    asset_type: String,
    source: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawStatistic {
    statistic_name: String,
    value: f64,
}

impl RawExtension {
    fn into_extension(self) -> Option<Extension> {
        let version = self.versions.into_iter().next()?;
        let statistic = |name: &str| {
            self.statistics
                .iter()
                .find(|statistic| statistic.statistic_name == name)
                .map(|statistic| statistic.value)
        };
        Some(Extension {
            display_name: if self.display_name.is_empty() {
                self.extension_name.clone()
            } else {
                self.display_name
            },
            publisher_display_name: if self.publisher.display_name.is_empty() {
                self.publisher.publisher_name.clone()
            } else {
                self.publisher.display_name
            },
            publisher: self.publisher.publisher_name,
            name: self.extension_name,
            description: self.short_description,
            version: version.version,
            installs: statistic("install").unwrap_or(0.0) as u64,
            rating: statistic("averagerating").map(|rating| rating as f32),
            tags: self.tags,
            assets: version
                .files
                .into_iter()
                .map(|file| (file.asset_type, file.source))
                .collect(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tagged(tags: &[&str]) -> Extension {
        Extension {
            publisher: "p".into(),
            publisher_display_name: "p".into(),
            name: "n".into(),
            display_name: "n".into(),
            description: String::new(),
            version: "1.0.0".into(),
            installs: 0,
            rating: None,
            tags: tags.iter().map(|tag| tag.to_string()).collect(),
            assets: HashMap::new(),
        }
    }

    #[test]
    fn tags_tell_color_themes_from_icon_themes() {
        assert_eq!(
            tagged(&["theme", "color-theme"]).has_color_themes_by_tags(),
            Some(true)
        );
        // An extension with both kinds is offered for its color themes.
        assert_eq!(
            tagged(&["color-theme", "icon-theme"]).has_color_themes_by_tags(),
            Some(true)
        );
        assert_eq!(
            tagged(&["icon-theme", "theme"]).has_color_themes_by_tags(),
            Some(false)
        );
        assert_eq!(
            tagged(&["product-icon-theme"]).has_color_themes_by_tags(),
            Some(false)
        );
        // Darcula was packaged before `vsce` tagged themes.
        assert_eq!(tagged(&[]).has_color_themes_by_tags(), None);
        assert_eq!(tagged(&["theme", "dark"]).has_color_themes_by_tags(), None);
    }

    /// Trimmed from an Open VSX answer.
    #[test]
    fn a_query_result_becomes_extensions() {
        let response = r#"{"results": [{
            "extensions": [{
                "extensionName": "tokyo-night",
                "displayName": "Tokyo Night",
                "shortDescription": "A clean theme",
                "publisher": {"publisherName": "enkia", "displayName": "enkia"},
                "versions": [{"version": "1.1.2", "files": [
                    {"assetType": "Microsoft.VisualStudio.Services.VSIXPackage",
                     "source": "https://open-vsx.org/vscode/asset/enkia/tokyo-night/1.1.2/Microsoft.VisualStudio.Services.VSIXPackage"},
                    {"assetType": "Microsoft.VisualStudio.Services.Icons.Default",
                     "source": "https://open-vsx.org/vscode/asset/enkia/tokyo-night/1.1.2/Microsoft.VisualStudio.Services.Icons.Default"}
                ]}],
                "tags": ["color-theme", "theme", "tokyo"],
                "statistics": [{"statisticName": "install", "value": 160685.0},
                               {"statisticName": "averagerating", "value": 5.0}]
            }, {
                "extensionName": "no-versions",
                "publisher": {"publisherName": "p"}
            }],
            "resultMetadata": [{"metadataType": "ResultCount",
                "metadataItems": [{"name": "TotalCount", "count": 134}]}]
        }]}"#;
        let page = parse_query(serde_json::from_str(response).unwrap());
        assert_eq!(page.total, 134);
        let [tokyo] = page.extensions.as_slice() else {
            panic!("one extension expected");
        };
        assert_eq!(tokyo.id(), "enkia.tokyo-night");
        assert_eq!(tokyo.installs, 160685);
        assert_eq!(tokyo.rating, Some(5.0));
        assert_eq!(tokyo.has_color_themes_by_tags(), Some(true));
        assert_eq!(
            tokyo.assets[ASSET_PACKAGE],
            "https://open-vsx.org/vscode/asset/enkia/tokyo-night/1.1.2/Microsoft.VisualStudio.Services.VSIXPackage"
        );
        assert!(!tokyo.assets.contains_key(ASSET_ICON));
    }
}
