//! Saving changes to collections of `.http` files.
//!
//! Requests are written back into their files by changing only the parts
//! that differ from what was loaded, so comments, blank lines and the rest of
//! the file stay as they were. Every change reloads the collection from disk.

use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result, bail};
use http_file::edit::{append_request, remove_block, set_variables, update_request};
use http_file::{Change, NewRequest, RequestEdit, Script};

use super::http_files::{self, request_names};
use super::manager::CollectionInfo;
use crate::domain::{AuthType, KeyValuePair, RequestData};

/// Where requests without a file go, such as new requests at the top of the
/// collection.
pub const DEFAULT_FILE: &str = "requests.http";

fn dir(info: &CollectionInfo) -> &Path {
    Path::new(&info.data.path)
}

/// Re-read the collection's groups and environments from disk.
fn reload(info: &mut CollectionInfo) -> Result<()> {
    let loaded = http_files::load(dir(info))?;
    info.groups = loaded.groups;
    info.toml.environments = loaded.toml.environments;
    Ok(())
}

fn read(path: &Path) -> Result<String> {
    match fs::read_to_string(path) {
        Ok(text) => Ok(text),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
        Err(error) => Err(error).with_context(|| format!("Failed to read {}", path.display())),
    }
}

fn write(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, text).with_context(|| format!("Failed to write {}", path.display()))
}

/// The request as last loaded from disk.
fn loaded<'a>(info: &'a CollectionInfo, file: &str, name: &str) -> Option<&'a RequestData> {
    info.groups
        .values()
        .find(|group| group.path == file)?
        .requests
        .values()
        .find(|request| request.name == name)
}

/// The full target: the URL with its query parameters.
fn full_url(request: &RequestData) -> String {
    let query: Vec<String> = request
        .query_params
        .iter()
        .filter(|param| param.enabled && !param.key.is_empty())
        .map(|param| {
            if param.value.is_empty() {
                param.key.clone()
            } else {
                format!("{}={}", param.key, param.value)
            }
        })
        .collect();
    if query.is_empty() {
        request.url.clone()
    } else {
        format!("{}?{}", request.url, query.join("&"))
    }
}

/// The headers to write, the request's auth included as a header.
fn headers(request: &RequestData) -> Result<Vec<(String, String)>> {
    let mut headers: Vec<(String, String)> = request
        .headers
        .iter()
        .filter(|header| header.enabled && !header.key.is_empty())
        .map(|header| (header.key.clone(), header.value.clone()))
        .collect();
    let auth = match &request.auth {
        AuthType::None | AuthType::Inherit => None,
        AuthType::Basic(basic) => Some((
            "Authorization".to_string(),
            format!("Basic {}:{}", basic.username, basic.password),
        )),
        AuthType::Digest(digest) => Some((
            "Authorization".to_string(),
            format!("Digest {} {}", digest.username, digest.password),
        )),
        AuthType::Key(key) => Some((key.header.clone(), key.value.clone())),
        other => bail!(
            "{} can't be saved in a .http file. Use an Authorization header instead",
            other.name()
        ),
    };
    if let Some((name, value)) = auth
        && !headers.iter().any(|(n, _)| n.eq_ignore_ascii_case(&name))
    {
        headers.push((name, value));
    }
    Ok(headers)
}

fn enabled_vars(vars: &[KeyValuePair]) -> Vec<(String, String)> {
    vars.iter()
        .filter(|var| var.enabled && !var.key.is_empty())
        .map(|var| (var.key.clone(), var.value.clone()))
        .collect()
}

fn script(code: &Option<String>) -> Option<String> {
    code.as_ref()
        .map(|code| code.trim().to_string())
        .filter(|code| !code.is_empty())
}

/// A changed script. A script that lives in its own file is written there
/// and the `.http` file keeps its reference. Returns the edit for the
/// `.http` file, if any.
fn script_edit(
    existing: &[Script],
    old: &Option<String>,
    new: &Option<String>,
    file_dir: &Path,
) -> Result<Option<Option<String>>> {
    if script(old) == script(new) {
        return Ok(None);
    }
    if let ([Script::File { path, .. }], Some(code)) = (existing, script(new)) {
        write(&file_dir.join(&path.text), &code)?;
        return Ok(None);
    }
    Ok(Some(script(new)))
}

