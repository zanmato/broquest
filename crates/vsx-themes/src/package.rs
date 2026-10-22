//! Reading a VS Code extension, from its `.vsix` or an unpacked folder, and
//! loading the color themes it contributes.

use std::collections::HashMap;
use std::io::Read as _;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, anyhow, bail};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::color::Rgba;

/// Files above this are not themes. A `.vsix` can bundle screenshots or a
/// whole language server, which are never read.
const MAX_FILE_BYTES: u64 = 8 * 1024 * 1024;
/// How deep `include` chains may go, against a theme that includes itself.
const MAX_INCLUDE_DEPTH: usize = 8;

/// The parts of an extension's `package.json` the conversion reads.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Manifest {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub publisher: String,
    #[serde(default)]
    pub version: String,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub repository: Option<Value>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub contributes: Contributes,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Contributes {
    #[serde(default)]
    pub themes: Vec<ThemeContribution>,
}

/// One entry of `contributes.themes`.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeContribution {
    #[serde(default)]
    pub id: Option<String>,
    #[serde(default)]
    pub label: Option<String>,
    #[serde(default)]
    pub ui_theme: Option<String>,
    pub path: String,
}

impl ThemeContribution {
    /// `vs` and `hc-light` are light, `vs-dark` and `hc-black` dark. `None`
    /// when the manifest does not say.
    pub fn is_light(&self) -> Option<bool> {
        self.ui_theme
            .as_deref()
            .map(|ui_theme| matches!(ui_theme, "vs" | "hc-light"))
    }
}

impl Manifest {
    pub fn parse(text: &str) -> anyhow::Result<Self> {
        Ok(serde_json::from_str(&strip_jsonc(text))?)
    }

    pub fn has_color_themes(&self) -> bool {
        !self.contributes.themes.is_empty()
    }

    /// The repository or homepage, for the theme file's `url`.
    pub fn url(&self) -> Option<String> {
        let repository = match &self.repository {
            Some(Value::String(url)) => Some(url.clone()),
            Some(Value::Object(object)) => object
                .get("url")
                .and_then(Value::as_str)
                .map(str::to_string),
            _ => None,
        };
        repository.or_else(|| self.homepage.clone()).map(|url| {
            url.trim_start_matches("git+")
                .trim_end_matches(".git")
                .to_string()
        })
    }
}

enum Source {
    /// Paths relative to the extension root, as `/`-separated strings.
    Files(HashMap<String, Vec<u8>>),
    Folder(PathBuf),
}

/// An extension's files, read from a `.vsix` or a folder.
pub struct Package {
    source: Source,
    pub manifest: Manifest,
    /// `package.nls.json`, which `%key%` labels resolve through.
    strings: Map<String, Value>,
}

/// A VS Code theme with its `include` chain merged in.
#[derive(Debug, Clone, Default)]
pub struct VsCodeTheme {
    pub name: String,
    /// From the contribution's `uiTheme`, else the file's `type`.
    pub light: Option<bool>,
    pub colors: HashMap<String, Rgba>,
    pub token_colors: Vec<TokenRule>,
}

/// One `tokenColors` entry.
#[derive(Debug, Clone, Default)]
pub struct TokenRule {
    /// Each selector of the rule, as its space-separated scopes.
    pub selectors: Vec<Vec<String>>,
    pub foreground: Option<Rgba>,
    /// `Some("")` resets the style, as VS Code reads it.
    pub font_style: Option<String>,
}

