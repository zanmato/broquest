//! Mapping a VS Code theme onto gpui-component's theme format.
//!
//! Workbench colors map by role: VS Code's activity bar is the app shell, its
//! side bar the side panels, the editor the content surface. A theme that
//! leaves a role out gets it derived from the editor colors, the way VS Code
//! falls back to its defaults. Syntax colors are found by asking, for every
//! capture name gpui-component highlights, which `tokenColors` rule VS Code
//! would apply to a representative TextMate scope.

use std::collections::{BTreeMap, HashMap};

use serde_json::{Map, Value, json};

use crate::color::Rgba;
use crate::package::{Package, TokenRule, VsCodeTheme};

/// What the app highlights, so syntax colors come from the rules VS Code
/// would use for the same language.
#[derive(Debug, Clone, Copy, Default)]
pub struct ConvertOptions {
    /// Scopes tried before the generic ones, per capture name.
    pub preferred_scopes: &'static [(&'static str, &'static [&'static str])],
    /// The grammar's root scope, which a selector's parent part may name,
    /// as in `source.sql keyword.other`.
    pub root_scope: Option<&'static str>,
}

/// For an app whose editor holds SQL. VS Code's SQL grammar gives most
/// keywords `keyword.other.DML.sql`, and themes such as Tokyo Night color that
/// apart from `keyword.control`.
pub const SQL: ConvertOptions = ConvertOptions {
    preferred_scopes: &[
        ("keyword", &["keyword.other.DML.sql", "keyword.other.sql"]),
        (
            "function",
            &[
                "support.function.scalar.sql",
                "support.function.aggregate.sql",
            ],
        ),
        ("type", &["storage.type.sql"]),
        ("operator", &["keyword.operator.sql"]),
        ("string", &["string.quoted.single.sql"]),
        ("number", &["constant.numeric.sql"]),
        ("comment", &["comment.line.double-dash.sql"]),
        ("boolean", &["constant.language.boolean.sql"]),
    ],
    root_scope: Some("source.sql"),
};

/// Every capture name gpui-component's `SyntaxColors` reads, with TextMate
/// scopes that stand for it, most specific first. `comment_doc` is spelled
/// with an underscore because that is the key the field deserializes from.
const CAPTURE_SCOPES: &[(&str, &[&str])] = &[
    (
        "attribute",
        &[
            "entity.other.attribute-name.html",
            "entity.other.attribute-name",
        ],
    ),
    (
        "boolean",
        &["constant.language.boolean", "constant.language"],
    ),
    ("comment", &["comment.line", "comment"]),
    ("comment_doc", &["comment.block.documentation", "comment"]),
    (
        "constant",
        &[
            "variable.other.constant",
            "constant.other",
            "constant.language",
            "constant",
        ],
    ),
    (
        "constructor",
        &[
            "entity.name.function.constructor",
            "variable.function.constructor",
            "entity.name.type.class",
        ],
    ),
    ("embedded", &["meta.embedded", "source"]),
    ("emphasis", &["markup.italic.markdown", "markup.italic"]),
    ("emphasis.strong", &["markup.bold.markdown", "markup.bold"]),
    ("enum", &["entity.name.type.enum", "entity.name.type"]),
    (
        "function",
        &[
            "entity.name.function",
            "support.function",
            "variable.function",
            "meta.function-call",
        ],
    ),
    ("keyword", &["keyword.control", "keyword"]),
    ("label", &["entity.name.label", "entity.name.tag"]),
    (
        "link_text",
        &["string.other.link.title.markdown", "markup.underline.link"],
    ),
    ("link_uri", &["markup.underline.link"]),
    ("number", &["constant.numeric"]),
    ("operator", &["keyword.operator"]),
    (
        "preproc",
        &["meta.preprocessor", "keyword.control.directive"],
    ),
    (
        "property",
        &[
            "variable.other.property",
            "support.type.property-name",
            "variable.other.object.property",
        ],
    ),
    ("punctuation", &["punctuation"]),
    (
        "punctuation.bracket",
        &[
            "punctuation.section.brackets",
            "meta.brace.round",
            "punctuation.definition.block",
            "punctuation",
        ],
    ),
    (
        "punctuation.delimiter",
        &[
            "punctuation.separator",
            "punctuation.terminator",
            "punctuation",
        ],
    ),
    (
        "punctuation.list_marker",
        &[
            "punctuation.definition.list.begin.markdown",
            "punctuation.definition.list_item.markdown",
        ],
    ),
    (
        "punctuation.special",
        &[
            "punctuation.definition.template-expression",
            "punctuation.section.embedded",
        ],
    ),
    ("string", &["string.quoted", "string"]),
    ("string.escape", &["constant.character.escape"]),
    ("string.regex", &["string.regexp.js", "string.regexp"]),
    ("string.special", &["string.other", "string"]),
    (
        "string.special.symbol",
        &["constant.other.symbol.ruby", "constant.other.symbol"],
    ),
    ("tag", &["entity.name.tag"]),
    (
        "tag.doctype",
        &["meta.tag.metadata.doctype", "entity.name.tag"],
    ),
    (
        "text.code.span",
        &["markup.inline.raw.string.markdown", "markup.inline.raw"],
    ),
    (
        "text.literal",
        &[
            "markup.inline.raw.string.markdown",
            "markup.fenced_code.block.markdown",
            "markup.raw",
        ],
    ),
    (
        "title",
        &[
            "markup.heading.markdown",
            "entity.name.section.markdown",
            "markup.heading",
        ],
    ),
    (
        "type",
        &["entity.name.type", "support.type", "storage.type"],
    ),
    (
        "variable",
        &["variable.other.readwrite", "variable.other", "variable"],
    ),
    (
        "variable.special",
        &[
            "variable.language.this.js",
            "variable.language.self.rust",
            "variable.language",
        ],
    ),
    (
        "variant",
        &["variable.other.enummember", "constant.other.enum"],
    ),
];

