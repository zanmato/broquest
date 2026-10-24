use gpui::{
    AnyElement, App, AppContext as _, Context, Entity, FontWeight, InteractiveElement, IntoElement,
    MouseButton, ParentElement, Render, ScrollHandle, StatefulInteractiveElement as _, Styled,
    Subscription, Window, div, prelude::FluentBuilder, px,
};
use gpui_kit::component::{
    ActiveTheme as _, Icon, Sizable as _, h_flex, menu::ContextMenuExt as _,
    scroll::ScrollableElement as _, v_flex,
};

use super::{CloseScope, DraggedTab, EditorPanel, TabType};
use crate::collections::CollectionManager;
use crate::ui::icon::IconName;
use crate::ui::method_badge::{MethodBadge, SLOT_WIDTH};

const ROW_HEIGHT: gpui::Pixels = px(28.);

/// The sidebar's list of the open tabs, grouped by collection. It holds no tab
/// state of its own: everything is read from and done through the
/// `EditorPanel`, so the strip and the list always agree.
pub struct OpenTabsPanel {
    editor_panel: Entity<EditorPanel>,
    scroll_handle: ScrollHandle,
    /// The tab the list last scrolled to. The list follows the active tab
    /// only when it changes, so scrolling by hand is not undone on every
    /// render.
    scrolled_to_tab: Option<usize>,
    _subscription: Subscription,
}

impl OpenTabsPanel {
    pub fn new(editor_panel: Entity<EditorPanel>, cx: &mut Context<Self>) -> Self {
        let subscription = cx.observe(&editor_panel, |_, _, cx| cx.notify());
        Self {
            editor_panel,
            scroll_handle: ScrollHandle::new(),
            scrolled_to_tab: None,
            _subscription: subscription,
        }
    }
}

impl Render for OpenTabsPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let (rows, active) = self
            .editor_panel
            .update(cx, |panel, cx| panel.render_open_tabs(cx));
        if let Some((tab_ix, row_ix)) = active
            && self.scrolled_to_tab != Some(tab_ix)
        {
            self.scrolled_to_tab = Some(tab_ix);
            self.scroll_handle.scroll_to_item(row_ix);
        }

        v_flex().id("open-tabs-panel").size_full().child(
            div()
                .flex_1()
                .min_h_0()
                .relative()
                .child(
                    v_flex()
                        .id("open-tabs-scroll")
                        .size_full()
                        .px_1()
                        .py_1()
                        .overflow_y_scroll()
                        .track_scroll(&self.scroll_handle)
                        .when(rows.is_empty(), |this| {
                            this.child(
                                div()
                                    .p_2()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child("No open tabs"),
                            )
                        })
                        .children(rows),
                )
                // Beside the list rather than inside it, so the list's
                // children are exactly the rows `scroll_to_item` counts.
                .vertical_scrollbar(&self.scroll_handle),
        )
    }
}

impl EditorPanel {
    /// Tab indices in strip order, bucketed by collection in the order each
    /// collection first appears. Tabs without one go into a trailing bucket.
    fn open_tab_groups(&self) -> Vec<(Option<&str>, Vec<usize>)> {
        let mut groups: Vec<(Option<&str>, Vec<usize>)> = Vec::new();
        let mut other = Vec::new();
        for (ix, tab) in self.tabs.iter().enumerate() {
            match Self::tab_collection_path(tab) {
                Some(path) => match groups.iter_mut().find(|(key, _)| *key == Some(path)) {
                    Some((_, indices)) => indices.push(ix),
                    None => groups.push((Some(path), vec![ix])),
                },
                None => other.push(ix),
            }
        }
        if !other.is_empty() {
            groups.push((None, other));
        }
        groups
    }

    /// The name of the collection at `path`, falling back to the name a
    /// request tab was opened with.
    fn group_title(&self, path: Option<&str>, first_tab: Option<usize>, cx: &App) -> String {
        let Some(path) = path else {
            return "Other".to_string();
        };
        CollectionManager::global(cx)
            .read(cx)
            .get_collection_by_path(path)
            .map(|info| info.toml.collection.name.clone())
            .or_else(|| match first_tab.and_then(|ix| self.tabs.get(ix)) {
                Some(TabType::Request(tab)) => Some(tab.collection_name.clone()),
                _ => None,
            })
            .unwrap_or_else(|| path.to_string())
    }

