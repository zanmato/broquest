use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, InteractiveElement,
    IntoElement, MouseButton, ParentElement, Render, ScrollHandle, SharedString,
    StatefulInteractiveElement as _, Styled, Window, actions, div, prelude::FluentBuilder, px,
};
use gpui_kit::component::{
    ActiveTheme, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::InputEvent,
    menu::ContextMenuExt as _,
    scroll::ScrollableElement,
    select::SelectEvent,
    tab::{Tab, TabBar},
};

use super::editor::{RequestEditor, RequestEditorEvent};
use crate::app_events::AppEvent;
use crate::app_settings::AppSettings;
use crate::collections::{
    CollectionEditor, CollectionManager, CollectionManagerEvent, CollectionToml, GroupEditor,
};
use crate::domain::{HttpMethod, RequestData};
use crate::session::SessionTab;
use crate::settings::SettingsView;
use crate::ui::icon::IconName;
use crate::ui::method_badge::MethodBadge;

mod open_tabs;
mod tab_actions;

pub use open_tabs::OpenTabsPanel;
pub use tab_actions::CloseScope;

/// The key context of the tab panel, where the tab shortcuts apply.
pub(crate) const EDITOR_PANEL_CONTEXT: &str = "editor_panel";

actions!(
    editor_panel,
    [
        CloseTab,
        CloseOtherTabs,
        CloseTabsToTheLeft,
        CloseTabsToTheRight,
        CloseAllTabs,
        NextTab,
        PrevTab
    ]
);

pub enum TabType {
    Request(RequestTab),
    Collection(CollectionTab),
    Group(GroupTab),
    Settings(SettingsTab),
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
    pub is_dirty: bool,
}

pub struct CollectionTab {
    #[allow(dead_code)]
    pub id: usize,
    pub title: String,
    pub collection_editor: Entity<CollectionEditor>,
    #[allow(dead_code)]
    pub collection_path: String, // Link to the collection this belongs to
}

pub struct GroupTab {
    #[allow(dead_code)]
    pub id: usize,
    pub title: String,
    pub group_editor: Entity<GroupEditor>,
    #[allow(dead_code)]
    pub collection_path: String, // Link to the collection this belongs to
}

pub struct SettingsTab {
    pub settings_view: Entity<SettingsView>,
}

impl EventEmitter<AppEvent> for Tab {}

pub struct EditorPanel {
    focus_handle: FocusHandle,
    tabs: Vec<TabType>,
    active_tab_ix: usize,
    tabbar_scroll_handle: ScrollHandle,
    /// The tab last right clicked in the strip, whose menu the strip's
    /// context menu shows. The gpui-kit `Tab` has no context menu of its own.
    context_menu_tab: Option<usize>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl EditorPanel {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let this = cx.entity().downgrade();
        let settings_subscription = window.observe_global::<AppSettings>(cx, move |window, cx| {
            if let Some(this) = this.upgrade() {
                this.update(cx, |this, cx| {
                    for tab in &this.tabs {
                        if let TabType::Request(request_tab) = tab {
                            request_tab.request_editor.update(cx, |editor, cx| {
                                editor.apply_editor_settings(window, cx);
                            });
                        }
                    }
                });
            }
        });

        // Keep open request tabs' environment dropdowns in sync when a
        // collection's environments change elsewhere (e.g. the collection
        // editor). Scoped by collection_path so unrelated collections don't
        // trigger a refresh.
        let manager = CollectionManager::global(cx);
        let manager_subscription = cx.subscribe_in(
            &manager,
            window,
            move |this, _manager, event: &CollectionManagerEvent, window, cx| {
                if let CollectionManagerEvent::EnvironmentsChanged { collection_path } = event {
                    let path = collection_path.to_string();
                    let editors: Vec<Entity<RequestEditor>> = this
                        .tabs
                        .iter()
                        .filter_map(|tab| match tab {
                            TabType::Request(request_tab)
                                if request_tab.collection_path == path =>
                            {
                                Some(request_tab.request_editor.clone())
                            }
                            _ => None,
                        })
                        .collect();
                    for editor in editors {
                        this.load_environments_for_request(Some(&path), &editor, window, cx);
                    }
                }
            },
        );

        Self {
            focus_handle: cx.focus_handle(),
            tabs: vec![],
            active_tab_ix: 0,
            tabbar_scroll_handle: ScrollHandle::default(),
            context_menu_tab: None,
            _subscriptions: vec![settings_subscription, manager_subscription],
        }
    }

