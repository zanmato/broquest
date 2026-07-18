//! Read-only inspector for a collection's variables.
//!
//! Shows two tables:
//! - **Collection variables** — the declarative, persisted key/value pairs
//!   defined on the collection (`CollectionMeta.vars`), resolvable via
//!   `{{name}}`.
//! - **Runtime variables** — in-memory values set by scripts via
//!   `bro.setVar`, session-scoped per collection
//!   (see [`crate::collections::manager`]).
//!
//! The view is read-only. Collection variables are edited in the Collection
//! editor's "Vars" tab; runtime variables are produced by scripts at request
//! time.

use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, Styled, Window, div,
};
use gpui_component::{ActiveTheme, h_flex, scroll::ScrollableElement, v_flex};

use crate::collections::manager::{CollectionManager, CollectionManagerEvent};

/// A single displayed variable row (key + value, value rendered as a string).
#[derive(Clone)]
struct VarRow {
    key: String,
    value: String,
}

/// Read-only variable inspector for a collection, scoped by `collection_path`.
pub struct VarsView {
    collection_path: Option<String>,
    collection_vars: Vec<VarRow>,
    runtime_vars: Vec<VarRow>,
    /// When true, render at natural height without an own scroll region (so it
    /// can be embedded inside a parent scroll area).
    embedded: bool,
    focus_handle: FocusHandle,
    _subscriptions: Vec<gpui::Subscription>,
}

impl VarsView {
    /// Create a view not yet bound to a collection. Call [`Self::set_collection`]
    /// to scope it.
    pub fn new(cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();

        // Refresh whenever the CollectionManager changes (e.g. after a request
        // writes back runtime vars).
        let manager = CollectionManager::global(cx);
        let subscription = cx.subscribe(
            &manager,
            |this, _manager, event: &CollectionManagerEvent, cx| {
                // Only refresh for changes to the bound collection (or a global
                // collection-set change).
                let relevant = match event {
                    CollectionManagerEvent::CollectionsChanged => true,
                    CollectionManagerEvent::EnvironmentsChanged { collection_path }
                    | CollectionManagerEvent::RequestsChanged { collection_path }
                    | CollectionManagerEvent::RuntimeVarsChanged { collection_path } => {
                        this.collection_path.as_deref() == Some(collection_path.as_ref())
                    }
                };
                if relevant {
                    this.refresh_from_manager(cx);
                }
            },
        );

        Self {
            collection_path: None,
            collection_vars: Vec::new(),
            runtime_vars: Vec::new(),
            embedded: false,
            focus_handle,
            _subscriptions: vec![subscription],
        }
    }

    /// Render at natural height for embedding in a parent scroll region.
    pub fn set_embedded(&mut self, embedded: bool) {
        self.embedded = embedded;
    }

    /// Scope this view to a collection and load its current vars.
    pub fn set_collection(&mut self, collection_path: Option<String>, cx: &mut Context<Self>) {
        self.collection_path = collection_path;
        self.refresh_from_manager(cx);
    }

    /// Reload both sections from the global CollectionManager. Cheap: clones a
    /// couple of small maps. A no-op when no collection is bound.
    fn refresh_from_manager(&mut self, cx: &mut Context<Self>) {
        let Some(path) = self.collection_path.clone() else {
            self.collection_vars.clear();
            self.runtime_vars.clear();
            cx.notify();
            return;
        };
        let manager = CollectionManager::global(cx);
        let manager = manager.read(cx);
        if let Some(info) = manager.get_collection_by_path(&path) {
            self.collection_vars = info
                .toml
                .collection
                .vars
                .iter()
                .filter(|v| v.enabled && !v.key.is_empty())
                .map(|v| VarRow {
                    key: v.key.clone(),
                    value: v.value.clone(),
                })
                .collect();
            self.runtime_vars = info
                .runtime_vars
                .iter()
                .map(|(k, v)| VarRow {
                    key: k.clone(),
                    value: value_to_display(v),
                })
                .collect();
        } else {
            self.collection_vars.clear();
            self.runtime_vars.clear();
        }
        cx.notify();
    }

