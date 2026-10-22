use std::fmt::Debug;

use ely_gpui_component::{
    buttons::SegmentedControl,
    feedback::InlineMessage,
    forms::{InputEvent, Switch, TextInput},
    lists::{ListItem, SelectableList},
    primitives::{FocusScope, Icon, IconName, Severity},
    shell::{ColumnShell, ResizeBorder, ShellColumn},
    theme::{ActiveTheme, ControlSize, IconSize},
};
use gpui::{
    AnyElement, App, AppContext, Bounds, Context, Entity, FocusHandle, InteractiveElement,
    IntoElement, KeyBinding, ParentElement, Render, SharedString, StatefulInteractiveElement,
    StyleRefinement, Styled, Subscription, Task, TitlebarOptions, Window,
    WindowBackgroundAppearance, WindowBounds, WindowDecorations, WindowOptions, actions, div,
    prelude::FluentBuilder, px, size,
};

use super::{Settings, SettingsStore};
use crate::settings::CursorStyle;
use crate::{app::APP_ID, menus, settings::language, term::BLINK, updates::Updates};

mod appearance;
mod custom;
mod general;
mod preview;
mod terminal;
mod text;
mod theme_import;
mod themegrid;

actions!(elyterminal, [OpenSettings, CloseSettings]);

/// The key context of the settings window's content.
const CONTEXT: &str = "Settings";

/// Each category's key and its icon; the name comes from the catalog.
const SECTIONS: [(&str, IconName); 3] = [
    ("appearance", IconName::Palette),
    ("terminal", IconName::Terminal),
    ("general", IconName::Settings),
];

/// Cmd-, opens the settings, Cmd-W closes them; off macOS Ctrl-, and Ctrl-Shift-W, as Ghostty's.
pub fn bind_keys(cx: &mut App) {
    let (open, close) = if cfg!(target_os = "macos") {
        ("cmd-,", "cmd-w")
    } else {
        ("ctrl-,", "ctrl-shift-w")
    };
    cx.bind_keys([
        KeyBinding::new(open, OpenSettings, None),
        KeyBinding::new(close, CloseSettings, Some(CONTEXT)),
    ]);
    cx.on_action(|_: &OpenSettings, cx| open_window(cx));
}

/// Opens the settings window, or brings the open one forward.
fn open_window(cx: &mut App) {
    let open = cx
        .windows()
        .into_iter()
        .find_map(|window| window.downcast::<SettingsWindow>());
    if let Some(window) = open {
        log::info!("settings: the window comes forward");
        let forward = window.update(cx, |_, window, _| window.activate_window());
        if let Err(error) = forward {
            log::error!("settings: the window did not come forward: {error:#}");
        }
        return;
    }
    let bounds = Bounds::centered(None, size(px(900.0), px(640.0)), cx);
    let title = language::message(cx, "settings.window");
    let options = WindowOptions {
        window_bounds: Some(WindowBounds::Windowed(bounds)),
        titlebar: Some(TitlebarOptions {
            title: Some(title.into()),
            appears_transparent: true,
            traffic_light_position: Some(cx.theme().traffic_light_origin()),
        }),
        // Ely's column draws the controls; only Linux reads this.
        window_decorations: Some(WindowDecorations::Client),
        // The settings window stays opaque, as upstream decided: a blurred one costs the compositor a pass over the desktop behind it, every frame.
        window_background: WindowBackgroundAppearance::Opaque,
        ..Default::default()
    };
    match cx.open_window(options, |window, cx| {
        window.set_app_id(APP_ID);
        cx.new(|cx| SettingsWindow::new(window, cx))
    }) {
        Ok(_) => log::info!("settings: the window opens"),
        Err(error) => log::error!("settings: the window did not open: {error:#}"),
    }
}