    fn close_tab(&mut self, tab_index: usize, cx: &mut Context<Self>) {
        if tab_index >= self.tabs.len() {
            return;
        }
        self.tabs.remove(tab_index);

        if tab_index < self.active_tab_ix {
            // The active tab shifted left, keep it active.
            self.active_tab_ix -= 1;
        } else if self.active_tab_ix >= self.tabs.len() || tab_index == self.active_tab_ix {
            self.active_tab_ix = self.tabs.len().saturating_sub(1);
        }

        cx.notify();
    }

    /// What the command palette needs to know to tell which commands can run.
    pub fn palette_context(&self, cx: &App) -> crate::ui::command_palette::PaletteContext {
        let request = match self.tabs.get(self.active_tab_ix) {
            Some(TabType::Request(tab)) => Some(tab.request_editor.read(cx)),
            _ => None,
        };
        crate::ui::command_palette::PaletteContext {
            tab_count: self.tabs.len(),
            active_tab: self.active_tab_ix,
            request_tab: request.is_some(),
            json_response: request.is_some_and(|editor| editor.has_json_response()),
        }
    }

    pub fn send_active_request(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(TabType::Request(tab)) = self.tabs.get(self.active_tab_ix) {
            let editor = tab.request_editor.clone();
            editor.update(cx, |editor, cx| editor.send_request(window, cx));
        }
    }

    pub fn save_active_request(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(TabType::Request(tab)) = self.tabs.get(self.active_tab_ix) {
            let editor = tab.request_editor.clone();
            editor.update(cx, |editor, cx| editor.save_request(window, cx));
        }
    }

