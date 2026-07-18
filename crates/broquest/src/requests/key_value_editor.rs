//! A generic key/value table editor.
//!
//! Request headers, query params, path params, form fields, and request
//! variables are all edited as a list of `(key, value, enabled)` rows with the
//! same add/remove/toggle/clear behavior. This component captures that shared
//! shape; per-use differences (placeholders, the form file-picker column, the
//! embedded natural-height layout) are expressed through [`KeyValueConfig`].

use gpui::{
    App, Context, Entity, EventEmitter, Focusable, SharedString, Window, div, prelude::*, px,
};
use gpui_component::{
    ActiveTheme, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    v_flex,
};

use crate::domain::KeyValuePair;
use crate::result_ext::ResultExt;
use crate::ui::icon::IconName;

/// Emitted whenever the set of rows or their contents changes.
#[derive(Debug, Clone, PartialEq)]
pub enum KeyValueEvent {
    Changed,
}

/// Per-use configuration for a [`KeyValueEditor`].
#[derive(Clone)]
pub struct KeyValueConfig {
    /// Stable prefix for the header buttons' element ids (must be unique per
    /// editor instance rendered in the same window).
    pub id_prefix: SharedString,
    /// Placeholder for the key column input.
    pub key_placeholder: SharedString,
    /// Placeholder for the value column input.
    pub value_placeholder: SharedString,
    /// Show a per-row file-picker button that writes `@/path` into the value
    /// (used by multipart form fields).
    pub show_file_picker: bool,
    /// Render at natural height without an own scroll region, for embedding in a
    /// parent scroll area (used by the request Vars tab).
    pub embedded: bool,
}

impl KeyValueConfig {
    /// A standard key/value editor with the given id prefix and placeholders.
    pub fn new(
        id_prefix: impl Into<SharedString>,
        key_placeholder: impl Into<SharedString>,
        value_placeholder: impl Into<SharedString>,
    ) -> Self {
        Self {
            id_prefix: id_prefix.into(),
            key_placeholder: key_placeholder.into(),
            value_placeholder: value_placeholder.into(),
            show_file_picker: false,
            embedded: false,
        }
    }

    /// Enable the per-row file picker (multipart form fields).
    pub fn with_file_picker(mut self) -> Self {
        self.show_file_picker = true;
        self
    }

    /// Render at natural height for embedding in a parent scroll region.
    pub fn embedded(mut self) -> Self {
        self.embedded = true;
        self
    }
}

#[derive(Clone)]
struct KeyValueRow {
    id: usize,
    key_input: Entity<InputState>,
    value_input: Entity<InputState>,
    enabled: bool,
}

pub struct KeyValueEditor {
    rows: Vec<KeyValueRow>,
    next_id: usize,
    config: KeyValueConfig,
    _subscriptions: Vec<gpui::Subscription>,
}

impl EventEmitter<KeyValueEvent> for KeyValueEditor {}

impl KeyValueEditor {
    pub fn new(config: KeyValueConfig, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut editor = Self {
            rows: Vec::new(),
            next_id: 0,
            config,
            _subscriptions: Vec::new(),
        };
        // Always start with one empty row to type into.
        editor.add_row(String::new(), String::new(), true, window, cx);
        editor
    }

    /// Replace the rows with `pairs`, keeping a trailing empty row.
    pub fn set_pairs(
        &mut self,
        pairs: &[KeyValuePair],
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rows.clear();
        for pair in pairs {
            self.add_row(
                pair.key.clone(),
                pair.value.clone(),
                pair.enabled,
                window,
                cx,
            );
        }
        if self
            .rows
            .last()
            .is_none_or(|row| !row.key_input.read(cx).value().is_empty())
        {
            self.add_row(String::new(), String::new(), true, window, cx);
        }
    }

    /// The non-empty rows as key/value pairs.
    pub fn get_pairs(&self, cx: &App) -> Vec<KeyValuePair> {
        self.rows
            .iter()
            .filter_map(|row| {
                let key = row.key_input.read(cx).value().to_string();
                if key.trim().is_empty() {
                    None
                } else {
                    Some(KeyValuePair {
                        key,
                        value: row.value_input.read(cx).value().to_string(),
                        enabled: row.enabled,
                    })
                }
            })
            .collect()
    }

    /// Number of defined (non-empty-key) rows, for the tab badge.
    pub fn count(&self, cx: &App) -> usize {
        self.rows
            .iter()
            .filter(|row| !row.key_input.read(cx).value().trim().is_empty())
            .count()
    }