/// The settings window: categories in one column, the open one's rows in the next, each scrolling on its own.
pub struct SettingsWindow {
    focus: FocusHandle,
    section: SharedString,
    shell: Entity<TextInput>,
    /// The command files open in.
    editor: Entity<TextInput>,
    /// The template the window's title is composed from.
    window_title: Entity<TextInput>,
    /// The start folder a fixed start reads.
    start_folder: Entity<TextInput>,
    /// The shortcut list's search.
    search: Entity<TextInput>,
    /// Why the typed shell is not saved.
    refused: Option<SharedString>,
    /// Why the last change did not save.
    failed: Option<SharedString>,
    /// Refusals so far; a field keyed by it drops its draft and shows the settings again.
    drafts: usize,
    font: Entity<TextInput>,
    /// A picker's input for each fallback family, and one for the family being added.
    fallbacks: Vec<Entity<TextInput>>,
    /// The empty picker that adds a fallback shows.
    adding: bool,
    /// Whether the cursor preview shows, as it blinks.
    lit: bool,
    /// The open page, in a view of its own: a scroll reuses its layout instead of building fifteen theme cards again.
    page: Option<Entity<PageView>>,
    _typed: Subscription,
    _start: Subscription,
    _editor: Subscription,
    _title: Subscription,
    _settings: Subscription,
    _updates: Subscription,
    _blink: Task<()>,
}

/// The open page, drawn apart from the window's frame so the frame's scroll does not rebuild it.
pub struct PageView {
    settings: Entity<SettingsWindow>,
}

impl Render for PageView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let settings = self.settings.clone();
        settings.read(cx).page(&settings, window, cx)
    }
}

impl SettingsWindow {
    fn new(window: &mut Window, cx: &mut Context<Self>) -> Self {
        let shell = cx.new(|cx| {
            let mut input = TextInput::new(window, cx)
                .placeholder(language::message(cx, "settings.terminal.shell.hint"));
            input.set_text(SettingsStore::get(cx).shell.clone(), cx);
            input
        });
        let start_folder = cx.new(|cx| {
            let mut input = TextInput::new(window, cx)
                .placeholder(language::message(cx, "settings.terminal.start.folder"));
            input.set_text(SettingsStore::get(cx).start_in_folder.clone(), cx);
            input
        });
        let (editor, editor_typed) = text::field(
            window,
            cx,
            "editor",
            language::message(cx, "settings.terminal.editor.hint"),
            SettingsStore::get(cx).editor.clone(),
            |settings, text| settings.editor = text,
        );
        let (window_title, title_typed) = text::field(
            window,
            cx,
            "window_title",
            crate::settings::title::DEFAULT.to_string(),
            SettingsStore::get(cx).window_title.clone(),
            |settings, text| settings.window_title = text,
        );

        let font = cx.new(|cx| TextInput::new(window, cx));
        let search = cx.new(|cx| {
            TextInput::new(window, cx)
                .placeholder(language::message(cx, "settings.terminal.search"))
        });
        let start_typed = cx.subscribe(&start_folder, |this, input, event: &InputEvent, cx| {
            if let InputEvent::Changed = event {
                let folder = input.read(cx).text().to_string();
                let saved =
                    SettingsStore::update(cx, |settings| settings.start_in_folder = folder.clone());
                if let Err(error) = saved {
                    log::debug!("settings: the start folder stays: {error:#}");
                }
                this.paged(cx);
            }
        });
        let typed = cx.subscribe(&shell, |this, input, event: &InputEvent, cx| {
            if let InputEvent::Changed = event {
                let shell = input.read(cx).text().to_string();
                this.save_shell(shell, cx);
            }
        });
        // Focus inside, so the window's keys find its context.
        let focus = cx.focus_handle();
        window.focus(&focus, cx);
        Self {
            focus,
            section: SECTIONS[0].0.into(),
            shell,
            start_folder,
            editor,
            window_title,
            search,
            refused: None,
            failed: None,
            drafts: 0,
            font,
            fallbacks: Vec::new(),
            adding: false,
            lit: true,
            page: None,
            _typed: typed,
            _start: start_typed,
            _editor: editor_typed,
            _title: title_typed,
            _settings: cx.observe_global_in::<SettingsStore>(window, |this, window, cx| {
                this.grow_fallbacks(window, cx);
                this.paged(cx);
            }),
            _updates: cx.observe_global::<Updates>(|this, cx| this.paged(cx)),
            _blink: Self::blink(cx),
        }
    }

