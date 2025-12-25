use gpui::{
    App, AppContext, BorrowAppContext as _, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, KeyBinding, KeybindingKeystroke, Keystroke,
    ParentElement as _, Render, SharedString, Styled as _, Subscription, Task, WeakEntity, Window,
    actions, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, IndexPath, Sizable, StyledExt, WindowExt,
    button::Button,
    h_flex,
    input::{Input, InputEvent, InputState},
    kbd::Kbd,
    notification::NotificationType,
    select::{Select, SelectEvent, SelectItem, SelectState},
    tab::{Tab, TabBar},
    v_flex,
};

use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::app_events::AppEvent;
use crate::collection_manager::CollectionManager;
use crate::collection_types::EnvironmentToml;
use crate::form_editor::FormEditor;
use crate::form_editor::FormEditorEvent;
use crate::header_editor::HeaderEditor;
use crate::header_editor::HeaderEditorEvent;
use crate::http::{ContentType, HttpMethod};
use crate::http_client::ResponseFormat;
use crate::icon::IconName;
use crate::path_parameter_editor::PathParamEditor;
use crate::path_parameter_editor::PathParamEvent;
use crate::query_parameter_editor::QueryParamEditor;
use crate::query_parameter_editor::QueryParamEvent;
use crate::script_editor::ScriptEditor;
use crate::script_editor::ScriptEditorEvent;
use crate::ui::tab_badge::TabBadge;

const CONTEXT: &str = "request_editor";

actions!(request_editor, [Save]);

/// Basic URL encoding function
fn url_encode(input: &str) -> String {
    input
        .chars()
        .map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            ' ' => "+".to_string(),
            _ => format!("%{:02X}", c as u8),
        })
        .collect()
}

/// Basic URL decoding function
fn url_decode(input: &str) -> String {
    let mut result = String::new();
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '+' => result.push(' '),
            '%' => {
                if let (Some(h1), Some(h2)) = (chars.next(), chars.next())
                    && let Ok(byte) = u8::from_str_radix(&format!("{}{}", h1, h2), 16)
                    && let Some(decoded) = char::from_u32(byte as u32)
                {
                    result.push(decoded);
                }
            }
            _ => result.push(c),
        }
    }

    result
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct KeyValuePair {
    pub key: String,
    pub value: String,
    pub enabled: bool,
}

impl Default for KeyValuePair {
    fn default() -> Self {
        Self {
            key: String::new(),
            value: String::new(),
            enabled: true,
        }
    }
}

