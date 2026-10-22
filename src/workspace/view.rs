//! How the window's content is drawn: the tab bar, the panes and the columns.

use std::time::Instant;

use ely_gpui_component::{
    buttons::{Button, ButtonVariant, IconButton, SegmentedControl},
    forms::Input,
    lists::{List, ListItem},
    primitives::{FocusScope, IconName},
    shell::{ColumnShell, ResizeBorder, ShellColumn},
    theme::ControlSize,
    typography::Caption,
};
use gpui::{
    AnyElement, App, ClipboardItem, Context, Entity, IntoElement, Render, SharedString,
    StyleRefinement, Styled, Window, div, prelude::*, px,
};

use super::{
    CONTEXT, CloneSession, ClosePane, CloseSession, CloseTab, FocusPane, LastSession, LastTab,
    NewSession, NewTab, NextSession, NextTab, NextUsed, PreviousSession, PreviousTab, PreviousUsed,
    SelectSession, SelectTab, Side, SplitDown, SplitRight, SwapPane, TogglePanel, ToggleSessions,
    Workspace, Zen, ZoomPane,
};
use crate::menus;
use crate::sessions::{self, Session};
use crate::settings::window::OpenSettings;
use crate::settings::{PanelTab, Settings, SettingsStore, language};
use crate::term::Failed;
use crate::workspace::TITLE;

/// Shows Git or Files in the right column, as the settings keep it.
fn show_tab(value: &SharedString, cx: &mut App) {
    let panel = match value.as_ref() {
        "git" => PanelTab::Git,
        "files" => PanelTab::Files,
        other => unreachable!("app: no {other} tab"),
    };
    if SettingsStore::get(cx).panel == panel {
        return;
    }
    match SettingsStore::update(cx, |settings| settings.panel = panel) {
        Ok(()) => log::info!("app: the right column shows {panel:?}"),
        Err(error) => log::error!("app: the right column stays: {error:#}"),
    }
}

/// A header button that hides or shows a column, saved.
fn column_toggle(
    workspace: &Entity<Workspace>,
    (id, icon): (&'static str, IconName),
    tip: String,
    flip: fn(&mut Settings),
) -> IconButton {
    let workspace = workspace.clone();
    IconButton::new(id, icon)
        .variant(ButtonVariant::Ghost)
        .size(ControlSize::Sm)
        .tooltip(tip)
        .on_click(move |_, _, cx| workspace.update(cx, |this, cx| this.toggle(flip, cx)))
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
            // A tab names its own sessions, so it shows how many it holds.
            let count = tab.panes.len();
            let tab_row = ListItem::new(("tab", at), title)
                .selected(shown)
                .strong(shown)
                .detail(if count > 1 {
                    format!("{count}")
                } else {
                    String::new()
                })
                .on_click(move |_, window, cx| {
                    select.update(cx, |this, cx| this.select_tab(at, window, cx))
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
            // The shown tab's panes hang under it, so the column reads as a tree.
            if !shown {
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

    /// The panes of the shown tab, laid out as a grid.
    fn panes(&self) -> impl IntoElement {
        let Some(tab) = self.tab() else {
            return div().size_full();
        };
        if let Some(id) = self.zoom
            && tab.holds(id)
            && let Some(session) = self.session(id)
        {
            return div().size_full().child(session.terminal.clone());
        }
        let rows = tab.rows();
        let mut out = div().flex().flex_col().size_full();
        for row in rows {
            let mut line = div().flex().flex_row().flex_1().min_h_0();
            for (at, index) in row.iter().enumerate() {
                let Some(pane) = tab.panes.get(*index) else {
                    continue;
                };
                let Some(session) = self.session(pane.session) else {
                    continue;
                };
                let focused = tab.focused == *index;
                let terminal = session.terminal.clone();
                let mut cell = div().flex_1().min_w_0().min_h_0().child(terminal);
                if !focused {
                    // A pane that is not focused is dimmed, so the focus is obvious.
                    cell = cell.opacity(0.72);
                }
                if at + 1 < row.len() {
                    cell = cell.border_r_1();
                }
                line = line.child(cell);
            }
            out = out.child(line);
        }
        out
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
            .child(self.panes());
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
            let adder = workspace.clone();
            let new = IconButton::new("new-tab", IconName::Plus)
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Sm)
                .tooltip(language::message(cx, "workspace.new_tab"))
                .on_click(move |_, window, cx| {
                    adder.update(cx, |this, cx| this.new_tab(window, cx))
                });
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
