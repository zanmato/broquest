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
use crate::domain::{
    HttpFileOptions, HttpMethod, KeyValuePair, Prompt, RequestData, ScriptDialect,
};

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

        // `< file` lines stay as written and are read when the request is sent.
        let body = request
            .body
            .as_ref()
            .map_or_else(String::new, |body| body.content.text.clone());

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
                pre_request_script: script_text(&request.pre_request_scripts, base_dir),
                post_response_script: script_text(&request.response_handlers, base_dir),
                script_dialect: ScriptDialect::JetBrains,
                http_file: Some(options(request, base_dir, relative_path)),
                ..Default::default()
            },
        );
    }
    requests
}

/// The request's `.http` options, from its directives.
fn options(request: &http_file::Request, base_dir: &Path, relative_path: &str) -> HttpFileOptions {
    let mut options = HttpFileOptions {
        dir: base_dir.to_path_buf(),
        ..Default::default()
    };
    for directive in &request.directives {
        let value = directive.value.as_ref().map(|v| v.text.as_str());
        match directive.name.text.to_ascii_lowercase().as_str() {
            "no-redirect" => options.no_redirect = true,
            "no-log" => options.no_log = true,
            "timeout" => match value.and_then(parse_duration) {
                Some(timeout) => options.timeout = Some(timeout),
                None => tracing::warn!("{relative_path}: could not read @timeout {value:?}"),
            },
            "note" => options.note = Some(value.unwrap_or_default().to_string()),
            "prompt" => {
                let Some(value) = value else { continue };
                let (name, description) = match value.split_once(char::is_whitespace) {
                    Some((name, description)) => (name, Some(description.trim().to_string())),
                    None => (value, None),
                };
                options.prompts.push(Prompt {
                    name: name.to_string(),
                    description,
                });
            }
            "connection-timeout" => tracing::info!(
                "{relative_path}: @connection-timeout is not supported, the request timeout applies"
            ),
            _ => {}
        }
    }
    options
}

/// `5000ms`, `30s`, `2 m` or a number of seconds.
fn parse_duration(text: &str) -> Option<std::time::Duration> {
    let text = text.trim();
    let digits = text
        .find(|c: char| !c.is_ascii_digit())
        .unwrap_or(text.len());
    let amount: u64 = text[..digits].parse().ok()?;
    match text[digits..].trim() {
        "ms" => Some(std::time::Duration::from_millis(amount)),
        "" | "s" => Some(std::time::Duration::from_secs(amount)),
        "m" => Some(std::time::Duration::from_secs(amount.checked_mul(60)?)),
        _ => None,
    }
}

