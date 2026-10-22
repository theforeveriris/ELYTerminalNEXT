//! What the terminal tells a program about itself: DA1, XTVERSION and DECRQM.

use alacritty_terminal::term::TermMode;

use super::TerminalView;
use crate::term::device::{self, Query};

impl TerminalView {
    /// The modes the grid holds now, as DECRQM reads them.
    pub(super) fn term_mode(&self) -> TermMode {
        *self.pty.term.lock().mode()
    }

    /// Answers a query the tap found, from the grid's modes and the mouse's.
    pub(super) fn answer(&mut self, query: Query) {
        log::debug!("terminal {}: answers {query:?}", self.pty.pid);
        let mouse = self.mouse;
        self.pty
            .write(query.answer(self.term_mode(), mouse).into_bytes());
    }

    /// The grid's own reply to a DA1 or DECRQM query, answered again honestly; another stands.
    ///
    /// The grid calls itself VT102 and knows almost nothing of the modes this terminal holds, so a
    /// program that asks would take it for less than it is. Its reply is the only `?…c` and `…$y`
    /// it writes, so each is read back here and said again.
    pub(super) fn reply(&mut self, text: String) {
        let mouse = self.mouse;
        let honest = device::honest(&text, self.term_mode(), mouse);
        self.pty.write(honest.unwrap_or(text).into_bytes());
    }
}
