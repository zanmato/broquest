use anyhow::Result;
use gpui::{Context, Task, Window};
use gpui_component::{RopeExt, input::CompletionProvider};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionResponse, CompletionTextEdit, TextEdit,
};
use ropey::Rope;
use std::rc::Rc;

/// The context type for script completions
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ScriptContext {
    /// Pre-request script context (has access to `req` and `bro`)
    PreRequest,
    /// Post-response script context (has access to `res` and `bro`)
    PostResponse,
}

/// Completion provider for script editors
pub struct ScriptCompletionProvider {
    /// The context (pre-request or post-response)
    pub context: ScriptContext,
}

impl ScriptCompletionProvider {
    /// Create a new completion provider for the given context
    pub fn new(context: ScriptContext) -> Rc<Self> {
        Rc::new(Self { context })
    }

    /// Get completion items for the `req` object (pre-request only)
    fn req_completions(
        &self,
        start_pos: lsp_types::Position,
        end_pos: lsp_types::Position,
    ) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "method".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("string".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "HTTP method (GET, POST, etc.)".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "method".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "url".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("string".to_string()),
                documentation: Some(lsp_types::Documentation::String("Request URL".to_string())),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "url".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "headers".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("object".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "HTTP headers as key-value object".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "headers".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "query".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("object".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "Query parameters as key-value object".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "query".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "body".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("string".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "Request body as string".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "body".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "path_params".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("array".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "Path parameters array".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "path_params".to_string(),
                })),
                ..Default::default()
            },
        ]
    }

    /// Get completion items for the `res` object (post-response only)
    fn res_completions(
        &self,
        start_pos: lsp_types::Position,
        end_pos: lsp_types::Position,
    ) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "status".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("number".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "HTTP status code".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "status".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "statusText".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("string".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "HTTP status text".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "statusText".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "headers".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("object".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "Response headers as key-value object".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "headers".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "body".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("string | object".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "Response body (parsed as JSON if content-type is application/json)"
                        .to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "body".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "latency".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("number".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "Request latency in milliseconds".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "latency".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "size".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("number".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "Response size in bytes".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "size".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "url".to_string(),
                kind: Some(CompletionItemKind::PROPERTY),
                detail: Some("string".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "Final URL (after redirects)".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "url".to_string(),
                })),
                ..Default::default()
            },
        ]
    }

    /// Get completion items for the `bro` object (both contexts)
    fn bro_completions(
        &self,
        start_pos: lsp_types::Position,
        end_pos: lsp_types::Position,
    ) -> Vec<CompletionItem> {
        vec![
            CompletionItem {
                label: "setEnvVar".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("(name: string, value: string) => void".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "Sets an environment variable".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "setEnvVar()".to_string(),
                })),
                ..Default::default()
            },
            CompletionItem {
                label: "getEnvVar".to_string(),
                kind: Some(CompletionItemKind::METHOD),
                detail: Some("(name: string) => string | undefined".to_string()),
                documentation: Some(lsp_types::Documentation::String(
                    "Gets an environment variable".to_string(),
                )),
                text_edit: Some(CompletionTextEdit::Edit(TextEdit {
                    range: lsp_types::Range {
                        start: start_pos,
                        end: end_pos,
                    },
                    new_text: "getEnvVar()".to_string(),
                })),
                ..Default::default()
            },
        ]
    }
}

impl CompletionProvider for ScriptCompletionProvider {
    fn completions(
        &self,
        rope: &Rope,
        offset: usize,
        _: lsp_types::CompletionContext,
        _: &mut Window,
        _cx: &mut Context<gpui_component::input::InputState>,
    ) -> Task<Result<CompletionResponse>> {
        // The dot is always at offset-1 since is_completion_trigger only returns true for "."
        let dot_pos = offset - 1;

        // Scan backwards from the dot to find the start of the identifier
        let mut obj_start = dot_pos;
        while obj_start > 0 {
            match rope.char_at(obj_start - 1) {
                Some(c) if c.is_ascii_alphanumeric() || c == '_' => {
                    obj_start -= 1;
                }
                _ => break,
            }
        }

        let obj_name = rope.slice(obj_start..dot_pos).to_string();

        // Get positions for text edits - replace everything after the dot
        let start_pos = rope.offset_to_position(dot_pos + 1);
        let end_pos = rope.offset_to_position(offset);

        let completions = if obj_name == "req" && self.context == ScriptContext::PreRequest {
            self.req_completions(start_pos, end_pos)
        } else if obj_name == "res" && self.context == ScriptContext::PostResponse {
            self.res_completions(start_pos, end_pos)
        } else if obj_name == "bro" {
            self.bro_completions(start_pos, end_pos)
        } else {
            vec![]
        };

        Task::ready(Ok(CompletionResponse::Array(completions)))
    }

    fn is_completion_trigger(
        &self,
        _offset: usize,
        new_text: &str,
        _: &mut Context<gpui_component::input::InputState>,
    ) -> bool {
        // Trigger on dot notation
        new_text == "."
    }
}
