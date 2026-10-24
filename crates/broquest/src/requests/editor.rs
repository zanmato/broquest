use crate::ui::resizable::{ResizableState, h_resizable, resizable_panel, v_resizable};
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, ImageSource,
    InteractiveElement as _, IntoElement, ObjectFit, ParentElement as _, Render, SharedString,
    StyleRefinement, Styled as _, StyledImage as _, Subscription, Task, Window, actions, div, img,
    prelude::FluentBuilder, px,
};
use gpui_kit::component::{
    ActiveTheme, Icon, IndexPath, Sizable, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    dialog::{DialogAction, DialogClose, DialogFooter},
    h_flex,
    input::{Editor, EditorState, Input, InputEvent, InputState},
    kbd::Kbd,
    notification::NotificationType,
    scroll::ScrollableElement,
    select::{Select, SelectEvent, SelectItem, SelectState},
    tab::{Tab, TabBar},
    v_flex,
};
use jsonpath_rust::JsonPath;
use std::collections::HashMap;
use std::sync::Arc;

use super::auth_editor::{AuthEditor, AuthEditorEvent};
use super::key_value_editor::{KeyValueConfig, KeyValueEditor, KeyValueEvent};
use crate::app_settings::AppSettings;
use crate::domain::{
    AuthType, ContentType, HttpFileOptions, HttpMethod, KeyValuePair, Prompt, RequestData,
    ResponseData,
};
use crate::http::ResponseFormat;
use crate::http::curl;
use crate::result_ext::ResultExt;
use crate::scripting::{ScriptEditor, ScriptEditorEvent};
use crate::settings::EditorLayout;
use crate::ui::icon::IconName;
use crate::ui::tab_badge::TabBadge;
use crate::{app_events::AppEvent, environments::EnvironmentResolver};
use crate::{
    collections::{CollectionManager, EnvironmentSettings, EnvironmentToml},
    http::HttpClientService,
};
use std::time::Duration;

/// The key context of the request editor, where its shortcuts apply.
pub(crate) const CONTEXT: &str = "request_editor";

actions!(request_editor, [Save, Send, CopyResponse, FormatBody]);

/// Percent-encode text for a URL or form body. `{{variables}}` stay as they
/// are, so they can still be resolved.
fn url_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut rest = input;
    while let Some(start) = rest.find("{{") {
        let Some(end) = rest[start..].find("}}") else {
            break;
        };
        out.push_str(&urlencoding::encode(&rest[..start]));
        out.push_str(&rest[start..start + end + 2]);
        rest = &rest[start + end + 2..];
    }
    out.push_str(&urlencoding::encode(rest));
    out
}

/// Decode percent-encoded text, with `+` as a space.
fn url_decode(input: &str) -> String {
    let spaced = input.replace('+', " ");
    urlencoding::decode(&spaced)
        .map(|text| text.into_owned())
        .unwrap_or(spaced)
}

/// The fields of a urlencoded form body. `name=@path` file fields keep
/// their path as written.
fn parse_form_body(body: &str) -> Vec<KeyValuePair> {
    body.split('&')
        .filter(|pair| !pair.trim().is_empty())
        .map(|pair| {
            let (key, value) = pair.split_once('=').unwrap_or((pair, ""));
            let value = if value.starts_with('@') {
                value.to_string()
            } else {
                url_decode(value)
            };
            KeyValuePair {
                key: url_decode(key),
                value,
                enabled: true,
            }
        })
        .collect()
}

/// A urlencoded form body from its fields. File fields stay `name=@path`.
fn form_body(fields: &[KeyValuePair]) -> String {
    fields
        .iter()
        .filter(|field| field.enabled && !field.key.is_empty())
        .map(|field| {
            if field.value.starts_with('@') {
                format!("{}={}", field.key, field.value)
            } else {
                format!("{}={}", url_encode(&field.key), url_encode(&field.value))
            }
        })
        .collect::<Vec<_>>()
        .join("&")
}

