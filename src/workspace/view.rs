//! How the window's content is drawn: the tab bar, the panes and the columns.

use std::time::Instant;

use super::chrome::{column_toggle, show_tab};

use ely_gpui_component::{
    buttons::{Button, ButtonVariant, IconButton, SegmentedControl},
    forms::Input,
    lists::{List, ListItem},
    primitives::{Disclosure, FocusScope, IconName},
    shell::{ColumnShell, ResizeBorder, ShellColumn},
    theme::{ControlSize, IconSize},
    typography::Caption,
};
use gpui::{
    AnyElement, App, ClipboardItem, Context, Entity, IntoElement, Render, SharedString,
    StyleRefinement, Styled, Window, div, prelude::*, px,
};

use super::{
    CONTEXT, CloneSession, ClosePane, CloseSession, CloseTab, EvenPanes, FocusPane, LastSession,
    LastTab, MoveTabDown, MoveTabUp, NewSession, NewTab, NextSession, NextTab, NextUsed, PinTab,
    PreviousSession, PreviousTab, PreviousUsed, SelectSession, SelectTab, Side, SplitDown,
    SplitRight, SwapPane, TogglePanel, ToggleSessions, Workspace, Zen, ZoomPane, badge, status,
};
use crate::menus;
use crate::sessions::{self, Session};
use crate::settings::window::OpenSettings;
use crate::settings::{PanelTab, Settings, SettingsStore, language};
use crate::term::Failed;
use crate::workspace::TITLE;

/// The new-tab button: its main press opens the default session, and its arrow lists each shell the scan found and each ssh host.
fn new_button(workspace: &Entity<Workspace>, cx: &App) -> impl IntoElement + use<> {
    use ely_gpui_component::menus::{Menu, MenuItem, SplitButton};
    let mut menu = Menu::new();
    let adder = workspace.clone();
    menu = menu.item(
        MenuItem::new(language::message(cx, "workspace.new_tab"))
            .on_click(move |window, cx| adder.update(cx, |this, cx| this.new_tab(window, cx))),
    );
    let shells = cx.global::<crate::term::shell::Shells>().0.clone();
    if !shells.is_empty() {
        menu = menu.separator();
    }
    for found in shells {
        let label = match &found.version {
            Some(version) => format!("{} ({version})", found.name),
            None => found.name.clone(),
        };
        let adder = workspace.clone();
        menu = menu.item(MenuItem::new(label).on_click(move |window, cx| {
            adder.update(cx, |this, cx| this.new_tab_with(&found, window, cx))
        }));
    }
    let hosts = cx.global::<crate::ssh::Hosts>().hosts.clone();
    if !hosts.is_empty() {
        menu = menu.separator();
    }
    for host in hosts {
        let name = host.name.clone();
        let adder = workspace.clone();
        menu = menu.item(
            MenuItem::new(host.name.clone()).on_click(move |window, cx| {
                adder.update(cx, |this, cx| this.connect(name.clone(), window, cx))
            }),
        );
    }
    let adder = workspace.clone();
    SplitButton::new("new-tab", "+", menu)
        .variant(ButtonVariant::Ghost)
        .on_click(move |_, window, cx| adder.update(cx, |this, cx| this.new_tab(window, cx)))
}

