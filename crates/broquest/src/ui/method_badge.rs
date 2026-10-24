//! A tinted pill showing an HTTP method, styled after the Mintlify docs
//! sidebar: the label in a Tailwind 400 shade (700 on light themes) over a
//! 20% tint of the 400 shade.

use gpui::{App, Hsla, IntoElement, ParentElement, RenderOnce, Rgba, Styled, Window, div, px, rgb};
use gpui_kit::component::{ActiveTheme, StyledExt, h_flex};

use crate::domain::HttpMethod;

/// Wide enough for the longest label, so request names line up.
/// The width of a fixed width badge, for icons that line up with badges.
pub(crate) const SLOT_WIDTH: f32 = 42.;

#[derive(IntoElement)]
pub struct MethodBadge {
    method: HttpMethod,
    fixed_width: bool,
}

impl MethodBadge {
    pub fn new(method: HttpMethod) -> Self {
        Self {
            method,
            fixed_width: false,
        }
    }

    /// Place the badge in a fixed width slot, so labels after it align.
    pub fn fixed_width(mut self) -> Self {
        self.fixed_width = true;
        self
    }
}

/// Tailwind (v3) 400 and 700 shades for each method.
fn shades(method: HttpMethod) -> (u32, u32) {
    match method {
        HttpMethod::Get => (0x4ade80, 0x15803d),     // green
        HttpMethod::Post => (0x60a5fa, 0x1d4ed8),    // blue
        HttpMethod::Put => (0xfacc15, 0xa16207),     // yellow
        HttpMethod::Delete => (0xf87171, 0xb91c1c),  // red
        HttpMethod::Patch => (0xfb923c, 0xc2410c),   // orange
        HttpMethod::Head => (0x22d3ee, 0x0e7490),    // cyan
        HttpMethod::Options => (0xc084fc, 0x7e22ce), // purple
    }
}

/// The badge's label colour, for places that show the method as plain text.
pub fn method_color(method: HttpMethod, cx: &App) -> Hsla {
    let (light, dark) = shades(method);
    rgb(if cx.theme().is_dark() { light } else { dark }).into()
}

fn method_tint(method: HttpMethod) -> Hsla {
    let tint: Rgba = rgb(shades(method).0);
    Hsla::from(tint).opacity(0.2)
}

impl RenderOnce for MethodBadge {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let badge = div()
            .flex_none()
            .px(px(4.))
            .py(px(1.))
            .rounded(px(6.))
            .bg(method_tint(self.method))
            .text_color(method_color(self.method, cx))
            .text_size(px(10.))
            .line_height(px(13.))
            .font_bold()
            .child(self.method.short_str());

        let slot = h_flex().flex_none().items_center();
        if self.fixed_width {
            slot.w(px(SLOT_WIDTH)).child(badge)
        } else {
            slot.child(badge)
        }
    }
}