    /// Shows the page again, as a change it draws has landed.
    fn paged(&self, cx: &mut Context<Self>) {
        if let Some(page) = &self.page {
            let page = page.clone();
            page.update(cx, |_, cx| cx.notify());
        }
        cx.notify();
    }

    /// Blinks the cursor preview as a terminal blinks its cursor, while it shows.
    fn blink(cx: &mut Context<Self>) -> Task<()> {
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor().timer(BLINK).await;
                let ticked = this.update(cx, |this, cx| {
                    if this.section == "appearance" && SettingsStore::get(cx).cursor_blink {
                        this.lit = !this.lit;
                        this.paged(cx);
                    }
                });
                if ticked.is_err() {
                    break;
                }
            }
        })
    }

    /// Saves `shell` when it names a program; else says why and keeps the saved one.
    fn save_shell(&mut self, shell: String, cx: &mut Context<Self>) {
        self.refused = match SettingsStore::update(cx, |settings| settings.shell = shell.clone()) {
            Ok(()) => {
                log::info!("settings: new sessions run {shell:?}");
                self.failed = None;
                None
            }
            Err(error) => {
                log::debug!("settings: the shell stays: {error:#}");
                Some(format!("{error:#}").into())
            }
        };
        cx.notify();
    }
}

/// The categories; a press, Up and Down, or Enter opens one.
fn sections(open: &SharedString, cx: &Context<SettingsWindow>) -> SelectableList {
    let muted = cx.theme().colors.fg_muted;
    let list = SECTIONS.iter().fold(
        SelectableList::new("settings-section-list").selected([open.clone()]),
        |list, (key, icon)| {
            let name = language::message(cx, &format!("settings.category.{key}"));
            let item = ListItem::new(SharedString::from(format!("settings-section-{key}")), name)
                .leading(Icon::new(*icon).size(IconSize::Sm).color(muted));
            list.row(*key, item)
        },
    );
    let show = {
        let this = cx.entity();
        move |key: &SharedString, cx: &mut App| {
            log::info!("settings: open {key}");
            this.update(cx, |this, cx| {
                this.section = key.clone();
                this.paged(cx);
            });
        }
    };
    let enter = show.clone();
    list.on_change(move |keys, _, cx| {
        if let Some(key) = keys.first() {
            show(key, cx);
        }
    })
    .on_activate(move |key, _, cx| enter(key, cx))
}

/// A switch that saves through `put`.
fn switch(
    this: &Entity<SettingsWindow>,
    field: &'static str,
    on: bool,
    put: fn(&mut Settings, bool),
) -> Switch {
    let this = this.clone();
    Switch::new(field, on).on_change(move |on, _, cx| {
        set(&this, format!("{field} is {on}"), cx, |settings| {
            put(settings, on)
        });
    })
}

/// The cursor's style, in the language the interface is in.
fn cursor_style(this: &Entity<SettingsWindow>, now: CursorStyle, cx: &App) -> SegmentedControl {
    choice(
        this,
        "cursor_style",
        &[
            (
                CursorStyle::Block,
                "block",
                language::message(cx, "settings.cursor.block"),
            ),
            (
                CursorStyle::Bar,
                "bar",
                language::message(cx, "settings.cursor.bar"),
            ),
            (
                CursorStyle::Underline,
                "underline",
                language::message(cx, "settings.cursor.underline"),
            ),
            (
                CursorStyle::Hollow,
                "hollow",
                language::message(cx, "settings.cursor.hollow"),
            ),
        ],
        now,
        |settings, style| settings.cursor_style = style,
    )
}

