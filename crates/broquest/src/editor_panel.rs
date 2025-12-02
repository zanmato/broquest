use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Render, Styled, Window, div, prelude::FluentBuilder,
    px,
};
use gpui_component::{
    ActiveTheme, Icon, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::InputEvent,
    select::SelectEvent,
    tab::{Tab, TabBar},
};

use crate::collection_editor::CollectionEditor;
use crate::collection_types::CollectionToml;
use crate::icon::IconName;
use crate::request_editor::{RequestData, RequestEditor};
use crate::{app_database::TabData, collection_manager::CollectionManager};
use crate::{app_events::AppEvent, request_editor::HttpMethod};

pub enum TabType {
    Request(RequestTab),
    Collection(CollectionTab),
}

pub struct RequestTab {
    #[allow(dead_code)]
    pub id: usize,
    pub title: String,
    pub method: HttpMethod,
    pub collection_name: String,
    pub request_editor: Entity<RequestEditor>,
    #[allow(dead_code)]
    pub collection_path: String, // Link to the collection this request belongs to
}

pub struct CollectionTab {
    #[allow(dead_code)]
    pub id: usize,
    pub title: String,
    pub collection_editor: Entity<CollectionEditor>,
    #[allow(dead_code)]
    pub collection_path: String, // Link to the collection this belongs to
}

impl EventEmitter<AppEvent> for Tab {}

pub struct EditorPanel {
    focus_handle: FocusHandle,
    tabs: Vec<TabType>,
    active_tab_ix: usize,
    next_tab_id: usize,
    sidebar_collapsed: bool,
    _subscriptions: Vec<gpui::Subscription>,
}

impl EditorPanel {
    pub fn set_sidebar_collapsed(&mut self, collapsed: bool, cx: &mut Context<Self>) {
        self.sidebar_collapsed = collapsed;
        cx.notify();
    }

    fn close_tab(&mut self, tab_index: usize, cx: &mut Context<Self>) {
        // Remove tab from UI
        self.tabs.remove(tab_index);

        // Adjust active tab index
        if self.active_tab_ix >= self.tabs.len() || tab_index == self.active_tab_ix {
            self.active_tab_ix = self.tabs.len().saturating_sub(1);
        }
        self.next_tab_id = match self.tabs.len() {
            0 => 0,
            _ => self.active_tab_ix + 1,
        };

        cx.notify();
    }

