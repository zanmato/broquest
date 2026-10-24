//! The shortcuts users can rebind on the Keybindings settings page, with
//! their defaults, and the key bindings built from them.

use gpui::KeyBinding;

use crate::app::{NewScratchRequest, OpenSettings, ToggleCommandPalette};
use crate::collections::{
    COLLECTION_EDITOR_CONTEXT, GROUP_EDITOR_CONTEXT, collection_editor, group_editor,
};
use crate::requests::{
    CloseAllTabs, CloseOtherTabs, CloseTab, CloseTabsToTheLeft, CloseTabsToTheRight, CopyResponse,
    EDITOR_PANEL_CONTEXT, FilterResponse, FormatBody, NextTab, PrevTab, REQUEST_EDITOR_CONTEXT,
    Save, Send,
};
use crate::settings::Settings;

/// A keyboard shortcut the user can rebind from the settings UI.
pub struct BindingDef {
    /// Stable identifier persisted in settings (`keybinding.<action>`) and used
    /// to resolve the concrete actions when registering the keymap.
    pub action: &'static str,
    /// Human-readable label shown in the settings UI.
    pub label: &'static str,
    /// Heading the settings page lists the action under.
    pub group: &'static str,
    /// Built-in keystroke. An empty string means the action is unbound by
    /// default and only becomes active once the user assigns a shortcut.
    pub default: &'static str,
}

/// The groups of the settings page, in order.
pub const BINDING_GROUPS: &[&str] = &["Request", "Collection", "Tabs", "View"];

/// The set of actions exposed for rebinding. Quit is platform-fixed, and
/// moving around lists with the arrow keys is not a shortcut, so neither is
/// listed.
pub const CUSTOMIZABLE_BINDINGS: &[BindingDef] = &[
    BindingDef {
        action: "SendRequest",
        label: "Send request",
        group: "Request",
        default: "secondary-enter",
    },
    BindingDef {
        action: "Save",
        label: "Save request, collection or group",
        group: "Request",
        default: "secondary-s",
    },
    BindingDef {
        action: "CopyResponse",
        label: "Copy response",
        group: "Request",
        default: "secondary-shift-c",
    },
    BindingDef {
        action: "FormatBody",
        label: "Format request body",
        group: "Request",
        default: "shift-alt-f",
    },
    BindingDef {
        action: "FilterResponse",
        label: "Filter response with JSONPath",
        group: "Request",
        default: "secondary-shift-f",
    },
    BindingDef {
        action: "NewRequest",
        label: "New request",
        group: "Request",
        default: "secondary-n",
    },
    BindingDef {
        action: "ToggleDocsEdit",
        label: "Edit or preview collection docs",
        group: "Collection",
        default: "secondary-e",
    },
    BindingDef {
        action: "CloseTab",
        label: "Close tab",
        group: "Tabs",
        default: "secondary-w",
    },
    BindingDef {
        action: "CloseOtherTabs",
        label: "Close other tabs",
        group: "Tabs",
        default: "secondary-alt-w",
    },
    BindingDef {
        action: "CloseTabsToTheLeft",
        label: "Close tabs to the left",
        group: "Tabs",
        default: "",
    },
    BindingDef {
        action: "CloseTabsToTheRight",
        label: "Close tabs to the right",
        group: "Tabs",
        default: "",
    },
    BindingDef {
        action: "CloseAllTabs",
        label: "Close all tabs",
        group: "Tabs",
        default: "secondary-alt-shift-w",
    },
    BindingDef {
        action: "NextTab",
        label: "Next tab",
        group: "Tabs",
        default: "ctrl-tab",
    },
    BindingDef {
        action: "PrevTab",
        label: "Previous tab",
        group: "Tabs",
        default: "ctrl-shift-tab",
    },
    BindingDef {
        action: "ToggleCommandPalette",
        label: "Toggle command palette",
        group: "View",
        default: "secondary-k",
    },
    BindingDef {
        action: "OpenSettings",
        label: "Open settings",
        group: "View",
        default: "secondary-,",
    },
];

/// The built-in keystroke for an action, or `""` when the action is unknown.
pub fn default_keystroke(action: &str) -> &'static str {
    CUSTOMIZABLE_BINDINGS
        .iter()
        .find(|binding| binding.action == action)
        .map(|binding| binding.default)
        .unwrap_or("")
}

/// The keystroke currently in effect for an action: the user override if one is
/// set, otherwise the built-in default.
pub fn effective_keystroke(settings: &Settings, action: &str) -> String {
    settings
        .keybindings
        .get(action)
        .cloned()
        .unwrap_or_else(|| default_keystroke(action).to_string())
}

