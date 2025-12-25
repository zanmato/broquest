//! Draggable tree component with drag and drop support.
//!
//! This is an alternative to the `tree` module that replaces the virtualized
//! `uniform_list` with a regular scroll container to enable proper drag and
//! drop event handling.

#![allow(dead_code)]

use std::{cell::RefCell, rc::Rc};

use gpui::{
    App, AppContext, Context, DragMoveEvent, ElementId, Entity, FocusHandle,
    InteractiveElement as _, IntoElement, KeyBinding, MouseButton, ParentElement, Pixels, Point,
    Render, RenderOnce, SharedString, StatefulInteractiveElement, StyleRefinement, Styled, Task,
    Window, div, prelude::FluentBuilder, px,
};

use gpui_component::{
    ActiveTheme, StyledExt, h_flex,
    list::ListItem,
    menu::{ContextMenuExt, PopupMenu},
    scroll::ScrollableElement,
};

use crate::ui::actions::{Confirm, SelectDown, SelectLeft, SelectRight, SelectUp};

const CONTEXT: &str = "BroDraggableTree";

struct TreeItemState {
    expanded: bool,
    disabled: bool,
}

/// A tree item with a label, children, and an expanded state.
#[derive(Clone)]
pub struct TreeItem {
    pub id: SharedString,
    pub label: SharedString,
    pub children: Vec<TreeItem>,
    state: Rc<RefCell<TreeItemState>>,
}

/// A flat representation of a tree item with its depth.
#[derive(Clone)]
pub struct TreeEntry {
    item: TreeItem,
    depth: usize,
}

impl TreeEntry {
    /// Create a new TreeEntry with the given item and depth.
    pub fn new(item: TreeItem, depth: usize) -> Self {
        Self { item, depth }
    }

    /// Get the source tree item.
    #[inline]
    pub fn item(&self) -> &TreeItem {
        &self.item
    }

    /// The depth of this item in the tree.
    #[inline]
    pub fn depth(&self) -> usize {
        self.depth
    }

    /// Return true if this entry is at the root level (depth 0).
    #[inline]
    pub fn is_root(&self) -> bool {
        self.depth == 0
    }

    /// Whether this item is a folder (has children).
    #[inline]
    pub fn is_folder(&self) -> bool {
        self.item.is_folder()
    }

    /// Return true if the item is expanded.
    #[inline]
    pub fn is_expanded(&self) -> bool {
        self.item.is_expanded()
    }

    #[inline]
    pub fn is_disabled(&self) -> bool {
        self.item.is_disabled()
    }
}

impl TreeItem {
    /// Create a new tree item with the given label.
    ///
    /// - The `id` for you to uniquely identify this item, then later you can use it for selection or other purposes.
    /// - The `label` is the text to display for this item.
    ///
    /// For example, the `id` is the full file path, and the `label` is the file name.
    ///
    /// ```ignore
    /// TreeItem::new("src/ui/button.rs", "button.rs")
    /// ```
    pub fn new(id: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            children: Vec::new(),
            state: Rc::new(RefCell::new(TreeItemState {
                expanded: false,
                disabled: false,
            })),
        }
    }

    /// Add a child item to this tree item.
    pub fn child(mut self, child: TreeItem) -> Self {
        self.children.push(child);
        self
    }

    /// Add multiple child items to this tree item.
    pub fn children(mut self, children: impl IntoIterator<Item = TreeItem>) -> Self {
        self.children.extend(children);
        self
    }

    /// Set expanded state for this tree item.
    pub fn expanded(self, expanded: bool) -> Self {
        self.state.borrow_mut().expanded = expanded;
        self
    }

    /// Set disabled state for this tree item.
    pub fn disabled(self, disabled: bool) -> Self {
        self.state.borrow_mut().disabled = disabled;
        self
    }

    /// Whether this item is a folder (has children).
    #[inline]
    pub fn is_folder(&self) -> bool {
        !self.children.is_empty()
    }

    /// Return true if the item is disabled.
    pub fn is_disabled(&self) -> bool {
        self.state.borrow().disabled
    }

    /// Return true if the item is expanded.
    #[inline]
    pub fn is_expanded(&self) -> bool {
        self.state.borrow().expanded
    }

    /// Toggle the expanded state of this tree item.
    pub fn toggle_expanded(&self) {
        let mut state = self.state.borrow_mut();
        state.expanded = !state.expanded;
    }
}

