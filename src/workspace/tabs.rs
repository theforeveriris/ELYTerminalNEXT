//! The tabs of the window and the panes in them.

use gpui::{Context, Window};

use super::Workspace;
use super::layout::{Direction, SessionId, Tab};
use crate::settings::SettingsStore;
use crate::term::shell::{ShellIntegration, launch};
use crate::term::{config, spawn};

impl Workspace {
    /// The sessions in the order the keys walk them: the shown tab's panes first, then the other tabs'.
    pub(super) fn session_order(&self) -> Vec<SessionId> {
        let mut order: Vec<SessionId> = Vec::new();
        for (at, tab) in self.tabs.iter().enumerate() {
            if at == self.active_tab {
                order.extend(tab.sessions());
            }
        }
        for (at, tab) in self.tabs.iter().enumerate() {
            if at != self.active_tab {
                order.extend(tab.sessions());
            }
        }
        order
    }

    /// Opens a tab, with a new session in it.
    pub fn new_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.new_session(window, cx) else {
            return;
        };
        self.tabs.push(Tab::new(id));
        self.active_tab = self.tabs.len() - 1;
        self.select(id, window, cx);
        log::info!("app: tab {} opens", self.active_tab + 1);
    }

    /// Closes the tab at `index`; its sessions' programs hang up.
    pub fn close_tab_at(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.tabs.len() {
            return;
        }
        let sessions: Vec<SessionId> = self.tabs[index]
            .panes
            .iter()
            .map(|pane| pane.session)
            .collect();
        self.tabs.remove(index);
        if self.tabs.is_empty() {
            log::info!("app: no tab left; the window closes");
            for id in sessions {
                self.close(id, window, cx);
            }
            window.remove_window();
            return;
        }
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        } else if index < self.active_tab {
            self.active_tab -= 1;
        }
        log::info!("app: tab {} closes", index + 1);
        for id in sessions {
            self.close(id, window, cx);
        }
        if let Some(next) = self.tab().and_then(|tab| tab.focused_session()) {
            self.select(next, window, cx);
        }
    }

    /// Closes the shown tab; its sessions' programs hang up.
    pub fn close_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        self.close_tab_at(self.active_tab, window, cx);
    }

    /// Shows the next tab, or the one before, round the ends.
    pub fn step_tab(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        let count = self.tabs.len();
        if count == 0 {
            return;
        }
        let by = if forward { 1 } else { count - 1 };
        self.active_tab = (self.active_tab + by) % count;
        self.show_tab(window, cx);
    }

    /// Shows the tab at `index`.
    pub fn select_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if index >= self.tabs.len() {
            return;
        }
        self.active_tab = index;
        self.show_tab(window, cx);
    }

    /// Shows the focused pane of the shown tab.
    fn show_tab(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.tab().and_then(|tab| tab.focused_session()) {
            self.select(id, window, cx);
        }
        cx.notify();
    }

    /// Splits the focused pane, and puts a new session in the new pane.
    pub fn split(&mut self, vertical: bool, window: &mut Window, cx: &mut Context<Self>) {
        let Some(folder) = self
            .session(self.active)
            .map(|session| session.folder.clone())
        else {
            return;
        };
        let settings = SettingsStore::get(cx);
        let started = launch(&settings.shell, folder, &cx.global::<ShellIntegration>().0)
            .and_then(|launch| Ok((spawn(&launch, (80, 24), (8, 16), config(settings))?, launch)));
        let Ok(((pty, events), launch)) = started else {
            log::error!("app: no split: the new session would not start");
            return;
        };
        let id = self.add(pty, events, launch, window, cx);
        let Some(tab) = self.tab_mut() else {
            return;
        };
        // The split takes the focused pane's room; the new session goes in.
        tab.split(vertical, id);
        self.select(id, window, cx);
        log::info!(
            "app: the pane splits {}",
            if vertical { "sideways" } else { "above" }
        );
    }

    /// Moves the focus to the pane in `direction`.
    pub fn focus_pane(
        &mut self,
        direction: Direction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(tab) = self.tab_mut() else {
            return;
        };
        let Some(next) = tab.focus_neighbour(direction) else {
            return;
        };
        tab.focus(next);
        if let Some(id) = tab.focused_session() {
            self.select(id, window, cx);
        }
    }

    /// Closes the focused pane; its program hangs up.
    pub fn close_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(tab) = self.tab_mut() else {
            return;
        };
        let Some(id) = tab.focused_session() else {
            return;
        };
        let at = tab.focused;
        let empty = tab.remove(at);
        if empty {
            self.tabs.remove(self.active_tab);
            if self.tabs.is_empty() {
                self.close(id, window, cx);
                window.remove_window();
                return;
            }
            if self.active_tab >= self.tabs.len() {
                self.active_tab = self.tabs.len() - 1;
            }
        }
        self.close(id, window, cx);
    }
}
