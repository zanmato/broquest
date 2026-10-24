use gpui::{
    App, AppContext, Context, Entity, EventEmitter, FocusHandle, Focusable,
    InteractiveElement as _, IntoElement, KeyBinding, ParentElement as _, Render,
    StatefulInteractiveElement as _, Styled as _, Subscription, WeakFocusHandle, Window, actions,
    anchored, deferred, div, point, prelude::FluentBuilder, px,
};
use gpui_kit::component::{
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
    /// More words the search finds the command by, besides its label: a
    /// request's collection, folder and URL, or a command's synonyms.
    pub keywords: Vec<String>,
    /// Whether the command can run now. One that can't is shown greyed out
    /// after the others, and can't be chosen.
    pub enabled: bool,
}

/// What the palette needs to know about the open tabs to tell which
/// commands can run.
#[derive(Clone, Copy, Debug, Default)]
pub struct PaletteContext {
    pub tab_count: usize,
    pub active_tab: usize,
    /// The active tab is a request.
    pub request_tab: bool,
    /// The active request has a JSON response to filter.
    pub json_response: bool,
    /// The active tab is a request that was saved, which can be renamed.
    pub saved_request_tab: bool,
}

impl PaletteContext {
    /// Whether `command` can run with the tabs as they are.
    fn allows(&self, command: &CommandType) -> bool {
        match command {
            CommandType::SendRequest
            | CommandType::SaveRequest
            | CommandType::CopyResponse
            | CommandType::FormatBody => self.request_tab,
            CommandType::FilterResponse => self.json_response,
            CommandType::RenameRequest => self.saved_request_tab,
            CommandType::CloseTab | CommandType::CloseAllTabs => self.tab_count > 0,
            CommandType::CloseOtherTabs | CommandType::NextTab | CommandType::PrevTab => {
                self.tab_count > 1
            }
            CommandType::CloseTabsToTheLeft => self.active_tab > 0,
            CommandType::CloseTabsToTheRight => self.active_tab + 1 < self.tab_count,
            CommandType::NewCollection
            | CommandType::OpenCollection
            | CommandType::NewRequest
            | CommandType::OpenSettings
            | CommandType::OpenRequest { .. } => true,
        }
    }
}

#[derive(Clone, Debug)]
pub enum CommandType {
    SendRequest,
    NewCollection,
    OpenCollection,
    NewRequest,
    SaveRequest,
    CopyResponse,
    FormatBody,
    FilterResponse,
    RenameRequest,
    CloseTab,
    CloseOtherTabs,
    CloseTabsToTheLeft,
    CloseTabsToTheRight,
    CloseAllTabs,
    NextTab,
    PrevTab,
    OpenSettings,
    OpenRequest {
        collection_path: String,
        request_name: String,
    },
}

/// A command with its label and keywords in lower case, made once when the
/// palette opens so that typing doesn't redo it for every command.
struct Entry {
    command: CommandItem,
    label: String,
    keywords: Vec<String>,
}

impl Entry {
    fn new(command: CommandItem) -> Self {
        Self {
            label: command.label.to_lowercase(),
            keywords: command.keywords.iter().map(|k| k.to_lowercase()).collect(),
            command,
        }
    }

    /// How well the command matches the query, `None` when it doesn't. A
    /// match on the label ranks above any match on keywords only, which needs
    /// every word of the query in the label or a keyword.
    fn score(&self, query: &[char], words: &[&str]) -> Option<i64> {
        const LABEL_MATCH: i64 = 1_000_000;
        if let Some(score) = fuzzy_score(&self.label, query) {
            return Some(LABEL_MATCH + score);
        }
        let found = |word: &&str| {
            self.label.contains(*word) || self.keywords.iter().any(|k| k.contains(*word))
        };
        (!words.is_empty() && words.iter().all(found)).then_some(0)
    }
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
                    palette.cancel(window, cx);
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
                        let ix = state.delegate().first_enabled();
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

