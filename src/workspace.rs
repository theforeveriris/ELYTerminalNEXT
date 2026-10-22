use std::time::Instant;

use gpui::{
    Action, App, AppContext, Context, Entity, FocusHandle, Focusable, KeyBinding, Subscription,
    Task, Window, actions,
};

use crate::{
    blur::Blur,
    files::Files,
    git::panel::{Acted, GitPanel},
    sessions::Session,
    sessions::orb,
    settings::SettingsStore,
    term::{Events, Failed, Launch, Pty},
};

mod events;
mod filter;
mod layout;
mod mru;
mod rename;
mod restore;
mod tabs;
mod view;
mod widths;

use layout::{Direction, Pane, SessionId, Tab};
use widths::{Dragged, Side};

actions!(
    elyterminal,
    [
        ToggleSessions,
        TogglePanel,
        NewSession,
        CloseSession,
        NextSession,
        PreviousSession,
        LastSession,
        NextUsed,
        PreviousUsed,
        NewTab,
        CloseTab,
        NextTab,
        PreviousTab,
        LastTab,
        SplitRight,
        SplitDown,
        ClosePane,
        ZoomPane,
        Zen,
        CloneSession
    ]
);

/// Shows the session at this place, from 0; past the last one, the key goes on to the program.
#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = elyterminal, no_json)]
pub struct SelectSession(pub usize);

/// Shows the tab at this place, from 0.
#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = elyterminal, no_json)]
pub struct SelectTab(pub usize);

/// Moves the focus to the pane in this direction.
#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = elyterminal, no_json)]
pub struct FocusPane(pub Direction);

/// Exchanges the focused pane with its neighbour in this direction.
#[derive(Clone, Debug, PartialEq, Action)]
#[action(namespace = elyterminal, no_json)]
pub struct SwapPane(pub Direction);

/// The key context of the main window's content.
pub const CONTEXT: &str = "Workspace";

/// The window's title while the program names none.
pub const TITLE: &str = "ELYTerminal";

