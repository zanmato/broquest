use gpui::{App, Context, Entity, Window, div, prelude::*};
use gpui_component::{
    ActiveTheme, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    tab::{Tab, TabBar},
    v_flex,
};
use std::collections::HashMap;

use crate::collection_types::EnvironmentVariable;
use crate::icon::IconName;

#[derive(Debug, Clone)]
pub struct VariableRow {
    pub id: usize,
    pub key_input: Entity<InputState>,
    pub value_input: Entity<InputState>,
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SecretRow {
    pub id: usize,
    pub key_input: Entity<InputState>,
    pub value_input: Entity<InputState>, // For display only - not persisted
    pub enabled: bool,
}

#[derive(Debug, Clone)]
pub struct EnvironmentData {
    #[allow(dead_code)]
    pub id: usize,
    pub name: String,
    pub variables: Vec<VariableRow>,
    pub secrets: Vec<SecretRow>,
}

pub struct EnvironmentEditor {
    environments: Vec<EnvironmentData>,
    active_environment_idx: usize,
    next_id: usize,
    collection_name: String,
}

impl EnvironmentEditor {
    pub fn new(_window: &mut Window, _cx: &mut Context<Self>, collection_name: &str) -> Self {
        Self {
            environments: Vec::new(),
            active_environment_idx: 0,
            next_id: 0,
            collection_name: collection_name.to_string(),
        }
    }

    /// Load environments from CollectionToml data
    pub fn load_environments(
        &mut self,
        environment_tomls: &[crate::collection_types::EnvironmentToml],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.environments.clear();

        for env_toml in environment_tomls {
            let mut variables = Vec::new();
            let mut secrets = Vec::new();

            // Load variables and secrets from the unified variables map
            for (key, env_var) in &env_toml.variables {
                if env_var.secret {
                    self.add_secret_row(key.clone(), true, window, cx, &mut secrets);
                } else {
                    // This is a regular variable
                    self.add_variable_row(
                        key.clone(),
                        env_var.value.clone(),
                        true,
                        window,
                        cx,
                        &mut variables,
                    );
                }
            }

            let environment = EnvironmentData {
                id: self.next_id,
                name: env_toml.name.clone(),
                variables,
                secrets,
            };

            self.environments.push(environment);
            self.next_id += 1;
        }

        if self.environments.is_empty() {
            // Create a default environment if none exist
            self.add_environment("Default".to_string(), window, cx);
        }

        cx.notify();
    }

    /// Add a new environment
    pub fn add_environment(&mut self, name: String, _window: &mut Window, cx: &mut Context<Self>) {
        let environment = EnvironmentData {
            id: self.next_id,
            name,
            variables: Vec::new(),
            secrets: Vec::new(),
        };

        self.environments.push(environment);
        self.next_id += 1;
        self.active_environment_idx = self.environments.len() - 1;
        cx.notify();
    }

    /// Remove environment by index
    pub fn remove_environment(
        &mut self,
        index: usize,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if index < self.environments.len() {
            self.environments.remove(index);
            // Adjust active environment index if needed
            if self.active_environment_idx >= self.environments.len()
                && !self.environments.is_empty()
            {
                self.active_environment_idx = self.environments.len() - 1;
            } else if self.environments.is_empty() {
                self.active_environment_idx = 0;
            }
            cx.notify();
        }
    }

    /// Switch to a different environment
    pub fn set_active_environment(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.environments.len() {
            self.active_environment_idx = index;
            cx.notify();
        }
    }

    /// Get the currently active environment
    pub fn active_environment(&self) -> Option<&EnvironmentData> {
        self.environments.get(self.active_environment_idx)
    }

    /// Get environments for saving to TOML
    pub fn get_environments_for_save(
        &self,
        cx: &App,
    ) -> Vec<crate::collection_types::EnvironmentToml> {
        let mut result = Vec::new();

        for env_data in &self.environments {
            let mut variables = HashMap::new();

            // Collect variables and secrets from the unified variables map
            for var in &env_data.variables {
                if var.enabled {
                    let key = var.key_input.read(cx).value().to_string();
                    let value = var.value_input.read(cx).value().to_string();
                    if !key.is_empty() {
                        variables.insert(
                            key,
                            EnvironmentVariable {
                                value,
                                secret: false, // Regular variables from the editor are not secrets
                                temporary: false, // Editor variables are not temporary
                            },
                        );
                    }
                }
            }

            // Collect secret keys and values from the secrets editor
            for secret in &env_data.secrets {
                if secret.enabled {
                    let key = secret.key_input.read(cx).value().to_string();
                    let value = secret.value_input.read(cx).value().to_string();

                    if !key.is_empty() {
                        variables.insert(
                            key,
                            EnvironmentVariable {
                                value,
                                secret: true,     // These are secrets
                                temporary: false, // Editor secrets are not temporary
                            },
                        );
                    }
                }
            }

            let env_toml = crate::collection_types::EnvironmentToml {
                name: env_data.name.clone(),
                variables,
            };

            result.push(env_toml);
        }

        result
    }

