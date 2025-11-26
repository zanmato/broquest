use gpui::{SharedString, *};
use gpui_component::{
    ActiveTheme, IndexPath, Sizable,
    button::Button,
    h_flex,
    input::{Input, InputState},
    select::{Select, SelectEvent, SelectItem, SelectState},
    tab::{Tab, TabBar},
    v_flex,
};
use gpui_tokio::Tokio;

use crate::app_events::AppEvent;
use crate::collection_manager::CollectionManager;
use crate::collection_types::EnvironmentToml;
use crate::header_editor::HeaderEditor;
use crate::http_client::ResponseFormat;
use crate::icon::IconName;
use crate::path_parameter_editor::PathParamEditor;
use crate::query_parameter_editor::QueryParamEditor;
use crate::script_editor::ScriptEditor;
use serde::{Deserialize, Serialize};
use std::time::Duration;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
    Head,
    Options,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ContentType {
    Json,
    Xml,
    Text,
    Html,
    Form,
    UrlEncoded,
}

impl HttpMethod {
    pub const ALL: [HttpMethod; 7] = [
        HttpMethod::Get,
        HttpMethod::Post,
        HttpMethod::Put,
        HttpMethod::Delete,
        HttpMethod::Patch,
        HttpMethod::Head,
        HttpMethod::Options,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
        }
    }

    /// Get color for HTTP method
    pub fn get_color(&self, cx: &mut App) -> gpui::Rgba {
        match self {
            HttpMethod::Get => cx.theme().green.into(),
            HttpMethod::Post => cx.theme().blue.into(),
            HttpMethod::Put => cx.theme().yellow.into(),
            HttpMethod::Delete => cx.theme().red.into(),
            HttpMethod::Patch => cx.theme().yellow.into(),
            HttpMethod::Head => cx.theme().blue.into(),
            HttpMethod::Options => cx.theme().cyan.into(),
        }
    }
}

impl ContentType {
    pub const ALL: [ContentType; 6] = [
        ContentType::Json,
        ContentType::Xml,
        ContentType::Text,
        ContentType::Html,
        ContentType::Form,
        ContentType::UrlEncoded,
    ];

    pub fn as_str(&self) -> &'static str {
        match self {
            ContentType::Json => "application/json",
            ContentType::Xml => "application/xml",
            ContentType::Text => "text/plain",
            ContentType::Html => "text/html",
            ContentType::Form => "application/x-www-form-urlencoded",
            ContentType::UrlEncoded => "application/x-www-form-urlencoded",
        }
    }

    pub fn language(&self) -> &'static str {
        match self {
            ContentType::Json => "json",
            ContentType::Xml => "xml",
            ContentType::Text => "text",
            ContentType::Html => "html",
            ContentType::Form => "text",
            ContentType::UrlEncoded => "text",
        }
    }
}

impl SelectItem for ContentType {
    type Value = ContentType;