pub fn init(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", SelectUp, Some(CONTEXT)),
        KeyBinding::new("down", SelectDown, Some(CONTEXT)),
        KeyBinding::new("left", SelectLeft, Some(CONTEXT)),
        KeyBinding::new("right", SelectRight, Some(CONTEXT)),
    ]);
}

/// Icon information for drag visuals.
#[derive(Clone, Debug)]
pub struct DragIcon {
    /// Optional prefix text (e.g., "GET", "POST" for HTTP requests)
    pub prefix: Option<SharedString>,
    /// Color function for the icon/prefix
    pub color_fn: fn(&App) -> gpui::Hsla,
}

/// The data type passed during drag operations for tree items.
#[derive(Clone, Debug)]
pub struct DraggedTreeItem {
    /// The unique ID of the item being dragged
    pub item_id: SharedString,
    /// The label of the item (for display in drag visual)
    pub label: SharedString,
    /// The collection this item belongs to (for validation)
    pub collection_path: SharedString,
    /// Optional icon information for the drag visual
    pub icon: Option<DragIcon>,
}

/// Tracks what is currently being hovered during a drag operation.
enum DragTarget {
    /// Dragging over a specific entry
    Entry {
        /// The entry currently under the mouse cursor
        entry_id: SharedString,
        /// The entry that should be highlighted
        highlight_entry_id: SharedString,
        /// Where to insert relative to the entry
        position: InsertPosition,
    },
    /// Dragging on the background (root level drop zone)
    Background,
}

/// Where to insert the dragged item relative to a target entry
#[derive(Clone, Copy, Debug, PartialEq)]
enum InsertPosition {
    /// Insert before the entry (show border above)
    Before,
    /// Insert after the entry (show border below)
    After,
    /// Insert into the entry (for folders - highlight the whole entry)
    Inside,
}

/// A delegate trait for providing tree data and rendering with drag and drop support.
pub trait DraggableTreeDelegate: Sized + 'static {
    /// Render the tree item at the given index.
    fn render_item(
        &self,
        ix: usize,
        entry: &TreeEntry,
        selected: bool,
        window: &mut Window,
        cx: &mut App,
    ) -> ListItem;

    /// Render the context menu for the tree item at the given index.
    fn context_menu(
        &self,
        _ix: usize,
        _entry: &TreeEntry,
        menu: PopupMenu,
        _window: &mut Window,
        _cx: &mut App,
    ) -> PopupMenu {
        menu
    }

    // === DRAG AND DROP METHODS ===

    /// Determine if the given item can be dragged.
    ///
    /// Return true to allow dragging this item.
    /// Default implementation returns false (nothing draggable).
    fn can_drag(&self, _item_id: &str, _entry: &TreeEntry, _cx: &App) -> bool {
        false
    }

    /// Create the drag data for the item being dragged.
    /// This is called when `can_drag` returns true.
    ///
    /// Default implementation creates a basic DraggedTreeItem.
    fn create_drag_data(
        &self,
        item_id: &str,
        entry: &TreeEntry,
        _cx: &App,
    ) -> Option<DraggedTreeItem> {
        if self.can_drag(item_id, entry, _cx) {
            Some(DraggedTreeItem {
                item_id: entry.item().id.clone(),
                label: entry.item().label.clone(),
                collection_path: "".into(),
                icon: None,
            })
        } else {
            None
        }
    }

    /// Determine if the dragged item can be dropped on the target entry.
    ///
    /// Return true to allow the drop.
    /// Default implementation returns false.
    fn can_drop_on(
        &self,
        _dragged_item: &DraggedTreeItem,
        _target_entry: &TreeEntry,
        _cx: &App,
    ) -> bool {
        false
    }

    /// Determine if the dragged item can be dropped at the root level.
    ///
    /// Default implementation returns false.
    fn can_drop_on_root(&self, _dragged_item: &DraggedTreeItem) -> bool {
        false
    }

    /// Handle the drop operation.
    ///
    /// This is where you would update your data model, save to disk, etc.
    fn on_drop(
        &mut self,
        _dragged_item: &DraggedTreeItem,
        _target_entry_id: Option<&str>,
        _window: &mut Window,
        _cx: &mut App,
    ) {
        // Default: do nothing
    }

    /// Optional: Get icon information for drag visual
    fn get_drag_icon(&self, _item_id: &str, _cx: &App) -> Option<DragIcon> {
        None
    }
}

