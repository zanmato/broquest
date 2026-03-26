use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, SharedString, Window,
    div, prelude::*, px,
};
use gpui_component::{
    ActiveTheme, h_flex,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    select::{Select, SelectItem, SelectState},
    v_flex,
};

use crate::domain::{
    AuthType, BasicAuth, DigestAuth, JwtAuth, KeyAuth, OAuth2Auth, OAuth2GrantType,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AuthEditorEvent {
    AuthChanged,
}

#[derive(Debug, Clone, PartialEq)]
pub enum AuthTypeOption {
    None,
    Inherit,
    Basic,
    Digest,
    Key,
    OAuth2,
    Jwt,
}

impl AuthTypeOption {
    pub fn all() -> &'static [AuthTypeOption] {
        static OPTIONS: &[AuthTypeOption] = &[
            AuthTypeOption::None,
            AuthTypeOption::Inherit,
            AuthTypeOption::Basic,
            AuthTypeOption::Digest,
            AuthTypeOption::Key,
            AuthTypeOption::OAuth2,
            AuthTypeOption::Jwt,
        ];
        OPTIONS
    }

    pub fn from_auth_type(auth_type: &AuthType) -> Self {
        match auth_type {
            AuthType::None => AuthTypeOption::None,
            AuthType::Inherit => AuthTypeOption::Inherit,
            AuthType::Basic(_) => AuthTypeOption::Basic,
            AuthType::Digest(_) => AuthTypeOption::Digest,
            AuthType::Key(_) => AuthTypeOption::Key,
            AuthType::OAuth2(_) => AuthTypeOption::OAuth2,
            AuthType::Jwt(_) => AuthTypeOption::Jwt,
        }
    }

    pub fn to_auth_type(&self) -> AuthType {
        match self {
            AuthTypeOption::None => AuthType::None,
            AuthTypeOption::Inherit => AuthType::Inherit,
            AuthTypeOption::Basic => AuthType::Basic(BasicAuth::default()),
            AuthTypeOption::Digest => AuthType::Digest(DigestAuth::default()),
            AuthTypeOption::Key => AuthType::Key(KeyAuth::default()),
            AuthTypeOption::OAuth2 => AuthType::OAuth2(OAuth2Auth::default()),
            AuthTypeOption::Jwt => AuthType::Jwt(JwtAuth::default()),
        }
    }

    pub fn name(&self) -> &'static str {
        self.to_auth_type().name()
    }
}

impl SelectItem for AuthTypeOption {
    type Value = AuthTypeOption;

    fn title(&self) -> SharedString {
        self.name().into()
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

pub struct AuthEditor {
    auth_type_options: Vec<AuthTypeOption>,
    auth_type_select: Entity<SelectState<Vec<AuthTypeOption>>>,
    username_input: Entity<InputState>,
    password_input: Entity<InputState>,
    header_input: Entity<InputState>,
    value_input: Entity<InputState>,
    client_id_input: Entity<InputState>,
    client_secret_input: Entity<InputState>,
    token_url_input: Entity<InputState>,
    scope_input: Entity<InputState>,
    // JWT inputs
    jwt_login_url_input: Entity<InputState>,
    jwt_username_field_input: Entity<InputState>,
    jwt_username_input: Entity<InputState>,
    jwt_password_field_input: Entity<InputState>,
    jwt_password_input: Entity<InputState>,
    jwt_token_field_input: Entity<InputState>,
    jwt_token_type_field_input: Entity<InputState>,
    jwt_expiry_field_input: Entity<InputState>,
    _subscriptions: Vec<gpui::Subscription>,
}

impl EventEmitter<AuthEditorEvent> for AuthEditor {}

impl AuthEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self::new_with_options(AuthTypeOption::all().to_vec(), window, cx)
    }

    pub fn new_without_inherit(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let options: Vec<AuthTypeOption> = AuthTypeOption::all()
            .iter()
            .filter(|o| !matches!(o, AuthTypeOption::Inherit))
            .cloned()
            .collect();
        Self::new_with_options(options, window, cx)
    }

