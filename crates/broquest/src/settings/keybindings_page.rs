use crate::app_settings::AppSettings;
use crate::keybindings::{
    BINDING_GROUPS, BindingDef, CUSTOMIZABLE_BINDINGS, conflicting_actions, default_keystroke,
    effective_keystroke, is_bindable, parse_keystrokes, same_keystrokes,
};
use crate::ui::icon::IconName;
use gpui::{
    App, Context, FocusHandle, InteractiveElement, IntoElement, Keystroke, ParentElement, Render,
    SharedString, StatefulInteractiveElement, Styled, Subscription, WeakEntity, Window, div,
    prelude::FluentBuilder as _, px,
};
use gpui_kit::component::{
    ActiveTheme, Sizable,
    button::{Button, ButtonVariants},
    h_flex,
    kbd::Kbd,
    label::Label,
    setting::{SettingGroup, SettingItem, SettingPage},
    v_flex,
};

use super::view::SettingsView;

/// Build the "Keybindings" settings page. Each customizable action gets a
/// recorder for its shortcut, listed under its group; the page's search finds
/// them by name. Edits are persisted as `keybinding.<action>` settings and
/// take effect on the next launch.
pub fn keybindings_page(view_handle: WeakEntity<SettingsView>) -> SettingPage {
    let groups = BINDING_GROUPS
        .iter()
        .map(|&group| {
            let items = CUSTOMIZABLE_BINDINGS
                .iter()
                .filter(|def| def.group == group)
                .map(|def| binding_item(def, view_handle.clone()))
                .collect::<Vec<_>>();
            SettingGroup::new().title(group).items(items)
        })
        .collect::<Vec<_>>();

    SettingPage::new("Keybindings")
        .description("Changes take effect after a restart.")
        .resettable(true)
        .groups(groups)
}

/// A row laid out like the page's other items. It is built by hand because
/// an item's description is text only and cannot hold the `Kbd` tags.
fn binding_item(def: &'static BindingDef, view_handle: WeakEntity<SettingsView>) -> SettingItem {
    let action = def.action;
    SettingItem::render({
        let view_handle = view_handle.clone();
        move |_, window, cx| {
            let recorder = window.use_keyed_state(
                SharedString::from(format!("keybinding-{action}")),
                cx,
                |window, cx| KeybindingRecorder::new(action, view_handle.clone(), window, cx),
            );
            // The page is rebuilt on every render, so a conflict shows as
            // soon as a shortcut is recorded.
            let conflicts = conflicting_actions(&AppSettings::global(cx).settings, action);

            h_flex()
                .w_full()
                .justify_between()
                .items_center()
                .gap_3()
                .child(
                    v_flex()
                        .flex_1()
                        .gap_1()
                        .child(Label::new(def.label).text_sm())
                        .child(
                            h_flex()
                                .gap_1()
                                .items_center()
                                .text_sm()
                                .text_color(cx.theme().muted_foreground)
                                .map(|this| match parse_keystrokes(def.default) {
                                    Some(chords) if !chords.is_empty() => this
                                        .child("Default")
                                        .children(chords.into_iter().map(Kbd::new)),
                                    _ => this.child("Unbound by default"),
                                }),
                        )
                        .when(!conflicts.is_empty(), |this| {
                            this.child(
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().warning)
                                    .child(format!("Also used by {}", conflicts.join(", "))),
                            )
                        }),
                )
                .child(recorder)
        }
    })
    .keywords([def.label, def.group])
    .on_reset(
        move |cx| !is_default(action, cx),
        move |_, cx| save_keystrokes(action, default_keystroke(action), &view_handle, cx),
    )
}

fn is_default(action: &str, cx: &App) -> bool {
    same_keystrokes(
        &effective_keystroke(&AppSettings::global(cx).settings, action),
        default_keystroke(action),
    )
}

fn save_keystrokes(
    action: &'static str,
    keystrokes: &str,
    view_handle: &WeakEntity<SettingsView>,
    cx: &mut App,
) {
    AppSettings::global_mut(cx)
        .settings
        .keybindings
        .insert(action.to_string(), keystrokes.to_string());

    if let Some(view) = view_handle.upgrade() {
        view.update(cx, |view, cx| {
            view.save_setting_debounced(format!("keybinding.{action}"), keystrokes.to_string(), cx);
        });
    }
}

/// The field of one action: shows its shortcut, and records a new one from
/// the keyboard after a click. Recording rather than typing means only a
/// keystroke the keyboard can produce is ever stored.
pub(super) struct KeybindingRecorder {
    action: &'static str,
    view_handle: WeakEntity<SettingsView>,
    focus_handle: FocusHandle,
    /// Held while recording. Keystrokes are intercepted ahead of the keymap,
    /// so a shortcut that is already bound can be recorded without running.
    interceptor: Option<Subscription>,
    _blur: Subscription,
}

impl KeybindingRecorder {
    pub(super) fn new(
        action: &'static str,
        view_handle: WeakEntity<SettingsView>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        let blur = cx.on_blur(&focus_handle, window, |this, _, cx| this.stop_recording(cx));
        Self {
            action,
            view_handle,
            focus_handle,
            interceptor: None,
            _blur: blur,
        }
    }