/// State for managing draggable tree items.
pub struct DraggableTreeState<D: DraggableTreeDelegate> {
    focus_handle: FocusHandle,
    entries: Vec<TreeEntry>,
    selected_ix: Option<usize>,
    right_clicked_index: Option<usize>,
    delegate: D,

    // Drag and drop state
    drag_target_entry: Option<DragTarget>,
    hover_scroll_task: Option<Task<()>>,
    hover_expand_task: Option<Task<()>>,
}

impl<D: DraggableTreeDelegate> DraggableTreeState<D> {
    /// Create a new empty draggable tree state.
    pub fn new(delegate: D, cx: &mut App) -> Self {
        Self {
            selected_ix: None,
            right_clicked_index: None,
            focus_handle: cx.focus_handle(),
            entries: Vec::new(),
            delegate,
            drag_target_entry: None,
            hover_scroll_task: None,
            hover_expand_task: None,
        }
    }

    pub fn entries(&self) -> &Vec<TreeEntry> {
        &self.entries
    }

    /// Set the tree items.
    pub fn set_items(&mut self, items: impl Into<Vec<TreeItem>>, cx: &mut Context<Self>) {
        let items = items.into();
        self.entries.clear();
        for item in items.into_iter() {
            self.add_entry(item, 0);
        }
        self.selected_ix = None;
        cx.notify();
    }

    /// Get the currently selected index, if any.
    pub fn selected_index(&self) -> Option<usize> {
        self.selected_ix
    }

    /// Set the selected index, or `None` to clear selection.
    pub fn set_selected_index(&mut self, ix: Option<usize>, cx: &mut Context<Self>) {
        self.selected_ix = ix;
        cx.notify();
    }

    /// Get the currently selected entry, if any.
    pub fn selected_entry(&self) -> Option<&TreeEntry> {
        self.selected_ix.and_then(|ix| self.entries.get(ix))
    }

    /// Get the delegate.
    pub fn delegate(&self) -> &D {
        &self.delegate
    }

    fn add_entry(&mut self, item: TreeItem, depth: usize) {
        self.entries.push(TreeEntry::new(item.clone(), depth));
        if item.is_expanded() {
            for child in &item.children {
                self.add_entry(child.clone(), depth + 1);
            }
        }
    }

    fn toggle_expand(&mut self, ix: usize) {
        let Some(entry) = self.entries.get(ix) else {
            return;
        };
        if !entry.is_folder() {
            return;
        }

        entry.item().toggle_expanded();
        self.rebuild_entries();
    }

    fn rebuild_entries(&mut self) {
        let root_items: Vec<TreeItem> = self
            .entries
            .iter()
            .filter(|e| e.is_root())
            .map(|e| e.item().clone())
            .collect();
        self.entries.clear();
        for item in root_items.into_iter() {
            self.add_entry(item, 0);
        }
    }

    fn on_action_confirm(&mut self, _: &Confirm, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(selected_ix) = self.selected_ix
            && let Some(entry) = self.entries.get(selected_ix)
            && entry.is_folder()
        {
            self.toggle_expand(selected_ix);
            cx.notify();
        }
    }

