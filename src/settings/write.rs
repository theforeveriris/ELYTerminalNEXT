//! Writing the settings file: a temp file beside it, renamed over it, so a crash leaves the old one.

use std::{
    fs,
    io::Write,
    path::Path,
    sync::atomic::{AtomicU64, Ordering},
};

use anyhow::Context as _;

use super::Settings;

/// Whether `path` names a TOML file.
pub(super) fn is_toml(path: &Path) -> bool {
    path.extension().and_then(|extension| extension.to_str()) == Some("toml")
}

/// Writes a temp file beside `path`, then renames it over `path`.
pub fn save(path: &Path, settings: &Settings) -> anyhow::Result<()> {
    let mut settings = settings.clone();
    settings.extra.clear();
    let folder = path
        .parent()
        .with_context(|| format!("settings: {} has no folder", path.display()))?;
    fs::create_dir_all(folder)
        .with_context(|| format!("settings: cannot create {}", folder.display()))?;
    let mut text = if is_toml(path) {
        toml::to_string_pretty(&settings).context("settings: cannot encode TOML")?
    } else {
        serde_json::to_string_pretty(&settings).context("settings: cannot encode")?
    };
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
        })
        .and_then(|()| fs::rename(&temp, path));
    if let Err(error) = written {
        if temp.exists()
            && let Err(left) = fs::remove_file(&temp)
        {
            log::error!("settings: {} stays behind: {left}", temp.display());
        }
        return Err(error).with_context(|| {
            format!(
                "settings: cannot write {} through {}",
                path.display(),
                temp.display()
            )
        });
    }
    log::info!("settings: saved {}", path.display());
    Ok(())
}

/// Saves made by this process, so each temp file has its own name.
static SAVES: AtomicU64 = AtomicU64::new(0);