impl KeyValuePair {
    /// Check if two KeyValuePair vectors are equal (order-independent comparison).
    /// Filters out entries with empty keys and compares the remaining entries.
    pub fn vec_equals(left: &[KeyValuePair], right: &[KeyValuePair]) -> bool {
        // Filter out entries with empty keys (they don't represent real data)
        let left_filtered: Vec<_> = left.iter().filter(|p| !p.key.trim().is_empty()).collect();
        let right_filtered: Vec<_> = right.iter().filter(|p| !p.key.trim().is_empty()).collect();

        if left_filtered.len() != right_filtered.len() {
            return false;
        }

        // For each item in left, find a matching item in right
        for l in &left_filtered {
            if !right_filtered
                .iter()
                .any(|r| l.key == r.key && l.value == r.value && l.enabled == r.enabled)
            {
                return false;
            }
        }

        true
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RequestData {
    pub name: String,
    pub method: HttpMethod,
    pub url: String,
    pub path_params: Vec<KeyValuePair>,
    pub query_params: Vec<KeyValuePair>,
    pub headers: Vec<KeyValuePair>,
    pub body: String,
    pub pre_request_script: Option<String>,
    pub post_response_script: Option<String>,
}

impl Default for RequestData {
    fn default() -> Self {
        Self {
            name: "New Request".to_string(),
            method: HttpMethod::Get,
            url: String::new(),
            path_params: Vec::new(),
            query_params: Vec::new(),
            headers: Vec::new(),
            body: String::new(),
            pre_request_script: None,
            post_response_script: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ResponseData {
    pub status_code: Option<u16>,
    pub status_text: Option<String>,
    pub latency: Option<Duration>,
    pub size: Option<usize>,
    pub headers: Vec<KeyValuePair>,
    pub body: String,
    pub url: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestTab {
    Path,
    Query,
    Body,
    Headers,
    Scripts,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseTab {
    Response,
    Raw,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RequestEditorEvent {
    DirtyStateChanged { is_dirty: bool },
}

pub struct RequestEditor {
    _focus_handle: FocusHandle,
    request_data: RequestData,
    original_request_data: Option<RequestData>,
    response_data: ResponseData,
    active_tab: RequestTab,
    active_response_tab: ResponseTab,
    is_loading: bool,
    collection_path: Option<String>,
    group_path: Option<String>,
    method_select: Entity<SelectState<Vec<HttpMethod>>>,
    environment_select: Entity<SelectState<Vec<EnvironmentOption>>>,
    content_type_select: Entity<SelectState<Vec<ContentType>>>,
    name_input: Entity<InputState>,
    url_input: Entity<InputState>,
    body_input: Entity<InputState>,
    response_input: Entity<InputState>,
    raw_response_input: Entity<InputState>,
    path_param_editor: Entity<PathParamEditor>,
    query_param_editor: Entity<QueryParamEditor>,
    header_editor: Entity<HeaderEditor>,
    form_editor: Entity<FormEditor>,
    script_editor: Entity<ScriptEditor>,
    send_keystroke: KeybindingKeystroke,
    _subscriptions: Vec<Subscription>,
    _updating_url_from_params: bool,
    _was_dirty: bool,
    _dirty_check_task: Task<()>,
}

impl RequestEditor {
    pub fn init(cx: &mut App) {
        cx.bind_keys([KeyBinding::new("secondary-s", Save, Some(CONTEXT))]);
    }

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
            InputState::new(window, cx)
                .placeholder("Enter request URL")
                .code_editor("url")
                .multi_line(false)
        });

        let name_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Request name")
                .default_value("New Request")
        });

        let body_input = cx.new(|cx| InputState::new(window, cx).code_editor("json"));

        let response_input = cx.new(|cx| InputState::new(window, cx).code_editor("text"));

        let raw_response_input = cx.new(|cx| InputState::new(window, cx).code_editor("text"));

        let path_param_editor = cx.new(|cx| PathParamEditor::new(window, cx));

        let query_param_editor = cx.new(|cx| QueryParamEditor::new(window, cx));

        let header_editor = cx.new(|cx| HeaderEditor::new(window, cx));

        let form_editor = cx.new(|cx| FormEditor::new(window, cx));

        let script_editor = cx.new(|cx| ScriptEditor::new(window, cx));

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
            collection_path: None,
            group_path: None,
            method_select,
            environment_select,
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
            script_editor,
            send_keystroke: KeybindingKeystroke::from_keystroke(Keystroke::parse("enter").unwrap()),
            _subscriptions: subscriptions,
            _updating_url_from_params: false,
            _was_dirty: false,
            _dirty_check_task: Task::ready(()),
        }
    }

    pub fn set_collection_path(&mut self, collection_path: Option<String>) {
        self.collection_path = collection_path;
    }

    pub fn set_group_path(&mut self, group_path: Option<String>) {
        self.group_path = group_path;
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
            editor.set_parameters(&data.path_params, window, cx);
        });

        // Update body input
        self.body_input.update(cx, |state, cx| {
            state.set_value(data.body.clone(), window, cx);
        });

        // Update query parameters
        self.query_param_editor.update(cx, |editor, cx| {
            editor.set_parameters(&data.query_params, window, cx);
        });

        // Update headers
        self.header_editor.update(cx, |editor, cx| {
            editor.set_headers(&data.headers, window, cx);
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

        // Update scripts
        self.script_editor.update(cx, |editor, cx| {
            editor.set_scripts(
                data.pre_request_script.as_deref(),
                data.post_response_script.as_deref(),
                window,
                cx,
            );
        });

        cx.notify();
    }

    pub fn set_environments(
        &mut self,
        environments: &[EnvironmentToml],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let mut env_options = vec![EnvironmentOption::None];

        // Add each environment to the options
        for env in environments {
            env_options.push(EnvironmentOption::Environment(env.clone()));
        }

        // Create a new environment selector with updated options
        let new_environment_select = cx.new(|cx| {
            SelectState::new(
                env_options,
                Some(IndexPath::default().row(0)), // Select None by default
                window,
                cx,
            )
        });

        // Subscribe to environment selection changes for the new selector
        cx.subscribe(
            &new_environment_select,
            |this, _state, _event: &SelectEvent<Vec<EnvironmentOption>>, cx| {
                this.on_environment_change(cx);
            },
        )
        .detach();

        self.environment_select = new_environment_select;

        cx.notify();
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
        let mut base_url = url.to_string();
        let mut query_params = Vec::new();

        // Check if URL has query parameters
        if let Some(query_start) = url.find('?') {
            // Extract the base URL (before ?)
            base_url = url[..query_start].to_string();

            // Extract query string (after ?, before # if present)
            let query_part = if let Some(fragment_start) = url.find('#') {
                // Fragment comes after query
                if fragment_start > query_start {
                    &url[query_start + 1..fragment_start]
                } else {
                    &url[query_start + 1..]
                }
            } else {
                &url[query_start + 1..]
            };

            // Parse query parameters
            for pair in query_part.split('&') {
                if let Some(eq_pos) = pair.find('=') {
                    let key = &pair[..eq_pos];
                    let value = &pair[eq_pos + 1..];

                    if !key.is_empty() {
                        query_params.push(KeyValuePair {
                            key: urlencoding::decode(key)
                                .unwrap_or_else(|_| key.to_string().into())
                                .to_string(),
                            value: urlencoding::decode(value)
                                .unwrap_or_else(|_| value.to_string().into())
                                .to_string(),
                            enabled: true,
                        });
                    }
                } else if !pair.is_empty() {
                    // Parameter without value
                    query_params.push(KeyValuePair {
                        key: urlencoding::decode(pair)
                            .unwrap_or_else(|_| pair.to_string().into())
                            .to_string(),
                        value: String::new(),
                        enabled: true,
                    });
                }
            }
        }

        (base_url, query_params)
    }

    pub fn get_request_data(&self, cx: &Context<Self>) -> RequestData {
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
            .read_with(cx, |editor, cx| editor.get_path_parameters(cx));

        let editor_query_params = self
            .query_param_editor
            .read_with(cx, |editor, cx| editor.get_query_parameters(cx));

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
            .read_with(cx, |editor, cx| editor.get_headers(cx));

        let (pre_request_script, post_response_script) =
            self.script_editor.read_with(cx, |editor, cx| {
                (
                    editor.get_pre_request_script(cx),
                    editor.get_post_response_script(cx),
                )
            });

        // Update body from input
        data.body = self.body_input.read(cx).value().to_string();
        data.path_params = path_params;
        data.query_params = query_params;
        data.headers = headers;
        data.pre_request_script = pre_request_script;
        data.post_response_script = post_response_script;

        // Update Content-Type header based on selected content type
        if let Some(selected_content_type) = self.content_type_select.read(cx).selected_value() {
            let content_type_value = selected_content_type.as_str();

            // Find existing Content-Type header and update it, or add it if it doesn't exist
            if let Some(header) = data
                .headers
                .iter_mut()
                .find(|h| h.key.to_lowercase() == "content-type")
            {
                header.value = content_type_value.to_string();
            } else {
                data.headers.push(KeyValuePair {
                    key: "Content-Type".to_string(),
                    value: content_type_value.to_string(),
                    enabled: true,
                });
            }
        }

        data
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

        false
    }

    fn schedule_dirty_check(&mut self, cx: &mut Context<Self>) {
        self._dirty_check_task = cx.spawn(
            async move |weak_entity: WeakEntity<RequestEditor>, async_cx| {
                smol::Timer::after(std::time::Duration::from_millis(1)).await;
                let _ = weak_entity.update(async_cx, |this: &mut RequestEditor, cx| {
                    let is_now_dirty = this.is_dirty(cx);
                    if is_now_dirty != this._was_dirty {
                        this._was_dirty = is_now_dirty;
                        cx.emit(RequestEditorEvent::DirtyStateChanged {
                            is_dirty: is_now_dirty,
                        });
                    }
                });
            },
        );
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

        cx.notify();
    }

    fn send_request(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.is_loading {
            window.push_notification(
                (NotificationType::Warning, "Request is already in progress"),
                cx,
            );
            return;
        }

        // Set loading state
        self.is_loading = true;
        cx.notify();

        // Get the current request data
        let mut request_data = self.get_request_data(cx);

        // Check if Form Data content type is selected and update form data
        if let Some(selected_content_type) = self.content_type_select.read(cx).selected_value()
            && selected_content_type == &ContentType::Form
        {
            let form_data = self.form_editor.read(cx).get_form_data(cx);
            // Convert form data to body format (key=value pairs, files as @path)
            let form_body = form_data
                .iter()
                .filter(|field| field.enabled && !field.key.is_empty())
                .map(|field| {
                    if field.value.starts_with('@') {
                        // File reference
                        format!("{}={}", field.key, field.value)
                    } else {
                        // Regular form field
                        format!("{}={}", url_encode(&field.key), url_encode(&field.value))
                    }
                })
                .collect::<Vec<_>>()
                .join("&");
            request_data.body = form_body;
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

        // Load environment variables and secrets in the main thread
        let (variables, secrets) = if let Some(selected_env) = self.get_selected_environment(cx) {
            let env_resolver = crate::environment_resolver::EnvironmentResolver::new();
            let env_name = selected_env.name.clone();
            let selected_env_clone = selected_env.clone();
            let collection_manager = CollectionManager::global(cx);

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

        // Get the HTTP client after updating UI to avoid borrow issues
        let http_client = crate::http_client::HttpClientService::global(cx);

        // Clone necessary data before moving into async closure
        let collection_path = self.collection_path.clone();
        let selected_env_for_later = self.get_selected_environment(cx);
        let response_input = self.response_input.clone();
        let raw_response_input = self.raw_response_input.clone();
        let editor_entity = cx.entity().clone();

        // Execute request using async-compat and GPUI's spawn
        let request_data_clone1 = final_request_data.clone();
        let http_client_clone = http_client.clone();

        cx.spawn_in(window, async move |_this, window| {
            match async_compat::Compat::new(
                http_client_clone
                    .send_request(request_data_clone1, variables, secrets)
            ).await {
                Ok((response_data, variable_store)) => {
                    // Check for dirty environment variables
                    let dirty_vars = variable_store.get_dirty_env_vars();
                    if !dirty_vars.is_empty() {
                        tracing::info!("Environment variables modified by scripts: {:?}", dirty_vars.keys().collect::<Vec<_>>());
                        tracing::info!("Dirty variables that need to be persisted: {:?}", dirty_vars);
                        if let (Some(collection_path), Some(selected_env)) = (collection_path.as_ref(), selected_env_for_later) {
                            // Update the CollectionManager with dirty variables
                            match window.update_global(|collection_manager: &mut CollectionManager, _window, cx| {
                                collection_manager.update_environment_variables(collection_path, selected_env.name.as_str(), &dirty_vars, cx)
                            }) {
                                Ok(inner_result) => {
                                    match inner_result {
                                        Ok(()) => {
                                            tracing::info!("Successfully updated {} environment variables in CollectionManager", dirty_vars.len());
                                        }
                                        Err(e) => {
                                            tracing::error!("Failed to update environment variables in CollectionManager: {}", e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    tracing::error!("Failed to access CollectionManager for update: {}", e);
                                }
                            }
                        } else {
                            tracing::warn!("No collection or environment selected, cannot update dirty variables");
                        }
                    }

                    // Successfully got response data
                    window.update(|window, cx| {
                        // Update the RequestEditor's response_data for status bar and reset loading state
                        editor_entity.update(cx, |request_editor, cx| {
                            request_editor.response_data = response_data.clone();
                            request_editor.is_loading = false;
                            cx.notify();
                        });

                        // Detect content type and get language for syntax highlighting
                        let format = ResponseFormat::detect_from_content(
                            &response_data.body,
                            &response_data.headers,
                        );
                        let language = format.language_string();

                        tracing::info!("Response format detected: {:?}, {}", format, language);

                        // Format content (pretty print JSON if applicable)
                        let formatted_content = format.format_content(&response_data.body);

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
                    // HTTP request failed - error is already HttpError
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
                            body: http_error.details.clone(),
                            url: None, // No URL available for error responses
                        };

                        // Update the RequestEditor's response_data for status bar and reset loading state
                        editor_entity.update(cx, |request_editor, cx| {
                            request_editor.response_data = response_data.clone();
                            request_editor.is_loading = false;
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
        })
        .detach();
    }

    fn save_request(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Get current request data
        let request_data = self.get_request_data(cx);

        // Get request name from the request data
        let request_name = if request_data.name.is_empty() || request_data.name == "New Request" {
            "untitled_request".to_string()
        } else {
            request_data.name.clone()
        };

        if let Some(ref collection_path) = self.collection_path {
            // Use cx.update_global to call the save_request method on CollectionManager
            let group_path_ref = self.group_path.as_deref();
            let save_result =
                cx.update_global(|collection_manager: &mut CollectionManager, _cx| {
                    collection_manager.save_request(
                        collection_path,
                        &request_data,
                        &request_name,
                        group_path_ref,
                    )
                });

            match save_result {
                Ok(()) => {
                    tracing::info!("Request saved successfully");
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
                }
                Err(e) => {
                    tracing::error!("Failed to save request: {}", e);
                }
            }
        } else {
            tracing::warn!("Cannot save request: no collection_id set");
        }
    }

    fn render_url_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_method = self
            .method_select
            .read(cx)
            .selected_value()
            .unwrap_or(&HttpMethod::Get);

        let method_color = selected_method.get_color(cx);

        h_flex()
            .gap_3()
            .w_full()
            .child(
                div()
                    .w(px(120.))
                    .font_bold()
                    .font_family(cx.theme().mono_font_family.clone())
                    .w(px(120.))
                    .child(Select::new(&self.method_select).text_color(method_color)),
            )
            .child(
                div()
                    .w(px(160.))
                    .child(Select::new(&self.environment_select)),
            )
            .child(
                div()
                    .flex_1()
                    .on_key_down(cx.listener(|this, evt: &gpui::KeyDownEvent, window, cx| {
                        if evt.keystroke.should_match(&this.send_keystroke) {
                            this.send_request(window, cx);
                        }
                    }))
                    .child(
                        Input::new(&self.url_input)
                            .cleanable(true)
                            .font_family(cx.theme().mono_font_family.clone())
                            .text_sm(),
                    ),
            )
            .child(
                Button::new("send-request")
                    .outline()
                    .icon(IconName::Send)
                    .loading(self.is_loading)
                    .loading_icon(IconName::LoaderCircle)
                    .on_click(cx.listener(|this, _, window, cx| {
                        this.send_request(window, cx);
                    })),
            )
    }

    fn render_request_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        // Calculate badge counts
        let query_count = self.get_query_badge_count(cx);
        let has_body = self.has_body_content(cx);
        let headers_count = self.get_headers_badge_count(cx);
        let path_count = self.get_path_badge_count(cx);
        let scripts_count = self.get_scripts_badge_count(cx);

        TabBar::new("request-tabs")
            .left(px(-1.)) // Avoid double border with container
            .selected_index(match self.active_tab {
                RequestTab::Query => 0,
                RequestTab::Body => 1,
                RequestTab::Headers => 2,
                RequestTab::Path => 3,
                RequestTab::Scripts => 4,
            })
            .on_click(cx.listener(|this, &index, _, cx| {
                this.active_tab = match index {
                    0 => RequestTab::Query,
                    1 => RequestTab::Body,
                    2 => RequestTab::Headers,
                    3 => RequestTab::Path,
                    4 => RequestTab::Scripts,
                    _ => RequestTab::Path,
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
            .child(Tab::new().label("Path").when(path_count > 0, |tab| {
                tab.pr_2().suffix(TabBadge::new().count(path_count))
            }))
            .child(Tab::new().label("Scripts").when(scripts_count > 0, |tab| {
                tab.pr_2().suffix(TabBadge::new().count(scripts_count))
            }))
    }

    fn render_tab_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        match self.active_tab {
            RequestTab::Path => div().flex_1().child(self.path_param_editor.clone()),
            RequestTab::Query => div().flex_1().child(self.query_param_editor.clone()),
            RequestTab::Body => {
                let selected_content_type =
                    self.content_type_select.read(cx).selected_value().copied();

                div().flex_1().child(
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
                                ),
                        )
                        .child(match selected_content_type {
                            Some(ContentType::Form) => div()
                                .flex_1()
                                .child(self.form_editor.clone())
                                .into_any_element(),
                            _ => Input::new(&self.body_input)
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
            RequestTab::Headers => div().flex_1().child(self.header_editor.clone()),
            RequestTab::Scripts => div().flex_1().child(self.script_editor.clone()),
        }
    }

    fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
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

        let size_text = if let Some(size) = self.response_data.size {
            if size >= 1024 {
                format!(" {:.1}KB", size as f64 / 1024.0)
            } else {
                format!(" {}B", size)
            }
        } else {
            String::new()
        };

        h_flex()
            .justify_between()
            .p_3()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
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
            .child(size_text)
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
                            .outline()
                            .compact()
                            .label("Save Request")
                            .icon(IconName::Save)
                            .children(vec![
                                Kbd::new(Keystroke::parse("secondary-s").unwrap())
                                    .into_any_element(),
                            ])
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.save_request(window, cx);
                            })),
                    ),
            )
    }

    fn render_response_area(
        &self,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        div()
            .flex_1()
            .border_t_1()
            .border_color(cx.theme().border)
            .bg(cx.theme().background)
            .child(
                v_flex()
                    .size_full()
                    .child(
                        // Response tabs
                        TabBar::new("response-tabs")
                            .left(px(-1.)) // Avoid double border with container
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
                            .child(Tab::new().label("Raw")),
                    )
                    .child(
                        // Response content
                        div().flex_1().child(match self.active_response_tab {
                            ResponseTab::Response => div()
                                .h_full()
                                .child(
                                    Input::new(&self.response_input)
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_size(px(12.))
                                        .h_full()
                                        .py_3()
                                        .bordered(false)
                                        .rounded_none()
                                        .cleanable(true),
                                )
                                .into_any_element(),
                            ResponseTab::Raw => div()
                                .h_full()
                                .child(
                                    Input::new(&self.raw_response_input)
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_size(px(12.))
                                        .h_full()
                                        .py_3()
                                        .bordered(false)
                                        .rounded_none()
                                        .cleanable(true),
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

                    // Don't update query params if we're currently updating URL from params
                    if this._updating_url_from_params {
                        return;
                    }

                    let current_url = this.url_input.read(cx).value().to_string();
                    let parsed_params = this.parse_query_params_from_url(&current_url);

                    // Only update query parameters if this is a genuine URL change (not from parameter editor)
                    query_param_editor.update(cx, |editor, cx| {
                        let existing_params = editor.get_query_parameters(cx);

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
                            editor.set_parameters(&parsed_params, window, cx);
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

                            editor.set_parameters(&merged_params, window, cx);
                        }
                    });

                    // Schedule dirty state check after URL change
                    this.schedule_dirty_check(cx);
                }
            }
        });
        self._subscriptions.push(url_subscription);

        // Set up subscription for query parameter changes
        let query_param_subscription = cx.subscribe_in(&self.query_param_editor, window, {
            move |this: &mut Self, _editor, event: &QueryParamEvent, window, cx| {
                match event {
                    QueryParamEvent::ParameterChanged => {
                        let current_params =
                            this.query_param_editor.read(cx).get_query_parameters(cx);
                        let current_url = this.url_input.read(cx).value().to_string();
                        let new_url =
                            this.build_url_with_query_params(&current_url, &current_params);

                        // Set flag to prevent URL change from triggering query param update
                        this._updating_url_from_params = true;
                        this.url_input.update(cx, |state, cx| {
                            state.set_value(new_url, window, cx);
                        });
                        this._updating_url_from_params = false;

                        // Schedule dirty state check after query param change
                        this.schedule_dirty_check(cx);
                    }
                }
            }
        });
        self._subscriptions.push(query_param_subscription);

        // Set up subscription for name input changes (for dirty state)
        let name_input = self.name_input.clone();
        let name_subscription =
            cx.subscribe(&name_input, |this, _input, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    this.schedule_dirty_check(cx);
                }
            });
        self._subscriptions.push(name_subscription);

        // Set up subscription for method select changes (for dirty state)
        let method_select = self.method_select.clone();
        let method_subscription = cx.subscribe(
            &method_select,
            |this, _state, _event: &SelectEvent<Vec<HttpMethod>>, cx| {
                this.schedule_dirty_check(cx);
            },
        );
        self._subscriptions.push(method_subscription);

        // Set up subscriptions for all editors to track dirty state changes
        // Subscribe to path parameter editor changes
        let path_param_subscription = cx.subscribe(
            &self.path_param_editor,
            |this, _editor, _: &PathParamEvent, cx| {
                this.schedule_dirty_check(cx);
                cx.notify();
            },
        );
        self._subscriptions.push(path_param_subscription);

        // Subscribe to header editor changes
        let header_subscription = cx.subscribe(
            &self.header_editor,
            |this, _editor, _: &HeaderEditorEvent, cx| {
                this.schedule_dirty_check(cx);
                cx.notify();
            },
        );
        self._subscriptions.push(header_subscription);

        // Subscribe to script editor changes
        let script_subscription = cx.subscribe(
            &self.script_editor,
            |this, _editor, _: &ScriptEditorEvent, cx| {
                this.schedule_dirty_check(cx);
                cx.notify();
            },
        );
        self._subscriptions.push(script_subscription);

        // Subscribe to form editor changes
        let form_subscription = cx.subscribe(
            &self.form_editor,
            |this, _editor, _: &FormEditorEvent, cx| {
                this.schedule_dirty_check(cx);
                cx.notify();
            },
        );
        self._subscriptions.push(form_subscription);

        // Subscribe to body input changes
        let body_subscription =
            cx.subscribe(&self.body_input, |this, _input, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    this.schedule_dirty_check(cx);
                    cx.notify();
                }
            });
        self._subscriptions.push(body_subscription);

        // Subscribe to content type changes (affects body badge)
        let content_type_subscription = cx.subscribe(
            &self.content_type_select,
            |this, _state, _event: &SelectEvent<Vec<ContentType>>, cx| {
                this.schedule_dirty_check(cx);
                cx.notify();
            },
        );
        self._subscriptions.push(content_type_subscription);
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

        // Parse query parameters - only accept well-formed key=value pairs
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
            .get_query_parameters(cx)
            .iter()
            .filter(|param| param.enabled && !param.key.is_empty())
            .count()
    }

    /// Calculate if Body tab should show a badge (has content)
    fn has_body_content(&self, cx: &App) -> bool {
        let selected_content_type = self.content_type_select.read(cx).selected_value().copied();

        match selected_content_type {
            Some(ContentType::Form) => {
                // Check if form editor has any non-empty fields
                self.form_editor
                    .read(cx)
                    .get_form_data(cx)
                    .iter()
                    .any(|field| {
                        field.enabled && (!field.key.is_empty() || !field.value.is_empty())
                    })
            }
            Some(ContentType::UrlEncoded) => {
                // Check if form editor has any non-empty fields
                self.form_editor
                    .read(cx)
                    .get_form_data(cx)
                    .iter()
                    .any(|field| {
                        field.enabled && (!field.key.is_empty() || !field.value.is_empty())
                    })
            }
            _ => {
                // For other content types, check if body has content
                !self.body_input.read(cx).value().trim().is_empty()
            }
        }
    }

    /// Calculate badge count for Headers tab (number of enabled headers)
    fn get_headers_badge_count(&self, cx: &App) -> usize {
        self.header_editor
            .read(cx)
            .get_headers(cx)
            .iter()
            .filter(|header| header.enabled && !header.key.is_empty())
            .count()
    }

    /// Calculate badge count for Path tab (number of enabled path params)
    fn get_path_badge_count(&self, cx: &App) -> usize {
        self.path_param_editor
            .read(cx)
            .get_path_parameters(cx)
            .iter()
            .filter(|param| param.enabled && !param.key.is_empty())
            .count()
    }

    /// Calculate badge count for Scripts tab (number of scripts)
    fn get_scripts_badge_count(&self, cx: &App) -> usize {
        let script_editor = self.script_editor.read(cx);
        let pre_request = script_editor.get_pre_request_script(cx);
        let post_response = script_editor.get_post_response_script(cx);

        let count = if pre_request.is_some() && !pre_request.unwrap().trim().is_empty() {
            1
        } else {
            0
        };

        if post_response.is_some() && !post_response.unwrap().trim().is_empty() {
            count + 1
        } else {
            count
        }
    }
}

impl Render for RequestEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .key_context(CONTEXT)
            .on_action(cx.listener(|this: &mut RequestEditor, &Save, window, cx| {
                tracing::info!("Save action triggered");
                this.save_request(window, cx);
            }))
            .size_full()
            .bg(cx.theme().background)
            .child(
                // URL bar with method selector and buttons
                div()
                    .p_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(self.render_url_bar(cx)),
            )
            .child(
                // Request configuration tabs
                self.render_request_tabs(cx),
            )
            .child(
                // Tab content area
                self.render_tab_content(cx),
            )
            .child(
                // Response area
                self.render_response_area(window, cx),
            )
            .child(
                // Status bar
                self.render_status_bar(cx),
            )
    }
}

impl ResponseData {
    pub fn format_raw_response(self) -> String {
        let mut raw_output = String::new();

        // Add request URL at the top
        if let Some(url) = self.url {
            raw_output.push_str(&format!("{}\n", url));
        }

        // Add status line
        if let Some(status_code) = self.status_code {
            let status_text = self.status_text.as_deref().unwrap_or("Unknown");
            raw_output.push_str(&format!("{} {}\n", status_code, status_text));
        }

        // Add headers
        for header in &self.headers {
            if header.enabled {
                raw_output.push_str(&format!("{}: {}\n", header.key, header.value));
            }
        }

        // Add empty line between headers and body
        if !self.headers.is_empty() && !self.body.is_empty() {
            raw_output.push('\n');
        }

        // Add body
        if !self.body.is_empty() {
            raw_output.push_str(&self.body);
        }

        raw_output
    }
}
