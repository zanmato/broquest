//! Stages the app for the website screenshots. Built only with the
//! `screenshots` feature and driven by `script/screenshots/capture`, which
//! runs Broquest on a virtual display against a throwaway data directory and
//! a local demo API.
//!
//! `BROQUEST_SCENE` names the scene to stage, `BROQUEST_DEMO_COLLECTION` the
//! collection directory to open, `BROQUEST_THEME` the theme and
//! `BROQUEST_SCENE_READY` the file written once the window shows the scene.

use std::time::Duration;

use gpui::{App, AsyncWindowContext, Entity, Pixels, Size, Window, px, size};

use crate::app::BroquestApp;
use crate::app_database::{AppDatabase, CollectionData};
use crate::collections::{CollectionFormat, CollectionManager};
use crate::requests::{EditorPanel, RequestEditor};

/// The environment every request is sent in. Its base URL points at the demo
/// API the capture script serves.
const ENVIRONMENT: &str = "Local";

/// What the window shows when the screenshot is taken.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(crate) enum Scene {
    /// A GET request with query parameters, sent, with its JSON response and
    /// the collection tree open on the side.
    Request,
    /// The token request's pre-request and post-response scripts after it
    /// has run.
    Scripting,
    /// The collection's environments, one of them with a secret.
    Environments,
    /// A response narrowed down with a JSONPath filter.
    Filter,
}

impl Scene {
    pub(crate) fn from_env() -> Option<Self> {
        let scene = std::env::var("BROQUEST_SCENE").ok()?;
        match scene.as_str() {
            "request" => Some(Self::Request),
            "scripting" => Some(Self::Scripting),
            "environments" => Some(Self::Environments),
            "filter" => Some(Self::Filter),
            _ => {
                tracing::error!("Unknown BROQUEST_SCENE {scene:?}");
                None
            }
        }
    }

    /// The request tab shown and the size of the request pane, which sits
    /// beside the response in [`Scene::Scripting`] and above it otherwise.
    fn request_pane(self) -> (&'static str, f32) {
        match self {
            Self::Request => ("Query", 250.),
            Self::Scripting => ("Scripts", 780.),
            Self::Environments => ("Query", 250.),
            Self::Filter => ("Path", 170.),
        }
    }

    /// The requests the scene opens as tabs, as (group, name), the last one
    /// in front.
    fn tabs(self) -> &'static [(Option<&'static str>, &'static str)] {
        match self {
            Self::Request => &[
                (Some("Auth"), "Log in"),
                (Some("Shipments"), "Get shipment"),
                (Some("Shipments"), "List shipments"),
            ],
            Self::Scripting => &[
                (Some("Shipments"), "List shipments"),
                (Some("Auth"), "Log in"),
            ],
            Self::Environments => &[
                (Some("Auth"), "Log in"),
                (Some("Shipments"), "List shipments"),
            ],
            Self::Filter => &[
                (Some("Shipments"), "List shipments"),
                (Some("Shipments"), "Get shipment"),
            ],
        }
    }
}

/// The window size for the screenshots, when a scene is being staged.
pub(crate) fn window_size() -> Option<Size<Pixels>> {
    // The visible frame is 1600x1000. On Linux the client-side frame adds a 20px
    // shadow inset on every side, which the capture script crops away.
    Scene::from_env().map(|_| size(px(1640.), px(1040.)))
}

/// Register the demo collection and the settings in a fresh app database.
/// Does nothing when the database already holds a collection, so a data
/// directory that was not thrown away is never written to twice.
pub(crate) fn seed(scene: Scene, database: &AppDatabase) {
    if let Err(error) = smol::block_on(seed_database(scene, database)) {
        tracing::error!("Failed to seed the screenshot database: {error:#}");
    }
}