/// A converted extension, ready to be written as a gpui-component theme file.
#[derive(Debug, Clone)]
pub struct ConvertedThemes {
    /// The theme set, as gpui-component's `ThemeSet` reads it.
    pub document: Value,
    /// Each theme's name and whether it is light, in manifest order.
    pub themes: Vec<ConvertedTheme>,
}

#[derive(Debug, Clone)]
pub struct ConvertedTheme {
    pub name: String,
    pub light: bool,
    /// A few colors that give the theme's look at a glance.
    pub swatches: Vec<Rgba>,
}

impl ConvertedThemes {
    pub fn to_json(&self) -> String {
        let mut json = serde_json::to_string_pretty(&self.document).unwrap_or_default();
        json.push('\n');
        json
    }
}

/// Convert every color theme `package` contributes. Themes that fail to load
/// are skipped with a warning, unless all of them do.
pub fn convert_package(
    package: &Package,
    options: &ConvertOptions,
) -> anyhow::Result<ConvertedThemes> {
    let mut themes = Vec::new();
    let mut entries = Vec::new();
    let mut last_error = None;
    for contribution in package.color_themes() {
        match package.load_theme(contribution) {
            Ok(theme) => {
                let (entry, converted) = convert_theme(&theme, options);
                entries.push(entry);
                themes.push(converted);
            }
            Err(error) => {
                tracing::warn!("Skipping theme {}: {error:#}", contribution.path);
                last_error = Some(error);
            }
        }
    }
    if entries.is_empty() {
        return Err(last_error
            .unwrap_or_else(|| anyhow::anyhow!("the extension contributes no color themes")));
    }

    let mut document = Map::new();
    document.insert(
        "$schema".into(),
        json!(
            "https://github.com/longbridge/gpui-component/raw/refs/heads/main/.theme-schema.json"
        ),
    );
    document.insert("name".into(), json!(package.display_name()));
    document.insert("author".into(), json!(package.manifest.publisher));
    if let Some(url) = package.manifest.url() {
        document.insert("url".into(), json!(url));
    }
    document.insert("themes".into(), Value::Array(entries));
    Ok(ConvertedThemes {
        document: Value::Object(document),
        themes,
    })
}

/// Workbench colors by VS Code key.
struct Workbench<'a>(&'a HashMap<String, Rgba>);

impl Workbench<'_> {
    /// The first of `keys` the theme sets. A fully transparent value counts
    /// as unset: Catppuccin and others write `#00000000` to hide a border
    /// VS Code would draw, which here should fall back rather than vanish.
    fn get(&self, keys: &[&str]) -> Option<Rgba> {
        keys.iter()
            .filter_map(|key| self.0.get(*key).copied())
            .find(|color| color.alpha > 0.05)
    }

    /// The first of `keys` that is set and opaque enough to read as a
    /// surface, composited over `under`.
    fn surface(&self, keys: &[&str], under: Rgba) -> Option<Rgba> {
        self.get(keys)
            .filter(|color| color.alpha > 0.5)
            .map(|color| color.over(under))
    }
}