    pub fn close_active_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_tabs_around_active(CloseScope::This, window, cx);
    }

    pub fn select_next_tab(&mut self, cx: &mut Context<Self>) {
        if self.tabs.len() > 1 {
            self.active_tab_ix = (self.active_tab_ix + 1) % self.tabs.len();
            cx.notify();
        }
    }

    pub fn select_prev_tab(&mut self, cx: &mut Context<Self>) {
        if self.tabs.len() > 1 {
            self.active_tab_ix = if self.active_tab_ix == 0 {
                self.tabs.len() - 1
            } else {
                self.active_tab_ix - 1
            };
            cx.notify();
        }
    }

    /// The editor of the active tab, when it is a request.
    pub(crate) fn active_request_editor(&self) -> Option<Entity<RequestEditor>> {
        match self.tabs.get(self.active_tab_ix)? {
            TabType::Request(tab) => Some(tab.request_editor.clone()),
            _ => None,
        }
    }

    /// The editor of the active tab, when it is a collection.
    #[cfg(feature = "screenshots")]
    pub(crate) fn active_collection_editor(&self) -> Option<Entity<CollectionEditor>> {
        match self.tabs.get(self.active_tab_ix)? {
            TabType::Collection(tab) => Some(tab.collection_editor.clone()),
            _ => None,
        }
    }

    fn next_tab_id(&self) -> usize {
        self.tabs
            .iter()
            .map(|tab| match tab {
                TabType::Request(t) => t.id,
                TabType::Collection(t) => t.id,
                TabType::Group(t) => t.id,
                TabType::Settings(_) => 0,
            })
            .max()
            .map_or(0, |max| max + 1)
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
        let tab_id = self.next_tab_id();

        tracing::info!(
            "Getting collection name for collection_path: {}",
            collection_path
        );
        let collection_suffix = CollectionManager::global(cx)
            .read(cx)
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
            editor.set_collection_path(Some(collection_path.clone()), cx);
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
            is_dirty: false,
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
                            if request_tab.request_editor.read(cx).name_input().clone()
                                == name_input_for_closure
                            {
                                let current_name = request_tab
                                    .request_editor
                                    .read(cx)
                                    .name_input()
                                    .read(cx)
                                    .value()
                                    .to_string();

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
            move |editor_panel: &mut Self,
                  _select_state,
                  _event: &SelectEvent<Vec<HttpMethod>>,
                  _window,
                  cx| {
                // Find the request tab that corresponds to this request editor
                for tab in editor_panel.tabs.iter_mut() {
                    if let TabType::Request(request_tab) = tab {
                        // Check if this is the same request editor by comparing the method_select entity
                        if request_tab.request_editor.read(cx).method_select().clone()
                            == method_select_for_closure
                        {
                            let current_method = request_tab
                                .request_editor
                                .read(cx)
                                .method_select()
                                .read(cx)
                                .selected_value()
                                .copied()
                                .unwrap_or(HttpMethod::Get);

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

        // Subscribe to dirty state changes from request editor
        let request_editor_for_closure = request_editor.clone();
        let dirty_subscription = cx.subscribe_in(&request_editor, window, {
            move |editor_panel: &mut Self, _editor, event: &RequestEditorEvent, _window, cx| {
                let is_dirty = match event {
                    RequestEditorEvent::DirtyStateChanged { is_dirty } => is_dirty,
                    // Notify so the session picks up the tab's new environment.
                    RequestEditorEvent::EnvironmentChanged => {
                        cx.notify();
                        return;
                    }
                };
                for tab in editor_panel.tabs.iter_mut() {
                    if let TabType::Request(request_tab) = tab
                        && request_tab.request_editor.clone() == request_editor_for_closure
                    {
                        if request_tab.is_dirty != *is_dirty {
                            request_tab.is_dirty = *is_dirty;
                            cx.notify();
                        }
                        break;
                    }
                }
            }
        });
        self._subscriptions.push(dirty_subscription);

        self.tabs.push(TabType::Request(request_tab));
        self.active_tab_ix = self.tabs.len() - 1;
        self.scroll_tabbar_to_active();

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
            // Read the current environments from the global CollectionManager.
            let manager = CollectionManager::global(cx);
            let manager = manager.read(cx);
            let environments = manager.get_collection_environments(collection_path);
            let settings = manager.environment_settings(collection_path);
            if let Some(environments) = environments {
                tracing::info!(
                    "Loading {} environments for request editor",
                    environments.len()
                );
                request_editor.update(cx, |editor, cx| {
                    editor.set_environments(&environments, &settings, window, cx);
                });
            } else {
                tracing::warn!(
                    "No environments found for collection_path: {}",
                    collection_path
                );
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
        let tab_id = self.next_tab_id();

        let collection_tab_title = if collection_data.collection.name.is_empty() {
            "New Collection".to_string()
        } else {
            collection_data.collection.name.clone()
        };

        let collection_editor = cx.new(|cx| {
            CollectionEditor::new(window, cx, collection_data.clone(), collection_path.clone())
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
                            if collection_tab
                                .collection_editor
                                .read(cx)
                                .name_input()
                                .clone()
                                == name_input_for_closure
                            {
                                let current_name = collection_tab
                                    .collection_editor
                                    .read(cx)
                                    .name_input()
                                    .read(cx)
                                    .value()
                                    .to_string();
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
        self.active_tab_ix = self.tabs.len() - 1;
        self.scroll_tabbar_to_active();

        cx.notify();
    }

    /// Create and add a group tab for editing an existing group
    pub fn create_and_add_group_tab_with_name(
        &mut self,
        collection_path: String,
        group_name: Option<String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let tab_id = self.next_tab_id();

        let group_tab_title = group_name
            .clone()
            .unwrap_or_else(|| "New Group".to_string());

        let group_editor =
            cx.new(|cx| GroupEditor::new(window, cx, collection_path.clone(), group_name.clone()));

        let group_tab = GroupTab {
            id: tab_id,
            title: group_tab_title,
            group_editor: group_editor.clone(),
            collection_path,
        };

        // Set up subscription to update tab title when group name changes
        let name_input = group_editor.read(cx).name_input().clone();
        let name_input_for_closure = name_input.clone();
        let subscription = cx.subscribe_in(&name_input, window, {
            move |editor_panel: &mut Self, _input_state, event: &InputEvent, _window, cx| {
                if let InputEvent::Change = event {
                    // Find the group tab that corresponds to this group editor
                    for tab in editor_panel.tabs.iter_mut() {
                        if let TabType::Group(group_tab) = tab {
                            // Check if this is the same group editor by comparing the name_input entity
                            if group_tab.group_editor.read(cx).name_input().clone()
                                == name_input_for_closure
                            {
                                let current_name = group_tab
                                    .group_editor
                                    .read(cx)
                                    .name_input()
                                    .read(cx)
                                    .value()
                                    .to_string();
                                let tab_name = if current_name.is_empty() {
                                    "New Group".to_string()
                                } else {
                                    current_name
                                };

                                if group_tab.title != tab_name {
                                    group_tab.title = tab_name;
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

        self.tabs.push(TabType::Group(group_tab));
        self.active_tab_ix = self.tabs.len() - 1;
        self.scroll_tabbar_to_active();

        cx.notify();
    }

    /// Add or focus the Settings tab (singleton)
    pub fn add_settings_tab(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        // Check if settings tab already exists
        for (ix, tab) in self.tabs.iter().enumerate() {
            if let TabType::Settings(_) = tab {
                self.active_tab_ix = ix;
                cx.notify();
                return;
            }
        }

        let settings_view = cx.new(SettingsView::new);

        self.tabs
            .push(TabType::Settings(SettingsTab { settings_view }));
        self.active_tab_ix = self.tabs.len() - 1;
        self.scroll_tabbar_to_active();

        cx.notify();
    }

    /// The open tabs as a session to restore on the next launch, and the
    /// index of the active one in it. Tabs with nothing saved on disk to
    /// reopen, such as a request that was never saved, are left out.
    pub fn session_tabs(&self, cx: &App) -> (Vec<SessionTab>, usize) {
        let mut tabs = Vec::new();
        let mut active = 0;
        for (ix, tab) in self.tabs.iter().enumerate() {
            let session_tab = match tab {
                TabType::Request(tab) => {
                    let editor = tab.request_editor.read(cx);
                    match (editor.collection_path(), editor.saved_name()) {
                        (Some(collection_path), Some(name)) if !collection_path.is_empty() => {
                            Some(SessionTab::Request {
                                collection_path: collection_path.to_string(),
                                group_path: editor.group_path().map(str::to_string),
                                name: name.to_string(),
                                environment: editor.selected_environment_name(cx),
                            })
                        }
                        _ => None,
                    }
                }
                TabType::Collection(tab) => Some(SessionTab::Collection {
                    collection_path: tab.collection_path.clone(),
                }),
                TabType::Group(tab) => Some(SessionTab::Group {
                    collection_path: tab.collection_path.clone(),
                    group_name: tab.title.clone(),
                }),
                TabType::Settings(_) => Some(SessionTab::Settings),
            };
            if let Some(session_tab) = session_tab {
                if ix <= self.active_tab_ix {
                    active = tabs.len();
                }
                tabs.push(session_tab);
            }
        }
        (tabs, active)
    }

    /// Reopen the tabs of a previous session. Tabs whose request, group or
    /// collection no longer exists are skipped.
    pub fn restore_tabs(
        &mut self,
        tabs: &[SessionTab],
        active_tab: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut active_ix = None;
        for (ix, tab) in tabs.iter().enumerate() {
            let restored = self.restore_tab(tab, window, cx);
            if restored && ix <= active_tab {
                active_ix = Some(self.tabs.len() - 1);
            }
        }

        if let Some(active_ix) = active_ix {
            self.active_tab_ix = active_ix;
            self.scroll_tabbar_to_active();
        }
        cx.notify();
    }

    fn restore_tab(
        &mut self,
        tab: &SessionTab,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let manager = CollectionManager::global(cx);
        match tab {
            SessionTab::Request {
                collection_path,
                group_path,
                name,
                environment,
            } => {
                let Some(request_data) = manager
                    .read(cx)
                    .find_request(collection_path, group_path.as_deref(), name)
                    .cloned()
                else {
                    tracing::info!("Not restoring tab, request {} is gone", name);
                    return false;
                };
                self.create_and_add_request_tab(
                    request_data,
                    collection_path.clone(),
                    group_path.clone(),
                    window,
                    cx,
                );
                if let (Some(environment), Some(TabType::Request(tab))) =
                    (environment, self.tabs.last())
                {
                    tab.request_editor.update(cx, |editor, cx| {
                        editor.select_environment(environment, window, cx);
                    });
                }
                true
            }
            SessionTab::Collection { collection_path } => {
                let Some(toml) = manager
                    .read(cx)
                    .get_collection_by_path(collection_path)
                    .map(|info| info.toml.clone())
                else {
                    return false;
                };
                self.create_and_add_collection_tab(toml, collection_path.clone(), window, cx);
                true
            }
            SessionTab::Group {
                collection_path,
                group_name,
            } => {
                let exists = manager
                    .read(cx)
                    .get_collection_by_path(collection_path)
                    .is_some_and(|info| info.groups.contains_key(group_name));
                if exists {
                    self.create_and_add_group_tab_with_name(
                        collection_path.clone(),
                        Some(group_name.clone()),
                        window,
                        cx,
                    );
                }
                exists
            }
            SessionTab::Settings => {
                self.add_settings_tab(window, cx);
                true
            }
        }
    }

    /// The tab's title, as the strip and the open tabs list show it.
    fn tab_title(tab: &TabType) -> SharedString {
        match tab {
            TabType::Request(tab) => tab.title.clone().into(),
            TabType::Collection(tab) => tab.title.clone().into(),
            TabType::Group(tab) => tab.title.clone().into(),
            TabType::Settings(_) => "Settings".into(),
        }
    }

    /// The collection the tab belongs to, `None` for settings and requests
    /// that were never saved.
    fn tab_collection_path(tab: &TabType) -> Option<&str> {
        let path = match tab {
            TabType::Request(tab) => &tab.collection_path,
            TabType::Collection(tab) => &tab.collection_path,
            TabType::Group(tab) => &tab.collection_path,
            TabType::Settings(_) => return None,
        };
        (!path.is_empty()).then_some(path.as_str())
    }

    /// Show the tab at `ix`.
    fn activate_tab(&mut self, ix: usize, cx: &mut Context<Self>) {
        if ix < self.tabs.len() {
            self.active_tab_ix = ix;
            self.scroll_tabbar_to_active();
            cx.notify();
        }
    }

    /// Move the tab at `from` to `to`, keeping the same tab active.
    fn move_tab(&mut self, from: usize, to: usize, cx: &mut Context<Self>) {
        if from == to || from >= self.tabs.len() || to >= self.tabs.len() {
            return;
        }
        let tab = self.tabs.remove(from);
        self.tabs.insert(to, tab);
        self.active_tab_ix = moved_index(self.active_tab_ix, from, to);
        cx.notify();
    }

    /// A button that closes the tab at `ix`.
    fn close_tab_button(
        &self,
        id: impl Into<gpui::ElementId>,
        ix: usize,
        cx: &mut Context<Self>,
    ) -> Button {
        Button::new(id)
            .ghost()
            .xsmall()
            .icon(IconName::Close)
            .on_click(cx.listener(move |this, _, window, cx| {
                cx.stop_propagation();
                this.close_tabs(CloseScope::This, ix, window, cx);
            }))
    }

    /// The tab's item in the strip. Tabs can be dragged along the strip to
    /// reorder them, and closed with the middle mouse button.
    fn render_tab_bar_item(&self, ix: usize, tab: &TabType, cx: &mut Context<Self>) -> Tab {
        let muted_color = cx.theme().muted_foreground;
        let close = self.close_tab_button(("close-tab", ix), ix, cx);
        let item = match tab {
            TabType::Request(request_tab) => Tab::new()
                .label(request_tab.title.clone())
                .prefix(
                    h_flex()
                        .gap_2()
                        .items_center()
                        // The prefix sits outside the tab's inner padding, so
                        // it needs its own inset to match the label.
                        .pl_3()
                        .when(request_tab.is_dirty, |this| {
                            this.child(div().w(px(6.)).h(px(6.)).rounded_full().bg(muted_color))
                        })
                        .child(MethodBadge::new(request_tab.method)),
                )
                .suffix(
                    h_flex()
                        .gap_1()
                        .items_center()
                        .child(
                            div()
                                .text_xs()
                                .text_color(muted_color)
                                .pr_1()
                                .child(request_tab.collection_name.clone()),
                        )
                        .child(close),
                ),
            _ => Tab::new()
                .label(Self::tab_title(tab))
                .suffix(h_flex().gap_2().items_center().child(close)),
        };
        let dragged = DraggedTab {
            ix,
            label: Self::tab_title(tab),
        };

        item.on_mouse_down(
            MouseButton::Left,
            cx.listener(move |_this, _event: &gpui::MouseDownEvent, _window, cx| {
                cx.emit(AppEvent::TabChanged { tab_id: ix });
            }),
        )
        .on_mouse_down(
            MouseButton::Middle,
            cx.listener(move |this, _, window, cx| {
                cx.stop_propagation();
                this.close_tabs(CloseScope::This, ix, window, cx);
            }),
        )
        // The strip's context menu shows the menu of the tab right clicked.
        .on_mouse_down(
            MouseButton::Right,
            cx.listener(move |this, _, _, _| {
                this.context_menu_tab = Some(ix);
            }),
        )
        .on_drag(dragged, |dragged, _, _, cx| {
            cx.stop_propagation();
            cx.new(|_| dragged.clone())
        })
        // A tab dragged rightwards lands after the one it is dropped on.
        .drag_over::<DraggedTab>(move |style, dragged, _, cx| {
            if dragged.ix < ix {
                style.border_r_2()
            } else {
                style.border_l_2()
            }
            .border_color(cx.theme().drag_border)
        })
        .on_drop(cx.listener(move |this, dragged: &DraggedTab, _, cx| {
            this.move_tab(dragged.ix, ix, cx);
        }))
    }

    /// Scroll the tab bar so the active tab is visible. `scroll_to_item` defers
    /// to the next paint, so this is safe to call right after adding a tab.
    fn scroll_tabbar_to_active(&self) {
        self.tabbar_scroll_handle.scroll_to_item(self.active_tab_ix);
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
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .flex_1()
            // Constrain to the allocated width. Without this, a flex-1 item
            // defaults to min-width:auto and grows to fit its widest child, so
            // the tab bar's scroll container never overflows and can't scroll.
            .min_w_0()
            .h_full()
            .overflow_hidden()
            .key_context(EDITOR_PANEL_CONTEXT)
            .on_action(cx.listener(|this: &mut Self, _: &CloseTab, window, cx| {
                this.close_active_tab(window, cx);
            }))
            .on_action(
                cx.listener(|this: &mut Self, _: &CloseOtherTabs, window, cx| {
                    this.close_tabs_around_active(CloseScope::Others, window, cx);
                }),
            )
            .on_action(
                cx.listener(|this: &mut Self, _: &CloseTabsToTheLeft, window, cx| {
                    this.close_tabs_around_active(CloseScope::Left, window, cx);
                }),
            )
            .on_action(
                cx.listener(|this: &mut Self, _: &CloseTabsToTheRight, window, cx| {
                    this.close_tabs_around_active(CloseScope::Right, window, cx);
                }),
            )
            .on_action(
                cx.listener(|this: &mut Self, _: &CloseAllTabs, window, cx| {
                    this.close_tabs_around_active(CloseScope::All, window, cx);
                }),
            )
            .on_action(cx.listener(|this: &mut Self, _: &NextTab, _, cx| {
                this.select_next_tab(cx);
            }))
            .on_action(cx.listener(|this: &mut Self, _: &PrevTab, _, cx| {
                this.select_prev_tab(cx);
            }))
            .child(
                // Tab bar. The segmented trough is full-bleed, so it is inset
                // by a wrapper instead of its own padding - otherwise it would
                // square off the editor card's top corners.
                div()
                    .id("editor-tab-strip")
                    .px(px(6.))
                    .pt(px(6.))
                    .min_w_0()
                    // A right click outside the tabs shows no menu. The tab
                    // under the pointer sets itself again after this.
                    .capture_any_mouse_down(cx.listener(
                        |this, event: &gpui::MouseDownEvent, _, _| {
                            if event.button == MouseButton::Right {
                                this.context_menu_tab = None;
                            }
                        },
                    ))
                    .child(
                        TabBar::new("editor-tabs")
                            .segmented()
                            .menu(true)
                            // No .w_full(): an explicit width can interfere with
                            // flex-shrink and stop the tab strip from overflowing to
                            // scroll. The dock's TabBar omits it and relies on flex
                            // stretch, so we match that.
                            .selected_index(self.active_tab_ix)
                            .on_click(cx.listener(|this, ix: &usize, _, cx| {
                                this.active_tab_ix = *ix;
                                cx.notify();
                            }))
                            .track_scroll(&self.tabbar_scroll_handle)
                            .children(
                                self.tabs
                                    .iter()
                                    .enumerate()
                                    .map(|(ix, tab)| self.render_tab_bar_item(ix, tab, cx)),
                            ),
                    )
                    .context_menu({
                        let panel = cx.entity().downgrade();
                        move |menu, _, cx| {
                            let target = panel
                                .upgrade()
                                .and_then(|panel| panel.read(cx).context_menu_tab);
                            match target {
                                Some(ix) => Self::tab_menu(&panel, ix, menu, cx),
                                None => menu,
                            }
                        }
                    }),
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
                            TabType::Group(group_tab) => {
                                this.child(
                                    // Show the GroupEditor directly
                                    div()
                                        .flex_1()
                                        .h_full()
                                        .child(group_tab.group_editor.clone()),
                                )
                            }
                            TabType::Settings(settings_tab) => this.child(
                                div()
                                    .flex_1()
                                    .h_full()
                                    .overflow_y_scrollbar()
                                    .child(settings_tab.settings_view.clone()),
                            ),
                        }
                    }),
            )
    }
}

/// A tab being dragged along the strip or the open tabs list.
#[derive(Clone)]
struct DraggedTab {
    ix: usize,
    label: SharedString,
}

impl Render for DraggedTab {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .px_3()
            .py_1()
            .text_sm()
            .rounded(cx.theme().radius)
            .bg(cx.theme().secondary)
            .border_1()
            .border_color(cx.theme().border)
            .child(self.label.clone())
    }
}

/// Where the tab at `ix` ends up after the tab at `from` moved to `to`.
fn moved_index(ix: usize, from: usize, to: usize) -> usize {
    if ix == from {
        to
    } else if from < to && (from + 1..=to).contains(&ix) {
        ix - 1
    } else if to < from && (to..from).contains(&ix) {
        ix + 1
    } else {
        ix
    }
}

#[cfg(test)]
mod tests {
    use super::moved_index;

    #[test]
    fn moving_a_tab_shifts_the_ones_between() {
        // Move the tab at 1 to 3: tabs 2 and 3 shift left.
        assert_eq!(moved_index(1, 1, 3), 3);
        assert_eq!(moved_index(2, 1, 3), 1);
        assert_eq!(moved_index(3, 1, 3), 2);
        assert_eq!(moved_index(0, 1, 3), 0);
        assert_eq!(moved_index(4, 1, 3), 4);
        // Move the tab at 3 to 1: tabs 1 and 2 shift right.
        assert_eq!(moved_index(3, 3, 1), 1);
        assert_eq!(moved_index(1, 3, 1), 2);
        assert_eq!(moved_index(2, 3, 1), 3);
        assert_eq!(moved_index(0, 3, 1), 0);
    }
}
