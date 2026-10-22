//! A settings panel that searches Open VSX for VS Code color themes,
//! previews what they convert to, and installs them into the app's theme
//! folder.
//!
//! The app embeds a [`ThemeBrowser`] with [`setting_group`] and says, through
//! [`ThemeBrowserOptions`], where its custom themes load from and how a theme
//! is chosen.
//!
//! Try shows a converted theme across the app without installing it: the
//! theme is applied to [`Theme`] directly, never registered, since the
//! registry has no way to drop a theme again.

use std::path::PathBuf;
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;

use gpui::{
    App, AppContext as _, Context, Entity, FontWeight, Image, ImageFormat, InteractiveElement as _,
    IntoElement, ParentElement, Render, SharedString, Styled, Subscription, Task, Window, div, img,
    prelude::FluentBuilder as _, px,
};
use gpui_component::{
    ActiveTheme as _, Disableable as _, Sizable as _, Theme, ThemeConfig, ThemeRegistry,
    button::{Button, ButtonVariants as _},
    h_flex,
    input::{Input, InputEvent, InputState},
    setting::{SettingGroup, SettingItem},
    spinner::Spinner,
    v_flex,
};

use crate::color::Rgba;
use crate::convert::{ConvertOptions, ConvertedThemes, convert_package};
use crate::gallery::{Client, Extension, OPEN_VSX};
use crate::package::Package;

const PAGE_SIZE: u32 = 20;
const SEARCH_DELAY: Duration = Duration::from_millis(350);

/// Choose the theme called by its name, as the app's theme setting would.
pub type UseTheme = Rc<dyn Fn(&str, &mut Window, &mut App)>;

pub struct ThemeBrowserOptions {
    /// The folder the app loads custom theme files from at startup.
    pub themes_dir: PathBuf,
    pub convert: ConvertOptions,
    /// Offers a Use button next to each installed theme when set.
    pub use_theme: Option<UseTheme>,
    /// The app watches `themes_dir` (`ThemeRegistry::watch_dir`), which
    /// rebuilds the registry from the folder on every change, so a
    /// reinstalled or removed theme takes effect at once rather than after a
    /// restart.
    pub watches_themes_dir: bool,
}

/// The browser as a group for a gpui-component settings page.
pub fn setting_group(browser: Entity<ThemeBrowser>) -> SettingGroup {
    SettingGroup::new()
        .title("VS Code Themes")
        .description(
            "Find VS Code color themes on Open VSX, the open extension registry. Try shows \
             one across the app until you revert it or close Settings. Install adds it to the \
             theme lists above.",
        )
        .items(vec![SettingItem::render(move |_, _, _| browser.clone())])
}

pub struct ThemeBrowser {
    options: ThemeBrowserOptions,
    client: Result<Arc<Client>, SharedString>,
    search_input: Entity<InputState>,
    entries: Vec<Entry>,
    total: u64,
    next_page: u32,
    searching: bool,
    search_error: Option<SharedString>,
    search_task: Option<Task<()>>,
    /// Manifest and icon lookups of the listed entries. Replaced with the
    /// list, so a new search drops the old ones.
    detail_tasks: Vec<Task<()>>,
    action_tasks: Vec<Task<()>>,
    preview: Option<Preview>,
    _subscriptions: Vec<Subscription>,
}

/// A theme shown with Try, and the theme to go back to.
struct Preview {
    name: String,
    applied: Rc<ThemeConfig>,
    /// The whole theme as it was before the first Try, so reverting also
    /// keeps whatever the app set on top of its theme file, such as fonts.
    before: Theme,
}

struct Entry {
    extension: Arc<Extension>,
    icon: Option<Arc<Image>>,
    kind: Kind,
    status: Status,
    converted: Option<Arc<ConvertedThemes>>,
    /// The themes of the installed file, by name and whether light.
    installed: Vec<(String, bool)>,
}

