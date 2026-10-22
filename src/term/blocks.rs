//! Command blocks: what a shell's OSC 133 marks divide a session's work into.

use std::time::{Duration, Instant};

use super::marks::Mark;

/// Where a command block stands.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    /// The prompt showed and waits for a command.
    Prompt,
    /// The user is typing the command.
    Input,
    /// The command runs and its output comes.
    Running,
    /// The command ended, with its status when the shell named one.
    Done(Option<i32>),
}

/// One command: its prompt, the command, its output, and how it ended.
#[derive(Clone, Debug)]
pub struct Block {
    pub id: u64,
    pub phase: Phase,
    /// The prompt's first mark.
    pub started: Instant,
    /// When the command's output began; none for a prompt that ran nothing.
    pub command_at: Option<Instant>,
    /// When the command ended.
    pub ended: Option<Instant>,
    /// The line typed at the prompt, when the shell's marks let it be read.
    pub command: Option<String>,
    /// The grid line the prompt starts on, and the one its output does.
    pub prompt_line: i32,
    pub output_line: i32,
    /// Where the typed line starts: the cursor when the prompt ended, `OSC 133 B`.
    pub input_line: i32,
    pub input_column: usize,
    /// The grid line the command ended on.
    pub end_line: Option<i32>,
}

impl Block {
    /// Whether the command is running now.
    pub fn running(&self) -> bool {
        self.phase == Phase::Running
    }

    /// Whether it ended with a status other than zero.
    pub fn failed(&self) -> bool {
        matches!(self.phase, Phase::Done(Some(status)) if status != 0)
    }

    /// The status the shell named, once it ended.
    pub fn status(&self) -> Option<i32> {
        match self.phase {
            Phase::Done(status) => status,
            _ => None,
        }
    }

    /// How long the command ran, once it ended; none for one that ran nothing.
    pub fn duration(&self) -> Option<Duration> {
        let start = self.command_at?;
        let end = self.ended?;
        Some(end.saturating_duration_since(start))
    }

    /// The command as a copy takes it: its lines joined, trimmed of the prompt's trailing spaces.
    pub fn typed(&self) -> Option<&str> {
        self.command
            .as_deref()
            .map(str::trim)
            .filter(|line| !line.is_empty())
    }
}

/// What a session's commands come to: how many, how many failed, and the slowest.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Summary {
    pub ran: usize,
    pub failed: usize,
    /// The slowest command's line and how long it took.
    pub slowest: Option<(String, Duration)>,
}

impl Blocks {
    /// What its commands come to, from those that ended.
    pub fn summary(&self) -> Summary {
        let mut summary = Summary::default();
        for block in &self.done {
            summary.ran += 1;
            if block.failed() {
                summary.failed += 1;
            }
            let Some(duration) = block.duration() else {
                continue;
            };
            if summary
                .slowest
                .as_ref()
                .is_none_or(|(_, slowest)| duration > *slowest)
            {
                let line = block.typed().unwrap_or("(a command)").to_string();
                summary.slowest = Some((line, duration));
            }
        }
        summary
    }

    /// The block after `from`, or before it, round the ends; with `failed`, only those that failed. Navigation walks the commands that ended, oldest first, so a key finds the same block twice.
    pub fn step(&self, from: Option<usize>, newer: bool, failed: bool) -> Option<usize> {
        let count = self.done.len();
        if count == 0 {
            return None;
        }
        let start = from.map(|at| at.min(count - 1));
        let fits = |at: usize| !failed || self.done[at].failed();
        let mut at = start;
        for _ in 0..count {
            at = match (at, newer) {
                (Some(at), true) => Some((at + 1) % count),
                (Some(at), false) => Some((at + count - 1) % count),
                (None, true) => Some(0),
                (None, false) => Some(count - 1),
            };
            if at.is_some_and(fits) {
                return at;
            }
        }
        None
    }

    /// Every command that failed, oldest first, with its line and status.
    pub fn failures(&self) -> Vec<(u64, &str, Option<i32>)> {
        self.done
            .iter()
            .filter(|block| block.failed())
            .map(|block| (block.id, block.typed().unwrap_or(""), block.status()))
            .collect()
    }
}

/// `blocks` as a text, a Markdown table or JSON, one row per command that ended. A command is what the user typed, its exit status and how long it ran, never its output, which is what a bug report wants and a shared note can hold without pasting a screen.
pub fn export(blocks: &Blocks, format: Export) -> String {
    let rows: Vec<(&Block, Option<&str>)> = blocks
        .done
        .iter()
        .map(|block| (block, block.typed()))
        .collect();
    match format {
        Export::Text => rows
            .iter()
            .map(|(block, typed)| {
                let status = block
                    .status()
                    .map_or_else(|| "?".to_string(), |code| code.to_string());
                let seconds = block.duration().map_or_else(
                    || "?".to_string(),
                    |time| format!("{:.1}s", time.as_secs_f64()),
                );
                format!("{}\t{}\t{}", status, seconds, typed.unwrap_or(""))
            })
            .collect::<Vec<_>>()
            .join("\n"),
        Export::Markdown => {
            let mut out = String::from("| Command | Exit | Time |\n|---|---|---|\n");
            for (block, typed) in rows {
                let status = block
                    .status()
                    .map_or_else(|| "?".to_string(), |code| code.to_string());
                let seconds = block.duration().map_or_else(
                    || "?".to_string(),
                    |time| format!("{:.1}s", time.as_secs_f64()),
                );
                let typed = typed.unwrap_or("").replace('|', "\\|");
                out.push_str(&format!("| `{typed}` | {status} | {seconds} |\n"));
            }
            out
        }
        Export::Json => {
            let rows: Vec<String> = rows
                .iter()
                .map(|(block, typed)| {
                    let status = block
                        .status()
                        .map_or_else(|| "null".to_string(), |code| code.to_string());
                    let seconds = block.duration().map_or_else(
                        || "null".to_string(),
                        |time| format!("{:.3}", time.as_secs_f64()),
                    );
                    format!(
                        "    {{\"id\": {}, \"command\": {}, \"exit\": {}, \"seconds\": {}}}",
                        block.id,
                        json_string(typed.unwrap_or("")),
                        status,
                        seconds
                    )
                })
                .collect();
            format!("[\n{}\n]", rows.join(",\n"))
        }
    }
}

