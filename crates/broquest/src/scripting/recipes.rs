use std::ops::Range;

use gpui::{
    App, AppContext, Context, Entity, Focusable, HighlightStyle, InteractiveElement, IntoElement,
    ParentElement, Render, SharedString, StatefulInteractiveElement, Styled, StyledText,
    Subscription, WeakEntity, Window, div, prelude::FluentBuilder, px,
};
use gpui_kit::component::{
    ActiveTheme, Disableable, StyledExt, WindowExt,
    button::{Button, ButtonVariants},
    h_flex,
    highlighter::SyntaxHighlighter,
    input::{Input, InputEvent, InputState},
    scroll::ScrollableElement,
    v_flex,
};

use super::{ScriptEditor, completion::ScriptContext};

#[derive(Clone, Copy, Debug)]
pub(crate) struct ScriptRecipe {
    pub id: &'static str,
    pub title: &'static str,
    pub description: &'static str,
    pub keywords: &'static str,
    pub context: ScriptContext,
    pub code: &'static str,
}

pub(crate) const SCRIPT_RECIPES: &[ScriptRecipe] = &[
    ScriptRecipe {
        id: "body-sha256-authentication",
        title: "Body SHA-256 authentication",
        description: "Hash the request body with a secret and build a userid:signature header.",
        keywords: "hash digest secret authentication uppercase userid",
        context: ScriptContext::PreRequest,
        code: r#"const payload = req.body + bro.getEnvVar("secret");
const signature = crypto
  .createHash("sha256")
  .update(payload)
  .digest("hex")
  .toUpperCase();

req.headers.Authentication =
  bro.getEnvVar("userid") + ":" + signature;
"#,
    },
    ScriptRecipe {
        id: "timestamped-hmac-sha256",
        title: "Timestamped HMAC-SHA256 signature",
        description: "Sign the method, URL, body, and timestamp with a shared secret.",
        keywords: "hmac signature timestamp secret x-signature x-timestamp",
        context: ScriptContext::PreRequest,
        code: r#"const timestamp = Date.now().toString();
const payload = req.method + req.url + req.body + timestamp;
const signature = crypto
  .createHmac("sha256", bro.getEnvVar("secret"))
  .update(payload)
  .digest("hex");

req.headers["X-Timestamp"] = timestamp;
req.headers["X-Signature"] = signature;
"#,
    },
    ScriptRecipe {
        id: "correlation-id",
        title: "UUID correlation header",
        description: "Generate a new UUID for tracing this request across services.",
        keywords: "uuid random request id trace correlation header",
        context: ScriptContext::PreRequest,
        code: r#"req.headers["X-Correlation-ID"] = crypto.randomUUID();
"#,
    },
    ScriptRecipe {
        id: "update-json-body",
        title: "Update a JSON request body",
        description: "Parse the current JSON body, change a field, and serialize it again.",
        keywords: "json parse stringify mutate request body field",
        context: ScriptContext::PreRequest,
        code: r#"const body = JSON.parse(req.body || "{}");
body.example = "value";
req.body = JSON.stringify(body);
"#,
    },
    ScriptRecipe {
        id: "save-access-token",
        title: "Save an access token",
        description: "Read an access token from a JSON response and save it to the environment.",
        keywords: "oauth login json response environment variable token",
        context: ScriptContext::PostResponse,
        code: r#"bro.setEnvVar("access_token", res.body.access_token);
"#,
    },
    ScriptRecipe {
        id: "save-response-runtime-variable",
        title: "Save a response value",
        description: "Store a value from the JSON response as a request-chain runtime variable.",
        keywords: "json response runtime variable chain setvar",
        context: ScriptContext::PostResponse,
        code: r#"bro.setVar("resource_id", res.body.id);
"#,
    },
];

pub(crate) fn recipes_for(context: ScriptContext) -> impl Iterator<Item = &'static ScriptRecipe> {
    SCRIPT_RECIPES
        .iter()
        .filter(move |recipe| recipe.context == context)
}

fn recipe_matches(recipe: &ScriptRecipe, query: &str) -> bool {
    let query = query.trim().to_lowercase();
    query.is_empty()
        || recipe.title.to_lowercase().contains(&query)
        || recipe.description.to_lowercase().contains(&query)
        || recipe.keywords.to_lowercase().contains(&query)
}

struct RecipePreview {
    recipe: &'static ScriptRecipe,
    text: SharedString,
    highlights: Box<[(Range<usize>, HighlightStyle)]>,
}