    /// Move the highlight by `delta` rows, wrapping around and skipping the
    /// commands that can't run.
    fn move_selection(&mut self, delta: isize, window: &mut Window, cx: &mut Context<Self>) {
        self.list_state.update(cx, |state, cx| {
            let count = state.delegate().items_count(0, cx) as isize;
            if count == 0 {
                return;
            }
            let mut row = state
                .selected_index()
                .map(|ix| ix.row as isize)
                .unwrap_or(if delta < 0 { 0 } else { -1 });
            for _ in 0..count {
                row = (row + delta).rem_euclid(count);
                if state.delegate().is_enabled(row as usize) {
                    let new_ix = IndexPath::new(row as usize);
                    state.set_selected_index(Some(new_ix), window, cx);
                    state.scroll_to_item(new_ix, gpui::ScrollStrategy::Center, window, cx);
                    return;
                }
            }
        });
    }

    /// Escape clears a typed search first, and closes the palette when there
    /// is none.
    fn cancel(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.query_input.read(cx).value().is_empty() {
            self.hide(window, cx);
            return;
        }
        // Setting the value sends no change event, so filter here.
        self.query_input
            .update(cx, |input, cx| input.set_value("", window, cx));
        self.list_state.update(cx, |state, cx| {
            state.delegate_mut().set_query("", cx);
            let ix = state.delegate().first_enabled();
            state.set_selected_index(ix, window, cx);
        });
    }

    fn confirm_selection(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let command = self.list_state.read(cx).delegate().selected_command();
        self.hide(window, cx);
        if let Some(cmd) = command {
            execute_command(&cmd.action_type, window, cx);
        }
    }