fn new_request(request: &RequestData, name: &str) -> Result<NewRequest> {
    Ok(NewRequest {
        name: Some(name.to_string()),
        method: request.method.as_str().to_string(),
        url: full_url(request),
        headers: headers(request)?,
        body: request.body.clone(),
        pre_request_script: script(&request.pre_request_script),
        response_handler: script(&request.post_response_script),
    })
}

/// Add any of the request's variables the file does not have yet.
fn merged_vars(file: &http_file::HttpFile, request: &RequestData) -> Vec<(String, String)> {
    let mut vars: Vec<(String, String)> = Vec::new();
    for variable in file.variables() {
        if !vars.iter().any(|(name, _)| *name == variable.name.text) {
            vars.push((variable.name.text.clone(), variable.value.text.clone()));
        }
    }
    for (name, value) in enabled_vars(&request.vars) {
        if !vars.iter().any(|(n, _)| *n == name) {
            vars.push((name, value));
        }
    }
    vars
}

pub fn save_request(
    info: &mut CollectionInfo,
    request: &RequestData,
    name: &str,
    group_path: Option<&str>,
) -> Result<()> {
    let source = request
        .http_file
        .as_ref()
        .and_then(|options| options.source.clone());
    let file_name = source
        .as_ref()
        .map(|source| source.file.clone())
        .or_else(|| group_path.map(str::to_string))
        .unwrap_or_else(|| DEFAULT_FILE.to_string());
    let path = dir(info).join(&file_name);
    let text = read(&path)?;
    let file = http_file::parse(&text);
    let names = request_names(&file);

    let block = source.as_ref().and_then(|source| {
        names
            .iter()
            .position(|n| n.as_deref() == Some(source.name.as_str()))
    });
    if names
        .iter()
        .enumerate()
        .any(|(index, n)| n.as_deref() == Some(name) && Some(index) != block)
    {
        bail!("A request named \"{name}\" already exists in {file_name}");
    }

    let mut changes: Vec<Change> = Vec::new();
    match (
        block,
        source
            .as_ref()
            .and_then(|s| loaded(info, &file_name, &s.name)),
    ) {
        (Some(block), Some(old)) => {
            let existing = file.blocks[block].request.as_ref();
            let file_dir = path.parent().unwrap_or(dir(info));
            let new_headers = headers(request)?;
            let edit = RequestEdit {
                name: (name != source.as_ref().map_or("", |s| s.name.as_str()))
                    .then(|| name.to_string()),
                method: (request.method != old.method).then(|| request.method.as_str().to_string()),
                url: (request.url != old.url || request.query_params != old.query_params)
                    .then(|| full_url(request)),
                headers: (new_headers != headers(old)?).then_some(new_headers),
                body: (request.body != old.body).then(|| request.body.clone()),
                pre_request_script: script_edit(
                    existing.map_or(&[][..], |r| &r.pre_request_scripts),
                    &old.pre_request_script,
                    &request.pre_request_script,
                    file_dir,
                )?,
                response_handler: script_edit(
                    existing.map_or(&[][..], |r| &r.response_handlers),
                    &old.post_response_script,
                    &request.post_response_script,
                    file_dir,
                )?,
            };
            changes.extend(update_request(&file, block, &edit));
            if enabled_vars(&request.vars) != enabled_vars(&old.vars) {
                changes.extend(set_variables(&file, &enabled_vars(&request.vars)));
            }
        }
        _ => {
            changes.extend(append_request(&file, &new_request(request, name)?));
            let vars = merged_vars(&file, request);
            changes.extend(set_variables(&file, &vars));
        }
    }

    if !changes.is_empty() {
        write(&path, &http_file::apply(&text, &changes))?;
    }
    reload(info)
}

/// The request's file and block, from where it was loaded.
fn locate(info: &CollectionInfo, request: &RequestData) -> Result<(String, String, usize)> {
    let Some(source) = request
        .http_file
        .as_ref()
        .and_then(|options| options.source.clone())
    else {
        bail!("This request has not been saved yet");
    };
    let path = dir(info).join(&source.file);
    let text = read(&path)?;
    let file = http_file::parse(&text);
    let block = request_names(&file)
        .iter()
        .position(|n| n.as_deref() == Some(source.name.as_str()))
        .with_context(|| format!("\"{}\" is no longer in {}", source.name, source.file))?;
    Ok((source.file, text, block))
}

