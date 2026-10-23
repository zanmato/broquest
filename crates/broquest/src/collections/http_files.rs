//! Collections made of `.http` and `.rest` files, as used by the JetBrains
//! HTTP Client and the VS Code REST Client.
//!
//! Each file is a group and each request in it a request of that group.
//! The file's `@name = value` variables become variables of every request in
//! it. Environments come from `http-client.env.json` and
//! `http-client.private.env.json` (JetBrains) and from the
//! `rest-client.environmentVariables` setting in `.vscode/settings.json`
//! (VS Code). A `$shared` environment's values go into every environment.
//!
//! These collections are read-only for now: saving writes nothing.

use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};

use super::manager::GroupInfo;
use super::types::{CollectionMeta, CollectionToml, EnvironmentToml, EnvironmentVariable};
use crate::domain::{HttpMethod, KeyValuePair, RequestData};

/// Folders that never hold a collection's request files.
const SKIPPED_DIRS: &[&str] = &["node_modules", "target", "dist", "build", "vendor"];
const MAX_DEPTH: usize = 6;

/// Whether `dir` holds `.http` or `.rest` files, here or in a subfolder.
pub fn is_http_collection(dir: &Path) -> bool {
    !request_files(dir).is_empty()
}

/// What a folder of `.http` files holds.
pub struct HttpCollection {
    pub toml: CollectionToml,
    /// Keyed by group name, which is the file's path without its extension.
    pub groups: HashMap<String, GroupInfo>,
}

pub fn load(dir: &Path) -> Result<HttpCollection> {
    let name = dir
        .file_name()
        .map(|name| name.to_string_lossy().to_string())
        .unwrap_or_else(|| dir.display().to_string());

    let mut groups = HashMap::new();
    for file in request_files(dir) {
        let relative = file.strip_prefix(dir).unwrap_or(&file);
        let relative_path = relative.to_string_lossy().replace('\\', "/");
        let group_name = relative
            .with_extension("")
            .to_string_lossy()
            .replace('\\', "/");
        let source = fs::read_to_string(&file)
            .with_context(|| format!("Failed to read {}", file.display()))?;
        let requests = requests_from_file(&source, &relative_path, file.parent().unwrap_or(dir));
        groups.insert(
            group_name.clone(),
            GroupInfo {
                name: group_name,
                requests,
                path: relative_path,
            },
        );
    }

    Ok(HttpCollection {
        toml: CollectionToml {
            collection: CollectionMeta {
                name,
                version: String::new(),
                collection_type: "collection".to_string(),
                docs: None,
                ignore: Vec::new(),
                auth: None,
                vars: Vec::new(),
            },
            environments: environments(dir),
        },
        groups,
    })
}

/// The `.http` and `.rest` files under `dir`, sorted, skipping hidden and
/// build folders.
fn request_files(dir: &Path) -> Vec<PathBuf> {
    fn walk(dir: &Path, depth: usize, files: &mut Vec<PathBuf>) {
        let Ok(entries) = fs::read_dir(dir) else {
            return;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if name.starts_with('.') {
                continue;
            }
            if path.is_dir() {
                if depth < MAX_DEPTH && !SKIPPED_DIRS.contains(&name.as_str()) {
                    walk(&path, depth + 1, files);
                }
            } else if matches!(
                path.extension().and_then(|ext| ext.to_str()),
                Some("http" | "rest")
            ) {
                files.push(path);
            }
        }
    }
    let mut files = Vec::new();
    walk(dir, 0, &mut files);
    files.sort();
    files
}

/// The requests of one file, keyed by `<file>#<n>`. `base_dir` is where
/// `< file` bodies are read from.
fn requests_from_file(
    source: &str,
    relative_path: &str,
    base_dir: &Path,
) -> HashMap<String, RequestData> {
    let file = http_file::parse(source);
    for diagnostic in &file.diagnostics {
        tracing::warn!("{relative_path}: {}", diagnostic.message);
    }

    // In VS Code, file variables apply to the whole file.
    let vars: Vec<KeyValuePair> = file
        .variables()
        .map(|variable| KeyValuePair {
            key: variable.name.text.clone(),
            value: variable.value.text.clone(),
            enabled: true,
        })
        .collect();

    let mut requests = HashMap::new();
    let mut taken: Vec<String> = Vec::new();
    for (index, block) in file.blocks.iter().enumerate() {
        let Some(request) = &block.request else {
            continue;
        };
        let Some(method) = method(request.method()) else {
            tracing::warn!(
                "{relative_path}: skipping {} request, the method is not supported",
                request.method()
            );
            continue;
        };

        let base_name = request
            .name()
            .map(str::to_string)
            .or_else(|| block.title.as_ref().map(|title| title.text.clone()))
            .unwrap_or_else(|| format!("{} {}", request.method(), request.url()));
        let name = unique_name(&base_name, &taken);
        taken.push(name.clone());

        let headers = request
            .headers
            .iter()
            .map(|header| KeyValuePair {
                key: header.name.text.clone(),
                value: header.full_value(),
                enabled: true,
            })
            .collect();

        let body = request
            .body
            .as_ref()
            .map_or_else(String::new, |body| match body.file_ref() {
                Some(path) => read_body_file(&base_dir.join(path))
                    .unwrap_or_else(|| body.content.text.clone()),
                None => body.content.text.clone(),
            });

        // The editor keeps the query in `query_params`, not in the URL.
        let (url, query_params) = crate::domain::split_query(&request.url());
        requests.insert(
            format!("{relative_path}#{index}"),
            RequestData {
                name,
                method,
                url,
                query_params,
                headers,
                body,
                vars: vars.clone(),
                ..Default::default()
            },
        );
    }
    requests
}

