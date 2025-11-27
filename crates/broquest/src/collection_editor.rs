use gpui::{App, Context, Entity, Window, div, prelude::*, px};
use gpui_component::{
    ActiveTheme, StyledExt, WindowExt,
    button::Button,
    h_flex,
    input::{Input, InputState},
    notification::NotificationType,
    tab::{Tab, TabBar},
    v_flex,
};

use crate::{
    app_database::AppDatabase, collection_types::CollectionMeta,
    environment_editor::EnvironmentEditor,
};
use crate::{app_database::CollectionData, icon::IconName};
use crate::{collection_manager::CollectionManager, collection_types::CollectionToml};

pub struct CollectionEditor {
    active_tab: usize,
    collection_data: CollectionToml,
    collection_path: String,
    #[allow(dead_code)]
    collection_id: Option<i64>,
    environment_editor: Entity<EnvironmentEditor>,
    name_input: Entity<InputState>,
    path_input: Entity<InputState>,
}

impl CollectionEditor {
    pub fn new(
        window: &mut Window,
        cx: &mut Context<Self>,
        collection_data: CollectionToml,
        collection_path: String,
        collection_id: Option<i64>,
    ) -> Self {
        let environment_editor =
            cx.new(|cx| EnvironmentEditor::new(window, cx, &collection_data.collection.name));

        let name_input = cx
            .new(|cx| InputState::new(window, cx).default_value(&collection_data.collection.name));

        let path_input = cx.new(|cx| InputState::new(window, cx).default_value(&collection_path));

        let editor = Self {
            active_tab: 0, // Collection tab
            collection_data,
            collection_path,
            collection_id,
            environment_editor,
            name_input,
            path_input,
        };

        // Load initial environments data from CollectionManager
        tracing::info!(
            "CollectionEditor::new called with collection_id: {:?}",
            collection_id
        );
        if let Some(collection_id) = collection_id {
            tracing::info!("Looking for CollectionManager global");
            let collection_manager = CollectionManager::global(cx);
            if let Some(environments) =
                collection_manager.get_collection_environments(collection_id)
            {
                tracing::info!(
                    "Loading {} environments for collection editor: {:?}",
                    environments.len(),
                    environments.iter().map(|e| &e.name).collect::<Vec<_>>()
                );
                editor.environment_editor.update(cx, |env_editor, cx| {
                    env_editor.load_environments(&environments, window, cx);
                });
            } else {
                tracing::warn!("No environments found for collection_id: {}", collection_id);
            }
        } else {
            tracing::info!("No collection_id provided for collection editor");
        }

        editor
    }

    pub fn get_collection_data_for_save(&self, cx: &App) -> CollectionToml {
        let name = self.name_input.read(cx).value().to_string();
        let version = self.collection_data.collection.version.clone();
        let collection_type = self.collection_data.collection.collection_type.clone();
        let description = self.collection_data.collection.description.clone();
        let ignore = self.collection_data.collection.ignore.clone();

        // Get environments from the environment editor
        let environments = self
            .environment_editor
            .read(cx)
            .get_environments_for_save(cx);

        CollectionToml {
            collection: CollectionMeta {
                name,
                version,
                collection_type,
                description,
                ignore,
            },
            environments,
        }
    }

    pub fn save_secrets(&self, cx: &App) -> Result<(), Box<dyn std::error::Error>> {
        self.environment_editor.read(cx).save_secrets(cx)
    }

    fn set_active_tab(&mut self, tab_index: usize, cx: &mut Context<Self>) {
        self.active_tab = tab_index;
        cx.notify();
    }

    fn render_collection_tab(&self, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .gap_3()
            .p_3()
            .child(
                // Collection name input
                v_flex().gap_2().children([
                    div()
                        .text_sm()
                        .font_medium()
                        .text_color(cx.theme().muted_foreground)
                        .child("Name"),
                    div().child(Input::new(&self.name_input)),
                ]),
            )
            .child(
                // Directory path input (read-only for now)
                v_flex().gap_2().children([
                    div()
                        .text_sm()
                        .font_medium()
                        .text_color(cx.theme().muted_foreground)
                        .child("Collection Path"),
                    h_flex()
                        .gap_2()
                        .child(
                            Input::new(&self.path_input)
                                .font_family(cx.theme().font_family.clone()),
                        )
                        .child(
                            Button::new("browse_path")
                                .outline()
                                .icon(IconName::FolderOpen)
                                .on_click(cx.listener(|this, _, window, cx| {
                                    this.handle_browse_directory(window, cx)
                                })),
                        ),
                ]),
            )
    }

