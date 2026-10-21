use ely_gpui_component::{
    buttons::{Button, SegmentedControl},
    feedback::InlineMessage,
    forms::{Copy, NumberInput, Paste, PathInput, SelectAll, Switch},
    primitives::Severity,
    settings::{KeyboardShortcutsList, SettingsRow, SettingsSection, Shortcut},
    theme::{ActiveTheme, ControlSize},
};
use gpui::{Action, App, Div, Entity, KeyContext, ParentElement, SharedString, Styled, div};

use super::{OpenSettings, SettingsWindow, choice, set, switch};
use crate::{
    menus::{Bigger, Minimize, Quit, ResetSize, Smaller},
    settings::{OptionAsAlt, SCROLLBACK, SettingsStore, StartIn, language},
    term::{
        input::{self, Find, NextPrompt, PreviousPrompt},
        shell::{ShellIntegration, Shells, repair_installed},
    },
    updates::{self, Updates, VERSION},
    workspace::{
        self, CloseSession, LastSession, NewSession, NextSession, PreviousSession, SelectSession,
        TogglePanel, ToggleSessions,
    },
};

/// The names of the sessions a digit shows.
const PLACES: [&str; 8] = [
    "Session 1",
    "Session 2",
    "Session 3",
    "Session 4",
    "Session 5",
    "Session 6",
    "Session 7",
    "Session 8",
];

