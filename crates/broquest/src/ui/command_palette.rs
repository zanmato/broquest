use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, KeyBinding, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Subscription, WeakFocusHandle, Window, actions,
    anchored, deferred, div, point, prelude::FluentBuilder, px,
};
use gpui_component::{
    ActiveTheme, IndexPath, Selectable,
    input::{Input, InputEvent, InputState},
    list::{List, ListDelegate, ListEvent, ListState},
    v_flex, window_paddings,
};

const COMMAND_PALETTE_CONTEXT: &str = "CommandPalette";

actions!(
    command_palette,
    [
        CancelCommandPalette,
        SelectNextCommand,
        SelectPrevCommand,
        ConfirmCommand
    ]
);

#[derive(Clone, Debug)]
pub struct CommandItem {
    pub label: String,
    pub group: String,
    pub action_type: CommandType,
}

#[derive(Clone, Debug)]
pub enum CommandType {
    SendRequest,
    NewCollection,
    OpenCollection,
    NewRequest,
    SaveRequest,
    CloseTab,
    NextTab,
    PrevTab,
    OpenSettings,
    OpenRequest {
        collection_path: String,
        request_name: String,
    },
}

struct ScoredItem {
    command: CommandItem,
    score: i64,
}

pub struct CommandPalette {
    focus_handle: FocusHandle,
    query_input: Entity<InputState>,
    list_state: Entity<ListState<CommandPaletteDelegate>>,
    visible: bool,
    previous_focus: Option<WeakFocusHandle>,
    _list_subscription: Subscription,
    _query_subscription: Subscription,
}

impl EventEmitter<()> for CommandPalette {}

impl Focusable for CommandPalette {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl CommandPalette {
    pub fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let query_input =
            cx.new(|cx| InputState::new(window, cx).placeholder("Type a command or search..."));

        let delegate = CommandPaletteDelegate::new(cx);
        let list_state = cx.new(|cx| ListState::new(delegate, window, cx).searchable(false));

        let list_entity = list_state.clone();
        let list_subscription = cx.subscribe_in(
            &list_state,
            window,
            move |palette, _, event, window, cx| match event {
                ListEvent::Confirm(_) => {
                    let command = list_entity.read(cx).delegate().selected_command();
                    palette.hide(window, cx);
                    if let Some(cmd) = command {
                        execute_command(&cmd.action_type, window, cx);
                    }
                }
                ListEvent::Cancel => {
                    palette.hide(window, cx);
                }
                _ => {}
            },
        );

        let query_clone = query_input.clone();
        let list_clone = list_state.clone();
        let query_subscription = cx.subscribe_in(
            &query_input,
            window,
            move |_this, _input, event, window, cx| {
                if let InputEvent::Change = event {
                    let query = query_clone.read(cx).value().to_string();
                    list_clone.update(cx, |state, cx| {
                        state.delegate_mut().set_query(&query, cx);
                        let ix = if state.delegate().items_count(0, cx) > 0 {
                            Some(IndexPath::new(0))
                        } else {
                            None
                        };
                        state.set_selected_index(ix, window, cx);
                    });
                }
            },
        );

        Self {
            focus_handle: cx.focus_handle(),
            query_input,
            list_state,
            visible: false,
            previous_focus: None,
            _list_subscription: list_subscription,
            _query_subscription: query_subscription,
        }
    }

    pub fn init(cx: &mut App) {
        cx.bind_keys([
            KeyBinding::new(
                "escape",
                CancelCommandPalette,
                Some(COMMAND_PALETTE_CONTEXT),
            ),
            KeyBinding::new("down", SelectNextCommand, Some(COMMAND_PALETTE_CONTEXT)),
            KeyBinding::new("up", SelectPrevCommand, Some(COMMAND_PALETTE_CONTEXT)),
            KeyBinding::new("enter", ConfirmCommand, Some(COMMAND_PALETTE_CONTEXT)),
        ]);
    }