    fn render_environments_tab(&self, _cx: &mut Context<Self>) -> impl IntoElement {
        v_flex().flex_1().child(self.environment_editor.clone())
    }

    fn render_tab_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        TabBar::new("collection-tabs")
            .left(px(-1.)) // Avoid double border
            .selected_index(self.active_tab)
            .on_click(cx.listener(|this, _ix: &usize, _window, cx| {
                this.set_active_tab(*_ix, cx);
            }))
            .children(vec![
                Tab::new().label("Collection"),
                Tab::new().label("Environments"),
            ])
    }

    fn render_tab_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div().flex_1().size_full().child(match self.active_tab {
            0 => {
                let content = self.render_collection_tab(cx);
                div().child(content)
            }
            1 => {
                let content = self.render_environments_tab(cx);
                div().child(content)
            }
            _ => {
                let content = self.render_collection_tab(cx);
                div().child(content)
            }
        })
    }

    // Event handlers
    fn handle_save_collection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let collection_data = self.get_collection_data_for_save(cx);

        // Get current path from input
        let current_path = self.path_input.read(cx).value().to_string();

        // Save collection to file
        if current_path.is_empty() {
            tracing::warn!("Cannot save collection: no path specified");
            // TODO: Show error to user
            return;
        }

        // Save to database first to get proper ID
        let app_database = AppDatabase::global(cx).clone();
        let collection_data_clone = collection_data.clone();
        let current_path_clone = current_path.clone();

        let database_id = async_std::task::block_on(async move {
            app_database
                .save_collection(&CollectionData {
                    id: None,
                    name: collection_data_clone.collection.name.clone(),
                    path: current_path_clone.clone(),
                    position: 2,
                    created_at: chrono::Utc::now(),
                    updated_at: chrono::Utc::now(),
                })
                .await
        });

        let database_id = match database_id {
            Ok(id) => {
                tracing::info!("Collection saved to database with id: {}", id);
                Some(id)
            }
            Err(e) => {
                tracing::error!("Failed to save collection to database: {}", e);
                None
            }
        };

        // Use CollectionManager to save the collection with proper database ID
        let save_result = cx.update_global(|collection_manager: &mut CollectionManager, _cx| {
            collection_manager.save_collection(&collection_data, &current_path, database_id)
        });

        match save_result {
            Ok(()) => {
                // Update the stored path
                self.collection_path = current_path.clone();

                tracing::info!("Collection saved successfully to: {}", current_path);
                tracing::info!("Collection name: {}", collection_data.collection.name);

                // Show success notification
                window.push_notification(
                    (NotificationType::Success, "Collection saved successfully."),
                    cx,
                );
            }
            Err(e) => {
                tracing::error!("Failed to save collection: {}", e);
                // TODO: Show error to user

                window
                    .push_notification((NotificationType::Error, "Failed to save collection."), cx);
            }
        }

        // Save secrets
        if let Err(e) = self.save_secrets(cx) {
            tracing::error!("Failed to save secrets: {}", e);
        } else {
            tracing::info!("Secrets saved successfully");
        }
    }

    fn handle_browse_directory(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let path = cx.prompt_for_paths(gpui::PathPromptOptions {
            files: false,
            directories: true,
            multiple: false,
            prompt: Some("Select a directory for the collection".into()),
        });

        let directory_input = self.path_input.clone();
        cx.spawn_in(window, async move |_, window| {
            if let Some(path) = path.await.ok()?.ok()?
                && let Some(dir_path) = path.first()
                && let Some(dir_str) = dir_path.to_str()
            {
                window
                    .update(|window, cx| {
                        directory_input.update(cx, |input, cx| {
                            input.set_value(dir_str.to_string(), window, cx);
                        });
                    })
                    .ok();
            }
            Some(())
        })
        .detach();
    }

    /// Get the name input entity for external subscriptions
    pub fn name_input(&self) -> &Entity<InputState> {
        &self.name_input
    }
}

impl Render for CollectionEditor {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .flex_1()
            .size_full()
            .child(
                // Tab bar
                self.render_tab_bar(cx),
            )
            .child(
                // Tab content
                self.render_tab_content(cx),
            )
            .child(
                // Save button at the bottom
                h_flex()
                    .gap_2()
                    .p_3()
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .justify_end()
                    .child(
                        Button::new("save_collection_bottom")
                            .icon(IconName::Save)
                            .label("Save Collection")
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.handle_save_collection(window, cx)
                            })),
                    ),
            )
    }
}
