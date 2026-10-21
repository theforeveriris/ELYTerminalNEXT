//! The language of the interface, and the messages of each.

use std::env;

use ely_gpui_component::i18n::{I18n, Locale, text};
use gpui::App;
use serde::{Deserialize, Serialize};

/// The language the interface is written in.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Language {
    /// The language the system speaks.
    #[default]
    System,
    English,
    Chinese,
}

impl Language {
    /// The choices the settings window offers, in their order.
    pub const ALL: [Language; 3] = [Language::System, Language::English, Language::Chinese];

    /// The tag of the language, as the messages name it.
    pub fn tag(self) -> Option<&'static str> {
        match self {
            Language::System => None,
            Language::English => Some("en-US"),
            Language::Chinese => Some("zh-CN"),
        }
    }

    /// The tag the interface shows, the system's language when the choice is the system.
    pub fn resolved(self) -> &'static str {
        self.tag().unwrap_or_else(system_tag)
    }

    /// The name of the language, in that language.
    pub fn name(self) -> &'static str {
        match self {
            Language::System => "System",
            Language::English => "English",
            Language::Chinese => "中文",
        }
    }

    /// The name of the language, in the language the interface is in.
    pub fn named(self, cx: &App) -> String {
        match self {
            Language::System => text(cx, "settings.language.system", &[]).to_string(),
            _ => self.name().to_string(),
        }
    }
}

/// The language the system speaks, as a tag Ely knows.
pub fn system_tag() -> &'static str {
    for tag in ["zh-CN", "en-US"] {
        if speaks(tag) {
            return tag;
        }
    }
    "en-US"
}

/// Whether the system speaks the language of `tag`.
fn speaks(tag: &str) -> bool {
    let Ok(locale) = env::var("LANG").or_else(|_| env::var("LC_ALL")) else {
        return false;
    };
    let locale = locale.to_lowercase();
    let wanted = tag.to_lowercase();
    locale.starts_with(&wanted[..wanted.find('-').unwrap_or(wanted.len())])
}

/// The messages of each language, keyed by the English text they stand for.
const EN: &[(&str, &str)] = &[
    ("settings.language.system", "System"),
    ("settings.language.label", "Language"),
    (
        "settings.language.description",
        "The language of the interface",
    ),
];

const ZH: &[(&str, &str)] = &[
    ("settings.language.system", "跟随系统"),
    ("settings.language.label", "语言"),
    ("settings.language.description", "界面使用的语言"),
];

/// Sets the interface's language in every window.
pub fn apply(language: Language, cx: &mut App) {
    let tag = language.resolved();
    if cx.has_global::<I18n>() {
        I18n::set_locale(tag, cx);
        return;
    }
    let i18n = I18n::new(tag).catalog("en-US", EN).catalog("zh-CN", ZH);
    cx.set_global(i18n);
}

/// The locale the interface shows, for the numbers and dates it writes.
pub fn locale(language: Language) -> Locale {
    Locale::of(language.resolved())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_system_language_is_english_unless_it_speaks_chinese() {
        // The test process carries the CI's locale, which is not Chinese.
        assert!(["en-US", "zh-CN"].contains(&system_tag()));
    }

    #[test]
    fn a_language_resolves_to_a_tag_ely_knows() {
        assert_eq!(Language::English.resolved(), "en-US");
        assert_eq!(Language::Chinese.resolved(), "zh-CN");
        for tag in [Language::English.tag(), Language::Chinese.tag()] {
            let tag = tag.expect("a tag");
            assert_eq!(Locale::of(tag).tag, tag, "Ely knows {tag}");
        }
    }

    #[test]
    fn the_system_language_has_no_tag_of_its_own() {
        assert_eq!(Language::System.tag(), None);
        assert_eq!(Language::System.resolved(), system_tag());
    }

    #[test]
    fn every_language_has_a_message_for_the_same_keys() {
        fn keys(messages: &[(&'static str, &'static str)]) -> Vec<&'static str> {
            let mut keys: Vec<&'static str> = messages.iter().map(|(key, _)| *key).collect();
            keys.sort();
            keys
        }
        assert_eq!(keys(EN), keys(ZH), "the catalogs hold the same keys");
    }
}
