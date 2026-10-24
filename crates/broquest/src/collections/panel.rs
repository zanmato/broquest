use gpui::SharedString;
use gpui::{
    App, AppContext, ClickEvent, Context, Entity, EventEmitter, Focusable as _, InteractiveElement,
    IntoElement, ParentElement, Render, StatefulInteractiveElement, Styled, Subscription, Window,
    actions, div, prelude::FluentBuilder, px,
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, Sizable, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    input::{Input, InputEvent, InputState},
    list::ListItem,
    menu::PopupMenuItem,
    notification::NotificationType,
    tooltip::Tooltip,
    v_flex,
};
use smol::Timer;
use std::collections::{HashMap, HashSet};

use crate::app_database::{AppDatabase, CollectionData};
use crate::app_events::AppEvent;
use crate::domain::{HttpMethod, RequestData};
use crate::ui::icon::IconName;
use crate::ui::method_badge::MethodBadge;

use super::format::CollectionFormat;
use super::manager::{CollectionInfo, CollectionManager};
use crate::ui::draggable_tree::{
    DragIcon, DraggableTree, DraggableTreeDelegate, DraggableTreeState, DraggedTreeItem,
    InsertPosition, TreeEntry, TreeItem,
};

actions!(collections_panel, [RenameItem, DuplicateItem]);

/// The key context of the collections tree, where its shortcuts apply.
pub(crate) const TREE_CONTEXT: &str = crate::ui::draggable_tree::CONTEXT;

/// How far the rename field's text sits inside its box: an extra-small
/// input's padding and its border.
const RENAME_FIELD_INSET: gpui::Pixels = px(5.);

/// A request or group being renamed in place.
#[derive(Clone, Debug, PartialEq)]
pub struct RenameTarget {
    pub kind: TreeItemKind,
    pub collection_path: String,
    /// The group's path for a group, or the path of the group a request is
    /// in.
    pub group_path: Option<String>,
    pub name: String,
}

impl RenameTarget {
    fn of(metadata: &TreeItemMetadata) -> Self {
        Self {
            kind: metadata.kind,
            collection_path: metadata.collection_path.clone(),
            group_path: metadata.group_path.clone(),
            name: metadata.name.clone(),
        }
    }
}

/// The name field shown in place of an item's label while it is renamed.
struct Renaming {
    target: RenameTarget,
    input: Entity<InputState>,
    _subscriptions: Vec<Subscription>,
}

/// Icon and color combination for tree items
#[derive(Clone)]
pub struct TreeItemIcon {
    pub icon: Option<IconName>,
    pub method: Option<HttpMethod>,
    pub color_fn: fn(&App) -> gpui::Hsla,
}

