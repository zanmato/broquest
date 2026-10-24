use gpui::{
    App, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement, SharedString, Window,
    div, prelude::*, px,
};
use gpui_kit::component::{
    ActiveTheme, Disableable, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    select::{Select, SelectEvent, SelectItem, SelectState},
    switch::Switch,
    v_flex,
};

use super::key_value_editor::{KeyValueConfig, KeyValueEditor, KeyValueEvent};
use crate::domain::{
    AuthType, BasicAuth, DigestAuth, JwtAuth, KeyAuth, OAuth2Auth, OAuth2ClientAuth,
    OAuth2GrantType, OAuth2TokenKind,
};

#[derive(Debug, Clone, PartialEq)]
pub enum AuthEditorEvent {
    AuthChanged,
    /// The Get token button: fetch a new OAuth2 token for the settings shown.
    GetToken,
    /// The Clear token button: forget the cached OAuth2 token.
    ClearToken,
}

impl SelectItem for OAuth2GrantType {
    type Value = OAuth2GrantType;

    fn title(&self) -> SharedString {
        match self {
            Self::ClientCredentials => "Client Credentials",
            Self::AuthorizationCode => "Authorization Code",
            Self::Password => "Password",
        }
        .into()
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

impl SelectItem for OAuth2ClientAuth {
    type Value = OAuth2ClientAuth;

    fn title(&self) -> SharedString {
        match self {
            Self::Body => "In the request body",
            Self::BasicHeader => "As a Basic auth header",
        }
        .into()
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

impl SelectItem for OAuth2TokenKind {
    type Value = OAuth2TokenKind;

    fn title(&self) -> SharedString {
        match self {
            Self::Access => "Access token",
            Self::Id => "ID token",
            Self::Both => "Access and ID token",
        }
        .into()
    }

    fn value(&self) -> &Self::Value {
        self
    }
}

const GRANT_TYPES: [OAuth2GrantType; 2] = [
    OAuth2GrantType::ClientCredentials,
    OAuth2GrantType::AuthorizationCode,
];
const CLIENT_AUTHS: [OAuth2ClientAuth; 2] = [OAuth2ClientAuth::Body, OAuth2ClientAuth::BasicHeader];
const TOKEN_KINDS: [OAuth2TokenKind; 3] = [
    OAuth2TokenKind::Access,
    OAuth2TokenKind::Id,
    OAuth2TokenKind::Both,
];

/// Select `value` in a select of `options`.
fn select_option<T: SelectItem<Value = T> + PartialEq + Clone + 'static>(
    select: &Entity<SelectState<Vec<T>>>,
    options: &[T],
    value: &T,
    window: &mut Window,
    cx: &mut App,
) {
    let row = options.iter().position(|o| o == value).unwrap_or(0);
    select.update(cx, |state, cx| {
        state.set_selected_index(
            Some(gpui_kit::component::IndexPath::default().row(row)),
            window,
            cx,
        );
    });
}

fn new_select<T: SelectItem<Value = T> + PartialEq + Clone + 'static>(
    options: &[T],
    window: &mut Window,
    cx: &mut Context<AuthEditor>,
) -> Entity<SelectState<Vec<T>>> {
    let options = options.to_vec();
    cx.new(|cx| {
        SelectState::new(
            options,
            Some(gpui_kit::component::IndexPath::default().row(0)),
            window,
            cx,
        )
    })
}

/// The OAuth2 inputs beyond client ID, secret, token URL and scope.
struct OAuth2Inputs {
    grant_type: Entity<SelectState<Vec<OAuth2GrantType>>>,
    client_auth: Entity<SelectState<Vec<OAuth2ClientAuth>>>,
    token_kind: Entity<SelectState<Vec<OAuth2TokenKind>>>,
    authorize_url: Entity<InputState>,
    redirect_url: Entity<InputState>,
    state: Entity<InputState>,
    refresh_url: Entity<InputState>,
    header_name: Entity<InputState>,
    header_prefix: Entity<InputState>,
    id_token_header: Entity<InputState>,
    token_id: Entity<InputState>,
    extra_params: Entity<KeyValueEditor>,
    pkce: bool,
    encode_url: bool,
    auto_fetch: bool,
    auto_refresh: bool,
    /// What the last Get token or Clear token did.
    token_status: Option<SharedString>,
    fetching_token: bool,
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
    /// An auth scheme broquest cannot edit (imported from OpenCollection).
    /// Selectable only when a request already carries such auth; preserved as-is.
    Unsupported,
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
            AuthTypeOption::Unsupported,
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
            AuthType::Unsupported { .. } => AuthTypeOption::Unsupported,
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
            // The real preserved value is held by the editor; this is only a
            // fallback used if a user somehow selects it for a fresh request.
            AuthTypeOption::Unsupported => AuthType::None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            AuthTypeOption::Unsupported => "Unsupported",
            other => other.to_auth_type().name(),
        }
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
    focus_handle: FocusHandle,
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
    oauth2: OAuth2Inputs,
    /// Preserved definition for an OpenCollection auth scheme broquest can't edit.
    unsupported_auth: Option<AuthType>,
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
                Some(gpui_kit::component::IndexPath::default().row(0)),
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

        let defaults = OAuth2Auth::default();
        let input = |placeholder: &'static str,
                     value: &str,
                     window: &mut Window,
                     cx: &mut Context<Self>| {
            let value = value.to_string();
            cx.new(|cx| {
                InputState::new(window, cx)
                    .placeholder(placeholder)
                    .default_value(value)
            })
        };
        let oauth2 = OAuth2Inputs {
            grant_type: new_select(&GRANT_TYPES, window, cx),
            client_auth: new_select(&CLIENT_AUTHS, window, cx),
            token_kind: new_select(&TOKEN_KINDS, window, cx),
            authorize_url: input("Authorization URL", "", window, cx),
            redirect_url: input("Empty listens on a free port of 127.0.0.1", "", window, cx),
            state: input("Empty sends a random one", "", window, cx),
            refresh_url: input("Empty uses the token URL", "", window, cx),
            header_name: input("Authorization", &defaults.header_name, window, cx),
            header_prefix: input(
                "Empty sends the token alone",
                &defaults.header_prefix,
                window,
                cx,
            ),
            id_token_header: input("X-ID-Token", &defaults.id_token_header, window, cx),
            token_id: input(
                "Empty shares the token with requests that have the same settings",
                "",
                window,
                cx,
            ),
            extra_params: cx.new(|cx| {
                KeyValueEditor::new(
                    KeyValueConfig::new("oauth2-extra-params", "Field, such as audience", "Value")
                        .embedded(),
                    window,
                    cx,
                )
            }),
            pkce: defaults.pkce,
            encode_url: defaults.encode_url,
            auto_fetch: defaults.auto_fetch,
            auto_refresh: defaults.auto_refresh,
            token_status: None,
            fetching_token: false,
        };

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
        for input in [
            &oauth2.authorize_url,
            &oauth2.redirect_url,
            &oauth2.state,
            &oauth2.refresh_url,
            &oauth2.header_name,
            &oauth2.header_prefix,
            &oauth2.id_token_header,
            &oauth2.token_id,
        ] {
            subscriptions.push(subscribe_to_input(input, cx));
        }
        subscriptions.push(cx.subscribe(
            &oauth2.extra_params,
            |_this, _editor, _: &KeyValueEvent, cx| {
                cx.emit(AuthEditorEvent::AuthChanged);
                cx.notify();
            },
        ));
        subscriptions.push(cx.subscribe(
            &auth_type_select,
            |_this, _select, _: &SelectEvent<Vec<AuthTypeOption>>, cx| {
                cx.emit(AuthEditorEvent::AuthChanged);
                cx.notify();
            },
        ));
        subscriptions.push(cx.subscribe(
            &oauth2.grant_type,
            |_this, _select, _: &SelectEvent<Vec<OAuth2GrantType>>, cx| {
                cx.emit(AuthEditorEvent::AuthChanged);
                cx.notify();
            },
        ));
        subscriptions.push(cx.subscribe(
            &oauth2.client_auth,
            |_this, _select, _: &SelectEvent<Vec<OAuth2ClientAuth>>, cx| {
                cx.emit(AuthEditorEvent::AuthChanged);
                cx.notify();
            },
        ));
        subscriptions.push(cx.subscribe(
            &oauth2.token_kind,
            |_this, _select, _: &SelectEvent<Vec<OAuth2TokenKind>>, cx| {
                cx.emit(AuthEditorEvent::AuthChanged);
                cx.notify();
            },
        ));

        Self {
            focus_handle: cx.focus_handle(),
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
            oauth2,
            unsupported_auth: None,
            _subscriptions: subscriptions,
        }
    }

