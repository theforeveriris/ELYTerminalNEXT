//! Whether a command still runs in a session.

use super::TerminalView;

impl TerminalView {
    /// Whether a command other than the shell runs in the foreground.
    #[cfg(unix)]
    pub fn busy(&self) -> bool {
        self.pty
            .foreground()
            .is_ok_and(|group| group != self.pty.pid as i32)
    }

    /// Windows has no equivalent foreground group yet, so it never holds a close.
    #[cfg(not(unix))]
    pub fn busy(&self) -> bool {
        false
    }
}
