//! The general rows: the language of the interface.

use ely_gpui_component::settings::{SettingsRow, SettingsSection};
use gpui::{Context, Div, ParentElement, Styled, div};

use super::SettingsWindow;
use crate::settings::{Language, SettingsStore};

/// The languages the interface offers, each a value, its key and its name.
const LANGUAGES: [(Language, &str, &str); 3] = [
    (Language::System, "system", "System"),
    (Language::English, "en", "English"),
    (Language::Chinese, "zh", "中文"),
];

impl SettingsWindow {
    /// The general page: the language of the interface.
    pub(super) fn general(&self, cx: &Context<Self>) -> Div {
        let settings = SettingsStore::get(cx);
        let control = super::choice(
            &cx.entity(),
            "language",
            &LANGUAGES,
            settings.language,
            |settings, language| settings.language = language,
        );
        let section = SettingsSection::new("Language").row(
            SettingsRow::new("Language")
                .description("The language of the interface")
                .control(control),
        );
        div().flex().flex_col().gap_6().child(section)
    }
}