/// Ghostty's keys: Cmd on macOS; off it, Ctrl-Shift for keys a terminal takes, Alt for digits.
pub fn bind_keys(cx: &mut App) {
    let (sessions, panel, new, close, digit) = if cfg!(target_os = "macos") {
        ("cmd-b", "cmd-alt-b", "cmd-t", "cmd-w", "cmd")
    } else {
        (
            "ctrl-shift-b",
            "ctrl-shift-alt-b",
            "ctrl-shift-t",
            "ctrl-shift-w",
            "alt",
        )
    };
    // The tabs and the panes, as Ghostty binds them.
    let (tab_new, tab_close, split, split_down, pane) = if cfg!(target_os = "macos") {
        ("cmd-t", "cmd-w", "cmd-d", "cmd-shift-d", "cmd-alt")
    } else {
        (
            "ctrl-shift-t",
            "ctrl-shift-w",
            "ctrl-shift-d",
            "ctrl-shift-alt-d",
            "ctrl-shift-alt",
        )
    };
    let (zoom, zen) = if cfg!(target_os = "macos") {
        ("cmd-shift-enter", "cmd-shift-z")
    } else {
        ("ctrl-shift-enter", "ctrl-shift-z")
    };
    let swap = if cfg!(target_os = "macos") {
        "cmd-alt-shift"
    } else {
        "ctrl-alt"
    };
    let tab_clone = if cfg!(target_os = "macos") {
        "cmd-shift-k"
    } else {
        "ctrl-shift-k"
    };
    let context = Some(CONTEXT);
    cx.bind_keys([
        KeyBinding::new(sessions, ToggleSessions, context),
        KeyBinding::new(panel, TogglePanel, context),
    ]);
    // The tabs take the plain keys, and the sessions the shifted ones.
    cx.bind_keys([
        KeyBinding::new(tab_new, NewTab, context),
        KeyBinding::new(tab_close, CloseTab, context),
        KeyBinding::new(split, SplitRight, context),
        KeyBinding::new(split_down, SplitDown, context),
        KeyBinding::new(&format!("{pane}-w"), ClosePane, context),
        KeyBinding::new(zoom, ZoomPane, context),
        KeyBinding::new(zen, Zen, context),
    ]);
    // TAB-13: a clone opens beside what it copies, with no scrollback and no running state.
    cx.bind_keys([KeyBinding::new(tab_clone, CloneSession, context)]);
    cx.bind_keys([
        KeyBinding::new(new, NewSession, context),
        KeyBinding::new(close, CloseSession, context),
        KeyBinding::new("ctrl-tab", NextUsed, context),
        KeyBinding::new("ctrl-shift-tab", PreviousUsed, context),
        KeyBinding::new("ctrl-pagedown", NextTab, context),
        KeyBinding::new("ctrl-pageup", PreviousTab, context),
        KeyBinding::new(&format!("{digit}-9"), LastTab, context),
        // The sessions step within the shown tab.
        KeyBinding::new("ctrl-shift-right", NextSession, context),
        KeyBinding::new("ctrl-shift-left", PreviousSession, context),
        KeyBinding::new(&format!("{digit}-shift-9"), LastSession, context),
    ]);
    cx.bind_keys(
        (1..=8).map(|n| KeyBinding::new(&format!("{digit}-{n}"), SelectTab(n - 1), context)),
    );
    cx.bind_keys(
        (1..=8)
            .map(|n| KeyBinding::new(&format!("{digit}-shift-{n}"), SelectSession(n - 1), context)),
    );
    cx.bind_keys([
        KeyBinding::new(&format!("{pane}-left"), FocusPane(Direction::Left), context),
        KeyBinding::new(
            &format!("{pane}-right"),
            FocusPane(Direction::Right),
            context,
        ),
        KeyBinding::new(&format!("{pane}-up"), FocusPane(Direction::Up), context),
        KeyBinding::new(&format!("{pane}-down"), FocusPane(Direction::Down), context),
        KeyBinding::new(&format!("{swap}-left"), SwapPane(Direction::Left), context),
        KeyBinding::new(
            &format!("{swap}-right"),
            SwapPane(Direction::Right),
            context,
        ),
        KeyBinding::new(&format!("{swap}-up"), SwapPane(Direction::Up), context),
        KeyBinding::new(&format!("{swap}-down"), SwapPane(Direction::Down), context),
    ]);
}

/// The main window's content: sessions, the terminal, and Git or Files, in Ely's columns.
pub struct Workspace {
    focus: FocusHandle,
    /// Every session, by its id.
    sessions: Vec<Session>,
    /// The tabs of the window, each a page of panes.
    tabs: Vec<Tab>,
    /// The tab shown.
    active_tab: usize,
    /// The session shown, as an id into `sessions`.
    active: SessionId,
    files: Entity<Files>,
    git: Entity<GitPanel>,
    /// What the window shows of the desktop, and the blur behind it.
    background: Blur,
    /// Column widths a drag shows before the settings hold them.
    dragged: Dragged,
    /// Draws the window at each step of the status marks while one shown moves.
    ticking: Option<Task<()>>,
    /// The next id a session takes, so an id is never reused.
    next_id: SessionId,
    /// The session whose pane fills the window, while one does.
    zoom: Option<SessionId>,
    /// Whether both columns step aside for the terminal.
    zen: bool,
    /// Writes the tabs a moment after the last change that could alter them.
    saving: Option<Task<()>>,
    /// The sessions in the order they were last shown.
    mru: mru::Mru,
    /// The field that filters the sessions column.
    filter: filter::Filter,
    /// Why the session this window opened with could not start, while it keeps none.
    pub(crate) start_failed: Option<Failed>,
    _settings: Subscription,
    _acted: Subscription,
}

