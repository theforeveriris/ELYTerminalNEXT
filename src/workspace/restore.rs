//! Keeping the window's tabs, and opening them again at the next start.

use std::{path::PathBuf, time::Duration};

use gpui::{Context, Window};

use super::{Tab, Workspace};
use crate::{
    session,
    settings::SettingsStore,
    term::{
        config,
        shell::{ShellIntegration, launch},
        spawn,
    },
};

impl Workspace {
    /// The tabs the last window kept, when they can be read.
    pub(super) fn saved() -> Option<session::Saved> {
        session::path().and_then(|path| session::load(&path))
    }

    /// Adds the saved tabs after the first one, then shows the tab saved as active.
    pub(super) fn restore(
        &mut self,
        saved: session::Saved,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let settings = SettingsStore::get(cx).clone();
        let dir = cx.global::<ShellIntegration>().0.clone();
        let home = dirs::home_dir().unwrap_or_default();
        for snapshot in saved.tabs.iter().skip(1) {
            let folder = PathBuf::from(&snapshot.cwd);
            let folder = if folder.is_dir() {
                folder
            } else {
                home.clone()
            };
            let started = launch(&settings.shell, folder, &dir).and_then(|launch| {
                Ok((
                    spawn(&launch, (80, 24), (8, 16), config(&settings))?,
                    launch,
                ))
            });
            let ((pty, events), launch) = match started {
                Ok(started) => started,
                Err(error) => {
                    log::warn!(
                        "app: the tab {:?} does not open again: {error:#}",
                        snapshot.name
                    );
                    continue;
                }
            };
            let id = self.add(pty, events, launch, window, cx);
            let mut tab = Tab::new(id);
            tab.name = snapshot.name.clone();
            self.tabs.push(tab);
        }
        let active = saved.active.min(self.tabs.len().saturating_sub(1));
        self.select_tab(active, window, cx);
        log::info!("app: restored {} tabs", self.tabs.len());
    }

    /// Writes the tabs after a moment, so a burst of changes saves once; off when restore is off.
    pub(super) fn save_soon(&mut self, cx: &mut Context<Self>) {
        if !SettingsStore::get(cx).restore_session {
            self.saving = None;
            return;
        }
        self.saving = Some(cx.spawn(async move |this, cx| {
            cx.background_executor()
                .timer(Duration::from_millis(500))
                .await;
            if this.update(cx, |this, cx| this.save_tabs(cx)).is_err() {
                log::debug!("app: the window closed before its tabs were kept");
            }
        }));
    }

    /// Writes the tabs and the one shown to the session file.
    fn save_tabs(&mut self, _: &mut Context<Self>) {
        let Some(path) = session::path() else {
            return;
        };
        let tabs: Vec<session::SavedTab> = self
            .tabs
            .iter()
            .take(session::MAX)
            .map(|tab| {
                let cwd = tab
                    .focused_session()
                    .and_then(|id| self.session(id))
                    .map(|session| session.folder.display().to_string())
                    .unwrap_or_default();
                session::SavedTab {
                    name: tab.name.clone(),
                    cwd,
                }
            })
            .collect();
        let saved = session::Saved {
            version: session::VERSION,
            active: self.active_tab.min(tabs.len().saturating_sub(1)),
            tabs,
        };
        match session::save(&path, &saved) {
            Ok(()) => log::debug!("app: kept {} tabs", saved.tabs.len()),
            Err(error) => log::error!("app: the tabs are not kept: {error:#}"),
        }
    }
}
