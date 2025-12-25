use gpui::{
    App, AppContext, ClickEvent, Context, Entity, EventEmitter, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui::{BorrowAppContext, SharedString};
use gpui_component::{
    ActiveTheme as _, Icon, Sizable, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    label::Label,
    list::ListItem,
    menu::PopupMenuItem,
    notification::NotificationType,
    v_flex,
};
use smol::Timer;
use std::collections::HashMap;

use crate::app_database::{AppDatabase, CollectionData};
use crate::app_events::AppEvent;
use crate::collection_manager::{CollectionInfo, CollectionManager};
use crate::icon::IconName;
use crate::ui::draggable_tree::{
    DragIcon, DraggableTree, DraggableTreeDelegate, DraggableTreeState, DraggedTreeItem, TreeEntry,
    TreeItem,
};

/// Icon and color combination for tree items
#[derive(Clone)]
pub struct TreeItemIcon {
    pub icon: Option<IconName>,
    pub prefix: Option<SharedString>,
    pub color_fn: fn(&App) -> gpui::Hsla,
}

pub struct CollectionsPanel {
    collections: Vec<CollectionData>,
    tree_state: Entity<DraggableTreeState<CollectionsTreeDelegate>>,
    tree_item_metadata: HashMap<String, TreeItemMetadata>, // Map hierarchical key -> metadata
    request_data_map: HashMap<String, crate::request_editor::RequestData>, // Map item_id -> RequestData
}

struct CollectionsTreeDelegate {
    parent: Entity<CollectionsPanel>,
}

impl DraggableTreeDelegate for CollectionsTreeDelegate {
    fn render_item(
        &self,
        ix: usize,
        entry: &TreeEntry,
        selected: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> ListItem {
        let item = entry.item();
        let depth = entry.depth();

        let connections_panel = self.parent.read(cx);

        // Get icon from metadata
        let mut tree_item_icon = connections_panel
            .get_tree_item_metadata(&item.id)
            .map(|metadata| metadata.icon.clone())
            .unwrap_or_else(|| TreeItemIcon {
                icon: None,
                prefix: None,
                color_fn: |cx: &App| cx.theme().foreground,
            });

        // Update folder icons based on expansion state (check if it's a collection or group by looking at metadata)
        if let Some(metadata) = connections_panel.get_tree_item_metadata(&item.id)
            && (metadata.kind == TreeItemKind::Collection || metadata.kind == TreeItemKind::Group)
            && entry.is_expanded()
        {
            // This is an expanded collection/group, change icon to FolderOpen
            tree_item_icon = TreeItemIcon {
                icon: Some(IconName::FolderOpen),
                prefix: None,
                color_fn: |cx: &App| cx.theme().foreground,
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
                        this.child(Icon::new(icon_name).text_color((tree_item_icon.color_fn)(cx)))
                    })
                    .when_some(tree_item_icon.prefix, |this, prefix| {
                        this.child(
                            div()
                                .font_family(cx.theme().mono_font_family.clone())
                                .text_sm()
                                .font_bold()
                                .text_color((tree_item_icon.color_fn)(cx))
                                .pt(px(3.))
                                .child(prefix),
                        )
                    })
                    .child(Label::new(item.label.clone()).text_sm())
                    //.context_menu(self.build_context_menu(item, cx))
                    .when(entry.is_folder(), |this| {
                        this.child(if entry.is_expanded() {
                            Icon::new(IconName::ChevronDown)
                        } else {
                            Icon::new(IconName::ChevronRight)
                        })
                    }),
            )
            .on_click(window.listener_for(&self.parent, {
                let item = entry.item().clone();
                move |this: &mut CollectionsPanel, event: &ClickEvent, _window, cx| {
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
                                TreeItemKind::Group => {
                                    // Open group in new tab for editing
                                    this.open_group_tab(
                                        &metadata.collection_path,
                                        &metadata.name,
                                        cx,
                                    );
                                }
                            }
                        }
                    }
                    // Single-click is handled by the tree component for expansion
                }
            }))
    }

    fn context_menu(
        &self,
        _ix: usize,
        entry: &TreeEntry,
        menu: gpui_component::menu::PopupMenu,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui_component::menu::PopupMenu {
        let item = entry.item().clone();
        let collections_panel = self.parent.read(cx);
        let metadata = collections_panel.get_tree_item_metadata(&item.id).cloned();

        // Build context menu based on item type using metadata
        if let Some(metadata) = &metadata {
            match metadata.kind {
                TreeItemKind::Collection => menu
                    .item(
                        PopupMenuItem::new("Open Collection").on_click(window.listener_for(
                            &self.parent,
                            {
                                let collection_path = metadata.collection_path.clone();
                                let collection_name = metadata.name.clone();
                                move |this, _, _, cx| {
                                    tracing::info!("Open collection: {}", collection_name);
                                    this.open_collection_tab(&collection_path, cx);
                                }
                            },
                        )),
                    )
                    .item(
                        PopupMenuItem::new("New Request").on_click(window.listener_for(
                            &self.parent,
                            {
                                let collection_path = metadata.collection_path.clone();
                                move |this, _, _, cx| {
                                    this.new_request_tab(&collection_path, cx);
                                }
                            },
                        )),
                    )
                    .item(
                        PopupMenuItem::new("New Group").on_click(window.listener_for(
                            &self.parent,
                            {
                                let collection_path = metadata.collection_path.clone();
                                move |this, _, _, cx| {
                                    this.new_group_tab(&collection_path, cx);
                                }
                            },
                        )),
                    )
                    .separator()
                    .item(
                        PopupMenuItem::new("Remove Collection").on_click(window.listener_for(
                            &self.parent,
                            {
                                let collection_path = metadata.collection_path.clone();
                                move |this, _, window, cx| {
                                    tracing::info!("Remove collection");
                                    this.delete_collection(&collection_path, window, cx);
                                }
                            },
                        )),
                    ),
                TreeItemKind::Request => menu.item(PopupMenuItem::new("Delete Request").on_click(
                    window.listener_for(&self.parent, {
                        let collection_path = metadata.collection_path.clone();
                        let request_id_for_deletion = item.id.clone();
                        move |this, _, _, cx| {
                            this.delete_request(&request_id_for_deletion, &collection_path, cx);
                        }
                    }),
                )),
                TreeItemKind::Group => {
                    let group_name = metadata.name.clone();
                    menu.item(
                        PopupMenuItem::new("New Request").on_click(window.listener_for(
                            &self.parent,
                            {
                                let collection_path = metadata.collection_path.clone();
                                let group_path = metadata.group_path.clone().unwrap_or_default();
                                move |this, _, _, cx| {
                                    this.new_request_in_group_tab(
                                        &collection_path,
                                        &group_path,
                                        cx,
                                    );
                                }
                            },
                        )),
                    )
                    .separator()
                    .item(
                        PopupMenuItem::new("Delete Group").on_click(window.listener_for(
                            &self.parent,
                            {
                                let collection_path = metadata.collection_path.clone();
                                move |this, _, window, cx| {
                                    this.delete_group(&collection_path, &group_name, window, cx);
                                }
                            },
                        )),
                    )
                }
            }
        } else {
            // Default context menu
            menu.label(item.label.clone())
        }
    }

    // === DRAG AND DROP METHODS ===

    fn can_drag(&self, item_id: &str, _entry: &TreeEntry, cx: &App) -> bool {
        let panel = self.parent.read(cx);

        // Only requests can be dragged
        panel
            .get_tree_item_metadata(item_id)
            .is_some_and(|m| m.kind == TreeItemKind::Request)
    }

    fn create_drag_data(
        &self,
        item_id: &str,
        entry: &TreeEntry,
        cx: &App,
    ) -> Option<DraggedTreeItem> {
        let panel = self.parent.read(cx);
        let metadata = panel.get_tree_item_metadata(item_id)?;

        Some(DraggedTreeItem {
            item_id: entry.item().id.clone(),
            label: entry.item().label.clone(),
            collection_path: metadata.collection_path.clone().into(),
            icon: None,
        })
    }

    fn can_drop_on(
        &self,
        dragged_item: &DraggedTreeItem,
        target_entry: &TreeEntry,
        cx: &App,
    ) -> bool {
        let panel = self.parent.read(cx);

        // Get target metadata
        let target_metadata = panel.get_tree_item_metadata(target_entry.item().id.as_ref());

        let Some(target_metadata) = target_metadata else {
            return false;
        };

        // Must be same collection
        if target_metadata.collection_path != dragged_item.collection_path.as_ref() {
            return false;
        }

        // Can drop on Groups (to add request to group), Collection (for root level), or Requests (to insert between)
        true
    }

    fn can_drop_on_root(&self, _dragged_item: &DraggedTreeItem) -> bool {
        // Can drop to root level to move request out of any group
        // (at collection level)
        true
    }

    fn on_drop(
        &mut self,
        dragged_item: &DraggedTreeItem,
        target_entry_id: Option<&str>,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Clone the data we need before the update
        let item_id = dragged_item.item_id.clone();
        let collection_path = dragged_item.collection_path.to_string();
        let target_entry_id_owned = target_entry_id.map(|s| s.to_string());

        // Determine target group path
        let target_group_path = self.parent.update(cx, |panel, _cx| {
            target_entry_id_owned.and_then(|id| {
                panel
                    .get_tree_item_metadata(&id)
                    .and_then(|m| m.group_path.clone())
            })
        });

        // Get the request data being moved
        let request_data = self.parent.update(cx, |panel, _cx| {
            panel.request_data_map.get(item_id.as_ref()).cloned()
        });

        let Some(request_data) = request_data else {
            tracing::error!("Request data not found for item_id: {}", item_id);
            return;
        };

        // Use CollectionManager to move the request
        let result = cx.update_global(|collection_manager: &mut CollectionManager, _cx| {
            collection_manager.move_request(
                &collection_path,
                &request_data,
                target_group_path.as_deref(),
            )
        });

        if let Err(e) = result {
            tracing::error!("Failed to move request: {}", e);
            window.push_notification("Failed to move request", cx);
        } else {
            // Spawn a background task to reload the tree after the drop completes
            // This avoids borrow conflicts since the update runs asynchronously
            let parent_weak = self.parent.downgrade();
            cx.spawn(async move |cx| {
                // Small delay to ensure the drop completes first
                Timer::after(std::time::Duration::from_millis(50)).await;
                if let Some(parent) = parent_weak.upgrade() {
                    let _ = parent.update(cx, |panel, cx| {
                        panel.load_collections(cx);
                    });
                }
                Some(())
            })
            .detach();
        }
    }

    fn get_drag_icon(&self, item_id: &str, cx: &App) -> Option<DragIcon> {
        let panel = self.parent.read(cx);
        panel
            .get_tree_item_metadata(item_id)
            .map(|metadata| DragIcon {
                prefix: None,
                color_fn: match metadata.kind {
                    TreeItemKind::Request => |cx: &App| cx.theme().foreground,
                    _ => |cx: &App| cx.theme().muted_foreground,
                },
            })
    }
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
        let parent_entity = cx.entity();
        let tree_state = cx.new(|cx| {
            DraggableTreeState::new(
                CollectionsTreeDelegate {
                    parent: parent_entity,
                },
                cx,
            )
        });

        Self {
            collections: Vec::new(),
            tree_state,
            tree_item_metadata: HashMap::new(),
            request_data_map: HashMap::new(),
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
        // Collect IDs of expanded items before clearing
        let expanded_item_ids: std::collections::HashSet<SharedString> = self
            .tree_state
            .read(cx)
            .entries()
            .iter()
            .filter(|entry| entry.is_expanded())
            .map(|entry| entry.item().id.clone())
            .collect();

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
                let request_tree_item = TreeItem::new(request_id.clone(), request.name.clone())
                    .children(vec![])
                    .expanded(expanded_item_ids.contains(&SharedString::from(request_id.clone())));

                let request_metadata = TreeItemMetadata {
                    name: request.name.clone(),
                    kind: TreeItemKind::Request,
                    icon: TreeItemIcon {
                        icon: None,
                        prefix: Some(SharedString::from(request.method.as_str())),
                        color_fn: request.method.get_color_fn(),
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
            sorted_groups.sort_by(|a, b| a.0.cmp(b.0));

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
                            color_fn: request.method.get_color_fn(),
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
                let group_tree_item = TreeItem::new(group_id.clone(), group_name.clone())
                    .children(group_child_items)
                    .expanded(expanded_item_ids.contains(&SharedString::from(group_id.clone())));

                // Store metadata for this group
                let group_metadata = TreeItemMetadata {
                    name: group_name.clone(),
                    kind: TreeItemKind::Group,
                    icon: TreeItemIcon {
                        icon: Some(IconName::Folder),
                        prefix: None,
                        color_fn: |cx: &App| cx.theme().magenta,
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
                    .children(child_items)
                    .expanded(
                        expanded_item_ids.contains(&SharedString::from(collection_id.clone())),
                    );

            // Store metadata for this collection
            let collection_metadata = TreeItemMetadata {
                name: collection_data.name.clone(),
                kind: TreeItemKind::Collection,
                icon: TreeItemIcon {
                    icon: Some(IconName::Folder),
                    prefix: None,
                    color_fn: |cx: &App| cx.theme().blue,
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

    /// Open a group in a new tab for editing
    fn open_group_tab(&mut self, collection_path: &str, group_name: &str, cx: &mut Context<Self>) {
        tracing::info!(
            "Opening group tab for collection_path: {}, group_name: {}",
            collection_path,
            group_name
        );

        // Emit CreateNewGroupTab event with group name
        cx.emit(AppEvent::CreateNewGroupTab {
            collection_path: collection_path.to_string().into(),
            group_name: Some(group_name.to_string().into()),
        });
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

    /// Create a new group tab
    fn new_group_tab(&mut self, collection_path: &str, cx: &mut Context<Self>) {
        tracing::info!(
            "Creating new group tab for collection_path: {}",
            collection_path
        );

        // Create a new group editor in a new tab
        let collection_path_for_editor = collection_path.to_string();

        cx.emit(AppEvent::CreateNewGroupTab {
            collection_path: collection_path_for_editor.into(),
            group_name: None, // No existing group name for new groups
        });
    }

    /// Delete a group from CollectionManager
    fn delete_group(
        &mut self,
        collection_path: &str,
        group_name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        tracing::info!(
            "Deleting group '{}' from collection: {}",
            group_name,
            collection_path
        );

        // Use CollectionManager to delete the group
        let result = cx.update_global(|collection_manager: &mut CollectionManager, _cx| {
            collection_manager.delete_group(collection_path, group_name)
        });

        match result {
            Ok(_) => {
                tracing::info!("Successfully deleted group '{}'", group_name);
                window.push_notification(
                    (NotificationType::Success, "Group deleted successfully"),
                    cx,
                );

                // Reload collections to rebuild the tree
                self.load_collections(cx);
            }
            Err(e) => {
                tracing::error!("Failed to delete group: {}", e);
                window.push_notification((NotificationType::Error, "Failed to delete group"), cx);
            }
        }

        // Emit event to notify other parts of the app
        cx.emit(AppEvent::GroupDeleted {
            collection_path: collection_path.to_string().into(),
            group_name: group_name.to_string().into(),
        });
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
}

impl Render for CollectionsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        v_flex()
            .size_full()
            .gap_2()
            .bg(cx.theme().sidebar_primary_foreground)
            .px(px(1.))
            .child(self.render_header_section(cx))
            .child(DraggableTree::new(&self.tree_state))
    }
}

impl EventEmitter<AppEvent> for CollectionsPanel {}
