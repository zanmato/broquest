//! Closing several tabs at once, asking to save the changed ones first, and
//! the tab context menu.

use std::rc::Rc;

use gpui::{
    App, Context, EntityId, ParentElement as _, Styled as _, WeakEntity, Window, div,
    prelude::FluentBuilder as _,
};
use gpui_kit::component::{
    ActiveTheme as _, WindowExt as _,
    button::{Button, ButtonVariants as _},
    dialog::{DialogClose, DialogFooter},
    menu::{PopupMenu, PopupMenuItem},
    v_flex,
};

use super::{EditorPanel, TabType};
use crate::result_ext::ResultExt as _;

/// Which tabs a close command closes, around the tab it is given.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CloseScope {
    This,
    Others,
    Left,
    Right,
    All,
}

impl CloseScope {
    /// The indices of the tabs it closes among `len` tabs, around `ix`.
    fn indices(self, ix: usize, len: usize) -> Vec<usize> {
        match self {
            Self::This if ix < len => vec![ix],
            Self::This => Vec::new(),
            Self::Others => (0..len).filter(|&other| other != ix).collect(),
            Self::Left => (0..ix.min(len)).collect(),
            Self::Right => (ix + 1..len).collect(),
            Self::All => (0..len).collect(),
        }
    }
}

impl EditorPanel {
    /// Close the tabs `scope` names around the tab at `ix`, asking first
    /// when some of them have unsaved changes.
    pub fn close_tabs(
        &mut self,
        scope: CloseScope,
        ix: usize,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let indices = scope.indices(ix, self.tabs.len());
        if indices.is_empty() {
            return;
        }
        let ids: Vec<EntityId> = indices
            .iter()
            .map(|&ix| Self::tab_entity_id(&self.tabs[ix]))
            .collect();
        // The tab to show afterwards: the active one when it stays open,
        // else the tab the command was given, as with Close Others.
        let active = self.tabs.get(self.active_tab_ix).map(Self::tab_entity_id);
        let anchor = self.tabs.get(ix).map(Self::tab_entity_id);
        let keep = [active, anchor]
            .into_iter()
            .flatten()
            .find(|id| !ids.contains(id));

        let unsaved: Vec<(EntityId, String)> = indices
            .iter()
            .filter_map(|&ix| match &self.tabs[ix] {
                TabType::Request(tab) if tab.is_dirty => {
                    Some((tab.request_editor.entity_id(), tab.title.clone()))
                }
                _ => None,
            })
            .collect();
        if unsaved.is_empty() {
            self.close_tabs_by_id(&ids, keep, window, cx);
            return;
        }
        self.confirm_close_unsaved(ids, unsaved, keep, window, cx);
    }

    /// Close the tabs `scope` names around the active tab.
    pub fn close_tabs_around_active(
        &mut self,
        scope: CloseScope,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.close_tabs(scope, self.active_tab_ix, window, cx);
    }

    /// Ask whether to save the changed requests among the tabs being closed.
    /// Saving closes the tabs that saved and keeps any that could not be.
    fn confirm_close_unsaved(
        &mut self,
        ids: Vec<EntityId>,
        unsaved: Vec<(EntityId, String)>,
        keep: Option<EntityId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let title = match unsaved.as_slice() {
            [(_, name)] => format!("Save changes to {name}?"),
            _ => format!("Save changes to {} requests?", unsaved.len()),
        };
        let names: Vec<String> = match unsaved.as_slice() {
            [_] => Vec::new(),
            _ => unsaved.iter().map(|(_, name)| name.clone()).collect(),
        };
        let save_label = if unsaved.len() > 1 {
            "Save All"
        } else {
            "Save"
        };

        let weak = cx.entity().downgrade();
        let respond = Rc::new(move |save: bool, window: &mut Window, cx: &mut App| {
            let _ = weak
                .update(cx, |this, cx| {
                    let mut ids = ids.clone();
                    if save {
                        for (id, _) in &unsaved {
                            let saved = this.request_editor_by_id(*id).is_some_and(|editor| {
                                editor.update(cx, |editor, cx| editor.save_request(window, cx))
                            });
                            if !saved {
                                ids.retain(|other| other != id);
                            }
                        }
                    }
                    this.close_tabs_by_id(&ids, keep, window, cx);
                })
                .log_err();
            window.close_dialog(cx);
        });

        window.open_dialog(cx, move |dialog, _, cx| {
            let save = respond.clone();
            let discard = respond.clone();
            dialog
                .title(title.clone())
                .child(
                    v_flex()
                        .gap_2()
                        .when(!names.is_empty(), |this| {
                            this.child(v_flex().children(names.iter().map(|name| {
                                div()
                                    .text_sm()
                                    .text_color(cx.theme().muted_foreground)
                                    .child(name.clone())
                            })))
                        })
                        .child("Your changes are lost if you close without saving."),
                )
                .footer(
                    DialogFooter::new()
                        .child(
                            DialogClose::new()
                                .child(Button::new("close-tabs-cancel").label("Cancel").outline()),
                        )
                        // Sibling `DialogAction`s share a click state, so the
                        // buttons close the dialog themselves.
                        .child(
                            Button::new("close-tabs-discard")
                                .outline()
                                .label("Don't Save")
                                .on_click(move |_, window, cx| discard(false, window, cx)),
                        )
                        .child(
                            Button::new("close-tabs-save")
                                .primary()
                                .label(save_label)
                                .on_click(move |_, window, cx| save(true, window, cx)),
                        ),
                )
        });
    }

