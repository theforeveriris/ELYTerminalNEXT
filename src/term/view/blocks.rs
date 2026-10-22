//! The command blocks a session's shell marks divide its work into.

use super::super::blocks::Blocks;
use super::TerminalView;

impl TerminalView {
    /// The commands its shell's marks divide its work into.
    pub(crate) fn blocks(&self) -> &Blocks {
        &self.blocks
    }
}
