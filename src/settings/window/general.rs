//! The general rows: the language of the interface and the hosts ssh knows.

use ely_gpui_component::settings::{SettingsRow, SettingsSection};
use gpui::{App, Div, Entity, ParentElement, Styled, div};

use super::SettingsWindow;
use crate::settings::{Language, SettingsStore, language};
use crate::ssh;

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
        let hosts = cx.global::<ssh::Hosts>();
        let description = if hosts.hosts.is_empty() {
            "No hosts in ~/.ssh/config".to_string()
        } else {
            let names: Vec<&str> = hosts
                .hosts
                .iter()
                .take(8)
                .map(|host| host.name.as_str())
                .collect();
            let more = hosts.hosts.len().saturating_sub(names.len());
            format!(
                "{} from {} files: {}{}",
                hosts.hosts.len(),
                hosts.files.len(),
                names.join(", "),
                if more > 0 {
                    format!(", and {more} more")
                } else {
                    String::new()
                }
            )
        };
        let remote = SettingsSection::new("Remote")
            .row(SettingsRow::new("SSH hosts").description(description));
        div().flex().flex_col().gap_6().child(section).child(remote)
    }
}