    /// Substitute `:name` placeholders in `url` with each enabled row's value.
    /// e.g. `hello/:productid` with `productid=8900` becomes `hello/8900`.
    pub fn replace_path_parameters(&self, url: &str, cx: &App) -> String {
        let mut result_url = url.to_string();
        for row in &self.rows {
            if row.enabled {
                let key = row.key_input.read(cx).value().to_string();
                let value = row.value_input.read(cx).value().to_string();
                if !key.is_empty() {
                    result_url = result_url.replace(&format!(":{}", key), &value);
                }
            }
        }
        result_url
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
                .placeholder(self.config.key_placeholder.clone())
                .default_value(&key)
        });
        let value_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder(self.config.value_placeholder.clone())
                .default_value(&value)
        });

        // Emit only when the change originates from user typing (the input is
        // focused), to avoid feedback loops when rows are set programmatically.
        let subscribe = |input: &Entity<InputState>, cx: &mut Context<Self>| {
            cx.subscribe_in(input, window, {
                move |_this: &mut Self, input_state, event: &InputEvent, window, cx| {
                    if let InputEvent::Change = event
                        && input_state.read(cx).focus_handle(cx).is_focused(window)
                    {
                        cx.emit(KeyValueEvent::Changed);
                    }
                }
            })
        };
        self._subscriptions.push(subscribe(&key_input, cx));
        self._subscriptions.push(subscribe(&value_input, cx));

        self.rows.push(KeyValueRow {
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
        cx.emit(KeyValueEvent::Changed);
    }

    fn remove_row(&mut self, id: usize, cx: &mut Context<Self>) {
        self.rows.retain(|row| row.id != id);
        cx.emit(KeyValueEvent::Changed);
        cx.notify();
    }

    fn toggle_row(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(row) = self.rows.iter_mut().find(|row| row.id == id) {
            row.enabled = !row.enabled;
            cx.emit(KeyValueEvent::Changed);
            cx.notify();
        }
    }

    fn clear_all(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.rows.clear();
        self.add_row(String::new(), String::new(), true, window, cx);
        cx.emit(KeyValueEvent::Changed);
        cx.notify();
    }

    /// Open a file picker and write the chosen path (prefixed with `@`) into the
    /// row's value input.
    fn select_file_for_row(&mut self, id: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(row) = self.rows.iter().find(|row| row.id == id) else {
            return;
        };
        let value_input = row.value_input.clone();
        let editor = cx.entity().downgrade();

        let path_future = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: true,
            directories: false,
            multiple: false,
            prompt: Some("Select file for form field".into()),
        });

        cx.spawn_in(window, async move |_, window| {
            match path_future.await {
                Ok(Ok(Some(paths))) => {
                    if let Some(path) = paths.first()
                        && let Some(path_str) = path.to_str()
                    {
                        let file_path = format!("@{}", path_str);
                        window
                            .update(|window, cx| {
                                value_input.update(cx, |state, cx| {
                                    state.set_value(file_path, window, cx);
                                    cx.notify();
                                });
                                if let Some(editor) = editor.upgrade() {
                                    editor.update(cx, |_this, cx| {
                                        cx.emit(KeyValueEvent::Changed);
                                    });
                                }
                            })
                            .log_err()
                            .ok();
                    }
                }
                Ok(Ok(None)) => {}
                Ok(Err(e)) => tracing::error!("Failed to select file: {}", e),
                Err(e) => tracing::error!("Failed to open file dialog: {}", e),
            }
            Some(())
        })
        .detach();
    }

    fn render_row(&self, row: &KeyValueRow, cx: &mut Context<Self>) -> impl IntoElement {
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
            .when(self.config.show_file_picker, |this| {
                this.child(
                    Button::new(("file", row.id))
                        .small()
                        .ghost()
                        .icon(IconName::File)
                        .w(px(24.))
                        .on_click(cx.listener({
                            let id = row.id;
                            move |this, _, window, cx| {
                                this.select_file_for_row(id, window, cx);
                            }
                        })),
                )
            })
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

    fn render_header(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .gap_3()
            .items_center()
            .p_3()
            .border_b_1()
            .border_color(cx.theme().border)
            .child(div().flex_1())
            .child(
                Button::new(SharedString::from(format!("{}-add", self.config.id_prefix)))
                    .small()
                    .outline()
                    .icon(IconName::Plus)
                    .label("Add")
                    .on_click(cx.listener(|this, _, window, cx| this.add_empty_row(window, cx))),
            )
            .child(
                Button::new(SharedString::from(format!(
                    "{}-clear",
                    self.config.id_prefix
                )))
                .small()
                .outline()
                .icon(IconName::Trash)
                .label("Clear All")
                .on_click(cx.listener(|this, _, window, cx| this.clear_all(window, cx))),
            )
    }
}

impl Render for KeyValueEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self
            .rows
            .iter()
            .map(|row| div().child(self.render_row(row, cx)))
            .collect::<Vec<_>>();

        if self.config.embedded {
            // Natural height: a parent scroll region handles scrolling.
            v_flex()
                .child(self.render_header(cx))
                .child(v_flex().children(rows))
                .into_any_element()
        } else {
            v_flex()
                .h_full()
                .child(self.render_header(cx))
                .child(
                    div()
                        .size_full()
                        .flex_1()
                        .min_h_0()
                        .child(v_flex().overflow_y_scrollbar().children(rows)),
                )
                .into_any_element()
        }
    }
}