    /// A section header with just the title (no redundant count).
    fn render_header(&self, title: &str, cx: &mut Context<Self>) -> impl IntoElement {
        h_flex()
            .items_center()
            .gap_2()
            // Align with the editable request-vars editor's key column: its row
            // pads pl_2 (8px) and the small Input adds 8px internal padding, so
            // the key text starts at 16px — match that here (pl_4).
            .pl_4()
            .pr_3()
            .pt_3()
            .pb_1()
            .child(
                div()
                    .text_sm()
                    .text_color(cx.theme().muted_foreground)
                    .child(SharedString::from(title.to_string())),
            )
    }

    /// A read-only key:value table mirroring the query-param editor's row look.
    /// Rows stretch their cells to equal height so the key/value divider spans
    /// the full row even when a value wraps.
    fn render_table(&self, rows: &[VarRow], cx: &mut Context<Self>) -> impl IntoElement {
        let mono = cx.theme().mono_font_family.clone();
        let table_bg = cx.theme().table;
        let border = cx.theme().border;
        let fg = cx.theme().foreground;
        let muted = cx.theme().muted_foreground;

        v_flex()
            .border_t_1()
            .border_color(border)
            .children(rows.iter().map(|row| {
                h_flex()
                    // Align the key column at 16px to match the header and the
                    // editable request-vars input above.
                    .pl_4()
                    .pr_3()
                    // Stretch cells to equal height so the divider spans the row.
                    .items_stretch()
                    .bg(table_bg)
                    .border_b_1()
                    .border_color(border)
                    .child(
                        div()
                            .flex_1()
                            .border_r_1()
                            .border_color(border)
                            // No left padding: the row's pl_4 already positions
                            // the key text at 16px.
                            .pr_2()
                            .py_2()
                            .font_family(mono.clone())
                            .text_sm()
                            .text_color(fg)
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(SharedString::from(row.key.clone())),
                    )
                    .child(
                        div()
                            .flex_1()
                            // No right border on the last column; the table's own
                            // borders frame the row.
                            .px_2()
                            .py_2()
                            .font_family(mono.clone())
                            .text_sm()
                            .text_color(muted)
                            // Allow wrapping for long values; the divider on the
                            // key cell stretches to match via items_stretch.
                            .child(SharedString::from(row.value.clone())),
                    )
            }))
    }

    fn render_section(
        &self,
        title: &str,
        rows: &[VarRow],
        empty_hint: &str,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let body = if rows.is_empty() {
            div()
                .pl_4()
                .pr_3()
                .py_2()
                .text_xs()
                .text_color(cx.theme().muted_foreground)
                .child(SharedString::from(empty_hint.to_string()))
                .into_any_element()
        } else {
            self.render_table(rows, cx).into_any_element()
        };

        v_flex()
            .gap_1()
            .child(self.render_header(title, cx))
            .child(body)
    }
}

/// Flatten a JSON value to a display string: strings verbatim, null empty,
/// everything else compact JSON.
fn value_to_display(v: &serde_json::Value) -> String {
    match v {
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Null => String::new(),
        other => other.to_string(),
    }
}

impl Focusable for VarsView {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl EventEmitter<()> for VarsView {}

impl Render for VarsView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Clone the row data so the borrows don't outlive the theme lookups.
        let collection = self.collection_vars.clone();
        let runtime = self.runtime_vars.clone();

        let content = v_flex()
            .child(self.render_section(
                "Collection Variables",
                &collection,
                "No collection variables defined. Add them in the collection's Vars tab.",
                cx,
            ))
            .child(self.render_section(
                "Runtime Variables",
                &runtime,
                "No runtime variables set. Use bro.setVar(name, value) in a script.",
                cx,
            ));

        if self.embedded {
            // Natural height; the parent scroll region handles scrolling.
            div()
                .track_focus(&self.focus_handle)
                .child(content)
                .into_any_element()
        } else {
            div()
                .track_focus(&self.focus_handle)
                .size_full()
                .overflow_y_scrollbar()
                .child(content)
                .into_any_element()
        }
    }
}
