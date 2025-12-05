use gpui::{
    AppContext, ClickEvent, Context, Entity, EventEmitter, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui::{BorrowAppContext, SharedString};
use gpui_component::menu::ContextMenuExt;
use gpui_component::{
    ActiveTheme as _, Icon, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    list::ListItem,
    menu::PopupMenuItem,
    tree::{TreeEntry, TreeItem, TreeState, tree},
    v_flex,
};

use crate::app_database::{AppDatabase, CollectionData};
use crate::app_events::AppEvent;
use crate::collection_manager::{CollectionInfo, CollectionManager};
use crate::icon::IconName;

/// Icon and color combination for tree items
#[derive(Clone)]
pub struct TreeItemIcon {
    pub icon: Option<IconName>,
    pub prefix: Option<SharedString>,
    pub color: gpui::Rgba,
}

pub struct CollectionsPanel {
    collections: Vec<CollectionData>,
    tree_state: Entity<TreeState>,
    tree_item_metadata: std::collections::HashMap<String, TreeItemMetadata>, // Map hierarchical key -> metadata
    request_data_map: std::collections::HashMap<String, crate::request_editor::RequestData>, // Map item_id -> RequestData
}

/// Type of tree item in the metadata context
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum TreeItemKind {
    Collection,
    Group,
    Request,
}

/// Metadata for tree items to enable proper context menu actions
#[derive(Clone)]
pub struct TreeItemMetadata {
    pub name: String,
    pub kind: TreeItemKind,
    pub icon: TreeItemIcon,
    pub collection_path: String, // Path to collection directory for collection items
    pub group_path: Option<String>, // Path to group directory for group items
}

impl CollectionsPanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        let tree_state = cx.new(|cx| TreeState::new(cx));

        Self {
            collections: Vec::new(),
            tree_state,
            tree_item_metadata: std::collections::HashMap::new(),
            request_data_map: std::collections::HashMap::new(),
        }
    }

    pub fn refresh_collections(&mut self, cx: &mut Context<Self>) {
        // Load collections, this will cause the observe global to call Self::load_collections
        cx.update_global(|collection_manager: &mut CollectionManager, _cx| {
            if let Err(e) = collection_manager.load_saved(_cx) {
                tracing::error!("Failed to load collections from database: {}", e);
            }
        });
    }

    pub fn load_collections(&mut self, cx: &mut Context<Self>) {
        // Get collections from the global CollectionManager after loading
        let mut collection_infos = {
            let collection_manager = CollectionManager::global(cx);
            collection_manager.get_all_collections()
        };

        // Sort collections alphabetically by name
        collection_infos.sort_by(|a, b| a.data.name.cmp(&b.data.name));

        // Convert CollectionInfo to CollectionData for the tree
        self.collections = collection_infos
            .iter()
            .map(|info| info.data.clone())
            .collect();

        tracing::info!(
            "Loaded {} collections from global manager",
            self.collections.len()
        );

        // Clone the collection info to avoid borrow issues
        let collection_infos_owned: Vec<CollectionInfo> =
            collection_infos.iter().map(|&info| info.clone()).collect();

        // Build the complete tree with all collections and their requests
        self.build_tree_from_collections(&collection_infos_owned, cx);
    }

    /// Build the complete tree with all collections, groups, and requests
    fn build_tree_from_collections(
        &mut self,
        collection_infos: &[CollectionInfo],
        cx: &mut Context<Self>,
    ) {
        // Clear existing metadata
        self.tree_item_metadata.clear();
        self.request_data_map.clear();

        let mut tree_items = Vec::new();
        let mut total_requests = 0;

        for collection_info in collection_infos {
            let collection_data = &collection_info.data;
            let collection_path = &collection_data.path;
            let collection_id = format!("collection_{}", collection_path.replace('/', "_"));

            let mut child_items = Vec::new();

            // Add direct collection requests (root level) - sort alphabetically by name
            let mut sorted_requests: Vec<_> = collection_info.requests.iter().collect();
            sorted_requests.sort_by(|a, b| a.1.name.cmp(&b.1.name));

            for (index, (_file_path, request)) in sorted_requests.into_iter().enumerate() {
                let request_id = format!("request_{}_{}", collection_path.replace('/', "_"), index);
                let request_tree_item =
                    TreeItem::new(request_id.clone(), request.name.clone()).children(vec![]);

                let request_metadata = TreeItemMetadata {
                    name: request.name.clone(),
                    kind: TreeItemKind::Request,
                    icon: TreeItemIcon {
                        icon: None,
                        prefix: Some(SharedString::from(request.method.as_str())),
                        color: request.method.get_color(cx),
                    },
                    collection_path: collection_data.path.clone(),
                    group_path: None, // Root level request
                };

                self.tree_item_metadata
                    .insert(request_id.clone(), request_metadata);
                self.request_data_map.insert(request_id, request.clone());
                child_items.push(request_tree_item);
            }

            // Add group folders and their requests - sort groups alphabetically
            let mut sorted_groups: Vec<_> = collection_info.groups.iter().collect();
            sorted_groups.sort_by(|a, b| a.0.cmp(&b.0));

            for (group_name, group_info) in sorted_groups {
                let group_id =
                    format!("group_{}_{}", collection_path.replace('/', "_"), group_name);

                // Build child items for group requests - sort alphabetically by name
                let mut sorted_group_requests: Vec<_> = group_info.requests.iter().collect();
                sorted_group_requests.sort_by(|a, b| a.1.name.cmp(&b.1.name));

                let mut group_child_items = Vec::new();
                for (index, (_file_path, request)) in sorted_group_requests.into_iter().enumerate()
                {
                    let request_id = format!(
                        "request_{}_{}_{}",
                        collection_path.replace('/', "_"),
                        group_name,
                        index
                    );
                    let request_tree_item =
                        TreeItem::new(request_id.clone(), request.name.clone()).children(vec![]);

                    let request_metadata = TreeItemMetadata {
                        name: request.name.clone(),
                        kind: TreeItemKind::Request,
                        icon: TreeItemIcon {
                            icon: None,
                            prefix: Some(SharedString::from(request.method.as_str())),
                            color: request.method.get_color(cx),
                        },
                        collection_path: collection_data.path.clone(),
                        group_path: Some(format!("{}/{}", collection_path, group_name)),
                    };

                    self.tree_item_metadata
                        .insert(request_id.clone(), request_metadata);
                    self.request_data_map.insert(request_id, request.clone());
                    group_child_items.push(request_tree_item);
                }

                total_requests += group_info.requests.len();

                // Create group tree item with its requests as children
                let group_tree_item =
                    TreeItem::new(group_id.clone(), group_name.clone()).children(group_child_items);

                // Store metadata for this group
                let group_metadata = TreeItemMetadata {
                    name: group_name.clone(),
                    kind: TreeItemKind::Group,
                    icon: TreeItemIcon {
                        icon: Some(IconName::Folder),
                        prefix: None,
                        color: cx.theme().magenta.into(), // Different color from collections
                    },
                    collection_path: collection_data.path.clone(),
                    group_path: Some(format!("{}/{}", collection_path, group_name)),
                };

                self.tree_item_metadata.insert(group_id, group_metadata);
                child_items.push(group_tree_item);
            }

            total_requests += collection_info.requests.len();

            // Create the collection tree item with all its children
            let collection_tree_item =
                TreeItem::new(collection_id.clone(), collection_data.name.clone())
                    .children(child_items);

            // Store metadata for this collection
            let collection_metadata = TreeItemMetadata {
                name: collection_data.name.clone(),
                kind: TreeItemKind::Collection,
                icon: TreeItemIcon {
                    icon: Some(IconName::Folder),
                    prefix: None,
                    color: cx.theme().blue.into(),
                },
                collection_path: collection_data.path.clone(),
                group_path: None,
            };

            self.tree_item_metadata
                .insert(collection_id, collection_metadata);
            tree_items.push(collection_tree_item);
        }

        tracing::info!(
            "Built tree with {} collections and {} total requests",
            collection_infos.len(),
            total_requests
        );

        // Update the tree state with all items including all children
        self.tree_state.update(cx, |state, cx| {
            state.set_items(tree_items, cx);
        });
    }

    /// Open a collection in a new tab
    fn open_collection_tab(&mut self, collection_path: &str, cx: &mut Context<Self>) {
        tracing::info!("Opening collection tab for path: {}", collection_path);

        let collection_manager = cx.try_global::<CollectionManager>();
        let Some(collection_manager) = collection_manager else {
            tracing::error!("Global CollectionManager not found");
            return;
        };

        match collection_manager.read_collection_toml(std::path::Path::new(collection_path)) {
            Ok(collection_data) => {
                tracing::info!(
                    "Successfully loaded collection data: {}",
                    collection_data.collection.name
                );

                tracing::info!("Opening collection tab for path: {}", collection_path);

                cx.emit(AppEvent::CreateNewCollectionTab {
                    collection_data,
                    collection_path: collection_path.to_string().into(),
                });
            }
            Err(e) => {
                tracing::error!(
                    "Failed to load collection data from {}: {}",
                    collection_path,
                    e
                );
            }
        }
    }

    /// Create a new request tab for a collection
    fn new_request_tab(&mut self, collection_path: &str, cx: &mut Context<Self>) {
        tracing::info!(
            "Creating new request tab for collection_path: {}",
            collection_path
        );

        // Emit NewRequest event to create the new request tab
        cx.emit(AppEvent::NewRequest {
            collection_path: collection_path.to_string().into(),
            group_path: None,
        });
    }

    /// Create a new request tab for a specific group
    fn new_request_in_group_tab(
        &mut self,
        collection_path: &str,
        group_path: &str,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            "Creating new request tab for collection_path: {} in group: {}",
            collection_path,
            group_path
        );

        cx.emit(AppEvent::NewRequest {
            collection_path: collection_path.to_string().into(),
            group_path: Some(group_path.to_string().into()),
        });
    }

    /// Delete a collection from CollectionManager and AppDatabase
    fn delete_collection(
        &mut self,
        collection_path: &str,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            "Deleting collection with collection_path: {}",
            collection_path
        );

        // Remove from local collections vector first
        self.collections
            .retain(|collection| collection.path != collection_path);

        // Update global CollectionManager to remove the collection
        cx.update_global(|collection_manager: &mut CollectionManager, _cx| {
            collection_manager.remove_collection(collection_path);
        });

        // Remove the collection from AppDatabase
        let db = AppDatabase::global(cx).clone();
        let collection_path_for_async = collection_path.to_string();
        let collection_path_for_event = collection_path.to_string().into();
        cx.spawn(async move |_, _| {
            match db.delete_collection(&collection_path_for_async).await {
                Ok(_) => {
                    tracing::info!(
                        "Successfully deleted collection '{}' from database",
                        collection_path_for_async
                    );
                }
                Err(e) => {
                    tracing::error!("Failed to delete collection from database: {}", e);
                }
            }
            Some(())
        })
        .detach();

        // Reload collections to rebuild the tree
        self.load_collections(cx);

        // Emit event to notify other parts of the app
        cx.emit(AppEvent::CollectionDeleted {
            collection_path: collection_path_for_event,
        });
    }

    /// Delete a request through CollectionManager
    fn delete_request(&mut self, request_id: &str, collection_path: &str, cx: &mut Context<Self>) {
        tracing::info!(
            "Deleting request with ID: {} from collection: {}",
            request_id,
            collection_path
        );

        // Get the request data for logging and file path extraction
        let Some(request_data) = self.request_data_map.get(request_id).cloned() else {
            tracing::error!("Could not find request data for ID: {}", request_id);
            return;
        };

        let request_name = request_data.name.clone();

        // Remove from local request_data_map first
        self.request_data_map.remove(request_id);

        // Use CollectionManager to delete the request
        cx.update_global(
            |collection_manager: &mut CollectionManager, _cx| match collection_manager
                .delete_request(collection_path, &request_data)
            {
                Ok(_) => {
                    tracing::info!(
                        "Successfully deleted request '{}' from collection {}",
                        request_name,
                        collection_path
                    );
                }
                Err(e) => {
                    tracing::error!(
                        "Failed to delete request '{}' from CollectionManager: {}",
                        request_name,
                        e
                    );
                }
            },
        );

        // Reload collections to rebuild the tree
        self.load_collections(cx);
    }

    /// Build context menu for a tree item based on its type
    fn build_context_menu(
        &self,
        item: &TreeItem,
        cx: &mut Context<Self>,
    ) -> impl Fn(
        gpui_component::menu::PopupMenu,
        &mut Window,
        &mut gpui::Context<gpui_component::menu::PopupMenu>,
    ) -> gpui_component::menu::PopupMenu
    + 'static {
        let item = item.clone();
        let entity = cx.entity();
        let metadata = self.get_tree_item_metadata(&item.id).cloned();

        move |this, window, _cx| {
            // Build context menu based on item type using metadata
            if let Some(metadata) = &metadata {
                match metadata.kind {
                    TreeItemKind::Collection => this
                        .item(
                            PopupMenuItem::new("Open Collection").on_click(window.listener_for(
                                &entity,
                                {
                                    let collection_path = metadata.collection_path.clone();
                                    let collection_name = metadata.name.clone();
                                    move |this, _, window, cx| {
                                        tracing::info!("Open collection: {}", collection_name);
                                        this.open_collection_tab(&collection_path, cx);
                                        window.focus_prev();
                                    }
                                },
                            )),
                        )
                        .item(
                            PopupMenuItem::new("New Request").on_click(window.listener_for(
                                &entity,
                                {
                                    let collection_path = metadata.collection_path.clone();
                                    move |this, _, window, cx| {
                                        this.new_request_tab(&collection_path, cx);
                                        window.focus_prev();
                                    }
                                },
                            )),
                        )
                        .separator()
                        .item(PopupMenuItem::new("Remove Collection").on_click(
                            window.listener_for(&entity, {
                                let collection_path = metadata.collection_path.clone();
                                move |this, _, window, cx| {
                                    tracing::info!("Remove collection");
                                    this.delete_collection(&collection_path, window, cx);
                                    window.focus_prev();
                                }
                            }),
                        )),
                    TreeItemKind::Request => {
                        this.item(PopupMenuItem::new("Delete Request").on_click(
                            window.listener_for(&entity, {
                                let collection_path = metadata.collection_path.clone();
                                let request_id_for_deletion = item.id.clone();
                                move |this, _, window, cx| {
                                    this.delete_request(
                                        &request_id_for_deletion,
                                        &collection_path,
                                        cx,
                                    );
                                    window.focus_prev();
                                }
                            }),
                        ))
                    }
                    TreeItemKind::Group => {
                        this.item(PopupMenuItem::new("New Request in Group").on_click(
                            window.listener_for(&entity, {
                                let collection_path = metadata.collection_path.clone();
                                let group_path = metadata.group_path.clone().unwrap_or_default();
                                move |this, _, window, cx| {
                                    this.new_request_in_group_tab(
                                        &collection_path,
                                        &group_path,
                                        cx,
                                    );
                                    window.focus_prev();
                                }
                            }),
                        ))
                    }
                }
            } else {
                // Default context menu
                this.label(item.label.clone())
            }
        }
    }

    /// Get tree item metadata by item_id
    fn get_tree_item_metadata(&self, item_id: &str) -> Option<&TreeItemMetadata> {
        self.tree_item_metadata.get(item_id)
    }

    fn render_header_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .gap_2()
            .px_3()
            .pt(px(5.))
            .pb(px(6.))
            .border_b_1()
            .border_color(cx.theme().border)
            .child(
                h_flex()
                    .justify_between()
                    .items_center()
                    .child(
                        Label::new("Collections")
                            .font_bold()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground),
                    )
                    .child(
                        Button::new("refresh_collections")
                            .xsmall()
                            .ghost()
                            .icon(Icon::new(IconName::Refresh).size(px(14.)))
                            .on_click(cx.listener(|this, _, _, cx| {
                                tracing::info!("Refresh collections clicked");
                                this.refresh_collections(cx);
                            })),
                    ),
            )
    }

    fn render_collections_tree(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let view = cx.entity();

        tree(&self.tree_state, move |ix, entry, selected, window, cx| {
            view.update(cx, |this, cx| {
                this.render_tree_item(ix, entry, selected, window, cx)
            })
        })
    }

    fn render_tree_item(
        &self,
        ix: usize,
        entry: &TreeEntry,
        selected: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> ListItem {
        let item = entry.item();
        let depth = entry.depth();

        // Get icon from metadata
        let mut tree_item_icon = self
            .get_tree_item_metadata(&item.id)
            .map(|metadata| metadata.icon.clone())
            .unwrap_or_else(|| TreeItemIcon {
                icon: None,
                prefix: None,
                color: cx.theme().foreground.into(),
            });

        // Update folder icons based on expansion state (check if it's a collection or group by looking at metadata)
        if let Some(metadata) = self.get_tree_item_metadata(&item.id)
            && (metadata.kind == TreeItemKind::Collection || metadata.kind == TreeItemKind::Group)
            && entry.is_expanded()
        {
            // This is an expanded collection/group, change icon to FolderOpen
            tree_item_icon = TreeItemIcon {
                icon: Some(IconName::FolderOpen),
                prefix: None,
                color: tree_item_icon.color,
            };
        }

        ListItem::new(ix)
            .selected(selected)
            .w_full()
            .px_3()
            .pl(px(12.) * depth as f32 + px(12.)) // Indent based on depth
            .my_0p5()
            .child(
                h_flex()
                    .id(("tree-item", ix))
                    .gap_2()
                    .items_center()
                    .when_some(tree_item_icon.icon, |this, icon_name| {
                        this.child(Icon::new(icon_name).text_color(tree_item_icon.color))
                    })
                    .when_some(tree_item_icon.prefix, |this, prefix| {
                        this.child(
                            div()
                                .font_family(cx.theme().mono_font_family.clone())
                                .text_sm()
                                .font_bold()
                                .text_color(tree_item_icon.color)
                                .pt(px(3.))
                                .child(prefix),
                        )
                    })
                    .child(Label::new(item.label.clone()).text_sm())
                    .context_menu(self.build_context_menu(item, cx))
                    .when(entry.is_folder(), |this| {
                        this.child(if entry.is_expanded() {
                            Icon::new(IconName::ChevronDown)
                        } else {
                            Icon::new(IconName::ChevronRight)
                        })
                    }),
            )
            .on_click(cx.listener({
                let item = entry.item().clone();
                move |this, event: &ClickEvent, _window, cx| {
                    tracing::info!(
                        "Click event received for tree item: {} (click_count: {})",
                        item.id,
                        event.click_count()
                    );

                    if event.click_count() == 2 {
                        // Double-click - open in tab
                        if let Some(metadata) = this.get_tree_item_metadata(&item.id) {
                            let metadata = metadata.clone();
                            match metadata.kind {
                                TreeItemKind::Request => {
                                    // Open request in new tab
                                    if let Some(request_data) =
                                        this.request_data_map.get(item.id.as_ref()).cloned()
                                    {
                                        tracing::info!(
                                            "Opening request in new tab: {}",
                                            metadata.name
                                        );
                                        cx.emit(AppEvent::CreateNewRequestTab {
                                            request_data,
                                            collection_path: metadata
                                                .collection_path
                                                .clone()
                                                .into(),
                                        });
                                    }
                                }
                                TreeItemKind::Collection => {
                                    // Open collection in new tab
                                    this.open_collection_tab(&metadata.collection_path, cx);
                                }
                                TreeItemKind::Group => {}
                            }
                        }
                    }
                    // Single-click is handled by the tree component for expansion
                }
            }))
    }
}

impl Render for CollectionsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_2()
            .bg(cx.theme().sidebar_primary_foreground)
            .px(px(1.))
            .child(self.render_header_section(cx))
            .child(self.render_collections_tree(cx))
    }
}

impl EventEmitter<AppEvent> for CollectionsPanel {}