fn method(method: &str) -> Option<HttpMethod> {
    HttpMethod::ALL
        .iter()
        .find(|known| known.as_str().eq_ignore_ascii_case(method))
        .copied()
}

/// `base`, or `base (2)`, `base (3)` and so on when it is taken.
fn unique_name(base: &str, taken: &[String]) -> String {
    if !taken.iter().any(|name| name == base) {
        return base.to_string();
    }
    (2..)
        .map(|n| format!("{base} ({n})"))
        .find(|name| !taken.contains(name))
        .unwrap_or_default()
}

/// A body file as text. Binary files are not read.
fn read_body_file(path: &Path) -> Option<String> {
    match fs::read(path) {
        Ok(bytes) => String::from_utf8(bytes).ok(),
        Err(error) => {
            tracing::warn!("Could not read body file {}: {}", path.display(), error);
            None
        }
    }
}

/// Environments from the JetBrains env files and the VS Code settings.
fn environments(dir: &Path) -> Vec<EnvironmentToml> {
    let mut envs: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    let mut merge = |source: Option<serde_json::Value>| {
        let Some(serde_json::Value::Object(source)) = source else {
            return;
        };
        for (env_name, vars) in source {
            let serde_json::Value::Object(vars) = vars else {
                continue;
            };
            let env = envs.entry(env_name).or_default();
            for (key, value) in vars {
                env.insert(key, json_to_string(&value));
            }
        }
    };

    merge(read_json(&dir.join("http-client.env.json")));
    merge(read_json(&dir.join("http-client.private.env.json")));
    let vscode = read_json(&dir.join(".vscode/settings.json"))
        .and_then(|settings| settings.get("rest-client.environmentVariables").cloned());
    merge(vscode);

    let shared = envs.remove("$shared").unwrap_or_default();
    envs.into_iter()
        .map(|(name, vars)| {
            let mut variables: HashMap<String, EnvironmentVariable> = shared
                .iter()
                .map(|(key, value)| (key.clone(), plain_variable(value)))
                .collect();
            for (key, value) in vars {
                variables.insert(key, plain_variable(&value));
            }
            EnvironmentToml { name, variables }
        })
        .collect()
}

/// A variable held in memory. Values from the private env file are not
/// marked secret, since secrets are read from the OS keychain instead.
fn plain_variable(value: &str) -> EnvironmentVariable {
    EnvironmentVariable {
        value: value.to_string(),
        secret: false,
        temporary: false,
    }
}

fn json_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

/// Read a JSON file that may hold comments and trailing commas, as VS Code
/// settings do.
fn read_json(path: &Path) -> Option<serde_json::Value> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&strip_jsonc(&text))
        .map_err(|error| tracing::warn!("Could not read {}: {}", path.display(), error))
        .ok()
}

/// Remove `//` and `/* */` comments and trailing commas, outside strings.
fn strip_jsonc(text: &str) -> String {
    remove_trailing_commas(&remove_comments(text))
}

/// Calls `f` for each character with whether it sits inside a string.
fn scan(text: &str, mut f: impl FnMut(char, bool, &mut std::iter::Peekable<std::str::Chars>)) {
    let mut chars = text.chars().peekable();
    let mut in_string = false;
    let mut escaped = false;
    while let Some(c) = chars.next() {
        let was_in_string = in_string;
        if in_string {
            if escaped {
                escaped = false;
            } else if c == '\\' {
                escaped = true;
            } else if c == '"' {
                in_string = false;
            }
        } else if c == '"' {
            in_string = true;
        }
        f(c, was_in_string || c == '"', &mut chars);
    }
}

