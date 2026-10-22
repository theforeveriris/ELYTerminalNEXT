//! The command blocks a session's shell marks divide its work into, and the text a block holds.

use alacritty_terminal::{
    grid::Dimensions,
    index::{Column, Line, Point},
    term::cell::Flags,
};
use gpui::{App, ClipboardItem, Context};

use super::super::blocks::{At, Blocks, Export};
use super::TerminalView;

/// A cell that only stands beside a wide character, and shows nothing of its own.
const SPACERS: Flags = Flags::WIDE_CHAR_SPACER.union(Flags::LEADING_WIDE_CHAR_SPACER);

impl TerminalView {
    /// The commands its shell's marks divide its work into.
    pub(crate) fn blocks(&self) -> &Blocks {
        &self.blocks
    }

    /// Walks to the next command that ended, or the previous one, and scrolls it into view.
    ///
    /// A command's prompt line is where a key lands, so the command and what it printed follow it.
    /// With `failed`, only the commands that failed are walked: the ones worth a second look.
    pub(super) fn step_block(&mut self, newer: bool, failed: bool, cx: &mut Context<Self>) {
        let Some(at) = self.blocks.step(self.block_at, newer, failed) else {
            return;
        };
        let Some(block) = self.blocks.done().get(at) else {
            return;
        };
        let point = Point::new(Line(block.prompt_line), Column(0));
        log::debug!("terminal {}: block {at} at {point:?}", self.pty.pid);
        self.block_at = Some(at);
        self.pty.term.lock().scroll_to_point(point);
        cx.notify();
    }

    /// Copies the commands that ran as Markdown, for a note or a report.
    pub(super) fn export_blocks(&mut self, format: Export, cx: &mut App) {
        let exported = super::super::blocks::export(self.blocks(), format);
        if exported.is_empty() {
            log::debug!("terminal {}: no command to export", self.pty.pid);
            return;
        }
        log::info!(
            "terminal {}: exports {} commands as {format:?}",
            self.pty.pid,
            self.blocks().len()
        );
        cx.write_to_clipboard(ClipboardItem::new_string(exported));
    }

    /// Where the marks stand now, as grid cells.
    pub(super) fn cell(&self) -> At {
        let term = self.pty.term.lock();
        let point = term.grid().cursor.point;
        At {
            line: point.line.0,
            column: point.column.0,
        }
    }

    /// The line typed since the prompt ended, from the first cell of the input to the cursor.
    ///
    /// The shell's `OSC 133 B` says where its prompt left off and the cursor where the typed line
    /// ends, so the text between them is what the user wrote — across soft wraps too. A program
    /// that redrew the line leaves its own marks; nothing here pretends otherwise.
    pub(super) fn typed(&self, from: At) -> String {
        let term = self.pty.term.lock();
        let grid = term.grid();
        let end = grid.cursor.point;
        let last = Line(end.line.0);
        let mut out = String::new();
        let mut line = Line(from.line);
        while line <= last {
            let row = &grid[line];
            let start = match line == Line(from.line) {
                true => from.column,
                false => 0,
            };
            let stop = match line == last {
                true => end.column.0 + 1,
                false => term.columns(),
            };
            for column in start..stop.min(term.columns()) {
                let cell = &row[Column(column)];
                if cell.flags.intersects(SPACERS) {
                    continue;
                }
                out.push(cell.c);
                out.extend(cell.zerowidth().into_iter().flatten().copied());
            }
            if line != last && !row[term.last_column()].flags.contains(Flags::WRAPLINE) {
                out.push('\n');
            }
            line += 1;
        }
        out.trim_end().to_string()
    }
}