    fn move_selection(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.update(cx, |state, cx| {
            let count = state.delegate().items_count(0, cx);
            if count == 0 {
                return;
            }
            let current = state
                .selected_index()
                .map(|ix| ix.row as isize)
                .unwrap_or(-1);
            let next = if current < 0 && delta < 0 {
                (count as isize) - 1
            } else {
                (current + delta).rem_euclid(count as isize)
            };
            let new_ix = IndexPath::new(next as usize);
            state.set_selected_index(Some(new_ix), window, cx);
            state.scroll_to_item(new_ix, gpui::ScrollStrategy::Center, window, cx);
        });
    }

    fn confirm_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let command = self.list_state.read(cx).delegate().selected_command();
        self.hide(window, cx);
        if let Some(cmd) = command {
            execute_command(&cmd.action_type, window, cx);
        }
    }

    pub fn show(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.previous_focus = window.focused(cx).map(|h| h.downgrade());
        self.visible = true;
        self.query_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.list_state.update(cx, |state, cx| {
            state.delegate_mut().refresh_commands(cx);
            state.delegate_mut().set_query("", cx);
            let ix = if state.delegate().items_count(0, cx) > 0 {
                Some(IndexPath::new(0))
            } else {
                None
            };
            state.set_selected_index(ix, window, cx);
        });
        self.query_input.focus_handle(cx).focus(window, cx);
        cx.notify();
    }

    pub fn hide(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.visible = false;
        if let Some(handle) = self.previous_focus.take().and_then(|h| h.upgrade()) {
            window.focus(&handle, cx);
        } else {
            window.blur();
        }
        cx.notify();
    }

    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.visible {
            self.hide(window, cx);
        } else {
            self.show(window, cx);
        }
    }
}

impl Render for CommandPalette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let base = div()
            .key_context(COMMAND_PALETTE_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &CancelCommandPalette, window, cx| {
                this.hide(window, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectNextCommand, window, cx| {
                this.move_selection(1, window, cx);
            }))
            .on_action(cx.listener(|this, _: &SelectPrevCommand, window, cx| {
                this.move_selection(-1, window, cx);
            }))
            .on_action(cx.listener(|this, _: &ConfirmCommand, window, cx| {
                this.confirm_selection(window, cx);
            }));

        if !self.visible {
            return base.into_any_element();
        }

        let input_entity = self.query_input.clone();

        let paddings = window_paddings(window);
        let viewport = window.viewport_size();
        let view_size = gpui::size(
            viewport.width - paddings.left - paddings.right,
            viewport.height - paddings.top - paddings.bottom,
        );

        let overlay = div()
            .id("palette-overlay")
            .absolute()
            .inset_0()
            .bg(gpui::black().opacity(0.5))
            .on_click(cx.listener(|this, _, window, cx| {
                this.hide(window, cx);
            }));

        let palette_content = div()
            .id("palette-content")
            .absolute()
            .top(px(4.))
            .left(px(0.))
            .right(px(0.))
            .mx_auto()
            .w(px(520.))
            .bg(cx.theme().background)
            .border_1()
            .border_color(cx.theme().border)
            .rounded(cx.theme().radius_lg)
            .shadow_lg()
            .occlude()
            .on_click(|_, _, _| {})
            .child(
                v_flex()
                    .child(
                        div()
                            .py_2()
                            .border_b_1()
                            .border_color(cx.theme().border)
                            .child(Input::new(&input_entity).appearance(false)),
                    )
                    .child(
                        div()
                            .px_1()
                            .py_1()
                            .h(px(360.))
                            .child(List::new(&self.list_state)),
                    ),
            );

        base.child(
            deferred(
                anchored()
                    .position(point(paddings.left, paddings.top))
                    .snap_to_window()
                    .child(
                        div()
                            .occlude()
                            .w(view_size.width)
                            .h(view_size.height)
                            .child(overlay)
                            .child(palette_content),
                    ),
            )
            .with_priority(2),
        )
        .into_any_element()
    }
}

