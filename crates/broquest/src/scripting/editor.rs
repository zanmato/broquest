use std::time::Duration;

use crate::app_settings::AppSettings;
use crate::result_ext::ResultExt;
use crate::ui::icon::IconName;
use gpui::{
    App, AppContext, Context, Entity, EventEmitter, Focusable, Subscription, Task, Window, div,
    prelude::*, px,
};
use gpui_component::{
    ActiveTheme, Sizable, StyledExt, WindowExt,
    button::Button,
    h_flex,
    highlighter::{Diagnostic, DiagnosticSeverity},
    input::{Editor, EditorState, InputEvent, Position},
    v_flex,
};

use super::completion::{ScriptCompletionProvider, ScriptContext};
use super::engine::ScriptExecutionService;
use super::recipes::RecipePicker;

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptEditorEvent {
    ScriptChanged,
}

#[derive(Debug)]
pub struct ScriptEditor {
    pre_request_input: Entity<EditorState>,
    post_response_input: Entity<EditorState>,
    _subscriptions: Vec<Subscription>,
    _lint_task: Task<()>,
}

impl EventEmitter<ScriptEditorEvent> for ScriptEditor {}

impl ScriptEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let editor_settings = AppSettings::global(cx).settings.editor.clone();

        let pre_request_input = cx.new(|cx| {
            let editor = EditorState::new("javascript", window, cx)
                .folding(editor_settings.folding)
                .show_whitespaces(editor_settings.show_whitespace);
            editor.base_state().update(cx, |state, cx| {
                state.set_soft_wrap(editor_settings.soft_wrap, window, cx);
                state.lsp.completion_provider =
                    Some(ScriptCompletionProvider::new(ScriptContext::PreRequest));
            });
            editor
        });

        let post_response_input = cx.new(|cx| {
            let editor = EditorState::new("javascript", window, cx)
                .folding(editor_settings.folding)
                .show_whitespaces(editor_settings.show_whitespace);
            editor.base_state().update(cx, |state, cx| {
                state.set_soft_wrap(editor_settings.soft_wrap, window, cx);
                state.lsp.completion_provider =
                    Some(ScriptCompletionProvider::new(ScriptContext::PostResponse));
            });
            editor
        });

        // Set up subscriptions for script input change events
        let pre_subscription = cx.subscribe_in(&pre_request_input, window, {
            move |this: &mut Self,
                  input_state: &Entity<EditorState>,
                  event: &InputEvent,
                  window,
                  cx| {
                if let InputEvent::Change = event
                    && input_state.read(cx).focus_handle(cx).is_focused(window)
                {
                    this.lint_script(input_state.clone(), ScriptContext::PreRequest, cx);
                    cx.emit(ScriptEditorEvent::ScriptChanged);
                }
            }
        });

        let post_subscription = cx.subscribe_in(&post_response_input, window, {
            move |this: &mut Self,
                  input_state: &Entity<EditorState>,
                  event: &InputEvent,
                  window,
                  cx| {
                if let InputEvent::Change = event
                    && input_state.read(cx).focus_handle(cx).is_focused(window)
                {
                    this.lint_script(input_state.clone(), ScriptContext::PostResponse, cx);
                    cx.emit(ScriptEditorEvent::ScriptChanged);
                }
            }
        });

        Self {
            pre_request_input,
            post_response_input,
            _subscriptions: vec![pre_subscription, post_subscription],
            _lint_task: Task::ready(()),
        }
    }

    fn lint_script(
        &mut self,
        input: Entity<EditorState>,
        context: ScriptContext,
        cx: &mut Context<Self>,
    ) {
        // Debounce: wait 500ms then run the parse
        self._lint_task = cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;

            let script = cx.read_entity(&input, |input, _cx| input.value().to_string());

            let result = cx
                .background_spawn(
                    async move { ScriptExecutionService::check_syntax(&script, context) },
                )
                .await;

            // Update diagnostics on main thread
            let _ = this
                .update(cx, |_this, cx| {
                    input.update(cx, |input, cx| {
                        input.base_state().update(cx, |input, _cx| {
                            if let Some(diagnostics) = input.diagnostics_mut() {
                                diagnostics.clear();
                                if let Err(err) = result {
                                    let severity = if err.is_warning {
                                        DiagnosticSeverity::Warning
                                    } else {
                                        DiagnosticSeverity::Error
                                    };
                                    diagnostics.push(
                                        Diagnostic::new(
                                            Position::new(err.line, err.column)
                                                ..Position::new(err.line, err.column + 1),
                                            err.message,
                                        )
                                        .with_severity(severity),
                                    );
                                }
                            }
                        });
                    });
                    cx.notify();
                })
                .log_err();
        });
    }

    pub fn apply_editor_settings(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let settings = AppSettings::global(cx).settings.editor.clone();
        for input in [&self.pre_request_input, &self.post_response_input] {
            input.update(cx, |state, cx| {
                state.base_state().update(cx, |state, cx| {
                    state.set_show_whitespaces(settings.show_whitespace, window, cx);
                    state.set_soft_wrap(settings.soft_wrap, window, cx);
                    state.set_folding(settings.folding, window, cx);
                });
            });
        }
    }

    pub fn set_scripts(
        &mut self,
        pre_request_script: Option<&str>,
        post_response_script: Option<&str>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Set pre-request script
        if let Some(script) = pre_request_script {
            let script = script.to_string();
            self.pre_request_input.update(cx, |input, cx| {
                input.set_value(&script, window, cx);
            });
        }

        // Set post-response script
        if let Some(script) = post_response_script {
            let script = script.to_string();
            self.post_response_input.update(cx, |input, cx| {
                input.set_value(&script, window, cx);
            });
        }
    }

    pub fn get_pre_request_script(&self, cx: &App) -> Option<String> {
        let script = self.pre_request_input.read(cx).value();
        if script.trim().is_empty() {
            None
        } else {
            Some(script.to_string())
        }
    }

    pub fn get_post_response_script(&self, cx: &App) -> Option<String> {
        let script = self.post_response_input.read(cx).value();
        if script.trim().is_empty() {
            None
        } else {
            Some(script.to_string())
        }
    }

    pub(crate) fn insert_recipe(
        &mut self,
        input: Entity<EditorState>,
        selection: std::ops::Range<usize>,
        code: &str,
        context: ScriptContext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let base = input.read(cx).base_state().clone();
        base.update(cx, |state, cx| {
            state.set_selected_range(selection, cx);
            state.replace(code, window, cx);
        });
        self.lint_script(input, context, cx);
        cx.emit(ScriptEditorEvent::ScriptChanged);
    }

    fn open_recipes(
        &self,
        input: Entity<EditorState>,
        context: ScriptContext,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let base = input.read(cx).base_state().clone();
        let selection = base.read(cx).selected_range();
        let script_editor = cx.entity().downgrade();
        let picker =
            cx.new(|cx| RecipePicker::new(context, input, selection, script_editor, window, cx));

        window.open_dialog(cx, move |dialog, _, _| {
            dialog.w(px(820.)).p_4().footer(div()).child(picker.clone())
        });
    }

    fn render_script_section(
        &self,
        title: &str,
        input: &Entity<EditorState>,
        context: ScriptContext,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let title_owned = title.to_string();
        let button_id = match title {
            "Pre-request Script" => "clear-pre-request-script",
            "Post-response Script" => "clear-post-response-script",
            _ => "clear-script",
        };

        v_flex()
            .flex_1()
            .child(
                h_flex()
                    .items_center()
                    .justify_between()
                    .p_3()
                    .border_b_1()
                    .border_color(cx.theme().border)
                    .child(
                        div()
                            .text_sm()
                            .font_semibold()
                            .text_color(cx.theme().foreground)
                            .child(title_owned.clone()),
                    )
                    .child(
                        h_flex()
                            .gap_2()
                            .child(
                                Button::new(gpui::SharedString::from(format!(
                                    "{button_id}-recipes"
                                )))
                                .small()
                                .outline()
                                .icon(IconName::Sparkles)
                                .label("Recipes")
                                .on_click(cx.listener({
                                    let input = input.clone();
                                    move |this, _event, window, cx| {
                                        this.open_recipes(input.clone(), context, window, cx);
                                    }
                                })),
                            )
                            .child(
                                Button::new(button_id)
                                    .small()
                                    .outline()
                                    .icon(IconName::Trash)
                                    .label("Clear")
                                    .on_click(cx.listener({
                                        let input = input.clone();
                                        move |_this, _event, window, cx| {
                                            input.update(cx, |input, cx| {
                                                input.set_value("", window, cx);
                                            });
                                            cx.emit(ScriptEditorEvent::ScriptChanged);
                                        }
                                    })),
                            ),
                    ),
            )
            .child(
                div().flex_1().child(
                    Editor::new(input)
                        .py_3()
                        .font_family(cx.theme().mono_font_family.clone())
                        .text_size(px(12.))
                        .h_full()
                        .bordered(false)
                        .rounded_none(),
                ),
            )
    }
}

impl Render for ScriptEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .child(self.render_script_section(
                "Pre-request Script",
                &self.pre_request_input,
                ScriptContext::PreRequest,
                cx,
            ))
            .child(div().h_px().bg(cx.theme().border))
            .child(self.render_script_section(
                "Post-response Script",
                &self.post_response_input,
                ScriptContext::PostResponse,
                cx,
            ))
    }
}
