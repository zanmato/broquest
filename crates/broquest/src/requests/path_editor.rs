use gpui::{App, Context, Entity, EventEmitter, Focusable, Window, div, prelude::*, px};
use gpui_component::{
    ActiveTheme, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

use crate::domain::KeyValuePair;
use crate::ui::icon::IconName;

#[derive(Debug, Clone, PartialEq)]
pub enum PathParamEvent {
    ParamChanged,
}

#[derive(Debug, Clone)]
pub struct PathParameterRow {
    pub id: usize,
    pub key_input: Entity<InputState>,
    pub value_input: Entity<InputState>,
    pub enabled: bool,
}

pub struct PathParamEditor {
    rows: Vec<PathParameterRow>,
    next_id: usize,
    _subscriptions: Vec<gpui::Subscription>,
}

impl EventEmitter<PathParamEvent> for PathParamEditor {}

impl PathParamEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut editor = Self {
            rows: Vec::new(),
            next_id: 0,
            _subscriptions: Vec::new(),
        };
        // Always start with one empty row
        editor.add_parameter_row(String::new(), String::new(), true, window, cx);
        editor
    }

    pub fn set_parameters(
        &mut self,
        parameters: &[KeyValuePair],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Clear existing rows
        self.rows.clear();

        // Create new rows for each parameter
        for param in parameters {
            self.add_parameter_row(
                param.key.clone(),
                param.value.clone(),
                param.enabled,
                window,
                cx,
            );
        }

        // Always ensure there's at least one empty row at the end
        if self.rows.is_empty()
            || !self
                .rows
                .last()
                .unwrap()
                .key_input
                .read(cx)
                .value()
                .is_empty()
        {
            self.add_parameter_row(String::new(), String::new(), true, window, cx);
        }
    }

    pub fn get_path_parameters(&self, cx: &App) -> Vec<KeyValuePair> {
        self.rows
            .iter()
            .filter_map(|row| {
                let key = row.key_input.read(cx).value().to_string();
                // Filter out empty keys (only whitespace or completely empty)
                if key.trim().is_empty() {
                    None
                } else {
                    let value = row.value_input.read(cx).value().to_string();
                    Some(KeyValuePair {
                        key,
                        value,
                        enabled: row.enabled,
                    })
                }
            })
            .collect()
    }

    /// Replace path parameters in URL with their values
    /// e.g., "hello/:productid" with productid=8900 becomes "hello/8900"
    pub fn replace_path_parameters(&self, url: &str, cx: &App) -> String {
        let mut result_url = url.to_string();

        for row in &self.rows {
            if row.enabled {
                let key = row.key_input.read(cx).value().to_string();
                let value = row.value_input.read(cx).value().to_string();

                if !key.is_empty() {
                    // Replace :param with value
                    let placeholder = format!(":{}", key);
                    result_url = result_url.replace(&placeholder, &value);
                }
            }
        }

        result_url
    }

    fn add_parameter_row(
        &mut self,
        key: String,
        value: String,
        enabled: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let id = self.next_id;
        self.next_id += 1;

        let key_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Parameter name")
                .default_value(&key)
        });

        let value_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Parameter value")
                .default_value(&value)
        });

        // Set up subscriptions for key and value input change events
        let key_subscription = cx.subscribe_in(&key_input, window, {
            move |_this: &mut Self, input_state, event: &InputEvent, window, cx| {
                if let InputEvent::Change = event
                    && input_state.read(cx).focus_handle(cx).is_focused(window)
                {
                    cx.emit(PathParamEvent::ParamChanged);
                }
            }
        });

        let value_subscription = cx.subscribe_in(&value_input, window, {
            move |_this: &mut Self, input_state, event: &InputEvent, window, cx| {
                if let InputEvent::Change = event
                    && input_state.read(cx).focus_handle(cx).is_focused(window)
                {
                    cx.emit(PathParamEvent::ParamChanged);
                }
            }
        });

        self.rows.push(PathParameterRow {
            id,
            key_input,
            value_input,
            enabled,
        });

        self._subscriptions.push(key_subscription);
        self._subscriptions.push(value_subscription);

        cx.notify();
    }

    fn add_new_parameter(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Check if the last row is empty; if not, add a new empty row
        if self
            .rows
            .last()
            .is_none_or(|row| !row.key_input.read(cx).value().is_empty())
        {
            self.add_parameter_row(String::new(), String::new(), true, window, cx);
        }
        cx.emit(PathParamEvent::ParamChanged);
    }

    fn remove_parameter(&mut self, id: usize, cx: &mut Context<Self>) {
        self.rows.retain(|row| row.id != id);
        cx.emit(PathParamEvent::ParamChanged);
        cx.notify();
    }

    fn toggle_parameter(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(row) = self.rows.iter_mut().find(|row| row.id == id) {
            row.enabled = !row.enabled;
            cx.emit(PathParamEvent::ParamChanged);
            cx.notify();
        }
    }

    fn clear_all_parameters(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.rows.clear();
        self.add_parameter_row(String::new(), String::new(), true, window, cx);
        cx.emit(PathParamEvent::ParamChanged);
        cx.notify();
    }

    fn render_parameter_row(
        &self,
        row: &PathParameterRow,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        h_flex()
            .gap_2()
            .pl_2()
            .pr_4()
            .items_center()
            .bg(cx.theme().table)
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
                        Input::new(&row.key_input)
                            .small()
                            .bordered(false)
                            .bg(cx.theme().table)
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
                        Input::new(&row.value_input)
                            .small()
                            .bordered(false)
                            .bg(cx.theme().table)
                            .text_sm()
                            .font_family(cx.theme().mono_font_family.clone()),
                    ),
            )
            .child(
                // Simple enabled toggle using button
                Button::new(("enabled", row.id))
                    .small()
                    .ghost()
                    .text_color(if row.enabled {
                        cx.theme().green
                    } else {
                        cx.theme().red
                    })
                    .w(px(24.))
                    .label(if row.enabled { "✓" } else { "○" })
                    .on_click(cx.listener({
                        let id = row.id;
                        move |this, _, _, cx| {
                            this.toggle_parameter(id, cx);
                        }
                    })),
            )
            .child(
                Button::new(("delete", row.id))
                    .small()
                    .ghost()
                    .icon(IconName::Trash)
                    .on_click(cx.listener({
                        let id = row.id;
                        move |this, _, _, cx| {
                            this.remove_parameter(id, cx);
                        }
                    })),
            )
    }
}

impl Render for PathParamEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .child(
                // Action buttons header
                h_flex()
                    .gap_3()
                    .items_center()
                    .p_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(div().flex_1())
                    .child(
                        Button::new("add-parameter")
                            .small()
                            .outline()
                            .icon(IconName::Plus)
                            .label("Add")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_new_parameter(window, cx);
                            })),
                    )
                    .child(
                        Button::new("clear-all-parameters")
                            .small()
                            .outline()
                            .icon(IconName::Trash)
                            .label("Clear All")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.clear_all_parameters(window, cx);
                            })),
                    ),
            )
            .child(
                div().flex_1().child(
                    v_flex().children(
                        self.rows
                            .iter()
                            .map(|row| div().child(self.render_parameter_row(row, cx))),
                    ),
                ),
            )
    }
}