    fn new_with_options(
        options: Vec<AuthTypeOption>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let auth_type_options = options.clone();
        let auth_type_select = cx.new(|cx| {
            SelectState::new(
                options,
                Some(gpui_component::IndexPath::default().row(0)),
                window,
                cx,
            )
        });

        let username_input = cx.new(|cx| InputState::new(window, cx).placeholder("Username"));

        let password_input = cx.new(|cx| InputState::new(window, cx).placeholder("Password"));

        let header_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Header name")
                .default_value("X-API-Key")
        });

        let value_input = cx.new(|cx| InputState::new(window, cx).placeholder("API key value"));

        let client_id_input = cx.new(|cx| InputState::new(window, cx).placeholder("Client ID"));

        let client_secret_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Client Secret"));

        let token_url_input = cx.new(|cx| InputState::new(window, cx).placeholder("Token URL"));

        let scope_input = cx.new(|cx| InputState::new(window, cx).placeholder("Scope (optional)"));

        // JWT inputs
        let jwt_login_url_input = cx.new(|cx| InputState::new(window, cx).placeholder("Login URL"));

        let jwt_username_field_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Username field name")
                .default_value("username")
        });

        let jwt_username_input = cx.new(|cx| InputState::new(window, cx).placeholder("Username"));

        let jwt_password_field_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Password field name")
                .default_value("password")
        });

        let jwt_password_input = cx.new(|cx| InputState::new(window, cx).placeholder("Password"));

        let jwt_token_field_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Token field name")
                .default_value("access_token")
        });

        let jwt_token_type_field_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Token type field")
                .default_value("token_type")
        });

        let jwt_expiry_field_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Expiry field")
                .default_value("expires_in")
        });

        let mut subscriptions = Vec::new();

        let subscribe_to_input = |input: &Entity<InputState>, cx: &mut Context<Self>| {
            cx.subscribe(input, |_this, _input, event: &InputEvent, cx| {
                if let InputEvent::Change = event {
                    cx.emit(AuthEditorEvent::AuthChanged);
                    cx.notify();
                }
            })
        };

        subscriptions.push(subscribe_to_input(&username_input, cx));
        subscriptions.push(subscribe_to_input(&password_input, cx));
        subscriptions.push(subscribe_to_input(&header_input, cx));
        subscriptions.push(subscribe_to_input(&value_input, cx));
        subscriptions.push(subscribe_to_input(&client_id_input, cx));
        subscriptions.push(subscribe_to_input(&client_secret_input, cx));
        subscriptions.push(subscribe_to_input(&token_url_input, cx));
        subscriptions.push(subscribe_to_input(&scope_input, cx));
        subscriptions.push(subscribe_to_input(&jwt_login_url_input, cx));
        subscriptions.push(subscribe_to_input(&jwt_username_field_input, cx));
        subscriptions.push(subscribe_to_input(&jwt_username_input, cx));
        subscriptions.push(subscribe_to_input(&jwt_password_field_input, cx));
        subscriptions.push(subscribe_to_input(&jwt_password_input, cx));
        subscriptions.push(subscribe_to_input(&jwt_token_field_input, cx));
        subscriptions.push(subscribe_to_input(&jwt_token_type_field_input, cx));
        subscriptions.push(subscribe_to_input(&jwt_expiry_field_input, cx));

        Self {
            auth_type_options,
            auth_type_select,
            username_input,
            password_input,
            header_input,
            value_input,
            client_id_input,
            client_secret_input,
            token_url_input,
            scope_input,
            jwt_login_url_input,
            jwt_username_field_input,
            jwt_username_input,
            jwt_password_field_input,
            jwt_password_input,
            jwt_token_field_input,
            jwt_token_type_field_input,
            jwt_expiry_field_input,
            _subscriptions: subscriptions,
        }
    }

    pub fn set_auth(&mut self, auth: &AuthType, window: &mut Window, cx: &mut Context<Self>) {
        let auth_type_option = AuthTypeOption::from_auth_type(auth);
        if let Some(index) = self
            .auth_type_options
            .iter()
            .position(|t| t == &auth_type_option)
        {
            self.auth_type_select.update(cx, |state, cx| {
                state.set_selected_index(
                    Some(gpui_component::IndexPath::default().row(index)),
                    window,
                    cx,
                );
            });
        }

        match auth {
            AuthType::Basic(basic) => {
                self.username_input.update(cx, |state, cx| {
                    state.set_value(basic.username.clone(), window, cx);
                });
                self.password_input.update(cx, |state, cx| {
                    state.set_value(basic.password.clone(), window, cx);
                });
            }
            AuthType::Digest(digest) => {
                self.username_input.update(cx, |state, cx| {
                    state.set_value(digest.username.clone(), window, cx);
                });
                self.password_input.update(cx, |state, cx| {
                    state.set_value(digest.password.clone(), window, cx);
                });
            }
            AuthType::Key(key) => {
                self.header_input.update(cx, |state, cx| {
                    state.set_value(key.header.clone(), window, cx);
                });
                self.value_input.update(cx, |state, cx| {
                    state.set_value(key.value.clone(), window, cx);
                });
            }
            AuthType::OAuth2(oauth) => {
                self.client_id_input.update(cx, |state, cx| {
                    state.set_value(oauth.client_id.clone(), window, cx);
                });
                self.client_secret_input.update(cx, |state, cx| {
                    state.set_value(oauth.client_secret.clone(), window, cx);
                });
                self.token_url_input.update(cx, |state, cx| {
                    state.set_value(oauth.token_url.clone(), window, cx);
                });
                self.scope_input.update(cx, |state, cx| {
                    state.set_value(oauth.scope.clone().unwrap_or_default(), window, cx);
                });
            }
            AuthType::Jwt(jwt) => {
                self.jwt_login_url_input.update(cx, |state, cx| {
                    state.set_value(jwt.login_url.clone(), window, cx);
                });
                self.jwt_username_field_input.update(cx, |state, cx| {
                    state.set_value(jwt.username_field.clone(), window, cx);
                });
                self.jwt_username_input.update(cx, |state, cx| {
                    state.set_value(jwt.username.clone(), window, cx);
                });
                self.jwt_password_field_input.update(cx, |state, cx| {
                    state.set_value(jwt.password_field.clone(), window, cx);
                });
                self.jwt_password_input.update(cx, |state, cx| {
                    state.set_value(jwt.password.clone(), window, cx);
                });
                self.jwt_token_field_input.update(cx, |state, cx| {
                    state.set_value(jwt.token_field.clone(), window, cx);
                });
                self.jwt_token_type_field_input.update(cx, |state, cx| {
                    state.set_value(jwt.token_type_field.clone(), window, cx);
                });
                self.jwt_expiry_field_input.update(cx, |state, cx| {
                    state.set_value(jwt.expiry_field.clone(), window, cx);
                });
            }
            AuthType::None | AuthType::Inherit => {}
        }

        cx.notify();
    }

    pub fn get_auth(&self, cx: &App) -> AuthType {
        let selected_type = self
            .auth_type_select
            .read(cx)
            .selected_value()
            .cloned()
            .unwrap_or(AuthTypeOption::None);

        match selected_type {
            AuthTypeOption::None => AuthType::None,
            AuthTypeOption::Inherit => AuthType::Inherit,
            AuthTypeOption::Basic => AuthType::Basic(BasicAuth {
                username: self.username_input.read(cx).value().to_string(),
                password: self.password_input.read(cx).value().to_string(),
            }),
            AuthTypeOption::Digest => AuthType::Digest(DigestAuth {
                username: self.username_input.read(cx).value().to_string(),
                password: self.password_input.read(cx).value().to_string(),
            }),
            AuthTypeOption::Key => AuthType::Key(KeyAuth {
                header: self.header_input.read(cx).value().to_string(),
                value: self.value_input.read(cx).value().to_string(),
            }),
            AuthTypeOption::OAuth2 => AuthType::OAuth2(OAuth2Auth {
                grant_type: OAuth2GrantType::ClientCredentials,
                client_id: self.client_id_input.read(cx).value().to_string(),
                client_secret: self.client_secret_input.read(cx).value().to_string(),
                token_url: self.token_url_input.read(cx).value().to_string(),
                scope: {
                    let scope = self.scope_input.read(cx).value().to_string();
                    if scope.is_empty() { None } else { Some(scope) }
                },
                authorize_url: None,
                redirect_url: None,
                access_token: None,
                refresh_token: None,
                expires_at: None,
            }),
            AuthTypeOption::Jwt => AuthType::Jwt(JwtAuth {
                login_url: self.jwt_login_url_input.read(cx).value().to_string(),
                username_field: self.jwt_username_field_input.read(cx).value().to_string(),
                username: self.jwt_username_input.read(cx).value().to_string(),
                password_field: self.jwt_password_field_input.read(cx).value().to_string(),
                password: self.jwt_password_input.read(cx).value().to_string(),
                token_field: self.jwt_token_field_input.read(cx).value().to_string(),
                token_type_field: self.jwt_token_type_field_input.read(cx).value().to_string(),
                expiry_field: self.jwt_expiry_field_input.read(cx).value().to_string(),
                access_token: None,
                token_type: None,
                expires_at: None,
            }),
        }
    }

    fn render_labeled_input(
        &self,
        label: impl Into<SharedString>,
        input: &Entity<InputState>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let label: SharedString = label.into();
        h_flex()
            .gap_3()
            .items_center()
            .child(
                div()
                    .w(px(140.))
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            )
            .child(
                div()
                    .flex_1()
                    .child(Input::new(input).font_family(cx.theme().mono_font_family.clone())),
            )
    }

    fn render_basic_auth(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .p_4()
            .child(self.render_labeled_input("Username", &self.username_input, cx))
            .child(self.render_labeled_input("Password", &self.password_input, cx))
    }

    fn render_digest_auth(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .p_4()
            .child(self.render_labeled_input("Username", &self.username_input, cx))
            .child(self.render_labeled_input("Password", &self.password_input, cx))
    }

    fn render_api_key(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .p_4()
            .child(self.render_labeled_input("Header", &self.header_input, cx))
            .child(self.render_labeled_input("Value", &self.value_input, cx))
    }

    fn render_oauth2(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .p_4()
            .child(self.render_labeled_input("Client ID", &self.client_id_input, cx))
            .child(self.render_labeled_input("Client Secret", &self.client_secret_input, cx))
            .child(self.render_labeled_input("Token URL", &self.token_url_input, cx))
            .child(self.render_labeled_input("Scope", &self.scope_input, cx))
    }

    fn render_jwt(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .p_4()
            .child(self.render_labeled_input("Login URL", &self.jwt_login_url_input, cx))
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .px_4()
                    .child("Request Configuration"),
            )
            .child(self.render_labeled_input("Username Field", &self.jwt_username_field_input, cx))
            .child(self.render_labeled_input("Username", &self.jwt_username_input, cx))
            .child(self.render_labeled_input("Password Field", &self.jwt_password_field_input, cx))
            .child(self.render_labeled_input("Password", &self.jwt_password_input, cx))
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .px_4()
                    .child("Response Field Mappings"),
            )
            .child(self.render_labeled_input("Token Field", &self.jwt_token_field_input, cx))
            .child(self.render_labeled_input(
                "Token Type Field",
                &self.jwt_token_type_field_input,
                cx,
            ))
            .child(self.render_labeled_input("Expiry Field", &self.jwt_expiry_field_input, cx))
    }

    fn render_inherit(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().gap_3().p_4().child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("Authentication will be inherited from the parent collection."),
        )
    }

    fn render_none(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().gap_3().p_4().child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child("No authentication will be used for this request."),
        )
    }
}

impl Render for AuthEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let selected_type = self
            .auth_type_select
            .read(cx)
            .selected_value()
            .cloned()
            .unwrap_or(AuthTypeOption::None);

        v_flex()
            .h_full()
            .child(
                h_flex()
                    .gap_3()
                    .items_center()
                    .p_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Auth Type"),
                    )
                    .child(div().w(px(240.)).child(Select::new(&self.auth_type_select))),
            )
            .child(div().size_full().flex_1().min_h_0().child(
                v_flex().overflow_y_scrollbar().child(match selected_type {
                    AuthTypeOption::None => self.render_none(cx).into_any_element(),
                    AuthTypeOption::Inherit => self.render_inherit(cx).into_any_element(),
                    AuthTypeOption::Basic => self.render_basic_auth(cx).into_any_element(),
                    AuthTypeOption::Digest => self.render_digest_auth(cx).into_any_element(),
                    AuthTypeOption::Key => self.render_api_key(cx).into_any_element(),
                    AuthTypeOption::OAuth2 => self.render_oauth2(cx).into_any_element(),
                    AuthTypeOption::Jwt => self.render_jwt(cx).into_any_element(),
                }),
            ))
    }
}

impl Focusable for AuthEditor {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        cx.focus_handle()
    }
}