impl Package {
    /// Read the theme-relevant files of a `.vsix`. Files sit under
    /// `extension/` in the archive.
    pub fn from_vsix(bytes: &[u8]) -> anyhow::Result<Self> {
        let mut archive =
            zip::ZipArchive::new(std::io::Cursor::new(bytes)).context("not a .vsix archive")?;
        let mut files = HashMap::new();
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            if !entry.is_file() || entry.size() > MAX_FILE_BYTES {
                continue;
            }
            let Some(path) = entry.name().strip_prefix("extension/").map(str::to_string) else {
                continue;
            };
            let lower = path.to_ascii_lowercase();
            if !(lower.ends_with(".json")
                || lower.ends_with(".jsonc")
                || lower.ends_with(".tmtheme")
                || lower.ends_with(".plist"))
            {
                continue;
            }
            let mut content = Vec::with_capacity(entry.size() as usize);
            entry.read_to_end(&mut content)?;
            files.insert(path, content);
        }
        Self::new(Source::Files(files))
    }

    /// Read an unpacked extension, such as a clone of a theme's repository.
    pub fn from_folder(path: &Path) -> anyhow::Result<Self> {
        Self::new(Source::Folder(path.to_path_buf()))
    }

    fn new(source: Source) -> anyhow::Result<Self> {
        let mut package = Self {
            source,
            manifest: Manifest::default(),
            strings: Map::new(),
        };
        let manifest = package.read_text("package.json")?;
        package.manifest = Manifest::parse(&manifest).context("package.json")?;
        if let Ok(strings) = package.read_text("package.nls.json") {
            match serde_json::from_str::<Value>(&strip_jsonc(&strings)) {
                Ok(Value::Object(strings)) => package.strings = strings,
                Ok(_) => {}
                Err(error) => tracing::warn!("Ignoring unreadable package.nls.json: {error}"),
            }
        }
        Ok(package)
    }

    fn read(&self, path: &str) -> anyhow::Result<Vec<u8>> {
        match &self.source {
            Source::Files(files) => files
                .get(path)
                .cloned()
                .ok_or_else(|| anyhow!("{path} is not in the package")),
            Source::Folder(root) => {
                let full = root.join(path);
                let size = std::fs::metadata(&full)
                    .with_context(|| format!("{path} is not in the package"))?
                    .len();
                if size > MAX_FILE_BYTES {
                    bail!("{path} is too large to be a theme");
                }
                Ok(std::fs::read(full)?)
            }
        }
    }

    fn read_text(&self, path: &str) -> anyhow::Result<String> {
        let bytes = self.read(path)?;
        let text = String::from_utf8(bytes).with_context(|| format!("{path} is not UTF-8"))?;
        Ok(text.trim_start_matches('\u{feff}').to_string())
    }

    /// `text` with a `%key%` placeholder replaced from `package.nls.json`.
    pub fn localized(&self, text: &str) -> String {
        text.strip_prefix('%')
            .and_then(|rest| rest.strip_suffix('%'))
            .and_then(|key| self.strings.get(key))
            .and_then(|value| match value {
                Value::String(text) => Some(text.clone()),
                // Newer extensions write `{"message": ..., "comment": ...}`.
                Value::Object(object) => object
                    .get("message")
                    .and_then(Value::as_str)
                    .map(str::to_string),
                _ => None,
            })
            .unwrap_or_else(|| text.to_string())
    }

    pub fn display_name(&self) -> String {
        let name = self
            .manifest
            .display_name
            .clone()
            .unwrap_or_else(|| self.manifest.name.clone());
        self.localized(&name)
    }

    pub fn color_themes(&self) -> &[ThemeContribution] {
        &self.manifest.contributes.themes
    }

    /// Load the theme `contribution` points at, following its `include`s.
    pub fn load_theme(&self, contribution: &ThemeContribution) -> anyhow::Result<VsCodeTheme> {
        let path = normalize(Path::new(""), &contribution.path)?;
        let mut theme = VsCodeTheme::default();
        self.merge_file(&path, &mut theme, 0)?;
        let label = contribution
            .label
            .as_deref()
            .or(contribution.id.as_deref())
            .map(|label| self.localized(label))
            .filter(|label| !label.trim().is_empty());
        if let Some(label) = label {
            theme.name = label;
        }
        if theme.name.trim().is_empty() {
            theme.name = Path::new(&path)
                .file_stem()
                .map(|stem| stem.to_string_lossy().to_string())
                .unwrap_or_else(|| self.display_name());
        }
        if let Some(light) = contribution.is_light() {
            theme.light = Some(light);
        }
        Ok(theme)
    }

    /// Merge the theme file at `path` into `theme`: its include first, then
    /// its own entries over it, as VS Code does.
    fn merge_file(&self, path: &str, theme: &mut VsCodeTheme, depth: usize) -> anyhow::Result<()> {
        if depth > MAX_INCLUDE_DEPTH {
            bail!("{path}: include chain is too deep");
        }
        let base = Path::new(path).parent().unwrap_or(Path::new(""));
        let text = self.read_text(path)?;
        if is_plist(path) {
            theme
                .token_colors
                .extend(plist_token_rules(text.as_bytes())?);
            return Ok(());
        }
        let document: Value =
            serde_json::from_str(&strip_jsonc(&text)).with_context(|| path.to_string())?;

        if let Some(include) = document.get("include").and_then(Value::as_str) {
            self.merge_file(&normalize(base, include)?, theme, depth + 1)?;
        }
        if let Some(name) = document.get("name").and_then(Value::as_str) {
            theme.name = name.to_string();
        }
        if let Some(kind) = document.get("type").and_then(Value::as_str) {
            theme.light = Some(matches!(kind, "light" | "hcLight"));
        }
        if let Some(colors) = document.get("colors").and_then(Value::as_object) {
            for (key, value) in colors {
                match value.as_str().and_then(Rgba::parse) {
                    Some(color) => {
                        theme.colors.insert(key.clone(), color);
                    }
                    // `null` clears a color an include set.
                    None => {
                        theme.colors.remove(key);
                    }
                }
            }
        }
        match document.get("tokenColors") {
            Some(Value::Array(rules)) => theme.token_colors.extend(rules.iter().map(token_rule)),
            Some(Value::String(file)) => {
                let file = normalize(base, file)?;
                self.merge_file(&file, theme, depth + 1)?;
            }
            _ => {}
        }
        Ok(())
    }
}