impl Workspace {
    /// The window's content with its first session, `pty` started from `launch`.
    pub fn new(
        pty: Pty,
        events: Events,
        launch: Launch,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let mut workspace = Self::bare(&launch, None, window, cx);
        let saved = SettingsStore::get(cx)
            .restore_session
            .then(Self::saved)
            .flatten();
        let first = workspace.add(pty, events, launch, window, cx);
        let mut first_tab = Tab::new(first);
        if let Some(saved) = &saved
            && let Some(tab) = saved.tabs.first()
        {
            first_tab.name = tab.name.clone();
        }
        workspace.tabs.push(first_tab);
        workspace.active_tab = 0;
        workspace.select(first, window, cx);
        if let Some(saved) = saved {
            workspace.restore(saved, window, cx);
        }
        workspace
    }

    /// The window's content with no session: the one it opened with could not start, and says why.
    pub fn failed(
        launch: Launch,
        failed: Failed,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        Self::bare(&launch, Some(failed), window, cx)
    }

    /// A window with no session yet; `failed` says why the one it opened with did not start.
    fn bare(
        launch: &Launch,
        failed: Option<Failed>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let git = cx.new(|cx| GitPanel::new(launch.cwd.clone(), window, cx));
        // A change the Git panel made shows in Files' marks too.
        let acted = cx.subscribe(&git, |this, _, _: &Acted, cx| {
            let folder = this
                .session(this.active)
                .map(|session| session.folder.clone())
                .unwrap_or_default();
            this.files.update(cx, |files, cx| files.show(&folder, cx));
        });
        Self {
            focus: cx.focus_handle(),
            sessions: Vec::new(),
            tabs: Vec::new(),
            active_tab: 0,
            active: 0,
            files: cx.new(|_| Files::new(launch.cwd.clone())),
            git,
            background: Blur::new(window, SettingsStore::get(cx)),
            dragged: Dragged::default(),
            ticking: None,
            next_id: 1,
            zoom: None,
            zen: false,
            saving: None,
            mru: mru::Mru::default(),
            filter: filter::Filter::new(window, cx),
            start_failed: failed,
            _settings: cx.observe_global_in::<SettingsStore>(window, |this, window, cx| {
                this.background.follow(window, SettingsStore::get(cx));
                cx.notify();
            }),
            _acted: acted,
        }
    }

    /// The session with the id, if it is still here.
    fn session(&self, id: SessionId) -> Option<&Session> {
        self.sessions.iter().find(|session| session.id == id)
    }

    /// The session with the id, for changing it.
    fn session_mut(&mut self, id: SessionId) -> Option<&mut Session> {
        self.sessions.iter_mut().find(|session| session.id == id)
    }

    /// Notes each session's mark `now`, and draws the window at each step while one shown moves.
    fn tick(&mut self, now: Instant, cx: &mut Context<Self>) {
        let mut moving = false;
        for session in &mut self.sessions {
            moving |= session.note(now, cx);
        }
        let moving =
            moving && SettingsStore::get(cx).show_sessions && !SettingsStore::get(cx).reduce_motion;
        if moving == self.ticking.is_some() {
            return;
        }
        log::debug!("app: status marks {}", if moving { "move" } else { "rest" });
        self.ticking = moving.then(|| {
            cx.spawn(async move |this, cx| {
                loop {
                    cx.background_executor()
                        .timer(orb::wait(Instant::now()))
                        .await;
                    if this.update(cx, |_, cx| cx.notify()).is_err() {
                        break;
                    }
                }
            })
        });
    }

    /// Opens a session in a new pane of the shown tab: beside the only one, or in a split of the focused one.
    pub fn new_session_in_pane(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let Some(id) = self.new_session(window, cx) else {
            return;
        };
        let Some(tab) = self.tab_mut() else {
            return;
        };
        if tab.panes.len() == 1 {
            // The first pane keeps its session; the new one joins it.
            tab.panes.push(Pane {
                session: id,
                width: 5,
                height: 10,
            });
            tab.focused = 1;
        } else {
            // A tab with panes splits the focused one, and the new session takes it.
            tab.split(true, id);
        }
        self.select(id, window, cx);
    }

    /// The tab and the pane that hold the session `id`.
    fn where_is(&self, id: SessionId) -> Option<(usize, usize)> {
        self.tabs.iter().enumerate().find_map(|(at, tab)| {
            tab.panes
                .iter()
                .position(|pane| pane.session == id)
                .map(|pane| (at, pane))
        })
    }