async fn seed_database(scene: Scene, database: &AppDatabase) -> anyhow::Result<()> {
    if !database.load_collections().await?.is_empty() {
        return Ok(());
    }

    let path = std::env::var("BROQUEST_DEMO_COLLECTION")?;
    database
        .save_collection(&CollectionData {
            id: None,
            name: "Kestrel Shipping".to_string(),
            path,
            position: 0,
            format: CollectionFormat::Broquest.as_db_str().to_string(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        })
        .await?;

    let theme = std::env::var("BROQUEST_THEME").unwrap_or_else(|_| "Catppuccin Mocha".into());
    // The scripts get the full height with the response beside them.
    let layout = if scene == Scene::Scripting {
        "horizontal"
    } else {
        "vertical"
    };
    for (key, value) in [
        ("appearance.theme", theme.as_str()),
        ("general.check_for_updates", "false"),
        ("editor.layout", layout),
    ] {
        database.save_setting(key, value).await?;
    }

    Ok(())
}

/// Stage `scene` in the window that holds `app`, then write the ready file.
pub(crate) fn stage(scene: Scene, app: Entity<BroquestApp>, window: &Window, cx: &App) {
    window
        .spawn(cx, async move |cx| {
            if let Err(error) = stage_scene(scene, &app, cx).await {
                tracing::error!("Failed to stage the screenshot scene: {error:#}");
                return;
            }
            if let Ok(path) = std::env::var("BROQUEST_SCENE_READY")
                && let Err(error) = std::fs::write(&path, "ready")
            {
                tracing::error!("Failed to write {path}: {error}");
            }
        })
        .detach();
}

async fn stage_scene(
    scene: Scene,
    app: &Entity<BroquestApp>,
    cx: &mut AsyncWindowContext,
) -> anyhow::Result<()> {
    let (editor_panel, collections_panel) = app.read_with(cx, |app, _| {
        (app.editor_panel().clone(), app.collections_panel().clone())
    });
    let collection_path = std::env::var("BROQUEST_DEMO_COLLECTION")?;
    settle(cx, 500).await;

    for (group, name) in scene.tabs() {
        open_request(&editor_panel, &collection_path, *group, name, cx)?;
    }
    let editor = editor_panel
        .read_with(cx, |panel, _| panel.active_request_editor())
        .ok_or_else(|| anyhow::anyhow!("no request tab"))?;
    let revealed = scene.tabs().last().map(|(_, name)| *name);

    match scene {
        Scene::Request => {
            send(&editor, cx).await?;
        }
        Scene::Scripting => {
            send(&editor, cx).await?;
        }
        Scene::Environments => {
            send(&editor, cx).await?;
            let collection = manager(cx)?
                .read_with(cx, |manager, _| {
                    manager
                        .get_collection_by_path(&collection_path)
                        .map(|info| info.toml.clone())
                })
                .ok_or_else(|| anyhow::anyhow!("the demo collection did not load"))?;
            editor_panel.update_in(cx, |panel, window, cx| {
                panel.create_and_add_collection_tab(
                    collection,
                    collection_path.clone(),
                    window,
                    cx,
                );
            })?;
            let collection_editor = editor_panel
                .read_with(cx, |panel, _| panel.active_collection_editor())
                .ok_or_else(|| anyhow::anyhow!("no collection tab"))?;
            collection_editor.update_in(cx, |editor, window, cx| {
                editor.show_environment(
                    "Staging",
                    &[
                        ("clientSecret", "ks_stg_4f9a1c7e2b8d6035"),
                        ("webhookSecret", "whsec_9d2e71b04c"),
                    ],
                    window,
                    cx,
                );
            })?;
        }
        Scene::Filter => {
            send(&editor, cx).await?;
            editor.update_in(cx, |editor, window, cx| {
                editor.filter_response("$.events[?(@.status == 'at_hub')]", window, cx);
            })?;
        }
    }

    // The panes have their sizes once the editor has drawn.
    let (request_tab, request_pane) = scene.request_pane();
    editor.update_in(cx, |editor, window, cx| {
        editor.show_request_tab(request_tab, cx);
        editor.resize_request_pane(px(request_pane), window, cx);
    })?;

    // Sending can reload the collection tree, which drops its selection, so
    // the tree is revealed last.
    collections_panel.update_in(cx, |panel, _, cx| {
        panel.reveal_request(
            if scene == Scene::Environments {
                ""
            } else {
                revealed.unwrap_or_default()
            },
            cx,
        );
    })?;

    settle(cx, 1500).await;
    Ok(())
}

/// Open the request `name` from the demo collection in a new tab, with the
/// demo environment selected.
fn open_request(
    editor_panel: &Entity<EditorPanel>,
    collection_path: &str,
    group: Option<&str>,
    name: &str,
    cx: &mut AsyncWindowContext,
) -> anyhow::Result<()> {
    let found = manager(cx)?.read_with(cx, |manager, _| {
        let collection = manager.get_collection_by_path(collection_path)?;
        match group {
            Some(group) => {
                let group = collection.groups.get(group)?;
                group
                    .requests
                    .values()
                    .find(|request| request.name == name)
                    .map(|request| (request.clone(), Some(group.path.clone())))
            }
            None => collection
                .requests
                .values()
                .find(|request| request.name == name)
                .map(|request| (request.clone(), None)),
        }
    });
    let (request, group_path) =
        found.ok_or_else(|| anyhow::anyhow!("no request {name:?} in the demo collection"))?;

    editor_panel.update_in(cx, |panel, window, cx| {
        panel.create_and_add_request_tab(
            request,
            collection_path.to_string(),
            group_path,
            window,
            cx,
        );
        if let Some(editor) = panel.active_request_editor() {
            editor.update(cx, |editor, cx| {
                editor.select_environment(ENVIRONMENT, window, cx);
            });
        }
    })
}

fn manager(cx: &mut AsyncWindowContext) -> anyhow::Result<Entity<CollectionManager>> {
    cx.update(|_, cx| CollectionManager::global(cx))
}

/// Send the request in `editor` and wait for its response.
async fn send(editor: &Entity<RequestEditor>, cx: &mut AsyncWindowContext) -> anyhow::Result<()> {
    editor.update_in(cx, |editor, window, cx| editor.send_request(window, cx))?;
    for _ in 0..200 {
        settle(cx, 50).await;
        if !editor.read_with(cx, |editor, _| editor.is_loading()) {
            return Ok(());
        }
    }
    anyhow::bail!("the request did not finish")
}

async fn settle(cx: &mut AsyncWindowContext, millis: u64) {
    cx.background_executor()
        .timer(Duration::from_millis(millis))
        .await;
}