fn is_plist(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".tmtheme") || lower.ends_with(".plist")
}

/// Resolve `relative` against `base`, inside the package.
fn normalize(base: &Path, relative: &str) -> anyhow::Result<String> {
    let mut parts: Vec<String> = base
        .components()
        .map(|part| part.as_os_str().to_string_lossy().to_string())
        .collect();
    for part in relative.split(['/', '\\']) {
        match part {
            "" | "." => {}
            ".." => {
                if parts.pop().is_none() {
                    bail!("{relative} points outside the package");
                }
            }
            part => parts.push(part.to_string()),
        }
    }
    Ok(parts.join("/"))
}

fn token_rule(rule: &Value) -> TokenRule {
    let selectors = match rule.get("scope") {
        Some(Value::String(scopes)) => scopes.split(',').map(selector).collect(),
        Some(Value::Array(scopes)) => scopes
            .iter()
            .filter_map(Value::as_str)
            .flat_map(|scopes| scopes.split(','))
            .map(selector)
            .collect(),
        // A rule without a scope sets the editor defaults.
        _ => vec![Vec::new()],
    };
    let settings = rule.get("settings");
    TokenRule {
        selectors,
        foreground: settings
            .and_then(|settings| settings.get("foreground"))
            .and_then(Value::as_str)
            .and_then(Rgba::parse),
        font_style: settings
            .and_then(|settings| settings.get("fontStyle"))
            .and_then(Value::as_str)
            .map(str::to_string),
    }
}

fn selector(text: &str) -> Vec<String> {
    text.split_whitespace().map(str::to_string).collect()
}

/// The `settings` array of a TextMate `.tmTheme`.
fn plist_token_rules(bytes: &[u8]) -> anyhow::Result<Vec<TokenRule>> {
    let document = plist::Value::from_reader(std::io::Cursor::new(bytes))?;
    let settings = document
        .as_dictionary()
        .and_then(|root| root.get("settings"))
        .and_then(plist::Value::as_array)
        .ok_or_else(|| anyhow!("tmTheme has no settings"))?;
    Ok(settings
        .iter()
        .filter_map(plist::Value::as_dictionary)
        .map(|rule| {
            let scope = rule.get("scope").and_then(plist::Value::as_string);
            let settings = rule.get("settings").and_then(plist::Value::as_dictionary);
            let text = |key: &str| {
                settings
                    .and_then(|settings| settings.get(key))
                    .and_then(plist::Value::as_string)
            };
            TokenRule {
                selectors: match scope {
                    Some(scope) => scope.split(',').map(selector).collect(),
                    None => vec![Vec::new()],
                },
                foreground: text("foreground").and_then(Rgba::parse),
                font_style: text("fontStyle").map(str::to_string),
            }
        })
        .collect())
}