    fn on_action_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(selected_ix) = self.selected_ix
            && let Some(entry) = self.entries.get(selected_ix)
            && entry.is_folder()
            && entry.is_expanded()
        {
            self.toggle_expand(selected_ix);
            cx.notify();
        }
    }

    fn on_action_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        if let Some(selected_ix) = self.selected_ix
            && let Some(entry) = self.entries.get(selected_ix)
            && entry.is_folder()
            && !entry.is_expanded()
        {
            self.toggle_expand(selected_ix);
            cx.notify();
        }
    }

    fn on_action_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        let mut selected_ix = self.selected_ix.unwrap_or(0);

        if selected_ix > 0 {
            selected_ix -= 1;
        } else {
            selected_ix = self.entries.len().saturating_sub(1);
        }

        self.selected_ix = Some(selected_ix);
        cx.notify();
    }

    fn on_action_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        let mut selected_ix = self.selected_ix.unwrap_or(0);
        if selected_ix + 1 < self.entries.len() {
            selected_ix += 1;
        } else {
            selected_ix = 0;
        }

        self.selected_ix = Some(selected_ix);
        cx.notify();
    }

    fn on_entry_click(&mut self, ix: usize, _: &mut Window, cx: &mut Context<Self>) {
        self.selected_ix = Some(ix);
        self.toggle_expand(ix);
        cx.notify();
    }

    /// Check if the given entry is currently highlighted for drag feedback.
    fn is_drag_highlighted(&self, entry_id: &str) -> bool {
        match &self.drag_target_entry {
            Some(DragTarget::Entry {
                highlight_entry_id, ..
            }) => highlight_entry_id.as_ref() == entry_id,
            Some(DragTarget::Background) => false,
            None => false,
        }
    }

    /// Get the insert position for the given entry if it's a drag target.
    fn get_insert_position(&self, entry_id: &str) -> Option<InsertPosition> {
        match &self.drag_target_entry {
            Some(DragTarget::Entry {
                entry_id: target_id,
                position,
                ..
            }) => {
                if target_id.as_ref() == entry_id {
                    Some(*position)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// The visual representation shown during drag operations.
struct DraggedRequestView {
    dragged_item: DraggedTreeItem,
    click_offset: Point<Pixels>,
}

impl Render for DraggedRequestView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .absolute()
            .top_0()
            .left_0()
            .pl(self.click_offset.x + px(12.))
            .pt(self.click_offset.y + px(12.))
            .child(
                div()
                    .flex()
                    .gap_2()
                    .items_center()
                    .py_1()
                    .px_3()
                    .rounded(cx.theme().radius)
                    .shadow_lg()
                    .bg(cx.theme().background)
                    .border_1()
                    .border_color(cx.theme().border)
                    .when_some(self.dragged_item.icon.as_ref(), |this, icon| {
                        this.child(h_flex().gap_2().when_some(
                            icon.prefix.as_ref(),
                            |this, prefix| {
                                this.child(
                                    div()
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_sm()
                                        .font_bold()
                                        .text_color((icon.color_fn)(cx))
                                        .child(prefix.clone()),
                                )
                            },
                        ))
                    })
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().foreground)
                            .child(self.dragged_item.label.clone()),
                    ),
            )
    }
}

