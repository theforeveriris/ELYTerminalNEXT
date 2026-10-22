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
    let name = wanted.split('-').next().unwrap_or(&wanted);
    locale.starts_with(name)
}

/// The messages of each language, keyed by the English text they stand for.
const EN: &[(&str, &str)] = &[
    // The categories and the window.
    ("settings.window", "Settings"),
    ("settings.category.appearance", "Appearance"),
    ("settings.category.terminal", "Terminal"),
    ("settings.category.general", "General"),
    // The general page.
    ("settings.language.section", "Language"),
    ("settings.language.label", "Language"),
    (
        "settings.language.description",
        "The language of the interface",
    ),
    ("settings.language.system", "System"),
    // The appearance page.
    ("settings.theme.section", "Theme"),
    ("settings.theme.separate", "Separate dark theme"),
    ("settings.theme.light", "Light"),
    ("settings.theme.dark", "Dark"),
    ("settings.font.section", "Font"),
    ("settings.font.family", "Family"),
    ("settings.font.fallbacks", "Fallbacks"),
    (
        "settings.font.fallbacks.hint",
        "Tried in order for characters the family lacks",
    ),
    ("settings.font.size", "Size"),
    ("settings.font.height", "Line height"),
    ("settings.font.ligatures", "Ligatures"),
    ("settings.font.add", "Add fallback"),
    ("settings.font.remove", "Remove"),
    ("settings.cursor.section", "Cursor"),
    ("settings.cursor.style", "Style"),
    ("settings.cursor.blink", "Blink"),
    ("settings.cursor.preview", "Preview"),
    ("settings.cursor.block", "Block"),
    ("settings.cursor.bar", "Bar"),
    ("settings.cursor.underline", "Underline"),
    ("settings.cursor.hollow", "Hollow"),
    ("settings.window.section", "Window"),
    ("settings.window.ui_size", "Interface size"),
    ("settings.window.opacity", "Background opacity"),
    ("settings.window.blur", "Background blur"),
    ("settings.window.padding_x", "Horizontal padding"),
    ("settings.window.padding_y", "Vertical padding"),
    ("settings.window.custom", "Custom theme"),
    ("settings.custom.from_theme", "Start from the theme in use"),
    ("settings.custom.window", "Window"),
    ("settings.custom.cursor", "Cursor"),
    ("settings.custom.selection", "Selection"),
    ("settings.custom.ansi", "ANSI colors"),
    ("settings.custom.name_hint", "Name this theme"),
    ("settings.custom.save", "Save as theme"),
    ("settings.custom.saved", "Saved themes"),
    ("settings.custom.delete", "Delete"),
    ("settings.window.cancel", "Cancel"),
    // The terminal page.
    ("settings.terminal.section", "Terminal"),
    ("settings.terminal.shell", "Shell"),
    ("settings.terminal.shell.hint", "Login shell"),
    ("settings.terminal.start", "Start in"),
    ("settings.terminal.start.home", "Home"),
    ("settings.terminal.start.session", "The session's folder"),
    ("settings.terminal.start.fixed", "A fixed folder"),
    ("settings.terminal.start.last", "The last folder"),
    ("settings.terminal.start.folder", "Start folder"),
    (
        "settings.terminal.start.folder.hint",
        "Read when the start is a fixed folder",
    ),
    ("settings.terminal.restore", "Reopen last time's tabs"),
    ("settings.terminal.editor", "Editor"),
    ("settings.terminal.title", "Window title"),
    (
        "settings.terminal.title.hint",
        "{title}, {cwd} and {shell} stand for the session, its folder and its program",
    ),
    (
        "settings.terminal.editor.hint",
        "Opens files from the panel; empty opens them with the system",
    ),
    ("settings.terminal.scrollback", "Scrollback lines"),
    ("settings.terminal.option", "Option as Alt"),
    ("settings.terminal.option.off", "Off"),
    ("settings.terminal.option.left", "Left"),
    ("settings.terminal.option.right", "Right"),
    ("settings.terminal.option.both", "Both"),
    ("settings.terminal.copy", "Copy on select"),
    ("settings.terminal.bell", "Flash on bell"),
    ("settings.terminal.still", "Reduce motion"),
    ("settings.terminal.quiet", "Do not disturb"),
    (
        "settings.terminal.confirm",
        "Confirm closing a running command",
    ),
    ("settings.terminal.ely", "Ely"),
    (
        "settings.terminal.updates",
        "Check for updates automatically",
    ),
    ("settings.terminal.updates.now", "Check now"),
    ("settings.terminal.updates.found", "The last check found"),
    ("settings.terminal.shortcuts", "Shortcuts"),
    ("settings.terminal.search", "Search shortcuts"),
    // The workspace.
    ("workspace.new_tab", "New tab"),
    ("workspace.close_tab", "Close tab"),
    ("workspace.split", "Split the pane"),
    ("workspace.new_session", "New session"),
    ("workspace.filter", "Filter sessions"),
    ("workspace.close_session", "Close session"),
    ("workspace.no_folder", "This shell reports no folder."),
    ("workspace.no_session", "No session could start."),
    ("workspace.copy", "Copy"),
    ("workspace.git", "Git"),
    ("workspace.files", "Files"),
];