    /// Save secrets to secure storage
    pub fn save_secrets(&self, cx: &App) -> Result<(), Box<dyn std::error::Error>> {
        for env_data in &self.environments {
            for secret in &env_data.secrets {
                if secret.enabled {
                    let key = secret.key_input.read(cx).value().to_string();
                    let value = secret.value_input.read(cx).value().to_string();

                    if !key.is_empty() && !value.is_empty() {
                        EnvironmentVariable::write_credential(
                            &self.collection_name,
                            &env_data.name,
                            &key,
                            &value,
                            cx,
                        )?;
                    }
                }
            }
        }
        Ok(())
    }

    fn add_variable_row(
        &mut self,
        key: String,
        value: String,
        enabled: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
        variables: &mut Vec<VariableRow>,
    ) {
        let id = self.next_id;
        self.next_id += 1;

        let key_input = cx.new(|cx| InputState::new(window, cx).default_value(&key));

        let value_input = cx.new(|cx| InputState::new(window, cx).default_value(&value));

        variables.push(VariableRow {
            id,
            key_input,
            value_input,
            enabled,
        });
    }

    fn add_secret_row(
        &mut self,
        key: String,
        enabled: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
        secrets: &mut Vec<SecretRow>,
    ) {
        let id = self.next_id;
        self.next_id += 1;

        let key_input = cx.new(|cx| InputState::new(window, cx).default_value(&key));

        let value_input = cx.new(|cx| InputState::new(window, cx).default_value(""));

        secrets.push(SecretRow {
            id,
            key_input,
            value_input,
            enabled,
        });
    }

    fn add_variable(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let id = self.next_id;
        self.next_id += 1;

        let key_input = cx.new(|cx| InputState::new(window, cx));
        let value_input = cx.new(|cx| InputState::new(window, cx));

        let variable = VariableRow {
            id,
            key_input,
            value_input,
            enabled: true,
        };

        if let Some(env) = self.environments.get_mut(self.active_environment_idx) {
            env.variables.push(variable);
            cx.notify();
        }
    }

    fn add_secret(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let id = self.next_id;
        self.next_id += 1;

        let key_input = cx.new(|cx| InputState::new(window, cx));
        let value_input = cx.new(|cx| InputState::new(window, cx));

        let secret = SecretRow {
            id,
            key_input,
            value_input,
            enabled: true,
        };

        if let Some(env) = self.environments.get_mut(self.active_environment_idx) {
            env.secrets.push(secret);
            cx.notify();
        }
    }

    fn remove_variable(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(env) = self.environments.get_mut(self.active_environment_idx) {
            env.variables.retain(|var| var.id != id);
            cx.notify();
        }
    }

    fn remove_secret(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(env) = self.environments.get_mut(self.active_environment_idx) {
            env.secrets.retain(|secret| secret.id != id);
            cx.notify();
        }
    }

    fn toggle_variable(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(env) = self.environments.get_mut(self.active_environment_idx)
            && let Some(var) = env.variables.iter_mut().find(|var| var.id == id)
        {
            var.enabled = !var.enabled;
            cx.notify();
        }
    }

    fn toggle_secret(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(env) = self.environments.get_mut(self.active_environment_idx)
            && let Some(secret) = env.secrets.iter_mut().find(|secret| secret.id == id)
        {
            secret.enabled = !secret.enabled;
            cx.notify();
        }
    }

    fn render_variable_row(&self, var: &VariableRow, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .pl_1()
            .pr_3()
            .items_center()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .flex_1()
                    .border_r_1()
                    .border_color(cx.theme().border)
                    .pr_2()
                    .py_2()
                    .child(
                        Input::new(&var.key_input)
                            .small()
                            .bordered(false)
                            .text_sm()
                            .font_family(cx.theme().mono_font_family.clone()),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .border_r_1()
                    .border_color(cx.theme().border)
                    .pr_2()
                    .py_2()
                    .child(
                        Input::new(&var.value_input)
                            .small()
                            .bordered(false)
                            .text_sm()
                            .font_family(cx.theme().mono_font_family.clone()),
                    ),
            )
            .child(
                Button::new(("var_enabled", var.id))
                    .small()
                    .ghost()
                    .text_color(if var.enabled {
                        cx.theme().green
                    } else {
                        cx.theme().red
                    })
                    .label(if var.enabled { "✓" } else { "○" })
                    .on_click(cx.listener({
                        let id = var.id;
                        move |this, _, _, cx| {
                            this.toggle_variable(id, cx);
                        }
                    })),
            )
            .child(
                Button::new(("delete_var", var.id))
                    .small()
                    .ghost()
                    .icon(IconName::Trash)
                    .on_click(cx.listener({
                        let id = var.id;
                        move |this, _, _, cx| {
                            this.remove_variable(id, cx);
                        }
                    })),
            )
    }

