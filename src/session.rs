//! The tabs a window had, written so the next start can show them again.

use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

/// The file's shape; a file of another version is not read.
pub const VERSION: u32 = 1;

/// The most tabs a file keeps, so a broken one cannot open a hundred programs.
pub const MAX: usize = 16;

/// A window as it is kept: its tabs and the one shown.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Saved {
    pub version: u32,
    /// The tab shown, clamped to the tabs the file holds.
    pub active: usize,
    pub tabs: Vec<SavedTab>,
}

/// One tab as it is kept: its name and the folder its session showed.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SavedTab {
    pub name: String,
    pub cwd: String,
}

/// Where the session file lives.
pub fn path() -> Option<PathBuf> {
    dirs::config_dir().map(|folder| folder.join("ELYTerminal").join("session.json"))
}

/// The tabs in `path`, or none when there is no file, it does not read, or it keeps none.
pub fn load(path: &Path) -> Option<Saved> {
    let text = match crate::settings::read_text(path) {
        Ok(text) => text,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return None,
        Err(error) => {
            log::warn!("session: cannot read {}: {error}", path.display());
            return None;
        }
    };
    let saved: Saved = match serde_json::from_str(&text) {
        Ok(saved) => saved,
        Err(error) => {
            log::warn!("session: {} does not parse: {error}", path.display());
            return None;
        }
    };
    if saved.version != VERSION || saved.tabs.is_empty() {
        log::debug!(
            "session: {} holds version {} with {} tabs; none to restore",
            path.display(),
            saved.version,
            saved.tabs.len()
        );
        return None;
    }
    let mut saved = saved;
    saved.tabs.truncate(MAX);
    saved.active = saved.active.min(saved.tabs.len() - 1);
    Some(saved)
}

/// Writes `saved` to `path`, a temp file first so a crash leaves the old one whole.
pub fn save(path: &Path, saved: &Saved) -> anyhow::Result<()> {
    static SAVES: AtomicU64 = AtomicU64::new(0);
    let folder = path
        .parent()
        .with_context(|| format!("session: {} has no folder", path.display()))?;
    fs::create_dir_all(folder)
        .with_context(|| format!("session: cannot create {}", folder.display()))?;
    let mut saved = saved.clone();
    saved.version = VERSION;
    saved.tabs.truncate(MAX);
    if !saved.tabs.is_empty() {
        saved.active = saved.active.min(saved.tabs.len() - 1);
    }
    let mut text = serde_json::to_string_pretty(&saved).context("session: cannot encode")?;
    text.push('\n');
    let count = SAVES.fetch_add(1, Ordering::Relaxed);
    let temp = path.with_extension(format!("json.{}-{count}.tmp", std::process::id()));
    let written = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temp)
        .and_then(|mut file| {
            file.write_all(text.as_bytes())?;
            file.sync_all()
        });
    if let Err(error) = written {
        let _ = fs::remove_file(&temp);
        return Err(error).with_context(|| format!("session: cannot write {}", temp.display()));
    }
    fs::rename(&temp, path).with_context(|| {
        format!(
            "session: cannot put {} in place of {}",
            temp.display(),
            path.display()
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests;