const ZH: &[(&str, &str)] = &[
    ("settings.window", "设置"),
    ("settings.category.appearance", "外观"),
    ("settings.category.terminal", "终端"),
    ("settings.category.general", "通用"),
    ("settings.language.section", "语言"),
    ("settings.language.label", "语言"),
    ("settings.language.description", "界面使用的语言"),
    ("settings.language.system", "跟随系统"),
    ("settings.theme.section", "主题"),
    ("settings.theme.separate", "单独的深色主题"),
    ("settings.theme.light", "浅色"),
    ("settings.theme.dark", "深色"),
    ("settings.font.section", "字体"),
    ("settings.font.family", "字体族"),
    ("settings.font.fallbacks", "回退字体"),
    (
        "settings.font.fallbacks.hint",
        "主字体族缺失某字符时按顺序尝试",
    ),
    ("settings.font.size", "字号"),
    ("settings.font.height", "行高"),
    ("settings.font.ligatures", "连字"),
    ("settings.font.add", "添加回退字体"),
    ("settings.font.remove", "移除"),
    ("settings.cursor.section", "光标"),
    ("settings.cursor.style", "样式"),
    ("settings.cursor.blink", "闪烁"),
    ("settings.cursor.preview", "预览"),
    ("settings.cursor.block", "块状"),
    ("settings.cursor.bar", "竖线"),
    ("settings.cursor.underline", "下划线"),
    ("settings.cursor.hollow", "空心"),
    ("settings.window.section", "窗口"),
    ("settings.window.ui_size", "界面字号"),
    ("settings.window.opacity", "背景不透明度"),
    ("settings.window.blur", "背景模糊"),
    ("settings.window.padding_x", "水平内边距"),
    ("settings.window.padding_y", "垂直内边距"),
    ("settings.window.custom", "自定义主题"),
    ("settings.custom.from_theme", "从当前主题载入"),
    ("settings.custom.window", "界面"),
    ("settings.custom.cursor", "光标"),
    ("settings.custom.selection", "选区"),
    ("settings.custom.ansi", "ANSI 颜色"),
    ("settings.custom.name_hint", "给这个主题起个名字"),
    ("settings.custom.save", "保存为主题"),
    ("settings.custom.saved", "已保存的主题"),
    ("settings.custom.delete", "删除"),
    ("settings.window.cancel", "取消"),
    ("settings.terminal.section", "终端"),
    ("settings.terminal.shell", "Shell"),
    ("settings.terminal.shell.hint", "登录 shell"),
    ("settings.terminal.start", "起始目录"),
    ("settings.terminal.start.home", "Home"),
    ("settings.terminal.start.session", "会话所在目录"),
    ("settings.terminal.start.fixed", "固定目录"),
    ("settings.terminal.start.last", "上次的目录"),
    ("settings.terminal.start.folder", "起始文件夹"),
    (
        "settings.terminal.start.folder.hint",
        "起始目录选固定目录时使用",
    ),
    ("settings.terminal.restore", "恢复上次的标签页"),
    ("settings.terminal.editor", "编辑器"),
    (
        "settings.terminal.editor.hint",
        "从文件面板打开文件；留空则交给系统默认程序",
    ),
    ("settings.terminal.title", "窗口标题"),
    (
        "settings.terminal.title.hint",
        "{title}、{cwd}、{shell} 分别代表会话名、目录与程序",
    ),
    ("settings.terminal.scrollback", "回滚行数"),
    ("settings.terminal.option", "Option 视作 Alt"),
    ("settings.terminal.option.off", "关闭"),
    ("settings.terminal.option.left", "左侧"),
    ("settings.terminal.option.right", "右侧"),
    ("settings.terminal.option.both", "两侧"),
    ("settings.terminal.copy", "选中即复制"),
    ("settings.terminal.bell", "响铃时闪烁"),
    ("settings.terminal.still", "减少动态效果"),
    ("settings.terminal.quiet", "勿扰模式"),
    ("settings.terminal.confirm", "关闭运行中的命令前确认"),
    ("settings.terminal.ely", "Ely"),
    ("settings.terminal.updates", "自动检查更新"),
    ("settings.terminal.updates.now", "立即检查"),
    ("settings.terminal.updates.found", "上次检查的结果"),
    ("settings.terminal.shortcuts", "快捷键"),
    ("settings.terminal.search", "搜索快捷键"),
    ("workspace.new_tab", "新建标签页"),
    ("workspace.close_tab", "关闭标签页"),
    ("workspace.split", "分屏"),
    ("workspace.new_session", "新建会话"),
    ("workspace.filter", "筛选会话"),
    ("workspace.close_session", "关闭会话"),
    ("workspace.no_folder", "此 shell 未报告目录。"),
    ("workspace.no_session", "没有会话能启动。"),
    ("workspace.copy", "复制"),
    ("workspace.git", "Git"),
    ("workspace.files", "文件"),
];