/// The forms a block list is exported in.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Export {
    /// One row per command: exit status, seconds, command, tab-separated.
    Text,
    /// A table a note holds.
    Markdown,
    /// One object per command.
    Json,
}

/// The export a command with this name asks for; none names no form this terminal has.
pub fn export_named(name: &str) -> Option<Export> {
    match name {
        "text" => Some(Export::Text),
        "markdown" => Some(Export::Markdown),
        "json" => Some(Export::Json),
        _ => None,
    }
}

/// `text` as a JSON string, with what JSON may not hold escaped.
fn json_string(text: &str) -> String {
    let mut out = String::with_capacity(text.len() + 2);
    out.push('"');
    for c in text.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// The blocks of one session, the current one and the last few that ended.
#[derive(Debug, Default)]
pub struct Blocks {
    current: Option<Block>,
    done: Vec<Block>,
    next: u64,
}

/// How many ended blocks a session keeps.
pub const KEEP: usize = 512;

/// A command longer than this is worth a notification when it ends.
pub const SLOW: Duration = Duration::from_secs(10);

/// What a notification says of `block` when it ran long enough, else none.
pub fn completion(block: &Block) -> Option<String> {
    let duration = block.duration()?;
    if duration < SLOW {
        return None;
    }
    let seconds = duration.as_secs_f64();
    Some(match block.status() {
        Some(0) | None => format!("Command finished in {seconds:.1}s"),
        Some(status) => format!("Command failed with exit {status} after {seconds:.1}s"),
    })
}

/// Marks a block follows: a grid line and a column on it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct At {
    pub line: i32,
    pub column: usize,
}

impl Blocks {
    /// Notes a mark `at`, with no grid cell to place it by.
    #[cfg(test)]
    pub(crate) fn mark(&mut self, mark: Mark, at: Instant) {
        self.mark_at(mark, at, At::default());
    }

    /// Notes a mark `at` on grid cell `cell`; marks outside a block change nothing.
    pub(crate) fn mark_at(&mut self, mark: Mark, at: Instant, cell: At) {
        match mark {
            Mark::Prompt(_) => {
                if let Some(ended) = self.current.take().filter(|block| block.ended.is_some()) {
                    self.keep(ended);
                }
                self.current = Some(Block {
                    id: self.next,
                    phase: Phase::Prompt,
                    started: at,
                    command_at: None,
                    ended: None,
                    command: None,
                    prompt_line: cell.line,
                    output_line: cell.line,
                    input_line: cell.line,
                    input_column: cell.column,
                    end_line: None,
                });
                self.next += 1;
            }
            Mark::Input => {
                if let Some(block) = self.current.as_mut()
                    && block.phase == Phase::Prompt
                {
                    block.phase = Phase::Input;
                    block.input_line = cell.line;
                    block.input_column = cell.column;
                }
            }
            Mark::Output => {
                if let Some(block) = self.current.as_mut() {
                    block.phase = Phase::Running;
                    block.command_at = Some(at);
                    block.output_line = cell.line;
                }
            }
            Mark::Done(status) => {
                if let Some(block) = self.current.as_mut() {
                    block.phase = Phase::Done(status);
                    block.ended = Some(at);
                    block.end_line = Some(cell.line);
                }
            }
        }
    }

    /// Keeps `command`, the line typed at the prompt, on the block whose prompt is at `prompt_line`.
    pub(crate) fn typed(&mut self, command: String, prompt_line: i32) {
        let found = self
            .current
            .as_mut()
            .filter(|block| block.prompt_line == prompt_line)
            .or_else(|| {
                self.done
                    .iter_mut()
                    .rev()
                    .find(|block| block.prompt_line == prompt_line)
            });
        if let Some(block) = found
            && block.command.is_none()
        {
            block.command = Some(command);
        }
    }

    /// The prompt line of the block running now: where the user is typing.
    pub fn typing_at(&self) -> Option<(i32, At)> {
        self.current.as_ref().map(|block| {
            (
                block.prompt_line,
                At {
                    line: block.input_line,
                    column: block.input_column,
                },
            )
        })
    }

    /// The block the prompt is on now.
    pub fn current(&self) -> Option<&Block> {
        self.current.as_ref()
    }

    /// The blocks that ended, oldest first.
    pub fn done(&self) -> &[Block] {
        &self.done
    }

    /// The current block, else the last that ended.
    pub fn last(&self) -> Option<&Block> {
        self.current.as_ref().or_else(|| self.done.last())
    }

    /// The current block when it ended, else the last block that ended.
    pub fn last_finished(&self) -> Option<&Block> {
        self.current
            .as_ref()
            .filter(|block| block.ended.is_some())
            .or_else(|| self.done.last())
    }

    /// How many blocks ended and are kept.
    pub fn len(&self) -> usize {
        self.done.len()
    }

    /// Whether no block ended.
    pub fn is_empty(&self) -> bool {
        self.done.is_empty()
    }

    fn keep(&mut self, block: Block) {
        self.done.push(block);
        if self.done.len() > KEEP {
            self.done.remove(0);
        }
    }
}

#[cfg(test)]
mod tests;
