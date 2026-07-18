//! Shared keybinding helpers.

use gpui::Keystroke;

/// Parse a keystroke specification (e.g. `"secondary-s"`) for display in a
/// [`gpui_component::kbd::Kbd`]. On failure this logs and returns a default
/// keystroke rather than panicking, since a bad spec should never crash the UI.
pub fn parse_keystroke(spec: &str) -> Keystroke {
    Keystroke::parse(spec).unwrap_or_else(|e| {
        tracing::error!("Failed to parse keystroke '{}': {}", spec, e);
        Keystroke::default()
    })
}