/// Segments for `choices`, each a value, its key and its name, that save the chosen value through `put`.
fn choice<T: Copy + PartialEq + Debug + 'static>(
    this: &Entity<SettingsWindow>,
    field: &'static str,
    choices: &[(T, &'static str, String)],
    now: T,
    put: fn(&mut Settings, T),
) -> SegmentedControl {
    let (_, key, _) = choices
        .iter()
        .find(|(each, ..)| *each == now)
        .expect("every choice is offered");
    let this = this.clone();
    let mut control = SegmentedControl::new(field, *key);
    for (_, key, name) in choices {
        control = control.segment(*key, name.clone(), None);
    }
    // The choices move into the handler, which outlives this call.
    let owned: Vec<(T, &'static str, String)> = choices.to_vec();
    control.size(ControlSize::Sm).on_change(move |key, _, cx| {
        let (value, ..) = owned
            .iter()
            .find(|(_, each, _)| *each == key.as_ref())
            .expect("a choice the control offers");
        let value = *value;
        set(&this, format!("{field} is {value:?}"), cx, |settings| {
            put(settings, value)
        });
    })
}

/// Saves one change and logs it, telling whether the settings hold it; a refused one changes nothing, and the window says why.
fn set(
    this: &Entity<SettingsWindow>,
    change: String,
    cx: &mut App,
    edit: impl FnOnce(&mut Settings),
) -> bool {
    let mut next = SettingsStore::get(cx).clone();
    edit(&mut next);
    if next == *SettingsStore::get(cx) {
        return true;
    }
    let failed = match SettingsStore::update(cx, |settings| *settings = next) {
        Ok(()) => {
            log::info!("settings: {change}");
            None
        }
        Err(error) => {
            log::error!("settings: refused {change}: {error:#}");
            Some(format!("{error:#}").into())
        }
    };
    let saved = failed.is_none();
    this.update(cx, |this, cx| {
        this.drafts += usize::from(!saved);
        this.failed = failed;
        cx.notify();
    });
    saved
}

impl SettingsWindow {
    /// The open page, with the failure it shows above it. Drawn by `PageView`, so a scroll of the window reuses it.
    fn page(&self, this: &Entity<SettingsWindow>, window: &Window, cx: &App) -> AnyElement {
        let page = match self.section.as_ref() {
            "appearance" => self.appearance(this, window, cx).into_any_element(),
            "terminal" => self.terminal(this, cx).into_any_element(),
            "general" => self.general(this, cx).into_any_element(),
            other => unreachable!("settings: no section {other}"),
        };
        div()
            .flex()
            .flex_col()
            .gap_4()
            .children((self.failed.clone()).map(|why| InlineMessage::new(Severity::Danger, why)))
            .child(page)
            .into_any_element()
    }
}

impl Render for SettingsWindow {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        window.set_rem_size(px(SettingsStore::get(cx).ui_font_size));
        // Built on the first frame, as it needs this window's own handle.
        let page = match &self.page {
            Some(page) => page.clone(),
            None => {
                let settings = cx.entity();
                let page = cx.new(|_| PageView { settings });
                self.page = Some(page.clone());
                page
            }
        };
        let body = div()
            .id("settings-body")
            .size_full()
            .overflow_y_scroll()
            .p_6()
            .child(page.cached(StyleRefinement::default().size_full()));
        let (key, _) = SECTIONS
            .iter()
            .find(|(key, ..)| *key == self.section.as_ref())
            .expect("the open section is listed");
        let name = language::message(cx, &format!("settings.category.{key}"));
        let list = div()
            .id("settings-list")
            .size_full()
            .overflow_y_scroll()
            .p_2()
            .child(sections(&self.section, cx));
        // The categories and the page scroll apart.
        let shell = ColumnShell::new("settings")
            .column(
                ShellColumn::new("settings-sections")
                    .width(cx.theme().label_width() * 1.25)
                    .title(language::message(cx, "settings.window"))
                    .child(list),
            )
            .column(ShellColumn::new("settings-page").title(name).child(body));
        // The context sits above the focus root, so its keys work wherever focus is.
        div()
            .size_full()
            .key_context(CONTEXT)
            .map(menus::window_actions)
            .on_action(|_: &CloseSettings, window, _| {
                log::info!("settings: the window closes");
                window.remove_window();
            })
            .on_action(|_: &OpenSettings, window, _| {
                log::info!("settings: the window comes forward");
                window.activate_window();
            })
            .child(
                ResizeBorder::new()
                    .child(FocusScope::new(&self.focus).root().size_full().child(shell)),
            )
    }
}