#[derive(PartialEq)]
enum Kind {
    Checking,
    /// With their number when the manifest was read.
    ColorThemes(Option<usize>),
    /// Icon and product icon themes share the Themes category.
    NoColorThemes,
    Unknown,
}

enum Status {
    Idle,
    Working(&'static str),
    Note(SharedString),
    Failed(SharedString),
}

impl ThemeBrowser {
    pub fn new(options: ThemeBrowserOptions, window: &mut Window, cx: &mut Context<Self>) -> Self {
        let search_input = cx.new(|cx| InputState::new(window, cx).placeholder("Search Open VSX"));
        let subscription = cx.subscribe(&search_input, |this, _, event: &InputEvent, cx| {
            if let InputEvent::Change = event {
                this.schedule_search(cx);
            }
        });
        // Leaving the panel, by closing its page or tab, ends a preview.
        let release = cx.on_release(|this, cx| this.end_preview(cx));
        let client = Client::new(OPEN_VSX)
            .map(Arc::new)
            .map_err(|error| SharedString::from(format!("{error:#}")));
        let mut browser = Self {
            options,
            client,
            search_input,
            entries: Vec::new(),
            total: 0,
            next_page: 1,
            searching: false,
            search_error: None,
            search_task: None,
            detail_tasks: Vec::new(),
            action_tasks: Vec::new(),
            preview: None,
            _subscriptions: vec![subscription, release],
        };
        browser.search(false, cx);
        browser
    }

    fn schedule_search(&mut self, cx: &mut Context<Self>) {
        self.search_task = Some(cx.spawn(async move |this, cx| {
            cx.background_executor().timer(SEARCH_DELAY).await;
            if let Err(error) = this.update(cx, |this, cx| this.search(false, cx)) {
                tracing::debug!("Theme browser closed before its search ran: {error}");
            }
        }));
    }

    /// Run the search, or with `more` fetch its next page.
    fn search(&mut self, more: bool, cx: &mut Context<Self>) {
        let client = match &self.client {
            Ok(client) => client.clone(),
            Err(error) => {
                self.search_error = Some(error.clone());
                return;
            }
        };
        let query = self.search_input.read(cx).value().to_string();
        let page = if more { self.next_page } else { 1 };
        self.searching = true;
        self.search_error = None;
        cx.notify();

        let search = cx.background_spawn(async move { client.search(&query, page, PAGE_SIZE) });
        self.search_task = Some(cx.spawn(async move |this, cx| {
            let result = search.await;
            let updated = this.update(cx, |this, cx| {
                this.searching = false;
                match result {
                    Ok(found) => {
                        if !more {
                            this.entries.clear();
                            this.detail_tasks.clear();
                        }
                        this.total = found.total;
                        this.next_page = page + 1;
                        for extension in found.extensions {
                            this.add_entry(extension, cx);
                        }
                    }
                    Err(error) => this.search_error = Some(format!("{error:#}").into()),
                }
                cx.notify();
            });
            if let Err(error) = updated {
                tracing::debug!("Theme browser closed during a search: {error}");
            }
        }));
    }