    pub(super) fn is_recording(&self) -> bool {
        self.interceptor.is_some()
    }

    pub(super) fn start_recording(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.focus_handle.focus(window, cx);
        let recorder = cx.weak_entity();
        self.interceptor = Some(cx.intercept_keystrokes(move |event, window, cx| {
            if let Some(recorder) = recorder.upgrade() {
                recorder.update(cx, |recorder, cx| {
                    recorder.record(&event.keystroke, window, cx)
                });
            }
        }));
        cx.notify();
    }

    fn stop_recording(&mut self, cx: &mut Context<Self>) {
        self.interceptor = None;
        cx.notify();
    }

    /// Escape cancels and Backspace or Delete unbinds, each without a
    /// modifier. Anything else that cannot be bound is swallowed and the
    /// recorder keeps waiting.
    pub(super) fn record(
        &mut self,
        keystroke: &Keystroke,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.focus_handle.is_focused(window) {
            return;
        }
        cx.stop_propagation();

        if !keystroke.modifiers.modified() {
            match keystroke.key.as_str() {
                "escape" => return self.stop_recording(cx),
                "backspace" | "delete" => return self.save("", cx),
                _ => {}
            }
        }
        if is_bindable(keystroke) {
            self.save(&keystroke.unparse(), cx);
        }
    }

    fn save(&mut self, keystrokes: &str, cx: &mut Context<Self>) {
        save_keystrokes(self.action, keystrokes, &self.view_handle, cx);
        self.stop_recording(cx);
    }
}

impl Render for KeybindingRecorder {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let action = self.action;
        let keystrokes = effective_keystroke(&AppSettings::global(cx).settings, action);
        let recording = self.is_recording();

        h_flex()
            .gap_1()
            .items_center()
            .when(!is_default(action, cx), |this| {
                this.child(
                    Button::new("reset")
                        .ghost()
                        .xsmall()
                        .icon(IconName::RotateCcw)
                        .tooltip("Reset to Default")
                        .on_click(cx.listener(move |this, _, _, cx| {
                            this.save(default_keystroke(action), cx);
                        })),
                )
            })
            .child(
                h_flex()
                    .id("shortcut")
                    .track_focus(&self.focus_handle)
                    .w(px(200.))
                    .h_8()
                    .px_2()
                    .gap_1()
                    .items_center()
                    .rounded(cx.theme().radius)
                    .border_1()
                    .border_color(if recording {
                        cx.theme().ring
                    } else {
                        cx.theme().input
                    })
                    .bg(cx.theme().background)
                    .text_sm()
                    .cursor_pointer()
                    .on_click(cx.listener(|this, _, window, cx| {
                        if this.is_recording() {
                            this.stop_recording(cx);
                        } else {
                            this.start_recording(window, cx);
                        }
                    }))
                    .map(|this| {
                        let muted = cx.theme().muted_foreground;
                        if recording {
                            return this.child(div().text_color(muted).child("Press a shortcut"));
                        }
                        match parse_keystrokes(&keystrokes) {
                            Some(chords) if chords.is_empty() => {
                                this.child(div().text_color(muted).child("Not set"))
                            }
                            Some(chords) => this.children(chords.into_iter().map(Kbd::new)),
                            // A value typed before shortcuts were recorded.
                            None => {
                                this.child(div().text_color(cx.theme().danger).child(keystrokes))
                            }
                        }
                    }),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Settings;
    use gpui::{AppContext as _, TestAppContext};

    #[gpui::test]
    fn test_recorder_stores_only_bindable_keystrokes(cx: &mut TestAppContext) {
        cx.update(|cx| {
            let settings = AppSettings::new(cx, Settings::default());
            cx.set_global(settings);
        });
        let cx = cx.add_empty_window();
        let action = "CopyResponse";

        let recorder = cx.update(|window, cx| {
            cx.new(|cx| KeybindingRecorder::new(action, WeakEntity::new_invalid(), window, cx))
        });
        let press = |keystroke: &str, cx: &mut gpui::VisualTestContext| {
            let keystroke = Keystroke::parse(keystroke).expect("keystroke");
            recorder.update_in(cx, |recorder, window, cx| {
                if !recorder.is_recording() {
                    recorder.start_recording(window, cx);
                }
                recorder.record(&keystroke, window, cx);
                recorder.is_recording()
            })
        };
        let stored = |cx: &mut gpui::VisualTestContext| {
            cx.update(|_, cx| effective_keystroke(&AppSettings::global(cx).settings, action))
        };

        assert!(press("a", cx), "a bare letter keeps the recorder open");
        assert!(press("shift-a", cx));
        assert_eq!(stored(cx), default_keystroke(action));

        assert!(!press("ctrl-shift-y", cx));
        assert_eq!(stored(cx), "ctrl-shift-y");

        assert!(!press("escape", cx), "escape cancels");
        assert_eq!(stored(cx), "ctrl-shift-y");

        assert!(!press("backspace", cx), "backspace unbinds");
        assert_eq!(stored(cx), "");
    }
}