    fn title(&self) -> SharedString {
        match self {
            ContentType::Json => "JSON".into(),
            ContentType::Xml => "XML".into(),
            ContentType::Text => "Plain Text".into(),
            ContentType::Html => "HTML".into(),
            ContentType::Form => "Form Data".into(),
            ContentType::UrlEncoded => "URL Encoded".into(),
        }
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

impl SelectItem for HttpMethod {
    type Value = HttpMethod;

    fn title(&self) -> SharedString {
        self.as_str().into()
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

pub struct RequestEditor {
    _focus_handle: FocusHandle,
    request_data: RequestData,
    response_data: ResponseData,
    active_tab: RequestTab,
    active_response_tab: ResponseTab,
    is_loading: bool,
    collection_id: Option<i64>,
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
    script_editor: Entity<ScriptEditor>,
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

        let url_input = cx.new(|cx| InputState::new(window, cx).placeholder("Enter request URL"));

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

        let script_editor = cx.new(|cx| ScriptEditor::new(window, cx));

        // Subscribe to Content-Type selection changes
        cx.subscribe(
            &content_type_select,
            |this, _state, _event: &SelectEvent<Vec<ContentType>>, cx| {
                this.on_content_type_change(cx);
            },
        )
        .detach();

        // Subscribe to environment selection changes
        cx.subscribe(
            &environment_select,
            |this, _state, _event: &SelectEvent<Vec<EnvironmentOption>>, cx| {
                this.on_environment_change(cx);
            },
        )
        .detach();

        Self {
            _focus_handle: cx.focus_handle(),
            request_data: RequestData::default(),
            response_data: ResponseData::default(),
            active_tab: RequestTab::Query,
            active_response_tab: ResponseTab::Response,
            is_loading: false,
            collection_id: None,
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
            script_editor,
        }
    }

    pub fn set_collection_id(&mut self, collection_id: Option<i64>) {
        self.collection_id = collection_id;
    }

    pub fn set_request_data(
        &mut self,
        data: RequestData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.request_data = data.clone();

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
            state.set_value(data.url.clone(), window, cx);
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

    pub fn get_request_data(&self, cx: &Context<Self>) -> RequestData {
        let mut data = self.request_data.clone();

        // Update method from select
        if let Some(selected_method) = self.method_select.read(cx).selected_value() {
            data.method = *selected_method;
        }

        // Update URL from input
        data.url = self.url_input.read(cx).value().to_string();

        // Update name from input
        data.name = self.name_input.read(cx).value().to_string();

        let path_params = self
            .path_param_editor
            .read_with(cx, |editor, cx| editor.get_path_parameters(cx));

        let query_params = self
            .query_param_editor
            .read_with(cx, |editor, cx| editor.get_query_parameters(cx));

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

            tracing::info!("Content-Type changed to: {}", content_type.as_str());
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

        // TODO: Apply environment variable resolution to request data
        // This would integrate with the environment resolver later

        cx.notify();
    }

    fn send_request(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Set loading state
        self.is_loading = true;
        cx.notify();

        // Get the current request data
        let request_data = self.get_request_data(cx);

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

            match env_resolver.load_environment_data("", &env_name, &[selected_env_clone], cx) {
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
        };

        // Get the HTTP client after updating UI to avoid borrow issues
        let http_client = crate::http_client::HttpClientService::global(cx);

        // Execute request using Tokio runtime and return data to UI
        let request_data_clone1 = final_request_data.clone();
        let http_client_clone = http_client.clone();

        // Create the task using Tokio::spawn_result - this returns a Task directly
        let task = Tokio::spawn_result(cx, async move {
            http_client_clone
                .send_request(request_data_clone1, variables, secrets)
                .await
        });

        // Clone necessary data before moving into async closure
        let collection_id = self.collection_id;
        let selected_env_for_later = self.get_selected_environment(cx);

        // Spawn a GPUI task to wait for the HTTP response
        let response_input = self.response_input.clone();
        let raw_response_input = self.raw_response_input.clone();
        let editor_entity = cx.entity().clone();
        cx.spawn_in(window, async move |_this, window| {
            match task.await {
                Ok((response_data, variable_store)) => {
                    // Check for dirty environment variables
                    let dirty_vars = variable_store.get_dirty_env_vars();
                    if !dirty_vars.is_empty() {
                        tracing::info!("Environment variables modified by scripts: {:?}", dirty_vars.keys().collect::<Vec<_>>());
                        tracing::info!("Dirty variables that need to be persisted: {:?}", dirty_vars);
                        if let (Some(collection_id), Some(selected_env)) = (collection_id, selected_env_for_later) {
                            tracing::info!("Collection ID: {}, Environment: {}", collection_id, selected_env.name);
                            // Update the CollectionManager with dirty variables
                            match window.update_global(|collection_manager: &mut CollectionManager, _window, cx| {
                                collection_manager.update_environment_variables(collection_id, selected_env.name.as_str(), &dirty_vars, cx)
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
                    // HTTP request failed
                    window.update(|window, cx| {
                        // Create error response
                        let response_data = ResponseData {
                            status_code: None,
                            status_text: Some("Error".to_string()),
                            latency: None,
                            size: Some(error.to_string().len()),
                            headers: vec![],
                            body: format!("Request failed: {}", error),
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
                    })?;
                }
            }
            Ok::<(), anyhow::Error>(())
        })
        .detach();
    }

    fn save_request(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        // Get current request data
        let request_data = self.get_request_data(cx);

        // Get request name from the request data
        let request_name = if request_data.name.is_empty() || request_data.name == "New Request" {
            "untitled_request"
        } else {
            &request_data.name
        };

        if let Some(collection_id) = self.collection_id {
            // Use cx.update_global to call the save_request method on CollectionManager
            cx.update_global(|collection_manager: &mut CollectionManager, _cx| {
                match collection_manager.save_request(collection_id, &request_data, request_name) {
                    Ok(()) => {
                        tracing::info!("Request saved successfully");
                    }
                    Err(e) => {
                        tracing::error!("Failed to save request: {}", e);
                    }
                }
            });
        } else {
            tracing::warn!("Cannot save request: no collection_id set");
        }
    }

    fn render_url_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_3()
            .w_full()
            .child(div().w(px(120.)).child(Select::new(&self.method_select)))
            .child(
                div()
                    .w(px(160.))
                    .child(Select::new(&self.environment_select)),
            )
            .child(
                div().flex_1().child(
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
            .child(Tab::new().label("Query"))
            .child(Tab::new().label("Body"))
            .child(Tab::new().label("Headers"))
            .child(Tab::new().label("Path"))
            .child(Tab::new().label("Scripts"))
    }

    fn render_tab_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        match self.active_tab {
            RequestTab::Path => div().flex_1().child(self.path_param_editor.clone()),
            RequestTab::Query => div().flex_1().child(self.query_param_editor.clone()),
            RequestTab::Body => div().flex_1().child(
                v_flex()
                    .gap_3()
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
                    .child(
                        Input::new(&self.body_input)
                            .font_family(cx.theme().mono_font_family.clone())
                            .text_size(px(12.))
                            .h_full()
                            .bordered(false)
                            .rounded_none(),
                    ),
            ),
            RequestTab::Headers => div().flex_1().child(self.header_editor.clone()),
            RequestTab::Scripts => div().flex_1().child(self.script_editor.clone()),
        }
    }

    fn render_status_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let status_text = if let Some(status) = self.response_data.status_code {
            format!(
                "{} {}{}",
                status,
                self.response_data.status_text.as_deref().unwrap_or(""),
                if let Some(latency) = self.response_data.latency {
                    format!(" • {}ms", latency.as_millis())
                } else {
                    String::new()
                }
            )
        } else {
            "".to_string()
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
            .child(status_text)
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
}

impl EventEmitter<AppEvent> for RequestEditor {}

impl Render for RequestEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
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
