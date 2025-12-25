use gpui::{App, Context, Entity, Window, div, prelude::*, px};
use gpui_component::{
    ActiveTheme, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputState},
    v_flex,
};

use crate::icon::IconName;
use crate::request_editor::KeyValuePair;

#[derive(Debug, Clone)]
pub struct FormRow {
    pub id: usize,
    pub key_input: Entity<InputState>,
    pub value_input: Entity<InputState>,
    pub enabled: bool,
}

pub struct FormEditor {
    rows: Vec<FormRow>,
    next_id: usize,
}

impl FormEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let mut editor = Self {
            rows: Vec::new(),
            next_id: 0,
        };
        // Always start with one empty row
        editor.add_form_row(String::new(), String::new(), true, window, cx);
        editor
    }

    pub fn get_form_data(&self, cx: &App) -> Vec<KeyValuePair> {
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

    fn add_form_row(
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
                .placeholder("Field name")
                .default_value(&key)
        });

        let value_input = cx.new(|cx| {
            InputState::new(window, cx)
                .placeholder("Field value or @/path/to/file")
                .default_value(&value)
        });

        self.rows.push(FormRow {
            id,
            key_input,
            value_input,
            enabled,
        });

        cx.notify();
    }

    fn add_new_form_field(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        // Check if the last row is empty; if not, add a new empty row
        if self
            .rows
            .last()
            .is_none_or(|row| !row.key_input.read(cx).value().is_empty())
        {
            self.add_form_row(String::new(), String::new(), true, window, cx);
        }
    }

    fn remove_form_field(&mut self, id: usize, cx: &mut Context<Self>) {
        self.rows.retain(|row| row.id != id);
        cx.notify();
    }

    fn toggle_form_field(&mut self, id: usize, cx: &mut Context<Self>) {
        if let Some(row) = self.rows.iter_mut().find(|row| row.id == id) {
            row.enabled = !row.enabled;
            cx.notify();
        }
    }

    fn clear_all_form_fields(&mut self, cx: &mut Context<Self>) {
        self.rows.clear();
        cx.notify();
    }

    fn select_file_for_row(&mut self, id: usize, window: &mut Window, cx: &mut Context<Self>) {
        // Find the row and clone the value input entity
        if let Some(row) = self.rows.iter().find(|row| row.id == id) {
            let value_input = row.value_input.clone();

            // Use GPUI's prompt_for_paths to select a file
            let path_future = cx.prompt_for_paths(gpui::PathPromptOptions {
                files: true,
                directories: false,
                multiple: false,
                prompt: Some("Select file for form field".into()),
            });

            // Spawn a task to handle the file selection
            cx.spawn_in(window, async move |_, window| {
                match path_future.await {
                    Ok(result) => match result {
                        Ok(Some(paths)) => {
                            if let Some(path) = paths.first()
                                && let Some(path_str) = path.to_str()
                            {
                                let file_path = format!("@{}", path_str);

                                // Update the value input with the selected file path
                                let _ = window.update(|window, cx| {
                                    value_input.update(cx, |state, cx| {
                                        state.set_value(file_path, window, cx);
                                        cx.notify();
                                    });
                                });
                            }
                        }
                        Ok(None) => {
                            // User cancelled file selection
                        }
                        Err(e) => {
                            tracing::error!("Failed to select file: {}", e);
                        }
                    },
                    Err(e) => {
                        tracing::error!("Failed to open file dialog: {}", e);
                    }
                }
                Some(())
            })
            .detach();
        }
    }

    fn render_form_row(&self, row: &FormRow, cx: &mut Context<Self>) -> impl IntoElement {
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
                // File selection button
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
                            this.toggle_form_field(id, cx);
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
                            this.remove_form_field(id, cx);
                        }
                    })),
            )
    }
}

impl Render for FormEditor {
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
                        Button::new("add-form-field")
                            .small()
                            .outline()
                            .icon(IconName::Plus)
                            .label("Add")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.add_new_form_field(window, cx);
                            })),
                    )
                    .child(
                        Button::new("clear-all-form-fields")
                            .small()
                            .outline()
                            .icon(IconName::Trash)
                            .label("Clear All")
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.clear_all_form_fields(cx);
                            })),
                    ),
            )
            .child(
                div().flex_1().child(
                    v_flex().children(
                        self.rows
                            .iter()
                            .map(|row| div().child(self.render_form_row(row, cx))),
                    ),
                ),
            )
    }
}