    /// Close the tabs whose views are `ids`, then show `keep` when it is
    /// still open.
    fn close_tabs_by_id(
        &mut self,
        ids: &[EntityId],
        keep: Option<EntityId>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        // Closing the tab that has focus would leave nothing focused, and
        // shortcuts need a focused element to reach. The panel takes it.
        let had_focus = self.focus_handle.contains_focused(window, cx);
        // Highest first, so the other indices stay put.
        for ix in (0..self.tabs.len()).rev() {
            if ids.contains(&Self::tab_entity_id(&self.tabs[ix])) {
                self.close_tab(ix, cx);
            }
        }
        if let Some(ix) = keep.and_then(|keep| {
            self.tabs
                .iter()
                .position(|tab| Self::tab_entity_id(tab) == keep)
        }) {
            self.active_tab_ix = ix;
            self.scroll_tabbar_to_active();
        }
        if had_focus {
            self.focus_handle.focus(window, cx);
        }
        cx.notify();
    }

    fn request_editor_by_id(&self, id: EntityId) -> Option<gpui::Entity<super::RequestEditor>> {
        self.tabs.iter().find_map(|tab| match tab {
            TabType::Request(tab) if tab.request_editor.entity_id() == id => {
                Some(tab.request_editor.clone())
            }
            _ => None,
        })
    }

    /// The id of the tab's view, which stays the same while tabs move.
    fn tab_entity_id(tab: &TabType) -> EntityId {
        match tab {
            TabType::Request(tab) => tab.request_editor.entity_id(),
            TabType::Collection(tab) => tab.collection_editor.entity_id(),
            TabType::Group(tab) => tab.group_editor.entity_id(),
            TabType::Settings(tab) => tab.settings_view.entity_id(),
        }
    }

    /// Open a copy of the request tab at `ix` right after it. The copy is
    /// named "<name> copy" and is not saved until the user saves it.
    pub(super) fn duplicate_tab(&mut self, ix: usize, window: &mut Window, cx: &mut Context<Self>) {
        let Some(TabType::Request(tab)) = self.tabs.get(ix) else {
            return;
        };
        let source = tab.request_editor.clone();
        let collection_path = tab.collection_path.clone();
        let (mut request, group_path, environment) = source.update(cx, |editor, cx| {
            (
                editor.edited_request_data(cx),
                editor.group_path().map(str::to_string),
                editor.selected_environment_name(cx),
            )
        });
        request.name = format!("{} copy", request.name);

        self.create_and_add_request_tab(request, collection_path, group_path, window, cx);
        let copy = self.tabs.len() - 1;
        if let Some(TabType::Request(tab)) = self.tabs.get(copy) {
            tab.request_editor.update(cx, |editor, cx| {
                if let Some(environment) = &environment {
                    editor.select_environment(environment, window, cx);
                }
                editor.mark_unsaved(cx);
            });
        }
        self.move_tab(copy, ix + 1, cx);
        self.activate_tab(ix + 1, cx);
    }

    /// The right click menu of the tab at `ix`. It reads the panel when the
    /// menu opens, so it can be built from a closure made at render time.
    pub(super) fn tab_menu(
        panel: &WeakEntity<Self>,
        ix: usize,
        menu: PopupMenu,
        cx: &App,
    ) -> PopupMenu {
        let Some((len, is_request)) = panel.upgrade().and_then(|panel| {
            let panel = panel.read(cx);
            let tab = panel.tabs.get(ix)?;
            Some((panel.tabs.len(), matches!(tab, TabType::Request(_))))
        }) else {
            return menu;
        };
        let duplicate = {
            let panel = panel.clone();
            PopupMenuItem::new("Duplicate")
                .disabled(!is_request)
                .on_click(move |_, window, cx| {
                    let _ = panel
                        .update(cx, |panel, cx| panel.duplicate_tab(ix, window, cx))
                        .log_err();
                })
        };
        let close = |label: &'static str, scope: CloseScope| {
            let panel = panel.clone();
            PopupMenuItem::new(label)
                .disabled(scope.indices(ix, len).is_empty())
                .on_click(move |_, window, cx| {
                    let _ = panel
                        .update(cx, |panel, cx| panel.close_tabs(scope, ix, window, cx))
                        .log_err();
                })
        };
        menu.item(duplicate)
            .separator()
            .item(close("Close", CloseScope::This))
            .item(close("Close Others", CloseScope::Others))
            .item(close("Close to the Left", CloseScope::Left))
            .item(close("Close to the Right", CloseScope::Right))
            .separator()
            .item(close("Close All", CloseScope::All))
    }
}

#[cfg(test)]
mod tests {
    use super::CloseScope;

    #[test]
    fn scopes_pick_the_tabs_around_the_given_one() {
        assert_eq!(CloseScope::This.indices(1, 4), [1]);
        assert_eq!(CloseScope::Others.indices(1, 4), [0, 2, 3]);
        assert_eq!(CloseScope::Left.indices(2, 4), [0, 1]);
        assert_eq!(CloseScope::Right.indices(1, 4), [2, 3]);
        assert_eq!(CloseScope::All.indices(1, 4), [0, 1, 2, 3]);
    }

    #[test]
    fn scopes_at_the_ends_close_nothing_past_them() {
        assert!(CloseScope::Left.indices(0, 3).is_empty());
        assert!(CloseScope::Right.indices(2, 3).is_empty());
        assert!(CloseScope::Others.indices(0, 1).is_empty());
        assert!(CloseScope::This.indices(5, 3).is_empty());
    }
}