    fn render_secret_row(&self, secret: &SecretRow, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_2()
            .pl_1()
            .pr_3()
            .items_center()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                div()
                    .flex_1()
                    .border_r_1()
                    .border_color(cx.theme().border)
                    .pr_2()
                    .py_2()
                    .child(
                        Input::new(&secret.key_input)
                            .small()
                            .bordered(false)
                            .text_sm()
                            .font_family(cx.theme().mono_font_family.clone()),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .border_r_1()
                    .border_color(cx.theme().border)
                    .pr_2()
                    .py_2()
                    .child(
                        Input::new(&secret.value_input)
                            .small()
                            .bordered(false)
                            .text_sm()
                            .font_family(cx.theme().mono_font_family.clone()),
                    ),
            )
            .child(
                Button::new(("secret_enabled", secret.id))
                    .small()
                    .ghost()
                    .text_color(if secret.enabled {
                        cx.theme().green
                    } else {
                        cx.theme().red
                    })
                    .label(if secret.enabled { "✓" } else { "○" })
                    .on_click(cx.listener({
                        let id = secret.id;
                        move |this, _, _, cx| {
                            this.toggle_secret(id, cx);
                        }
                    })),
            )
            .child(
                Button::new(("delete_secret", secret.id))
                    .small()
                    .ghost()
                    .icon(IconName::Trash)
                    .on_click(cx.listener({
                        let id = secret.id;
                        move |this, _, _, cx| {
                            this.remove_secret(id, cx);
                        }
                    })),
            )
    }

    fn render_environment_selector(&self, cx: &mut Context<Self>) -> impl IntoElement {
        TabBar::new("environment-tabs")
            .selected_index(self.active_environment_idx)
            .on_click(cx.listener(|this, ix: &usize, _window, cx| {
                this.set_active_environment(*ix, cx);
            }))
            .children(self.environments.iter().enumerate().map(|(env_idx, env)| {
                Tab::new().label(&env.name).suffix(
                    Button::new(("delete_env", env_idx))
                        .small()
                        .ghost()
                        .icon(IconName::Trash)
                        .on_click(cx.listener(move |this, _, window, cx| {
                            this.remove_environment(env_idx, window, cx);
                        })),
                )
            }))
            .suffix(
                Button::new("add_env")
                    .small()
                    .ghost()
                    .icon(IconName::Plus)
                    .on_click(cx.listener(|this, _, window, cx| {
                        let env_name = format!("Environment {}", this.environments.len() + 1);
                        this.add_environment(env_name, window, cx);
                    })),
            )
    }
}

impl Render for EnvironmentEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .child(self.render_environment_selector(cx))
            .when_some(self.active_environment(), |this, env| {
                this.child(
                    div().flex_1().child(
                        v_flex()
                            .gap_6()
                            // Variables section
                            .child(
                                v_flex().children([
                                    // Variables header
                                    h_flex()
                                        .gap_2()
                                        .items_center()
                                        .p_3()
                                        .border_b_1()
                                        .border_color(cx.theme().border)
                                        .child(
                                            div()
                                                .flex_1()
                                                .text_sm()
                                                .font_medium()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("Variables"),
                                        )
                                        .child(
                                            Button::new("add_variable")
                                                .small()
                                                .outline()
                                                .icon(IconName::Plus)
                                                .label("Add Variable")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.add_variable(window, cx);
                                                })),
                                        ),
                                    // Variables table
                                    div().flex_1().child(v_flex().children(
                                        env.variables.iter().map(|var| {
                                            div().child(self.render_variable_row(var, cx))
                                        }),
                                    )),
                                ]),
                            )
                            // Secrets section
                            .child(
                                v_flex().children([
                                    // Secrets header
                                    h_flex()
                                        .gap_2()
                                        .items_center()
                                        .p_3()
                                        .border_b_1()
                                        .border_color(cx.theme().border)
                                        .child(
                                            div()
                                                .flex_1()
                                                .text_sm()
                                                .font_medium()
                                                .text_color(cx.theme().muted_foreground)
                                                .child("Secrets"),
                                        )
                                        .child(
                                            Button::new("add_secret")
                                                .small()
                                                .outline()
                                                .icon(IconName::Plus)
                                                .label("Add Secret")
                                                .on_click(cx.listener(|this, _, window, cx| {
                                                    this.add_secret(window, cx);
                                                })),
                                        ),
                                    // Secrets table
                                    div().flex_1().child(v_flex().children(
                                        env.secrets.iter().map(|secret| {
                                            div().child(self.render_secret_row(secret, cx))
                                        }),
                                    )),
                                ]),
                            ),
                    ),
                )
            })
    }
}