impl SettingsWindow {
    /// Shell, start in, scrollback, Option as Alt on macOS, copy on select, Ely and updates, then the shortcuts.
    pub(super) fn terminal(&self, this: &Entity<Self>, cx: &App) -> Div {
        let settings = SettingsStore::get(cx);
        let names: Vec<String> = cx
            .global::<Shells>()
            .0
            .iter()
            .map(|shell| shell.name.clone())
            .collect();
        let found = if names.is_empty() {
            "No local shell found; the login shell starts".to_string()
        } else {
            format!("Found: {}", names.join(", "))
        };
        let shell = div()
            .w(cx.theme().prose_width())
            .flex()
            .flex_col()
            .gap_1()
            .child(PathInput::new("settings-shell", &self.shell))
            .children((self.refused.clone()).map(|why| InlineMessage::new(Severity::Danger, why)));
        let start = match settings.start_in {
            StartIn::Home => "home",
            StartIn::Session => "session",
            StartIn::Fixed => "fixed",
            StartIn::Last => "last",
        };
        let start = SegmentedControl::new("settings-start-in", start)
            .segment(
                "home",
                language::message(cx, "settings.terminal.start.home"),
                None,
            )
            .segment(
                "session",
                language::message(cx, "settings.terminal.start.session"),
                None,
            )
            .segment(
                "fixed",
                language::message(cx, "settings.terminal.start.fixed"),
                None,
            )
            .segment(
                "last",
                language::message(cx, "settings.terminal.start.last"),
                None,
            )
            .size(ControlSize::Sm)
            .on_change({
                let this = this.clone();
                move |value, _, cx| start_in(&this, value, cx)
            });
        let fixed = div()
            .w(cx.theme().prose_width())
            .flex()
            .flex_col()
            .gap_1()
            .child(PathInput::new("settings-start-folder", &self.start_folder));
        let lines = NumberInput::new(("scrollback", self.drafts), f64::from(settings.scrollback))
            .range(f64::from(*SCROLLBACK.start()), f64::from(*SCROLLBACK.end()))
            .step(1000.)
            .size(ControlSize::Sm)
            .on_commit({
                let this = this.clone();
                move |lines, _, cx| {
                    let lines = lines as u32;
                    set(&this, format!("scrollback is {lines}"), cx, |settings| {
                        settings.scrollback = lines
                    });
                }
            });
        let repair = Button::new("repair-integration", "Reinstall")
            .size(ControlSize::Sm)
            .on_click({
                let this = this.clone();
                move |_, _, cx| {
                    let result = repair_installed(cx);
                    this.update(cx, |window, cx| {
                        window.failed = match &result {
                            Ok(()) => {
                                log::info!("shell integration: the scripts are written again");
                                None
                            }
                            Err(error) => Some(format!("{error:#}").into()),
                        };
                        cx.notify();
                    });
                }
            });
        let integration = SettingsRow::new("Shell integration")
            .description(cx.global::<ShellIntegration>().0.display().to_string())
            .control(repair);
        let mut section = SettingsSection::new(language::message(cx, "settings.terminal.section"))
            .row(
                SettingsRow::new(language::message(cx, "settings.terminal.shell"))
                    .description(found)
                    .control(shell),
            )
            .row(integration)
            .row(SettingsRow::new(language::message(cx, "settings.terminal.start")).control(start))
            .row(
                SettingsRow::new(language::message(cx, "settings.terminal.start.folder"))
                    .description(language::message(cx, "settings.terminal.start.folder.hint"))
                    .control(fixed),
            )
            .row(
                SettingsRow::new(language::message(cx, "settings.terminal.scrollback"))
                    .control(lines),
            );
        // Off macOS, Alt is always Alt.
        if cfg!(target_os = "macos") {
            let option = choice(
                this,
                "option_as_alt",
                &[
                    (
                        OptionAsAlt::Off,
                        "off",
                        language::message(cx, "settings.terminal.option.off"),
                    ),
                    (
                        OptionAsAlt::Left,
                        "left",
                        language::message(cx, "settings.terminal.option.left"),
                    ),
                    (
                        OptionAsAlt::Right,
                        "right",
                        language::message(cx, "settings.terminal.option.right"),
                    ),
                    (
                        OptionAsAlt::Both,
                        "both",
                        language::message(cx, "settings.terminal.option.both"),
                    ),
                ],
                settings.option_as_alt,
                |settings, option| settings.option_as_alt = option,
            );
            section = section.row(
                SettingsRow::new(language::message(cx, "settings.terminal.option")).control(option),
            );
        }
        let copy = switch(
            this,
            "copy_on_select",
            settings.copy_on_select,
            |settings, on| settings.copy_on_select = on,
        );
        let ely = switch(this, "ely", settings.ely, |settings, on| settings.ely = on);
        let section = section
            .row(SettingsRow::new(language::message(cx, "settings.terminal.copy")).control(copy))
            .row(
                SettingsRow::new(language::message(cx, "settings.terminal.ely"))
                    .description("Peeks over the window, her face as your sessions are")
                    .control(ely),
            )
            .row(
                SettingsRow::new(language::message(cx, "settings.terminal.updates"))
                    .description("At start and once a day")
                    .control(auto_check(this, settings.check_updates)),
            )
            .row(updates_row(cx));
        let keys = SettingsSection::new(language::message(cx, "settings.terminal.shortcuts"))
            .row(KeyboardShortcutsList::new(shortcuts(cx), &self.search));
        div().flex().flex_col().gap_6().child(section).child(keys)
    }
}

/// The switch for automatic checks; turning it on checks at once.
fn auto_check(this: &Entity<SettingsWindow>, on: bool) -> Switch {
    let this = this.clone();
    Switch::new("check_updates", on).on_change(move |on, _, cx| {
        let change = format!("check_updates is {on}");
        if set(&this, change, cx, |settings| settings.check_updates = on) && on {
            updates::check(false, cx);
        }
    })
}

/// What the last check found, a button that checks now, and one that opens a newer release's page.
fn updates_row(cx: &App) -> SettingsRow {
    let found = cx.global::<Updates>();
    let status: SharedString = match found {
        Updates::Unchecked => format!("ELYTerminal {VERSION}").into(),
        Updates::Checking => "Checking for a newer release…".into(),
        Updates::Latest => format!("ELYTerminal {VERSION} is the latest").into(),
        Updates::Newer { version, .. } => {
            format!("ELYTerminal {version} is out; this is {VERSION}").into()
        }
        Updates::Failed(why) => format!("The check failed: {why}").into(),
    };
    let page = match found {
        Updates::Newer { page, .. } => Some(page.clone()),
        _ => None,
    };
    let check = Button::new("check-updates", "Check now")
        .size(ControlSize::Sm)
        .loading(*found == Updates::Checking)
        .on_click(|_, _, cx| updates::check(false, cx));
    let open = page.map(|page| {
        Button::new("open-update", "Download")
            .primary()
            .size(ControlSize::Sm)
            .on_click(move |_, _, cx| {
                log::info!("updates: Settings opens {page}");
                cx.open_url(&page);
            })
    });
    let control = div()
        .flex()
        .items_center()
        .gap_2()
        .children(open)
        .child(check);
    SettingsRow::new("Updates")
        .description(status)
        .control(control)
}

