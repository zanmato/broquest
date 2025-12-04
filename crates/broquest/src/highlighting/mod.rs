use gpui::App;
use gpui_component::highlighter::{LanguageConfig, LanguageRegistry};

/// Register syntax highlighting for the application
pub fn register_highlighting(_cx: &mut App) {
    // Register custom URL language using tree-sitter-url
    LanguageRegistry::singleton().register(
        "url",
        &LanguageConfig::new(
            "url",
            tree_sitter_url::LANGUAGE.into(),
            vec![],
            tree_sitter_url::HIGHLIGHTS_QUERY,
            "",
            "",
        ),
    );

    // Register XML language using tree-sitter-xml
    LanguageRegistry::singleton().register(
        "xml",
        &LanguageConfig::new(
            "xml",
            tree_sitter_xml::LANGUAGE_XML.into(),
            vec![],
            tree_sitter_xml::XML_HIGHLIGHT_QUERY,
            "",
            "",
        ),
    );
}