pub fn delete_request(info: &mut CollectionInfo, request: &RequestData) -> Result<()> {
    let (file_name, text, block) = locate(info, request)?;
    let file = http_file::parse(&text);
    write(
        &dir(info).join(&file_name),
        &http_file::apply(&text, &remove_block(&file, block)),
    )?;
    reload(info)
}

pub fn move_request(
    info: &mut CollectionInfo,
    request: &RequestData,
    target_group_path: Option<&str>,
) -> Result<()> {
    let target = target_group_path.unwrap_or(DEFAULT_FILE).to_string();
    let (file_name, text, block) = locate(info, request)?;
    if file_name == target {
        return Ok(());
    }

    let target_path = dir(info).join(&target);
    let target_text = read(&target_path)?;
    let target_file = http_file::parse(&target_text);
    if request_names(&target_file)
        .iter()
        .any(|n| n.as_deref() == Some(request.name.as_str()))
    {
        bail!(
            "A request named \"{}\" already exists in {target}",
            request.name
        );
    }
    let mut changes = append_request(&target_file, &new_request(request, &request.name)?);
    changes.extend(set_variables(
        &target_file,
        &merged_vars(&target_file, request),
    ));
    write(&target_path, &http_file::apply(&target_text, &changes))?;

    let file = http_file::parse(&text);
    write(
        &dir(info).join(&file_name),
        &http_file::apply(&text, &remove_block(&file, block)),
    )?;
    reload(info)
}

pub fn create_group(info: &mut CollectionInfo, name: &str) -> Result<()> {
    let path = dir(info).join(format!("{name}.http"));
    if path.exists() {
        bail!("{} already exists", path.display());
    }
    write(&path, "")?;
    reload(info)
}

fn group_file(info: &CollectionInfo, name: &str) -> Result<std::path::PathBuf> {
    let group = info
        .groups
        .get(name)
        .with_context(|| format!("There is no file for {name}"))?;
    Ok(dir(info).join(&group.path))
}

pub fn rename_group(info: &mut CollectionInfo, old: &str, new: &str) -> Result<()> {
    let old_path = group_file(info, old)?;
    let extension = old_path
        .extension()
        .map_or("http".to_string(), |e| e.to_string_lossy().to_string());
    let new_path = dir(info).join(format!("{new}.{extension}"));
    if new_path.exists() {
        bail!("{} already exists", new_path.display());
    }
    if let Some(parent) = new_path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::rename(&old_path, &new_path)
        .with_context(|| format!("Failed to rename {}", old_path.display()))?;
    reload(info)
}

pub fn delete_group(info: &mut CollectionInfo, name: &str) -> Result<()> {
    let path = group_file(info, name)?;
    fs::remove_file(&path).with_context(|| format!("Failed to delete {}", path.display()))?;
    reload(info)
}

type JsonMap = serde_json::Map<String, serde_json::Value>;

fn read_object(path: &Path) -> Result<JsonMap> {
    let text = read(path)?;
    if text.trim().is_empty() {
        return Ok(JsonMap::new());
    }
    match serde_json::from_str(&http_files::strip_jsonc(&text)) {
        Ok(serde_json::Value::Object(map)) => Ok(map),
        _ => bail!("{} is not a JSON object", path.display()),
    }
}

fn object<'a>(map: &'a JsonMap, key: &str) -> Option<&'a JsonMap> {
    map.get(key).and_then(|value| value.as_object())
}