fn keybinding_for_command(cmd: &CommandType, window: &Window) -> Option<gpui_component::kbd::Kbd> {
    use gpui_component::kbd::Kbd;
    let action: Box<dyn gpui::Action> = match cmd {
        CommandType::SendRequest => Box::new(crate::requests::Send),
        CommandType::SaveRequest => Box::new(crate::requests::Save),
        CommandType::NewRequest => Box::new(crate::app::NewScratchRequest),
        CommandType::NewCollection => Box::new(crate::app::OpenNewCollectionTab),
        CommandType::OpenCollection => Box::new(crate::app::OpenCollection),
        CommandType::CloseTab => Box::new(crate::requests::CloseTab),
        CommandType::NextTab => Box::new(crate::requests::NextTab),
        CommandType::PrevTab => Box::new(crate::requests::PrevTab),
        CommandType::OpenSettings => Box::new(crate::app::OpenSettings),
        CommandType::OpenRequest { .. } => return None,
    };
    Kbd::binding_for_action(action.as_ref(), None, window)
}

fn execute_command(cmd: &CommandType, window: &mut Window, cx: &mut App) {
    match cmd {
        CommandType::SendRequest => {
            window.dispatch_action(Box::new(crate::requests::Send), cx);
        }
        CommandType::NewCollection => {
            window.dispatch_action(Box::new(crate::app::OpenNewCollectionTab), cx);
        }
        CommandType::OpenCollection => {
            window.dispatch_action(Box::new(crate::app::OpenCollection), cx);
        }
        CommandType::NewRequest => {
            window.dispatch_action(Box::new(crate::app::NewScratchRequest), cx);
        }
        CommandType::SaveRequest => {
            window.dispatch_action(Box::new(crate::requests::Save), cx);
        }
        CommandType::CloseTab => {
            window.dispatch_action(Box::new(crate::requests::CloseTab), cx);
        }
        CommandType::NextTab => {
            window.dispatch_action(Box::new(crate::requests::NextTab), cx);
        }
        CommandType::PrevTab => {
            window.dispatch_action(Box::new(crate::requests::PrevTab), cx);
        }
        CommandType::OpenSettings => {
            window.dispatch_action(Box::new(crate::app::OpenSettings), cx);
        }
        CommandType::OpenRequest {
            collection_path,
            request_name,
        } => {
            window.dispatch_action(
                Box::new(crate::app::OpenRequestFromPalette {
                    collection_path: collection_path.clone().into(),
                    request_name: request_name.clone().into(),
                }),
                cx,
            );
        }
    }
}

pub struct CommandPaletteDelegate {
    all_commands: Vec<CommandItem>,
    filtered: Vec<ScoredItem>,
    selected_index: Option<IndexPath>,
    query: String,
}

impl CommandPaletteDelegate {
    fn new(cx: &App) -> Self {
        let mut delegate = Self {
            all_commands: Vec::new(),
            filtered: Vec::new(),
            selected_index: None,
            query: String::new(),
        };
        delegate.refresh_commands(cx);
        delegate
    }

    fn refresh_commands(&mut self, cx: &App) {
        self.all_commands = build_commands(cx);
        self.apply_filter();
    }

    fn set_query(&mut self, query: &str, cx: &mut Context<ListState<Self>>) {
        self.query = query.to_string();
        self.apply_filter();
        self.selected_index = if self.filtered.is_empty() {
            None
        } else {
            Some(IndexPath::new(0))
        };
        cx.notify();
    }

    fn apply_filter(&mut self) {
        if self.query.is_empty() {
            self.filtered = self
                .all_commands
                .iter()
                .enumerate()
                .map(|(i, cmd)| ScoredItem {
                    command: cmd.clone(),
                    score: 10000 - i as i64,
                })
                .collect();
            return;
        }

        let query_lower = self.query.to_lowercase();
        let mut results: Vec<ScoredItem> = self
            .all_commands
            .iter()
            .filter_map(|cmd| {
                let label_lower = cmd.label.to_lowercase();
                let score = fuzzy_score(&label_lower, &query_lower)?;
                Some(ScoredItem {
                    command: cmd.clone(),
                    score,
                })
            })
            .collect();

        results.sort_by_key(|item| std::cmp::Reverse(item.score));
        self.filtered = results;
    }

