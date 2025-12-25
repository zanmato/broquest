use crate::icon::IconName;
use gpui::{
    App, Context, Entity, EventEmitter, Focusable, Subscription, Window, div, prelude::*, px,
};
use gpui_component::{
    ActiveTheme, Sizable, StyledExt,
    button::Button,
    h_flex,
    input::{Input, InputEvent, InputState},
    v_flex,
};

#[derive(Debug, Clone, PartialEq)]
pub enum ScriptEditorEvent {
    ScriptChanged,
}

#[derive(Debug)]
pub struct ScriptEditor {
    pre_request_input: Entity<InputState>,
    post_response_input: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

impl EventEmitter<ScriptEditorEvent> for ScriptEditor {}

impl ScriptEditor {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let pre_request_input = cx.new(|cx| InputState::new(window, cx).code_editor("javascript"));

        let post_response_input =
            cx.new(|cx| InputState::new(window, cx).code_editor("javascript"));

        // Set up subscriptions for script input change events
        let pre_subscription = cx.subscribe_in(&pre_request_input, window, {
            move |_this: &mut Self,
                  input_state: &Entity<InputState>,
                  event: &InputEvent,
                  window,
                  cx| {
                if let InputEvent::Change = event {
                    if input_state.read(cx).focus_handle(cx).is_focused(window) {
                        cx.emit(ScriptEditorEvent::ScriptChanged);
                    }
                }
            }
        });

        let post_subscription = cx.subscribe_in(&post_response_input, window, {
            move |_this: &mut Self,
                  input_state: &Entity<InputState>,
                  event: &InputEvent,
                  window,
                  cx| {
                if let InputEvent::Change = event {
                    if input_state.read(cx).focus_handle(cx).is_focused(window) {
                        cx.emit(ScriptEditorEvent::ScriptChanged);
                    }
                }
            }
        });

        Self {
            pre_request_input,
            post_response_input,
            _subscriptions: vec![pre_subscription, post_subscription],
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

    fn render_script_section(
        &self,
        title: &str,
        input: &Entity<InputState>,
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
            )
            .child(
                div().flex_1().child(
                    Input::new(input)
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
            .child(self.render_script_section("Pre-request Script", &self.pre_request_input, cx))
            .child(div().h_px().bg(cx.theme().border))
            .child(self.render_script_section(
                "Post-response Script",
                &self.post_response_input,
                cx,
            ))
    }
}