    /// Create and add a new tab with full RequestData
    pub fn create_and_add_request_tab(
        &mut self,
        request_data: RequestData,
        collection_path: String,
        group_path: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        tracing::info!(
            "Getting collection name for collection_path: {}",
            collection_path
        );
        let collection_suffix = CollectionManager::global(cx)
            .get_collection_by_path(&collection_path)
            .map_or_else(
                || {
                    tracing::error!("Collection not found for path: {}", collection_path);
                    String::new()
                },
                |collection_info| collection_info.data.name.clone(),
            );

        let request_editor = cx.new(|cx| RequestEditor::new(window, cx));

        // Set the request data from the collection
        request_editor.update(cx, |editor, cx| {
            editor.set_collection_path(Some(collection_path.clone()));
            if let Some(ref group_path) = group_path {
                editor.set_group_path(Some(group_path.clone()));
            }
            editor.set_request_data(request_data.clone(), window, cx);
        });

        // Set up two-way binding between URL input and query parameter editor
        request_editor.update(cx, |editor, cx| {
            editor.setup_url_query_binding(window, cx);
        });

        // Load environments for this request
        self.load_environments_for_request(Some(&collection_path), &request_editor, window, cx);

        let request_tab = RequestTab {
            id: tab_id,
            title: request_data.name,
            method: request_data.method,
            collection_name: collection_suffix.clone(),
            request_editor: request_editor.clone(),
            collection_path: collection_path.clone(),
        };

        // Set up subscriptions to update tab title when request name or method changes
        let name_input = request_editor.read(cx).name_input().clone();
        let name_input_for_closure = name_input.clone();
        let name_subscription = cx.subscribe_in(&name_input, window, {
            move |editor_panel: &mut Self, _input_state, event: &InputEvent, _window, cx| {
                if let InputEvent::Change = event {
                    // Find the request tab that corresponds to this request editor
                    for tab in editor_panel.tabs.iter_mut() {
                        if let TabType::Request(request_tab) = tab {
                            // Check if this is the same request editor by comparing the name_input entity
                            if request_tab.request_editor.read(cx).name_input().clone() == name_input_for_closure {
                                let current_name = request_tab.request_editor.read(cx).name_input().read(cx).value().to_string();

                                if request_tab.title != current_name {
                                    request_tab.title = current_name;
                                    cx.notify();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });
        self._subscriptions.push(name_subscription);

        let method_select = request_editor.read(cx).method_select().clone();
        let method_select_for_closure = method_select.clone();
        let method_subscription = cx.subscribe_in(&method_select, window, {
            move |editor_panel: &mut Self, _select_state, _event: &SelectEvent<Vec<HttpMethod>>, _window, cx| {
                // Find the request tab that corresponds to this request editor
                for tab in editor_panel.tabs.iter_mut() {
                    if let TabType::Request(request_tab) = tab {
                        // Check if this is the same request editor by comparing the method_select entity
                        if request_tab.request_editor.read(cx).method_select().clone() == method_select_for_closure {
                            let current_method = request_tab.request_editor.read(cx).method_select().read(cx).selected_value().copied().unwrap_or(HttpMethod::Get);

                            if request_tab.method != current_method {
                                request_tab.method = current_method;
                                cx.notify();
                                break;
                            }
                        }
                    }
                }
            }
        });
        self._subscriptions.push(method_subscription);

        self.tabs.push(TabType::Request(request_tab));
        self.active_tab_ix = tab_id;

        cx.notify();
    }

    /// Load environments from global CollectionManager and set them on a request editor
    fn load_environments_for_request(
        &mut self,
        collection_path: Option<&str>,
        request_editor: &Entity<RequestEditor>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(collection_path) = collection_path {
            // Get the global CollectionManager
            let collection_manager = CollectionManager::global(cx);
            if let Some(environments) =
                collection_manager.get_collection_environments(collection_path)
            {
                tracing::info!(
                    "Loading {} environments for request editor",
                    environments.len()
                );
                request_editor.update(cx, |editor, cx| {
                    editor.set_environments(&environments, window, cx);
                });
            } else {
                tracing::warn!("No environments found for collection_path: {}", collection_path);
            }
        }
    }

    /// Create and add a new collection tab
    pub fn create_and_add_collection_tab(
        &mut self,
        collection_data: CollectionToml,
        collection_path: String,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tab_id = self.next_tab_id;
        self.next_tab_id += 1;

        let collection_tab_title = if collection_data.collection.name.is_empty() {
            "New Collection".to_string()
        } else {
            collection_data.collection.name.clone()
        };

        let collection_editor = cx.new(|cx| {
            CollectionEditor::new(
                window,
                cx,
                collection_data.clone(),
                collection_path.clone(),
            )
        });

        let collection_tab = CollectionTab {
            id: tab_id,
            title: collection_tab_title,
            collection_editor: collection_editor.clone(),
            collection_path,
        };

        // Set up subscription to update tab title when collection name changes
        let name_input = collection_editor.read(cx).name_input().clone();
        let name_input_for_closure = name_input.clone();
        let subscription = cx.subscribe_in(&name_input, window, {
            move |editor_panel: &mut Self, _input_state, event: &InputEvent, _window, cx| {
                if let InputEvent::Change = event {
                    // Find the collection tab that corresponds to this collection editor
                    for tab in editor_panel.tabs.iter_mut() {
                        if let TabType::Collection(collection_tab) = tab {
                            // Check if this is the same collection editor by comparing the name_input entity
                            if collection_tab.collection_editor.read(cx).name_input().clone() == name_input_for_closure {
                                let current_name = collection_tab.collection_editor.read(cx).name_input().read(cx).value().to_string();
                                let tab_name = if current_name.is_empty() {
                                    "New Collection".to_string()
                                } else {
                                    current_name
                                };

                                if collection_tab.title != tab_name {
                                    collection_tab.title = tab_name;
                                    cx.notify();
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        });
        self._subscriptions.push(subscription);

        self.tabs.push(TabType::Collection(collection_tab));
        self.active_tab_ix = tab_id;

        cx.notify();
    }

    fn set_active_tab(&mut self, ix: usize, _: &mut Window, cx: &mut Context<Self>) {
        if ix < self.tabs.len() {
            // Tab switching no longer saves automatically - tabs are only saved on query execution
            self.active_tab_ix = ix;
            cx.notify();
        }
    }

    /// Load saved query tabs from the app database
    pub fn new_with_saved_tabs(
        window: &mut Window,
        cx: &mut Context<Self>,
        sidebar_collapsed: bool,
        saved_tabs: Vec<TabData>,
    ) -> Self {
        tracing::info!("Loading {} saved tabs", saved_tabs.len());

        let mut panel = Self {
            focus_handle: cx.focus_handle(),
            tabs: vec![],
            active_tab_ix: 0,
            next_tab_id: 0,
            sidebar_collapsed,
            _subscriptions: Vec::new(),
        };

        panel.restore_saved_tabs_with_connections_sync(saved_tabs, window, cx);

        panel
    }

    /// Restore saved tabs
    pub fn restore_saved_tabs_with_connections_sync(
        &mut self,
        saved_tabs: Vec<TabData>,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        if saved_tabs.is_empty() {
            tracing::debug!("No saved tabs to restore");
        }

        // TODO: Restore saved tabs
    }
}

impl Focusable for EditorPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<AppEvent> for EditorPanel {}

impl Render for EditorPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let current_tab = self.tabs.get(self.active_tab_ix);

        div()
            .flex()
            .flex_col()
            .flex_1()
            .h_full()
            .overflow_hidden()
            .child(
                // Tab bar
                TabBar::new("editor-tabs")
                    .w_full()
                    .min_h(px(32.))
                    .selected_index(self.active_tab_ix)
                    .on_click(cx.listener(|this, ix: &usize, window, cx| {
                        this.set_active_tab(*ix, window, cx);
                    }))
                    .prefix(
                        Button::new("toggle-sidebar")
                            .ghost()
                            .small()
                            .icon(if self.sidebar_collapsed {
                                Icon::new(IconName::PanelLeftOpen).size_4()
                            } else {
                                Icon::new(IconName::PanelLeftClose).size_4()
                            })
                            .on_click(cx.listener(|_this, _event, _window, cx| {
                                cx.emit(AppEvent::ToggleSidebar);
                            })),
                    )
                    .children(self.tabs.iter().enumerate().map(|(ix, tab)| {
                        match tab {
                            TabType::Request(request_tab) => {
                                let tab_index = ix;

                                Tab::new()
                                    .label(request_tab.title.clone())
                                    .prefix(
                                        div()
                                            .pl_3()
                                            .pt(px(3.))
                                            .font_family(cx.theme().mono_font_family.clone())
                                            .font_bold()
                                            .text_color(request_tab.method.get_color(cx))
                                            .child(request_tab.method.as_str()),
                                    )
                                    .suffix(
                                        h_flex()
                                            .gap_1()
                                            .items_center()
                                            .child(
                                                div()
                                                    .text_xs()
                                                    .text_color(cx.theme().muted_foreground)
                                                    .pr_1()
                                                    .child(request_tab.collection_name.clone()),
                                            )
                                            .child(
                                                Button::new(("close-tab", ix))
                                                    .ghost()
                                                    .xsmall()
                                                    .icon(IconName::Close)
                                                    .on_click(cx.listener(
                                                        move |this, _, _, cx| {
                                                            this.close_tab(tab_index, cx);
                                                        },
                                                    )),
                                            )
                                            .into_any_element(),
                                    )
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(
                                            move |_this,
                                                  _event: &gpui::MouseDownEvent,
                                                  _window,
                                                  cx| {
                                                cx.emit(AppEvent::TabChanged { tab_id: tab_index });
                                            },
                                        ),
                                    )
                            }
                            TabType::Collection(collection_tab) => {
                                let tab_index = ix;

                                // Clone the tab title to avoid lifetime issues
                                let tab_title = collection_tab.title.clone();
                                Tab::new()
                                    .label(&tab_title)
                                    .on_mouse_down(
                                        MouseButton::Left,
                                        cx.listener(
                                            move |_this,
                                                  _event: &gpui::MouseDownEvent,
                                                  _window,
                                                  cx| {
                                                // Single click - activate the tab by setting active tab index
                                                cx.emit(AppEvent::TabChanged { tab_id: tab_index });
                                            },
                                        ),
                                    )
                                    .suffix(
                                        h_flex()
                                            .gap_2()
                                            .items_center()
                                            .child(
                                                Button::new(("close-tab", ix))
                                                    .ghost()
                                                    .xsmall()
                                                    .icon(IconName::Close)
                                                    .on_click(cx.listener(
                                                        move |this, _, _, cx| {
                                                            this.close_tab(tab_index, cx);
                                                        },
                                                    )),
                                            )
                                            .into_any_element(),
                                    )
                            }
                        }
                    })),
            )
            // Render the active tab's complete view
            .child(
                div()
                    .flex_1()
                    .overflow_hidden()
                    .when_some(current_tab, |this, tab| {
                        match tab {
                            TabType::Request(request_tab) => {
                                this.child(
                                    // Just show the RequestEditor directly, which has its own layout
                                    div()
                                        .flex_1()
                                        .h_full()
                                        .child(request_tab.request_editor.clone()),
                                )
                            }
                            TabType::Collection(collection_tab) => {
                                this.child(
                                    // Show the CollectionEditor directly
                                    div()
                                        .flex_1()
                                        .h_full()
                                        .child(collection_tab.collection_editor.clone()),
                                )
                            }
                        }
                    }),
            )
    }
}