    fn add_entry(&mut self, extension: Extension, cx: &mut Context<Self>) {
        let extension = Arc::new(extension);
        let id = extension.id();
        let installed = crate::installed_theme_names(&self.options.themes_dir, &id);
        // Tags settle most extensions, so only the untagged need a manifest
        // request, and icon themes need no request at all.
        let kind = match extension.has_color_themes_by_tags() {
            Some(true) => Kind::ColorThemes(None),
            Some(false) => Kind::NoColorThemes,
            None => Kind::Checking,
        };
        let check_manifest = kind == Kind::Checking;
        if kind == Kind::NoColorThemes {
            self.entries.push(Entry {
                extension,
                icon: None,
                kind,
                status: Status::Idle,
                converted: None,
                installed,
            });
            return;
        }
        self.entries.push(Entry {
            extension: extension.clone(),
            icon: None,
            kind,
            status: Status::Idle,
            converted: None,
            installed,
        });

        let Ok(client) = self.client.clone() else {
            return;
        };
        let details = cx.background_spawn(async move {
            let manifest = check_manifest.then(|| client.manifest(&extension));
            let icon = client.icon(&extension);
            (manifest, icon)
        });
        self.detail_tasks.push(cx.spawn(async move |this, cx| {
            let (manifest, icon) = details.await;
            let updated = this.update(cx, |this, cx| {
                let Some(entry) = this.entry_mut(&id) else {
                    return;
                };
                match manifest {
                    Some(Ok(manifest)) if manifest.has_color_themes() => {
                        entry.kind = Kind::ColorThemes(Some(manifest.contributes.themes.len()))
                    }
                    Some(Ok(_)) => entry.kind = Kind::NoColorThemes,
                    Some(Err(error)) => {
                        tracing::debug!("No manifest for {id}: {error:#}");
                        entry.kind = Kind::Unknown;
                    }
                    None => {}
                }
                match icon {
                    Ok(Some(bytes)) => entry.icon = image(bytes),
                    Ok(None) => {}
                    Err(error) => tracing::debug!("No icon for {id}: {error:#}"),
                }
                cx.notify();
            });
            if let Err(error) = updated {
                tracing::debug!("Theme browser closed during a lookup: {error}");
            }
        }));
    }

    fn entry_mut(&mut self, id: &str) -> Option<&mut Entry> {
        self.entries
            .iter_mut()
            .find(|entry| entry.extension.id() == id)
    }

    /// Download and convert the extension, then `install` it too.
    fn convert(&mut self, id: String, install: bool, cx: &mut Context<Self>) {
        let Ok(client) = self.client.clone() else {
            return;
        };
        let options = self.options.convert;
        let themes_dir = self.options.themes_dir.clone();
        let Some(entry) = self.entry_mut(&id) else {
            return;
        };
        let extension = entry.extension.clone();
        let converted = entry.converted.clone();
        entry.status = Status::Working(if install { "Installing" } else { "Converting" });
        cx.notify();

        let work = cx.background_spawn({
            let id = id.clone();
            async move {
                let converted = match converted {
                    Some(converted) => converted,
                    None => {
                        let vsix = client.download(&extension)?;
                        let package = Package::from_vsix(&vsix)?;
                        Arc::new(convert_package(&package, &options)?)
                    }
                };
                if install {
                    crate::install(&themes_dir, &id, &converted)?;
                }
                anyhow::Ok(converted)
            }
        });
        self.action_tasks.push(cx.spawn(async move |this, cx| {
            let result = work.await;
            let updated = this.update(cx, |this, cx| {
                let registered = match &result {
                    Ok(converted) if install => Some(register(converted, cx)),
                    _ => None,
                };
                let watched = this.options.watches_themes_dir;
                let Some(entry) = this.entry_mut(&id) else {
                    return;
                };
                match result {
                    Ok(converted) => {
                        if install {
                            entry.installed = converted
                                .themes
                                .iter()
                                .map(|theme| (theme.name.clone(), theme.light))
                                .collect();
                        }
                        entry.status = match registered {
                            Some(Err(error)) => Status::Failed(format!("{error:#}").into()),
                            Some(Ok(true)) if !watched => Status::Note(
                                "Installed. A theme of the same name was already loaded, so \
                                 the new one takes over after a restart."
                                    .into(),
                            ),
                            _ => Status::Idle,
                        };
                        entry.converted = Some(converted);
                    }
                    Err(error) => entry.status = Status::Failed(format!("{error:#}").into()),
                }
                cx.notify();
            });
            if let Err(error) = updated {
                tracing::debug!("Theme browser closed during an install: {error}");
            }
        }));
    }

