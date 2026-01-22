use gpui::{
    Action, App, AppContext, BorrowAppContext, Context, Entity, EventEmitter, FocusHandle,
    Focusable, InteractiveElement, IntoElement, Menu, MenuItem, ParentElement, Render,
    SharedString, Styled, Subscription, Window, actions, div, prelude::FluentBuilder, px, svg,
};
use gpui_component::{
    ActiveTheme, Root, TITLE_BAR_HEIGHT, Theme, ThemeRegistry, TitleBar, WindowExt,
    menu::AppMenuBar, notification::NotificationType,
};

use crate::{
    app_database::{AppDatabase, CollectionData, UserSetting},
    app_events::AppEvent,
    collection_manager::CollectionManager,
    collections_panel::CollectionsPanel,
    editor_panel::EditorPanel,
    http::HttpMethod,
    request_editor::RequestData,
};

actions!(broquest_app, [Quit, OpenNewCollectionTab, OpenCollection]);

#[derive(Action, Clone, PartialEq)]
#[action(namespace = broquest_app, no_json)]
pub(crate) struct SwitchTheme(pub(crate) SharedString);

pub struct BroquestApp {
    focus_handle: FocusHandle,
    sidebar_collapsed: bool,
    collections_panel: Entity<CollectionsPanel>,
    editor_panel: Entity<EditorPanel>,
    app_menu_bar: Entity<AppMenuBar>,
    _subscriptions: Vec<Subscription>,
}

impl BroquestApp {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        init_menus(cx);

        let collections_panel = cx.new(|cx| CollectionsPanel::new(window, cx));

        // Load collections after creating the panel
        collections_panel.update(cx, |panel, cx| {
            panel.load_collections(cx);
        });

        let editor_panel = cx.new(|cx| EditorPanel::new(window, cx, false));
        let app_menu_bar = AppMenuBar::new(cx);

        let mut subscriptions = Vec::new();

        // Set up event handling for CreateNewRequestTab events with window access
        let editor_panel_clone = editor_panel.clone();
        let collections_panel_clone = collections_panel.clone();
        let subscription = cx.subscribe_in(
            &collections_panel,
            window,
            move |_app, _panel, event, window, cx| {
                if let AppEvent::CreateNewRequestTab {
                    request_data,
                    collection_path,
                } = event
                {
                    tracing::info!(
                        "Received CreateNewRequestTab event for: {}",
                        request_data.name
                    );
                    editor_panel_clone.update(cx, |editor_panel, cx| {
                        editor_panel.create_and_add_request_tab(
                            request_data.clone(),
                            collection_path.to_string(),
                            None,
                            window,
                            cx,
                        );
                    });
                }

                if let AppEvent::NewRequest {
                    collection_path,
                    group_path,
                } = event
                {
                    tracing::info!(
                        "Received NewRequest event for collection_path: {:?}, group_path: {:?}",
                        collection_path,
                        group_path
                    );

                    // Create a new empty request
                    let request_data = RequestData {
                        name: "New Request".to_string(),
                        method: HttpMethod::Get,
                        url: "".to_string(),
                        path_params: Vec::new(),
                        query_params: Vec::new(),
                        headers: Vec::new(),
                        body: "".to_string(),
                        pre_request_script: None,
                        post_response_script: None,
                    };

                    editor_panel_clone.update(cx, |editor_panel, cx| {
                        editor_panel.create_and_add_request_tab(
                            request_data.clone(),
                            collection_path.to_string(),
                            group_path.as_ref().map(|gp| gp.to_string()),
                            window,
                            cx,
                        );
                    });
                }

                if let AppEvent::CreateNewCollectionTab {
                    collection_data,
                    collection_path,
                } = event
                {
                    tracing::info!(
                        "Received CreateNewCollectionTab event for: {}",
                        collection_data.collection.name
                    );
                    editor_panel_clone.update(cx, |editor_panel, cx| {
                        editor_panel.create_and_add_collection_tab(
                            collection_data.clone(),
                            collection_path.to_string(),
                            window,
                            cx,
                        );
                    });
                }

                if let AppEvent::CreateNewGroupTab {
                    collection_path,
                    group_name,
                } = event
                {
                    tracing::info!(
                        "Received CreateNewGroupTab event for: {:?}",
                        group_name.as_ref().map(|s| s.as_ref())
                    );
                    editor_panel_clone.update(cx, |editor_panel, cx| {
                        editor_panel.create_and_add_group_tab_with_name(
                            collection_path.to_string(),
                            group_name.as_ref().map(|s| s.to_string()),
                            window,
                            cx,
                        );
                    });
                }

                if let AppEvent::CollectionDeleted { collection_path } = event {
                    tracing::info!("Received CollectionDeleted event for: {}", collection_path);
                }

                if let AppEvent::GroupCreated { .. } = event {
                    // Reload collections panel to pick up the new group
                    collections_panel_clone.update(cx, |panel, cx| {
                        panel.load_collections(cx);
                    });
                }

                if let AppEvent::GroupDeleted { .. } = event {
                    // Reload collections panel to remove the deleted group
                    collections_panel_clone.update(cx, |panel, cx| {
                        panel.load_collections(cx);
                    });
                }
            },
        );
        subscriptions.push(subscription);