/// The chords of a shortcut, or `None` when one of them cannot be bound. An
/// empty string has no chords and means "unbound".
pub fn parse_keystrokes(keystrokes: &str) -> Option<Vec<gpui::Keystroke>> {
    keystrokes
        .split_whitespace()
        .map(|chord| gpui::Keystroke::parse(chord).ok().filter(is_bindable))
        .collect()
}

/// Whether a keystroke can serve as a shortcut. `Keystroke::parse` takes any
/// word for a key, so the key has to be one a keyboard has, and it needs a
/// modifier other than shift unless it is a function key: a bare letter as a
/// shortcut would swallow typing.
pub fn is_bindable(keystroke: &gpui::Keystroke) -> bool {
    let key = keystroke.key.as_str();
    let is_function_key = key
        .strip_prefix('f')
        .is_some_and(|number| !number.is_empty() && number.bytes().all(|b| b.is_ascii_digit()));
    let is_named_key = matches!(
        key,
        "enter"
            | "tab"
            | "space"
            | "escape"
            | "backspace"
            | "delete"
            | "insert"
            | "home"
            | "end"
            | "pageup"
            | "pagedown"
            | "up"
            | "down"
            | "left"
            | "right"
    );
    let modifiers = keystroke.modifiers;
    let has_modifier =
        modifiers.control || modifiers.alt || modifiers.platform || modifiers.function;
    is_function_key || (has_modifier && (is_named_key || key.chars().count() == 1))
}

/// True when every space-separated chord can be bound. An empty string is
/// valid and means "unbound".
pub fn keystrokes_valid(keystrokes: &str) -> bool {
    parse_keystrokes(keystrokes).is_some()
}

/// The key bindings an action's shortcut makes. Empty for an empty or invalid
/// shortcut or an unknown action, so callers never hit `KeyBinding::new`'s
/// panic on malformed input. Save makes three, one for each editor that saves.
fn key_bindings(action: &str, keystrokes: &str) -> Vec<KeyBinding> {
    if keystrokes.trim().is_empty() || !keystrokes_valid(keystrokes) {
        return Vec::new();
    }
    let k = keystrokes;
    let request = Some(REQUEST_EDITOR_CONTEXT);
    let panel = Some(EDITOR_PANEL_CONTEXT);
    match action {
        "SendRequest" => vec![KeyBinding::new(k, Send, request)],
        "Save" => vec![
            KeyBinding::new(k, Save, request),
            KeyBinding::new(k, collection_editor::Save, Some(COLLECTION_EDITOR_CONTEXT)),
            KeyBinding::new(k, group_editor::Save, Some(GROUP_EDITOR_CONTEXT)),
        ],
        "CopyResponse" => vec![KeyBinding::new(k, CopyResponse, request)],
        "FormatBody" => vec![KeyBinding::new(k, FormatBody, request)],
        "FilterResponse" => vec![KeyBinding::new(k, FilterResponse, request)],
        "NewRequest" => vec![KeyBinding::new(k, NewScratchRequest, None)],
        "ToggleDocsEdit" => vec![KeyBinding::new(
            k,
            collection_editor::ToggleDocsEdit,
            Some(COLLECTION_EDITOR_CONTEXT),
        )],
        "CloseTab" => vec![KeyBinding::new(k, CloseTab, panel)],
        "CloseOtherTabs" => vec![KeyBinding::new(k, CloseOtherTabs, panel)],
        "CloseTabsToTheLeft" => vec![KeyBinding::new(k, CloseTabsToTheLeft, panel)],
        "CloseTabsToTheRight" => vec![KeyBinding::new(k, CloseTabsToTheRight, panel)],
        "CloseAllTabs" => vec![KeyBinding::new(k, CloseAllTabs, panel)],
        "NextTab" => vec![KeyBinding::new(k, NextTab, None)],
        "PrevTab" => vec![KeyBinding::new(k, PrevTab, None)],
        "ToggleCommandPalette" => vec![KeyBinding::new(k, ToggleCommandPalette, None)],
        "OpenSettings" => vec![KeyBinding::new(k, OpenSettings, None)],
        _ => Vec::new(),
    }
}

