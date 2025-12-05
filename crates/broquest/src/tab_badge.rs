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
        div()
            .flex()
            .items_center()
            .justify_center()
            .bg(cx.theme().red)
            .w(px(16.))
            .h(px(16.))
            .text_color(gpui::white())
            .rounded_full()
            .text_size(px(9.))
            .font_bold()
            .child(self.count.to_string())
    }
}