pub struct CollectionsPanel {
    collections: Vec<CollectionData>,
    tree_state: Entity<DraggableTreeState<CollectionsTreeDelegate>>,
    tree_item_metadata: HashMap<String, TreeItemMetadata>, // Map hierarchical key -> metadata
    request_data_map: HashMap<String, RequestData>,        // Map item_id -> RequestData
    /// Items to expand on the next tree build, from a restored session.
    restored_expanded_ids: HashSet<SharedString>,
    /// A collection to reveal once the tree holds it.
    pending_reveal: Option<String>,
    renaming: Option<Renaming>,
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
                method: None,
                color_fn: |cx: &App| cx.theme().foreground,
            });

        // Update folder icons based on expansion state (check if it's a collection or group by looking at metadata)
        if let Some(metadata) = connections_panel.get_tree_item_metadata(&item.id)
            && (metadata.kind == TreeItemKind::Collection || metadata.kind == TreeItemKind::Group)
            && entry.is_expanded()
        {
            // This is an expanded collection/group, change icon to FolderOpen but preserve the original color
            tree_item_icon = TreeItemIcon {
                icon: Some(IconName::FolderOpen),
                method: None,
                color_fn: metadata.icon.color_fn,
            };
        }

        let format = connections_panel
            .get_tree_item_metadata(&item.id)
            .and_then(|metadata| metadata.format);
        let rename_input = connections_panel.rename_input(&item.id);

        ListItem::new(ix)
            .selected(selected)
            .w_full()
            .px_3()
            .pl(px(12.) * depth as f32 + px(12.)) // Indent based on depth
            .my_0p5()
            // `ListItem` copies its corner radii onto the selection highlight,
            // so this rounds both the fill and its border.
            .rounded(cx.theme().radius)
            .child(
                h_flex()
                    .id(("tree-item", ix))
                    .gap_2()
                    .items_center()
                    .when_some(tree_item_icon.icon, |this, icon_name| {
                        this.child(Icon::new(icon_name).text_color((tree_item_icon.color_fn)(cx)))
                    })
                    .when_some(tree_item_icon.method, |this, method| {
                        this.child(MethodBadge::new(method).fixed_width())
                    })
                    .child(match rename_input {
                        // The field is laid over the hidden label, so the row
                        // keeps its height and nothing in it moves. It is taller
                        // than the label and overflows it evenly, and it starts
                        // its padding and border to the left so the name stays
                        // where it was.
                        Some(input) => div()
                            .flex_1()
                            .min_w_0()
                            .relative()
                            .child(
                                div()
                                    .text_sm()
                                    .truncate()
                                    .invisible()
                                    .child(item.label.clone()),
                            )
                            .child(
                                h_flex()
                                    .absolute()
                                    .top_0()
                                    .bottom_0()
                                    .left(-RENAME_FIELD_INSET)
                                    .right_0()
                                    .items_center()
                                    .child(Input::new(&input).xsmall()),
                            )
                            .into_any_element(),
                        None => div()
                            .flex_1()
                            .text_sm()
                            // Never wraps: text less than a pixel too wide is
                            // not truncated, and would wrap its last
                            // character onto a second line instead.
                            .truncate()
                            .child(item.label.clone())
                            .into_any_element(),
                    })
                    .when_some(
                        format.and_then(|format| Some((format, format.tag()?))),
                        |this, (format, tag)| {
                            this.child(
                                div()
                                    .id(("format-tag", ix))
                                    .flex_none()
                                    .px(px(4.))
                                    .py(px(1.))
                                    .rounded(px(6.))
                                    .bg(cx.theme().muted_foreground.opacity(0.15))
                                    .text_color(cx.theme().muted_foreground)
                                    .text_size(px(10.))
                                    .line_height(px(13.))
                                    .font_bold()
                                    .child(tag)
                                    .tooltip(move |window, cx| {
                                        Tooltip::new(format!(
                                            "{}. {}",
                                            format.label(),
                                            format.description()
                                        ))
                                        .build(window, cx)
                                    }),
                            )
                        },
                    )
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
                    // Clicks in the name field place the caret, and open nothing.
                    if this.rename_input(&item.id).is_some() {
                        return;
                    }
                    if event.click_count() == 2 {
                        // Double-click, open in tab
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
                                            group_path: metadata.group_path.clone().map(Into::into),
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
                }
            }))
    }

    fn context_menu(
        &self,
        _ix: usize,
        entry: &TreeEntry,
        menu: gpui_kit::component::menu::PopupMenu,
        window: &mut Window,
        cx: &mut App,
    ) -> gpui_kit::component::menu::PopupMenu {
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
                                move |this, _, window, cx| {
                                    this.new_request(&collection_path, None, window, cx);
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
                TreeItemKind::Request => menu
                    .item(rename_item(&self.parent, metadata, window))
                    .item(
                        PopupMenuItem::new("Duplicate").on_click(window.listener_for(
                            &self.parent,
                            {
                                let request_id_to_duplicate = item.id.clone();
                                move |this, _, window, cx| {
                                    this.duplicate_request(&request_id_to_duplicate, window, cx);
                                }
                            },
                        )),
                    )
                    .separator()
                    .item(
                        PopupMenuItem::new("Delete Request").on_click(window.listener_for(
                            &self.parent,
                            {
                                let collection_path = metadata.collection_path.clone();
                                let request_id_for_deletion = item.id.clone();
                                move |this, _, _, cx| {
                                    this.delete_request(
                                        &request_id_for_deletion,
                                        &collection_path,
                                        cx,
                                    );
                                }
                            },
                        )),
                    ),
                TreeItemKind::Group => {
                    let group_name = metadata.name.clone();
                    menu.item(
                        PopupMenuItem::new("New Request").on_click(window.listener_for(
                            &self.parent,
                            {
                                let collection_path = metadata.collection_path.clone();
                                let group_path = metadata.group_path.clone();
                                move |this, _, window, cx| {
                                    this.new_request(
                                        &collection_path,
                                        group_path.as_deref(),
                                        window,
                                        cx,
                                    );
                                }
                            },
                        )),
                    )
                    .item(rename_item(&self.parent, metadata, window))
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
        position: InsertPosition,
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

        // Can drop on Groups (to add request to group), Collection (for root
        // level), or Requests (to join that request's group).
        //
        // Requests carry no order of their own, so the edge bands on a group
        // row would promise a reordering that cannot be honoured. Only accept
        // them on requests, where they are the only positions available and
        // still mean "move into this request's group".
        match position {
            InsertPosition::Inside => true,
            InsertPosition::Before | InsertPosition::After => {
                target_metadata.kind == TreeItemKind::Request
            }
        }
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
        _position: InsertPosition,
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
        let manager = CollectionManager::global(cx);
        let result = manager.update(cx, |collection_manager, cx| {
            collection_manager.move_request(
                &collection_path,
                &request_data,
                target_group_path.as_deref(),
                cx,
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
                    parent.update(cx, |panel, cx| {
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
    /// The format of a collection item.
    pub format: Option<CollectionFormat>,
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

        // Pass tree changes, such as a folder expanding, on to this panel's
        // observers so the session is saved.
        cx.observe(&tree_state, |_, _, cx| cx.notify()).detach();

        Self {
            collections: Vec::new(),
            tree_state,
            tree_item_metadata: HashMap::new(),
            request_data_map: HashMap::new(),
            restored_expanded_ids: HashSet::new(),
            pending_reveal: None,
            renaming: None,
        }
    }

    pub fn refresh_collections(&mut self, cx: &mut Context<Self>) {
        // Load collections; the manager emits CollectionsChanged, which the app
        // subscription turns into a Self::load_collections refresh.
        let manager = CollectionManager::global(cx);
        manager.update(cx, |collection_manager, cx| {
            if let Err(e) = collection_manager.load_saved(cx) {
                tracing::error!("Failed to load collections from database: {}", e);
            }
        });
    }

    /// Ids of the expanded collections and groups.
    pub fn expanded_ids(&self, cx: &App) -> Vec<String> {
        self.tree_state
            .read(cx)
            .entries()
            .iter()
            .filter(|entry| entry.is_expanded())
            .filter(|entry| {
                self.get_tree_item_metadata(&entry.item().id)
                    .is_some_and(|metadata| metadata.kind != TreeItemKind::Request)
            })
            .map(|entry| entry.item().id.to_string())
            .collect()
    }

    /// Expand, select and scroll to the collection at `path`, now or as soon
    /// as the tree holds it.
    pub fn reveal_collection(&mut self, path: &str, cx: &mut Context<Self>) {
        self.pending_reveal = Some(path.to_string());
        self.reveal_pending(cx);
    }

    fn reveal_pending(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.pending_reveal.clone() else {
            return;
        };
        let id = format!("collection_{}", path.replace('/', "_"));
        if self.tree_state.update(cx, |tree, cx| tree.reveal(&id, cx)) {
            self.pending_reveal = None;
        }
    }

    /// Expand the given items, typically from a restored session.
    pub fn restore_expanded(&mut self, ids: &[String], cx: &mut Context<Self>) {
        self.restored_expanded_ids = ids.iter().cloned().map(SharedString::from).collect();
        self.load_collections(cx);
    }

    pub fn load_collections(&mut self, cx: &mut Context<Self>) {
        // Get collections from the global CollectionManager (owned clones so the
        // manager borrow doesn't outlive the subsequent mutations).
        let manager = CollectionManager::global(cx);
        let mut collection_infos_owned: Vec<CollectionInfo> = manager
            .read(cx)
            .get_all_collections()
            .into_iter()
            .cloned()
            .collect();

        // Sort collections alphabetically by name
        collection_infos_owned.sort_by(|a, b| a.data.name.cmp(&b.data.name));

        // Convert CollectionInfo to CollectionData for the tree
        self.collections = collection_infos_owned
            .iter()
            .map(|info| info.data.clone())
            .collect();

        tracing::info!(
            "Loaded {} collections from global manager",
            self.collections.len()
        );

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
        let mut expanded_item_ids: std::collections::HashSet<SharedString> = self
            .tree_state
            .read(cx)
            .entries()
            .iter()
            .filter(|entry| entry.is_expanded())
            .map(|entry| entry.item().id.clone())
            .collect();
        expanded_item_ids.extend(self.restored_expanded_ids.drain());

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

            // Add direct collection requests (root level)
            let mut sorted_requests: Vec<_> = collection_info.requests.iter().collect();
            sorted_requests.sort_by_key(|(_, request)| request.name.to_lowercase());

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
                        method: Some(request.method),
                        color_fn: |cx: &App| cx.theme().foreground,
                    },
                    collection_path: collection_data.path.clone(),
                    group_path: None, // Root level request
                    format: None,
                };

                self.tree_item_metadata
                    .insert(request_id.clone(), request_metadata);
                self.request_data_map.insert(request_id, request.clone());
                child_items.push(request_tree_item);
            }

            // Add group folders and their requests
            let mut sorted_groups: Vec<_> = collection_info.groups.iter().collect();
            sorted_groups.sort_by_key(|(name, _)| name.to_lowercase());

            for (group_name, group_info) in sorted_groups {
                let group_id =
                    format!("group_{}_{}", collection_path.replace('/', "_"), group_name);

                // Build child items for group requests
                let mut sorted_group_requests: Vec<_> = group_info.requests.iter().collect();
                sorted_group_requests.sort_by_key(|(_, request)| request.name.to_lowercase());

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
                            method: Some(request.method),
                            color_fn: |cx: &App| cx.theme().foreground,
                        },
                        collection_path: collection_data.path.clone(),
                        group_path: Some(group_info.path.clone()),
                        format: None,
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
                        method: None,
                        color_fn: |cx: &App| cx.theme().magenta,
                    },
                    collection_path: collection_data.path.clone(),
                    group_path: Some(group_info.path.clone()),
                    format: None,
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
                    method: None,
                    color_fn: |cx: &App| cx.theme().blue,
                },
                collection_path: collection_data.path.clone(),
                group_path: None,
                format: Some(collection_info.format),
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
        self.reveal_pending(cx);
    }

    /// Expand the whole tree and select the request named `name`.
    #[cfg(feature = "screenshots")]
    pub(crate) fn reveal_request(&mut self, name: &str, cx: &mut Context<Self>) {
        self.tree_state.update(cx, |state, cx| {
            state.expand_all(cx);
            let ix = state
                .entries()
                .iter()
                .position(|entry| !entry.is_folder() && entry.item().label.as_ref() == name);
            state.set_selected_index(ix, cx);
        });
    }

    /// Open a collection in a new tab
    fn open_collection_tab(&mut self, collection_path: &str, cx: &mut Context<Self>) {
        tracing::info!("Opening collection tab for path: {}", collection_path);

        // Use the in-memory cache (works for both native TOML and OpenCollection
        // collections, which have no collection.toml on disk). Fall back to
        // reading collection.toml only if the collection isn't cached yet.
        let manager = CollectionManager::global(cx);
        let collection_manager = manager.read(cx);
        let collection_data = collection_manager
            .get_collection_by_path(collection_path)
            .map(|info| info.toml.clone())
            .or_else(|| {
                collection_manager
                    .read_collection_toml(std::path::Path::new(collection_path))
                    .ok()
            });

        match collection_data {
            Some(collection_data) => {
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
            None => {
                tracing::error!(
                    "Failed to load collection data from {}: collection not cached and no collection.toml on disk",
                    collection_path,
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

    /// Add a request named "New Request" (or "New Request 2" and so on) to
    /// the collection or group, open it, and start renaming it.
    fn new_request(
        &mut self,
        collection_path: &str,
        group_path: Option<&str>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let manager = CollectionManager::global(cx);
        let name = manager
            .read(cx)
            .unique_request_name(collection_path, group_path, "New Request");
        let request = RequestData {
            name: name.clone(),
            ..Default::default()
        };
        if !self.save_new_request(collection_path, group_path, &request, window, cx) {
            return;
        }
        let Some((group_path, request)) =
            self.locate_request(collection_path, group_path, &name, cx)
        else {
            return;
        };
        cx.emit(AppEvent::CreateNewRequestTab {
            request_data: request,
            collection_path: collection_path.to_string().into(),
            group_path: group_path.clone().map(Into::into),
        });
        self.rename_after_menu(
            RenameTarget {
                kind: TreeItemKind::Request,
                collection_path: collection_path.to_string(),
                group_path,
                name,
            },
            window,
            cx,
        );
    }

    /// Save a copy of a request next to it, named "<name> copy", and start
    /// renaming the copy.
    fn duplicate_request(&mut self, request_id: &str, window: &mut Window, cx: &mut Context<Self>) {
        let Some(metadata) = self.get_tree_item_metadata(request_id).cloned() else {
            tracing::error!("Could not find metadata for ID: {}", request_id);
            return;
        };
        let Some(mut copy) = self.request_data_map.get(request_id).cloned() else {
            tracing::error!("Could not find request data for ID: {}", request_id);
            return;
        };
        let group_path = metadata.group_path.as_deref();
        copy.name = CollectionManager::global(cx).read(cx).unique_request_name(
            &metadata.collection_path,
            group_path,
            &format!("{} copy", copy.name),
        );
        // A duplicate is a new request in its .http file, not the original.
        if let Some(options) = copy.http_file.as_mut() {
            options.source = None;
        }
        if !self.save_new_request(&metadata.collection_path, group_path, &copy, window, cx) {
            return;
        }
        let group_path = self
            .locate_request(&metadata.collection_path, group_path, &copy.name, cx)
            .and_then(|(group_path, _)| group_path);
        self.rename_after_menu(
            RenameTarget {
                kind: TreeItemKind::Request,
                collection_path: metadata.collection_path.clone(),
                group_path,
                name: copy.name,
            },
            window,
            cx,
        );
    }

    /// Write a request that doesn't exist yet. Returns whether it was saved.
    fn save_new_request(
        &mut self,
        collection_path: &str,
        group_path: Option<&str>,
        request: &RequestData,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let result = CollectionManager::global(cx).update(cx, |manager, cx| {
            manager.save_request(collection_path, request, &request.name, group_path, cx)
        });
        match result {
            Ok(()) => {
                self.load_collections(cx);
                true
            }
            Err(error) => {
                window.push_notification(
                    (
                        NotificationType::Error,
                        SharedString::from(error.to_string()),
                    ),
                    cx,
                );
                false
            }
        }
    }

    /// Where the request called `name` was stored, looked for in `group_path`
    /// first. A request saved to a collection of .http files goes into a
    /// file, which is its group.
    fn locate_request(
        &self,
        collection_path: &str,
        group_path: Option<&str>,
        name: &str,
        cx: &App,
    ) -> Option<(Option<String>, RequestData)> {
        let manager = CollectionManager::global(cx);
        let manager = manager.read(cx);
        if let Some(request) = manager.find_request(collection_path, group_path, name) {
            return Some((group_path.map(str::to_string), request.clone()));
        }
        let info = manager.get_collection_by_path(collection_path)?;
        info.groups.values().find_map(|group| {
            group
                .requests
                .values()
                .find(|request| request.name == name)
                .map(|request| (Some(group.path.clone()), request.clone()))
        })
    }

    /// Start renaming once the menu that asked for it has closed, since
    /// closing it gives focus back to where it was.
    fn rename_after_menu(
        &mut self,
        target: RenameTarget,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        cx.defer_in(window, move |this, window, cx| {
            this.start_rename(target, window, cx);
        });
    }

    /// Rename the request called `name` in place, revealing it first.
    pub fn rename_request(
        &mut self,
        collection_path: &str,
        group_path: Option<&str>,
        name: &str,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.rename_after_menu(
            RenameTarget {
                kind: TreeItemKind::Request,
                collection_path: collection_path.to_string(),
                group_path: group_path.map(str::to_string),
                name: name.to_string(),
            },
            window,
            cx,
        );
    }

    /// Rename the selected request or group.
    fn rename_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let target = self
            .tree_state
            .read(cx)
            .selected_entry()
            .and_then(|entry| self.get_tree_item_metadata(&entry.item().id))
            .map(RenameTarget::of);
        if let Some(target) = target {
            self.start_rename(target, window, cx);
        }
    }

    /// Duplicate the selected request.
    fn duplicate_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let id = self
            .tree_state
            .read(cx)
            .selected_entry()
            .map(|entry| entry.item().id.to_string())
            .filter(|id| {
                self.get_tree_item_metadata(id)
                    .is_some_and(|metadata| metadata.kind == TreeItemKind::Request)
            });
        if let Some(id) = id {
            self.duplicate_request(&id, window, cx);
        }
    }

    /// The tree id of the item `target` names.
    fn item_id(&self, target: &RenameTarget) -> Option<String> {
        self.tree_item_metadata
            .iter()
            .find(|(_, metadata)| RenameTarget::of(metadata) == *target)
            .map(|(id, _)| id.clone())
    }

    /// The name field of the item with `id`, while it is being renamed.
    fn rename_input(&self, id: &str) -> Option<Entity<InputState>> {
        let renaming = self.renaming.as_ref()?;
        let metadata = self.get_tree_item_metadata(id)?;
        (RenameTarget::of(metadata) == renaming.target).then(|| renaming.input.clone())
    }

    /// Show a name field in place of the item's label. Enter or leaving the
    /// field renames it, and Escape leaves it as it was.
    fn start_rename(&mut self, target: RenameTarget, window: &mut Window, cx: &mut Context<Self>) {
        if target.kind == TreeItemKind::Collection {
            return;
        }
        let Some(id) = self.item_id(&target) else {
            return;
        };
        self.tree_state
            .update(cx, |tree, cx| tree.select_item(&id, cx));

        let input = cx.new(|cx| InputState::new(window, cx).default_value(target.name.clone()));
        input.update(cx, |input, cx| {
            input.focus(window, cx);
            input.select_all(window, cx);
        });
        let events =
            cx.subscribe_in(
                &input,
                window,
                |this, _, event: &InputEvent, window, cx| match event {
                    InputEvent::PressEnter { .. } => this.commit_rename(true, window, cx),
                    InputEvent::Blur => this.commit_rename(false, window, cx),
                    InputEvent::Change | InputEvent::Focus => {}
                },
            );
        // Escape belongs to the field itself, so it is caught before the
        // field handles it.
        let panel = cx.entity().downgrade();
        let escape = cx.intercept_keystrokes(move |event, window, cx| {
            if event.keystroke.key != "escape" || event.keystroke.modifiers.modified() {
                return;
            }
            let _ = panel.update(cx, |panel, cx| {
                let focused = panel
                    .renaming
                    .as_ref()
                    .is_some_and(|renaming| renaming.input.focus_handle(cx).is_focused(window));
                if focused {
                    cx.stop_propagation();
                    panel.cancel_rename(window, cx);
                }
            });
        });
        self.renaming = Some(Renaming {
            target,
            input,
            _subscriptions: vec![events, escape],
        });
        cx.notify();
    }

    fn focus_tree(&self, window: &mut Window, cx: &mut Context<Self>) {
        self.tree_state
            .update(cx, |tree, cx| tree.focus(window, cx));
    }

    fn cancel_rename(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.renaming = None;
        self.focus_tree(window, cx);
        cx.notify();
    }

    /// Rename the item to what the name field holds. A name that can't be
    /// used is reported, and keeps the field open after Enter.
    fn commit_rename(&mut self, from_enter: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(renaming) = self.renaming.take() else {
            return;
        };
        let target = renaming.target.clone();
        let new_name = renaming.input.read(cx).value().trim().to_string();
        let was_expanded = self.item_id(&target).is_some_and(|id| {
            self.tree_state
                .read(cx)
                .entries()
                .iter()
                .any(|entry| entry.item().id.as_ref() == id && entry.is_expanded())
        });
        if new_name.is_empty() || new_name == target.name {
            if from_enter {
                self.focus_tree(window, cx);
            }
            cx.notify();
            return;
        }

        let result = CollectionManager::global(cx).update(cx, |manager, cx| match target.kind {
            TreeItemKind::Request => manager.rename_request(
                &target.collection_path,
                target.group_path.as_deref(),
                &target.name,
                &new_name,
                cx,
            ),
            TreeItemKind::Group => {
                manager.rename_group(&target.collection_path, &target.name, &new_name, cx)
            }
            TreeItemKind::Collection => Ok(()),
        });

        match result {
            Ok(()) => {
                self.load_collections(cx);
                // Keep the renamed item selected. A group's path follows its
                // name, so it is found by name.
                let id = self
                    .tree_item_metadata
                    .iter()
                    .find(|(_, metadata)| {
                        metadata.kind == target.kind
                            && metadata.collection_path == target.collection_path
                            && metadata.name == new_name
                            && (target.kind == TreeItemKind::Group
                                || metadata.group_path == target.group_path)
                    })
                    .map(|(id, _)| id.clone());
                if let Some(id) = id {
                    self.tree_state.update(cx, |tree, cx| {
                        tree.select_item(&id, cx);
                        // A renamed folder stays open if it was.
                        if was_expanded {
                            tree.reveal(&id, cx);
                        }
                    });
                }
                if from_enter {
                    self.focus_tree(window, cx);
                }
            }
            Err(error) => {
                window.push_notification(
                    (
                        NotificationType::Error,
                        SharedString::from(error.to_string()),
                    ),
                    cx,
                );
                if from_enter {
                    renaming
                        .input
                        .update(cx, |input, cx| input.focus(window, cx));
                    self.renaming = Some(renaming);
                }
            }
        }
        cx.notify();
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
        let manager = CollectionManager::global(cx);
        manager.update(cx, |collection_manager, cx| {
            collection_manager.remove_collection(collection_path, cx);
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
        let manager = CollectionManager::global(cx);
        manager.update(cx, |collection_manager, cx| {
            match collection_manager.delete_request(collection_path, &request_data, cx) {
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
            }
        });

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
        let manager = CollectionManager::global(cx);
        let result = manager.update(cx, |collection_manager, cx| {
            collection_manager.delete_group(collection_path, group_name, cx)
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

    fn render_footer_section(&self, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .justify_end()
            .items_center()
            .px(px(4.))
            .py(px(4.))
            .border_t_1()
            .border_color(cx.theme().border)
            .child(
                Button::new("refresh_collections")
                    .xsmall()
                    .ghost()
                    .icon(Icon::new(IconName::Refresh).size(px(14.)))
                    .on_click(cx.listener(|this, _, _, cx| {
                        tracing::info!("Refresh collections clicked");
                        this.refresh_collections(cx);
                    })),
            )
    }
}

impl Render for CollectionsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // No background of its own: the sidebar card in `BroquestApp::render`
        // owns the surface colour so the rounded corners stay clean.
        v_flex()
            .size_full()
            .on_action(cx.listener(|this, _: &RenameItem, window, cx| {
                this.rename_selected(window, cx);
            }))
            .on_action(cx.listener(|this, _: &DuplicateItem, window, cx| {
                this.duplicate_selected(window, cx);
            }))
            // Keeps the rounded row selection off the sidebar card's border.
            .px(px(4.))
            .child(
                div()
                    .flex_1()
                    .min_h_0()
                    .child(DraggableTree::new(&self.tree_state)),
            )
            .child(self.render_footer_section(cx))
    }
}

impl EventEmitter<AppEvent> for CollectionsPanel {}

/// The context menu item that renames the item `metadata` describes.
fn rename_item(
    panel: &Entity<CollectionsPanel>,
    metadata: &TreeItemMetadata,
    window: &mut Window,
) -> PopupMenuItem {
    let target = RenameTarget::of(metadata);
    PopupMenuItem::new("Rename").on_click(window.listener_for(panel, move |this, _, window, cx| {
        this.rename_after_menu(target.clone(), window, cx);
    }))
}