impl<D: DraggableTreeDelegate> Render for DraggableTreeState<D> {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .id("draggable-tree-state")
            .size_full()
            .relative()
            .context_menu({
                let view = cx.entity().clone();
                move |this, window: &mut Window, cx: &mut Context<PopupMenu>| {
                    if let Some(ix) = view.read(cx).right_clicked_index {
                        view.update(cx, |state, cx| {
                            let entry = state.entries.get(ix).unwrap();
                            state.delegate().context_menu(ix, entry, this, window, cx)
                        })
                    } else {
                        this
                    }
                }
            })
            .child(
                div()
                    .flex()
                    .flex_col()
                    .size_full()
                    .overflow_y_scrollbar()
                    // Background drop zone for root-level drops - receives drag events when hovering empty space
                    .on_drag_move::<DraggedTreeItem>(cx.listener(
                        |this, _event: &DragMoveEvent<DraggedTreeItem>, _, cx| {
                            // Check if we're hovering in empty space (not over any entry)
                            let is_over_entry = this.entries.iter().any(|_entry| {
                                // This is a simplified check - in practice, we'd need to check actual bounds
                                // For now, we use the drag_target_entry state to determine if we're over an entry
                                matches!(this.drag_target_entry, Some(DragTarget::Entry { .. }))
                            });

                            if !is_over_entry {
                                if !matches!(this.drag_target_entry, Some(DragTarget::Background)) {
                                    this.drag_target_entry = Some(DragTarget::Background);
                                    cx.notify();
                                }
                            } else if matches!(this.drag_target_entry, Some(DragTarget::Background))
                            {
                                this.drag_target_entry = None;
                                cx.notify();
                            }
                        },
                    ))
                    .on_drop::<DraggedTreeItem>(cx.listener(|this, dropped_item, window, cx| {
                        this.drag_target_entry = None;
                        this.hover_scroll_task.take();

                        if this.delegate.can_drop_on_root(dropped_item) {
                            this.delegate.on_drop(dropped_item, None, window, cx);
                        }

                        cx.notify();
                    }))
                    .children(self.entries.iter().enumerate().map(|(ix, entry)| {
                        let item = entry.item();
                        let selected = Some(ix) == self.selected_ix;
                        let cx_app = &**cx; // Convert &mut Context<Self> to &App

                        // Check if this item can be dragged and create drag data
                        let can_drag = self.delegate.can_drag(item.id.as_ref(), entry, cx_app);
                        let drag_data = if can_drag {
                            self.delegate
                                .create_drag_data(item.id.as_ref(), entry, cx_app)
                        } else {
                            None
                        };

                        div()
                            .id(("entry", ix))
                            .when(self.is_drag_highlighted(item.id.as_ref()), |this| {
                                this.bg(cx.theme().muted_foreground.opacity(0.2))
                            })
                            .child(
                                self.delegate
                                    .render_item(ix, entry, selected, window, cx)
                                    .disabled(entry.item().is_disabled())
                                    .selected(selected),
                            )
                            .when(!entry.item().is_disabled(), |this| {
                                this.on_mouse_down(
                                    MouseButton::Left,
                                    cx.listener(move |this, _, window, cx| {
                                        this.on_entry_click(ix, window, cx);
                                    }),
                                )
                                .on_mouse_down(
                                    MouseButton::Right,
                                    cx.listener(move |this, _, _, cx| {
                                        this.right_clicked_index = Some(ix);
                                        cx.notify();
                                    }),
                                )
                            })
                            // === DRAG AND DROP HANDLERS ===
                            .when_some(drag_data, |this, data| {
                                // Start drag - creates drag visual
                                this.on_drag(
                                    data,
                                    |dragged_data: &DraggedTreeItem, click_offset, _window, cx| {
                                        cx.new(|_| DraggedRequestView {
                                            dragged_item: dragged_data.clone(),
                                            click_offset,
                                        })
                                    },
                                )
                            })
                            .on_drag_move::<DraggedTreeItem>(cx.listener({
                                let item_id = item.id.clone();
                                let item_for_delegate = entry.clone();
                                let is_folder = entry.is_folder();
                                move |this, event: &DragMoveEvent<DraggedTreeItem>, _window, cx| {
                                    let dragged_item = event.drag(cx);
                                    let cx_app = &**cx; // Convert to &App

                                    // Check if this is the current target (prevent duplicate handling)
                                    let is_current_target = match &this.drag_target_entry {
                                        Some(DragTarget::Entry { entry_id, .. }) => {
                                            entry_id.as_ref() == item_id.as_ref()
                                        }
                                        _ => false,
                                    };

                                    // Clear highlight if mouse left this element's bounds
                                    if !event.bounds.contains(&event.event.position) {
                                        if is_current_target {
                                            this.drag_target_entry = None;
                                            this.hover_scroll_task.take();
                                            this.hover_expand_task.take();
                                            cx.notify();
                                        }
                                        return;
                                    }

                                    // Check if we can drop on this entry
                                    let can_drop = this.delegate.can_drop_on(
                                        dragged_item,
                                        &item_for_delegate,
                                        cx_app,
                                    );

                                    if can_drop && !is_current_target {
                                        // Calculate position based on cursor location
                                        let relative_y =
                                            event.event.position.y - event.bounds.origin.y;
                                        let height = event.bounds.size.height;
                                        let position = if is_folder {
                                            // For folders, always insert inside
                                            InsertPosition::Inside
                                        } else if relative_y < height / 2.0 {
                                            // Top half of non-folder entry - insert before
                                            InsertPosition::Before
                                        } else {
                                            // Bottom half of non-folder entry - insert after
                                            InsertPosition::After
                                        };

                                        this.drag_target_entry = Some(DragTarget::Entry {
                                            entry_id: item_id.clone(),
                                            highlight_entry_id: item_id.clone(),
                                            position,
                                        });

                                        cx.notify();
                                    }
                                }
                            }))
                            .on_drop::<DraggedTreeItem>(cx.listener({
                                let item_id = item.id.clone();
                                let item_for_delegate = entry.clone();
                                move |this, dropped_item: &DraggedTreeItem, window, cx| {
                                    let cx_app = &**cx; // Convert to &App

                                    // Clear all drag state
                                    this.drag_target_entry = None;
                                    this.hover_scroll_task.take();
                                    this.hover_expand_task.take();

                                    // Check if we can drop on this target
                                    let can_drop = this.delegate.can_drop_on(
                                        dropped_item,
                                        &item_for_delegate,
                                        cx_app,
                                    );

                                    if can_drop {
                                        // Notify delegate to handle the drop
                                        this.delegate.on_drop(
                                            dropped_item,
                                            Some(item_id.as_ref()),
                                            window,
                                            cx,
                                        );
                                    }

                                    cx.notify();
                                }
                            }))
                    })),
            )
    }
}