    pub fn selected_command(&self) -> Option<CommandItem> {
        let ix = self.selected_index?;
        self.filtered.get(ix.row).map(|item| item.command.clone())
    }
}

fn fuzzy_score(text: &str, query: &str) -> Option<i64> {
    let text_chars: Vec<char> = text.chars().collect();
    let query_chars: Vec<char> = query.chars().collect();

    if query_chars.is_empty() {
        return Some(0);
    }
    if query_chars.len() > text_chars.len() {
        return None;
    }

    let mut score: i64 = 0;
    let mut query_pos = 0;
    let mut last_match_pos: Option<usize> = None;

    for (i, &ch) in text_chars.iter().enumerate() {
        if query_pos >= query_chars.len() {
            break;
        }
        if ch == query_chars[query_pos] {
            let mut bonus: i64 = 1;
            if let Some(last) = last_match_pos
                && last + 1 == i
            {
                bonus += 10;
            }
            if i == 0 || text_chars[i - 1] == ' ' || text_chars[i - 1] == '-' {
                bonus += 5;
            }
            score += bonus;
            last_match_pos = Some(i);
            query_pos += 1;
        }
    }

    if query_pos == query_chars.len() {
        Some(score)
    } else {
        None
    }
}

fn build_commands(cx: &App) -> Vec<CommandItem> {
    let mut commands = vec![
        CommandItem {
            label: "Send Request".into(),
            group: "Request".into(),
            action_type: CommandType::SendRequest,
        },
        CommandItem {
            label: "Save Request".into(),
            group: "Request".into(),
            action_type: CommandType::SaveRequest,
        },
        CommandItem {
            label: "New Request".into(),
            group: "Request".into(),
            action_type: CommandType::NewRequest,
        },
        CommandItem {
            label: "New Collection".into(),
            group: "Collection".into(),
            action_type: CommandType::NewCollection,
        },
        CommandItem {
            label: "Open Collection".into(),
            group: "Collection".into(),
            action_type: CommandType::OpenCollection,
        },
        CommandItem {
            label: "Close Tab".into(),
            group: "Tab".into(),
            action_type: CommandType::CloseTab,
        },
        CommandItem {
            label: "Next Tab".into(),
            group: "Tab".into(),
            action_type: CommandType::NextTab,
        },
        CommandItem {
            label: "Previous Tab".into(),
            group: "Tab".into(),
            action_type: CommandType::PrevTab,
        },
        CommandItem {
            label: "Open Settings".into(),
            group: "View".into(),
            action_type: CommandType::OpenSettings,
        },
    ];

    add_collection_requests(&mut commands, cx);

    commands
}

fn add_collection_requests(commands: &mut Vec<CommandItem>, cx: &App) {
    let manager = crate::collections::CollectionManager::global(cx);
    let manager = manager.read(cx);
    let collections = manager.get_all_collections();

    for collection in collections {
        let mut push_request = |request: &crate::domain::RequestData, group: Option<&str>| {
            let method = crate::domain::HttpMethod::ALL
                .iter()
                .find(|m| m.as_str() == request.method.as_str())
                .copied()
                .unwrap_or(crate::domain::HttpMethod::Get);
            let group_label = match group {
                Some(g) => format!("{} / {}", collection.data.name, g),
                None => collection.data.name.clone(),
            };
            commands.push(CommandItem {
                label: format!("{} {}", method.as_str(), request.name),
                group: group_label,
                action_type: CommandType::OpenRequest {
                    collection_path: collection.data.path.clone(),
                    request_name: request.name.clone(),
                },
            });
        };

        for request in collection.requests.values() {
            push_request(request, None);
        }
        for group in collection.groups.values() {
            for request in group.requests.values() {
                push_request(request, Some(&group.name));
            }
        }
    }
}

pub struct CommandPaletteItemElement {
    ix: IndexPath,
    command: CommandItem,
    selected: bool,
    muted_color: gpui::Hsla,
    selected_bg: gpui::Hsla,
    selected_fg: gpui::Hsla,
    keybinding: Option<gpui_component::kbd::Kbd>,
}

