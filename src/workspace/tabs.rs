//! The tabs of the window and the panes in them.

use gpui::{AppContext, Context, PromptLevel, Window};

use super::layout::{Direction, SessionId, Tab};
use super::{TITLE, Workspace};
use crate::sessions::Session;
use crate::settings::{Settings, SettingsStore};
use crate::term::shell::{ShellIntegration, launch};
use crate::term::{Events, Launch, Pty, Quote, Target, TerminalView, config, opened, spawn};

/// How a session is started again as a clone: its program, in the folder it last reported.
///
/// A clone takes no scrollback and no running state with it: the terminal is new, and only the
/// program, its variables and the folder are the same, so a tab opened beside one in `~/src/api`
/// starts there too.
pub(crate) fn clone_of(launch: &Launch, folder: &std::path::Path) -> Launch {
    Launch {
        cwd: folder.to_path_buf(),
        ..launch.clone()
    }
}

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

    /// Adds a session, and shows it in a new pane of the shown tab.
    pub(super) fn add(
        &mut self,
        pty: Pty,
        events: Events,
        launch: Launch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> SessionId {
        let terminal = cx.new(|cx| TerminalView::new(pty, events, window, cx));
        let id = terminal.entity_id();
        let events = cx.subscribe_in(&terminal, window, move |this, _, event, window, cx| {
            this.session_event(id, event, window, cx)
        });
        let pid = terminal.read(cx).pid();
        log::info!("app: session {pid} opens in {}", launch.cwd.display());
        let session = self.next_id;
        self.next_id += 1;
        self.sessions.push(Session {
            id: session,
            terminal,
            title: String::new(),
            name: String::new(),
            editing: None,
            folder: launch.cwd.clone(),
            reports: launch.reports,
            launch,
            bell: false,
            waiting: false,
            look: None,
            _events: events,
        });
        session
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

    /// Opens a tab on `host`, running the ssh the system has.
    pub fn connect(&mut self, host: String, window: &mut Window, cx: &mut Context<Self>) {
        let Some(home) = dirs::home_dir() else {
            log::error!("app: no home folder, so no ssh to {host}");
            return;
        };
        let ssh = match crate::settings::program_path("ssh") {
            Ok(ssh) => ssh,
            Err(error) => {
                log::error!("app: no ssh to {host}: {error:#}");
                return;
            }
        };
        let settings = SettingsStore::get(cx).clone();
        let launch = Launch {
            program: Some((ssh, vec![host.clone()])),
            cwd: home,
            env: Vec::new(),
            reports: false,
            quote: Quote::Posix,
            target: Target::Local,
        };
        let started = spawn(&launch, (80, 24), (8, 16), config(&settings));
        let (pty, events) = match started {
            Ok(started) => started,
            Err(error) => {
                log::error!("app: no ssh to {host}: {error:#}");
                return;
            }
        };
        let id = self.add(pty, events, launch, window, cx);
        self.tabs.push(Tab::new(id));
        self.active_tab = self.tabs.len() - 1;
        self.select(id, window, cx);
        log::info!("app: ssh connects to {host}");
    }

    /// Opens a tab with a session of its own: the same program, in the same folder as `id`'s.
    pub fn clone_session(&mut self, id: SessionId, window: &mut Window, cx: &mut Context<Self>) {
        let Some(session) = self.session(id) else {
            return;
        };
        let launch = clone_of(&session.launch, &session.folder);
        let name = session.named().to_string();
        let settings = SettingsStore::get(cx).clone();
        let (pty, events) = match opened(&launch, (80, 24), (8, 16), config(&settings)) {
            Ok(started) => started,
            Err(failed) => {
                log::error!("app: no clone of {id}:\n{}", failed.report());
                return;
            }
        };
        let clone = self.add(pty, events, launch, window, cx);
        self.tabs.push(Tab::new(clone));
        self.active_tab = self.tabs.len() - 1;
        self.select(clone, window, cx);
        log::info!(
            "app: tab {} opens as a clone of {name}",
            self.active_tab + 1
        );
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
        // A pane with a single session closes it, and asks first when it is busy.
        if self.tab().is_some_and(|tab| tab.panes.len() <= 1)
            && let Some(id) = self.tab().and_then(|tab| tab.focused_session())
        {
            self.request_close(id, window, cx);
            return;
        }
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

    /// Shows or hides a side column, as the settings keep it.
    pub(super) fn toggle(&mut self, flip: fn(&mut Settings), cx: &mut Context<Self>) {
        self.zen = false;
        match SettingsStore::update(cx, flip) {
            Ok(()) => {
                let settings = SettingsStore::get(cx);
                let shown = |on: bool| if on { "shown" } else { "hidden" };
                log::info!(
                    "app: sessions {}, right column {}",
                    shown(settings.show_sessions),
                    shown(settings.show_panel)
                );
            }
            Err(error) => log::error!("app: the columns stay: {error:#}"),
        }
    }

    /// Shows the next session, or the one before, round the ends.
    pub(super) fn step(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        let count = self.sessions.len();
        if count == 0 {
            return;
        }
        let by = if forward { 1 } else { count - 1 };
        let next = self.session_order();
        let at = next
            .iter()
            .position(|id| *id == self.active)
            .map(|at| (at + by) % count)
            .unwrap_or(0);
        let id = next[at];
        self.select(id, window, cx);
    }

    /// Shows the newest session not shown, or the oldest, as Ctrl-Tab walks them.
    pub(super) fn step_used(&mut self, forward: bool, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(id) = self.mru.next(self.active, forward) {
            self.select(id, window, cx);
        }
    }

    /// Exchanges the focused pane with its neighbour, and keeps the moved pane focused.
    pub(super) fn swap_pane(
        &mut self,
        direction: Direction,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.tab_mut().is_some_and(|tab| tab.swap(direction)) {
            return;
        }
        if let Some(id) = self.tab().and_then(|tab| tab.focused_session()) {
            self.select(id, window, cx);
        }
    }

    /// Closes session `id`, asking first when its command still runs and the settings ask.
    pub fn request_close(&mut self, id: SessionId, window: &mut Window, cx: &mut Context<Self>) {
        let busy = self
            .session(id)
            .is_some_and(|session| session.terminal.read(cx).busy());
        if !self.needs_confirmation(busy, cx) {
            self.close(id, window, cx);
            return;
        }
        let answer = window.prompt(
            PromptLevel::Warning,
            "Close this session?",
            Some("A command is still running."),
            &["Cancel", "Close"],
            cx,
        );
        let this = cx.entity();
        cx.spawn_in(window, async move |_, cx| {
            if matches!(answer.await, Ok(1)) {
                this.update_in(cx, |this, window, cx| this.close(id, window, cx))
                    .ok();
            }
        })
        .detach();
    }

    /// Closes the tab at `index`, asking first when one of its commands still runs.
    pub fn request_close_tab(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        let busy = self.tabs.get(index).is_some_and(|tab| {
            tab.sessions().into_iter().any(|id| {
                self.session(id)
                    .is_some_and(|session| session.terminal.read(cx).busy())
            })
        });
        if !self.needs_confirmation(busy, cx) {
            self.close_tab_at(index, window, cx);
            return;
        }
        let answer = window.prompt(
            PromptLevel::Warning,
            "Close this tab?",
            Some("A command is still running."),
            &["Cancel", "Close"],
            cx,
        );
        let this = cx.entity();
        cx.spawn_in(window, async move |_, cx| {
            if matches!(answer.await, Ok(1)) {
                this.update_in(cx, |this, window, cx| this.close_tab_at(index, window, cx))
                    .ok();
            }
        })
        .detach();
    }

    /// Whether closing with `busy` needs a prompt, as the settings decide.
    fn needs_confirmation(&self, busy: bool, cx: &Context<Self>) -> bool {
        busy && SettingsStore::get(cx).confirm_close
    }

    /// What the shown session is called.
    pub(super) fn title(&self) -> &str {
        self.session(self.active).map_or(TITLE, Session::named)
    }

    pub(super) fn retitle(&self, window: &mut Window) {
        let title = self.title();
        if window.window_title() != title {
            log::debug!("app: the title is {title:?}");
            window.set_window_title(title);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn launch(cwd: &str) -> Launch {
        Launch {
            program: Some(("/bin/sh".into(), vec!["-l".into()])),
            cwd: PathBuf::from(cwd),
            env: vec![("ELY".into(), "ok".into())],
            reports: true,
            quote: Quote::Posix,
            target: Target::Local,
        }
    }

    #[test]
    fn a_clone_runs_the_same_program_in_the_folder_its_original_reports() {
        let started = launch("/home/me");
        let clone = clone_of(&started, &PathBuf::from("/home/me/src/api"));
        assert_eq!(clone.cwd, PathBuf::from("/home/me/src/api"));
        assert_eq!(clone.program, started.program, "the same program");
        assert_eq!(clone.env, started.env, "the same variables");
        assert_eq!(clone.reports, started.reports);
        assert_eq!(clone.quote, started.quote);
        assert_eq!(clone.target, started.target);
        assert_eq!(
            started.cwd,
            PathBuf::from("/home/me"),
            "the original is left alone"
        );
    }
}