        let subscription =
            cx.subscribe_in(&editor_panel, window, move |app, panel, event, _, cx| {
                if let AppEvent::ToggleSidebar = event {
                    app.sidebar_collapsed = !app.sidebar_collapsed;
                    tracing::info!("New sidebar_collapsed state: {}", app.sidebar_collapsed);

                    // Update editor panel's sidebar state
                    panel.update(cx, |panel, cx| {
                        panel.set_sidebar_collapsed(app.sidebar_collapsed, cx);
                    });
                }
            });
        subscriptions.push(subscription);

        // Subscribe to CollectionManager global updates
        let collections_panel_updates = collections_panel.clone();
        let collection_subscription =
            window.observe_global::<CollectionManager>(cx, move |_window, cx| {
                // This callback will be triggered whenever CollectionManager is updated
                tracing::debug!("CollectionManager updated");

                // Refresh collections panel when collections change
                collections_panel_updates.update(cx, |panel: &mut CollectionsPanel, cx| {
                    panel.load_collections(cx);
                });
            });

        subscriptions.push(collection_subscription);

        Self {
            focus_handle: cx.focus_handle(),
            sidebar_collapsed: false,
            collections_panel,
            editor_panel,
            app_menu_bar,
            _subscriptions: subscriptions,
        }
    }

    fn on_quit(&mut self, _: &Quit, _window: &mut Window, cx: &mut Context<Self>) {
        cx.quit();
    }

    fn on_open_new_collection_tab(
        &mut self,
        _: &OpenNewCollectionTab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Create an empty collection and open it in a new tab
        let collection_data = crate::collection_types::create_empty_collection();
        let collection_path = "".to_string();

        // Directly create the new collection tab
        self.editor_panel.update(cx, |editor_panel, cx| {
            editor_panel.create_and_add_collection_tab(
                collection_data,
                collection_path,
                window,
                cx,
            );
        });
    }

    fn on_open_collection_dialog(
        &mut self,
        _: &OpenCollection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let path = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Select collection directory".into()),
        });

        cx.spawn_in(window, async move |_, window| {
            if let Some(path) = path.await.ok()?.ok()?
                && let Some(dir_path) = path.first()
                && let Some(dir_str) = dir_path.to_str()
            {
                let dir_str = dir_str.to_owned();

                // Validate that the path contains a collection.toml file
                let collection_file = std::path::PathBuf::from(&dir_str).join("collection.toml");

                if collection_file.exists() {
                    let _ = window
                        .update(|window, cx| {
                            // Load the collection with CollectionManager
                            let collection = cx.update_global(
                                |collection_manager: &mut CollectionManager, cx| {
                                    match collection_manager.load_collection_toml(
                                        std::path::PathBuf::from(&dir_str).as_path(),
                                    ) {
                                        Ok(col) => {
                                            window.push_notification(
                                                (
                                                    NotificationType::Success,
                                                    SharedString::from(
                                                        format!(
                                                            "Opened {}",
                                                            col.collection.name.clone()
                                                        )
                                                        .to_string(),
                                                    ),
                                                ),
                                                cx,
                                            );

                                            Some(col)
                                        }
                                        Err(e) => {
                                            window.push_notification(
                                                (
                                                    NotificationType::Error,
                                                    "Failed to load collection",
                                                ),
                                                cx,
                                            );

                                            tracing::error!(
                                                "Failed to load collection from {}: {}",
                                                dir_str,
                                                e
                                            );

                                            None
                                        }
                                    }
                                },
                            );

                            // Save the collection to the database and get its ID
                            if let Some(collection) = collection {
                                let app_database = AppDatabase::global(cx).clone();

                                cx.spawn(async move |_| {
                                    app_database
                                        .save_collection(&CollectionData {
                                            id: None,
                                            name: collection.collection.name.clone(),
                                            path: dir_str.to_string(),
                                            position: 1,
                                            created_at: chrono::Utc::now(),
                                            updated_at: chrono::Utc::now(),
                                        })
                                        .await
                                })
                                .detach();
                            }
                        })
                        .ok();
                } else {
                    let _ = window
                        .update(|window, cx| {
                            window.push_notification(
                                "No collection.toml found in selected directory",
                                cx,
                            );
                        })
                        .ok();
                }
            }
            Some(())
        })
        .detach();
    }

    fn on_switch_theme(
        &mut self,
        switch: &SwitchTheme,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let theme_name = switch.0.clone();
        if let Some(theme_config) = ThemeRegistry::global(cx).themes().get(&theme_name).cloned() {
            Theme::global_mut(cx).apply_config(&theme_config);
        }

        let app_database = AppDatabase::global(cx).clone();
        cx.spawn(async move |_, _| {
            app_database
                .save_user_settings(&UserSetting {
                    theme: theme_name.to_string(),
                })
                .await
                .ok();
        })
        .detach();

        window.refresh();
    }
}