/// The actions whose shortcut is the same keystroke as `action`'s. Shortcuts
/// limited to different key contexts can still meet (an editor sits inside
/// the tab panel), so any two with the same keystroke count as a conflict.
pub fn conflicting_actions(settings: &Settings, action: &str) -> Vec<&'static str> {
    let keystroke = normalized(&effective_keystroke(settings, action));
    if keystroke.is_empty() {
        return Vec::new();
    }
    CUSTOMIZABLE_BINDINGS
        .iter()
        .filter(|other| other.action != action)
        .filter(|other| normalized(&effective_keystroke(settings, other.action)) == keystroke)
        .map(|other| other.label)
        .collect()
}

/// Whether two shortcuts are the same keys, however they are spelled.
pub fn same_keystrokes(a: &str, b: &str) -> bool {
    normalized(a) == normalized(b)
}

/// One spelling per shortcut, so `Secondary-Enter` and a recorded `ctrl-enter`
/// compare equal where secondary is ctrl.
fn normalized(keystrokes: &str) -> String {
    match parse_keystrokes(keystrokes) {
        Some(chords) => chords
            .iter()
            .map(gpui::Keystroke::unparse)
            .collect::<Vec<_>>()
            .join(" "),
        None => keystrokes
            .split_whitespace()
            .map(str::to_lowercase)
            .collect::<Vec<_>>()
            .join(" "),
    }
}

/// Build the customizable key bindings from settings, layering user overrides
/// on top of the built-in defaults. Empty or invalid entries are skipped.
pub fn customizable_key_bindings(settings: &Settings) -> Vec<KeyBinding> {
    CUSTOMIZABLE_BINDINGS
        .iter()
        .flat_map(|def| key_bindings(def.action, &effective_keystroke(settings, def.action)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_binding_resolves_to_an_action() {
        for def in CUSTOMIZABLE_BINDINGS {
            assert!(
                !key_bindings(def.action, "ctrl-f12").is_empty(),
                "{} has no action",
                def.action
            );
            assert!(BINDING_GROUPS.contains(&def.group), "{}", def.action);
        }
    }

    #[test]
    fn defaults_do_not_conflict_and_overrides_are_flagged() {
        let mut settings = Settings::default();
        for def in CUSTOMIZABLE_BINDINGS {
            assert!(
                conflicting_actions(&settings, def.action).is_empty(),
                "{} conflicts by default",
                def.action
            );
        }
        settings
            .keybindings
            .insert("CloseAllTabs".to_string(), "Secondary-Enter".to_string());
        assert_eq!(
            conflicting_actions(&settings, "SendRequest"),
            ["Close all tabs"]
        );
    }

    #[test]
    fn only_real_keystrokes_can_be_bound() {
        for def in CUSTOMIZABLE_BINDINGS {
            assert!(keystrokes_valid(def.default), "{}", def.action);
        }
        for valid in [
            "",
            "f5",
            "ctrl-k",
            "shift-alt-f",
            "super-,",
            "ctrl-k ctrl-s",
        ] {
            assert!(keystrokes_valid(valid), "{valid}");
        }
        for invalid in [
            "ASDASDA",
            "ctrl-asdasda",
            "a",
            "shift-a",
            "enter",
            "ctrl-k x",
        ] {
            assert!(!keystrokes_valid(invalid), "{invalid}");
        }
    }

    #[test]
    fn shortcuts_compare_by_keys_not_spelling() {
        assert!(same_keystrokes("Secondary-Enter", "secondary-enter"));
        assert!(same_keystrokes("shift-alt-f", "alt-shift-f"));
        assert!(!same_keystrokes("ctrl-k", "ctrl-j"));
    }

    #[test]
    fn overrides_replace_defaults_and_empty_unbinds() {
        let mut settings = Settings::default();
        assert_eq!(effective_keystroke(&settings, "CloseTab"), "secondary-w");
        settings
            .keybindings
            .insert("CloseTab".to_string(), "ctrl-shift-x".to_string());
        assert_eq!(effective_keystroke(&settings, "CloseTab"), "ctrl-shift-x");

        // Save binds in three editors. Unbinding two shortcuts and giving one
        // an invalid value drops their bindings.
        let bound_by_default: usize = CUSTOMIZABLE_BINDINGS
            .iter()
            .filter(|def| !def.default.is_empty())
            .map(|def| if def.action == "Save" { 3 } else { 1 })
            .sum();
        assert_eq!(customizable_key_bindings(&settings).len(), bound_by_default);
        settings
            .keybindings
            .insert("SendRequest".to_string(), String::new());
        settings
            .keybindings
            .insert("CopyResponse".to_string(), "ctrl-a-b".to_string());
        settings
            .keybindings
            .insert("Save".to_string(), String::new());
        assert_eq!(
            customizable_key_bindings(&settings).len(),
            bound_by_default - 5
        );
    }
}
