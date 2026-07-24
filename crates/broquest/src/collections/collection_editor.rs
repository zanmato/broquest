use gpui::{
    App, Context, Entity, FocusHandle, Focusable, KeyBinding, SharedString, Subscription, Window,
    actions, div, prelude::*, px,
};
use gpui_component::{
    ActiveTheme, Icon, IndexPath, Sizable as _, StyledExt, WindowExt,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    kbd::Kbd,
    notification::NotificationType,
    resizable::{ResizableState, h_resizable, resizable_panel},
    scroll::ScrollableElement,
    select::{Select, SelectItem, SelectState},
    switch::Switch,
    tab::{Tab, TabBar},
    text, v_flex,
};

use super::manager::{CollectionManager, CollectionManagerEvent};
use super::openapi::OpenAPIImporter;
use super::types::{CollectionMeta, CollectionToml};

use crate::{
    app_database::{AppDatabase, CollectionData},
    domain::AuthType,
    environments::EnvironmentEditor,
    requests::AuthEditor,
    result_ext::ResultExt,
    ui::icon::IconName,
};

const CONTEXT: &str = "collection_editor";

/// What to import into a newly-created collection.
#[derive(Debug, Clone, Copy, PartialEq)]
enum ImportKind {
    None,
    OpenApi,
    Wsdl,
}

impl ImportKind {
    fn all() -> Vec<ImportKind> {
        vec![ImportKind::None, ImportKind::OpenApi, ImportKind::Wsdl]
    }

    fn label(&self) -> &'static str {
        match self {
            ImportKind::None => "No import",
            ImportKind::OpenApi => "OpenAPI",
            ImportKind::Wsdl => "WSDL / SOAP",
        }
    }
}

impl SelectItem for ImportKind {
    type Value = ImportKind;