impl Workspace {
    /// The sessions column: a row for each tab, and under the shown one a row for each of its panes.
    fn tabs_column(&self, workspace: &Entity<Workspace>, cx: &App) -> impl IntoElement {
        let home = dirs::home_dir();
        let now = Instant::now();
        let query = self.filter.text.trim();
        let mut rows: Vec<AnyElement> = Vec::new();
        for (at, tab) in self.tabs.iter().enumerate() {
            let shown = at == self.active_tab;
            let title = tab
                .focused_session()
                .and_then(|id| self.session(id))
                .map_or(TITLE, Session::named);
            let title = tab.named(title, TITLE).to_string();
            let tab_matches = sessions::matches(query, &title, "");
            let visible: Vec<_> = tab
                .sessions()
                .into_iter()
                .filter(|id| {
                    tab_matches
                        || self.session(*id).is_some_and(|session| {
                            sessions::matches(
                                query,
                                session.named(),
                                &sessions::shown(&session.folder, home.as_deref()),
                            )
                        })
                })
                .collect();
            if !query.is_empty() && !tab_matches && visible.is_empty() {
                continue;
            }
            let select = workspace.clone();
            let close = workspace.clone();
            // A tab names its own sessions, so it shows how many it holds; folded, that count is all that is left.
            let count = tab.panes.len();
            let folded = tab.collapsed;
            let badge = match (tab.pinned, count > 1 || folded) {
                (true, true) => format!("\u{2605} {count}"),
                (true, false) => "\u{2605}".to_string(),
                (false, true) => format!("{count}"),
                (false, false) => String::new(),
            };
            let tab_row = ListItem::new(("tab", at), title)
                .selected(shown)
                .strong(shown)
                .leading(Disclosure::new(("tab-fold", at), !folded).size(IconSize::Sm))
                .detail(badge)
                .on_click(move |_, window, cx| {
                    select.update(cx, |this, cx| this.tap_tab(at, window, cx))
                })
                .trailing(
                    IconButton::new(("close-tab", at), IconName::X)
                        .variant(ButtonVariant::Ghost)
                        .size(ControlSize::Sm)
                        .tooltip(language::message(cx, "workspace.close_tab"))
                        .on_click(move |_, window, cx| {
                            cx.stop_propagation();
                            close.update(cx, |this, cx| this.request_close_tab(at, window, cx))
                        }),
                );
            rows.push(tab_row.into_any_element());
            // The shown tab's panes hang under it, so the column reads as a tree; folded, they stay away.
            if !shown || folded {
                continue;
            }
            for id in visible {
                let Some(session) = self.session(id) else {
                    continue;
                };
                let active = session.id == self.active;
                let row = sessions::row(
                    workspace,
                    session.id,
                    session,
                    active,
                    home.as_deref(),
                    now,
                    cx,
                );
                rows.push(row.into_any_element());
            }
        }
        let hosts = cx.global::<crate::ssh::Hosts>();
        if !hosts.hosts.is_empty() {
            rows.push(
                div()
                    .px_2()
                    .pt_3()
                    .child(Caption::new("Remote"))
                    .into_any_element(),
            );
            for (at, host) in hosts.hosts.iter().enumerate() {
                let connect = workspace.clone();
                let name = host.name.clone();
                rows.push(
                    ListItem::new(("ssh", at), host.name.clone())
                        .description(host.summary())
                        .on_click(move |_, window, cx| {
                            connect.update(cx, |this, cx| this.connect(name.clone(), window, cx))
                        })
                        .into_any_element(),
                );
            }
        }
        div()
            .size_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .px_2()
                    .pt_2()
                    .child(Input::new(&self.filter.field).size(ControlSize::Sm)),
            )
            .child(
                div()
                    .id("tabs")
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .p_2()
                    .child(List::new().children(rows)),
            )
    }

    /// Fills the window with the shown pane, or gives the grid back.
    pub fn toggle_zoom(&mut self, cx: &mut Context<Self>) {
        match self.zoom.take() {
            Some(id) => log::info!("app: pane {id} gives the grid back"),
            None => {
                self.zoom = Some(self.active);
                log::info!("app: pane {} fills the window", self.active);
            }
        }
        cx.notify();
    }

    /// Hides both columns for the terminal, or shows them as the settings keep them.
    pub fn toggle_zen(&mut self, cx: &mut Context<Self>) {
        self.zen = !self.zen;
        log::info!("app: zen {}", if self.zen { "on" } else { "off" });
        cx.notify();
    }

    /// Whether each side column shows now: zen hides both, and otherwise the settings decide.
    fn columns(&self, settings: &Settings) -> (bool, bool) {
        if self.zen {
            (false, false)
        } else {
            (settings.show_sessions, settings.show_panel)
        }
    }

    /// The report of the health check, over the terminal, until the next key takes it away.
    fn health(&self) -> impl IntoElement {
        let Some(report) = &self.health else {
            return div();
        };
        let card = div()
            .absolute()
            .top_2()
            .left_2()
            .right_2()
            .p_3()
            .border_1()
            .child(Caption::new(SharedString::from(report.text())));
        div().relative().child(card)
    }
}

