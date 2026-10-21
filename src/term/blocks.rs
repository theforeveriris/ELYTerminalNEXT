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

impl Blocks {
    /// Notes a mark `at`; marks outside a block change nothing.
    pub(crate) fn mark(&mut self, mark: Mark, at: Instant) {
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
                });
                self.next += 1;
            }
            Mark::Input => {
                if let Some(block) = self.current.as_mut()
                    && block.phase == Phase::Prompt
                {
                    block.phase = Phase::Input;
                }
            }
            Mark::Output => {
                if let Some(block) = self.current.as_mut() {
                    block.phase = Phase::Running;
                    block.command_at = Some(at);
                }
            }
            Mark::Done(status) => {
                if let Some(block) = self.current.as_mut() {
                    block.phase = Phase::Done(status);
                    block.ended = Some(at);
                }
            }
        }
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
