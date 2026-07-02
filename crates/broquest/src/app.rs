use gpui::{
    Action, App, AppContext, BorrowAppContext, Context, Entity, EventEmitter, FocusHandle,
    Focusable, InteractiveElement, IntoElement, Menu, MenuItem, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Subscription, Window, actions, div,
    prelude::FluentBuilder, px, svg,
};
use gpui_component::{
    ActiveTheme, Icon, Root, Selectable, Sizable as _, Theme, ThemeRegistry, TitleBar, WindowExt,
    button::{Button, ButtonVariants as _},
    global_state::GlobalState,
    h_flex,
    kbd::Kbd,
    menu::AppMenuBar,
    notification::{Notification, NotificationType},
    v_flex,
};

#[allow(unused_imports)]
use crate::requests::{CloseTab, NextTab, PrevTab};
use crate::{
    app_database::{AppDatabase, CollectionData, UserSetting},
    app_events::AppEvent,
    collections::{CollectionManager, CollectionsPanel},
    domain::{AuthType, HttpMethod, RequestData},
    history::HistoryPanel,
    requests::EditorPanel,
    result_ext::ResultExt,
    update_manager::UpdateManager,
};

actions!(
    broquest_app,
    [
        Quit,
        OpenNewCollectionTab,
        OpenCollection,
        OpenSettings,
        ToggleCommandPalette,
        NewScratchRequest
    ]
);

#[derive(Action, Clone, PartialEq)]
#[action(namespace = broquest_app, no_json)]
pub(crate) struct SwitchTheme(pub(crate) SharedString);

#[derive(Action, Clone, PartialEq)]
#[action(namespace = broquest_app, no_json)]
pub(crate) struct OpenRequestFromPalette {
    pub(crate) collection_path: SharedString,
    pub(crate) request_name: SharedString,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SidebarTab {
    Collections,
    History,
}

pub struct BroquestApp {
    focus_handle: FocusHandle,
    sidebar_collapsed: bool,
    sidebar_tab: SidebarTab,
    collections_panel: Entity<CollectionsPanel>,
    history_panel: Entity<HistoryPanel>,
    editor_panel: Entity<EditorPanel>,
    command_palette: Entity<crate::ui::command_palette::CommandPalette>,
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

        let history_panel = cx.new(|cx| HistoryPanel::new(window, cx));

        let editor_panel = cx.new(|cx| EditorPanel::new(window, cx, false));
        let command_palette =
            cx.new(|cx| crate::ui::command_palette::CommandPalette::new(window, cx));
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
                        auth: AuthType::None,
                        pre_request_script: None,
                        post_response_script: None,
                        vars: Vec::new(),
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

        // Subscribe to history events from request editors
        let history_panel_for_events = history_panel.clone();
        let history_subscription = cx.subscribe_in(
            &editor_panel,
            window,
            move |_app, _panel, event, _window, cx| {
                if let AppEvent::RequestHistoryRecorded(entry) = event {
                    history_panel_for_events.update(cx, |panel, cx| {
                        panel.add_entry(entry.clone(), cx);
                    });
                }
            },
        );
        subscriptions.push(history_subscription);

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

        // Check for post-update notification
        let update_manager = UpdateManager::global(cx);
        if let Some(_prev_version) = update_manager.just_updated_from.read(cx).as_ref() {
            let current = env!("CARGO_PKG_VERSION");

            let app_entity = cx.entity().downgrade();
            window.defer(cx, move |window, cx| {
                if let Some(app_entity) = app_entity.upgrade() {
                    window.push_notification(
                        Notification::new()
                            .message(format!("Updated to v{}, click to view changelog", current))
                            .with_type(NotificationType::Success)
                            .on_click(window.listener_for(&app_entity, move |_, _, _, cx| {
                                let changelog_url =
                                    UpdateManager::changelog_url(&format!("v{}", current));
                                cx.open_url(&changelog_url);
                            })),
                        cx,
                    );
                }
            });
        }

        // Defer font application so it runs after the theme has been loaded from disk.
        window.defer(cx, |window, cx| {
            crate::settings::apply_font_settings(cx);
            window.refresh();
        });

        let focus_handle = cx.focus_handle();
        window.focus(&focus_handle, cx);

