//! The general rows: the language of the interface.

use ely_gpui_component::settings::{SettingsRow, SettingsSection};
use gpui::{App, Div, Entity, ParentElement, Styled, div};

use super::SettingsWindow;
use crate::settings::{Language, SettingsStore, language};

impl SettingsWindow {
    /// The general page: the language of the interface.
    pub(super) fn general(&self, this: &Entity<Self>, cx: &App) -> Div {
        let settings = SettingsStore::get(cx);
        let control = super::choice(
            this,
            "language",
            &[
                (
                    Language::System,
                    "system",
                    language::message(cx, "settings.language.system"),
                ),
                (Language::English, "en", "English".into()),
                (Language::Chinese, "zh", "中文".into()),
            ],
            settings.language,
            |settings, language| settings.language = language,
        );
        let section = SettingsSection::new(language::message(cx, "settings.language.section")).row(
            SettingsRow::new(language::message(cx, "settings.language.label"))
                .description(language::message(cx, "settings.language.description"))
                .control(control),
        );
        div().flex().flex_col().gap_6().child(section)
    }
}