impl RecipePreview {
    /// Parse and style a recipe once when the picker opens. Rendering then only
    /// clones the cached text and styles; it never invokes tree-sitter.
    fn new(recipe: &'static ScriptRecipe, cx: &App) -> Self {
        let mut highlighter = SyntaxHighlighter::new("javascript");
        let rope = ropey::Rope::from(recipe.code);
        highlighter.update(None, &rope, None);
        let highlights = highlighter
            .styles(&(0..recipe.code.len()), cx.theme().highlight_theme.as_ref())
            .into_boxed_slice();

        Self {
            recipe,
            text: SharedString::from(recipe.code),
            highlights,
        }
    }

    fn styled_text(&self) -> StyledText {
        StyledText::new(self.text.clone()).with_highlights(self.highlights.iter().cloned())
    }
}

pub(crate) struct RecipePicker {
    context: ScriptContext,
    previews: Vec<RecipePreview>,
    query_input: Entity<InputState>,
    selected_id: Option<&'static str>,
    target_input: Entity<gpui_kit::component::input::EditorState>,
    target_selection: Range<usize>,
    script_editor: WeakEntity<ScriptEditor>,
    _query_subscription: Subscription,
}

impl RecipePicker {
    pub(crate) fn new(
        context: ScriptContext,
        target_input: Entity<gpui_kit::component::input::EditorState>,
        target_selection: Range<usize>,
        script_editor: WeakEntity<ScriptEditor>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let query_input = cx.new(|cx| InputState::new(window, cx).placeholder("Search recipes..."));
        let previews = recipes_for(context)
            .map(|recipe| RecipePreview::new(recipe, cx))
            .collect();
        let selected_id = recipes_for(context).next().map(|recipe| recipe.id);
        let query_subscription = cx.subscribe(&query_input, |this, input, event, cx| {
            if matches!(event, InputEvent::Change) {
                let query = input.read(cx).value();
                this.selected_id = recipes_for(this.context)
                    .find(|recipe| recipe_matches(recipe, query.as_ref()))
                    .map(|recipe| recipe.id);
                cx.notify();
            }
        });

        let search_focus = query_input.focus_handle(cx);
        window.defer(cx, move |window, cx| search_focus.focus(window, cx));

        Self {
            context,
            previews,
            query_input,
            selected_id,
            target_input,
            target_selection,
            script_editor,
            _query_subscription: query_subscription,
        }
    }

    fn filtered_previews(&self, cx: &App) -> Vec<&RecipePreview> {
        let query = self.query_input.read(cx).value();
        self.previews
            .iter()
            .filter(|preview| recipe_matches(preview.recipe, query.as_ref()))
            .collect()
    }

    fn selected_preview(&self, cx: &App) -> Option<&RecipePreview> {
        let selected_id = self.selected_id?;
        self.filtered_previews(cx)
            .into_iter()
            .find(|preview| preview.recipe.id == selected_id)
    }

    fn insert_selected(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(preview) = self.selected_preview(cx) else {
            return;
        };
        let target_input = self.target_input.clone();
        let target_selection = self.target_selection.clone();
        let context = self.context;
        let code = preview.recipe.code;

        let _ = self.script_editor.update(cx, |editor, cx| {
            editor.insert_recipe(
                target_input.clone(),
                target_selection,
                code,
                context,
                window,
                cx,
            );
        });

        window.close_dialog(cx);
        let focus = target_input.focus_handle(cx);
        window.defer(cx, move |window, cx| focus.focus(window, cx));
    }
}

impl Render for RecipePicker {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let previews = self.filtered_previews(cx);
        let selected = self.selected_preview(cx);
        let has_selection = selected.is_some();
        let editor_background = cx
            .theme()
            .highlight_theme
            .style
            .editor_background
            .unwrap_or(cx.theme().background);
        let title = match self.context {
            ScriptContext::PreRequest => "Pre-request recipes",
            ScriptContext::PostResponse => "Post-response recipes",
        };