/// Which environment a request tab's dropdown selects, and whether it has a
/// "No environment" choice.
///
/// `previous` is the selection before the environments were loaded again,
/// `Some(None)` for "No environment", or `None` for a new tab. The returned
/// index is into `environments`, `None` meaning "No environment".
fn pick_environment(
    environments: &[EnvironmentToml],
    settings: &EnvironmentSettings,
    previous: Option<Option<String>>,
) -> (bool, Option<usize>) {
    let position = |name: &str| environments.iter().position(|env| env.name == name);
    let default = settings.default_environment.as_deref().and_then(position);
    let selected = match previous {
        // A new tab, or one whose environment is gone, gets the default.
        None => default,
        Some(Some(name)) => position(&name).or(default),
        Some(None) => None,
    };
    // A collection without environments still needs one choice.
    let allows_none = !settings.require_environment || environments.is_empty();
    if allows_none {
        (true, selected)
    } else {
        (false, selected.or(default).or(Some(0)))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum EnvironmentOption {
    None,
    Environment(EnvironmentToml),
}

impl EnvironmentOption {
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        match self {
            EnvironmentOption::None => "No environment",
            EnvironmentOption::Environment(env) => &env.name,
        }
    }
}

impl SelectItem for EnvironmentOption {
    type Value = EnvironmentOption;

    fn title(&self) -> SharedString {
        match self {
            EnvironmentOption::None => "No environment".into(),
            EnvironmentOption::Environment(env) => env.name.clone().into(),
        }
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestTab {
    Path,
    Query,
    Body,
    Headers,
    Auth,
    Scripts,
    Vars,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseTab {
    Response,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestEditorEvent {
    DirtyStateChanged { is_dirty: bool },
    EnvironmentChanged,
}

pub struct RequestEditor {
    _focus_handle: FocusHandle,
    request_data: RequestData,
    original_request_data: Option<RequestData>,
    response_data: ResponseData,
    active_tab: RequestTab,
    active_response_tab: ResponseTab,
    is_loading: bool,
    current_request_task: Option<Task<Result<(), anyhow::Error>>>,
    /// The last `# @prompt` values entered, to fill the next prompt.
    prompt_values: HashMap<String, String>,
    collection_path: Option<String>,
    group_path: Option<String>,
    method_select: Entity<SelectState<Vec<HttpMethod>>>,
    environment_select: Entity<SelectState<Vec<EnvironmentOption>>>,
    /// Whether the environment dropdown has the "No environment" row.
    allows_no_environment: bool,
    /// Whether the collection's environments were loaded into the dropdown.
    environments_loaded: bool,
    content_type_select: Entity<SelectState<Vec<ContentType>>>,
    name_input: Entity<InputState>,
    /// A URL bar with tree-sitter highlighting: a code editor laid out as one line.
    url_input: Entity<EditorState>,
    body_input: Entity<EditorState>,
    response_input: Entity<EditorState>,
    raw_response_input: Entity<EditorState>,
    path_param_editor: Entity<KeyValueEditor>,
    query_param_editor: Entity<KeyValueEditor>,
    header_editor: Entity<KeyValueEditor>,
    form_editor: Entity<KeyValueEditor>,
    auth_editor: Entity<AuthEditor>,
    script_editor: Entity<ScriptEditor>,
    /// Editable request-level variables (Bruno `runtime.variables`).
    request_vars_editor: Entity<KeyValueEditor>,
    /// Read-only variable inspector scoped to the owning collection.
    vars_view: Entity<crate::collections::VarsView>,
    _subscriptions: Vec<Subscription>,
    _updating_url_from_params: bool,
    _was_dirty: bool,
    // JSONPath filtering
    jsonpath_input: Entity<InputState>,
    original_response_body: Option<String>,
    _jsonpath_filter_task: Task<()>,
    response_format: ResponseFormat,
    response_image: Option<Arc<gpui::Image>>,
    // Resizable panels
    request_response_state: Entity<ResizableState>,
    layout: EditorLayout,
    url_bar_style_refinement: StyleRefinement,
}

impl RequestEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let method_select = cx.new(|cx| {
            SelectState::new(
                HttpMethod::ALL.to_vec(),
                Some(IndexPath::default().row(0)), // Select GET by default
                window,
                cx,
            )
        });

        let environment_select = cx.new(|cx| {
            SelectState::new(
                vec![EnvironmentOption::None],     // Start with just None option
                Some(IndexPath::default().row(0)), // Select None by default
                window,
                cx,
            )
        });

        let content_type_select = cx.new(|cx| {
            SelectState::new(
                ContentType::ALL.to_vec(),
                Some(IndexPath::default().row(0)), // Select JSON by default
                window,
                cx,
            )
        });

        let url_input = cx.new(|cx| {
            EditorState::new(window, cx)
                .language("url")
                .single_line(true)
                .placeholder("Enter request URL")
        });

        let name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Request name")
                .default_value("New Request")
        });

        let editor_settings = AppSettings::global(cx).settings.editor.clone();

        let body_input = cx.new(|cx| {
            let mut editor = EditorState::new(window, cx)
                .language("json")
                .folding(editor_settings.folding)
                .show_whitespaces(editor_settings.show_whitespace);
            editor.set_soft_wrap(editor_settings.soft_wrap, window, cx);
            editor
        });

        let response_input = cx.new(|cx| {
            let mut editor = EditorState::new(window, cx)
                .language("text")
                .folding(editor_settings.folding)
                .show_whitespaces(editor_settings.show_whitespace);
            editor.set_soft_wrap(editor_settings.soft_wrap, window, cx);
            editor
        });

        let raw_response_input = cx.new(|cx| {
            let mut editor = EditorState::new(window, cx)
                .language("text")
                .folding(editor_settings.folding)
                .show_whitespaces(editor_settings.show_whitespace);
            editor.set_soft_wrap(editor_settings.soft_wrap, window, cx);
            editor
        });

        let path_param_editor = cx.new(|cx| {
            KeyValueEditor::new(
                KeyValueConfig::new("path-params", "Parameter name", "Parameter value"),
                window,
                cx,
            )
        });

        let query_param_editor = cx.new(|cx| {
            KeyValueEditor::new(
                KeyValueConfig::new("query-params", "Parameter name", "Parameter value"),
                window,
                cx,
            )
        });

        let request_vars_editor = cx.new(|cx| {
            KeyValueEditor::new(
                KeyValueConfig::new("request-vars", "Variable name", "Variable value").embedded(),
                window,
                cx,
            )
        });

        let header_editor = cx.new(|cx| {
            KeyValueEditor::new(
                KeyValueConfig::new("headers", "Header name", "Header value"),
                window,
                cx,
            )
        });

        let form_editor = cx.new(|cx| {
            KeyValueEditor::new(
                KeyValueConfig::new("form-fields", "Field name", "Field value or @/path/to/file")
                    .with_file_picker(),
                window,
                cx,
            )
        });

        let script_editor = cx.new(|cx| ScriptEditor::new(window, cx));

        // Read-only variable inspector (scoped later via set_collection_path).
        // Embedded so it shares the Vars tab's scroll region with the editor.
        let vars_view = cx.new(|cx| {
            let mut view = crate::collections::VarsView::new(cx);
            view.set_embedded(true);
            view
        });

        let auth_editor = cx.new(|cx| AuthEditor::new(window, cx));

        let jsonpath_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("$.store.book[*].author"));

        let request_response_state = cx.new(|_cx| ResizableState::default());

        let mut subscriptions = Vec::new();
        // Subscribe to Content-Type selection changes
        subscriptions.push(cx.subscribe(
            &content_type_select,
            |this, _state, _event: &SelectEvent<Vec<ContentType>>, cx| {
                this.on_content_type_change(cx);
            },
        ));

        // Subscribe to environment selection changes
        subscriptions.push(cx.subscribe(
            &environment_select,
            |this, _state, _event: &SelectEvent<Vec<EnvironmentOption>>, cx| {
                this.on_environment_change(cx);
            },
        ));

        Self {
            _focus_handle: cx.focus_handle(),
            request_data: RequestData::default(),
            original_request_data: None,
            response_data: ResponseData::default(),
            active_tab: RequestTab::Query,
            active_response_tab: ResponseTab::Response,
            is_loading: false,
            current_request_task: None,
            prompt_values: HashMap::new(),
            collection_path: None,
            group_path: None,
            method_select,
            environment_select,
            allows_no_environment: true,
            environments_loaded: false,
            content_type_select,
            name_input,
            url_input,
            body_input,
            response_input,
            raw_response_input,
            path_param_editor,
            query_param_editor,
            header_editor,
            form_editor,
            auth_editor,
            script_editor,
            request_vars_editor,
            vars_view,
            _subscriptions: subscriptions,
            _updating_url_from_params: false,
            _was_dirty: false,
            jsonpath_input,
            original_response_body: None,
            _jsonpath_filter_task: Task::ready(()),
            response_format: ResponseFormat::Unknown,
            response_image: None,
            request_response_state,
            layout: AppSettings::global(cx).settings.editor.layout,
            url_bar_style_refinement: StyleRefinement {
                flex_grow: Some(4.0),
                flex_shrink: Some(1.0),
                ..Default::default()
            },
        }
    }

    /// Show the request tab labelled `label`.
    #[cfg(feature = "screenshots")]
    pub(crate) fn show_request_tab(&mut self, label: &str, cx: &mut Context<Self>) {
        self.active_tab = match label {
            "Path" => RequestTab::Path,
            "Body" => RequestTab::Body,
            "Headers" => RequestTab::Headers,
            "Auth" => RequestTab::Auth,
            "Scripts" => RequestTab::Scripts,
            "Vars" => RequestTab::Vars,
            _ => RequestTab::Query,
        };
        cx.notify();
    }

    /// Resize the request pane, above the response or beside it.
    #[cfg(feature = "screenshots")]
    pub(crate) fn resize_request_pane(
        &mut self,
        size: gpui::Pixels,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.request_response_state.update(cx, |state, cx| {
            state.resize_panel(0, size, window, cx);
        });
    }

    #[cfg(feature = "screenshots")]
    pub(crate) fn is_loading(&self) -> bool {
        self.is_loading
    }

    /// Type `query` into the JSONPath filter under the response.
    #[cfg(feature = "screenshots")]
    pub(crate) fn filter_response(
        &mut self,
        query: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.jsonpath_input.update(cx, |input, cx| {
            input.set_value(query.to_string(), window, cx);
        });
        self.handle_jsonpath_input_change(window, cx);
    }

    pub fn set_collection_path(&mut self, collection_path: Option<String>, cx: &mut Context<Self>) {
        self.collection_path = collection_path.clone();
        // Scope the read-only Vars view to this collection so it shows the right
        // collection + runtime variables.
        self.vars_view.update(cx, |view, cx| {
            view.set_collection(collection_path, cx);
        });
    }

    pub fn set_group_path(&mut self, group_path: Option<String>) {
        self.group_path = group_path;
    }

    /// Treat the request as not saved yet, as for a copy of another request,
    /// so it shows as changed and closing it asks to save.
    pub fn mark_unsaved(&mut self, cx: &mut Context<Self>) {
        // Saved data under no name never matches the request's own.
        let mut original = self.get_request_data(cx);
        original.name.clear();
        self.original_request_data = Some(original);
        self.recompute_dirty(cx);
    }

    pub fn collection_path(&self) -> Option<&str> {
        self.collection_path.as_deref()
    }

    pub fn group_path(&self) -> Option<&str> {
        self.group_path.as_deref()
    }

    /// The request's name as last saved, which is how it is found on disk.
    pub fn saved_name(&self) -> Option<&str> {
        self.original_request_data
            .as_ref()
            .map(|data| data.name.as_str())
    }

    pub fn apply_editor_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let settings = AppSettings::global(cx).settings.editor.clone();
        for input in [
            &self.body_input,
            &self.response_input,
            &self.raw_response_input,
        ] {
            input.update(cx, |state, cx| {
                state.set_show_whitespaces(settings.show_whitespace, window, cx);
                state.set_soft_wrap(settings.soft_wrap, window, cx);
                state.set_folding(settings.folding, window, cx);
            });
        }
        self.script_editor.update(cx, |editor, cx| {
            editor.apply_editor_settings(window, cx);
        });
        if self.layout != settings.layout {
            self.layout = settings.layout;
            self.request_response_state = cx.new(|_cx| ResizableState::default());
        }
    }

    pub fn set_request_data(
        &mut self,
        data: RequestData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.original_request_data = Some(data.clone());
        self.request_data = data.clone();
        self._was_dirty = false; // Reset dirty state when loading saved data

        // Update method selector
        self.method_select.update(cx, |state, cx| {
            if let Some(index) = HttpMethod::ALL
                .iter()
                .position(|&method| method == data.method)
            {
                state.set_selected_index(Some(IndexPath::default().row(index)), window, cx);
            }
        });

        // Update URL input
        self.url_input.update(cx, |state, cx| {
            let url = self.build_url_with_query_params(&data.url, &data.query_params);
            state.set_value(url, window, cx);
        });

        // Update name input
        self.name_input.update(cx, |state, cx| {
            state.set_value(data.name.clone(), window, cx);
        });

        // Update path parameters
        self.path_param_editor.update(cx, |editor, cx| {
            editor.set_pairs(&data.path_params, window, cx);
        });

        // Update body input
        self.body_input.update(cx, |state, cx| {
            state.set_value(data.body.clone(), window, cx);
        });

        // Update query parameters
        self.query_param_editor.update(cx, |editor, cx| {
            editor.set_pairs(&data.query_params, window, cx);
        });

        // Update request variables
        self.request_vars_editor.update(cx, |editor, cx| {
            editor.set_pairs(&data.vars, window, cx);
        });

        // Update headers
        self.header_editor.update(cx, |editor, cx| {
            editor.set_pairs(&data.headers, window, cx);
        });

        // Update content type selector based on Content-Type header
        let content_type = data
            .headers
            .iter()
            .find(|h| h.key.to_lowercase() == "content-type" && h.enabled)
            .map(|h| ContentType::from_header(&h.value))
            .unwrap_or(ContentType::Json);
        let content_type_index = ContentType::ALL
            .iter()
            .position(|ct| *ct == content_type)
            .unwrap_or(0);
        self.content_type_select.update(cx, |state, cx| {
            state.set_selected_index(
                Some(IndexPath::default().row(content_type_index)),
                window,
                cx,
            );
        });

        self.on_content_type_change(cx);

        // A urlencoded body shows as form fields. Requests from .http files
        // keep their body as written.
        if content_type == ContentType::Form && data.http_file.is_none() {
            let fields = parse_form_body(&data.body);
            self.form_editor.update(cx, |editor, cx| {
                editor.set_pairs(&fields, window, cx);
            });
        }

        // Update scripts
        self.script_editor.update(cx, |editor, cx| {
            editor.set_scripts(
                data.pre_request_script.as_deref(),
                data.post_response_script.as_deref(),
                window,
                cx,
            );
        });

        // Update auth
        self.auth_editor.update(cx, |editor, cx| {
            editor.set_auth(&data.auth, window, cx);
        });

        cx.notify();
    }

    /// Fill the environment dropdown with the collection's environments.
    ///
    /// A new tab selects the collection's default environment. Later calls,
    /// after the environments changed elsewhere, keep the current selection
    /// when it still exists. When the collection requires an environment the
    /// "No environment" choice is left out.
    pub fn set_environments(
        &mut self,
        environments: &[EnvironmentToml],
        settings: &EnvironmentSettings,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let previous = self
            .environments_loaded
            .then(|| self.selected_environment_name(cx));
        let (allows_none, selected) = pick_environment(environments, settings, previous);

        let mut env_options = Vec::with_capacity(environments.len() + 1);
        if allows_none {
            env_options.push(EnvironmentOption::None);
        }
        env_options.extend(
            environments
                .iter()
                .cloned()
                .map(EnvironmentOption::Environment),
        );

        self.allows_no_environment = allows_none;
        self.environments_loaded = true;
        let row = selected.map_or(0, |index| self.environment_row(index));

        // Mutate the existing SelectState (rather than replacing the entity) so
        // the selection-change subscription set up in `new` stays intact.
        self.environment_select.update(cx, |state, cx| {
            state.set_items(env_options, window, cx);
            state.set_selected_index(Some(IndexPath::default().row(row)), window, cx);
        });

        cx.notify();
    }

    /// The dropdown row of the environment at `index`, after the "No
    /// environment" row when there is one.
    fn environment_row(&self, index: usize) -> usize {
        index + usize::from(self.allows_no_environment)
    }

    /// Name of the selected environment, if one is selected.
    pub fn selected_environment_name(&self, cx: &App) -> Option<String> {
        match self.environment_select.read(cx).selected_value()? {
            EnvironmentOption::None => None,
            EnvironmentOption::Environment(env) => Some(env.name.clone()),
        }
    }

    /// Select the environment called `name`. Does nothing when the request's
    /// collection has no environment by that name.
    pub fn select_environment(&mut self, name: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(collection_path) = self.collection_path.as_deref() else {
            return;
        };
        let row = CollectionManager::global(cx)
            .read(cx)
            .get_collection_environments(collection_path)
            .and_then(|envs| envs.iter().position(|env| env.name == name))
            .map(|index| self.environment_row(index));
        if let Some(row) = row {
            self.environment_select.update(cx, |state, cx| {
                state.set_selected_index(Some(IndexPath::default().row(row)), window, cx);
            });
            cx.notify();
        }
    }

    pub fn get_selected_environment(&self, cx: &Context<Self>) -> Option<EnvironmentToml> {
        if let Some(selected_env) = self.environment_select.read(cx).selected_value() {
            match selected_env {
                EnvironmentOption::None => None,
                EnvironmentOption::Environment(env) => Some(env.clone()),
            }
        } else {
            None
        }
    }

    /// Strip query parameters from URL, returning base URL and extracted parameters
    fn strip_query_params_from_url(url: &str) -> (String, Vec<KeyValuePair>) {
        crate::domain::split_query(url)
    }

    pub fn get_request_data(&self, cx: &Context<Self>) -> RequestData {
        let mut data = self.edited_request_data(cx);

        // Set Content-Type header from the dropdown only if the user hasn't set one
        // in the headers editor. A request from a .http file sends the headers
        // its file has.
        if data.http_file.is_none()
            && let Some(selected_content_type) = self.content_type_select.read(cx).selected_value()
        {
            let has_content_type = data
                .headers
                .iter()
                .any(|h| h.key.to_lowercase() == "content-type" && h.enabled);

            if !has_content_type {
                data.headers.push(KeyValuePair {
                    key: "Content-Type".to_string(),
                    value: selected_content_type.as_str().to_string(),
                    enabled: true,
                });
            }
        }
        data
    }

    /// The request as edited, without the Content-Type header the body type
    /// dropdown adds when the request is sent or saved. A copy of the request
    /// starts from this, so it shows the same headers as the original.
    pub fn edited_request_data(&self, cx: &Context<Self>) -> RequestData {
        let mut data = self.request_data.clone();

        // Update method from select
        if let Some(selected_method) = self.method_select.read(cx).selected_value() {
            data.method = *selected_method;
        }

        // Update URL from input and strip query parameters
        let raw_url = self.url_input.read(cx).value().to_string();
        let (base_url, url_query_params) = Self::strip_query_params_from_url(&raw_url);
        data.url = base_url;

        // Update name from input
        data.name = self.name_input.read(cx).value().to_string();

        let path_params = self
            .path_param_editor
            .read_with(cx, |editor, cx| editor.get_pairs(cx));

        let editor_query_params = self
            .query_param_editor
            .read_with(cx, |editor, cx| editor.get_pairs(cx));

        // Merge URL query params with editor query params
        // Editor params take precedence, but preserve disabled state for URL params not in editor
        let mut query_params = editor_query_params.clone();

        // Add URL parameters that aren't already in the editor (as disabled)
        for url_param in url_query_params {
            if !query_params.iter().any(|p| p.key == url_param.key) {
                query_params.push(KeyValuePair {
                    key: url_param.key,
                    value: url_param.value,
                    enabled: false, // Mark as disabled since they're not in the editor
                });
            }
        }

        let headers = self
            .header_editor
            .read_with(cx, |editor, cx| editor.get_pairs(cx));

        let (pre_request_script, post_response_script) =
            self.script_editor.read_with(cx, |editor, cx| {
                (
                    editor.get_pre_request_script(cx),
                    editor.get_post_response_script(cx),
                )
            });

        // Update body from input
        data.body = self.body_input.read(cx).value().to_string();

        // The form editor holds a urlencoded body as fields.
        if self.uses_form_editor(cx) {
            data.body = form_body(&self.form_editor.read(cx).get_pairs(cx));
        }

        let vars = self
            .request_vars_editor
            .read_with(cx, |editor, cx| editor.get_pairs(cx));

        data.path_params = path_params;
        data.query_params = query_params;
        data.headers = headers;
        data.pre_request_script = pre_request_script;
        data.post_response_script = post_response_script;
        data.vars = vars;

        // Update auth from auth editor
        data.auth = self.auth_editor.read(cx).get_auth(cx);

        data
    }

    /// Whether the body is edited as form fields: a urlencoded body, unless
    /// the request comes from a .http file, which keeps its body as written.
    fn uses_form_editor(&self, cx: &App) -> bool {
        self.request_data.http_file.is_none()
            && self.content_type_select.read(cx).selected_value() == Some(&ContentType::Form)
    }

    pub fn is_dirty(&self, cx: &Context<Self>) -> bool {
        let Some(original) = &self.original_request_data else {
            return false;
        };

        let current = self.get_request_data(cx);

        // Compare scalar fields
        if original.name != current.name
            || original.method != current.method
            || original.url != current.url
            || original.body != current.body
        {
            return true;
        }

        // Compare script options
        match (&original.pre_request_script, &current.pre_request_script) {
            (None, None) => {}
            (Some(o), Some(c)) if o == c => {}
            _ => return true,
        }

        match (
            &original.post_response_script,
            &current.post_response_script,
        ) {
            (None, None) => {}
            (Some(o), Some(c)) if o == c => {}
            _ => return true,
        }

        // Compare vectors using order-independent comparison
        if !KeyValuePair::vec_equals(&original.path_params, &current.path_params) {
            return true;
        }

        if !KeyValuePair::vec_equals(&original.query_params, &current.query_params) {
            return true;
        }

        if !KeyValuePair::vec_equals(&original.headers, &current.headers) {
            return true;
        }

        // Compare auth
        if original.auth != current.auth {
            return true;
        }

        false
    }

    /// Recompute dirty state and emit a change event if it flipped. Called
    /// synchronously from the child-editor/input subscriptions, which already
    /// fire after their value has settled.
    fn recompute_dirty(&mut self, cx: &mut Context<Self>) {
        let is_now_dirty = self.is_dirty(cx);
        if is_now_dirty != self._was_dirty {
            self._was_dirty = is_now_dirty;
            cx.emit(RequestEditorEvent::DirtyStateChanged {
                is_dirty: is_now_dirty,
            });
        }
    }

    fn on_content_type_change(&mut self, cx: &mut Context<Self>) {
        // Get the selected content type
        let content_type = {
            let content_type_select = self.content_type_select.read(cx);
            content_type_select.selected_value().copied()
        };

        if let Some(content_type) = content_type {
            // Update body input syntax highlighting
            let language = content_type.language();
            self.body_input.update(cx, |input_state, cx| {
                input_state.set_highlighter(language, cx);
                cx.notify();
            });

            // Update request data to include proper Content-Type header
            self.request_data = self.get_request_data(cx);
        }
    }

    fn on_environment_change(&mut self, cx: &mut Context<Self>) {
        // Get the selected environment
        let selected_env = self.get_selected_environment(cx);

        if let Some(env) = &selected_env {
            tracing::info!("Environment changed to: {}", env.name);
        } else {
            tracing::info!("Environment changed to: None");
        }

        cx.emit(RequestEditorEvent::EnvironmentChanged);
        cx.notify();
    }

    fn cancel_request(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Dropping the task aborts the in-flight request.
        self.current_request_task.take();
        self.is_loading = false;

        window.push_notification((NotificationType::Info, "Request cancelled"), cx);
        cx.notify();
    }

    /// Send the request, first asking for `# @prompt` values or confirming
    /// a `# @note` when the request has them.
    pub fn send_request(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_loading {
            return;
        }
        match self
            .request_data
            .http_file
            .clone()
            .filter(|options| options.note.is_some() || !options.prompts.is_empty())
        {
            Some(options) => self.open_send_dialog(&options, window, cx),
            None => self.send_request_with(HashMap::new(), window, cx),
        }
    }

    /// Ask for the request's `# @prompt` values and show its `# @note`, then
    /// send it.
    fn open_send_dialog(
        &mut self,
        options: &HttpFileOptions,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let inputs: Vec<(Prompt, Entity<InputState>)> = options
            .prompts
            .iter()
            .map(|prompt| {
                let last = self
                    .prompt_values
                    .get(&prompt.name)
                    .cloned()
                    .unwrap_or_default();
                let input = cx.new(|cx| {
                    InputState::new(window, cx)
                        .masked(prompt.is_secret())
                        .placeholder(prompt.description.clone().unwrap_or_default())
                });
                input.update(cx, |input, cx| input.set_value(last, window, cx));
                (prompt.clone(), input)
            })
            .collect();
        if let Some((_, first)) = inputs.first() {
            first.update(cx, |input, cx| input.focus(window, cx));
        }

        let note = options.note.clone().map(|note| {
            if note.is_empty() {
                SharedString::from("This request is marked with @note. Send it?")
            } else {
                SharedString::from(note)
            }
        });
        let title = if inputs.is_empty() {
            "Send this request?"
        } else {
            "Values for this request"
        };
        let editor = cx.entity().downgrade();
        window.open_dialog(cx, move |dialog, _, cx| {
            let fields = inputs.iter().map(|(prompt, input)| {
                v_flex()
                    .gap_1()
                    .child(div().text_sm().font_medium().child(prompt.name.clone()))
                    .child(Input::new(input))
            });
            let inputs = inputs.clone();
            let editor = editor.clone();
            dialog
                .title(title)
                .w(px(440.))
                .child(
                    v_flex()
                        .gap_3()
                        .when_some(note.clone(), |this, note| {
                            this.child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(note),
                            )
                        })
                        .children(fields),
                )
                .footer(
                    DialogFooter::new()
                        .child(
                            DialogClose::new()
                                .child(Button::new("cancel-send").label("Cancel").outline()),
                        )
                        .child(
                            DialogAction::new()
                                .child(Button::new("confirm-send").label("Send").primary()),
                        ),
                )
                .on_ok(move |_, window, cx| {
                    let values: HashMap<String, String> = inputs
                        .iter()
                        .map(|(prompt, input)| {
                            (prompt.name.clone(), input.read(cx).value().to_string())
                        })
                        .collect();
                    editor
                        .update(cx, |editor, cx| {
                            editor.prompt_values.extend(values.clone());
                            editor.send_request_with(values, window, cx);
                        })
                        .log_err()
                        .ok();
                    true
                })
        });
    }

    /// Send the request. `prompted` holds `# @prompt` values, which apply as
    /// request variables.
    fn send_request_with(
        &mut self,
        prompted: HashMap<String, String>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.is_loading {
            return;
        }

        // Set loading state
        self.is_loading = true;
        cx.notify();

        // Get the current request data
        let mut request_data = self.get_request_data(cx);

        // The form editor holds a urlencoded body as fields.
        if self.uses_form_editor(cx) {
            request_data.body = form_body(&self.form_editor.read(cx).get_pairs(cx));
        }

        // Perform path parameter replacement on the URL
        let final_url = self
            .path_param_editor
            .read(cx)
            .replace_path_parameters(&request_data.url, cx);

        tracing::info!(
            "Sending request: {} {} (original: {})",
            request_data.method.as_str(),
            final_url,
            request_data.url
        );

        // Create a copy of request data with the final URL
        let mut final_request_data = request_data.clone();
        final_request_data.url = final_url;

        // Values asked for by `# @prompt` win over the request's variables.
        for (name, value) in prompted {
            match final_request_data
                .vars
                .iter_mut()
                .find(|var| var.key == name)
            {
                Some(var) => {
                    var.value = value;
                    var.enabled = true;
                }
                None => final_request_data.vars.push(KeyValuePair {
                    key: name,
                    value,
                    enabled: true,
                }),
            }
        }

        // Load environment variables and secrets in the main thread
        let (variables, secrets) = if let Some(selected_env) = self.get_selected_environment(cx) {
            let env_resolver = EnvironmentResolver::new();
            let env_name = selected_env.name.clone();
            let selected_env_clone = selected_env.clone();
            let collection_manager = CollectionManager::global(cx);
            let collection_manager = collection_manager.read(cx);

            if let Some(ref collection_path) = self.collection_path
                && let Some(collection) = collection_manager.get_collection_by_path(collection_path)
            {
                match env_resolver.load_environment_data(
                    &collection.data.name,
                    &env_name,
                    &[selected_env_clone],
                    cx,
                ) {
                    Ok((vars, secs)) => {
                        tracing::info!(
                            "Loaded {} variables and {} secrets for environment '{}'",
                            vars.len(),
                            secs.len(),
                            env_name
                        );
                        (Some(vars), Some(secs))
                    }
                    Err(e) => {
                        tracing::error!("Failed to load environment data: {}", e);
                        (None, None)
                    }
                }
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        // Resolve inherited auth from collection
        if matches!(final_request_data.auth, crate::domain::AuthType::Inherit) {
            final_request_data.auth = self
                .collection_path
                .as_ref()
                .and_then(|path| {
                    CollectionManager::global(cx)
                        .read(cx)
                        .get_collection_by_path(path)
                })
                .and_then(|collection| collection.toml.collection.auth.clone())
                .unwrap_or_default();
        }

        // Gather the collection's session runtime vars and declared collection
        // vars so they can be seeded into the request's VariableStore and
        // participate in {{}} resolution (precedence: runtime > collection).
        let (runtime_vars_for_request, collection_vars_for_request) =
            if let Some(path) = &self.collection_path {
                let manager = CollectionManager::global(cx);
                let collection = manager.read(cx).get_collection_by_path(path);
                if let Some(info) = collection {
                    let cv: HashMap<String, String> = info
                        .toml
                        .collection
                        .vars
                        .iter()
                        .filter(|v| v.enabled && !v.key.is_empty())
                        .map(|v| (v.key.clone(), v.value.clone()))
                        .collect();
                    (Some(info.runtime_vars.clone()), Some(cv))
                } else {
                    (None, None)
                }
            } else {
                (None, None)
            };

        // Resolve references to earlier exchanges, such as
        // {{login.response.body.$.token}}, wherever the request or its
        // variables use them, and pass them on as collection variables.
        let collection_vars_for_request = self.with_exchange_references(
            &final_request_data,
            variables.as_ref(),
            collection_vars_for_request,
            cx,
        );

        // Get the HTTP client after updating UI to avoid borrow issues
        let http_client = HttpClientService::global(cx);
        // Capture the manager entity so the async task can write back dirty env
        // vars and runtime vars after the request completes.
        let collection_manager = CollectionManager::global(cx);

        // Clone necessary data before moving into async closure
        let collection_path = self.collection_path.clone();
        let selected_env_for_later = self.get_selected_environment(cx);
        let response_input = self.response_input.clone();
        let raw_response_input = self.raw_response_input.clone();
        let editor_entity = cx.entity().clone();

        // Capture data for history. `# @no-log` requests stay out of it.
        let record_history = !request_data.http_file.as_ref().is_some_and(|o| o.no_log);
        let history_request_name = request_data.name.clone();
        let history_method = request_data.method.as_str().to_string();
        let history_url = request_data.url.clone();
        let history_collection_path = self.collection_path.clone();

        // Execute request using async-compat and GPUI's spawn
        let request_data_clone1 = final_request_data.clone();
        let http_client_clone = http_client.clone();

        let task = cx.spawn_in(window, async move |_this, window| {
            match async_compat::Compat::new(
                http_client_clone.send_request(
                    request_data_clone1,
                    variables,
                    secrets,
                    runtime_vars_for_request,
                    collection_vars_for_request,
                )
            ).await {
                Ok((response_data, variable_store)) => {
                    // Check for dirty environment variables
                    let dirty_vars = variable_store.get_dirty_env_vars();
                    if !dirty_vars.is_empty() {
                        tracing::info!("Environment variables modified by scripts: {:?}", dirty_vars.keys().collect::<Vec<_>>());
                        tracing::info!("Dirty variables that need to be persisted: {:?}", dirty_vars);
                        if let (Some(collection_path), Some(selected_env)) = (collection_path.as_ref(), selected_env_for_later) {
                            // Update the CollectionManager with dirty variables
                            match collection_manager.update(window, |collection_manager, cx| {
                                collection_manager.update_environment_variables(collection_path, selected_env.name.as_str(), &dirty_vars, cx)
                            }) {
                                Ok(()) => {
                                    tracing::info!("Successfully updated {} environment variables in CollectionManager", dirty_vars.len());
                                }
                                Err(e) => {
                                    tracing::error!("Failed to update environment variables in CollectionManager: {}", e);
                                }
                            }
                        } else {
                            tracing::warn!("No collection or environment selected, cannot update dirty variables");
                        }
                    }

                    // Write runtime vars set by scripts (bru.setVar) back into the
                    // collection's session store so they survive to the next
                    // request and refresh the Vars view.
                    let runtime_vars = variable_store.get_all_vars();
                    if let Some(collection_path) = collection_path.as_ref()
                        && let Err(e) = collection_manager.update(window, |collection_manager, cx| {
                            collection_manager.update_runtime_vars(collection_path, runtime_vars, cx)
                        })
                    {
                        tracing::error!("Failed to update runtime variables: {}", e);
                    }

                    // Remember the exchange for {{name.response...}} references.
                    if let Some(collection_path) = collection_path.as_ref() {
                        collection_manager
                            .update(window, |collection_manager, _| {
                                collection_manager.record_exchange(
                                    collection_path,
                                    &history_request_name,
                                    response_data.clone(),
                                );
                            })
                            ;
                    }

                    // Successfully got response data
                    window.update(|window, cx| {
                        // Save to history
                        if record_history {
                            let history_entry = crate::app_database::HistoryEntry {
                                id: None,
                                method: history_method.clone(),
                                url: history_url.clone(),
                                status_code: response_data.status_code.map(|c| c as i32),
                                latency_ms: response_data.latency.map(|l| l.as_millis() as i64),
                                response_size: response_data.size.map(|s| s as i64),
                                request_name: Some(history_request_name.clone()),
                                collection_path: history_collection_path.clone(),
                                created_at: chrono::Utc::now(),
                            };
                            let history_entry_for_db = history_entry.clone();
                            let db = crate::app_database::AppDatabase::global(cx).clone();
                            cx.spawn(async move |_| {
                                db.insert_history(&history_entry_for_db).await.log_err().ok();
                            })
                            .detach();
                            editor_entity.update(cx, |_, cx| {
                                cx.emit(AppEvent::RequestHistoryRecorded(history_entry));
                            });
                        }

                        // Detect content type and get language for syntax highlighting
                        let format = ResponseFormat::detect_from_content(
                            &response_data.body,
                            &response_data.headers,
                        );
                        let language = format.language_string();

                        tracing::info!("Response format detected: {:?}, {}", format, language);

                        // Format content (pretty print JSON if applicable)
                        let formatted_content = format.format_content(&response_data.body);

                        // Update the RequestEditor's response_data for status bar and reset loading state
                        editor_entity.update(cx, |request_editor, cx| {
                            request_editor.response_data = response_data.clone();
                            request_editor.is_loading = false;
                            request_editor.response_format = format;
                            request_editor.original_response_body = Some(formatted_content.clone());
                            request_editor.response_image = match (format, &response_data.body_bytes) {
                                (ResponseFormat::Image(img_format), Some(bytes)) => {
                                    Some(Arc::new(gpui::Image::from_bytes(img_format, bytes.clone())))
                                }
                                _ => None,
                            };
                            request_editor.jsonpath_input.update(cx, |input, cx| {
                                input.set_value("", window, cx);
                            });
                            cx.notify();
                        });

                        // Update the response input with the correct language and formatted content
                        response_input.update(cx, |input_state, cx| {
                            input_state.set_highlighter(language, cx);
                            input_state.set_value(&formatted_content, window, cx);
                            cx.notify();
                        });

                        raw_response_input.update(cx, |input_state, cx| {
                            let raw_content = response_data.format_raw_response();
                            input_state.set_value(raw_content, window, cx);
                            cx.notify();
                        })
                    })?;
                }
                Err(error) => {
                    let http_error = error;

                    let error_summary = SharedString::from(http_error.summary.clone());

                    window.update(|window, cx| {
                        // Create error response
                        let response_data = ResponseData {
                            status_code: None,
                            status_text: Some("Error".to_string()),
                            latency: None,
                            size: Some(http_error.details.len()),
                            headers: vec![],
                            request_headers: vec![],
                            body: http_error.details.clone(),
                            body_bytes: None,
                            url: None,
                            encoded: None,
                            request_body: String::new(),
                        };

                        // Update the RequestEditor's response_data for status bar and reset loading state
                        editor_entity.update(cx, |request_editor, cx| {
                            request_editor.response_data = response_data.clone();
                            request_editor.is_loading = false;
                            request_editor.response_image = None;
                            cx.notify();
                        });

                        response_input.update(cx, |input_state, cx| {
                            input_state.set_value(&response_data.body, window, cx);
                            cx.notify();
                        });

                        window.push_notification(
                            (NotificationType::Error, error_summary.clone()),
                             cx,
                        );
                    })?;
                }
            }
            Ok::<(), anyhow::Error>(())
        });

        // Store the task so it can be cancelled
        self.current_request_task = Some(task);
    }

    /// `collection_vars` with the values of any `{{name.response...}}`
    /// references found in the request or the variables it may use.
    fn with_exchange_references(
        &self,
        request: &RequestData,
        environment: Option<&HashMap<String, String>>,
        collection_vars: Option<HashMap<String, String>>,
        cx: &App,
    ) -> Option<HashMap<String, String>> {
        use crate::environments::references;

        let Some(exchanges) = self
            .collection_path
            .as_deref()
            .and_then(|path| CollectionManager::global(cx).read(cx).exchanges(path))
        else {
            return collection_vars;
        };

        let pairs = request
            .headers
            .iter()
            .chain(&request.query_params)
            .chain(&request.path_params)
            .chain(&request.vars);
        let texts = [request.url.as_str(), request.body.as_str()]
            .into_iter()
            .chain(pairs.flat_map(|pair| [pair.key.as_str(), pair.value.as_str()]))
            .chain(
                environment
                    .into_iter()
                    .flat_map(|vars| vars.values().map(String::as_str)),
            )
            .chain(
                collection_vars
                    .iter()
                    .flat_map(|vars| vars.values().map(String::as_str)),
            );

        let mut resolved = HashMap::new();
        for text in texts {
            for name in references::references_in(text) {
                if let Some(value) = references::evaluate(&name, exchanges) {
                    resolved.insert(name, value);
                }
            }
        }
        if resolved.is_empty() {
            return collection_vars;
        }
        let mut vars = collection_vars.unwrap_or_default();
        vars.extend(resolved);
        Some(vars)
    }

    /// After saving into a `.http` file, take on where the request is now
    /// stored, so the next save updates it instead of adding it again.
    fn adopt_http_source(
        &mut self,
        collection_path: &str,
        name: &str,
        request_data: &mut RequestData,
        cx: &App,
    ) {
        let manager = CollectionManager::global(cx).read(cx);
        let Some(info) = manager.get_collection_by_path(collection_path) else {
            return;
        };
        if info.format != crate::collections::CollectionFormat::Http {
            return;
        }
        let file = request_data
            .http_file
            .as_ref()
            .and_then(|options| options.source.as_ref())
            .map(|source| source.file.clone())
            .or_else(|| self.group_path.clone())
            .unwrap_or_else(|| crate::collections::HTTP_DEFAULT_FILE.to_string());
        let Some(saved) = manager.find_request(collection_path, Some(&file), name) else {
            return;
        };
        request_data.http_file = saved.http_file.clone();
        request_data.script_dialect = saved.script_dialect;
        self.request_data.http_file = saved.http_file.clone();
        self.request_data.script_dialect = saved.script_dialect;
        self.group_path = Some(file);
    }

    /// Prettify the body when it is JSON or XML. The change can be undone in
    /// the body editor. A body that can't be formatted is left as it is.
    pub fn format_body(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.uses_form_editor(cx) {
            return;
        }
        let body = self.body_input.read(cx).value().to_string();
        if body.trim().is_empty() {
            return;
        }
        let content_type = self.content_type_select.read(cx).selected_value().copied();
        match super::format_body::format_body(&body, content_type) {
            Ok(formatted) if formatted != body => {
                self.body_input.update(cx, |state, cx| {
                    state.replace_all(formatted, window, cx);
                });
            }
            Ok(_) => {}
            Err(error) => {
                window.push_notification((NotificationType::Warning, error.message()), cx);
            }
        }
    }

    /// Save the request to its collection. Returns whether it was saved.
    pub fn save_request(&mut self, window: &mut Window, cx: &mut Context<Self>) -> bool {
        // Get current request data
        let request_data = self.get_request_data(cx);

        // Get request name from the request data
        let request_name = if request_data.name.is_empty() || request_data.name == "New Request" {
            "untitled_request".to_string()
        } else {
            request_data.name.clone()
        };

        if let Some(collection_path) = self.collection_path.clone() {
            // Persist the request through the CollectionManager entity.
            let group_path_ref = self.group_path.as_deref();
            let manager = CollectionManager::global(cx);
            let save_result = manager.update(cx, |collection_manager, cx| {
                collection_manager.save_request(
                    &collection_path,
                    &request_data,
                    &request_name,
                    group_path_ref,
                    cx,
                )
            });

            match save_result {
                Ok(()) => {
                    tracing::info!("Request saved successfully");
                    let mut request_data = request_data;
                    self.adopt_http_source(&collection_path, &request_name, &mut request_data, cx);
                    // Reset dirty state after successful save
                    self.original_request_data = Some(request_data);
                    self._was_dirty = false;

                    // Emit clean state event
                    cx.emit(RequestEditorEvent::DirtyStateChanged { is_dirty: false });

                    cx.notify();

                    window.push_notification(
                        (
                            NotificationType::Success,
                            SharedString::from(format!("Saved {}", request_name).to_string()),
                        ),
                        cx,
                    );
                    true
                }
                Err(e) => {
                    tracing::error!("Failed to save request: {}", e);
                    window.push_notification(
                        (NotificationType::Error, SharedString::from(e.to_string())),
                        cx,
                    );
                    false
                }
            }
        } else {
            tracing::warn!("Cannot save request: no collection_id set");
            false
        }
    }

    fn copy_as_curl(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let request_data = self.get_request_data(cx);
        let curl_string = curl::to_curl(&request_data);
        cx.write_to_clipboard(gpui::ClipboardItem::new_string(curl_string));
        window.push_notification((NotificationType::Info, "Copied as cURL"), cx);
    }

    fn try_import_curl(
        &mut self,
        input: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let parsed = match curl::parse_curl(input) {
            Some(p) => p,
            None => return false,
        };

        // Set method
        if let Some(index) = HttpMethod::ALL.iter().position(|&m| m == parsed.method) {
            self.method_select.update(cx, |state, cx| {
                state.set_selected_index(Some(IndexPath::default().row(index)), window, cx);
            });
        }

        // Set URL (strip query params so they go to the query editor)
        let (base_url, url_query) = Self::strip_query_params_from_url(&parsed.url);
        self.url_input.update(cx, |state, cx| {
            state.set_value(&base_url, window, cx);
        });

        // Set headers
        self.header_editor.update(cx, |editor, cx| {
            editor.set_pairs(&parsed.headers, window, cx);
        });

        // Set body
        if let Some(body) = parsed.body {
            self.body_input.update(cx, |state, cx| {
                state.set_value(&body, window, cx);
            });
        }

        // Handle basic auth from -u flag
        if let Some((user, pass)) = parsed.basic_auth {
            self.auth_editor.update(cx, |editor, cx| {
                editor.set_auth(
                    &AuthType::Basic(crate::domain::BasicAuth {
                        username: user,
                        password: pass,
                    }),
                    window,
                    cx,
                );
            });
        }

        // Merge URL query params into query editor
        if !url_query.is_empty() {
            self.query_param_editor.update(cx, |editor, cx| {
                let existing = editor.get_pairs(cx);
                let mut merged = existing;
                for param in url_query {
                    if !merged.iter().any(|p| p.key == param.key) {
                        merged.push(param);
                    }
                }
                editor.set_pairs(&merged, window, cx);
            });
        }

        if parsed.is_multipart {
            window.push_notification(
                (
                    NotificationType::Warning,
                    "Multipart cURL imported with limitations: form fields kept as raw body",
                ),
                cx,
            );
        } else {
            window.push_notification((NotificationType::Success, "Imported from cURL"), cx);
        }
        true
    }

    fn render_url_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_3()
            .w_full()
            .flex_wrap()
            .child(
                h_flex()
                    .flex_1()
                    .gap_3()
                    .child(div().w(px(120.)).child(Select::new(&self.method_select)))
                    .child(
                        div()
                            .flex_auto()
                            .min_w(px(160.))
                            .child(Select::new(&self.environment_select)),
                    ),
            )
            .child(
                h_flex()
                    .gap_3()
                    .refine_style(&self.url_bar_style_refinement)
                    .min_w(px(300.))
                    .child(
                        div().flex_1().child(
                            Editor::new(&self.url_input)
                                .font_family(cx.theme().mono_font_family.clone())
                                .text_sm(),
                        ),
                    )
                    .child(
                        // A single mouse-down handler toggles between send and
                        // cancel. The Button is purely visual (`loading` gates
                        // off its own on_click), so cancel and send can never
                        // both fire for one click.
                        div()
                            .on_mouse_down(
                                gpui::MouseButton::Left,
                                cx.listener(|this, _, window, cx| {
                                    if this.is_loading {
                                        this.cancel_request(window, cx);
                                    } else {
                                        this.send_request(window, cx);
                                    }
                                }),
                            )
                            .child(
                                Button::new("send-request")
                                    .primary()
                                    .icon(IconName::Send)
                                    .loading(self.is_loading)
                                    .loading_icon(IconName::LoaderCircle),
                            ),
                    )
                    .child(
                        Button::new("copy-curl")
                            .ghost()
                            .compact()
                            .icon(IconName::Copy)
                            .tooltip("Copy as cURL")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.copy_as_curl(window, cx);
                            })),
                    ),
            )
    }

    fn render_request_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Calculate badge counts
        let query_count = self.get_query_badge_count(cx);
        let has_body = self.has_body_content(cx);
        let headers_count = self.get_headers_badge_count(cx);
        let path_count = self.get_path_badge_count(cx);
        let scripts_count = self.get_scripts_badge_count(cx);
        let has_auth = self.has_auth_configured(cx);
        // Badge counts the editable request variables defined on this request.
        let vars_count = self
            .request_vars_editor
            .read_with(cx, |editor, cx| editor.count(cx));

        // The segmented trough is full-bleed, so inset it with a wrapper.
        div().p(px(6.)).min_w_0().child(
            TabBar::new("request-tabs")
                .segmented()
                .selected_index(match self.active_tab {
                    RequestTab::Query => 0,
                    RequestTab::Body => 1,
                    RequestTab::Headers => 2,
                    RequestTab::Auth => 3,
                    RequestTab::Path => 4,
                    RequestTab::Vars => 5,
                    RequestTab::Scripts => 6,
                })
                .on_click(cx.listener(|this, &index, _, cx| {
                    this.active_tab = match index {
                        0 => RequestTab::Query,
                        1 => RequestTab::Body,
                        2 => RequestTab::Headers,
                        3 => RequestTab::Auth,
                        4 => RequestTab::Path,
                        5 => RequestTab::Vars,
                        6 => RequestTab::Scripts,
                        _ => RequestTab::Query,
                    };
                    cx.notify();
                }))
                .child(Tab::new().label("Query").when(query_count > 0, |tab| {
                    tab.pr_2().suffix(TabBadge::new().count(query_count))
                }))
                .child(
                    Tab::new()
                        .label("Body")
                        .when(has_body, |tab| tab.pr_2().suffix(TabBadge::new().count(1))),
                )
                .child(Tab::new().label("Headers").when(headers_count > 0, |tab| {
                    tab.pr_2().suffix(TabBadge::new().count(headers_count))
                }))
                .child(
                    Tab::new()
                        .label("Auth")
                        .when(has_auth, |tab| tab.pr_2().suffix(TabBadge::new().count(1))),
                )
                .child(Tab::new().label("Path").when(path_count > 0, |tab| {
                    tab.pr_2().suffix(TabBadge::new().count(path_count))
                }))
                .child(Tab::new().label("Vars").when(vars_count > 0, |tab| {
                    tab.pr_2().suffix(TabBadge::new().count(vars_count))
                }))
                .child(Tab::new().label("Scripts").when(scripts_count > 0, |tab| {
                    tab.pr_2().suffix(TabBadge::new().count(scripts_count))
                })),
        )
    }

    fn render_tab_content(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        match self.active_tab {
            RequestTab::Path => div().size_full().child(self.path_param_editor.clone()),
            RequestTab::Query => div().size_full().child(self.query_param_editor.clone()),
            RequestTab::Body => {
                let selected_content_type =
                    self.content_type_select.read(cx).selected_value().copied();
                let shows_text = !self.uses_form_editor(cx);

                div().size_full().child(
                    v_flex()
                        .h_full()
                        .child(
                            h_flex()
                                .p_3()
                                .gap_3()
                                .items_center()
                                .border_b_1()
                                .border_color(cx.theme().border)
                                .child(
                                    div()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("Content-Type"),
                                )
                                .child(
                                    div()
                                        .w(px(150.))
                                        .child(Select::new(&self.content_type_select).small()),
                                )
                                .when(shows_text, |this| {
                                    this.child(div().flex_1()).child(
                                        Button::new("format-body")
                                            .ghost()
                                            .small()
                                            .icon(IconName::Braces)
                                            .label("Format")
                                            .tooltip("Prettify a JSON or XML body")
                                            // The shortcut as bound, which the user can change.
                                            .children(Kbd::binding_for_action(
                                                &FormatBody,
                                                Some(CONTEXT),
                                                window,
                                            ))
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.format_body(window, cx);
                                            })),
                                    )
                                }),
                        )
                        .child(match selected_content_type {
                            Some(ContentType::Form) if self.request_data.http_file.is_none() => {
                                div()
                                    .size_full()
                                    .child(self.form_editor.clone())
                                    .into_any_element()
                            }
                            _ => Editor::new(&self.body_input)
                                .font_family(cx.theme().mono_font_family.clone())
                                .text_size(px(12.))
                                .h_full()
                                .bordered(false)
                                .rounded_none()
                                .py_3()
                                .into_any_element(),
                        }),
                )
            }
            RequestTab::Headers => div().size_full().child(self.header_editor.clone()),
            RequestTab::Auth => div().size_full().child(self.auth_editor.clone()),
            RequestTab::Scripts => div().size_full().child(self.script_editor.clone()),
            RequestTab::Vars => div().size_full().child(
                // A single scroll region (like the other editors): editable
                // request variables on top, then the read-only inherited
                // collection + runtime variables, all scrolling together.
                v_flex()
                    .h_full()
                    .overflow_y_scrollbar()
                    .child(self.request_vars_editor.clone())
                    .child(
                        div()
                            // Space the inherited (read-only) variables away
                            // from the editable request variables above.
                            .mt_6()
                            .border_t_1()
                            .border_color(cx.theme().border)
                            .child(self.vars_view.clone()),
                    ),
            ),
        }
    }

    fn render_status_bar(&self, window: &Window, cx: &mut Context<Self>) -> impl IntoElement {
        let status_text = if let Some(status) = self.response_data.status_code {
            let text = format!(
                "{} {}",
                status,
                self.response_data.status_text.as_deref().unwrap_or("")
            );
            Some((status, text))
        } else {
            None
        };

        let latency_text = if let Some(latency) = self.response_data.latency {
            format!("{}ms", latency.as_millis())
        } else {
            String::new()
        };

        let size_text = match (self.response_data.size, &self.response_data.encoded) {
            (Some(size), Some(encoded)) => format!(
                " {} ({} {})",
                format_size(size),
                format_size(encoded.size),
                encoded.encoding
            ),
            (Some(size), None) => format!(" {}", format_size(size)),
            (None, _) => String::new(),
        };

        h_flex()
            .justify_between()
            .p_3()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            // The status bar paints the bottom of the editor card, so it has to
            // carry the card radius itself: GPUI's content mask is rectangular
            // and won't clip a child's background to rounded corners.
            .rounded_b(crate::app::PANEL_RADIUS)
            .child(
                h_flex()
                    .font_family(cx.theme().mono_font_family.clone())
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .when_some(status_text, |this, (status, text)| {
                        let text_color = match status {
                            100..=199 => cx.theme().blue,   // Informational responses
                            200..=299 => cx.theme().green,  // Successful responses
                            300..=399 => cx.theme().blue,   // Redirection messages
                            400..=499 => cx.theme().yellow, // Client error responses
                            500..=599 => cx.theme().red,    // Server error responses
                            _ => cx.theme().foreground,     // Default for unknown ranges
                        };
                        this.child(div().text_color(text_color).child(text))
                    })
                    .when(!latency_text.is_empty(), |this| {
                        this.child(div().child(format!(" • {}", latency_text)))
                    })
                    .child(size_text),
            )
            .child(div().flex_1())
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .child(
                        div()
                            .w(px(200.))
                            .child(Input::new(&self.name_input).text_sm()),
                    )
                    .child(
                        Button::new("save-request")
                            .primary()
                            .compact()
                            .label("Save Request")
                            .icon(IconName::Save)
                            // The shortcut as bound, which the user can change.
                            .children(Kbd::binding_for_action(&Save, Some(CONTEXT), window))
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.save_request(window, cx);
                            })),
                    ),
            )
    }

    /// The response text the active response tab shows: the formatted (and
    /// filtered) body on Response, the body as received on Raw. `None` when
    /// there is nothing to copy, or the response is an image or a PDF.
    fn copyable_response_text(&self, cx: &App) -> Option<String> {
        if self.response_format == ResponseFormat::Pdf || self.response_image.is_some() {
            return None;
        }
        let input = match self.active_response_tab {
            ResponseTab::Response => &self.response_input,
            ResponseTab::Raw => &self.raw_response_input,
        };
        let text = input.read(cx).value().to_string();
        (!text.is_empty()).then_some(text)
    }

    pub fn copy_response(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(text) = self.copyable_response_text(cx) {
            cx.write_to_clipboard(gpui::ClipboardItem::new_string(text));
            window.push_notification((NotificationType::Info, "Copied response"), cx);
        }
    }

    fn render_response_area(
        &self,
        layout: EditorLayout,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex_1()
            .h_full()
            .bg(cx.theme().tab_active)
            .border_color(cx.theme().border)
            // Stacked: a divider separates the response from the request pane
            // above. Side-by-side: the resize handle already draws its own 1px
            // line down the split, so any border here just doubles up.
            .when(layout == EditorLayout::Vertical, |this| this.border_t_1())
            .child(
                v_flex()
                    .size_full()
                    .child(
                        // Response tabs. The segmented trough is full-bleed, so
                        // inset it with a wrapper.
                        div().p(px(6.)).min_w_0().child(
                            TabBar::new("response-tabs")
                                .segmented()
                                .selected_index(match self.active_response_tab {
                                    ResponseTab::Response => 0,
                                    ResponseTab::Raw => 1,
                                })
                                .on_click(cx.listener(|this, &index, _, cx| {
                                    this.active_response_tab = match index {
                                        0 => ResponseTab::Response,
                                        1 => ResponseTab::Raw,
                                        _ => ResponseTab::Response,
                                    };
                                    cx.notify();
                                }))
                                .child(Tab::new().label("Response"))
                                .child(Tab::new().label("Raw"))
                                .when(self.copyable_response_text(cx).is_some(), |this| {
                                    this.suffix(
                                        Button::new("copy-response")
                                            .ghost()
                                            .xsmall()
                                            .icon(IconName::Copy)
                                            .tooltip_with_action(
                                                "Copy Response",
                                                &CopyResponse,
                                                Some(CONTEXT),
                                            )
                                            .on_click(cx.listener(|this, _, window, cx| {
                                                this.copy_response(window, cx);
                                            })),
                                    )
                                }),
                        ),
                    )
                    .child(
                        // Response content
                        div().flex_1().child(match self.active_response_tab {
                            ResponseTab::Response => {
                                if self.response_format == ResponseFormat::Pdf {
                                    div()
                                        .h_full()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .child(
                                            Button::new("open-pdf")
                                                .label("Open PDF")
                                                .primary()
                                                .icon(IconName::ExternalLink)
                                                .on_click(cx.listener(|this, _, _window, cx| {
                                                    if let Some(bytes) = &this.response_data.body_bytes {
                                                        let timestamp = std::time::SystemTime::now()
                                                            .duration_since(std::time::UNIX_EPOCH)
                                                            .unwrap_or_default()
                                                            .as_millis();
                                                        let file_name = format!("broquest-{}.pdf", timestamp);
                                                        let path = std::env::temp_dir().join(file_name);
                                                        match std::fs::write(&path, bytes) {
                                                            Ok(()) => cx.open_with_system(&path),
                                                            Err(error) => {
                                                                tracing::error!("Failed to write PDF temp file: {}", error);
                                                            }
                                                        }
                                                    }
                                                })),
                                        )
                                        .into_any_element()
                                } else if let Some(image) = &self.response_image {
                                    div()
                                        .h_full()
                                        .flex()
                                        .items_center()
                                        .justify_center()
                                        .overflow_hidden()
                                        .child(
                                            img(ImageSource::Image(image.clone()))
                                                .max_w_full()
                                                .max_h_full()
                                                .object_fit(ObjectFit::Contain),
                                        )
                                        .into_any_element()
                                } else {
                                    let is_json = self.is_response_json();
                                    v_flex()
                                        .h_full()
                                        .child(
                                            div().flex_1().child(
                                                Editor::new(&self.response_input)
                                                    .font_family(
                                                        cx.theme().mono_font_family.clone(),
                                                    )
                                                    .text_size(px(12.))
                                                    .h_full()
                                                    .py_3()
                                                    .bordered(false)
                                                    .rounded_none(),
                                            ),
                                        )
                                        .when(is_json, |this| {
                                            this.child(
                                                h_flex()
                                                    .px_3()
                                                    .py_2()
                                                    .border_t_1()
                                                    .border_color(cx.theme().border)
                                                    .bg(cx.theme().background)
                                                    .gap_3()
                                                    .items_center()
                                                    .child(
                                                        div()
                                                            .text_color(cx.theme().blue)
                                                            .child(Icon::new(IconName::Funnel)),
                                                    )
                                                    .child(
                                                        div().flex_1().child(
                                                            Input::new(&self.jsonpath_input)
                                                                .font_family(
                                                                    cx.theme()
                                                                        .mono_font_family
                                                                        .clone(),
                                                                )
                                                                .text_size(px(12.))
                                                                .cleanable(true),
                                                        ),
                                                    ),
                                            )
                                        })
                                        .into_any_element()
                                }
                            }
                            ResponseTab::Raw => div()
                                .h_full()
                                .child(
                                    Editor::new(&self.raw_response_input)
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_size(px(12.))
                                        .h_full()
                                        .py_3()
                                        .bordered(false)
                                        .rounded_none(),
                                )
                                .into_any_element(),
                        }),
                    ),
            )
    }

    /// Set up two-way binding between URL input and query parameter editor
    pub fn setup_url_query_binding(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Set up subscription for URL input changes
        let url_input = self.url_input.clone();
        let query_param_editor = self.query_param_editor.clone();
        let url_subscription = cx.subscribe_in(&url_input, window, {
            move |this: &mut Self, input_state, event: &InputEvent, window, cx| {
                if let InputEvent::Change = event {
                    // Check if the URL input is focused before emitting events
                    if !input_state.read(cx).focus_handle(cx).is_focused(window) {
                        return;
                    }

                    let current_url = this.url_input.read(cx).value().to_string();

                    // Try to detect and import cURL commands
                    if current_url.trim().starts_with("curl") {
                        this.try_import_curl(&current_url, window, cx);
                        return;
                    }

                    // Don't update query params if we're currently updating URL from params
                    if this._updating_url_from_params {
                        return;
                    }
                    let parsed_params = this.parse_query_params_from_url(&current_url);

                    // Only update query parameters if this is a genuine URL change (not from parameter editor)
                    query_param_editor.update(cx, |editor, cx| {
                        let existing_params = editor.get_pairs(cx);

                        // Skip if URL doesn't have query parameters
                        if !current_url.contains('?') {
                            return;
                        }

                        // Skip if no meaningful parsed parameters
                        if parsed_params.is_empty() {
                            return;
                        }

                        // Check if this would actually change anything
                        // Only proceed if parsed params differ from existing enabled params
                        let existing_enabled: Vec<_> =
                            existing_params.iter().filter(|p| p.enabled).collect();

                        // Quick check: if counts differ, update is needed
                        if existing_enabled.len() != parsed_params.len() {
                            editor.set_pairs(&parsed_params, window, cx);
                            return;
                        }

                        // Check if any existing enabled parameters differ from parsed ones
                        let needs_update = existing_enabled.iter().any(|existing| {
                            !parsed_params.iter().any(|parsed| {
                                parsed.key == existing.key && parsed.value == existing.value
                            })
                        });

                        if needs_update {
                            // Only update if there are actual differences
                            let mut merged_params = Vec::new();

                            // Preserve all disabled parameters
                            for existing_param in &existing_params {
                                if !existing_param.enabled {
                                    merged_params.push(existing_param.clone());
                                }
                            }

                            // Add new/updated parameters from URL
                            merged_params.extend(parsed_params);

                            editor.set_pairs(&merged_params, window, cx);
                        }
                    });

                    // Schedule dirty state check after URL change
                    this.recompute_dirty(cx);
                }
            }
        });
        self._subscriptions.push(url_subscription);

        // Set up subscription for query parameter changes
        let query_param_subscription = cx.subscribe_in(&self.query_param_editor, window, {
            move |this: &mut Self, _editor, event: &KeyValueEvent, window, cx| match event {
                KeyValueEvent::Changed => {
                    let current_params = this.query_param_editor.read(cx).get_pairs(cx);
                    let current_url = this.url_input.read(cx).value().to_string();
                    let new_url = this.build_url_with_query_params(&current_url, &current_params);

                    // Set flag to prevent URL change from triggering query param update
                    this._updating_url_from_params = true;
                    this.url_input.update(cx, |state, cx| {
                        state.set_value(new_url, window, cx);
                    });
                    this._updating_url_from_params = false;

                    // Schedule dirty state check after query param change
                    this.recompute_dirty(cx);
                }
            }
        });
        self._subscriptions.push(query_param_subscription);

        // Set up subscription for name input changes (for dirty state)
        let name_input = self.name_input.clone();
        let name_subscription =
            cx.subscribe(&name_input, |this, _input, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    this.recompute_dirty(cx);
                }
            });
        self._subscriptions.push(name_subscription);

        // Set up subscription for method select changes (for dirty state)
        let method_select = self.method_select.clone();
        let method_subscription = cx.subscribe(
            &method_select,
            |this, _state, _event: &SelectEvent<Vec<HttpMethod>>, cx| {
                this.recompute_dirty(cx);
            },
        );
        self._subscriptions.push(method_subscription);

        // Set up subscriptions for all editors to track dirty state changes
        // Subscribe to path parameter editor changes
        let path_param_subscription = cx.subscribe(
            &self.path_param_editor,
            |this, _editor, _: &KeyValueEvent, cx| {
                this.recompute_dirty(cx);
                cx.notify();
            },
        );
        self._subscriptions.push(path_param_subscription);

        // Subscribe to header editor changes
        let header_subscription = cx.subscribe(
            &self.header_editor,
            |this, _editor, _: &KeyValueEvent, cx| {
                this.recompute_dirty(cx);
                cx.notify();
            },
        );
        self._subscriptions.push(header_subscription);

        // Subscribe to script editor changes
        let script_subscription = cx.subscribe(
            &self.script_editor,
            |this, _editor, _: &ScriptEditorEvent, cx| {
                this.recompute_dirty(cx);
                cx.notify();
            },
        );
        self._subscriptions.push(script_subscription);

        // Subscribe to auth editor changes
        let auth_subscription = cx.subscribe(
            &self.auth_editor,
            |this, _editor, _: &AuthEditorEvent, cx| {
                this.recompute_dirty(cx);
                cx.notify();
            },
        );
        self._subscriptions.push(auth_subscription);

        // Subscribe to form editor changes
        let form_subscription =
            cx.subscribe(&self.form_editor, |this, _editor, _: &KeyValueEvent, cx| {
                this.recompute_dirty(cx);
                cx.notify();
            });
        self._subscriptions.push(form_subscription);

        // Subscribe to body input changes
        let body_subscription =
            cx.subscribe(&self.body_input, |this, _input, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    this.recompute_dirty(cx);
                    cx.notify();
                }
            });
        self._subscriptions.push(body_subscription);

        // Subscribe to content type changes (affects body badge)
        let content_type_subscription = cx.subscribe(
            &self.content_type_select,
            |this, _state, _event: &SelectEvent<Vec<ContentType>>, cx| {
                this.recompute_dirty(cx);
                cx.notify();
            },
        );
        self._subscriptions.push(content_type_subscription);

        // Subscribe to JSONPath input changes for filtering response
        let jsonpath_input = self.jsonpath_input.clone();
        let jsonpath_subscription = cx.subscribe_in(&jsonpath_input, window, {
            move |this: &mut Self, _input_state, event: &InputEvent, window, cx| {
                if let InputEvent::Change = event {
                    this.handle_jsonpath_input_change(window, cx);
                }
            }
        });
        self._subscriptions.push(jsonpath_subscription);
    }

    /// Parse query parameters from a URL string
    fn parse_query_params_from_url(&self, url: &str) -> Vec<KeyValuePair> {
        // Find the start of query string (after ? and before # or end)
        let query_start = match url.find('?') {
            Some(pos) => pos + 1,
            None => return Vec::new(),
        };

        // Find the end of query string (before # or end of string)
        let query_end = url[query_start..]
            .find('#')
            .map(|pos| query_start + pos)
            .unwrap_or(url.len());

        let query_string = &url[query_start..query_end];
        if query_string.is_empty() {
            return Vec::new();
        }

        // Parse query parameters, nly accept well-formed key=value pairs
        query_string
            .split('&')
            .filter_map(|pair| {
                let mut parts = pair.splitn(2, '=');
                let key = parts.next()?;
                let value = parts.next();

                // Only accept parameters that have both key and value separated by =
                // This prevents partial typing like "?h" or "?hello" from creating parameters
                value?;

                let key = key.trim();
                let value = value.unwrap_or("").trim();

                // Don't create parameters if key is empty
                if key.is_empty() {
                    return None;
                }

                // Allow empty values (like "hello=") but be more restrictive about
                // very short keys with empty values that are likely from partial typing
                if key.len() == 1 && value.is_empty() {
                    return None;
                }

                // Decode URL encoding (basic implementation)
                let decoded_key = url_decode(key);
                let decoded_value = url_decode(value);

                if !decoded_key.is_empty() {
                    Some(KeyValuePair {
                        key: decoded_key,
                        value: decoded_value,
                        enabled: true,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Build a URL with the given query parameters
    fn build_url_with_query_params(&self, url: &str, params: &[KeyValuePair]) -> String {
        // Find the start of query string (after ? and before # or end)
        let query_start = url.find('?');
        let fragment_start = url.find('#');

        // Extract base URL (without query string and fragment)
        let base_url_end = query_start.or(fragment_start).unwrap_or(url.len());
        let base_url = &url[..base_url_end];

        // Extract fragment if present
        let fragment = fragment_start.map(|pos| &url[pos..]).unwrap_or("");

        // Filter enabled parameters and build query string
        let enabled_params: Vec<_> = params.iter().filter(|p| p.enabled).collect();
        if enabled_params.is_empty() {
            return format!("{}{}", base_url, fragment);
        }

        let query_string = enabled_params
            .iter()
            .map(|param| {
                // Basic URL encoding
                format!("{}={}", url_encode(&param.key), url_encode(&param.value))
            })
            .collect::<Vec<_>>()
            .join("&");

        format!("{}?{}{}", base_url, query_string, fragment)
    }

    /// Get the method select entity for external subscriptions
    pub fn method_select(&self) -> &Entity<SelectState<Vec<HttpMethod>>> {
        &self.method_select
    }

    /// Get the name input entity for external subscriptions
    pub fn name_input(&self) -> &Entity<InputState> {
        &self.name_input
    }
}

impl EventEmitter<AppEvent> for RequestEditor {}
impl EventEmitter<RequestEditorEvent> for RequestEditor {}

impl RequestEditor {
    /// Calculate badge count for Query tab (number of enabled query params)
    fn get_query_badge_count(&self, cx: &App) -> usize {
        self.query_param_editor
            .read(cx)
            .get_pairs(cx)
            .iter()
            .filter(|param| param.enabled && !param.key.is_empty())
            .count()
    }

    /// Calculate if Body tab should show a badge (has content), from
    /// whichever editor holds the body.
    fn has_body_content(&self, cx: &App) -> bool {
        if self.uses_form_editor(cx) {
            self.form_editor
                .read(cx)
                .get_pairs(cx)
                .iter()
                .any(|field| field.enabled && (!field.key.is_empty() || !field.value.is_empty()))
        } else {
            !self.body_input.read(cx).value().trim().is_empty()
        }
    }

    /// Calculate badge count for Headers tab (number of enabled headers)
    fn get_headers_badge_count(&self, cx: &App) -> usize {
        self.header_editor
            .read(cx)
            .get_pairs(cx)
            .iter()
            .filter(|header| header.enabled && !header.key.is_empty())
            .count()
    }

    /// Calculate badge count for Path tab (number of enabled path params)
    fn get_path_badge_count(&self, cx: &App) -> usize {
        self.path_param_editor
            .read(cx)
            .get_pairs(cx)
            .iter()
            .filter(|param| param.enabled && !param.key.is_empty())
            .count()
    }

    /// Calculate badge count for Scripts tab (number of scripts)
    fn get_scripts_badge_count(&self, cx: &App) -> usize {
        let script_editor = self.script_editor.read(cx);
        let pre_request = script_editor.get_pre_request_script(cx);
        let post_response = script_editor.get_post_response_script(cx);

        let count = if pre_request.is_some_and(|s| !s.trim().is_empty()) {
            1
        } else {
            0
        };

        if post_response.is_some_and(|s| !s.trim().is_empty()) {
            count + 1
        } else {
            count
        }
    }

    fn has_auth_configured(&self, cx: &App) -> bool {
        let auth = self.auth_editor.read(cx).get_auth(cx);
        !matches!(auth, AuthType::None)
    }

    fn is_response_json(&self) -> bool {
        self.response_format == ResponseFormat::Json && self.original_response_body.is_some()
    }

    fn handle_jsonpath_input_change(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self._jsonpath_filter_task = Task::ready(());

        if self.response_format != ResponseFormat::Json {
            return;
        }

        let jsonpath_query = self.jsonpath_input.read(cx).value().to_string();
        let response_input = self.response_input.clone();

        if jsonpath_query.trim().is_empty() {
            if let Some(original) = &self.original_response_body {
                response_input.update(cx, |input_state, cx| {
                    input_state.set_value(original, window, cx);
                    cx.notify();
                });
            }
            return;
        }

        let query = jsonpath_query.clone();
        self._jsonpath_filter_task = cx.spawn_in(window, async move |this, window| {
            window
                .background_executor()
                .timer(Duration::from_millis(100))
                .await;

            let original =
                match this.read_with(window, |editor, _| editor.original_response_body.clone()) {
                    Ok(Some(body)) => body.clone(),
                    _ => return,
                };

            let result = (|| -> Result<String, String> {
                let json: serde_json::Value =
                    serde_json::from_str(&original).map_err(|e| format!("Invalid JSON: {}", e))?;

                let matches: Vec<&serde_json::Value> = json
                    .query(&query)
                    .map_err(|e| format!("Invalid JSONPath: {}", e))?;

                if matches.is_empty() {
                    Ok("[]".to_string())
                } else if matches.len() == 1 {
                    Ok(serde_json::to_string_pretty(matches[0])
                        .unwrap_or_else(|_| matches[0].to_string()))
                } else {
                    Ok(serde_json::to_string_pretty(&matches).unwrap_or_else(|_| "[]".to_string()))
                }
            })();

            let result = window.update(|window, cx| {
                response_input.update(cx, |input_state, cx| {
                    match result {
                        Ok(filtered) => {
                            input_state.set_value(&filtered, window, cx);
                        }
                        Err(error_msg) => {
                            input_state.set_value(format!("// Error: {}", error_msg), window, cx);
                        }
                    }
                    cx.notify();
                })
            });

            if let Err(e) = result {
                tracing::error!("Failed to update response input: {}", e);
            }
        });
    }
}

impl RequestEditor {
    fn render_vertical_layout(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(
                div()
                    .py_3()
                    .px(px(6.))
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(self.render_url_bar(cx)),
            )
            .child(self.render_request_tabs(cx))
            .child(
                div().flex_1().min_h_0().child(
                    v_resizable("request-response")
                        // The response area draws its own top border, so the
                        // handle would only double it up.
                        .invisible_handles()
                        .with_state(&self.request_response_state)
                        .child(
                            resizable_panel()
                                .size(px(200.))
                                .size_range(px(100.)..gpui::Pixels::MAX)
                                .child(self.render_tab_content(window, cx)),
                        )
                        .child(resizable_panel().child(self.render_response_area(
                            EditorLayout::Vertical,
                            window,
                            cx,
                        ))),
                ),
            )
            .child(self.render_status_bar(window, cx))
    }

    fn render_horizontal_layout(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(
                div().flex_1().min_h_0().child(
                    h_resizable("request-response")
                        .with_state(&self.request_response_state)
                        .child(
                            resizable_panel()
                                .size(px(400.))
                                .size_range(px(200.)..gpui::Pixels::MAX)
                                .child(
                                    v_flex()
                                        .size_full()
                                        .child(
                                            div()
                                                .pb_3()
                                                .pt(px(6.))
                                                .px(px(6.))
                                                .border_b_1()
                                                .border_color(cx.theme().border)
                                                .child(self.render_url_bar(cx)),
                                        )
                                        .child(self.render_request_tabs(cx))
                                        .child(
                                            div()
                                                .flex_1()
                                                .min_h_0()
                                                .child(self.render_tab_content(window, cx)),
                                        ),
                                ),
                        )
                        .child(resizable_panel().child(self.render_response_area(
                            EditorLayout::Horizontal,
                            window,
                            cx,
                        ))),
                ),
            )
            .child(self.render_status_bar(window, cx))
    }
}

impl Render for RequestEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = match self.layout {
            EditorLayout::Vertical => self.render_vertical_layout(window, cx).into_any_element(),
            EditorLayout::Horizontal => {
                self.render_horizontal_layout(window, cx).into_any_element()
            }
        };

        div()
            .key_context(CONTEXT)
            .on_action(cx.listener(|this: &mut RequestEditor, &Save, window, cx| {
                tracing::info!("Save action triggered");
                this.save_request(window, cx);
            }))
            .on_action(cx.listener(|this: &mut RequestEditor, &Send, window, cx| {
                this.send_request(window, cx);
            }))
            .on_action(
                cx.listener(|this: &mut RequestEditor, &FormatBody, window, cx| {
                    this.format_body(window, cx);
                }),
            )
            .on_action(
                cx.listener(|this: &mut RequestEditor, &CopyResponse, window, cx| {
                    this.copy_response(window, cx);
                }),
            )
            // No background: the editor card in `BroquestApp::render` owns the
            // surface so this square-cornered div can't paint over the card's
            // rounded border.
            .size_full()
            .child(content)
    }
}

/// A byte count for the status bar, such as `412B` or `1.1KB`.
fn format_size(bytes: usize) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{}B", bytes)
    }
}

impl ResponseData {
    pub fn format_raw_response(&self) -> String {
        let mut raw_output = String::new();

        // Add request URL at the top
        if let Some(url) = &self.url {
            raw_output.push_str(&format!("{}\n", url));
        }

        // Add request headers section
        if !self.request_headers.is_empty() {
            raw_output.push_str("\n--- Request Headers ---\n");
            for header in &self.request_headers {
                if header.enabled {
                    raw_output.push_str(&format!("{}: {}\n", header.key, header.value));
                }
            }
        }

        // Add status line
        if let Some(status_code) = self.status_code {
            let status_text = self.status_text.as_deref().unwrap_or("Unknown");
            raw_output.push_str(&format!("\n{} {}\n", status_code, status_text));
        }

        // Add response headers section
        if !self.headers.is_empty() {
            raw_output.push_str("\n--- Response Headers ---\n");
            for header in &self.headers {
                if header.enabled {
                    raw_output.push_str(&format!("{}: {}\n", header.key, header.value));
                }
            }
        }

        // Add empty line between headers and body
        if !self.body.is_empty() {
            raw_output.push('\n');
        }

        // Add body
        if !self.body.is_empty() {
            raw_output.push_str(&self.body);
        }

        raw_output
    }
}

#[cfg(test)]
mod form_tests {
    use super::*;

    fn field(key: &str, value: &str) -> KeyValuePair {
        KeyValuePair {
            key: key.to_string(),
            value: value.to_string(),
            enabled: true,
        }
    }

    #[test]
    fn encoding_keeps_variables_and_handles_unicode() {
        assert_eq!(url_encode("a b&c=é"), "a%20b%26c%3D%C3%A9");
        assert_eq!(url_encode("Bearer {{token}}!"), "Bearer%20{{token}}%21");
        assert_eq!(url_encode("{{unclosed"), "%7B%7Bunclosed");
        assert_eq!(url_decode("a+b%26c%3D%C3%A9"), "a b&c=é");
        assert_eq!(url_decode("100%"), "100%");
    }

    #[test]
    fn form_bodies_round_trip() {
        let fields =
            parse_form_body("email=a%40b.com&name=Ann+Lee&city=Ume%C3%A5&file=@/tmp/x.pdf&flag");
        assert_eq!(
            fields,
            vec![
                field("email", "a@b.com"),
                field("name", "Ann Lee"),
                field("city", "Umeå"),
                field("file", "@/tmp/x.pdf"),
                field("flag", ""),
            ]
        );
        assert_eq!(
            form_body(&fields),
            "email=a%40b.com&name=Ann%20Lee&city=Ume%C3%A5&file=@/tmp/x.pdf&flag="
        );
        assert_eq!(form_body(&[field("token", "{{token}}")]), "token={{token}}");
        assert!(parse_form_body("").is_empty());
    }

    fn environments(names: &[&str]) -> Vec<EnvironmentToml> {
        names
            .iter()
            .map(|name| EnvironmentToml {
                name: name.to_string(),
                variables: HashMap::new(),
            })
            .collect()
    }

    fn settings(default: Option<&str>, require: bool) -> EnvironmentSettings {
        EnvironmentSettings {
            default_environment: default.map(str::to_string),
            require_environment: require,
        }
    }

    #[test]
    fn new_tab_selects_the_default_environment() {
        let envs = environments(&["dev", "prod"]);
        assert_eq!(
            pick_environment(&envs, &settings(Some("prod"), false), None),
            (true, Some(1))
        );
        // Without a default a new tab has no environment.
        assert_eq!(
            pick_environment(&envs, &settings(None, false), None),
            (true, None)
        );
        // A default that no longer exists is ignored.
        assert_eq!(
            pick_environment(&envs, &settings(Some("gone"), false), None),
            (true, None)
        );
    }

    #[test]
    fn reload_keeps_the_selection() {
        let envs = environments(&["dev", "prod"]);
        let prod = settings(Some("prod"), false);
        assert_eq!(
            pick_environment(&envs, &prod, Some(Some("dev".into()))),
            (true, Some(0))
        );
        // "No environment" stays chosen.
        assert_eq!(pick_environment(&envs, &prod, Some(None)), (true, None));
        // A removed environment falls back to the default.
        assert_eq!(
            pick_environment(&envs, &prod, Some(Some("gone".into()))),
            (true, Some(1))
        );
    }

    #[test]
    fn required_environment_hides_no_environment() {
        let envs = environments(&["dev", "prod"]);
        assert_eq!(
            pick_environment(&envs, &settings(None, true), None),
            (false, Some(0))
        );
        assert_eq!(
            pick_environment(&envs, &settings(Some("prod"), true), Some(None)),
            (false, Some(1))
        );
        // A collection without environments keeps "No environment".
        assert_eq!(
            pick_environment(&[], &settings(None, true), None),
            (true, None)
        );
    }
}