    /// Shows the session `id`, clears its bell and its waiting mark, and gives its terminal the keys.
    pub fn select(&mut self, id: SessionId, window: &mut Window, cx: &mut Context<Self>) {
        if self.session(id).is_none() {
            return;
        }
        // A session in another tab takes that tab and its pane to the front.
        if let Some((tab, pane)) = self.where_is(id) {
            if tab != self.active_tab {
                self.active_tab = tab;
            }
            if let Some(tab) = self.tabs.get_mut(tab) {
                tab.focus(pane);
            }
        }
        if id != self.active
            && let Some(hidden) = self.session(self.active)
        {
            hidden.terminal.update(cx, |view, cx| {
                view.let_go(cx);
                view.set_front(false);
            });
        }
        let Some(session) = self.session_mut(id) else {
            return;
        };
        session.bell = false;
        session.waiting = false;
        session.terminal.update(cx, |view, _| view.set_front(true));
        window.focus(&session.terminal.focus_handle(cx), cx);
        let folder = session.folder.clone();
        self.files.update(cx, |files, cx| files.show(&folder, cx));
        self.git.update(cx, |git, cx| git.show(&folder, cx));
        self.active = id;
        self.mru.touch(id);
        if self.zoom.is_some() {
            self.zoom = Some(id);
        }
        log::debug!("app: session {id} shown");
        self.retitle(window);
        self.save_soon(cx);
        cx.notify();
    }

    /// Closes the session `id`; its program hangs up, and the pane it held goes with it.
    pub fn close(&mut self, id: SessionId, window: &mut Window, cx: &mut Context<Self>) {
        if self.session(id).is_none() {
            return;
        }
        let pid = self
            .session(id)
            .map(|session| session.terminal.read(cx).pid());
        log::info!("app: session {id} closes");
        self.mru.forget(id);
        if self.zoom == Some(id) {
            self.zoom = None;
        }
        // The session leaves the list, and every pane that held it.
        self.sessions.retain(|session| session.id != id);
        for tab in &mut self.tabs {
            tab.panes.retain(|pane| pane.session != id);
            if tab.focused >= tab.panes.len() {
                tab.focused = tab.panes.len().saturating_sub(1);
            }
        }
        // A tab with no pane is gone.
        let emptied: Vec<usize> = self
            .tabs
            .iter()
            .enumerate()
            .filter(|(_, tab)| tab.panes.is_empty())
            .map(|(at, _)| at)
            .collect();
        for at in emptied.into_iter().rev() {
            self.tabs.remove(at);
        }
        if self.tabs.is_empty() {
            log::info!("app: no tab left; the window closes");
            window.remove_window();
            return;
        }
        if self.active_tab >= self.tabs.len() {
            self.active_tab = self.tabs.len() - 1;
        }
        // The shown session may have gone; the focused pane of the shown tab takes over.
        let shown = self
            .tab()
            .and_then(|tab| tab.focused_session())
            .unwrap_or(id);
        if self.session(shown).is_none() {
            let next = self
                .tab()
                .and_then(|tab| tab.focused_session())
                .or_else(|| self.tabs.first().and_then(|tab| tab.focused_session()));
            match next {
                Some(next) => self.select(next, window, cx),
                None => {
                    self.active = 0;
                    cx.notify();
                }
            }
        } else if self.active == id {
            self.select(shown, window, cx);
        } else {
            cx.notify();
        }
        self.save_soon(cx);
        let _ = pid;
    }

    /// The tab shown.
    fn tab(&self) -> Option<&Tab> {
        self.tabs.get(self.active_tab)
    }

    /// The tab shown, for changing it.
    fn tab_mut(&mut self) -> Option<&mut Tab> {
        self.tabs.get_mut(self.active_tab)
    }

    /// Its sessions, in their rows' order.
    pub fn sessions(&self) -> &[Session] {
        &self.sessions
    }
}