/// One theme entry of a gpui-component theme set, and its summary.
pub fn convert_theme(theme: &VsCodeTheme, options: &ConvertOptions) -> (Value, ConvertedTheme) {
    let workbench = Workbench(&theme.colors);
    let syntax = Syntax {
        rules: &theme.token_colors,
        root_scope: options.root_scope,
    };

    let default_background = workbench.get(&["editor.background"]);
    let light = theme
        .light
        .unwrap_or_else(|| default_background.is_some_and(|background| !background.is_dark()));
    let fallback_background = if light {
        Rgba::WHITE
    } else {
        Rgba::opaque(0.118, 0.118, 0.118)
    };
    let editor = default_background
        .map(|background| background.over(fallback_background))
        .unwrap_or(fallback_background);
    let editor_text = workbench
        .get(&["editor.foreground"])
        .or_else(|| syntax.default_foreground())
        .unwrap_or(if light {
            Rgba::BLACK
        } else {
            Rgba::opaque(0.831, 0.831, 0.831)
        });
    let extreme = if light { Rgba::BLACK } else { Rgba::WHITE };

    let sidebar = workbench
        .surface(&["sideBar.background", "panel.background"], editor)
        .unwrap_or_else(|| editor.mix(extreme, 0.03));
    let text = workbench
        .get(&["foreground", "sideBar.foreground"])
        .unwrap_or(editor_text)
        .with_contrast(sidebar, editor_text, 4.5);
    // The shell. A loud activity bar (a few themes paint it in the accent)
    // would put the whole window in that color, so it must carry the text.
    let readable = |color: Rgba| text.contrast(color) >= 3.0;
    let shell = workbench
        .surface(&["activityBar.background"], editor)
        .filter(|color| readable(*color))
        .unwrap_or(sidebar);
    let title_bar = workbench
        .surface(&["titleBar.activeBackground"], editor)
        .filter(|color| readable(*color))
        .unwrap_or(shell);
    // VS Code's default status bar is blue, and many themes keep a colored
    // one, which Blanco-style status bars would show in plain text.
    let status_bar = workbench
        .surface(&["statusBar.background"], editor)
        .filter(|color| readable(*color))
        .unwrap_or(shell);
    let border = workbench
        .get(&[
            "editorGroup.border",
            "sideBar.border",
            "panel.border",
            "contrastBorder",
        ])
        .unwrap_or_else(|| sidebar.mix(editor_text, 0.12));
    // Muted text must set itself apart from text, which Catppuccin's
    // `descriptionForeground` does not.
    let muted_text = workbench
        .get(&["descriptionForeground"])
        .filter(|color| color.over(sidebar).contrast(text.over(sidebar)) > 1.15)
        .unwrap_or_else(|| text.mix(sidebar, 0.25))
        .with_contrast(sidebar, editor_text, 3.0);

    let primary = workbench
        .get(&[
            "button.background",
            "activityBarBadge.background",
            "progressBar.background",
            "textLink.foreground",
        ])
        .map(|color| color.over(sidebar))
        .unwrap_or_else(|| Rgba::parse("#007acc").unwrap_or(Rgba::BLACK));
    let primary_text = workbench
        .get(&["button.foreground"])
        .filter(|color| color.contrast(primary) >= 3.0)
        .unwrap_or_else(|| text_on(primary, [shell, editor_text]));
    let primary_hover = workbench
        .get(&["button.hoverBackground"])
        .map(|color| color.over(sidebar))
        .unwrap_or(primary);
    let ring = workbench
        .get(&["focusBorder"])
        .filter(|color| color.alpha >= 0.5)
        .unwrap_or(primary);

    let list_active = workbench
        .get(&[
            "list.activeSelectionBackground",
            "list.inactiveSelectionBackground",
        ])
        .unwrap_or_else(|| sidebar.mix(editor_text, 0.12));
    let list_active_text = workbench
        .get(&["list.activeSelectionForeground"])
        .unwrap_or(editor_text);
    let list_inactive = workbench
        .get(&["list.inactiveSelectionBackground"])
        .unwrap_or(list_active);
    let list_hover = workbench
        .get(&["list.hoverBackground"])
        .unwrap_or_else(|| sidebar.mix(editor_text, 0.06));

    let secondary = workbench
        .get(&["button.secondaryBackground"])
        .map(|color| color.over(sidebar))
        .unwrap_or_else(|| sidebar.mix(editor_text, 0.1));
    let secondary_text = workbench
        .get(&["button.secondaryForeground"])
        .filter(|color| color.contrast(secondary) >= 3.0)
        .unwrap_or_else(|| editor_text.with_contrast(secondary, extreme, 4.5));
    let secondary_hover = workbench
        .get(&["button.secondaryHoverBackground"])
        .map(|color| color.over(sidebar))
        .unwrap_or_else(|| secondary.mix(editor_text, 0.08));

    let input_background = workbench
        .surface(&["input.background", "dropdown.background"], sidebar)
        .unwrap_or(sidebar);
    let input_border = workbench
        .get(&["input.border", "dropdown.border"])
        .unwrap_or(border)
        .with_contrast(input_background, editor_text, 1.3);
    let popover = workbench
        .surface(
            &[
                "editorWidget.background",
                "menu.background",
                "dropdown.background",
            ],
            editor,
        )
        .unwrap_or(sidebar);
    let popover_text = workbench
        .get(&["editorWidget.foreground", "menu.foreground"])
        .unwrap_or(text)
        .with_contrast(popover, editor_text, 4.5);

    let tab_bar = workbench
        .surface(&["editorGroupHeader.tabsBackground"], editor)
        .unwrap_or(shell);
    let tab = workbench
        .surface(&["tab.inactiveBackground"], tab_bar)
        .unwrap_or(tab_bar);
    // Tokyo Night and others tell the active tab apart by an underline,
    // which gpui-component tabs do not draw, so it takes the editor's surface.
    let tab_active = workbench
        .surface(&["tab.activeBackground"], tab_bar)
        .filter(|color| color.to_hex() != tab.to_hex())
        .unwrap_or(editor);
    let tab_text = workbench
        .get(&["tab.inactiveForeground"])
        .unwrap_or(muted_text)
        .with_contrast(tab, editor_text, 3.0);
    let tab_active_text = workbench
        .get(&["tab.activeForeground"])
        .unwrap_or(editor_text)
        .with_contrast(tab_active, editor_text, 4.5);

    let link = workbench.get(&["textLink.foreground"]).unwrap_or(primary);
    let link_active = workbench
        .get(&["textLink.activeForeground"])
        .unwrap_or(link);
    let selection = workbench
        .get(&["editor.selectionBackground", "selection.background"])
        .unwrap_or_else(|| primary.with_alpha(0.4));
    let line_highlight = workbench
        .get(&["editor.lineHighlightBackground"])
        .unwrap_or_else(|| editor_text.with_alpha(0.07));

    let ansi = |keys: &[&str], fallback: &str| {
        workbench
            .get(keys)
            .unwrap_or_else(|| Rgba::parse(fallback).unwrap_or(editor_text))
    };
    let red = ansi(
        &["terminal.ansiRed"],
        if light { "#cd3131" } else { "#f14c4c" },
    );
    let green = ansi(
        &["terminal.ansiGreen"],
        if light { "#00bc00" } else { "#23d18b" },
    );
    let yellow = ansi(
        &["terminal.ansiYellow"],
        if light { "#949800" } else { "#f5f543" },
    );
    let blue = ansi(
        &["terminal.ansiBlue"],
        if light { "#0451a5" } else { "#3b8eea" },
    );
    let magenta = ansi(
        &["terminal.ansiMagenta"],
        if light { "#bc05bc" } else { "#d670d6" },
    );
    let cyan = ansi(
        &["terminal.ansiCyan"],
        if light { "#0598bc" } else { "#29b8db" },
    );

    let danger = workbench.get(&["editorError.foreground"]).unwrap_or(red);
    let warning = workbench
        .get(&["editorWarning.foreground"])
        .unwrap_or(yellow);
    let info = workbench.get(&["editorInfo.foreground"]).unwrap_or(blue);
    let success = workbench
        .get(&["gitDecoration.addedResourceForeground"])
        .filter(|color| color.contrast(editor) >= 3.0)
        .unwrap_or(green);
    let hint = workbench.get(&["editorHint.foreground"]).unwrap_or(info);
    let chart = |key: &str, fallback: Rgba| workbench.get(&[key]).unwrap_or(fallback);

    let scrollbar_thumb = workbench
        .get(&["scrollbarSlider.background"])
        .unwrap_or_else(|| editor_text.with_alpha(0.2));
    // Some themes make the hovered thumb fainter than the resting one, which
    // reads as the scrollbar fading out under the pointer.
    let scrollbar_hover = [
        workbench.get(&["scrollbarSlider.hoverBackground"]),
        workbench.get(&["scrollbarSlider.activeBackground"]),
        Some(scrollbar_thumb.with_alpha((scrollbar_thumb.alpha * 1.5).min(1.0))),
    ]
    .into_iter()
    .flatten()
    .max_by(|first, second| first.contrast(editor).total_cmp(&second.contrast(editor)))
    .unwrap_or(scrollbar_thumb);
    let row_border = workbench
        .get(&[
            "editorIndentGuide.background1",
            "editorIndentGuide.background",
        ])
        .unwrap_or(border);
    let switch = sidebar.mix(editor_text, 0.25);
    let overlay = if light {
        editor_text.with_alpha(0.4)
    } else {
        Rgba::BLACK.with_alpha(0.6)
    };

    let mut colors: BTreeMap<&str, Rgba> = BTreeMap::new();
    let mut set = |key: &'static str, color: Rgba| {
        colors.insert(key, color);
    };
    set("background", shell);
    set("foreground", text);
    set("border", border);
    set(
        "window.border",
        workbench.get(&["window.activeBorder"]).unwrap_or(shell),
    );
    set("title_bar.background", title_bar);
    set(
        "title_bar.border",
        workbench.get(&["titleBar.border"]).unwrap_or(title_bar),
    );
    set("status_bar.background", status_bar);
    set(
        "status_bar.border",
        workbench.get(&["statusBar.border"]).unwrap_or(status_bar),
    );
    set("tab_bar.background", tab_bar);
    set("tab_bar.segmented.background", input_background);
    set("tab.background", tab);
    set("tab.foreground", tab_text);
    set("tab.active.background", tab_active);
    set("tab.active.foreground", tab_active_text);
    set("popover.background", popover);
    set("popover.foreground", popover_text);
    set("accordion.background", editor);
    set("group_box.background", sidebar);
    set("group_box.foreground", text);
    set("group_box.title.foreground", text);
    set(
        "description_list.label.background",
        list_inactive.over(sidebar),
    );
    set("description_list.label.foreground", muted_text);
    set("overlay", overlay);
    set("sidebar.background", sidebar);
    set(
        "sidebar.foreground",
        workbench
            .get(&["sideBar.foreground"])
            .unwrap_or(text)
            .with_contrast(sidebar, editor_text, 4.5),
    );
    set(
        "sidebar.border",
        workbench.get(&["sideBar.border"]).unwrap_or(border),
    );
    set("sidebar.accent.background", list_active);
    set("sidebar.accent.foreground", list_active_text);
    set("sidebar.primary.background", primary);
    set("sidebar.primary.foreground", primary_text);
    set("accent.background", list_active);
    set("accent.foreground", list_active_text);
    set("muted.background", list_inactive);
    set("muted.foreground", muted_text);
    set("primary.background", primary);
    set("primary.foreground", primary_text);
    set("primary.hover.background", primary_hover);
    set("primary.active.background", primary.with_alpha(0.6));
    set("secondary.background", secondary);
    set("secondary.foreground", secondary_text);
    set("secondary.hover.background", secondary_hover);
    set("secondary.active.background", secondary_hover);
    set("ring", ring);
    set(
        "caret",
        workbench
            .get(&["editorCursor.foreground"])
            .unwrap_or(editor_text),
    );
    set("selection.background", selection);
    set("input.border", input_border);
    set("drag.border", primary);
    set(
        "drop_target.background",
        workbench
            .get(&["list.dropBackground"])
            .map(|color| color.with_alpha(color.alpha.min(0.5)))
            .unwrap_or_else(|| primary.with_alpha(0.2)),
    );
    set("link", link);
    set("link.hover", link_active);
    set("link.active", link_active);
    set("button.background", secondary);
    set("button.foreground", secondary_text);
    set("button.hover.background", secondary_hover);
    set(
        "button.active.background",
        secondary_hover.mix(editor_text, 0.08),
    );
    set("scrollbar.background", sidebar.with_alpha(0.0));
    set("scrollbar.thumb.background", scrollbar_thumb);
    set("scrollbar.thumb.hover.background", scrollbar_hover);
    set("skeleton.background", secondary);
    set("switch.background", switch);
    set("switch.thumb.background", editor);
    set("slider.background", switch);
    set("slider.thumb.background", primary);
    set(
        "progress.bar.background",
        workbench
            .get(&["progressBar.background"])
            .unwrap_or(primary),
    );
    set("list.background", sidebar.with_alpha(0.0));
    set("list.active.background", list_active);
    set(
        "list.active.border",
        workbench.get(&["list.focusOutline"]).unwrap_or(primary),
    );
    set("list.even.background", editor_text.with_alpha(0.025));
    set("list.head.background", editor_text.with_alpha(0.03));
    set("list.hover.background", list_hover);
    set("table.background", editor);
    set("table.head.background", sidebar);
    set("table.head.foreground", muted_text);
    set("table.foot.background", sidebar);
    set("table.foot.foreground", muted_text);
    set("table.even.background", editor_text.with_alpha(0.025));
    set("table.hover.background", line_highlight);
    set("table.active.background", selection);
    set("table.active.border", primary);
    set("table.row.border", row_border);
    for ([background, hover, active, foreground], color) in [
        (
            [
                "danger.background",
                "danger.hover.background",
                "danger.active.background",
                "danger.foreground",
            ],
            danger,
        ),
        (
            [
                "success.background",
                "success.hover.background",
                "success.active.background",
                "success.foreground",
            ],
            success,
        ),
        (
            [
                "warning.background",
                "warning.hover.background",
                "warning.active.background",
                "warning.foreground",
            ],
            warning,
        ),
        (
            [
                "info.background",
                "info.hover.background",
                "info.active.background",
                "info.foreground",
            ],
            info,
        ),
    ] {
        set(background, color);
        set(hover, color);
        set(active, color.mix(editor, 0.2));
        set(foreground, text_on(color, [shell, editor_text]));
    }
    for (key, light_key, color) in [
        ("base.blue", "base.blue.light", blue),
        ("base.cyan", "base.cyan.light", cyan),
        ("base.green", "base.green.light", green),
        ("base.magenta", "base.magenta.light", magenta),
        ("base.red", "base.red.light", red),
        ("base.yellow", "base.yellow.light", yellow),
    ] {
        set(key, color);
        set(light_key, color.with_alpha(0.4));
    }
    set("chart.1", chart("charts.blue", blue));
    set("chart.2", chart("charts.green", green));
    set("chart.3", chart("charts.yellow", yellow));
    set("chart.4", chart("charts.orange", red.mix(yellow, 0.5)));
    set("chart.5", chart("charts.purple", magenta));
    set("chart.bullish", chart("charts.green", green));
    set("chart.bearish", chart("charts.red", red));

    let mut highlight = Map::new();
    // The status colors sit directly in `highlight`: gpui-component flattens
    // them into it, and a nested `status` object is ignored.
    for (name, color) in [
        ("error", danger),
        ("warning", warning),
        ("info", info),
        ("success", success),
        ("hint", hint),
    ] {
        highlight.insert(name.into(), json!(color.to_hex()));
        highlight.insert(
            format!("{name}.background"),
            json!(color.with_alpha(0.15).to_hex()),
        );
        highlight.insert(
            format!("{name}.border"),
            json!(color.with_alpha(0.4).to_hex()),
        );
    }

    let mut syntax_styles = Map::new();
    for (capture, generic) in CAPTURE_SCOPES {
        let preferred = options
            .preferred_scopes
            .iter()
            .filter(|(name, _)| name == capture)
            .flat_map(|(_, scopes)| scopes.iter());
        let probes: Vec<&str> = preferred.chain(generic.iter()).copied().collect();
        if let Some(style) = syntax.style(&probes) {
            syntax_styles.insert((*capture).into(), style);
        }
    }
    let hint_text = workbench
        .get(&["editorInlayHint.foreground", "editorCodeLens.foreground"])
        .unwrap_or_else(|| editor_text.mix(editor, 0.4));
    syntax_styles.insert(
        "hint".into(),
        json!({"color": hint_text.to_hex(), "font_style": "italic"}),
    );
    let ghost_text = workbench
        .get(&["editorGhostText.foreground"])
        .unwrap_or_else(|| editor_text.mix(editor, 0.5));
    syntax_styles.insert("predictive".into(), json!({"color": ghost_text.to_hex()}));

    let line_number = workbench
        .get(&["editorLineNumber.foreground"])
        .unwrap_or_else(|| editor_text.mix(editor, 0.5))
        .with_contrast(editor, editor_text, 1.8);
    let invisible = workbench
        .get(&["editorWhitespace.foreground"])
        .unwrap_or_else(|| editor_text.with_alpha(0.3))
        .with_contrast(editor, editor_text, 1.5);

    highlight.insert("editor.background".into(), json!(editor.to_hex()));
    highlight.insert("editor.foreground".into(), json!(editor_text.to_hex()));
    highlight.insert(
        "editor.gutter.background".into(),
        json!(
            workbench
                .surface(&["editorGutter.background"], editor)
                .unwrap_or(editor)
                .to_hex()
        ),
    );
    highlight.insert(
        "editor.active_line.background".into(),
        json!(line_highlight.to_hex()),
    );
    highlight.insert("editor.line_number".into(), json!(line_number.to_hex()));
    highlight.insert(
        "editor.active_line_number".into(),
        json!(
            workbench
                .get(&["editorLineNumber.activeForeground"])
                .unwrap_or(editor_text)
                .to_hex()
        ),
    );
    highlight.insert("editor.invisible".into(), json!(invisible.to_hex()));
    highlight.insert("syntax".into(), Value::Object(syntax_styles));

    let colors: Map<String, Value> = colors
        .into_iter()
        .map(|(key, color)| (key.to_string(), json!(color.to_hex())))
        .collect();

    let mut entry = Map::new();
    entry.insert("name".into(), json!(theme.name));
    entry.insert("mode".into(), json!(if light { "light" } else { "dark" }));
    entry.insert("colors".into(), Value::Object(colors));
    entry.insert("highlight".into(), Value::Object(highlight));

    let swatch = |capture: &str| {
        let probes: Vec<&str> = CAPTURE_SCOPES
            .iter()
            .filter(|(name, _)| *name == capture)
            .flat_map(|(_, scopes)| scopes.iter().copied())
            .collect();
        syntax.foreground(&probes)
    };
    let swatches = [
        Some(shell),
        Some(sidebar),
        Some(editor),
        Some(editor_text),
        Some(primary),
        swatch("keyword"),
        swatch("function"),
        swatch("string"),
        swatch("number"),
        swatch("type"),
    ]
    .into_iter()
    .flatten()
    .collect();

    (
        Value::Object(entry),
        ConvertedTheme {
            name: theme.name.clone(),
            light,
            swatches,
        },
    )
}