    /// Show the converted theme `name` across the app, until another is tried
    /// or the preview ends.
    fn try_theme(&mut self, name: &str, converted: &ConvertedThemes, cx: &mut Context<Self>) {
        let entry = converted.document["themes"]
            .as_array()
            .and_then(|themes| themes.iter().find(|theme| theme["name"] == name));
        let config = match entry.map(|entry| serde_json::from_value::<ThemeConfig>(entry.clone())) {
            Some(Ok(config)) => Rc::new(config),
            Some(Err(error)) => {
                tracing::warn!("Cannot try {name}: {error}");
                return;
            }
            None => return,
        };
        // A preview still showing hands its snapshot on. One the app replaced
        // in the meantime is over, and the current theme is what to go back to.
        let before = match self.preview.take() {
            Some(preview) if is_showing(&preview.applied, cx) => preview.before,
            _ => Theme::global(cx).clone(),
        };
        show_theme(&config, cx);
        self.preview = Some(Preview {
            name: name.to_string(),
            applied: config,
            before,
        });
        cx.notify();
    }

    /// Put back the theme from before the preview, unless the app has chosen
    /// a theme since, which then stays.
    fn end_preview(&mut self, cx: &mut App) {
        let Some(preview) = self.preview.take() else {
            return;
        };
        restore_theme(&preview.applied, preview.before, cx);
    }

    fn previewing(&self, name: &str, cx: &App) -> bool {
        self.preview
            .as_ref()
            .is_some_and(|preview| preview.name == name && is_showing(&preview.applied, cx))
    }

    fn remove(&mut self, id: &str, cx: &mut Context<Self>) {
        let themes_dir = self.options.themes_dir.clone();
        let watched = self.options.watches_themes_dir;
        let Some(entry) = self.entry_mut(id) else {
            return;
        };
        match crate::uninstall(&themes_dir, id) {
            Ok(()) => {
                entry.installed.clear();
                // The registry has no way to drop a theme, so unless the app
                // rebuilds it from the folder, it stays until the next start.
                entry.status = if watched {
                    Status::Note("Removed.".into())
                } else {
                    Status::Note("Removed. Its themes leave the lists after a restart.".into())
                };
            }
            Err(error) => entry.status = Status::Failed(format!("{error:#}").into()),
        }
        cx.notify();
    }