/// Write the environments to `http-client.env.json`, keeping values that
/// came from `http-client.private.env.json` there, and leaving values that
/// only repeat `$shared` to `$shared`.
pub fn save_environments(info: &CollectionInfo) -> Result<()> {
    let public_path = dir(info).join("http-client.env.json");
    let private_path = dir(info).join("http-client.private.env.json");
    if !public_path.exists()
        && !private_path.exists()
        && !info.toml.environments.is_empty()
        && http_files::has_vscode_environments(dir(info))
    {
        bail!("Environments from .vscode/settings.json can't be saved yet");
    }

    let public = read_object(&public_path)?;
    let private = read_object(&private_path)?;
    let empty = JsonMap::new();
    let shared_public = object(&public, "$shared").unwrap_or(&empty);
    let shared_private = object(&private, "$shared").unwrap_or(&empty);

    let mut new_public = JsonMap::new();
    let mut new_private = JsonMap::new();
    if let Some(shared) = public.get("$shared") {
        new_public.insert("$shared".to_string(), shared.clone());
    }
    if let Some(shared) = private.get("$shared") {
        new_private.insert("$shared".to_string(), shared.clone());
    }

    // Environments in their existing order, new ones after.
    let mut environments: Vec<_> = info.toml.environments.iter().collect();
    let position = |name: &str| public.keys().chain(private.keys()).position(|k| k == name);
    environments.sort_by_key(|env| position(&env.name).unwrap_or(usize::MAX));

    for env in environments {
        let old_public = object(&public, &env.name).unwrap_or(&empty);
        let old_private = object(&private, &env.name).unwrap_or(&empty);
        let mut keys: Vec<&String> = env.variables.keys().collect();
        let key_position = |key: &str| {
            old_public
                .keys()
                .chain(old_private.keys())
                .position(|k| k == key)
        };
        keys.sort_by(|a, b| {
            key_position(a)
                .unwrap_or(usize::MAX)
                .cmp(&key_position(b).unwrap_or(usize::MAX))
                .then_with(|| a.cmp(b))
        });

        let mut public_env = JsonMap::new();
        let mut private_env = JsonMap::new();
        for key in keys {
            let value = &env.variables[key].value;
            let is_private = old_private.contains_key(key.as_str())
                || (!old_public.contains_key(key.as_str())
                    && shared_private.contains_key(key.as_str()));
            let (old_env, shared) = if is_private {
                (old_private, shared_private)
            } else {
                (old_public, shared_public)
            };
            // A value only inherited from `$shared` stays there.
            if !old_env.contains_key(key.as_str())
                && shared
                    .get(key.as_str())
                    .map(http_files::json_to_string)
                    .as_ref()
                    == Some(value)
            {
                continue;
            }
            // Keep a number or boolean a number or boolean.
            let json = match old_env.get(key.as_str()) {
                Some(old) if http_files::json_to_string(old) == *value => old.clone(),
                _ => serde_json::Value::String(value.clone()),
            };
            if is_private {
                private_env.insert(key.clone(), json);
            } else {
                public_env.insert(key.clone(), json);
            }
        }
        if !private_env.is_empty() || private.contains_key(&env.name) {
            new_private.insert(env.name.clone(), serde_json::Value::Object(private_env));
        }
        new_public.insert(env.name.clone(), serde_json::Value::Object(public_env));
    }

    let pretty = |map: JsonMap| -> Result<String> {
        Ok(serde_json::to_string_pretty(&serde_json::Value::Object(map))? + "\n")
    };
    if new_public != public {
        write(&public_path, &pretty(new_public)?)?;
    }
    if new_private != private {
        write(&private_path, &pretty(new_private)?)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app_database::CollectionData;
    use crate::collections::CollectionManager;
    use crate::domain::{BasicAuth, HttpMethod};

    const AUTH: &str = "\
@host = https://api.example.com

### Log in
# keeps this comment
POST {{host}}/login
Content-Type: application/json

{\"user\": \"kestrel\"}

> ./check.js

###
# @name me
GET {{host}}/me
Accept: application/json
";

    fn collection(files: &[(&str, &str)]) -> (std::path::PathBuf, CollectionInfo) {
        let dir = std::env::temp_dir().join(format!(
            "broquest-http-save-{}-{}",
            std::process::id(),
            fastrand::u64(..)
        ));
        for (name, text) in files {
            write(&dir.join(name), text).unwrap();
        }
        let data = CollectionData {
            id: None,
            name: String::new(),
            path: dir.to_string_lossy().to_string(),
            position: 0,
            format: "http".to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let info = CollectionManager::build_http_info(&dir, data).unwrap();
        (dir, info)
    }

    fn request(info: &CollectionInfo, group: &str, name: &str) -> RequestData {
        info.groups[group]
            .requests
            .values()
            .find(|r| r.name == name)
            .cloned()
            .unwrap_or_else(|| panic!("no {name} in {group}"))
    }

    fn file(dir: &Path, name: &str) -> String {
        fs::read_to_string(dir.join(name)).unwrap()
    }

    fn save(info: &mut CollectionInfo, request: &RequestData) -> Result<()> {
        save_request(info, request, &request.name.clone(), None)
    }

    #[test]
    fn editing_changes_only_what_changed() {
        let (dir, mut info) = collection(&[("auth.http", AUTH), ("check.js", "// check")]);

        let mut login = request(&info, "auth", "Log in");
        login.headers[0].value = "application/xml".to_string();
        save(&mut info, &login).unwrap();
        assert_eq!(
            file(&dir, "auth.http"),
            AUTH.replace(
                "Content-Type: application/json",
                "Content-Type: application/xml"
            )
        );

        // Saving again without changes leaves the file alone.
        let login = request(&info, "auth", "Log in");
        save(&mut info, &login).unwrap();
        assert!(file(&dir, "auth.http").contains("application/xml"));

        // Query parameters and the body.
        let mut me = request(&info, "auth", "me");
        me.query_params.push(KeyValuePair {
            key: "fields".to_string(),
            value: "id,name".to_string(),
            enabled: true,
        });
        me.method = HttpMethod::Post;
        me.body = "{}".to_string();
        save(&mut info, &me).unwrap();
        let text = file(&dir, "auth.http");
        assert!(text.ends_with(
            "# @name me\nPOST {{host}}/me?fields=id,name\nAccept: application/json\n\n{}\n"
        ));
        assert!(text.contains("# keeps this comment"));
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn renaming_and_name_clashes() {
        let (dir, mut info) = collection(&[("auth.http", AUTH)]);
        let mut login = request(&info, "auth", "Log in");
        save_request(&mut info, &login, "Sign in", None).unwrap();
        assert!(file(&dir, "auth.http").contains("### Sign in\n"));
        assert_eq!(request(&info, "auth", "Sign in").method, HttpMethod::Post);

        let mut me = request(&info, "auth", "me");
        let error = save_request(&mut info, &me, "Sign in", None).unwrap_err();
        assert!(error.to_string().contains("already exists"));
        me.name = "profile".to_string();
        save(&mut info, &me).unwrap();
        assert!(file(&dir, "auth.http").contains("# @name profile\n"));

        // The old identity no longer points anywhere after a rename.
        login.body = "changed".to_string();
        assert!(delete_request(&mut info, &login).is_err());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn new_requests_and_duplicates_are_appended() {
        let (dir, mut info) = collection(&[("auth.http", AUTH)]);
        let new = RequestData {
            name: "Log out".to_string(),
            method: HttpMethod::Post,
            url: "{{host}}/logout".to_string(),
            auth: AuthType::Basic(BasicAuth {
                username: "kestrel".to_string(),
                password: "secret".to_string(),
            }),
            ..Default::default()
        };
        save_request(&mut info, &new, "Log out", Some("auth.http")).unwrap();
        assert!(file(&dir, "auth.http").ends_with(
            "\n### Log out\nPOST {{host}}/logout\nAuthorization: Basic kestrel:secret\n"
        ));

        // Saving it again with the identity it now has updates it in place.
        let mut saved = request(&info, "auth", "Log out");
        saved.url = "{{host}}/v2/logout".to_string();
        save(&mut info, &saved).unwrap();
        let text = file(&dir, "auth.http");
        assert_eq!(text.matches("### Log out").count(), 1);
        assert!(text.contains("POST {{host}}/v2/logout"));

        // A duplicate has no identity and becomes a new block.
        let mut duplicate = request(&info, "auth", "me");
        duplicate.http_file.as_mut().unwrap().source = None;
        duplicate.name = "me (copy)".to_string();
        save_request(&mut info, &duplicate, "me (copy)", Some("auth.http")).unwrap();
        assert!(file(&dir, "auth.http").contains("### me (copy)\nGET {{host}}/me\n"));
        assert_eq!(info.groups["auth"].requests.len(), 4);

        // OAuth2 has no .http form.
        let mut oauth = new.clone();
        oauth.auth = AuthType::OAuth2(Default::default());
        assert!(save_request(&mut info, &oauth, "OAuth", Some("auth.http")).is_err());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn file_variables_scripts_and_script_files() {
        let (dir, mut info) = collection(&[("auth.http", AUTH), ("check.js", "// check")]);
        let mut login = request(&info, "auth", "Log in");
        login.vars[0].value = "http://localhost:4010".to_string();
        login.vars.push(KeyValuePair {
            key: "version".to_string(),
            value: "v1".to_string(),
            enabled: true,
        });
        login.post_response_script = Some("client.log(response.status);".to_string());
        login.pre_request_script = Some("request.variables.set('n', 1);".to_string());
        save(&mut info, &login).unwrap();

        let text = file(&dir, "auth.http");
        assert!(text.starts_with("@host = http://localhost:4010\n@version = v1\n\n### Log in"));
        // The handler from check.js is written to check.js.
        assert!(text.contains("> ./check.js"));
        assert_eq!(file(&dir, "check.js"), "client.log(response.status);");
        // The new pre-request script is inline.
        assert!(text.contains("< {%\nrequest.variables.set('n', 1);\n%}\nPOST {{host}}/login"));
        // Every request of the file now has the new variables.
        assert_eq!(request(&info, "auth", "me").vars.len(), 2);
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn delete_and_move_requests() {
        let (dir, mut info) = collection(&[("auth.http", AUTH), ("users.http", "GET /users\n")]);
        let me = request(&info, "auth", "me");
        move_request(&mut info, &me, Some("users.http")).unwrap();
        assert!(!file(&dir, "auth.http").contains("@name me"));
        let users = file(&dir, "users.http");
        assert!(users.starts_with("@host = https://api.example.com\n\nGET /users\n"));
        assert!(users.contains("### me\nGET {{host}}/me\nAccept: application/json\n"));

        let login = request(&info, "auth", "Log in");
        delete_request(&mut info, &login).unwrap();
        assert!(!file(&dir, "auth.http").contains("Log in"));
        assert!(file(&dir, "auth.http").starts_with("@host = https://api.example.com\n"));
        assert!(info.groups["auth"].requests.is_empty());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn groups_are_files() {
        let (dir, mut info) = collection(&[("auth.http", AUTH)]);
        create_group(&mut info, "admin/users").unwrap();
        assert!(dir.join("admin/users.http").exists());
        assert!(info.groups.contains_key("admin/users"));
        assert!(create_group(&mut info, "auth").is_err());

        rename_group(&mut info, "auth", "login").unwrap();
        assert!(!dir.join("auth.http").exists());
        assert_eq!(file(&dir, "login.http"), AUTH);
        assert!(info.groups.contains_key("login"));

        delete_group(&mut info, "admin/users").unwrap();
        assert!(!dir.join("admin/users.http").exists());
        fs::remove_dir_all(dir).ok();
    }

    #[test]
    fn environments_keep_their_files() {
        let public = "{\n  \"$shared\": {\n    \"version\": \"v1\"\n  },\n  \"dev\": {\n    \"host\": \"localhost\",\n    \"port\": 8080\n  }\n}\n";
        let private = "{\n  \"dev\": {\n    \"token\": \"secret\"\n  }\n}\n";
        let (dir, mut info) = collection(&[
            ("auth.http", AUTH),
            ("http-client.env.json", public),
            ("http-client.private.env.json", private),
        ]);

        // Saving unchanged environments changes nothing.
        save_environments(&info).unwrap();
        assert_eq!(file(&dir, "http-client.env.json"), public);
        assert_eq!(file(&dir, "http-client.private.env.json"), private);

        let dev = &mut info.toml.environments[0];
        dev.variables.get_mut("host").unwrap().value = "127.0.0.1".to_string();
        dev.variables.get_mut("token").unwrap().value = "rotated".to_string();
        let mut staging = dev.clone();
        staging.name = "staging".to_string();
        info.toml.environments.push(staging);
        save_environments(&info).unwrap();

        let public: serde_json::Value =
            serde_json::from_str(&file(&dir, "http-client.env.json")).unwrap();
        let private: serde_json::Value =
            serde_json::from_str(&file(&dir, "http-client.private.env.json")).unwrap();
        assert_eq!(public["dev"]["host"], "127.0.0.1");
        assert_eq!(public["dev"]["port"], 8080);
        assert!(public["dev"].get("version").is_none());
        assert!(public["dev"].get("token").is_none());
        assert_eq!(private["dev"]["token"], "rotated");
        assert_eq!(public["$shared"]["version"], "v1");
        assert_eq!(public["staging"]["host"], "127.0.0.1");
        fs::remove_dir_all(dir).ok();
    }
}
