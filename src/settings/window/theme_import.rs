//! Importing a theme file into the user's own colors.

use std::fs;

use gpui::{App, Entity, PathPromptOptions, WindowAppearance};

use super::SettingsWindow;
use crate::{settings::SettingsStore, themes};

/// Asks for a theme file, imports its colors into the custom theme, and shows why when it cannot.
pub(super) fn import_theme(this: &Entity<SettingsWindow>, cx: &mut App) {
    let picks = cx.prompt_for_paths(PathPromptOptions {
        files: true,
        directories: false,
        multiple: false,
        prompt: Some("Import a theme".into()),
    });
    let this = this.clone();
    cx.spawn(async move |cx| {
        let picked = match picks.await {
            Ok(Ok(Some(paths))) => paths.into_iter().next(),
            Ok(Ok(None)) => return,
            Ok(Err(error)) => {
                log::warn!("themes: the picker failed: {error:#}");
                return;
            }
            Err(error) => {
                log::warn!("themes: the picker closed: {error}");
                return;
            }
        };
        let Some(path) = picked else {
            return;
        };
        let imported = themes::import::read(&path);
        let failed = match imported {
            Ok(colors) => {
                let saved = cx.update(|cx| {
                    SettingsStore::update(cx, |settings| {
                        settings.custom_colors = colors;
                        settings.theme = themes::CUSTOM.into();
                    })
                });
                match saved {
                    Ok(()) => {
                        log::info!("themes: imported {}", path.display());
                        None
                    }
                    Err(error) => Some(format!("{error:#}").into()),
                }
            }
            Err(error) => {
                log::warn!("themes: {} does not import: {error:#}", path.display());
                Some(format!("{error:#}").into())
            }
        };
        this.update(cx, |this, cx| {
            this.failed = failed;
            this.drafts += 1;
            cx.notify();
        });
    })
    .detach();
}

/// Asks where to write the theme the window wears, and writes it as a Ghostty file.
pub(super) fn export_theme(
    this: &Entity<SettingsWindow>,
    appearance: WindowAppearance,
    cx: &mut App,
) {
    let theme = themes::active(SettingsStore::get(cx), appearance, cx);
    let colors = themes::export::colors(&theme);
    let Some(home) = dirs::home_dir() else {
        log::warn!("themes: no home folder to write a theme in");
        return;
    };
    let picks = cx.prompt_for_new_path(&home, Some("theme"));
    let this = this.clone();
    cx.spawn(async move |cx| {
        let picked = match picks.await {
            Ok(Ok(Some(path))) => Some(path),
            Ok(Ok(None)) => None,
            Ok(Err(error)) => {
                log::warn!("themes: the save dialog failed: {error:#}");
                None
            }
            Err(error) => {
                log::warn!("themes: the save dialog closed: {error}");
                None
            }
        };
        let Some(path) = picked else {
            return;
        };
        let text = themes::export::ghostty(&colors);
        let failed = match fs::write(&path, text) {
            Ok(()) => {
                log::info!("themes: wrote {}", path.display());
                None
            }
            Err(error) => {
                log::warn!("themes: cannot write {}: {error}", path.display());
                Some(format!("{error}").into())
            }
        };
        this.update(cx, |this, cx| {
            this.failed = failed;
            this.drafts += 1;
            cx.notify();
        });
    })
    .detach();
}