    fn render_entry(
        &self,
        index: usize,
        entry: &Entry,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();
        let extension = &entry.extension;
        let id = extension.id();
        let busy = matches!(entry.status, Status::Working(_));
        let is_installed = !entry.installed.is_empty();
        let count = match (&entry.converted, &entry.kind) {
            (Some(converted), _) => Some(converted.themes.len()),
            (None, Kind::ColorThemes(count)) => *count,
            _ => None,
        };
        let variants = match count {
            Some(1) => "1 theme".to_string(),
            Some(count) => format!("{count} themes"),
            None => String::new(),
        };
        let byline = [
            format!("by {}", extension.publisher_display_name),
            format!("{} installs", short_count(extension.installs)),
            format!("v{}", extension.version),
            variants,
        ]
        .into_iter()
        .filter(|part| !part.is_empty())
        .collect::<Vec<_>>()
        .join(" · ");

        let icon = match &entry.icon {
            Some(icon) => img(icon.clone()).size(px(36.)).into_any_element(),
            None => div()
                .size(px(36.))
                .rounded(theme.radius)
                .bg(theme.muted)
                .into_any_element(),
        };

        let buttons = h_flex()
            .gap_2()
            .flex_none()
            .when(busy, |this| this.child(Spinner::new().small()))
            .when(entry.converted.is_none() && !is_installed, |this| {
                this.child(
                    Button::new(("vscode-theme-preview", index))
                        .small()
                        .outline()
                        .label("Preview")
                        .disabled(busy)
                        .on_click(cx.listener({
                            let id = id.clone();
                            move |this, _, _, cx| this.convert(id.clone(), false, cx)
                        })),
                )
            })
            .child(
                Button::new(("vscode-theme-install", index))
                    .small()
                    .when(!is_installed, |this| this.primary())
                    .when(is_installed, |this| this.outline())
                    .label(if is_installed { "Reinstall" } else { "Install" })
                    .disabled(busy)
                    .on_click(cx.listener({
                        let id = id.clone();
                        move |this, _, _, cx| this.convert(id.clone(), true, cx)
                    })),
            )
            .when(is_installed, |this| {
                this.child(
                    Button::new(("vscode-theme-remove", index))
                        .small()
                        .ghost()
                        .label("Remove")
                        .disabled(busy)
                        .on_click(cx.listener({
                            let id = id.clone();
                            move |this, _, _, cx| this.remove(&id, cx)
                        })),
                )
            });

        let status = match &entry.status {
            Status::Idle => None,
            Status::Working(what) => Some((
                SharedString::from(format!("{what}…")),
                theme.muted_foreground,
            )),
            Status::Note(note) => Some((note.clone(), theme.muted_foreground)),
            Status::Failed(error) => Some((error.clone(), theme.danger)),
        };

        v_flex()
            .id(("vscode-theme", index))
            .w_full()
            .gap_2()
            .py_2()
            .border_b_1()
            .border_color(theme.border)
            .child(
                h_flex()
                    .w_full()
                    .gap_3()
                    .items_start()
                    .child(div().flex_none().child(icon))
                    .child(
                        v_flex()
                            .flex_1()
                            .min_w_0()
                            .gap_0p5()
                            .child(
                                div()
                                    .text_sm()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(extension.display_name.clone()),
                            )
                            .child(
                                div()
                                    .text_xs()
                                    .text_color(theme.muted_foreground)
                                    .child(byline),
                            )
                            .when(!extension.description.is_empty(), |this| {
                                this.child(
                                    div()
                                        .text_xs()
                                        .text_color(theme.muted_foreground)
                                        .line_clamp(2)
                                        .child(extension.description.clone()),
                                )
                            }),
                    )
                    .child(buttons),
            )
            .when_some(status, |this, (text, color)| {
                this.child(div().text_xs().text_color(color).child(text))
            })
            .when_some(entry.converted.clone(), |this, converted| {
                this.child(self.render_variants(index, entry, &converted, cx))
            })
    }

    fn render_variants(
        &self,
        index: usize,
        entry: &Entry,
        converted: &Arc<ConvertedThemes>,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let use_theme = self.options.use_theme.clone();
        let rows: Vec<_> = converted
            .themes
            .iter()
            .map(|summary| (summary.clone(), self.previewing(&summary.name, cx)))
            .collect();
        let theme = cx.theme();
        v_flex()
            .gap_1()
            .pl(px(48.))
            .children(
                rows.into_iter()
                    .enumerate()
                    .map(|(variant, (summary, previewing))| {
                        let installed = entry
                            .installed
                            .iter()
                            .any(|(name, _)| *name == summary.name);
                        let name = summary.name.clone();
                        h_flex()
                            .gap_2()
                            .items_center()
                            .child(
                                h_flex()
                                    .flex_none()
                                    .rounded(theme.radius)
                                    .overflow_hidden()
                                    .border_1()
                                    .border_color(theme.border)
                                    .children(summary.swatches.iter().map(|color| {
                                        div().w(px(16.)).h(px(18.)).bg(gpui_color(*color))
                                    })),
                            )
                            .child(div().flex_1().min_w_0().text_sm().truncate().child(format!(
                                "{} ({})",
                                summary.name,
                                if summary.light { "light" } else { "dark" }
                            )))
                            .child(
                                Button::new(("vscode-theme-try", index * 100 + variant))
                                    .xsmall()
                                    .when(previewing, |this| this.primary())
                                    .when(!previewing, |this| this.outline())
                                    .label(if previewing { "Revert" } else { "Try" })
                                    .on_click(cx.listener({
                                        let name = name.clone();
                                        let converted = converted.clone();
                                        move |this, _, _, cx| {
                                            if previewing {
                                                this.end_preview(cx);
                                                cx.notify();
                                            } else {
                                                this.try_theme(&name, &converted, cx);
                                            }
                                        }
                                    })),
                            )
                            .when_some(
                                use_theme.clone().filter(|_| installed),
                                |this, use_theme| {
                                    this.child(
                                        Button::new(("vscode-theme-use", index * 100 + variant))
                                            .xsmall()
                                            .outline()
                                            .label("Use")
                                            .on_click(cx.listener(move |this, _, window, cx| {
                                                // The chosen theme replaces the preview
                                                // rather than ending it.
                                                this.preview = None;
                                                use_theme(&name, window, cx);
                                                cx.notify();
                                            })),
                                    )
                                },
                            )
                    }),
            )
    }
}

impl Render for ThemeBrowser {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let hidden = self
            .entries
            .iter()
            .filter(|entry| entry.kind == Kind::NoColorThemes)
            .count();
        let summary = if self.entries.is_empty() && !self.searching {
            "No themes found".to_string()
        } else {
            let mut summary = format!(
                "{} of {} extensions",
                self.entries.len(),
                short_count(self.total)
            );
            if hidden > 0 {
                summary.push_str(&format!(", {hidden} icon themes hidden"));
            }
            summary
        };
        let more = (self.entries.len() as u64) < self.total;
        let muted = theme.muted_foreground;
        let danger = theme.danger;

