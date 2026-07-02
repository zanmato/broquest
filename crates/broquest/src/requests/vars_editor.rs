//! Editable request-level variables (Bruno `runtime.variables`).
//!
//! A key/value table like the other request editors (query/headers/path), but
//! purpose-built for request variables. It renders at natural height so it can
//! sit above the read-only inherited-variable inspector in a single scroll
//! region on the request editor's "Vars" tab.

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

#[derive(Debug, Clone)]
pub enum RequestVarEvent {
    Changed,
}

#[derive(Debug, Clone)]
struct VarRow {
    id: usize,
    key_input: Entity<InputState>,
    value_input: Entity<InputState>,
    enabled: bool,
}

pub struct RequestVarsEditor {
    rows: Vec<VarRow>,
    next_id: usize,
    _subscriptions: Vec<gpui::Subscription>,
}

impl RequestVarsEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut editor = Self {
            rows: Vec::new(),
            next_id: 0,
            _subscriptions: Vec::new(),
        };
        // Always keep a trailing empty row to type into.
        editor.add_row(String::new(), String::new(), true, window, cx);
        editor
    }

    pub fn set_vars(&mut self, vars: &[KeyValuePair], window: &mut Window, cx: &mut Context<Self>) {
        self.rows.clear();
        for var in vars {
            self.add_row(var.key.clone(), var.value.clone(), var.enabled, window, cx);
        }
        // Ensure a trailing empty row.
        if self
            .rows
            .last()
            .is_none_or(|row| !row.key_input.read(cx).value().is_empty())
        {
            self.add_row(String::new(), String::new(), true, window, cx);
        }
    }

    pub fn get_vars(&self, cx: &App) -> Vec<KeyValuePair> {
        self.rows
            .iter()
            .filter_map(|row| {
                let key = row.key_input.read(cx).value().to_string();
                if key.trim().is_empty() {
                    return None;
                }
                Some(KeyValuePair {
                    key,
                    value: row.value_input.read(cx).value().to_string(),
                    enabled: row.enabled,
                })
            })
            .collect()
    }

    /// Number of defined (non-empty) variables — used for the tab badge.
    pub fn count(&self, cx: &App) -> usize {
        self.rows
            .iter()
            .filter(|row| !row.key_input.read(cx).value().trim().is_empty())
            .count()
    }

    fn add_row(
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
                .placeholder("Variable name")
                .default_value(&key)
        });
        let value_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Variable value")
                .default_value(&value)
        });

        let subscribe = |input: &Entity<InputState>, cx: &mut Context<Self>| {
            cx.subscribe_in(input, window, {
                move |_this: &mut Self, input_state, event: &InputEvent, window, cx| {
                    if let InputEvent::Change = event
                        && input_state.read(cx).focus_handle(cx).is_focused(window)
                    {
                        cx.emit(RequestVarEvent::Changed);
                    }
                }
            })
        };
        self._subscriptions.push(subscribe(&key_input, cx));
        self._subscriptions.push(subscribe(&value_input, cx));

        self.rows.push(VarRow {
            id,
            key_input,
            value_input,
            enabled,
        });
        cx.notify();
    }

    fn add_empty_row(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self
            .rows
            .last()
            .is_none_or(|row| !row.key_input.read(cx).value().is_empty())
        {
            self.add_row(String::new(), String::new(), true, window, cx);
        }
    }

    fn remove_row(&mut self, id: usize, cx: &mut Context<Self>) {
        self.rows.retain(|row| row.id != id);
        cx.emit(RequestVarEvent::Changed);
        cx.notify();
    }

    fn toggle_row(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(row) = self.rows.iter_mut().find(|row| row.id == id) {
            row.enabled = !row.enabled;
            cx.emit(RequestVarEvent::Changed);
            cx.notify();
        }
    }

    fn clear_all(&mut self, cx: &mut Context<Self>) {
        self.rows.clear();
        cx.emit(RequestVarEvent::Changed);
        cx.notify();
    }

    fn render_row(&self, row: &VarRow, cx: &mut Context<Self>) -> impl IntoElement {
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
                            .appearance(false)
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
                            .appearance(false)
                            .text_sm()
                            .font_family(cx.theme().mono_font_family.clone()),
                    ),
            )
            .child(
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
                        move |this, _, _, cx| this.toggle_row(id, cx)
                    })),
            )
            .child(
                Button::new(("delete", row.id))
                    .small()
                    .ghost()
                    .icon(IconName::Trash)
                    .on_click(cx.listener({
                        let id = row.id;
                        move |this, _, _, cx| this.remove_row(id, cx)
                    })),
            )
    }
}

impl Render for RequestVarsEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let header = h_flex()
            .gap_3()
            .items_center()
            .p_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(div().flex_1())
            .child(
                Button::new("add-var")
                    .small()
                    .outline()
                    .icon(IconName::Plus)
                    .label("Add")
                    .on_click(cx.listener(|this, _, window, cx| this.add_empty_row(window, cx))),
            )
            .child(
                Button::new("clear-vars")
                    .small()
                    .outline()
                    .icon(IconName::Trash)
                    .label("Clear All")
                    .on_click(cx.listener(|this, _, _, cx| this.clear_all(cx))),
            );

        // Natural height: the Vars tab provides the shared scroll region.
        v_flex().child(header).child(
            v_flex().children(
                self.rows
                    .iter()
                    .map(|row| div().child(self.render_row(row, cx))),
            ),
        )
    }
}

impl EventEmitter<RequestVarEvent> for RequestVarsEditor {}
