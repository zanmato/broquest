use gpui::{
    AppContext, BorrowAppContext, Context, Entity, EventEmitter, IntoElement, ParentElement,
    Render, Styled, Window, div,
};
use gpui_component::{
    ActiveTheme as _, StyledExt, WindowExt,
    button::Button,
    h_flex,
    input::{Input, InputState},
    notification::NotificationType,
    v_flex,
};

use crate::{app_events::AppEvent, collection_manager::CollectionManager, icon::IconName};

pub struct GroupEditor {
    collection_path: String,
    group_name: Option<String>, // Some for editing, None for new group
    name_input: Entity<InputState>,
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

        let result = cx.update_global(|collection_manager: &mut CollectionManager, _cx| {
            if let Some(old_name) = &old_group_name {
                // Renaming existing group
                if old_name == &group_name {
                    // Name hasn't changed, nothing to do
                    Ok(())
                } else {
                    collection_manager.rename_group(&collection_path, old_name, &group_name)
                }
            } else {
                // Creating new group
                collection_manager.create_group(&collection_path, &group_name)
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
    pub fn name_input(&self) -> &Entity<InputState> {
        &self.name_input
    }
}

impl EventEmitter<AppEvent> for GroupEditor {}

impl Render for GroupEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .flex_1()
            .size_full()
            .gap_4()
            .p_6()
            .child(
                v_flex()
                    .gap_2()
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
                h_flex().gap_2().justify_end().child(
                    Button::new("save_group")
                        .icon(IconName::Save)
                        .label("Save Group")
                        .on_click(cx.listener(|this, _, window, cx| this.save_group(window, cx))),
                ),
            )
    }
}
