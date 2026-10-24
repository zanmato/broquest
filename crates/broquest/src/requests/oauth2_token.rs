//! The Get Token and Clear Token buttons of the OAuth2 auth editor.

use gpui::{App, Context, Entity, SharedString, Window};
use gpui_kit::component::{WindowExt, notification::NotificationType};

use super::auth_editor::AuthEditor;
use crate::collections::{CollectionManager, EnvironmentToml};
use crate::domain::{AuthType, OAuth2Auth};
use crate::environments::EnvironmentResolver;
use crate::http::{AuthVariables, HttpClientService, current_unix_timestamp, oauth2::CachedToken};

/// Which cache a request's token lives in: its collection and environment.
pub fn token_scope(collection_path: Option<&str>, environment: Option<&str>) -> String {
    format!(
        "{}\n{}",
        collection_path.unwrap_or_default(),
        environment.unwrap_or_default()
    )
}

/// The OAuth2 settings a request sends with, following Inherit to the
/// collection.
pub fn oauth2_settings(
    auth: AuthType,
    collection_path: Option<&str>,
    cx: &App,
) -> Option<OAuth2Auth> {
    let auth = match auth {
        AuthType::Inherit => collection_path
            .and_then(|path| {
                CollectionManager::global(cx)
                    .read(cx)
                    .get_collection_by_path(path)
            })
            .and_then(|collection| collection.toml.collection.auth.clone())?,
        auth => auth,
    };
    match auth {
        AuthType::OAuth2(oauth) => Some(oauth),
        _ => None,
    }
}

/// The variables OAuth2 settings resolve with, as sending would use them.
pub fn auth_variables(
    collection_path: Option<&str>,
    environment: Option<&EnvironmentToml>,
    cx: &App,
) -> AuthVariables {
    let mut vars = AuthVariables::default();
    let Some(collection) = collection_path.and_then(|path| {
        CollectionManager::global(cx)
            .read(cx)
            .get_collection_by_path(path)
    }) else {
        return vars;
    };

    vars.collection = collection
        .toml
        .collection
        .vars
        .iter()
        .filter(|v| v.enabled && !v.key.is_empty())
        .map(|v| (v.key.clone(), v.value.clone()))
        .collect();
    vars.runtime = collection
        .runtime_vars
        .iter()
        .map(|(k, v)| {
            let value = match v {
                serde_json::Value::String(s) => s.clone(),
                other => other.to_string(),
            };
            (k.clone(), value)
        })
        .collect();
    if let Some(environment) = environment {
        match EnvironmentResolver::new().load_environment_data(
            &collection.data.name,
            &environment.name,
            std::slice::from_ref(environment),
            cx,
        ) {
            Ok((variables, secrets)) => {
                vars.variables = variables;
                vars.secrets = secrets;
            }
            Err(e) => tracing::error!("Failed to load environment data: {}", e),
        }
    }
    vars
}

/// How long a token has left, for the editor.
fn describe(token: &CachedToken) -> SharedString {
    match token.expires_at {
        None => "Got a token without an expiry".into(),
        Some(at) => {
            let left = at - current_unix_timestamp();
            if left <= 0 {
                "The token has expired".into()
            } else if left < 120 {
                format!("Got a token, it expires in {left} seconds").into()
            } else {
                format!("Got a token, it expires in {} minutes", left / 60).into()
            }
        }
    }
}

/// Fetch a new token for `oauth` and cache it where sending looks.
pub fn get_token<T: 'static>(
    auth_editor: Entity<AuthEditor>,
    oauth: OAuth2Auth,
    vars: AuthVariables,
    scope: String,
    window: &mut Window,
    cx: &mut Context<T>,
) {
    let http = HttpClientService::global(cx).clone();
    let resolved = http.resolve_oauth2(&oauth, &vars);
    auth_editor.update(cx, |editor, cx| {
        let waiting = (resolved.grant_type == crate::domain::OAuth2GrantType::AuthorizationCode)
            .then(|| SharedString::from("Waiting for the browser sign in"));
        editor.set_token_status(waiting, true, cx);
    });

    cx.spawn_in(window, async move |_, cx| {
        let result = async_compat::Compat::new(http.fetch_oauth2_token(resolved, &scope)).await;
        cx.update(|window, cx| {
            let status = match result {
                Ok(token) => {
                    let status = describe(&token);
                    window.push_notification((NotificationType::Success, status.clone()), cx);
                    status
                }
                Err(error) => {
                    window.push_notification(
                        (
                            NotificationType::Error,
                            SharedString::from(format!("{}: {}", error.summary, error.details)),
                        ),
                        cx,
                    );
                    SharedString::from(error.summary)
                }
            };
            auth_editor.update(cx, |editor, cx| {
                editor.set_token_status(Some(status), false, cx);
            });
        })
        .ok();
    })
    .detach();
}

/// Forget the cached token for `oauth`.
pub fn clear_token(
    auth_editor: &Entity<AuthEditor>,
    oauth: OAuth2Auth,
    vars: AuthVariables,
    scope: &str,
    cx: &mut App,
) {
    let http = HttpClientService::global(cx);
    let resolved = http.resolve_oauth2(&oauth, &vars);
    let status = if http.clear_oauth2_token(&resolved, scope) {
        "Token cleared, the next send fetches a new one"
    } else {
        "No token was cached"
    };
    auth_editor.update(cx, |editor, cx| {
        editor.set_token_status(Some(status.into()), false, cx);
    });
}
