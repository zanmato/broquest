use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    ParentElement, Render, Styled, Window, actions, div, prelude::*,
};
use gpui_kit::component::{
    ActiveTheme as _, StyledExt, WindowExt,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputState},
    kbd::Kbd,
    notification::NotificationType,
    v_flex,
};

use super::manager::CollectionManager;
use crate::{app_events::AppEvent, ui::icon::IconName};

/// The key context of the group editor, where its Save shortcut applies.
pub(crate) const CONTEXT: &str = "group_editor";

actions!(group_editor, [Save]);

pub struct GroupEditor {
    collection_path: String,
    group_name: Option<String>, // Some for editing, None for new group
    name_input: Entity<InputState>,
    focus_handle: FocusHandle,
}

impl GroupEditor {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        collection_path: String,
        group_name: Option<String>,
    ) -> Self {
        let initial_name = group_name.clone().unwrap_or_default();
        let name_input = cx.new(|cx| InputState::new(window, cx).default_value(&initial_name));

        Self {
            collection_path,
            group_name,
            name_input,
            focus_handle: cx.focus_handle(),
        }
    }

    fn save_group(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let group_name = self.name_input.read(cx).value().trim().to_string();

        if group_name.is_empty() {
            window.push_notification((NotificationType::Error, "Group name cannot be empty"), cx);
            return;
        }

        let collection_path = self.collection_path.clone();
        let old_group_name = self.group_name.clone();

        let manager = CollectionManager::global(cx);
        let result = manager.update(cx, |collection_manager, cx| {
            if let Some(old_name) = &old_group_name {
                // Renaming existing group
                if old_name == &group_name {
                    // Name hasn't changed, nothing to do
                    Ok(())
                } else {
                    collection_manager.rename_group(&collection_path, old_name, &group_name, cx)
                }
            } else {
                // Creating new group
                collection_manager.create_group(&collection_path, &group_name, cx)
            }
        });

        match result {
            Ok(()) => {
                tracing::info!(
                    "Group '{}' saved successfully in collection '{}'",
                    group_name,
                    collection_path
                );

                window
                    .push_notification((NotificationType::Success, "Group saved successfully"), cx);

                // Update the old group name to the new name so subsequent edits work correctly
                self.group_name = Some(group_name.clone());

                // Emit event to refresh the collections panel
                cx.emit(AppEvent::GroupCreated {
                    collection_path: collection_path.into(),
                    group_name: group_name.into(),
                });
            }
            Err(e) => {
                tracing::error!("Failed to save group: {}", e);
                window.push_notification((NotificationType::Error, "Failed to save group"), cx);
            }
        }
    }

    /// Get the name input entity for external subscriptions
    /// Take the group's new name after it was renamed elsewhere, such as in
    /// the sidebar. Returns whether this editor is of that group.
    pub fn follow_rename(
        &mut self,
        old_name: &str,
        new_name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.group_name.as_deref() != Some(old_name) {
            return false;
        }
        self.group_name = Some(new_name.to_string());
        self.name_input.update(cx, |input, cx| {
            input.set_value(new_name.to_string(), window, cx);
        });
        true
    }

    pub fn name_input(&self) -> &Entity<InputState> {
        &self.name_input
    }
}

impl EventEmitter<AppEvent> for GroupEditor {}

impl Focusable for GroupEditor {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for GroupEditor {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .track_focus(&self.focus_handle)
            .key_context(CONTEXT)
            .on_action(cx.listener(|this: &mut GroupEditor, &Save, window, cx| {
                this.save_group(window, cx);
            }))
            .flex_1()
            .size_full()
            .child(
                // Content area with padding
                v_flex()
                    .flex_1()
                    .gap_3()
                    .p_3()
                    .child(
                        div()
                            .text_sm()
                            .font_medium()
                            .text_color(cx.theme().muted_foreground)
                            .child("Group Name"),
                    )
                    .child(div().child(Input::new(&self.name_input))),
            )
            .child(
                h_flex()
                    .gap_2()
                    .p_3()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .justify_end()
                    .child(
                        Button::new("save_group")
                            .primary()
                            .compact()
                            .icon(IconName::Save)
                            .label("Save Group")
                            // The shortcut as bound, which the user can change.
                            .children(Kbd::binding_for_action(&Save, Some(CONTEXT), window))
                            .on_click(
                                cx.listener(|this, _, window, cx| this.save_group(window, cx)),
                            ),
                    ),
            )
    }
}
