//! How the window's content is drawn: the tab bar, the panes and the columns.

use std::time::Instant;

use ely_gpui_component::{
    buttons::{ButtonVariant, IconButton, SegmentedControl},
    lists::List,
    primitives::{FocusScope, IconName},
    shell::{ColumnShell, ResizeBorder, ShellColumn, TabBar, WindowTab},
    theme::ControlSize,
    typography::Caption,
};
use gpui::{
    App, Context, Entity, IntoElement, Render, SharedString, StyleRefinement, Styled, Window, div,
    prelude::*,
};

use super::layout::Tab;
use super::{
    CONTEXT, ClosePane, CloseSession, CloseTab, FocusPane, LastSession, LastTab, NewSession,
    NewTab, NextSession, NextTab, PreviousSession, PreviousTab, SelectSession, SelectTab, Side,
    SplitDown, SplitRight, TogglePanel, ToggleSessions, Workspace,
};
use crate::menus;
use crate::sessions::{self, Session};
use crate::settings::window::OpenSettings;
use crate::settings::{PanelTab, Settings, SettingsStore};
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
    /// The tab bar across the top of the terminal column.
    fn tab_bar(&self, workspace: &Entity<Workspace>) -> impl IntoElement {
        let mut bar = TabBar::new("tab-bar");
        for (at, tab) in self.tabs.iter().enumerate() {
            let title = tab
                .focused_session()
                .and_then(|id| self.session(id))
                .map_or(TITLE, Session::named);
            let title = tab.named(title, TITLE).to_string();
            // A tab with panes names them all, so the count shows beside it.
            let title = match tab.panes.len() {
                1 => title,
                count => format!("{title} ({count})"),
            };
            bar = bar.tab(WindowTab::new(format!("tab-{at}"), title));
        }
        let selected = format!("tab-{}", self.active_tab);
        let on_select = workspace.clone();
        let on_close = workspace.clone();
        let on_add = workspace.clone();
        bar.selected(selected)
            .on_select(move |id, window, cx| {
                let Some(at) = id.strip_prefix("tab-").and_then(|at| at.parse().ok()) else {
                    return;
                };
                on_select.update(cx, |this, cx| this.select_tab(at, window, cx));
            })
            .on_close(move |id, window, cx| {
                let Some(at) = id.strip_prefix("tab-").and_then(|at| at.parse().ok()) else {
                    return;
                };
                on_close.update(cx, |this, cx| this.close_tab_at(at, window, cx));
            })
            .on_add(move |window, cx| on_add.update(cx, |this, cx| this.new_tab(window, cx)))
    }

    /// The panes of the shown tab, laid out as a grid.
    fn panes(&self) -> impl IntoElement {
        let Some(tab) = self.tab() else {
            return div().size_full();
        };
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
        let now = Instant::now();
        self.tick(now, cx);
        let settings = SettingsStore::get(cx);
        let workspace = cx.entity();
        let tip =
            |shown: bool, what: &str| format!("{} {what}", if shown { "Hide" } else { "Show" });
        // It leads the first column or ends the last.
        let left = column_toggle(
            &workspace,
            ("toggle-sessions", IconName::PanelLeft),
            tip(settings.show_sessions, "sessions"),
            |settings| settings.show_sessions = !settings.show_sessions,
        );
        let right = column_toggle(
            &workspace,
            ("toggle-panel", IconName::PanelRight),
            tip(settings.show_panel, "Git and Files"),
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
            .child(
                div()
                    .flex()
                    .flex_col()
                    .w_full()
                    .flex_none()
                    .px_2()
                    .pt_1()
                    .child(self.tab_bar(&workspace)),
            )
            .child(self.panes());
        if settings.show_sessions {
            let home = dirs::home_dir();
            // The column lists the sessions of the shown tab, so the bar above and the list beside it each name one thing.
            let ids = self.tab().map(Tab::sessions).unwrap_or_default();
            let rows = ids
                .iter()
                .filter_map(|id| self.session(*id))
                .map(|session| {
                    let active = session.id == self.active;
                    sessions::row(
                        &workspace,
                        session.id,
                        session,
                        active,
                        home.as_deref(),
                        now,
                        cx,
                    )
                });
            let list = List::new().children(rows);
            let new = IconButton::new("new-session", IconName::Plus)
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Sm)
                .tooltip("New session")
                .on_click(move |_, window, cx| {
                    workspace.update(cx, |this, cx| this.new_session_in_pane(window, cx))
                });
            shell = shell.column(
                self.side_column(Side::Sessions, cx)
                    .leading(left)
                    .action(new)
                    .child(
                        div()
                            .id("sessions")
                            .size_full()
                            .overflow_y_scroll()
                            .p_2()
                            .child(list),
                    ),
            );
        } else {
            terminal = terminal.leading(left);
        }
        if settings.show_panel {
            let tab = settings.panel;
            let chosen = match tab {
                PanelTab::Git => "git",
                PanelTab::Files => "files",
            };
            let switch = SegmentedControl::new("panel-tab", chosen)
                .segment("git", "Git", None)
                .segment("files", "Files", None)
                .size(ControlSize::Sm)
                .on_change(|value, _, cx| show_tab(value, cx));
            let quiet = self
                .session(self.active)
                .is_some_and(|session| !session.reports);
            let caption = || Caption::new("This shell reports no folder.").px_3().pt_2();
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
            .on_action(
                cx.listener(|this, _: &CloseSession, window, cx| {
                    this.close(this.active, window, cx)
                }),
            )
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
            .on_action(cx.listener(|this, _: &CloseTab, window, cx| this.close_tab(window, cx)))
            .on_action(cx.listener(|this, _: &NextTab, window, cx| this.step_tab(true, window, cx)))
            .on_action(
                cx.listener(|this, _: &PreviousTab, window, cx| this.step_tab(false, window, cx)),
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
            .on_action(cx.listener(|this, focus: &FocusPane, window, cx| {
                this.focus_pane(focus.0, window, cx)
            }))
            .child(
                ResizeBorder::new()
                    .when(self.background.see_through(), ResizeBorder::glass)
                    .child(FocusScope::new(&self.focus).root().size_full().child(shell)),
            )
    }
}
