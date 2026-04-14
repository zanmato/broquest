use gpui::{App, IntoElement, ParentElement, RenderOnce, Styled, Window, div, px};

use gpui_component::{ActiveTheme, StyledExt};

#[allow(unused)]
#[derive(IntoElement)]
pub struct TabBadge {
    count: usize,
}

#[allow(unused)]
impl TabBadge {
    pub fn new() -> Self {
        Self { count: 0 }
    }

    pub fn count(mut self, count: usize) -> Self {
        self.count = count;
        self
    }
}

impl RenderOnce for TabBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let (badge_bg, badge_color) = if cx.theme().is_dark() {
            (gpui::rgb(0xd8ebee), gpui::black())
        } else {
            (gpui::rgb(0x22d3ee), gpui::white())
        };

        div()
            .flex()
            .items_center()
            .justify_center()
            .bg(badge_bg)
            .w(px(16.))
            .h(px(16.))
            .text_color(badge_color)
            .rounded_full()
            .text_size(px(10.))
            .font_bold()
            .child(self.count.to_string())
    }
}