/// The code of a request's scripts, read from their files where needed.
/// Several scripts each get a block of their own.
fn script_text(scripts: &[http_file::Script], base_dir: &Path) -> Option<String> {
    let parts: Vec<String> = scripts
        .iter()
        .map(|script| match script {
            http_file::Script::Inline { code, .. } => {
                code.text.trim_matches(['\n', '\r']).to_string()
            }
            http_file::Script::File { path, .. } => {
                let file = base_dir.join(&path.text);
                fs::read_to_string(&file).unwrap_or_else(|error| {
                    tracing::warn!("Could not read script {}: {}", file.display(), error);
                    format!(
                        "throw new Error({:?});",
                        format!("Could not read the script {}", path.text)
                    )
                })
            }
        })
        .collect();
    match parts.as_slice() {
        [] => None,
        [only] => Some(only.clone()),
        many => Some(
            many.iter()
                .map(|part| format!("{{\n{part}\n}}"))
                .collect::<Vec<_>>()
                .join("\n\n"),
        ),
    }
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
        assert_eq!(login.body, "< ./login.json");
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

    #[tokio::test]
    async fn jetbrains_scripts_run_around_the_request() {
        use wiremock::matchers::{body_string, header, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/login"))
            .and(body_string("nonce=n-42"))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(r#"{"token": "abc"}"#, "application/json"),
            )
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/me"))
            .and(header("authorization", "Bearer abc"))
            .respond_with(ResponseTemplate::new(200).set_body_string("me"))
            .mount(&server)
            .await;

        let dir = temp_dir();
        write(
            &dir,
            "check.js",
            "client.test('token', function () { client.assert(response.body.token, 'no token'); });",
        );
        let source = format!(
            "### Log in\n< {{% request.variables.set(\"nonce\", \"n-42\"); %}}\nPOST {uri}/login\n\nnonce={{{{nonce}}}}\n\n> {{%\n  client.global.set(\"token\", response.body.token);\n%}}\n> ./check.js\n\n### Me\nGET {uri}/me\nAuthorization: Bearer {{{{token}}}}\n",
            uri = server.uri()
        );
        let requests = requests_from_file(&source, "auth.http", &dir);
        let request = |name: &str| {
            requests
                .values()
                .find(|request| request.name == name)
                .unwrap()
                .clone()
        };
        let login = request("Log in");
        assert_eq!(login.script_dialect, ScriptDialect::JetBrains);
        // Both handlers, the inline one and the one read from check.js.
        let handlers = login.post_response_script.as_deref().unwrap();
        assert!(handlers.contains("client.global.set") && handlers.contains("client.test"));

        let client = crate::http::HttpClientService::new(30).unwrap();
        let (response, store) = client
            .send_request(login, None, None, None, None)
            .await
            .unwrap_or_else(|error| panic!("{}", error.details));
        assert_eq!(response.status_code, Some(200), "{}", response.body);

        assert_eq!(store.get_var("token"), Some(serde_json::json!("abc")));

        // The editor carries the session variables on to the next request.
        let (response, _) = client
            .send_request(request("Me"), None, None, Some(store.get_all_vars()), None)
            .await
            .unwrap();
        assert_eq!(response.body, "me");

        fs::remove_dir_all(dir).ok();
    }

    #[tokio::test]
    async fn sends_body_files_at_send_time() {
        use wiremock::matchers::{body_string, method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/items"))
            .and(body_string("{\"name\": \"kestrel\"}"))
            .respond_with(ResponseTemplate::new(201))
            .mount(&server)
            .await;

        let dir = temp_dir();
        write(&dir, "payload.json", "{\"name\": \"kestrel\"}");
        let source = format!(
            "POST {}/items\nContent-Type: application/json\n\n< ./payload.json\n",
            server.uri()
        );
        let request = requests_from_file(&source, "items.http", &dir)
            .into_values()
            .next()
            .unwrap();
        assert_eq!(request.body, "< ./payload.json");

        let client = crate::http::HttpClientService::new(30).unwrap();
        let (response, _) = client
            .send_request(request, None, None, None, None)
            .await
            .unwrap();
        assert_eq!(response.status_code, Some(201));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn directives_become_options() {
        let requests = requests_from_file(
            "# @name Profile\n# @no-redirect\n# @no-log\n# @timeout 1500ms\n# @note\n# @prompt username\n# @prompt password Your password\n# @prompt refCode A code like RC-1\nGET /me\n###\n# @timeout 2 m\n# @note Deletes everything\nDELETE /all\n###\n# @timeout soon\nGET /x\n",
            "me.http",
            Path::new("/work"),
        );
        let request = |name: &str| {
            requests
                .values()
                .find(|r| r.name == name)
                .and_then(|r| r.http_file.clone())
                .unwrap()
        };

        let profile = request("Profile");
        assert_eq!(profile.dir, Path::new("/work"));
        assert!(profile.no_redirect && profile.no_log);
        assert_eq!(
            profile.timeout,
            Some(std::time::Duration::from_millis(1500))
        );
        assert_eq!(profile.note.as_deref(), Some(""));
        let prompts: Vec<(&str, Option<&str>, bool)> = profile
            .prompts
            .iter()
            .map(|p| (p.name.as_str(), p.description.as_deref(), p.is_secret()))
            .collect();
        assert_eq!(
            prompts,
            [
                ("username", None, false),
                ("password", Some("Your password"), true),
                ("refCode", Some("A code like RC-1"), false),
            ]
        );

        let delete = request("DELETE /all");
        assert_eq!(delete.timeout, Some(std::time::Duration::from_secs(120)));
        assert_eq!(delete.note.as_deref(), Some("Deletes everything"));
        assert!(!delete.no_redirect);

        assert_eq!(request("GET /x").timeout, None);
    }

    #[tokio::test]
    async fn no_redirect_and_timeout_apply_when_sending() {
        use wiremock::matchers::{method, path};
        use wiremock::{Mock, MockServer, ResponseTemplate};

        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/old"))
            .respond_with(ResponseTemplate::new(302).insert_header("Location", "/new"))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/new"))
            .respond_with(ResponseTemplate::new(200).set_body_string("new"))
            .mount(&server)
            .await;
        Mock::given(method("GET"))
            .and(path("/slow"))
            .respond_with(ResponseTemplate::new(200).set_delay(std::time::Duration::from_secs(5)))
            .mount(&server)
            .await;

        let source = format!(
            "### Follow\nGET {uri}/old\n###\n# @no-redirect\n### Stay\n# @no-redirect\nGET {uri}/old\n###\n# @timeout 200ms\n### Slow\n# @timeout 200ms\nGET {uri}/slow\n",
            uri = server.uri()
        );
        let requests = requests_from_file(&source, "r.http", Path::new("."));
        let request = |name: &str| requests.values().find(|r| r.name == name).unwrap().clone();
        let client = crate::http::HttpClientService::new(30).unwrap();

        let (followed, _) = client
            .send_request(request("Follow"), None, None, None, None)
            .await
            .unwrap();
        assert_eq!(followed.status_code, Some(200));

        let (stayed, _) = client
            .send_request(request("Stay"), None, None, None, None)
            .await
            .unwrap();
        assert_eq!(stayed.status_code, Some(302));

        let started = std::time::Instant::now();
        let slow = client
            .send_request(request("Slow"), None, None, None, None)
            .await;
        assert!(slow.is_err());
        assert!(started.elapsed() < std::time::Duration::from_secs(3));
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