    fn title(&self) -> SharedString {
        self.label().into()
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

actions!(collection_editor, [Save, ToggleDocsEdit]);

pub struct CollectionEditor {
    active_tab: usize,
    collection_data: CollectionToml,
    collection_path: String,
    environment_editor: Entity<EnvironmentEditor>,
    auth_editor: Entity<AuthEditor>,
    name_input: Entity<InputState>,
    path_input: Entity<InputState>,
    /// Editable collection-level variables (`CollectionMeta.vars`).
    vars_editor: Entity<crate::requests::KeyValueEditor>,
    /// Read-only runtime variable inspector for this collection.
    vars_view: Entity<super::VarsView>,
    /// Markdown docs editor (code editor mode).
    docs_input: Entity<InputState>,
    /// Whether the docs pane shows the editor (true) or the rendered preview.
    docs_editing: bool,
    /// Split state between the collection form and the docs pane.
    docs_split_state: Entity<ResizableState>,
    focus_handle: FocusHandle,
    // Import: pick a spec format to import from (or None).
    import_select: Entity<SelectState<Vec<ImportKind>>>,
    openapi_spec_input: Entity<InputState>,
    openapi_spec_path: Option<String>,
    wsdl_spec_input: Entity<InputState>,
    // Save this collection in Bruno's OpenCollection (YAML) format instead of
    // broquest's native TOML.
    use_opencollection: bool,
    _subscriptions: Vec<Subscription>,
}

impl CollectionEditor {
    pub fn init(cx: &mut App) {
        cx.bind_keys([
            KeyBinding::new("secondary-s", Save, Some(CONTEXT)),
            KeyBinding::new("secondary-e", ToggleDocsEdit, Some(CONTEXT)),
        ]);
    }

    fn on_toggle_docs_edit(
        &mut self,
        _: &ToggleDocsEdit,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.docs_editing = !self.docs_editing;
        cx.notify();
    }

    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        collection_data: CollectionToml,
        collection_path: String,
    ) -> Self {
        let collection_name = collection_data.collection.name.clone();
        let environments_count = collection_data.environments.len();
        let collection_path_for_log = collection_path.clone();
        let collection_path_for_lookup = collection_path.clone();

        let environment_editor =
            cx.new(|cx| EnvironmentEditor::new(window, cx, &collection_data.collection.name));

        let collection_auth = collection_data.collection.auth.clone();
        let auth_editor = cx.new(|cx| {
            let mut editor = AuthEditor::new_without_inherit(window, cx);
            if let Some(auth) = &collection_auth {
                editor.set_auth(auth, window, cx);
            }
            editor
        });

        let name_input = cx
            .new(|cx| InputState::new(window, cx).default_value(&collection_data.collection.name));

        let path_input = cx.new(|cx| InputState::new(window, cx).default_value(&collection_path));

        let openapi_spec_input = cx.new(|cx| InputState::new(window, cx));
        let wsdl_spec_input = cx.new(|cx| InputState::new(window, cx));
        let import_select = cx.new(|cx| {
            SelectState::new(
                ImportKind::all(),
                Some(IndexPath::default().row(0)),
                window,
                cx,
            )
        });

        // Collection vars editor (editable key/value list) + read-only runtime
        // vars view, both scoped to this collection.
        let vars_editor = cx.new(|cx| {
            crate::requests::KeyValueEditor::new(
                crate::requests::KeyValueConfig::new(
                    "collection-vars",
                    "Variable name",
                    "Variable value",
                ),
                window,
                cx,
            )
        });
        let vars_view = cx.new(|cx| {
            let mut view = super::VarsView::new(cx);
            view.set_collection(
                if collection_path.is_empty() {
                    None
                } else {
                    Some(collection_path.clone())
                },
                cx,
            );
            view
        });

        // Markdown docs editor, seeded from the collection's docs.
        let docs_seed = collection_data.collection.docs.clone().unwrap_or_default();
        let docs_input = cx.new(|cx| {
            InputState::new(window, cx)
                .code_editor("markdown")
                .placeholder("Write collection docs in Markdown...")
                .default_value(docs_seed)
        });
        let docs_split_state = cx.new(|_cx| ResizableState::default());

        // Keep the environment editor fresh when this collection's environments
        // change elsewhere (another tab, a script writing back dirty vars, etc.).
        let manager = CollectionManager::global(cx);
        let sub_path = collection_path_for_lookup.clone();
        let manager_subscription = cx.subscribe_in(
            &manager,
            window,
            move |this, _manager, event: &CollectionManagerEvent, window, cx| {
                if let CollectionManagerEvent::EnvironmentsChanged { collection_path } = event
                    && collection_path.as_ref() == sub_path
                {
                    this.reload_environments(&sub_path, window, cx);
                }
            },
        );

        let editor = Self {
            active_tab: 0, // Collection tab
            collection_data,
            collection_path,
            environment_editor,
            auth_editor,
            name_input,
            path_input,
            vars_editor,
            vars_view,
            docs_input,
            docs_editing: false,
            docs_split_state,
            focus_handle: cx.focus_handle(),
            import_select,
            openapi_spec_input,
            openapi_spec_path: None,
            wsdl_spec_input,
            use_opencollection: false,
            _subscriptions: vec![manager_subscription],
        };

        // Load initial environments data from CollectionManager
        tracing::info!(
            "CollectionEditor::new called with collection_path: '{}', collection_name: '{}', environments_count: {}",
            collection_path_for_log,
            collection_name,
            environments_count
        );

        // Always load from CollectionManager cache since we're using path-based approach.
        // Extract all needed data up front so the immutable borrow of `cx` ends
        // before we mutate the editor below.
        let (environments, collection_vars) = {
            let collection_manager = CollectionManager::global(cx);
            let collection_manager = collection_manager.read(cx);
            let envs = collection_manager.get_collection_environments(&collection_path_for_lookup);
            let vars = collection_manager
                .get_collection_by_path(&collection_path_for_lookup)
                .map(|info| info.toml.collection.vars.clone())
                .unwrap_or_default();
            (envs, vars)
        };

        if let Some(environments) = &environments {
            tracing::info!(
                "Found {} environments for collection '{}': {:?}",
                environments.len(),
                collection_name,
                environments.iter().map(|e| &e.name).collect::<Vec<_>>()
            );
            editor.environment_editor.update(cx, |env_editor, cx| {
                env_editor.load_environments(environments, window, cx);
            });
        } else {
            tracing::warn!(
                "No environments found for collection '{}' (path: {})",
                collection_name,
                collection_path_for_log
            );
        }

        // Seed the vars editor with any existing collection-level variables.
        editor.vars_editor.update(cx, |vars_editor, cx| {
            vars_editor.set_pairs(&collection_vars, window, cx);
        });

        editor
    }

    pub fn get_collection_data_for_save(&self, cx: &App) -> CollectionToml {
        let name = self.name_input.read(cx).value().to_string();
        let version = self.collection_data.collection.version.clone();
        let collection_type = self.collection_data.collection.collection_type.clone();
        let docs_text = self.docs_input.read(cx).value().to_string();
        let docs = if docs_text.trim().is_empty() {
            None
        } else {
            Some(docs_text)
        };
        let ignore = self.collection_data.collection.ignore.clone();

        let auth = match self.auth_editor.read(cx).get_auth(cx) {
            AuthType::None => None,
            auth => Some(auth),
        };

        // Get environments from the environment editor
        let environments = self
            .environment_editor
            .read(cx)
            .get_environments_for_save(cx);

        CollectionToml {
            collection: CollectionMeta {
                name,
                version,
                collection_type,
                docs,
                ignore,
                auth,
                vars: self.vars_editor.read(cx).get_pairs(cx),
            },
            environments,
        }
    }

    pub fn save_secrets(&self, cx: &App) -> Result<(), Box<dyn std::error::Error>> {
        let collection_name = self.name_input.read(cx).value().to_string();
        self.environment_editor
            .read(cx)
            .save_secrets(&collection_name, cx)
    }

    fn reload_environments(
        &mut self,
        collection_path: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let environments = CollectionManager::global(cx)
            .read(cx)
            .get_collection_environments(collection_path);
        if let Some(environments) = environments {
            self.environment_editor.update(cx, |env_editor, cx| {
                env_editor.load_environments(&environments, window, cx);
            });
        }
    }

    fn set_active_tab(&mut self, tab_index: usize, cx: &mut Context<Self>) {
        self.active_tab = tab_index;
        cx.notify();
    }

    /// The currently-selected import source.
    fn selected_import(&self, cx: &App) -> ImportKind {
        self.import_select
            .read(cx)
            .selected_value()
            .copied()
            .unwrap_or(ImportKind::None)
    }

    /// The left-hand form: name, path, import toggles, OpenCollection switch.
    fn render_collection_form(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let import_kind = self.selected_import(cx);
        let label_color = cx.theme().muted_foreground;

        v_flex()
            .gap_3()
            .p_3()
            .child(
                // Collection name input
                v_flex().gap_2().children([
                    div()
                        .text_sm()
                        .font_medium()
                        .text_color(label_color)
                        .child("Name"),
                    div().child(Input::new(&self.name_input)),
                ]),
            )
            .child(
                // Import section: a single dropdown (No import / OpenAPI /
                // WSDL) with the relevant input shown below the selection.
                v_flex()
                    .gap_2()
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .text_color(label_color)
                            .child("Import from"),
                    )
                    .child(
                        div()
                            .w_full()
                            .child(Select::new(&self.import_select).small()),
                    )
                    .when(import_kind == ImportKind::OpenApi, |this| {
                        this.child(
                            h_flex()
                                .gap_2()
                                .child(Input::new(&self.openapi_spec_input))
                                .child(
                                    Button::new("browse-spec")
                                        .outline()
                                        .icon(IconName::FolderOpen)
                                        .on_click(cx.listener(|this, _, window, cx| {
                                            this.handle_browse_spec_file(window, cx)
                                        })),
                                ),
                        )
                    })
                    .when(import_kind == ImportKind::Wsdl, |this| {
                        this.child(h_flex().child(Input::new(&self.wsdl_spec_input)))
                    }),
            )
            .child(
                // Directory path input
                v_flex().gap_2().children([
                    div()
                        .text_sm()
                        .font_medium()
                        .text_color(cx.theme().muted_foreground)
                        .child("Collection Path"),
                    h_flex().gap_2().child(Input::new(&self.path_input)).child(
                        Button::new("browse_path")
                            .outline()
                            .icon(IconName::FolderOpen)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.handle_browse_directory(window, cx)
                            })),
                    ),
                ]),
            )
            .child(
                // OpenCollection format toggle (kept at the bottom of the form).
                v_flex().gap_2().child(
                    h_flex().gap_2().items_center().child(
                        Switch::new("use-opencollection")
                            .small()
                            .label("Use OpenCollection format")
                            .checked(self.use_opencollection)
                            .on_click(cx.listener(|this, checked, _window, cx| {
                                this.use_opencollection = *checked;
                                cx.notify();
                            })),
                    ),
                ),
            )
    }

    /// The Collection tab: a horizontal split with the form on the left and the
    /// docs view/editor on the right.
    fn render_collection_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_resizable("collection-docs")
            .with_state(&self.docs_split_state)
            .child(
                resizable_panel()
                    .size(px(420.))
                    .size_range(px(280.)..gpui::Pixels::MAX)
                    .child(
                        v_flex()
                            .size_full()
                            .overflow_y_scrollbar()
                            .child(self.render_collection_form(cx)),
                    ),
            )
            .child(resizable_panel().child(v_flex().size_full().child(self.render_docs_pane(cx))))
    }

    /// The docs pane: a toggle between a rendered markdown preview and a
    /// markdown code editor.
    fn render_docs_pane(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let is_editing = self.docs_editing;
        // The button shows the action it performs: while previewing it offers
        // Edit (pen); while editing it offers Preview (eye).
        let toggle_icon = if is_editing {
            IconName::Eye
        } else {
            IconName::SquarePen
        };

        let body = if is_editing {
            // The code editor must fill the remaining height. Wrap in a flex
            // container that can shrink (min_h_0) so the Input's h_full resolves.
            div()
                .flex_1()
                .min_h_0()
                .child(
                    Input::new(&self.docs_input)
                        .h_full()
                        .p_0()
                        .border_0()
                        .focus_bordered(false)
                        .rounded_none(),
                )
                .into_any_element()
        } else {
            let md = self.docs_input.read(cx).value().to_string();
            let empty = md.trim().is_empty();
            if empty {
                div()
                    .flex_1()
                    .min_h_0()
                    .p_3()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child("No docs yet. Click the edit button to write Markdown documentation for this collection.")
                    .into_any_element()
            } else {
                div()
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scrollbar()
                    .p_3()
                    .child(text::markdown(md).selectable(true))
                    .into_any_element()
            }
        };

        // Floating edit/preview toggle in the top-right corner, overlaid on the
        // content instead of a dedicated header row.
        let toggle = div()
            .absolute()
            .top_2()
            .right_2()
            .rounded_md()
            .bg(cx.theme().background)
            .child(
                Button::new("toggle-docs-edit")
                    .small()
                    .ghost()
                    .icon(Icon::new(toggle_icon).size(px(14.)))
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.docs_editing = !this.docs_editing;
                        cx.notify();
                    })),
            );

        v_flex()
            .flex_1()
            .min_h_0()
            .relative()
            .child(body)
            .child(toggle)
    }

    fn render_environments_tab(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().flex_1().child(self.environment_editor.clone())
    }

    fn render_tab_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        TabBar::new("collection-tabs")
            .left(px(-1.)) // Avoid double border
            .selected_index(self.active_tab)
            .on_click(cx.listener(|this, _ix: &usize, _window, cx| {
                this.set_active_tab(*_ix, cx);
            }))
            .children(vec![
                Tab::new().label("Collection"),
                Tab::new().label("Environments"),
                Tab::new().label("Vars"),
                Tab::new().label("Auth"),
            ])
    }

    fn render_tab_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div().flex_1().size_full().child(match self.active_tab {
            0 => {
                let content = self.render_collection_tab(cx);
                div().size_full().child(content)
            }
            1 => {
                let content = self.render_environments_tab(cx);
                div().child(content)
            }
            2 => div().size_full().child(self.render_vars_tab(cx)),
            3 => div().size_full().child(self.auth_editor.clone()),
            _ => {
                let content = self.render_collection_tab(cx);
                div().child(content)
            }
        })
    }

    /// Vars tab: editable collection-level variables (top) + read-only runtime
    /// variable inspector (bottom) for this collection.
    fn render_vars_tab(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(
                v_flex()
                    .flex_1()
                    .min_h_0()
                    .child(div().flex_1().min_h_0().child(self.vars_editor.clone())),
            )
            .child(
                v_flex()
                    .h(px(220.))
                    .border_t_1()
                    .border_color(_cx.theme().border)
                    .child(self.vars_view.clone()),
            )
    }

    // Event handlers
    fn handle_save_collection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let collection_data = self.get_collection_data_for_save(cx);

        // Get current path from input
        let current_path = self.path_input.read(cx).value().to_string();

        // Save collection to file
        if current_path.is_empty() {
            window.push_notification(
                (
                    NotificationType::Error,
                    "Can't save collection, no path specified",
                ),
                cx,
            );
            return;
        }

        let format = if self.use_opencollection {
            super::CollectionFormat::OpenCollection
        } else {
            super::CollectionFormat::Broquest
        };

        // Save to database first to get proper ID
        let app_database = AppDatabase::global(cx).clone();
        let collection_data_clone = collection_data.clone();
        let current_path_clone = current_path.clone();

        cx.spawn(async move |_, _| {
            app_database
                .save_collection(&CollectionData {
                    id: None,
                    name: collection_data_clone.collection.name.clone(),
                    path: current_path_clone.clone(),
                    position: 2,
                    format: format.as_db_str().to_string(),
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })
                .await
        })
        .detach();

        let use_opencollection = self.use_opencollection;
        let manager = CollectionManager::global(cx);
        let save_result = manager.update(cx, |collection_manager, cx| {
            // If the collection already exists in the manager, save in its
            // current format; otherwise create it in the chosen format.
            if collection_manager
                .get_collection_by_path(&current_path)
                .is_none()
                && use_opencollection
            {
                collection_manager.create_opencollection(&collection_data, &current_path, cx)
            } else {
                collection_manager.save_collection(&collection_data, &current_path, cx)
            }
        });

        match save_result {
            Ok(()) => {
                // Update stored state so subsequent saves use fresh data
                self.collection_path = current_path.clone();
                self.collection_data = collection_data.clone();

                tracing::info!("Collection saved successfully to: {}", current_path);
                tracing::info!("Collection name: {}", collection_data.collection.name);

                // Import from the selected spec source, if any. The selection
                // is reset afterwards so subsequent saves don't re-run the
                // import (which would duplicate the imported environment).
                match self.selected_import(cx) {
                    ImportKind::OpenApi => {
                        if let Some(spec_path_value) = self.openapi_spec_path.clone() {
                            self.import_from_openapi(&spec_path_value, &current_path, window, cx);
                        }
                        self.reset_import_selection(window, cx);
                    }
                    ImportKind::Wsdl => {
                        let wsdl_url = self.wsdl_spec_input.read(cx).value().to_string();
                        if !wsdl_url.is_empty() {
                            self.import_from_wsdl_url(&wsdl_url, &current_path, window, cx);
                        }
                        self.reset_import_selection(window, cx);
                    }
                    ImportKind::None => {}
                }

                // Show success notification
                window.push_notification(
                    (NotificationType::Success, "Collection saved successfully."),
                    cx,
                );
            }
            Err(e) => {
                tracing::error!("Failed to save collection: {}", e);
                // TODO: Show error to user

                window
                    .push_notification((NotificationType::Error, "Failed to save collection."), cx);
            }
        }

        // Save secrets
        if let Err(e) = self.save_secrets(cx) {
            tracing::error!("Failed to save secrets: {}", e);
        } else {
            tracing::info!("Secrets saved successfully");
        }
    }

    /// Clear the import selection and spec inputs after an import has been
    /// triggered, so saving the collection again doesn't re-run the import.
    fn reset_import_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.openapi_spec_path = None;
        self.openapi_spec_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.wsdl_spec_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.import_select.update(cx, |select, cx| {
            select.set_selected_index(Some(IndexPath::default().row(0)), window, cx);
        });
        cx.notify();
    }

    fn handle_browse_directory(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let path = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Select a directory for the collection".into()),
        });

        let directory_input = self.path_input.clone();
        cx.spawn_in(window, async move |_, window| {
            if let Some(path) = path.await.ok()?.ok()?
                && let Some(dir_path) = path.first()
                && let Some(dir_str) = dir_path.to_str()
            {
                window
                    .update(|window, cx| {
                        directory_input.update(cx, |input, cx| {
                            input.set_value(dir_str.to_string(), window, cx);
                        });
                    })
                    .ok();
            }
            Some(())
        })
        .detach();
    }

    fn handle_browse_spec_file(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let spec_input = self.openapi_spec_input.clone();
        let path = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Select OpenAPI spec file".into()),
        });

        cx.spawn_in(window, async move |entity, window| {
            if let Some(path) = path.await.ok()?.ok()?
                && let Some(file_path) = path.first()
                && let Some(file_str) = file_path.to_str()
            {
                let file_str = file_str.to_string();
                window
                    .update(|window, cx| {
                        spec_input.update(cx, |input, cx| {
                            input.set_value(file_str.clone(), window, cx);
                        });
                        // Update the parent struct's stored path via entity update
                        let _ = entity
                            .update(cx, |this: &mut Self, cx| {
                                this.openapi_spec_path = Some(file_str);
                                cx.notify();
                            })
                            .log_err();
                    })
                    .ok();
            }
            Some(())
        })
        .detach();
    }

    fn import_from_wsdl_url(
        &mut self,
        wsdl_url: &str,
        collection_path: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let wsdl_url = wsdl_url.to_string();
        let collection_path = collection_path.to_string();
        let manager = CollectionManager::global(cx);

        cx.spawn_in(window, async move |entity, window| {
            match super::wsdl::import_from_wsdl_url(&wsdl_url).await {
                Ok(result) => {
                    match manager.update(window, |collection_manager, cx| {
                        if let Err(e) = collection_manager.add_environment_to_collection(
                            &collection_path,
                            result.environment,
                            cx,
                        ) {
                            tracing::error!("Failed to add environment to collection: {}", e);
                        }

                        for (group_name, requests) in result.groups {
                            if let Err(e) =
                                collection_manager.create_group(&collection_path, &group_name, cx)
                            {
                                tracing::error!("Failed to create group '{}': {}", group_name, e);
                            }

                            for request in requests {
                                let request_name = request.name.clone();
                                if let Err(e) = collection_manager.save_request(
                                    &collection_path,
                                    &request,
                                    &request_name,
                                    Some(&group_name),
                                    cx,
                                ) {
                                    tracing::error!(
                                        "Failed to save request '{}': {}",
                                        request_name,
                                        e
                                    );
                                }
                            }
                        }

                        for request in result.requests {
                            let request_name = request.name.clone();
                            if let Err(e) = collection_manager.save_request(
                                &collection_path,
                                &request,
                                &request_name,
                                None,
                                cx,
                            ) {
                                tracing::error!("Failed to save request '{}': {}", request_name, e);
                            }
                        }
                        Ok::<(), anyhow::Error>(())
                    }) {
                        Ok(()) => {
                            window
                                .update(|window, cx| {
                                    window.push_notification(
                                        (NotificationType::Success, "WSDL imported successfully."),
                                        cx,
                                    );
                                    let _ = entity
                                        .update(cx, |this, cx| {
                                            this.reload_environments(&collection_path, window, cx);
                                        })
                                        .log_err();
                                })
                                .ok();
                        }
                        _ => {
                            tracing::error!("Failed to update global state during WSDL import");
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to import WSDL: {}", e);
                    window
                        .update(|window, cx| {
                            window.push_notification(
                                (NotificationType::Error, "Failed to import WSDL."),
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

    fn import_from_openapi(
        &mut self,
        spec_path: &str,
        collection_path: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let spec_path = spec_path.to_string();
        let collection_path = collection_path.to_string();
        let manager = CollectionManager::global(cx);

        cx.spawn_in(window, async move |entity, window| {
            match OpenAPIImporter::from_path(&spec_path) {
                Ok(importer) => {
                    match importer.import() {
                        Ok(result) => {
                            // Import is successful, now update the collection manager
                            match manager.update(window, |collection_manager, cx| {
                                // First, create the Default environment with baseUrl variable
                                // We need to update the collection to include this environment
                                if let Err(e) = collection_manager.add_environment_to_collection(
                                    &collection_path,
                                    result.environment,
                                    cx,
                                ) {
                                    tracing::error!(
                                        "Failed to add environment to collection: {}",
                                        e
                                    );
                                }

                                // Create groups and requests
                                for (group_name, requests) in result.groups {
                                    if let Err(e) = collection_manager.create_group(
                                        &collection_path,
                                        &group_name,
                                        cx,
                                    ) {
                                        tracing::error!(
                                            "Failed to create group '{}': {}",
                                            group_name,
                                            e
                                        );
                                    }

                                    for request in requests {
                                        let request_name = request.name.clone();
                                        if let Err(e) = collection_manager.save_request(
                                            &collection_path,
                                            &request,
                                            &request_name,
                                            Some(&group_name),
                                            cx,
                                        ) {
                                            tracing::error!(
                                                "Failed to save request '{}': {}",
                                                request_name,
                                                e
                                            );
                                        }
                                    }
                                }

                                // Add root-level requests
                                for request in result.requests {
                                    let request_name = request.name.clone();
                                    if let Err(e) = collection_manager.save_request(
                                        &collection_path,
                                        &request,
                                        &request_name,
                                        None,
                                        cx,
                                    ) {
                                        tracing::error!(
                                            "Failed to save request '{}': {}",
                                            request_name,
                                            e
                                        );
                                    }
                                }
                                Ok::<(), anyhow::Error>(())
                            }) {
                                Ok(()) => {
                                    window
                                        .update(|window, cx| {
                                            window.push_notification(
                                                (
                                                    NotificationType::Success,
                                                    "OpenAPI spec imported successfully.",
                                                ),
                                                cx,
                                            );
                                            let _ = entity
                                                .update(cx, |this, cx| {
                                                    this.reload_environments(
                                                        &collection_path,
                                                        window,
                                                        cx,
                                                    );
                                                })
                                                .log_err();
                                        })
                                        .ok();
                                }
                                _ => {
                                    tracing::error!("Failed to update global state during import");
                                }
                            }
                        }
                        Err(e) => {
                            tracing::error!("Failed to import OpenAPI spec: {}", e);
                            window
                                .update(|window, cx| {
                                    window.push_notification(
                                        (NotificationType::Error, "Failed to import OpenAPI spec."),
                                        cx,
                                    );
                                })
                                .ok();
                        }
                    }
                }
                Err(e) => {
                    tracing::error!("Failed to parse OpenAPI spec: {}", e);
                    window
                        .update(|window, cx| {
                            window.push_notification(
                                (
                                    NotificationType::Error,
                                    "Failed to parse OpenAPI spec file.",
                                ),
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

    /// Get the name input entity for external subscriptions
    pub fn name_input(&self) -> &Entity<InputState> {
        &self.name_input
    }
}

impl Focusable for CollectionEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for CollectionEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .track_focus(&self.focus_handle)
            .key_context(CONTEXT)
            .on_action(
                cx.listener(|this: &mut CollectionEditor, &Save, window, cx| {
                    this.handle_save_collection(window, cx);
                }),
            )
            .on_action(
                cx.listener(|this: &mut CollectionEditor, &ToggleDocsEdit, window, cx| {
                    this.on_toggle_docs_edit(&ToggleDocsEdit, window, cx);
                }),
            )
            .flex_1()
            .size_full()
            .child(
                // Tab bar
                self.render_tab_bar(cx),
            )
            .child(
                // Tab content
                self.render_tab_content(cx),
            )
            .child(
                // Save button at the bottom
                h_flex()
                    .gap_2()
                    .p_3()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .justify_end()
                    .child(
                        Button::new("save_collection_bottom")
                            .primary()
                            .compact()
                            .icon(IconName::Save)
                            .label("Save Collection")
                            .children(vec![
                                Kbd::new(crate::ui::keybinding::parse_keystroke("secondary-s"))
                                    .into_any_element(),
                            ])
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.handle_save_collection(window, cx)
                            })),
                    ),
            )
    }
}