/// Where new sessions start, as the settings keep it.
fn start_in(this: &Entity<SettingsWindow>, value: &SharedString, cx: &mut App) {
    let start = match value.as_ref() {
        "home" => StartIn::Home,
        "session" => StartIn::Session,
        "fixed" => StartIn::Fixed,
        "last" => StartIn::Last,
        other => unreachable!("settings: no start {other}"),
    };
    set(
        this,
        format!("new sessions start in {start:?}"),
        cx,
        |settings| settings.start_in = start,
    );
}

// The listed commands, in order: group, name and action. Built once, as each action is a heap.
thread_local! {
    static LISTED: Vec<(&'static str, &'static str, Box<dyn Action>)> = {
        let mut listed = vec![
            ("Sessions", "New session", NewSession.boxed_clone()),
            ("Sessions", "Close session", CloseSession.boxed_clone()),
            ("Sessions", "Next session", NextSession.boxed_clone()),
            (
                "Sessions",
                "Previous session",
                PreviousSession.boxed_clone(),
            ),
        ];
        let places = PLACES.iter().enumerate();
        listed.extend(places.map(|(ix, name)| ("Sessions", *name, SelectSession(ix).boxed_clone())));
        listed.extend([
            ("Sessions", "Last session", LastSession.boxed_clone()),
            ("Edit", "Copy", Copy.boxed_clone()),
            ("Edit", "Paste", Paste.boxed_clone()),
            ("Edit", "Select all", SelectAll.boxed_clone()),
            ("Edit", "Find", Find.boxed_clone()),
            ("View", "Previous prompt", PreviousPrompt.boxed_clone()),
            ("View", "Next prompt", NextPrompt.boxed_clone()),
            ("View", "Toggle sessions", ToggleSessions.boxed_clone()),
            ("View", "Toggle right panel", TogglePanel.boxed_clone()),
            ("View", "Bigger", Bigger.boxed_clone()),
            ("View", "Smaller", Smaller.boxed_clone()),
            ("View", "Reset size", ResetSize.boxed_clone()),
            ("App", "Settings", OpenSettings.boxed_clone()),
            ("App", "Minimize", Minimize.boxed_clone()),
            ("App", "Quit", Quit.boxed_clone()),
        ]);
        listed
    };
}

/// Each listed command's keys in the main window's terminal, a line per binding, from the keymap.
fn shortcuts(cx: &App) -> Vec<Shortcut> {
    let stack = [workspace::CONTEXT, input::CONTEXT]
        .map(|name| KeyContext::parse(name).expect("a context name parses"));
    let keymap = cx.key_bindings();
    let keymap = keymap.borrow();
    let mut shortcuts = Vec::new();
    LISTED.with(|listed| {
        for (group, name, action) in listed {
            for binding in keymap.bindings_for_action(action.as_ref()) {
                // gpui matches a binding at any depth of the context stack.
                let applies = binding.predicate().is_none_or(|predicate| {
                    (1..=stack.len()).any(|depth| predicate.eval(&stack[..depth]))
                });
                if applies {
                    let strokes: Vec<String> = binding
                        .keystrokes()
                        .iter()
                        .map(|key| key.unparse())
                        .collect();
                    shortcuts.push(Shortcut {
                        group: (*group).into(),
                        name: (*name).into(),
                        keys: strokes.join(" ").into(),
                    });
                }
            }
        }
    });
    shortcuts
}
