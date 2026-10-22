//! The window's title, as the shown session's name, folder and program compose it.

use gpui::App;

use super::Workspace;
use crate::settings::{SettingsStore, title};

impl Workspace {
    /// The title the window shows: the composed template, or the session's name when there is none.
    pub(super) fn window_title(&self, cx: &App) -> String {
        let name = self.title();
        let Some(session) = self.session(self.active) else {
            return name.to_string();
        };
        let program = session
            .launch
            .program
            .as_ref()
            .map_or("", |(program, _)| program);
        title::compose(
            &SettingsStore::get(cx).window_title,
            name,
            &session.folder,
            dirs::home_dir().as_deref(),
            program,
        )
    }
}
