use gpui::{
    App, Context, EventEmitter, FocusHandle, Focusable, InteractiveElement, IntoElement,
    ParentElement, Render, Styled, Window, div, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, Sizable, StyledExt,
    button::{Button, ButtonVariants},
    h_flex,
    scroll::ScrollableElement,
    v_flex,
};

use crate::app_database::{AppDatabase, HistoryEntry};
use crate::app_events::AppEvent;
use crate::domain::HttpMethod;
use crate::result_ext::ResultExt;
use crate::ui::icon::IconName;

pub struct HistoryPanel {
    focus_handle: FocusHandle,
    entries: Vec<HistoryEntry>,
    pub loaded: bool,
}

impl EventEmitter<AppEvent> for HistoryPanel {}

impl Focusable for HistoryPanel {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl HistoryPanel {
    pub fn new(_window: &mut Window, cx: &mut Context<Self>) -> Self {
        Self {
            focus_handle: cx.focus_handle(),
            entries: Vec::new(),
            loaded: false,
        }
    }

    pub fn load_history(&mut self, cx: &mut Context<Self>) {
        let db = AppDatabase::global(cx).clone();
        cx.spawn(async move |this, cx| {
            let entries = db
                .load_recent_history(200)
                .await
                .log_err()
                .unwrap_or_default();
            this.update(cx, |panel, cx| {
                panel.entries = entries;
                panel.loaded = true;
                cx.notify();
            })
            .log_err()
            .ok();
        })
        .detach();
    }

    pub fn add_entry(&mut self, entry: HistoryEntry, cx: &mut Context<Self>) {
        self.entries.insert(0, entry);
        if self.entries.len() > 200 {
            self.entries.truncate(200);
        }
        cx.notify();
    }

    fn clear_history(&mut self, cx: &mut Context<Self>) {
        let db = AppDatabase::global(cx).clone();
        cx.spawn(async move |this, cx| {
            db.clear_history().await.log_err().ok();
            this.update(cx, |panel, cx| {
                panel.entries.clear();
                cx.notify();
            })
            .log_err()
            .ok();
        })
        .detach();
    }

    fn format_relative_time(&self, created_at: &chrono::DateTime<chrono::Utc>) -> String {
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(*created_at);

        if diff.num_seconds() < 60 {
            "just now".to_string()
        } else if diff.num_minutes() < 60 {
            format!("{}m ago", diff.num_minutes())
        } else if diff.num_hours() < 24 {
            format!("{}h ago", diff.num_hours())
        } else if diff.num_days() < 7 {
            format!("{}d ago", diff.num_days())
        } else {
            created_at.format("%Y-%m-%d").to_string()
        }
    }

    fn render_entry(&self, entry: &HistoryEntry, cx: &mut Context<Self>) -> impl IntoElement {
        let method = HttpMethod::ALL
            .iter()
            .find(|m| m.as_str() == entry.method)
            .copied()
            .unwrap_or(HttpMethod::Get);
        let method_color = method.get_color(cx);

        let status_color = entry
            .status_code
            .map(|code| match code {
                100..=199 => cx.theme().blue,
                200..=299 => cx.theme().green,
                300..=399 => cx.theme().blue,
                400..=499 => cx.theme().yellow,
                500..=599 => cx.theme().red,
                _ => cx.theme().muted_foreground,
            })
            .unwrap_or(cx.theme().muted_foreground);

        let relative_time = self.format_relative_time(&entry.created_at);

        v_flex()
            .gap_1()
            .px_3()
            .py_2()
            .border_b_1()
            .border_color(cx.theme().border)
            .hover(|s| s.bg(cx.theme().secondary))
            .cursor_pointer()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .w(px(56.))
                            .font_family(cx.theme().mono_font_family.clone())
                            .font_bold()
                            .text_sm()
                            .text_color(method_color)
                            .child(entry.method.clone()),
                    )
                    .child(
                        div()
                            .flex_1()
                            .text_xs()
                            .font_family(cx.theme().mono_font_family.clone())
                            .text_color(status_color)
                            .when_some(entry.status_code, |this, code| {
                                this.child(format!("{}", code))
                            }),
                    )
                    .child(
                        div()
                            .text_xs()
                            .text_color(cx.theme().muted_foreground)
                            .child(relative_time),
                    ),
            )
            .child(
                div()
                    .text_xs()
                    .text_color(cx.theme().muted_foreground)
                    .font_family(cx.theme().mono_font_family.clone())
                    .overflow_hidden()
                    .child(entry.url.clone()),
            )
    }
}

impl Render for HistoryPanel {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let content = if !self.loaded {
            v_flex()
                .items_center()
                .justify_center()
                .h_full()
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child("Loading..."),
                )
                .into_any_element()
        } else if self.entries.is_empty() {
            v_flex()
                .items_center()
                .justify_center()
                .h_full()
                .child(
                    div()
                        .text_color(cx.theme().muted_foreground)
                        .child("No history yet"),
                )
                .into_any_element()
        } else {
            v_flex()
                .overflow_y_scrollbar()
                .children(
                    self.entries
                        .iter()
                        .map(|entry| div().child(self.render_entry(entry, cx))),
                )
                .into_any_element()
        };

        v_flex()
            .h_full()
            .child(div().flex_1().min_h_0().child(content))
            .child(
                h_flex()
                    .justify_end()
                    .items_center()
                    .px(px(4.))
                    .py(px(4.))
                    .border_t_1()
                    .border_color(cx.theme().border)
                    .child(
                        Button::new("clear-history")
                            .xsmall()
                            .ghost()
                            .icon(IconName::Trash)
                            .on_click(cx.listener(|this, _, _, cx| {
                                this.clear_history(cx);
                            })),
                    ),
            )
    }
}