/// The message for `key`, in the language the interface is in; `args` fill each `{name}` the message holds.
pub fn message_with(cx: &App, key: &str, args: &[(&str, &str)]) -> String {
    text(cx, key, args).to_string()
}

/// The message for `key`, in the language the interface is in.
pub fn message(cx: &App, key: &str) -> String {
    text(cx, key, &[]).to_string()
}

/// Sets the interface's language in every window.
pub fn apply(language: Language, cx: &mut App) {
    let tag = language.resolved();
    log::info!("language: the interface speaks {tag}");
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

    #[test]
    fn no_message_is_empty_and_none_is_the_same_in_both_languages() {
        for (key, english) in EN {
            assert!(!english.is_empty(), "{key} is empty");
            let chinese = ZH
                .iter()
                .find(|(other, _)| other == key)
                .map(|(_, text)| *text)
                .expect("the key");
            assert!(!chinese.is_empty(), "{key} is empty in Chinese");
        }
    }

    #[test]
    fn every_key_the_code_uses_is_in_the_catalog() {
        // Each key the interface asks for, and the text it stands for.
        const USED: [&str; 6] = [
            "settings.window",
            "settings.category.appearance",
            "settings.terminal.section",
            "settings.language.system",
            "workspace.new_tab",
            "settings.cursor.block",
        ];
        for key in USED {
            assert!(
                EN.iter().any(|(other, _)| *other == key),
                "{key} is not in the catalog"
            );
        }
    }
}