fn remove_comments(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    scan(text, |c, in_string, chars| {
        if in_string || c != '/' {
            out.push(c);
            return;
        }
        match chars.peek() {
            Some('/') => {
                for c in chars.by_ref() {
                    if c == '\n' {
                        out.push('\n');
                        break;
                    }
                }
            }
            Some('*') => {
                chars.next();
                let mut previous = ' ';
                for c in chars.by_ref() {
                    if previous == '*' && c == '/' {
                        break;
                    }
                    previous = c;
                }
            }
            _ => out.push(c),
        }
    });
    out
}

fn remove_trailing_commas(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    scan(text, |c, in_string, chars| {
        if !in_string && c == ',' {
            let next = chars.clone().find(|c| !c.is_whitespace());
            if matches!(next, Some('}' | ']')) {
                return;
            }
        }
        out.push(c);
    });
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp_dir() -> PathBuf {
        let dir = std::env::temp_dir().join(format!(
            "broquest-http-files-{}-{}",
            std::process::id(),
            fastrand::u64(..)
        ));
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    fn write(dir: &Path, path: &str, text: &str) {
        let path = dir.join(path);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(path, text).unwrap();
    }

    #[test]
    fn loads_files_as_groups() {
        let dir = temp_dir();
        write(
            &dir,
            "auth.http",
            "@user = kestrel\n\n### Log in\nPOST {{host}}/login\nContent-Type: application/json\n\n< ./login.json\n\n###\nGET {{host}}/me\n\n###\n# @name Me again\nGET {{host}}/me\n###\nPROPFIND {{host}}/files\n",
        );
        write(&dir, "login.json", "{\"user\": \"{{user}}\"}");
        write(
            &dir,
            "shipments/list.rest",
            "GET {{host}}/shipments?limit=4\n",
        );
        write(&dir, "node_modules/pkg/ignored.http", "GET /nope\n");
        write(&dir, ".hidden/ignored.http", "GET /nope\n");

        assert!(is_http_collection(&dir));
        let collection = load(&dir).unwrap();
        let mut names: Vec<&str> = collection.groups.keys().map(String::as_str).collect();
        names.sort();
        assert_eq!(names, ["auth", "shipments/list"]);

        let auth = &collection.groups["auth"];
        assert_eq!(auth.path, "auth.http");
        let mut requests: Vec<&RequestData> = auth.requests.values().collect();
        requests.sort_by(|a, b| a.name.cmp(&b.name));
        let summary: Vec<(&str, HttpMethod, &str)> = requests
            .iter()
            .map(|r| (r.name.as_str(), r.method, r.url.as_str()))
            .collect();
        assert_eq!(
            summary,
            [
                ("GET {{host}}/me", HttpMethod::Get, "{{host}}/me"),
                ("Log in", HttpMethod::Post, "{{host}}/login"),
                ("Me again", HttpMethod::Get, "{{host}}/me"),
            ]
        );
        let login = requests[1];
        assert_eq!(login.body, "{\"user\": \"{{user}}\"}");
        assert_eq!(login.headers[0].key, "Content-Type");
        assert_eq!(login.vars[0].key, "user");
        assert_eq!(login.vars[0].value, "kestrel");

        let list = collection.groups["shipments/list"]
            .requests
            .values()
            .next()
            .unwrap();
        assert_eq!(list.url, "{{host}}/shipments");
        assert_eq!(list.query_params.len(), 1);
        assert_eq!(list.query_params[0].key, "limit");
        assert_eq!(list.query_params[0].value, "4");

        fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn sends_a_request_with_file_environment_and_built_in_variables() {
        use wiremock::matchers::{body_string, header, method, path, query_param};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/token"))
            .and(query_param("client", "kestrel"))
            .and(header("content-type", "application/json"))
            .and(body_string("{\"client\": \"kestrel\"}"))
            .respond_with(ResponseTemplate::new(200).set_body_string("ok"))
            .mount(&server)
            .await;

        let dir = temp_dir();
        write(
            &dir,
            "http-client.env.json",
            &format!(r#"{{"local": {{"baseUrl": "{}"}}}}"#, server.uri()),
        );
        write(
            &dir,
            "auth.http",
            "@client = kestrel\n\n### Token\nPOST {{baseUrl}}/token?client={{client}}\nContent-Type: application/json\nX-Request-Id: {{$guid}}\n\n{\"client\": \"{{client}}\"}\n",
        );

        let collection = load(&dir).unwrap();
        let request = collection.groups["auth"]
            .requests
            .values()
            .next()
            .unwrap()
            .clone();
        let variables: HashMap<String, String> = collection.toml.environments[0]
            .variables
            .iter()
            .map(|(key, variable)| (key.clone(), variable.value.clone()))
            .collect();

        let client = crate::http::HttpClientService::new(30).unwrap();
        let (response, _) = client
            .send_request(request, Some(variables), Some(HashMap::new()), None, None)
            .await
            .unwrap_or_else(|error| panic!("{}", error.details));
        assert_eq!(response.status_code, Some(200));

        let received = server.received_requests().await.unwrap();
        let request_id = received[0].headers.get("x-request-id").unwrap();
        assert!(uuid::Uuid::parse_str(request_id.to_str().unwrap()).is_ok());

        fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn chained_requests_use_the_last_response() {
        use crate::environments::references;
        use wiremock::matchers::{header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/login"))
            .respond_with(ResponseTemplate::new(200).set_body_string(r#"{"token": "abc"}"#))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/me"))
            .and(header("authorization", "Bearer abc"))
            .respond_with(ResponseTemplate::new(200).set_body_string("me"))
            .mount(&server)
            .await;

        let source = format!(
            "@token = {{{{login.response.body.$.token}}}}\n\n# @name login\nPOST {uri}/login\n\n###\n# @name me\nGET {uri}/me\nAuthorization: Bearer {{{{token}}}}\n",
            uri = server.uri()
        );
        let requests = requests_from_file(&source, "auth.http", Path::new("."));
        let request = |name: &str| {
            requests
                .values()
                .find(|request| request.name == name)
                .unwrap()
                .clone()
        };
        let client = crate::http::HttpClientService::new(30).unwrap();

        let (login, _) = client
            .send_request(request("login"), None, None, None, None)
            .await
            .unwrap();
        let exchanges = HashMap::from([("login".to_string(), login)]);

        // What the editor does before sending: resolve the references the
        // request's variables make, and pass them on as collection variables.
        let me = request("me");
        let resolved: HashMap<String, String> = me
            .vars
            .iter()
            .flat_map(|var| references::references_in(&var.value))
            .filter_map(|name| Some((name.clone(), references::evaluate(&name, &exchanges)?)))
            .collect();
        assert_eq!(resolved.len(), 1);

        let (response, _) = client
            .send_request(me, None, None, None, Some(resolved))
            .await
            .unwrap();
        assert_eq!(response.status_code, Some(200));
        assert_eq!(response.body, "me");
    }

    #[test]
    fn duplicate_names_get_numbers() {
        let requests = requests_from_file(
            "### Ping\nGET /a\n### Ping\nGET /b\n### Ping\nGET /c\n",
            "ping.http",
            Path::new("."),
        );
        let mut names: Vec<&str> = requests.values().map(|r| r.name.as_str()).collect();
        names.sort();
        assert_eq!(names, ["Ping", "Ping (2)", "Ping (3)"]);
    }

    #[test]
    fn environments_merge_shared_private_and_vscode() {
        let dir = temp_dir();
        write(
            &dir,
            "http-client.env.json",
            r#"{"$shared": {"version": "v1", "host": "shared"}, "dev": {"host": "localhost", "port": 8080}}"#,
        );
        write(
            &dir,
            "http-client.private.env.json",
            r#"{"dev": {"token": "secret-dev"}}"#,
        );
        write(
            &dir,
            ".vscode/settings.json",
            r#"{
                // VS Code settings allow comments
                "editor.tabSize": 2,
                "rest-client.environmentVariables": {
                    "staging": { "host": "staging.example.com", }, /* and trailing commas */
                },
            }"#,
        );

        let envs = environments(&dir);
        let names: Vec<&str> = envs.iter().map(|env| env.name.as_str()).collect();
        assert_eq!(names, ["dev", "staging"]);
        let value = |env: usize, key: &str| envs[env].variables[key].value.clone();
        assert_eq!(value(0, "host"), "localhost");
        assert_eq!(value(0, "port"), "8080");
        assert_eq!(value(0, "token"), "secret-dev");
        assert_eq!(value(0, "version"), "v1");
        assert_eq!(value(1, "host"), "staging.example.com");
        assert_eq!(value(1, "version"), "v1");

        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn strips_comments_outside_strings() {
        let text = r#"{"url": "http://a//b", /* c */ "list": [1, 2,], // d
"x": "/* kept */"}"#;
        let value: serde_json::Value = serde_json::from_str(&strip_jsonc(text)).unwrap();
        assert_eq!(value["url"], "http://a//b");
        assert_eq!(value["list"], serde_json::json!([1, 2]));
        assert_eq!(value["x"], "/* kept */");
    }

    #[test]
    fn folders_without_request_files_are_not_http_collections() {
        let dir = temp_dir();
        write(&dir, "collection.toml", "");
        assert!(!is_http_collection(&dir));
        fs::remove_dir_all(dir).ok();
    }
}