impl EventEmitter<AppEvent> for BroquestApp {}

impl Focusable for BroquestApp {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for BroquestApp {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let sheet_layer = Root::render_sheet_layer(window, cx);
        let dialog_layer = Root::render_dialog_layer(window, cx);
        let notification_layer = Root::render_notification_layer(window, cx);
        let window_bounds = window.bounds();

        div()
            .flex()
            .flex_col()
            .on_action(cx.listener(Self::on_quit))
            .on_action(cx.listener(Self::on_open_new_collection_tab))
            .on_action(cx.listener(Self::on_open_collection_dialog))
            .on_action(cx.listener(Self::on_switch_theme))
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            // Title bar
            .child(
                TitleBar::new().child(
                    div()
                        .flex()
                        .items_center()
                        .gap_4()
                        .child(
                            svg()
                                .h(px(22.))
                                .w(px(100.))
                                .text_color(window.text_style().color)
                                .path("img/broquest.svg"),
                        )
                        .child(self.app_menu_bar.clone()),
                ),
            )
            // Main content area
            .child(
                div()
                    .flex()
                    .flex_1()
                    .w_full()
                    .items_start()
                    // Left side: Connections panel sidebar
                    .when(!self.sidebar_collapsed, |this| {
                        let window_height = window_bounds.size.height;
                        this.child(
                            div()
                                .h(window_height - TITLE_BAR_HEIGHT - px(25.))
                                .w(px(256.))
                                .overflow_hidden()
                                .border_r_1()
                                .border_color(cx.theme().border)
                                .child(self.collections_panel.clone()),
                        )
                    })
                    // Main panel
                    .child({
                        let window_height = window_bounds.size.height;
                        div()
                            .flex()
                            .flex_1()
                            .h(window_height - TITLE_BAR_HEIGHT - px(25.))
                            .overflow_hidden()
                            .child(self.editor_panel.clone())
                    }),
            )
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
    }
}

fn init_menus(cx: &mut App) {
    cx.bind_keys([
        #[cfg(target_os = "macos")]
        gpui::KeyBinding::new("cmd-q", Quit, None),
        #[cfg(not(target_os = "macos"))]
        gpui::KeyBinding::new("alt-f4", Quit, None),
    ]);

    cx.set_menus(vec![
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New Collection", OpenNewCollectionTab),
                MenuItem::action("Open Collection", OpenCollection),
                MenuItem::Separator,
                theme_menu(cx),
                MenuItem::Separator,
                MenuItem::action("Quit", Quit),
            ],
        },
        Menu {
            name: "Edit".into(),
            items: vec![
                MenuItem::action("Undo", gpui_component::input::Undo),
                MenuItem::action("Redo", gpui_component::input::Redo),
                MenuItem::separator(),
                MenuItem::action("Cut", gpui_component::input::Cut),
                MenuItem::action("Copy", gpui_component::input::Copy),
                MenuItem::action("Paste", gpui_component::input::Paste),
                MenuItem::separator(),
                MenuItem::action("Select All", gpui_component::input::SelectAll),
            ],
        },
    ]);
}

fn theme_menu(cx: &App) -> MenuItem {
    let themes = ThemeRegistry::global(cx).sorted_themes();
    MenuItem::Submenu(Menu {
        name: "Theme".into(),
        items: themes
            .iter()
            .map(|theme| MenuItem::action(theme.name.clone(), SwitchTheme(theme.name.clone())))
            .collect(),
    })
}
