#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod app_database;
mod app_events;
mod assets;
mod collection_editor;
mod collection_manager;
mod collection_types;
mod collections_panel;
mod editor_panel;
mod environment_editor;
mod environment_resolver;
mod form_editor;
mod header_editor;
mod highlighting;
mod http_client;
mod icon;
mod path_parameter_editor;
mod query_parameter_editor;
mod request_editor;
mod script_editor;
mod script_engine;
mod themes_manager;
mod ui;
mod variable_store;

use assets::Assets;
use gpui::{AppContext, Application, SharedString, WindowBounds, WindowOptions, px, size};
use gpui_component::{Theme, ThemeRegistry};
use tracing_subscriber::{layer::SubscriberExt as _, util::SubscriberInitExt as _};

use collection_manager::CollectionManager;
use highlighting::register_highlighting;
use themes_manager::ThemesManager;

fn main() {
    let app = Application::new().with_assets(Assets);

    app.run(move |cx| {
        tracing_subscriber::registry()
            .with(tracing_subscriber::fmt::layer())
            .with(
                tracing_subscriber::EnvFilter::from_default_env()
                    .add_directive("gpui_component=trace".parse().unwrap()),
            )
            .init();

        gpui_component::init(cx);
        gpui_tokio::init(cx);

        // Register syntax highlighting
        register_highlighting(cx);

        // Initialize themes by copying embedded themes to user directory
        if let Err(e) = ThemesManager::init() {
            tracing::error!("Failed to initialize themes: {}", e);
        } else {
            tracing::info!("Themes initialized successfully");
        }

        // Initialize app database
        let db = async_std::task::block_on(async {
            match app_database::AppDatabase::new().await {
                Ok(db) => {
                    tracing::info!("App database initialized");
                    Ok(db)
                }
                Err(e) => {
                    tracing::error!("Failed to initialize app database: {}", e);
                    Err(anyhow::anyhow!("App database init failed: {}", e))
                }
            }
        })
        .unwrap();

        // Get user settings from database
        let user_settings =
            async_std::task::block_on(async { db.get_user_settings().await }).unwrap();

        // Load and watch themes from ./themes directory
        let theme_name = match user_settings {
            Some(settings) => SharedString::from(settings.theme),
            None => SharedString::from("Catppuccin Macchiato"),
        };

        if let Err(err) =
            ThemeRegistry::watch_dir(ThemesManager::themes_directory(), cx, move |cx| {
                if let Some(theme) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
                    Theme::global_mut(cx).apply_config(&theme);
                    tracing::info!("Applying theme {}", theme_name);
                }
            })
        {
            tracing::error!("Failed to watch themes directory: {}", err);
        }

        cx.set_global(db);

        // Initialize global CollectionManager
        let mut collection_manager = CollectionManager::new();

        // Load collections from database and cache them
        if let Err(e) = collection_manager.load_saved(cx) {
            tracing::error!("Failed to load saved collections: {}", e);
        }

        cx.set_global(collection_manager);

        // Initialize HTTP client
        let http_client = http_client::HttpClientService::new();
        cx.set_global(http_client);
        cx.activate(true);

        let window_bounds = gpui::Bounds::centered(None, size(px(1280.), px(900.)), cx);

        let window_options = WindowOptions {
            window_bounds: Some(WindowBounds::Windowed(window_bounds)),
            titlebar: Some(gpui::TitlebarOptions {
                title: Some("broquest".into()),
                appears_transparent: true,
                traffic_light_position: Some(gpui::Point {
                    x: px(8.0),
                    y: px(6.0),
                }),
            }),
            window_decorations: Some(gpui::WindowDecorations::Client),
            window_min_size: Some(size(px(800.), px(600.))),
            focus: true,
            show: true,
            kind: gpui::WindowKind::Normal,
            is_movable: true,
            is_minimizable: true,
            is_resizable: true,
            tabbing_identifier: None,
            display_id: None,
            window_background: gpui::WindowBackgroundAppearance::Opaque,
            app_id: Some("broquest".into()),
        };

        cx.spawn(async move |cx| {
            cx.open_window(window_options, |window, cx| {
                let broquest_app = cx.new(|cx| app::BroquestApp::new(window, cx));
                cx.new(|cx| gpui_component::Root::new(broquest_app, window, cx))
            })?;
            Ok::<_, anyhow::Error>(())
        })
        .detach();
    });
}