        let rows: Vec<_> = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.kind != Kind::NoColorThemes)
            .map(|(index, entry)| self.render_entry(index, entry, cx).into_any_element())
            .collect();

        v_flex()
            .w_full()
            .gap_2()
            .child(
                h_flex()
                    .gap_2()
                    .items_center()
                    .child(
                        div()
                            .flex_1()
                            .child(Input::new(&self.search_input).small().cleanable(true)),
                    )
                    .when(self.searching, |this| this.child(Spinner::new().small())),
            )
            .child(div().text_xs().text_color(muted).child(summary))
            .when_some(self.search_error.clone(), |this, error| {
                this.child(div().text_sm().text_color(danger).child(error))
            })
            .children(rows)
            .when(more, |this| {
                this.child(
                    h_flex().justify_center().child(
                        Button::new("vscode-theme-more")
                            .small()
                            .outline()
                            .label("Load more")
                            .disabled(self.searching)
                            .on_click(cx.listener(|this, _, _, cx| this.search(true, cx))),
                    ),
                )
            })
    }
}

/// Show `config` across the app. A converted theme names no fonts, so the
/// fonts the app set stay.
fn show_theme(config: &Rc<ThemeConfig>, cx: &mut App) {
    Theme::update(cx, |theme| {
        theme.apply_config(config);
        theme.mode = config.mode;
    });
}

/// Put `before` back if `applied` is still the theme on screen. False when
/// the app chose a theme since, which then stays.
fn restore_theme(applied: &Rc<ThemeConfig>, before: Theme, cx: &mut App) -> bool {
    if !is_showing(applied, cx) {
        return false;
    }
    Theme::update(cx, |theme| *theme = before);
    true
}

/// Whether `config` is the theme on screen. A theme the app applies after a
/// preview, from the registry or its settings, is a different config.
fn is_showing(config: &Rc<ThemeConfig>, cx: &App) -> bool {
    let theme = Theme::global(cx);
    let active = if theme.mode.is_dark() {
        &theme.dark_theme
    } else {
        &theme.light_theme
    };
    Rc::ptr_eq(active, config)
}

/// Add the installed themes to the registry so the theme lists offer them at
/// once. True when one of the names was taken: the registry keeps the first
/// theme of a name, so that one is replaced only at the next start.
fn register(converted: &ConvertedThemes, cx: &mut App) -> anyhow::Result<bool> {
    let registry = ThemeRegistry::global_mut(cx);
    let taken = converted.themes.iter().any(|theme| {
        registry
            .themes()
            .contains_key(&SharedString::from(theme.name.clone()))
    });
    registry.load_themes_from_str(&converted.to_json())?;
    cx.refresh_windows();
    Ok(taken)
}