    /// The list's rows, group headers included, and the active tab's index
    /// with the position of its row.
    pub(super) fn render_open_tabs(
        &mut self,
        cx: &mut Context<Self>,
    ) -> (Vec<AnyElement>, Option<(usize, usize)>) {
        let groups: Vec<(String, Vec<usize>)> = self
            .open_tab_groups()
            .into_iter()
            .map(|(path, indices)| {
                (
                    self.group_title(path, indices.first().copied(), cx),
                    indices,
                )
            })
            .collect();
        let mut rows = Vec::new();
        let mut active = None;
        for (group_ix, (title, indices)) in groups.into_iter().enumerate() {
            rows.push(
                h_flex()
                    .flex_none()
                    .px_2()
                    .items_center()
                    .h(px(24.))
                    .when(group_ix > 0, |this| this.mt_2())
                    .child(
                        div()
                            .flex_1()
                            .min_w_0()
                            .truncate()
                            .text_xs()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_color(cx.theme().foreground)
                            .child(title),
                    )
                    .into_any_element(),
            );
            for ix in indices {
                if let Some(row) = self.render_open_tab_row(ix, cx) {
                    if ix == self.active_tab_ix {
                        active = Some((ix, rows.len()));
                    }
                    rows.push(row);
                }
            }
        }
        (rows, active)
    }

    fn render_open_tab_row(&self, ix: usize, cx: &mut Context<Self>) -> Option<AnyElement> {
        let tab = self.tabs.get(ix)?;
        let title = Self::tab_title(tab);
        let muted = cx.theme().muted_foreground;
        let selected = ix == self.active_tab_ix;
        let dragged = DraggedTab {
            ix,
            label: title.clone(),
        };

        // What kind of tab it is: the method for a request, an icon otherwise.
        let kind = match tab {
            TabType::Request(tab) => MethodBadge::new(tab.method)
                .fixed_width()
                .into_any_element(),
            TabType::Collection(_) => Icon::new(IconName::Folder)
                .small()
                .text_color(muted)
                .into_any_element(),
            TabType::Group(_) => Icon::new(IconName::FolderOpen)
                .small()
                .text_color(muted)
                .into_any_element(),
            TabType::Settings(_) => Icon::new(IconName::Settings)
                .small()
                .text_color(muted)
                .into_any_element(),
        };
        let is_dirty = matches!(tab, TabType::Request(tab) if tab.is_dirty);

        let row = h_flex()
            .id(("open-tab", ix))
            .group("open-tab")
            .relative()
            .w_full()
            .flex_none()
            .h(ROW_HEIGHT)
            .px_2()
            .gap_2()
            .items_center()
            .rounded(cx.theme().radius)
            .cursor_pointer()
            .text_sm()
            .text_color(cx.theme().foreground)
            .map(|this| {
                if selected {
                    this.bg(cx.theme().list_active)
                } else {
                    this.hover(|this| this.bg(cx.theme().list_hover))
                }
            })
            .on_click(cx.listener(move |this, _, _, cx| this.activate_tab(ix, cx)))
            .on_mouse_down(
                MouseButton::Middle,
                cx.listener(move |this, _, window, cx| {
                    cx.stop_propagation();
                    this.close_tabs(CloseScope::This, ix, window, cx);
                }),
            )
            .on_drag(dragged, |dragged, _, _, cx| {
                cx.stop_propagation();
                cx.new(|_| dragged.clone())
            })
            // A tab dragged downwards lands after the one it is dropped on.
            .drag_over::<DraggedTab>(move |style, dragged, _, cx| {
                if dragged.ix < ix {
                    style.border_b_2()
                } else {
                    style.border_t_2()
                }
                .border_color(cx.theme().drag_border)
            })
            .on_drop(cx.listener(move |this, dragged: &DraggedTab, _, cx| {
                this.move_tab(dragged.ix, ix, cx);
            }))
            // As wide as a method badge, so the titles line up.
            .child(div().flex_none().w(px(SLOT_WIDTH)).child(kind))
            .child(div().flex_1().min_w_0().truncate().child(title))
            .when(is_dirty, |this| {
                this.child(
                    div()
                        .flex_none()
                        .w(px(6.))
                        .h(px(6.))
                        .rounded_full()
                        .bg(muted)
                        .group_hover("open-tab", |this| this.invisible()),
                )
            })
            // Overlaid on the row's right end so showing it on hover does not
            // shift the title.
            .child(
                div()
                    .absolute()
                    .right_1()
                    .invisible()
                    .group_hover("open-tab", |this| this.visible())
                    .child(self.close_tab_button(("open-tab-close", ix), ix, cx)),
            )
            .context_menu({
                let panel = cx.entity().downgrade();
                move |menu, _, cx| EditorPanel::tab_menu(&panel, ix, menu, cx)
            });

        Some(row.into_any_element())
    }
}