/// JSON with comments and trailing commas, as VS Code reads its files, turned
/// into plain JSON.
pub fn strip_jsonc(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    while let Some(character) = chars.next() {
        if in_string {
            out.push(character);
            match character {
                '\\' => {
                    if let Some(escaped) = chars.next() {
                        out.push(escaped);
                    }
                }
                '"' => in_string = false,
                _ => {}
            }
            continue;
        }
        match (character, chars.peek()) {
            ('"', _) => {
                in_string = true;
                out.push(character);
            }
            ('/', Some('/')) => {
                for skipped in chars.by_ref() {
                    if skipped == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            ('/', Some('*')) => {
                chars.next();
                let mut previous = '\0';
                for skipped in chars.by_ref() {
                    if previous == '*' && skipped == '/' {
                        break;
                    }
                    previous = skipped;
                }
                out.push(' ');
            }
            _ => out.push(character),
        }
    }
    remove_trailing_commas(&out)
}

fn remove_trailing_commas(text: &str) -> String {
    let characters: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    let mut in_string = false;
    let mut index = 0;
    while let Some(&character) = characters.get(index) {
        if in_string {
            out.push(character);
            if character == '\\' {
                if let Some(&escaped) = characters.get(index + 1) {
                    out.push(escaped);
                    index += 1;
                }
            } else if character == '"' {
                in_string = false;
            }
        } else if character == '"' {
            in_string = true;
            out.push(character);
        } else if character == ',' {
            let next = characters[index + 1..]
                .iter()
                .find(|next| !next.is_whitespace());
            if !matches!(next, Some('}') | Some(']')) {
                out.push(character);
            }
        } else {
            out.push(character);
        }
        index += 1;
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn comments_and_trailing_commas_are_removed() {
        let text = r##"{
            // a comment
            "url": "http://example.com", /* block */
            "color": "#ffffff", //"#000000",
            "list": [1, 2,],
            "quote": "say \"hi\" // not a comment",
        }"##;
        let value: Value = serde_json::from_str(&strip_jsonc(text)).unwrap();
        assert_eq!(value["url"], "http://example.com");
        assert_eq!(value["color"], "#ffffff");
        assert_eq!(value["list"], serde_json::json!([1, 2]));
        assert_eq!(value["quote"], "say \"hi\" // not a comment");
    }

    #[test]
    fn paths_resolve_inside_the_package() {
        assert_eq!(
            normalize(Path::new("themes"), "./base/dark.json").unwrap(),
            "themes/base/dark.json"
        );
        assert_eq!(
            normalize(Path::new("themes"), "../x.json").unwrap(),
            "x.json"
        );
        assert!(normalize(Path::new(""), "../x.json").is_err());
    }

    fn write(dir: &Path, path: &str, content: &str) {
        let full = dir.join(path);
        std::fs::create_dir_all(full.parent().unwrap()).unwrap();
        std::fs::write(full, content).unwrap();
    }

    #[test]
    fn includes_merge_under_the_including_theme() {
        let dir = tempfile::tempdir().unwrap();
        write(
            dir.path(),
            "package.json",
            r#"{"name": "x", "publisher": "p", "displayName": "%name%",
                "contributes": {"themes": [
                    {"label": "%dark%", "uiTheme": "vs-dark", "path": "./themes/dark.json"}
                ]}}"#,
        );
        write(
            dir.path(),
            "package.nls.json",
            r#"{"name": "Pretty", "dark": {"message": "Pretty Dark"}}"#,
        );
        write(
            dir.path(),
            "themes/base.json",
            r##"{"colors": {"editor.background": "#000000", "sideBar.background": "#111111"},
                "tokenColors": [{"scope": "comment", "settings": {"foreground": "#222222"}}]}"##,
        );
        write(
            dir.path(),
            "themes/dark.json",
            r##"{"include": "./base.json", "type": "light",
                "colors": {"editor.background": "#101010", "sideBar.background": null},
                "tokenColors": [{"scope": "string, keyword.control", "settings": {"foreground": "#333333"}}]}"##,
        );

        let package = Package::from_folder(dir.path()).unwrap();
        assert_eq!(package.display_name(), "Pretty");
        let theme = package.load_theme(&package.color_themes()[0]).unwrap();
        assert_eq!(theme.name, "Pretty Dark");
        // The contribution's uiTheme wins over the file's type.
        assert_eq!(theme.light, Some(false));
        assert_eq!(theme.colors["editor.background"].to_hex(), "#101010");
        assert!(!theme.colors.contains_key("sideBar.background"));
        assert_eq!(theme.token_colors.len(), 2);
        assert_eq!(
            theme.token_colors[1].selectors,
            [
                vec!["string".to_string()],
                vec!["keyword.control".to_string()]
            ]
        );
    }

    #[test]
    fn tm_theme_token_colors_are_read() {
        let tm_theme = r#"<?xml version="1.0" encoding="UTF-8"?>
<plist version="1.0"><dict><key>settings</key><array>
  <dict><key>settings</key><dict><key>foreground</key><string>#F8F8F2</string></dict></dict>
  <dict><key>scope</key><string>comment</string>
    <key>settings</key><dict><key>foreground</key><string>#75715E</string>
    <key>fontStyle</key><string>italic</string></dict></dict>
</array></dict></plist>"#;
        let rules = plist_token_rules(tm_theme.as_bytes()).unwrap();
        assert_eq!(rules.len(), 2);
        assert_eq!(rules[0].selectors, [Vec::<String>::new()]);
        assert_eq!(rules[1].foreground.unwrap().to_hex(), "#75715e");
        assert_eq!(rules[1].font_style.as_deref(), Some("italic"));
    }
}