    pub fn show(&mut self, context: PaletteContext, window: &mut Window, cx: &mut Context<Self>) {
        self.previous_focus = window.focused(cx).map(|h| h.downgrade());
        self.visible = true;
        self.query_input.update(cx, |input, cx| {
            input.set_value("", window, cx);
        });
        self.list_state.update(cx, |state, cx| {
            state.delegate_mut().refresh_commands(context, cx);
            state.delegate_mut().set_query("", cx);
            let ix = state.delegate().first_enabled();
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
            window.blur(cx);
        }
        cx.notify();
    }

    pub fn toggle(&mut self, context: PaletteContext, window: &mut Window, cx: &mut Context<Self>) {
        if self.visible {
            self.hide(window, cx);
        } else {
            self.show(context, window, cx);
        }
    }
}

impl Render for CommandPalette {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let base = div()
            .key_context(COMMAND_PALETTE_CONTEXT)
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(|this, _: &CancelCommandPalette, window, cx| {
                this.cancel(window, cx);
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

fn keybinding_for_command(
    cmd: &CommandType,
    window: &Window,
) -> Option<gpui_kit::component::kbd::Kbd> {
    use crate::requests::{EDITOR_PANEL_CONTEXT, REQUEST_EDITOR_CONTEXT};
    use gpui_kit::component::kbd::Kbd;
    // The key context each command's shortcut is bound in. The palette has
    // focus while it is open, so the lookup names the context rather than
    // using the focused one.
    let request = Some(REQUEST_EDITOR_CONTEXT);
    let panel = Some(EDITOR_PANEL_CONTEXT);
    let (action, context): (Box<dyn gpui::Action>, Option<&str>) = match cmd {
        CommandType::SendRequest => (Box::new(crate::requests::Send), request),
        CommandType::SaveRequest => (Box::new(crate::requests::Save), request),
        CommandType::CopyResponse => (Box::new(crate::requests::CopyResponse), request),
        CommandType::FormatBody => (Box::new(crate::requests::FormatBody), request),
        CommandType::FilterResponse => (Box::new(crate::requests::FilterResponse), request),
        CommandType::RenameRequest => (Box::new(crate::collections::RenameItem), None),
        CommandType::NewRequest => (Box::new(crate::app::NewScratchRequest), None),
        CommandType::NewCollection => (Box::new(crate::app::OpenNewCollectionTab), None),
        CommandType::OpenCollection => (Box::new(crate::app::OpenCollection), None),
        CommandType::CloseTab => (Box::new(crate::requests::CloseTab), panel),
        CommandType::CloseOtherTabs => (Box::new(crate::requests::CloseOtherTabs), panel),
        CommandType::CloseTabsToTheLeft => (Box::new(crate::requests::CloseTabsToTheLeft), panel),
        CommandType::CloseTabsToTheRight => (Box::new(crate::requests::CloseTabsToTheRight), panel),
        CommandType::CloseAllTabs => (Box::new(crate::requests::CloseAllTabs), panel),
        CommandType::NextTab => (Box::new(crate::requests::NextTab), None),
        CommandType::PrevTab => (Box::new(crate::requests::PrevTab), None),
        CommandType::OpenSettings => (Box::new(crate::app::OpenSettings), None),
        CommandType::OpenRequest { .. } => return None,
    };
    Kbd::binding_for_action(action.as_ref(), context, window)
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
        CommandType::CopyResponse => {
            window.dispatch_action(Box::new(crate::requests::CopyResponse), cx);
        }
        CommandType::FormatBody => {
            window.dispatch_action(Box::new(crate::requests::FormatBody), cx);
        }
        CommandType::FilterResponse => {
            window.dispatch_action(Box::new(crate::requests::FilterResponse), cx);
        }
        CommandType::RenameRequest => {
            window.dispatch_action(Box::new(crate::collections::RenameItem), cx);
        }
        CommandType::CloseTab => {
            window.dispatch_action(Box::new(crate::requests::CloseTab), cx);
        }
        CommandType::CloseOtherTabs => {
            window.dispatch_action(Box::new(crate::requests::CloseOtherTabs), cx);
        }
        CommandType::CloseTabsToTheLeft => {
            window.dispatch_action(Box::new(crate::requests::CloseTabsToTheLeft), cx);
        }
        CommandType::CloseTabsToTheRight => {
            window.dispatch_action(Box::new(crate::requests::CloseTabsToTheRight), cx);
        }
        CommandType::CloseAllTabs => {
            window.dispatch_action(Box::new(crate::requests::CloseAllTabs), cx);
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
    entries: Vec<Entry>,
    /// The matching commands, as indices into `entries`, best first.
    filtered: Vec<usize>,
    selected_index: Option<IndexPath>,
    query: String,
}

impl CommandPaletteDelegate {
    fn new(cx: &App) -> Self {
        let mut delegate = Self {
            entries: Vec::new(),
            filtered: Vec::new(),
            selected_index: None,
            query: String::new(),
        };
        delegate.refresh_commands(PaletteContext::default(), cx);
        delegate
    }

    fn refresh_commands(&mut self, context: PaletteContext, cx: &App) {
        self.entries = build_commands(context, cx)
            .into_iter()
            .map(Entry::new)
            .collect();
        self.apply_filter();
    }

    fn set_query(&mut self, query: &str, cx: &mut Context<ListState<Self>>) {
        self.query = query.to_string();
        self.apply_filter();
        self.selected_index = self.first_enabled();
        cx.notify();
    }

    fn apply_filter(&mut self) {
        let query = self.query.trim().to_lowercase();
        let mut scored: Vec<(usize, i64)> = if query.is_empty() {
            (0..self.entries.len()).map(|ix| (ix, 0)).collect()
        } else {
            let chars: Vec<char> = query.chars().collect();
            let words: Vec<&str> = query.split_whitespace().collect();
            self.entries
                .iter()
                .enumerate()
                .filter_map(|(ix, entry)| Some((ix, entry.score(&chars, &words)?)))
                .collect()
        };
        // The best first, and the commands that can't run after the others.
        // The sort is stable, so equal ones keep the order they were added in.
        scored.sort_by_key(|&(ix, score)| {
            (!self.entries[ix].command.enabled, std::cmp::Reverse(score))
        });
        self.filtered = scored.into_iter().map(|(ix, _)| ix).collect();
    }

    fn command_at(&self, row: usize) -> Option<&CommandItem> {
        self.filtered.get(row).map(|&ix| &self.entries[ix].command)
    }

    fn is_enabled(&self, row: usize) -> bool {
        self.command_at(row).is_some_and(|command| command.enabled)
    }

    /// The first row whose command can run.
    fn first_enabled(&self) -> Option<IndexPath> {
        (0..self.filtered.len())
            .find(|&row| self.is_enabled(row))
            .map(IndexPath::new)
    }

    /// The highlighted command, when it can run.
    pub fn selected_command(&self) -> Option<CommandItem> {
        let ix = self.selected_index?;
        self.command_at(ix.row)
            .filter(|command| command.enabled)
            .cloned()
    }
}

/// How well `query` matches `text` as a subsequence, higher for runs of
/// consecutive characters and for matches at the start of a word. `None`
/// when it doesn't match. Both are expected in lower case.
fn fuzzy_score(text: &str, query: &[char]) -> Option<i64> {
    if query.is_empty() {
        return Some(0);
    }
    let mut score: i64 = 0;
    let mut query_pos = 0;
    let mut last_match: Option<usize> = None;
    let mut previous: Option<char> = None;
    for (i, ch) in text.chars().enumerate() {
        if query_pos == query.len() {
            break;
        }
        if ch == query[query_pos] {
            let mut bonus: i64 = 1;
            if last_match.is_some_and(|last| last + 1 == i) {
                bonus += 10;
            }
            if matches!(previous, None | Some(' ') | Some('-')) {
                bonus += 5;
            }
            score += bonus;
            last_match = Some(i);
            query_pos += 1;
        }
        previous = Some(ch);
    }
    (query_pos == query.len()).then_some(score)
}

fn build_commands(context: PaletteContext, cx: &App) -> Vec<CommandItem> {
    let command =
        |label: &str, group: &str, action_type: CommandType, keywords: &[&str]| CommandItem {
            label: label.into(),
            group: group.into(),
            enabled: context.allows(&action_type),
            action_type,
            keywords: keywords.iter().map(|k| k.to_string()).collect(),
        };
    let mut commands = vec![
        command(
            "Send Request",
            "Request",
            CommandType::SendRequest,
            &["run", "execute"],
        ),
        command(
            "Save Request",
            "Request",
            CommandType::SaveRequest,
            &["write"],
        ),
        command(
            "Copy Response",
            "Request",
            CommandType::CopyResponse,
            &["clipboard"],
        ),
        command(
            "Format Request Body",
            "Request",
            CommandType::FormatBody,
            &["prettify", "beautify", "indent", "json", "xml"],
        ),
        command(
            "Filter Response with JSONPath",
            "Request",
            CommandType::FilterResponse,
            &["search", "query", "find"],
        ),
        command(
            "Rename Request",
            "Request",
            CommandType::RenameRequest,
            &["name"],
        ),
        command(
            "New Request",
            "Request",
            CommandType::NewRequest,
            &["create", "scratch"],
        ),
        command(
            "New Collection",
            "Collection",
            CommandType::NewCollection,
            &["create"],
        ),
        command(
            "Open Collection",
            "Collection",
            CommandType::OpenCollection,
            &["import", "folder", "load"],
        ),
        command("Close Tab", "Tab", CommandType::CloseTab, &[]),
        command("Close Other Tabs", "Tab", CommandType::CloseOtherTabs, &[]),
        command(
            "Close Tabs to the Left",
            "Tab",
            CommandType::CloseTabsToTheLeft,
            &[],
        ),
        command(
            "Close Tabs to the Right",
            "Tab",
            CommandType::CloseTabsToTheRight,
            &[],
        ),
        command("Close All Tabs", "Tab", CommandType::CloseAllTabs, &[]),
        command("Next Tab", "Tab", CommandType::NextTab, &["switch"]),
        command("Previous Tab", "Tab", CommandType::PrevTab, &["switch"]),
        command(
            "Open Settings",
            "View",
            CommandType::OpenSettings,
            &[
                "preferences",
                "options",
                "keybindings",
                "shortcuts",
                "theme",
            ],
        ),
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
            // A request is also found by its collection, folder and URL.
            let mut keywords = vec![collection.data.name.clone(), request.url.clone()];
            keywords.extend(group.map(str::to_string));
            commands.push(CommandItem {
                label: format!("{} {}", method.as_str(), request.name),
                group: group_label,
                action_type: CommandType::OpenRequest {
                    collection_path: collection.data.path.clone(),
                    request_name: request.name.clone(),
                },
                keywords,
                enabled: true,
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
    keybinding: Option<gpui_kit::component::kbd::Kbd>,
}

impl CommandPaletteItemElement {
    pub fn new(
        ix: IndexPath,
        command: CommandItem,
        selected: bool,
        muted_color: gpui::Hsla,
        selected_bg: gpui::Hsla,
        selected_fg: gpui::Hsla,
        keybinding: Option<gpui_kit::component::kbd::Kbd>,
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
        let enabled = self.command.enabled;
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
            .when(enabled, |el| el.cursor_pointer())
            // A command that can't run now is shown faded.
            .when(!enabled, |el| el.opacity(0.5))
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
        let command = self.command_at(ix.row)?.clone();
        let selected = self.selected_index == Some(ix) && command.enabled;
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
        self.selected_index = self.first_enabled();
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

    fn score(text: &str, query: &str) -> Option<i64> {
        fuzzy_score(text, &query.chars().collect::<Vec<_>>())
    }

    fn item(label: &str, keywords: &[&str], enabled: bool) -> CommandItem {
        CommandItem {
            label: label.into(),
            group: "Group".into(),
            action_type: CommandType::NewRequest,
            keywords: keywords.iter().map(|k| k.to_string()).collect(),
            enabled,
        }
    }

    fn delegate(items: Vec<CommandItem>) -> CommandPaletteDelegate {
        CommandPaletteDelegate {
            entries: items.into_iter().map(Entry::new).collect(),
            filtered: Vec::new(),
            selected_index: None,
            query: String::new(),
        }
    }

    fn labels(delegate: &CommandPaletteDelegate) -> Vec<&str> {
        (0..delegate.filtered.len())
            .map(|row| delegate.command_at(row).unwrap().label.as_str())
            .collect()
    }

    #[test]
    fn test_fuzzy_score_exact_match() {
        assert!(score("send request", "send request").is_some());
    }

    #[test]
    fn test_fuzzy_score_prefix() {
        assert!(score("send request", "send").is_some());
    }

    #[test]
    fn test_fuzzy_score_subsequence() {
        assert!(score("switch to dark theme", "std").is_some());
    }

    #[test]
    fn test_fuzzy_score_no_match() {
        assert!(score("send request", "xyz").is_none());
    }

    #[test]
    fn test_fuzzy_score_empty_query() {
        assert!(score("send request", "").is_some());
    }

    #[test]
    fn test_fuzzy_score_consecutive_beats_spread() {
        let consecutive = score("close tab", "clot").unwrap();
        let spread = score("clear history", "clt").unwrap();
        assert!(consecutive > spread);
    }

    #[test]
    fn keywords_find_requests_by_collection_and_url() {
        let mut palette = delegate(vec![
            item(
                "GET List shipments",
                &["Kestrel", "{{baseUrl}}/v1/shipments"],
                true,
            ),
            item(
                "POST Log in",
                &["Kestrel", "{{baseUrl}}/v1/auth/token"],
                true,
            ),
            item("GET Health", &["Other", "{{baseUrl}}/health"], true),
        ]);
        palette.query = "/v1/auth".into();
        palette.apply_filter();
        assert_eq!(labels(&palette), ["POST Log in"]);

        // Every word has to be found, in the label or a keyword.
        palette.query = "kestrel list".into();
        palette.apply_filter();
        assert_eq!(labels(&palette), ["GET List shipments"]);
    }

    #[test]
    fn label_matches_rank_above_keyword_matches() {
        let mut palette = delegate(vec![
            item("Open Settings", &["theme"], true),
            item("Switch Theme", &[], true),
        ]);
        palette.query = "theme".into();
        palette.apply_filter();
        assert_eq!(labels(&palette), ["Switch Theme", "Open Settings"]);
    }

    #[test]
    fn commands_that_cant_run_come_last_and_are_skipped() {
        let mut palette = delegate(vec![
            item("Send Request", &[], false),
            item("Save Request", &[], true),
        ]);
        palette.query = "request".into();
        palette.apply_filter();
        assert_eq!(labels(&palette), ["Save Request", "Send Request"]);
        assert_eq!(palette.first_enabled(), Some(IndexPath::new(0)));

        palette.selected_index = Some(IndexPath::new(1));
        assert!(
            palette.selected_command().is_none(),
            "a disabled command can't run"
        );
    }

    #[test]
    fn tab_commands_follow_the_open_tabs() {
        let context = PaletteContext {
            tab_count: 3,
            active_tab: 0,
            request_tab: true,
            json_response: false,
            saved_request_tab: true,
        };
        assert!(context.allows(&CommandType::SendRequest));
        assert!(!context.allows(&CommandType::FilterResponse));
        assert!(!context.allows(&CommandType::CloseTabsToTheLeft));
        assert!(context.allows(&CommandType::CloseTabsToTheRight));

        let none = PaletteContext::default();
        assert!(!none.allows(&CommandType::CloseTab));
        assert!(!none.allows(&CommandType::SendRequest));
        assert!(none.allows(&CommandType::NewRequest));
    }
}