        v_flex()
            .w_full()
            .child(
                v_flex()
                    .gap_2()
                    .pb_4()
                    .child(div().text_lg().font_semibold().child(title))
                    .child(
                        div()
                            .text_sm()
                            .text_color(cx.theme().muted_foreground)
                            .child("Choose a starting point, then adapt the inserted JavaScript to your API."),
                    )
                    .child(Input::new(&self.query_input)),
            )
            .child(
                h_flex()
                    .items_stretch()
                    .h(px(390.))
                    .border_1()
                    .border_color(cx.theme().border)
                    .rounded(cx.theme().radius)
                    .overflow_hidden()
                    .child(
                        v_flex()
                            .w(px(300.))
                            .h_full()
                            .overflow_y_scrollbar()
                            .border_r_1()
                            .border_color(cx.theme().border)
                            .when(previews.is_empty(), |this| {
                                this.child(
                                    div()
                                        .p_4()
                                        .text_sm()
                                        .text_color(cx.theme().muted_foreground)
                                        .child("No matching recipes."),
                                )
                            })
                            .children(previews.into_iter().map(|preview| {
                                let recipe = preview.recipe;
                                let recipe_id = recipe.id;
                                let is_selected = self.selected_id == Some(recipe_id);
                                v_flex()
                                    .id(SharedString::from(format!("script-recipe-{recipe_id}")))
                                    .p_3()
                                    .gap_1()
                                    .cursor_pointer()
                                    .border_b_1()
                                    .border_color(cx.theme().border)
                                    .when(is_selected, |this| this.bg(cx.theme().accent))
                                    .hover(|this| this.bg(cx.theme().accent.opacity(0.65)))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.selected_id = Some(recipe_id);
                                        cx.notify();
                                    }))
                                    .child(div().text_sm().font_semibold().child(recipe.title))
                                    .child(
                                        div()
                                            .text_xs()
                                            .text_color(cx.theme().muted_foreground)
                                            .child(recipe.description),
                                    )
                            })),
                    )
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .p_4()
                            .gap_3()
                            .when_some(selected, |this, preview| {
                                this.child(div().font_semibold().child(preview.recipe.title))
                                .child(
                                    v_flex()
                                        .flex_1()
                                        .min_h_0()
                                        .overflow_y_scrollbar()
                                        .p_3()
                                        .rounded(cx.theme().radius)
                                        .bg(editor_background)
                                        .font_family(cx.theme().mono_font_family.clone())
                                        .text_sm()
                                        .child(preview.styled_text()),
                                )
                            }),
                    ),
            )
            .child(
                h_flex()
                    .justify_end()
                    .gap_2()
                    .pt_4()
                    .child(
                        Button::new("cancel-recipe")
                            .outline()
                            .label("Cancel")
                            .on_click(|_, window, cx| window.close_dialog(cx)),
                    )
                    .child(
                        Button::new("insert-recipe")
                            .primary()
                            .label("Insert Recipe")
                            .disabled(!has_selection)
                            .on_click(cx.listener(|this, _, window, cx| {
                                this.insert_selected(window, cx);
                            })),
                    ),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::{KeyValuePair, RequestData, ResponseData};
    use crate::scripting::{ScriptExecutionService, VariableStore};
    use std::collections::HashSet;
    use std::time::Duration;

    #[test]
    fn recipe_ids_are_unique_and_contexts_are_balanced() {
        let ids: HashSet<_> = SCRIPT_RECIPES.iter().map(|recipe| recipe.id).collect();
        assert_eq!(ids.len(), SCRIPT_RECIPES.len());
        assert_eq!(recipes_for(ScriptContext::PreRequest).count(), 4);
        assert_eq!(recipes_for(ScriptContext::PostResponse).count(), 2);
    }

    #[test]
    fn recipe_search_uses_title_description_and_keywords() {
        let recipe = &SCRIPT_RECIPES[0];
        assert!(recipe_matches(recipe, "SHA-256"));
        assert!(recipe_matches(recipe, "userid:signature"));
        assert!(recipe_matches(recipe, "DIGEST"));
        assert!(!recipe_matches(recipe, "oauth"));
    }

    #[test]
    fn bundled_recipes_execute_in_their_script_context() {
        let service = ScriptExecutionService::new().expect("create script service");

        for recipe in recipes_for(ScriptContext::PreRequest) {
            let store = VariableStore::new();
            store.set_env_var_str("secret", "test-secret");
            store.set_env_var_str("userid", "test-user");
            let mut request = RequestData {
                url: "https://api.example.com/items".to_string(),
                body: r#"{"original":true}"#.to_string(),
                ..Default::default()
            };

            service
                .execute_pre_request_script(recipe.code, &mut request, &store)
                .unwrap_or_else(|error| panic!("pre-request recipe {} failed: {error}", recipe.id));
        }

        for recipe in recipes_for(ScriptContext::PostResponse) {
            let store = VariableStore::new();
            let request = RequestData::default();
            let response = ResponseData {
                status_code: Some(200),
                latency: Some(Duration::from_millis(20)),
                headers: vec![KeyValuePair {
                    key: "Content-Type".to_string(),
                    value: "application/json".to_string(),
                    enabled: true,
                }],
                body: r#"{"access_token":"token","id":42,"data":{}}"#.to_string(),
                ..Default::default()
            };

            service
                .execute_post_response_script(recipe.code, &request, &response, &store)
                .unwrap_or_else(|error| {
                    panic!("post-response recipe {} failed: {error}", recipe.id)
                });
        }
    }
}