impl Render for Workspace {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.background.fit(window);
        window.set_rem_size(px(SettingsStore::get(cx).ui_font_size));
        let now = Instant::now();
        self.tick(now, cx);
        let settings = SettingsStore::get(cx);
        let (show_sessions, show_panel) = self.columns(settings);
        let workspace = cx.entity();
        let tip =
            |shown: bool, what: &str| format!("{} {what}", if shown { "Hide" } else { "Show" });
        // It leads the first column or ends the last.
        let left = column_toggle(
            &workspace,
            ("toggle-sessions", IconName::PanelLeft),
            tip(show_sessions, "sessions"),
            |settings| settings.show_sessions = !settings.show_sessions,
        );
        let right = column_toggle(
            &workspace,
            ("toggle-panel", IconName::PanelRight),
            tip(show_panel, "Git and Files"),
            |settings| settings.show_panel = !settings.show_panel,
        );
        // No menu bar off macOS: a gear opens Settings.
        let gear = (!cfg!(target_os = "macos")).then(|| {
            IconButton::new("open-settings", IconName::Settings)
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Sm)
                .tooltip("Settings")
                .on_click(|_, window, cx| window.dispatch_action(Box::new(OpenSettings), cx))
        });
        let end = div()
            .flex()
            .items_center()
            .gap_2()
            .children(gear)
            .child(right);
        let mut shell = ColumnShell::new("workspace").glass();
        let mut terminal = ShellColumn::new("terminal")
            .title(self.title().to_string())
            .child(super::panes::grid(self, &workspace));
        // STAT-14: a line under the terminal says where the session sits, its size and its branch. Its cells are read here, before the pane is drawn, from the grid the session holds.
        let cells = self
            .session(self.active)
            .map(|session| {
                let term = session.terminal.read(cx).size();
                (
                    i32::try_from(term.0).unwrap_or(0),
                    i32::try_from(term.1).unwrap_or(0),
                )
            })
            .map(|cells| status::of(self.session(self.active), cells));
        // The health check's report, over the terminal, until the next key takes it away.
        terminal = terminal.child(self.health());
        // STAT-17: which branch the shown session's folder sits in, when it sits in one.
        let badge = self
            .session(self.active)
            .and_then(|session| badge::Branch::of(&session.folder))
            .map(|branch| Caption::new(SharedString::from(branch.label())));
        if let Some(badge) = badge {
            terminal = terminal.action(badge);
        }
        if let Some(line) = cells {
            let shown = div()
                .flex()
                .items_center()
                .px_2()
                .py_1()
                .child(Caption::new(SharedString::from(line)));
            terminal = terminal.child(shown);
        }
        if self.tabs.is_empty() {
            // A session that would not start says why, in words the user can copy into an issue.
            let report = self.start_failed.as_ref().map(Failed::report);
            if let Some(report) = report {
                let copied = report.clone();
                terminal = ShellColumn::new("terminal")
                    .title(self.title().to_string())
                    .child(
                        div()
                            .size_full()
                            .flex()
                            .flex_col()
                            .gap_3()
                            .p_6()
                            .child(Caption::new(SharedString::from(report.clone())))
                            .child(
                                Button::new(
                                    "copy-failure",
                                    language::message(cx, "workspace.copy"),
                                )
                                .on_click(move |_, _, cx| {
                                    cx.write_to_clipboard(ClipboardItem::new_string(copied.clone()))
                                }),
                            ),
                    );
            }
        }
        if show_sessions {
            let new = new_button(&workspace, cx);
            let splitter = workspace.clone();
            let split = IconButton::new("split-pane", IconName::PanelRight)
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Sm)
                .tooltip(language::message(cx, "workspace.split"))
                .on_click(move |_, window, cx| {
                    splitter.update(cx, |this, cx| this.split(true, window, cx))
                });
            let column = self.tabs_column(&workspace, cx);
            shell = shell.column(
                self.side_column(Side::Sessions, cx)
                    .leading(left)
                    .action(div().flex().items_center().gap_1().child(new).child(split))
                    .child(column),
            );
        } else {
            terminal = terminal.leading(left);
        }
        if show_panel {
            let tab = settings.panel;
            let chosen = match tab {
                PanelTab::Git => "git",
                PanelTab::Files => "files",
            };
            let switch = SegmentedControl::new("panel-tab", chosen)
                .segment("git", language::message(cx, "workspace.git"), None)
                .segment("files", language::message(cx, "workspace.files"), None)
                .size(ControlSize::Sm)
                .on_change(|value, _, cx| show_tab(value, cx));
            let quiet = self
                .session(self.active)
                .is_some_and(|session| !session.reports);
            let caption = || {
                Caption::new(language::message(cx, "workspace.no_folder"))
                    .px_3()
                    .pt_2()
            };
            // Hidden, not dropped, so each keeps its state.
            let body = |on: bool| {
                div()
                    .flex_1()
                    .min_h_0()
                    .p_2()
                    .when(!on, |body| body.hidden())
            };
            // Git's list draws every row, so terminal output must not redraw it.
            let fill = || StyleRefinement::default().size_full();
            shell = shell.column(terminal).column(
                self.side_column(Side::Panel, cx)
                    .action(end)
                    // Below the title bar, clear of Windows' caption buttons.
                    .child(div().flex().justify_center().px_2().pt_2().child(switch))
                    .children(quiet.then(caption))
                    .child(body(tab == PanelTab::Git).child(self.git.clone().cached(fill())))
                    .child(body(tab == PanelTab::Files).child(self.files.clone())),
            );
        } else {
            shell = shell.column(terminal.action(end));
        }
        // The context sits above the focus root, so its keys work wherever focus is.
        div()
            .size_full()
            .key_context(CONTEXT)
            .map(menus::window_actions)
            .on_action(cx.listener(|this, _: &ToggleSessions, _, cx| {
                this.toggle(
                    |settings| settings.show_sessions = !settings.show_sessions,
                    cx,
                )
            }))
            .on_action(cx.listener(|this, _: &TogglePanel, _, cx| {
                this.toggle(|settings| settings.show_panel = !settings.show_panel, cx)
            }))
            .on_action(
                cx.listener(|this, _: &NewSession, window, cx| {
                    this.new_session_in_pane(window, cx)
                }),
            )
            .on_action(cx.listener(|this, _: &CloseSession, window, cx| {
                this.request_close(this.active, window, cx)
            }))
            .on_action(cx.listener(|this, _: &NextSession, window, cx| this.step(true, window, cx)))
            .on_action(
                cx.listener(|this, _: &PreviousSession, window, cx| this.step(false, window, cx)),
            )
            .on_action(cx.listener(|this, _: &LastSession, window, cx| {
                if let Some(last) = this.session_order().last().copied() {
                    this.select(last, window, cx);
                }
            }))
            .on_action(cx.listener(|this, select: &SelectSession, window, cx| {
                let order = this.session_order();
                if let Some(id) = order.get(select.0).copied() {
                    this.select(id, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .on_action(cx.listener(|this, _: &NewTab, window, cx| this.new_tab(window, cx)))
            .on_action(cx.listener(|this, _: &CloneSession, window, cx| {
                let id = this.active;
                this.clone_session(id, window, cx)
            }))
            .on_action(cx.listener(|this, _: &CloseTab, window, cx| {
                this.request_close_tab(this.active_tab, window, cx)
            }))
            .on_action(cx.listener(|this, _: &NextTab, window, cx| this.step_tab(true, window, cx)))
            .on_action(
                cx.listener(|this, _: &PreviousTab, window, cx| this.step_tab(false, window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &NextUsed, window, cx| this.step_used(true, window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &PreviousUsed, window, cx| this.step_used(false, window, cx)),
            )
            .on_action(cx.listener(|this, _: &LastTab, window, cx| {
                let last = this.tabs.len().saturating_sub(1);
                this.select_tab(last, window, cx);
            }))
            .on_action(cx.listener(|this, select: &SelectTab, window, cx| {
                this.select_tab(select.0, window, cx);
            }))
            .on_action(cx.listener(|this, _: &SplitRight, window, cx| this.split(true, window, cx)))
            .on_action(cx.listener(|this, _: &SplitDown, window, cx| this.split(false, window, cx)))
            .on_action(cx.listener(|this, _: &ClosePane, window, cx| this.close_pane(window, cx)))
            .on_action(cx.listener(|this, _: &ZoomPane, _, cx| this.toggle_zoom(cx)))
            .on_action(cx.listener(|this, _: &Zen, _, cx| this.toggle_zen(cx)))
            .on_action(cx.listener(|this, _: &EvenPanes, _, cx| {
                let focused = this.tab().map_or(0, |tab| tab.focused);
                this.even_panes(focused, cx)
            }))
            .on_action(
                cx.listener(|this, _: &MoveTabUp, window, cx| this.move_tab(true, window, cx)),
            )
            .on_action(
                cx.listener(|this, _: &MoveTabDown, window, cx| this.move_tab(false, window, cx)),
            )
            .on_action(cx.listener(|this, _: &PinTab, window, cx| this.toggle_pin(window, cx)))
            .on_action(cx.listener(|this, focus: &FocusPane, window, cx| {
                this.focus_pane(focus.0, window, cx)
            }))
            .on_action(
                cx.listener(|this, swap: &SwapPane, window, cx| this.swap_pane(swap.0, window, cx)),
            )
            .child(
                ResizeBorder::new()
                    .when(self.background.see_through(), ResizeBorder::glass)
                    .child(FocusScope::new(&self.focus).root().size_full().child(shell)),
            )
    }
}