fn image(bytes: Vec<u8>) -> Option<Arc<Image>> {
    let format = match bytes.as_slice() {
        [0x89, b'P', b'N', b'G', ..] => ImageFormat::Png,
        [0xff, 0xd8, ..] => ImageFormat::Jpeg,
        [b'G', b'I', b'F', ..] => ImageFormat::Gif,
        [
            b'R',
            b'I',
            b'F',
            b'F',
            _,
            _,
            _,
            _,
            b'W',
            b'E',
            b'B',
            b'P',
            ..,
        ] => ImageFormat::Webp,
        bytes if bytes.trim_ascii_start().starts_with(b"<") => ImageFormat::Svg,
        _ => return None,
    };
    Some(Arc::new(Image::from_bytes(format, bytes)))
}

fn gpui_color(color: Rgba) -> gpui::Rgba {
    gpui::Rgba {
        r: color.red,
        g: color.green,
        b: color.blue,
        a: color.alpha,
    }
}

fn short_count(count: u64) -> String {
    match count {
        0..1_000 => count.to_string(),
        1_000..1_000_000 => format!("{:.0}K", count as f64 / 1_000.0),
        _ => format!("{:.1}M", count as f64 / 1_000_000.0),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(name: &str, dark: bool, background: &str) -> Rc<ThemeConfig> {
        let mode = if dark { "dark" } else { "light" };
        Rc::new(
            serde_json::from_value(serde_json::json!({
                "name": name,
                "mode": mode,
                "colors": {"background": background}
            }))
            .unwrap(),
        )
    }

    #[gpui::test]
    fn a_preview_shows_a_theme_and_puts_the_old_one_back(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            gpui_component::init(cx);
            // As an app does on top of its theme file.
            Theme::update(cx, |theme| theme.font_family = "My Font".into());
            let before = Theme::global(cx).clone();

            let tried = config("Tried", true, "#123456");
            show_theme(&tried, cx);
            let theme = Theme::global(cx);
            assert!(theme.mode.is_dark());
            assert_eq!(theme.background, gpui::Hsla::from(gpui::rgb(0x123456)));
            assert_eq!(theme.font_family.as_ref(), "My Font");

            assert!(restore_theme(&tried, before.clone(), cx));
            let theme = Theme::global(cx);
            assert_eq!(theme.mode, before.mode);
            assert_eq!(theme.background, before.background);
            assert_eq!(theme.font_family.as_ref(), "My Font");
        });
    }

    #[gpui::test]
    fn a_theme_chosen_during_a_preview_stays(cx: &mut gpui::TestAppContext) {
        cx.update(|cx| {
            gpui_component::init(cx);
            let before = Theme::global(cx).clone();
            let tried = config("Tried", true, "#123456");
            show_theme(&tried, cx);
            // The app applies a theme of its own, as Use or a setting does.
            let chosen = config("Chosen", false, "#abcdef");
            show_theme(&chosen, cx);

            assert!(!restore_theme(&tried, before, cx));
            assert_eq!(
                Theme::global(cx).background,
                gpui::Hsla::from(gpui::rgb(0xabcdef))
            );
        });
    }

    #[test]
    fn counts_are_short() {
        assert_eq!(short_count(950), "950");
        assert_eq!(short_count(126_954), "127K");
        assert_eq!(short_count(2_989_383), "3.0M");
    }

    #[test]
    fn icon_formats_are_sniffed() {
        assert!(matches!(
            image(b"\x89PNG....".to_vec()).map(|i| i.format),
            Some(ImageFormat::Png)
        ));
        assert!(matches!(
            image(b"  <svg/>".to_vec()).map(|i| i.format),
            Some(ImageFormat::Svg)
        ));
        assert!(image(b"nope".to_vec()).is_none());
    }
}