        Self {
            focus_handle,
            sidebar_collapsed: false,
            sidebar_tab: SidebarTab::Collections,
            collections_panel,
            history_panel,
            editor_panel,
            command_palette,
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
        let collection_data = crate::collections::create_empty_collection();
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

    fn on_new_scratch_request(
        &mut self,
        _: &NewScratchRequest,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let request_data = RequestData {
            name: "New Request".to_string(),
            method: HttpMethod::Get,
            url: String::new(),
            path_params: Vec::new(),
            query_params: Vec::new(),
            headers: Vec::new(),
            body: String::new(),
            auth: AuthType::None,
            pre_request_script: None,
            post_response_script: None,
            vars: Vec::new(),
        };

        self.editor_panel.update(cx, |editor_panel, cx| {
            editor_panel.create_and_add_request_tab(request_data, String::new(), None, window, cx);
        });
    }

    fn on_open_request_from_palette(
        &mut self,
        action: &OpenRequestFromPalette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let collection_path = action.collection_path.to_string();
        let request_name = action.request_name.to_string();

        let request_data = {
            let manager = CollectionManager::global(cx);
            let Some(collection) = manager.get_collection_by_path(&collection_path) else {
                window.push_notification(
                    (
                        NotificationType::Warning,
                        SharedString::from(format!("Collection not found: {}", collection_path)),
                    ),
                    cx,
                );
                return;
            };

            let direct = collection
                .requests
                .values()
                .find(|r| r.name == request_name)
                .cloned();

            direct.or_else(|| {
                collection
                    .groups
                    .values()
                    .flat_map(|g| g.requests.values())
                    .find(|r| r.name == request_name)
                    .cloned()
            })
        };

        let Some(request_data) = request_data else {
            window.push_notification(
                (
                    NotificationType::Warning,
                    SharedString::from(format!("Request not found: {}", request_name)),
                ),
                cx,
            );
            return;
        };

        self.editor_panel.update(cx, |editor_panel, cx| {
            editor_panel.create_and_add_request_tab(
                request_data,
                collection_path,
                None,
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

                // Detect the collection format: prefer OpenCollection
                // (opencollection.yml) when present, otherwise the native
                // collection.toml.
                let dir_path = std::path::PathBuf::from(&dir_str);
                let is_opencollection =
                    crate::collections::find_opencollection_file(&dir_path).is_some();
                let has_toml = dir_path.join("collection.toml").exists();

                if is_opencollection || has_toml {
                    let _ = window
                        .update(|window, cx| {
                            // Load the collection with CollectionManager
                            let collection = cx.update_global(
                                |collection_manager: &mut CollectionManager, cx| {
                                    let load_result = if is_opencollection {
                                        collection_manager.load_opencollection_dir(&dir_path)
                                    } else {
                                        collection_manager.load_collection_toml(&dir_path)
                                    };
                                    match load_result {
                                        Ok(col) => {
                                            window.push_notification(
                                                (
                                                    NotificationType::Success,
                                                    SharedString::from(format!(
                                                        "Opened {}",
                                                        col.collection.name.clone()
                                                    )),
                                                ),
                                                cx,
                                            );

                                            Some(col)
                                        }
                                        Err(e) => {
                                            window.push_notification(
                                                (
                                                    NotificationType::Error,
                                                    SharedString::from(format!(
                                                        "Failed to load collection: {e}"
                                                    )),
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
                                let format = if is_opencollection {
                                    crate::collections::CollectionFormat::OpenCollection
                                } else {
                                    crate::collections::CollectionFormat::Broquest
                                };

                                cx.spawn(async move |_| {
                                    app_database
                                        .save_collection(&CollectionData {
                                            id: None,
                                            name: collection.collection.name.clone(),
                                            path: dir_str.to_string(),
                                            position: 1,
                                            format: format.as_db_str().to_string(),
                                            created_at: chrono::Utc::now(),
                                            updated_at: chrono::Utc::now(),
                                        })
                                        .await
                                })
                                .detach();
                            }
                        })
                        .log_err()
                        .ok();
                } else {
                    let _ = window
                        .update(|window, cx| {
                            window.push_notification(
                                "No collection.toml or opencollection.yml found in selected directory",
                                cx,
                            );
                        })
                        .log_err()
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
            crate::settings::apply_font_settings(cx);
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

    fn on_settings(&mut self, _: &OpenSettings, window: &mut Window, cx: &mut Context<Self>) {
        self.editor_panel.update(cx, |editor_panel, cx| {
            editor_panel.add_settings_tab(window, cx);
        });
    }

    fn on_toggle_command_palette(
        &mut self,
        _: &ToggleCommandPalette,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.command_palette.update(cx, |palette, cx| {
            palette.toggle(window, cx);
        });
    }

    fn render_palette_trigger(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let shortcut = Kbd::binding_for_action(&ToggleCommandPalette, None, window);
        div()
            .id("palette-trigger")
            .flex()
            .items_center()
            .justify_between()
            .gap_8()
            .h(px(26.))
            .w(px(420.))
            .px_3()
            .rounded_md()
            .bg(cx.theme().input_background())
            .border_1()
            .border_color(cx.theme().border)
            .text_color(cx.theme().muted_foreground)
            .text_sm()
            .cursor_pointer()
            .hover(|this| this.border_color(cx.theme().accent_foreground))
            .on_click(cx.listener(|_, _, window, cx| {
                window.dispatch_action(Box::new(ToggleCommandPalette), cx);
            }))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(Icon::new(crate::ui::icon::IconName::Search).size(px(14.)))
                    .child("Search commands..."),
            )
            .when_some(shortcut, |this, kbd| this.child(kbd))
    }

    fn render_update_button(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let update_manager = UpdateManager::global(cx);
        let has_update = update_manager.pending_update.read(cx).is_some();

        if has_update {
            div().child(
                Button::new("update-available")
                    .ghost()
                    .compact()
                    .small()
                    .label("Update available, click to restart")
                    .on_click(|_, _window, _cx| {
                        UpdateManager::apply_pending_update();
                    }),
            )
        } else {
            div()
        }
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

        div()
            .track_focus(&self.focus_handle)
            .key_context("BroquestApp")
            .flex()
            .flex_col()
            .on_action(cx.listener(Self::on_quit))
            .on_action(cx.listener(Self::on_open_new_collection_tab))
            .on_action(cx.listener(Self::on_open_collection_dialog))
            .on_action(cx.listener(Self::on_switch_theme))
            .on_action(cx.listener(Self::on_settings))
            .on_action(cx.listener(Self::on_toggle_command_palette))
            .on_action(cx.listener(Self::on_open_request_from_palette))
            .on_action(cx.listener(Self::on_new_scratch_request))
            .on_action(cx.listener(|this, _: &crate::requests::Send, window, cx| {
                this.editor_panel.update(cx, |panel, cx| {
                    panel.send_active_request(window, cx);
                });
            }))
            .on_action(cx.listener(|this, _: &crate::requests::Save, window, cx| {
                this.editor_panel.update(cx, |panel, cx| {
                    panel.save_active_request(window, cx);
                });
            }))
            .on_action(cx.listener(|this, _: &CloseTab, _, cx| {
                this.editor_panel.update(cx, |panel, cx| {
                    panel.close_active_tab(cx);
                });
            }))
            .on_action(cx.listener(|this, _: &NextTab, _, cx| {
                this.editor_panel.update(cx, |panel, cx| {
                    panel.select_next_tab(cx);
                });
            }))
            .on_action(cx.listener(|this, _: &PrevTab, _, cx| {
                this.editor_panel.update(cx, |panel, cx| {
                    panel.select_prev_tab(cx);
                });
            }))
            .size_full()
            .bg(cx.theme().background)
            .text_color(cx.theme().foreground)
            // Title bar
            .child(
                TitleBar::new().child(
                    div()
                        .flex()
                        .items_center()
                        .justify_between()
                        .w_full()
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_4()
                                .child(
                                    svg()
                                        .h(px(32.))
                                        .w(px(145.))
                                        .text_color(window.text_style().color)
                                        .path("img/broquest.svg"),
                                )
                                .child(self.app_menu_bar.clone()),
                        )
                        .child(self.render_palette_trigger(window, cx))
                        .child(self.render_update_button(cx)),
                ),
            )
            // Main content area
            .child(
                div()
                    .flex()
                    .flex_1()
                    .w_full()
                    .overflow_hidden()
                    .items_start()
                    // Left side: sidebar with tab switcher
                    .when(!self.sidebar_collapsed, |this| {
                        this.child(
                            div()
                                .h_full()
                                .w(px(256.))
                                .overflow_hidden()
                                .border_r_1()
                                .border_color(cx.theme().border)
                                .child(
                                    v_flex()
                                        .h_full()
                                        .child(
                                            // Sidebar tab switcher
                                            h_flex()
                                                .items_center()
                                                .gap_1()
                                                .pt(px(3.))
                                                .pb(px(4.))
                                                .px(px(4.))
                                                .border_b_1()
                                                .border_color(cx.theme().border)
                                                .child(
                                                    Button::new("tab-collections")
                                                        .small()
                                                        .ghost()
                                                        .selected(
                                                            self.sidebar_tab
                                                                == SidebarTab::Collections,
                                                        )
                                                        .label("Collections")
                                                        .flex_1()
                                                        .on_click(cx.listener(|this, _, _, cx| {
                                                            this.sidebar_tab =
                                                                SidebarTab::Collections;
                                                            cx.notify();
                                                        })),
                                                )
                                                .child(
                                                    Button::new("tab-history")
                                                        .small()
                                                        .ghost()
                                                        .selected(
                                                            self.sidebar_tab == SidebarTab::History,
                                                        )
                                                        .label("History")
                                                        .flex_1()
                                                        .on_click(cx.listener(|this, _, _, cx| {
                                                            this.sidebar_tab = SidebarTab::History;
                                                            if !this.history_panel.read(cx).loaded {
                                                                this.history_panel.update(
                                                                    cx,
                                                                    |panel, cx| {
                                                                        panel.load_history(cx);
                                                                    },
                                                                );
                                                            }
                                                            cx.notify();
                                                        })),
                                                ),
                                        )
                                        .child(match self.sidebar_tab {
                                            SidebarTab::Collections => div()
                                                .flex_1()
                                                .min_h_0()
                                                .child(self.collections_panel.clone()),
                                            SidebarTab::History => div()
                                                .flex_1()
                                                .min_h_0()
                                                .child(self.history_panel.clone()),
                                        }),
                                ),
                        )
                    })
                    // Main panel
                    .child(
                        div()
                            .flex()
                            .flex_1()
                            // Allow this flex item to shrink below its content
                            // width; without it the panel grows to fit the widest
                            // tab/content and the tab bar never overflows to scroll.
                            .min_w_0()
                            .h_full()
                            .overflow_hidden()
                            .child(self.editor_panel.clone()),
                    ),
            )
            .children(sheet_layer)
            .children(dialog_layer)
            .children(notification_layer)
            .child(self.command_palette.clone())
    }
}

fn init_menus(cx: &mut App) {
    cx.bind_keys([
        #[cfg(target_os = "macos")]
        gpui::KeyBinding::new("cmd-q", Quit, None),
        #[cfg(not(target_os = "macos"))]
        gpui::KeyBinding::new("alt-f4", Quit, None),
        #[cfg(target_os = "macos")]
        gpui::KeyBinding::new("cmd-,", OpenSettings, None),
        #[cfg(not(target_os = "macos"))]
        gpui::KeyBinding::new("ctrl-,", OpenSettings, None),
        gpui::KeyBinding::new("secondary-k", ToggleCommandPalette, None),
        gpui::KeyBinding::new("secondary-n", NewScratchRequest, None),
        gpui::KeyBinding::new("ctrl-tab", NextTab, None),
        gpui::KeyBinding::new("ctrl-shift-tab", PrevTab, None),
    ]);

    cx.set_menus(build_menu());

    let menu = build_menu().into_iter().map(|menu| menu.owned()).collect();
    GlobalState::global_mut(cx).set_app_menus(menu);
}

fn build_menu() -> Vec<Menu> {
    vec![
        Menu {
            name: "File".into(),
            items: vec![
                MenuItem::action("New Collection", OpenNewCollectionTab),
                MenuItem::action("Open Collection", OpenCollection),
                MenuItem::Separator,
                MenuItem::action("Settings", OpenSettings),
                MenuItem::Separator,
                MenuItem::action("Quit", Quit),
            ],
            disabled: false,
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
            disabled: false,
        },
    ]
}