/// Text for a saturated `fill`: whichever of the theme's own `candidates`
/// reads best, as the theme's badges do, else black or white.
fn text_on(fill: Rgba, candidates: [Rgba; 2]) -> Rgba {
    candidates
        .into_iter()
        .max_by(|first, second| first.contrast(fill).total_cmp(&second.contrast(fill)))
        .filter(|text| text.contrast(fill) >= 4.0)
        .unwrap_or_else(|| fill.readable_text())
}

/// TextMate rule matching, reduced to what a theme needs: a selector matches
/// a scope when its last part is a dot-wise prefix of it, and any parent
/// parts name the grammar's root scope. The deeper match wins, then the later
/// rule, as in VS Code.
struct Syntax<'a> {
    rules: &'a [TokenRule],
    root_scope: Option<&'static str>,
}

impl Syntax<'_> {
    /// The scope-less rule's foreground: the editor text color of themes
    /// written as TextMate themes.
    fn default_foreground(&self) -> Option<Rgba> {
        self.rules
            .iter()
            .filter(|rule| rule.selectors.iter().any(Vec::is_empty))
            .find_map(|rule| rule.foreground)
    }

    fn score(&self, selector: &[String], scope: &str) -> Option<(usize, usize)> {
        let (last, parents) = selector.split_last()?;
        let matches = |part: &str, scope: &str| {
            scope == part
                || scope
                    .strip_prefix(part)
                    .is_some_and(|rest| rest.starts_with('.'))
        };
        if !matches(last, scope) {
            return None;
        }
        match parents {
            [] => Some((last.split('.').count(), 0)),
            [parent] if self.root_scope.is_some_and(|root| matches(parent, root)) => {
                Some((last.split('.').count(), 1))
            }
            _ => None,
        }
    }

    /// The rule with a value for `field` that wins for `scope`.
    fn best<T>(&self, scope: &str, field: impl Fn(&TokenRule) -> Option<T>) -> Option<T> {
        let mut best: Option<((usize, usize, usize), T)> = None;
        for (index, rule) in self.rules.iter().enumerate() {
            let Some(value) = field(rule) else { continue };
            let score = rule
                .selectors
                .iter()
                .filter_map(|selector| self.score(selector, scope))
                .max();
            if let Some((depth, parents)) = score {
                let rank = (depth, parents, index);
                if best.as_ref().is_none_or(|(current, _)| rank >= *current) {
                    best = Some((rank, value));
                }
            }
        }
        best.map(|(_, value)| value)
    }

    fn foreground(&self, probes: &[&str]) -> Option<Rgba> {
        probes
            .iter()
            .find_map(|scope| self.best(scope, |rule| rule.foreground))
    }

    /// The style for the first of `probes` some rule colors.
    fn style(&self, probes: &[&str]) -> Option<Value> {
        let (scope, color) = probes.iter().find_map(|scope| {
            self.best(scope, |rule| rule.foreground)
                .map(|color| (*scope, color))
        })?;
        let mut style = Map::new();
        style.insert("color".into(), json!(color.to_hex()));
        let font_style = self.best(scope, |rule| rule.font_style.clone());
        if let Some(font_style) = font_style {
            let words: Vec<&str> = font_style.split_whitespace().collect();
            if words.contains(&"italic") {
                style.insert("font_style".into(), json!("italic"));
            }
            if words.contains(&"bold") {
                style.insert("font_weight".into(), json!(700));
            }
        }
        Some(Value::Object(style))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Paths in `written` that did not come back from gpui-component's own
    /// types: keys its theme loader ignores. Each app pins its own
    /// gpui-component, whose key names drift (`chart.bullish` was once
    /// `chart_bullish`), so this checks the copy against the app's schema.
    fn ignored_keys(written: &Value, read: &Value, path: &str, out: &mut Vec<String>) {
        if let Value::Object(written) = written {
            for (key, value) in written {
                if key.starts_with('$') {
                    continue;
                }
                let child = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                match read.get(key) {
                    None | Some(Value::Null) => out.push(child),
                    Some(read) => ignored_keys(value, read, &child, out),
                }
            }
        }
    }

    #[test]
    fn every_converted_key_is_one_gpui_component_reads() {
        let mut colors = HashMap::new();
        colors.insert(
            "editor.background".to_string(),
            Rgba::parse("#1a1b26").unwrap(),
        );
        colors.insert(
            "editor.foreground".to_string(),
            Rgba::parse("#a9b1d6").unwrap(),
        );
        // A rule for every capture, so every syntax key is written.
        let token_colors = CAPTURE_SCOPES
            .iter()
            .map(|(_, scopes)| rule(scopes[0], "#bb9af7", Some("italic bold")))
            .collect();
        let theme = VsCodeTheme {
            name: "Every Key".into(),
            light: Some(false),
            colors,
            token_colors,
        };
        let (entry, _) = convert_theme(&theme, &ConvertOptions::default());
        let config: gpui_component::ThemeConfig = serde_json::from_value(entry.clone()).unwrap();
        let read = serde_json::to_value(&config).unwrap();
        let mut ignored = Vec::new();
        ignored_keys(&entry, &read, "", &mut ignored);
        assert!(
            ignored.is_empty(),
            "keys the theme loader ignores: {ignored:?}"
        );
    }

    fn rule(scopes: &str, foreground: &str, font_style: Option<&str>) -> TokenRule {
        TokenRule {
            selectors: scopes
                .split(',')
                .map(|selector| selector.split_whitespace().map(str::to_string).collect())
                .collect(),
            foreground: Rgba::parse(foreground),
            font_style: font_style.map(str::to_string),
        }
    }

    #[test]
    fn the_deepest_then_latest_rule_wins() {
        let rules = vec![
            rule("comment", "#111111", Some("italic")),
            rule("keyword", "#222222", None),
            rule("keyword.control", "#333333", None),
            rule("keyword", "#444444", None),
            rule("source.sql keyword.other", "#555555", None),
            rule("source.css keyword.other", "#666666", None),
            rule("comment", "#777777", None),
        ];
        let syntax = Syntax {
            rules: &rules,
            root_scope: Some("source.sql"),
        };
        assert_eq!(
            syntax
                .foreground(&["keyword.control.flow"])
                .unwrap()
                .to_hex(),
            "#333333"
        );
        assert_eq!(syntax.foreground(&["keyword"]).unwrap().to_hex(), "#444444");
        assert_eq!(
            syntax
                .foreground(&["keyword.other.DML.sql"])
                .unwrap()
                .to_hex(),
            "#555555"
        );
        // `keywords` is not under `keyword`.
        assert!(syntax.foreground(&["keywords"]).is_none());
        // Color and font style resolve on their own, as in VS Code.
        assert_eq!(
            syntax.style(&["comment.line"]).unwrap(),
            json!({"color": "#777777", "font_style": "italic"})
        );
    }

    #[test]
    fn missing_workbench_colors_are_derived_and_readable() {
        let mut colors = HashMap::new();
        colors.insert(
            "editor.background".to_string(),
            Rgba::parse("#fafafa").unwrap(),
        );
        colors.insert(
            "editor.foreground".to_string(),
            Rgba::parse("#333333").unwrap(),
        );
        // A loud status bar and a foreground too faint to read.
        colors.insert(
            "statusBar.background".to_string(),
            Rgba::parse("#007acc").unwrap(),
        );
        colors.insert("foreground".to_string(), Rgba::parse("#e0e0e0").unwrap());
        let theme = VsCodeTheme {
            name: "Plain".into(),
            light: None,
            colors,
            token_colors: vec![rule("string", "#a31515", None)],
        };
        let (entry, summary) = convert_theme(&theme, &ConvertOptions::default());
        assert!(summary.light);
        assert_eq!(entry["mode"], "light");
        let color = |key: &str| Rgba::parse(entry["colors"][key].as_str().unwrap()).unwrap();
        assert!(color("foreground").contrast(color("sidebar.background")) >= 4.5);
        assert_ne!(entry["colors"]["status_bar.background"], "#007acc");
        assert_eq!(entry["highlight"]["syntax"]["string"]["color"], "#a31515");
        assert!(entry["highlight"]["syntax"].get("keyword").is_none());
    }
}
