use gpui::{App, Context, Entity, EventEmitter, Focusable, Window, div, prelude::*, px};
use gpui_component::{
    ActiveTheme, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

use crate::icon::IconName;
use crate::request_editor::KeyValuePair;

#[derive(Debug, Clone, PartialEq)]
pub enum HeaderEditorEvent {
    ParamChanged,
}

#[derive(Debug, Clone)]
pub struct HeaderRow {
    pub id: usize,
    pub key_input: Entity<InputState>,
    pub value_input: Entity<InputState>,
    pub enabled: bool,
}

pub struct HeaderEditor {
    rows: Vec<HeaderRow>,
    next_id: usize,
    _subscriptions: Vec<gpui::Subscription>,
}

impl EventEmitter<HeaderEditorEvent> for HeaderEditor {}

impl HeaderEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut editor = Self {
            rows: Vec::new(),
            next_id: 0,
            _subscriptions: Vec::new(),
        };
        // Always start with one empty row
        editor.add_header_row(String::new(), String::new(), true, window, cx);
        editor
    }

    pub fn set_headers(
        &mut self,
        headers: &[KeyValuePair],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Clear existing rows
        self.rows.clear();

        // Create new rows for each header
        for header in headers {
            self.add_header_row(
                header.key.clone(),
                header.value.clone(),
                header.enabled,
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
            self.add_header_row(String::new(), String::new(), true, window, cx);
        }
    }

    pub fn get_headers(&self, cx: &App) -> Vec<KeyValuePair> {
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

    fn add_header_row(
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
                .placeholder("Header name")
                .default_value(&key)
        });

        let value_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Header value")
                .default_value(&value)
        });

        // Set up subscriptions for key and value input change events
        let key_subscription = cx.subscribe_in(&key_input, window, {
            move |_this: &mut Self, input_state, event: &InputEvent, window, cx| {
                if let InputEvent::Change = event
                    && input_state.read(cx).focus_handle(cx).is_focused(window)
                {
                    cx.emit(HeaderEditorEvent::ParamChanged);
                }
            }
        });

        let value_subscription = cx.subscribe_in(&value_input, window, {
            move |_this: &mut Self, input_state, event: &InputEvent, window, cx| {
                if let InputEvent::Change = event
                    && input_state.read(cx).focus_handle(cx).is_focused(window)
                {
                    cx.emit(HeaderEditorEvent::ParamChanged);
                }
            }
        });

        self.rows.push(HeaderRow {
            id,
            key_input,
            value_input,
            enabled,
        });

        self._subscriptions.push(key_subscription);
        self._subscriptions.push(value_subscription);

        cx.notify();
    }

    fn add_new_header(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Check if the last row is empty; if not, add a new empty row
        if self
            .rows
            .last()
            .is_none_or(|row| !row.key_input.read(cx).value().is_empty())
        {
            self.add_header_row(String::new(), String::new(), true, window, cx);
        }
        cx.emit(HeaderEditorEvent::ParamChanged);
    }

    fn remove_header(&mut self, id: usize, cx: &mut Context<Self>) {
        self.rows.retain(|row| row.id != id);
        cx.emit(HeaderEditorEvent::ParamChanged);
        cx.notify();
    }

    fn toggle_header(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(row) = self.rows.iter_mut().find(|row| row.id == id) {
            row.enabled = !row.enabled;
            cx.emit(HeaderEditorEvent::ParamChanged);
            cx.notify();
        }
    }

    fn clear_all_headers(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.rows.clear();
        self.add_header_row(String::new(), String::new(), true, window, cx);
        cx.emit(HeaderEditorEvent::ParamChanged);
        cx.notify();
    }

    fn render_header_row(&self, row: &HeaderRow, cx: &mut Context<Self>) -> impl IntoElement {
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
                            this.toggle_header(id, cx);
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
                            this.remove_header(id, cx);
                        }
                    })),
            )
    }
}

impl Render for HeaderEditor {
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
                        Button::new("add-header")
                            .small()
                            .outline()
                            .icon(IconName::Plus)
                            .label("Add")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_new_header(window, cx);
                            })),
                    )
                    .child(
                        Button::new("clear-all-headers")
                            .small()
                            .outline()
                            .icon(IconName::Trash)
                            .label("Clear All")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.clear_all_headers(window, cx);
                            })),
                    ),
            )
            .child(
                div().flex_1().child(
                    v_flex().children(
                        self.rows
                            .iter()
                            .map(|row| div().child(self.render_header_row(row, cx))),
                    ),
                ),
            )
    }
}
