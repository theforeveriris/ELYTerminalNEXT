use gpui::{Context, EntityId, Window};

use super::Workspace;
use crate::term::{State, TerminalEvent};

impl Workspace {
    /// What session `id`'s terminal reported.
    pub(super) fn session_event(
        &mut self,
        id: EntityId,
        event: &TerminalEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(session) = self
            .sessions
            .iter()
            .position(|session| session.terminal.entity_id() == id)
        else {
            return;
        };
        let key = self.sessions[session].id;
        let shown = key == self.active;
        let session = &mut self.sessions[session];
        match event {
            TerminalEvent::Title(title) => {
                log::debug!("app: session {key}'s program names it {title:?}");
                session.title.clone_from(title);
            }
            TerminalEvent::Folder(folder) => {
                session.folder.clone_from(folder);
                session.reports = true;
                if shown {
                    self.files.update(cx, |files, cx| files.show(folder, cx));
                    self.git.update(cx, |git, cx| git.show(folder, cx));
                }
            }
            TerminalEvent::Bell if !shown => {
                if !session.bell {
                    log::debug!("app: a bell in hidden session {key}");
                }
                session.bell = true;
            }
            TerminalEvent::Bell => {}
            // A hidden session that starts waiting marks its row until it is shown.
            TerminalEvent::Shows(Some(State::Blocked)) if !shown => {
                log::debug!("app: hidden session {key} waits");
                session.waiting = true;
            }
            TerminalEvent::Shows(_) => {}
            TerminalEvent::Exited(status) => {
                let pid = session.terminal.read(cx).pid();
                log::info!("app: session {pid}'s program ended, {status}");
                self.close(key, window, cx);
                return;
            }
        }
        if shown {
            self.retitle(window, cx);
        }
        cx.notify();
    }

    /// A press on notification `tag`: its session hears of it and shows, unless it asked not to; false when no session here posted it.
    pub fn pressed(&mut self, tag: &str, window: &mut Window, cx: &mut Context<Self>) -> bool {
        let found = self.sessions.iter().find_map(|session| {
            let focus = session.terminal.update(cx, |view, _| view.pressed(tag))?;
            Some((session.id, focus))
        });
        let Some((id, focus)) = found else {
            return false;
        };
        if focus {
            cx.activate(true);
            window.activate_window();
            self.select(id, window, cx);
        }
        true
    }
}