/// A draggable tree view element that displays hierarchical data with drag and drop support.
#[derive(IntoElement)]
pub struct DraggableTree<D: DraggableTreeDelegate> {
    id: ElementId,
    state: Entity<DraggableTreeState<D>>,
    style: StyleRefinement,
}

impl<D: DraggableTreeDelegate> DraggableTree<D> {
    pub fn new(state: &Entity<DraggableTreeState<D>>) -> Self {
        Self {
            id: ElementId::Name(format!("draggable-tree-{}", state.entity_id()).into()),
            state: state.clone(),
            style: StyleRefinement::default(),
        }
    }
}

impl<D: DraggableTreeDelegate> Styled for DraggableTree<D> {
    fn style(&mut self) -> &mut StyleRefinement {
        &mut self.style
    }
}

impl<D: DraggableTreeDelegate> RenderOnce for DraggableTree<D> {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let focus_handle = self.state.read(cx).focus_handle.clone();

        div()
            .id(self.id)
            .key_context(CONTEXT)
            .track_focus(&focus_handle)
            .on_action(window.listener_for(&self.state, DraggableTreeState::<D>::on_action_confirm))
            .on_action(window.listener_for(&self.state, DraggableTreeState::<D>::on_action_left))
            .on_action(window.listener_for(&self.state, DraggableTreeState::<D>::on_action_right))
            .on_action(window.listener_for(&self.state, DraggableTreeState::<D>::on_action_up))
            .on_action(window.listener_for(&self.state, DraggableTreeState::<D>::on_action_down))
            .size_full()
            .child(self.state)
            .refine_style(&self.style)
    }
}