    pub fn set_auth(&mut self, auth: &AuthType, window: &mut Window, cx: &mut Context<Self>) {
        // Preserve any unmodeled (OpenCollection) auth so get_auth can return it
        // unchanged; clear it when a normal auth type is loaded.
        self.unsupported_auth = match auth {
            AuthType::Unsupported { .. } => Some(auth.clone()),
            _ => None,
        };
        let auth_type_option = AuthTypeOption::from_auth_type(auth);
        if let Some(index) = self
            .auth_type_options
            .iter()
            .position(|t| t == &auth_type_option)
        {
            self.auth_type_select.update(cx, |state, cx| {
                state.set_selected_index(
                    Some(gpui_kit::component::IndexPath::default().row(index)),
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
                self.set_oauth2(oauth, window, cx);
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
            AuthType::None | AuthType::Inherit | AuthType::Unsupported { .. } => {}
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
            AuthTypeOption::Unsupported => {
                // Return the preserved, unmodeled auth definition unchanged.
                self.unsupported_auth.clone().unwrap_or(AuthType::None)
            }
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
            AuthTypeOption::OAuth2 => AuthType::OAuth2(self.get_oauth2(cx)),
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

    fn set_oauth2(&mut self, oauth: &OAuth2Auth, window: &mut Window, cx: &mut Context<Self>) {
        let inputs = &self.oauth2;
        select_option(
            &inputs.grant_type,
            &GRANT_TYPES,
            &oauth.grant_type,
            window,
            cx,
        );
        select_option(
            &inputs.client_auth,
            &CLIENT_AUTHS,
            &oauth.client_auth,
            window,
            cx,
        );
        select_option(
            &inputs.token_kind,
            &TOKEN_KINDS,
            &oauth.token_to_send,
            window,
            cx,
        );
        for (input, value) in [
            (
                &inputs.authorize_url,
                oauth.authorize_url.clone().unwrap_or_default(),
            ),
            (
                &inputs.redirect_url,
                oauth.redirect_url.clone().unwrap_or_default(),
            ),
            (&inputs.state, oauth.state.clone().unwrap_or_default()),
            (
                &inputs.refresh_url,
                oauth.refresh_url.clone().unwrap_or_default(),
            ),
            (&inputs.header_name, oauth.header_name.clone()),
            (&inputs.header_prefix, oauth.header_prefix.clone()),
            (&inputs.id_token_header, oauth.id_token_header.clone()),
            (&inputs.token_id, oauth.token_id.clone()),
        ] {
            input.update(cx, |state, cx| state.set_value(value, window, cx));
        }
        inputs.extra_params.update(cx, |editor, cx| {
            editor.set_pairs(&oauth.extra_params, window, cx);
        });
        self.oauth2.pkce = oauth.pkce;
        self.oauth2.encode_url = oauth.encode_url;
        self.oauth2.auto_fetch = oauth.auto_fetch;
        self.oauth2.auto_refresh = oauth.auto_refresh;
        self.oauth2.token_status = None;
    }

    fn get_oauth2(&self, cx: &App) -> OAuth2Auth {
        let inputs = &self.oauth2;
        let text = |input: &Entity<InputState>| input.read(cx).value().to_string();
        let optional = |input: &Entity<InputState>| Some(text(input)).filter(|v| !v.is_empty());
        OAuth2Auth {
            grant_type: inputs
                .grant_type
                .read(cx)
                .selected_value()
                .cloned()
                .unwrap_or_default(),
            client_id: text(&self.client_id_input),
            client_secret: text(&self.client_secret_input),
            token_url: text(&self.token_url_input),
            scope: optional(&self.scope_input),
            authorize_url: optional(&inputs.authorize_url),
            redirect_url: optional(&inputs.redirect_url),
            refresh_url: optional(&inputs.refresh_url),
            client_auth: inputs
                .client_auth
                .read(cx)
                .selected_value()
                .copied()
                .unwrap_or_default(),
            extra_params: inputs
                .extra_params
                .read(cx)
                .get_pairs(cx)
                .into_iter()
                .filter(|p| !p.key.is_empty())
                .collect(),
            state: optional(&inputs.state),
            pkce: inputs.pkce,
            encode_url: inputs.encode_url,
            token_to_send: inputs
                .token_kind
                .read(cx)
                .selected_value()
                .copied()
                .unwrap_or_default(),
            header_name: text(&inputs.header_name),
            header_prefix: text(&inputs.header_prefix),
            id_token_header: text(&inputs.id_token_header),
            auto_fetch: inputs.auto_fetch,
            auto_refresh: inputs.auto_refresh,
            token_id: text(&inputs.token_id),
            ..Default::default()
        }
    }

    /// Show what the last Get token or Clear token did.
    pub fn set_token_status(
        &mut self,
        status: Option<SharedString>,
        fetching: bool,
        cx: &mut Context<Self>,
    ) {
        self.oauth2.token_status = status;
        self.oauth2.fetching_token = fetching;
        cx.notify();
    }

    fn render_section(&self, title: &'static str, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .pt_2()
            .text_xs()
            .text_color(cx.theme().muted_foreground)
            .child(title)
    }

    fn render_labeled(
        &self,
        label: &'static str,
        child: impl IntoElement,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .gap_3()
            .items_center()
            .child(
                div()
                    .w(px(140.))
                    .flex_shrink_0()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(label),
            )
            .child(div().flex_1().child(child))
    }

    fn render_switch(
        &self,
        id: &'static str,
        label: &'static str,
        checked: bool,
        set: fn(&mut OAuth2Inputs, bool),
        cx: &mut Context<Self>,
    ) -> Switch {
        Switch::new(id)
            .small()
            .label(label)
            .checked(checked)
            .on_click(cx.listener(move |this, checked: &bool, _window, cx| {
                set(&mut this.oauth2, *checked);
                cx.emit(AuthEditorEvent::AuthChanged);
                cx.notify();
            }))
    }

    fn render_labeled_switch(
        &self,
        id: &'static str,
        label: &'static str,
        checked: bool,
        set: fn(&mut OAuth2Inputs, bool),
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let switch = self.render_switch(id, label, checked, set, cx);
        self.render_labeled("", switch, cx)
    }

    fn render_oauth2(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let inputs = &self.oauth2;
        let grant_type = inputs
            .grant_type
            .read(cx)
            .selected_value()
            .cloned()
            .unwrap_or_default();
        let token_kind = inputs
            .token_kind
            .read(cx)
            .selected_value()
            .copied()
            .unwrap_or_default();
        let authorization_code = grant_type == OAuth2GrantType::AuthorizationCode;
        let fetching = inputs.fetching_token;

        v_flex()
            .gap_3()
            .p_4()
            .child(
                self.render_labeled(
                    "Grant Type",
                    div()
                        .w_full()
                        .max_w(px(240.))
                        .child(Select::new(&inputs.grant_type)),
                    cx,
                ),
            )
            .when(authorization_code, |this| {
                this.child(self.render_labeled_input(
                    "Authorization URL",
                    &inputs.authorize_url,
                    cx,
                ))
            })
            .child(self.render_labeled_input("Token URL", &self.token_url_input, cx))
            .child(self.render_labeled_input("Client ID", &self.client_id_input, cx))
            .child(self.render_labeled_input("Client Secret", &self.client_secret_input, cx))
            .child(self.render_labeled_input("Scope", &self.scope_input, cx))
            .when(authorization_code, |this| {
                this.child(self.render_labeled_input("Redirect URL", &inputs.redirect_url, cx))
                    .child(self.render_labeled_input("State", &inputs.state, cx))
                    .child(self.render_labeled_switch(
                        "oauth2-pkce",
                        "Use PKCE (S256)",
                        inputs.pkce,
                        |o, v| o.pkce = v,
                        cx,
                    ))
                    .child(self.render_labeled_switch(
                        "oauth2-encode-url",
                        "Encode the authorization URL",
                        inputs.encode_url,
                        |o, v| o.encode_url = v,
                        cx,
                    ))
            })
            .child(self.render_section("Token Request", cx))
            .child(
                self.render_labeled(
                    "Client Credentials",
                    div()
                        .w_full()
                        .max_w(px(240.))
                        .child(Select::new(&inputs.client_auth)),
                    cx,
                ),
            )
            .child(self.render_labeled_input("Refresh URL", &inputs.refresh_url, cx))
            .child(
                v_flex()
                    .gap_1()
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Extra Fields"),
                    )
                    .child(inputs.extra_params.clone()),
            )
            .child(self.render_section("Sending the Token", cx))
            .child(
                self.render_labeled(
                    "Send",
                    div()
                        .w_full()
                        .max_w(px(240.))
                        .child(Select::new(&inputs.token_kind)),
                    cx,
                ),
            )
            .child(self.render_labeled_input("Header", &inputs.header_name, cx))
            .child(self.render_labeled_input("Prefix", &inputs.header_prefix, cx))
            .when(token_kind == OAuth2TokenKind::Both, |this| {
                this.child(self.render_labeled_input(
                    "ID Token Header",
                    &inputs.id_token_header,
                    cx,
                ))
            })
            .child(self.render_section("Token", cx))
            .child(self.render_labeled_input("Token ID", &inputs.token_id, cx))
            .child(self.render_labeled_switch(
                "oauth2-auto-fetch",
                "Fetch a token when sending",
                inputs.auto_fetch,
                |o, v| o.auto_fetch = v,
                cx,
            ))
            .child(self.render_labeled_switch(
                "oauth2-auto-refresh",
                "Refresh expired tokens",
                inputs.auto_refresh,
                |o, v| o.auto_refresh = v,
                cx,
            ))
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        Button::new("oauth2-get-token")
                            .label("Get Token")
                            .small()
                            .primary()
                            .loading(fetching)
                            .disabled(fetching)
                            .on_click(cx.listener(|_this, _, _window, cx| {
                                cx.emit(AuthEditorEvent::GetToken);
                            })),
                    )
                    .child(
                        Button::new("oauth2-clear-token")
                            .label("Clear Token")
                            .small()
                            .outline()
                            .disabled(fetching)
                            .on_click(cx.listener(|_this, _, _window, cx| {
                                cx.emit(AuthEditorEvent::ClearToken);
                            })),
                    )
                    .when_some(inputs.token_status.clone(), |this, status| {
                        this.child(
                            div()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .child(status),
                        )
                    }),
            )
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

    fn render_unsupported(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let kind = match &self.unsupported_auth {
            Some(AuthType::Unsupported { kind, .. }) => kind.clone(),
            _ => "unknown".to_string(),
        };
        v_flex().gap_3().p_4().child(
            div()
                .text_sm()
                .text_color(cx.theme().muted_foreground)
                .child(format!(
                    "This request uses '{kind}' authentication, which broquest cannot edit. It is preserved unchanged when you save."
                )),
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
            .track_focus(&self.focus_handle)
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
                    AuthTypeOption::Unsupported => self.render_unsupported(cx).into_any_element(),
                }),
            ))
    }
}

impl Focusable for AuthEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
