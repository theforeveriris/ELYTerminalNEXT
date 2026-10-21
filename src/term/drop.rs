//! Files dropped on the terminal type their paths, as macOS Terminal does.

use std::path::{Path, PathBuf};

use anyhow::Context as _;
use gpui::{Context, ExternalPaths};

use super::view::TerminalView;

/// What a POSIX shell reads specially in a path: Ghostty's list, and zsh's `extendedglob` marks.
const SPECIAL: &str = "\\ ()[]{}<>\"'`!#$&;|*?^~";

/// PowerShell's single quotes, each one doubled inside a quoted string.
const SINGLE: [char; 5] = ['\'', '\u{2018}', '\u{2019}', '\u{201a}', '\u{201b}'];

/// Where a session's shell reads its paths, which decides how a dropped path is written.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Target {
    /// A shell on this machine.
    Local,
    /// Git Bash, which reads `C:\x` as `/c/x`.
    GitBash,
    /// A WSL distribution, which reads `C:\x` as `/mnt/c/x`.
    Wsl { distro: String },
}

impl Target {
    /// `path` as this target's shell reads it; a path with no such form stays as it is.
    pub fn convert(&self, path: &Path) -> PathBuf {
        let Some(text) = path.to_str() else {
            return path.to_path_buf();
        };
        let mapped = match self {
            Self::Local => None,
            Self::GitBash => crate::location::windows_to_posix(text),
            Self::Wsl { .. } => crate::location::windows_to_wsl(text),
        };
        mapped
            .map(PathBuf::from)
            .unwrap_or_else(|| path.to_path_buf())
    }
}

/// The text dropping `paths` on a shell at `target` types, each followed by a space.
pub fn typed_for(paths: &[PathBuf], target: &Target, quote: Quote) -> anyhow::Result<String> {
    let mapped: Vec<PathBuf> = paths.iter().map(|path| target.convert(path)).collect();
    typed(&mapped, quote)
}

/// How a session's shell reads a path typed into it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Quote {
    /// sh, bash, zsh, fish: a backslash before each special character.
    Posix,
    /// Single quotes, which expand nothing.
    PowerShell,
    /// Double quotes, past which cmd still expands `%` and, under `/v:on`, `!`.
    Cmd,
}

/// The text `paths` type, each followed by a space, as `quote` keeps each one literal.
pub fn typed(paths: &[PathBuf], quote: Quote) -> anyhow::Result<String> {
    paths
        .iter()
        .map(|path| {
            let text = (path.to_str())
                .with_context(|| format!("{} is no UTF-8 a session can type", path.display()))?;
            anyhow::ensure!(
                !text.chars().any(char::is_control),
                "{text:?} holds a control character, which would act as typed"
            );
            let mut out = String::with_capacity(text.len() + 3);
            match quote {
                Quote::Posix => text.chars().for_each(|ch| {
                    if SPECIAL.contains(ch) {
                        out.push('\\');
                    }
                    out.push(ch);
                }),
                Quote::PowerShell => {
                    out.push('\'');
                    text.chars().for_each(|ch| {
                        if SINGLE.contains(&ch) {
                            out.push(ch);
                        }
                        out.push(ch);
                    });
                    out.push('\'');
                }
                Quote::Cmd => {
                    anyhow::ensure!(
                        !text.contains(['%', '!', '"']),
                        "{text:?} holds %, ! or \", which cmd reads inside quotes"
                    );
                    // A program reads a backslash before the closing quote as a quote.
                    let trailing = text.len() - text.trim_end_matches('\\').len();
                    out = format!("\"{text}{}\"", "\\".repeat(trailing));
                }
            }
            out.push(' ');
            Ok(out)
        })
        .collect()
}

impl TerminalView {
    /// Types the dropped paths at the cursor as a paste; a path it cannot type stops the drop.
    pub(super) fn drop_paths(&mut self, dropped: &ExternalPaths, cx: &mut Context<Self>) {
        let (paths, quote) = (dropped.paths(), self.pty.quote);
        match typed_for(paths, &self.pty.target, quote) {
            Ok(text) => {
                log::info!(
                    "terminal {}: {} paths dropped, quoted for {quote:?} at {:?}",
                    self.pty.pid,
                    paths.len(),
                    self.pty.target
                );
                self.paste_text(&text, cx);
            }
            Err(error) => log::error!(
                "terminal {}: the drop types nothing: {error:#}",
                self.pty.pid
            ),
        }
    }
}

#[cfg(test)]
mod tests;
