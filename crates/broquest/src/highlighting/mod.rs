use gpui::App;
use gpui_component::highlighter::{LanguageConfig, LanguageRegistry};

/// Register URL syntax highlighting for the application
pub fn register_url_highlighting(_cx: &mut App) {
    // Load highlight rules from the tree-sitter-url crate
    let url_highlights = include_str!("../../../tree-sitter-url/queries/highlights.scm");

    // Register custom URL language using tree-sitter-url parser
    LanguageRegistry::singleton().register(
        "url",
        &LanguageConfig::new(
            "url",
            tree_sitter_url::LANGUAGE.into(),
            vec![],
            url_highlights,
            "",
            "",
        ),
    );

    // Register XML language using tree-sitter-xml parser
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