impl CommandPaletteItemElement {
    pub fn new(
        ix: IndexPath,
        command: CommandItem,
        selected: bool,
        muted_color: gpui::Hsla,
        selected_bg: gpui::Hsla,
        selected_fg: gpui::Hsla,
        keybinding: Option<gpui_component::kbd::Kbd>,
    ) -> Self {
        Self {
            ix,
            command,
            selected,
            muted_color,
            selected_bg,
            selected_fg,
            keybinding,
        }
    }
}

impl Selectable for CommandPaletteItemElement {
    fn selected(mut self, selected: bool) -> Self {
        self.selected = selected;
        self
    }

    fn is_selected(&self) -> bool {
        self.selected
    }
}

impl IntoElement for CommandPaletteItemElement {
    type Element = gpui::Stateful<gpui::Div>;

    fn into_element(self) -> Self::Element {
        let id = gpui::ElementId::Name(format!("cmd-{}", self.ix.row).into());
        let selected = self.selected;
        let selected_bg = self.selected_bg;
        let selected_fg = self.selected_fg;
        let group_color = if selected {
            selected_fg
        } else {
            self.muted_color
        };
        div()
            .id(id)
            .w_full()
            .px_2()
            .py_1()
            .rounded(px(4.))
            .cursor_pointer()
            .when(selected, |el| el.bg(selected_bg).text_color(selected_fg))
            .child(
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .text_sm()
                    .child(div().text_color(group_color).child(self.command.group))
                    .child(div().text_color(group_color).child("/"))
                    .child(div().flex_1().child(self.command.label))
                    .when_some(self.keybinding, |this, kbd| this.child(kbd)),
            )
    }
}

impl ListDelegate for CommandPaletteDelegate {
    type Item = CommandPaletteItemElement;

    fn items_count(&self, _section: usize, _cx: &App) -> usize {
        self.filtered.len()
    }

    fn render_item(
        &mut self,
        ix: IndexPath,
        window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> Option<Self::Item> {
        let selected = self.selected_index == Some(ix);
        let command = self.filtered.get(ix.row)?.command.clone();
        let theme = cx.theme();
        let muted_color = theme.muted_foreground;
        let selected_bg = theme.accent;
        let selected_fg = theme.accent_foreground;
        let keybinding = keybinding_for_command(&command.action_type, window);
        Some(CommandPaletteItemElement::new(
            ix,
            command,
            selected,
            muted_color,
            selected_bg,
            selected_fg,
            keybinding,
        ))
    }

    fn perform_search(
        &mut self,
        query: &str,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) -> gpui::Task<()> {
        self.query = query.to_string();
        self.apply_filter();
        cx.notify();
        gpui::Task::ready(())
    }

    fn set_selected_index(
        &mut self,
        ix: Option<IndexPath>,
        _window: &mut Window,
        cx: &mut Context<ListState<Self>>,
    ) {
        self.selected_index = ix;
        cx.notify();
    }

    fn confirm(
        &mut self,
        _secondary: bool,
        _window: &mut Window,
        _cx: &mut Context<ListState<Self>>,
    ) {
    }

    fn cancel(&mut self, _window: &mut Window, _cx: &mut Context<ListState<Self>>) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fuzzy_score_exact_match() {
        assert!(fuzzy_score("send request", "send request").is_some());
    }

    #[test]
    fn test_fuzzy_score_prefix() {
        assert!(fuzzy_score("send request", "send").is_some());
    }

    #[test]
    fn test_fuzzy_score_subsequence() {
        assert!(fuzzy_score("switch to dark theme", "std").is_some());
    }

    #[test]
    fn test_fuzzy_score_no_match() {
        assert!(fuzzy_score("send request", "xyz").is_none());
    }

    #[test]
    fn test_fuzzy_score_empty_query() {
        assert!(fuzzy_score("send request", "").is_some());
    }

    #[test]
    fn test_fuzzy_score_consecutive_beats_spread() {
        let consecutive = fuzzy_score("close tab", "clot").unwrap();
        let spread = fuzzy_score("clear history", "clt").unwrap();
        assert!(consecutive > spread);
    }
}
