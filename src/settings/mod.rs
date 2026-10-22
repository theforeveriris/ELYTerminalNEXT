pub mod language;
mod store;
pub mod window;

use std::{
    collections::HashMap,
    env,
    ffi::OsStr,
    ops::RangeInclusive,
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use serde::{Deserialize, Serialize};

use crate::themes;

pub use language::Language;
pub use store::{SettingsStore, load, save};

/// Where a new session starts.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartIn {
    #[default]
    Home,
    Session,
    /// The folder `start_in_folder` names.
    Fixed,
    /// The folder the last new session started in.
    Last,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CursorStyle {
    #[default]
    Block,
    Bar,
    Underline,
    Hollow,
}

/// Which Option keys type Alt on macOS.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OptionAsAlt {
    #[default]
    Off,
    Left,
    Right,
    Both,
}

/// What the right panel shows.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PanelTab {
    #[default]
    Git,
    Files,
}

/// Everything the settings window edits. A field the file leaves out takes its default.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// A program path; empty runs the login shell.
    pub shell: String,
    /// The editor a file opens in; empty opens it with the system.
    pub editor: String,
    pub start_in: StartIn,
    /// The folder a session starts in when `start_in` is `Fixed`.
    pub start_in_folder: String,
    /// The folder the last new session started in, for `start_in`'s `Last`.
    pub last_folder: Option<String>,
    /// Opens the tabs the window last had at start.
    pub restore_session: bool,
    pub theme: String,
    /// The theme while the system is dark; none keeps `theme`.
    pub dark_theme: Option<String>,
    pub font_family: String,
    /// Families tried in order for a character Font family lacks.
    pub font_fallbacks: Vec<String>,
    pub font_size: f32,
    /// The size of the interface's rem, in pixels; the terminal's font has its own size.
    pub ui_font_size: f32,
    pub line_height: f32,
    pub ligatures: bool,
    pub cursor_style: CursorStyle,
    pub cursor_blink: bool,
    pub background_opacity: f32,
    pub background_blur: bool,
    pub padding_x: f32,
    pub padding_y: f32,
    pub scrollback: u32,
    pub option_as_alt: OptionAsAlt,
    pub copy_on_select: bool,
    /// A flash on the pane when a program rings the bell.
    pub bell_visual: bool,
    /// No blinking, orb motion or bell flash.
    pub reduce_motion: bool,
    /// Posts no system notification while on.
    pub do_not_disturb: bool,
    /// Asks before closing a session whose command still runs.
    pub confirm_close: bool,
    /// Ely peeks over the main window's top edge.
    pub ely: bool,
    /// How far in from the pane's left edge she stands, in points, as a drag left her.
    pub ely_left: f32,
    /// A check for a newer release at start and once a day.
    pub check_updates: bool,
    pub show_sessions: bool,
    pub show_panel: bool,
    pub panel: PanelTab,
    /// The sessions and right columns' widths, in pixels, as their borders were dragged.
    pub sessions_width: f32,
    pub panel_width: f32,
    /// The main window's size, unmaximized, in pixels.
    pub window_width: f32,
    pub window_height: f32,
    /// Where the main window's top-left sat, unmaximized; none centers it.
    pub window_x: Option<f32>,
    pub window_y: Option<f32>,
    /// Whether the main window was maximized when it last settled.
    pub window_maximized: bool,
    /// The language of the interface; the system's by default.
    pub language: Language,
    /// The colors of the theme of the user's own.
    pub custom_colors: themes::Colors,
    /// Fields a later version wrote; kept in memory, warned about, and dropped on the next save.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shell: String::new(),
            editor: String::new(),
            start_in: StartIn::Home,
            start_in_folder: String::new(),
            last_folder: None,
            restore_session: true,
            theme: themes::default_id().into(),
            dark_theme: None,
            font_family: "JetBrains Mono".into(),
            font_fallbacks: Vec::new(),
            font_size: 13.0,
            ui_font_size: 16.0,
            line_height: 1.2,
            ligatures: true,
            cursor_style: CursorStyle::Block,
            cursor_blink: true,
            background_opacity: 0.88,
            background_blur: true,
            padding_x: 10.0,
            padding_y: 8.0,
            scrollback: 10_000,
            option_as_alt: OptionAsAlt::Off,
            copy_on_select: false,
            bell_visual: true,
            reduce_motion: false,
            do_not_disturb: false,
            confirm_close: true,
            ely: true,
            ely_left: 140.0,
            check_updates: true,
            show_sessions: true,
            show_panel: true,
            panel: PanelTab::Git,
            sessions_width: 240.0,
            panel_width: 240.0,
            window_width: 1200.0,
            window_height: 760.0,
            window_x: None,
            window_y: None,
            window_maximized: false,
            language: Language::default(),
            custom_colors: themes::Colors::default(),
            extra: HashMap::new(),
        }
    }
}

/// `settings` as a `config.toml`, with a comment over each field saying what it sets.
///
/// The file the settings window writes holds every field and no comments; the one a first run writes
/// should be one the user can read, so each field is written by hand under a line that says what it
/// does and, where it takes a few values, what they are.
pub fn commented(settings: &Settings) -> String {
    let d = |value: f32| format!("{value}");
    let b = |value: bool| if value { "true" } else { "false" };
    let q = |value: &str| format!("{value:?}");
    let mut out = String::new();
    let line = |out: &mut String, text: &str| {
        out.push_str(text);
        out.push('\n');
    };
    let field = |out: &mut String, note: &str, name: &str, value: String| {
        line(out, &format!("# {note}"));
        line(out, &format!("{name} = {value}"));
    };
    // An enum goes as its snake_case name, as serde writes it, so TOML reads it back the same way.
    fn named(value: &impl serde::Serialize) -> String {
        #[derive(serde::Serialize)]
        struct One<'a, T: serde::Serialize> {
            v: &'a T,
        }
        toml::to_string(&One { v: value })
            .expect("a value encodes")
            .trim()
            .strip_prefix("v = ")
            .expect("the one field")
            .to_string()
    }
    line(&mut out, "# ELYTerminal's settings.");
    line(&mut out, "#");
    line(
        &mut out,
        "# Each field says what it sets. Change one and save: the window follows the file,",
    );
    line(
        &mut out,
        "# and the window writes the file back. A file that does not parse leaves the last",
    );
    line(
        &mut out,
        "# good settings in force, and the error names its line.",
    );
    line(
        &mut out,
        "# A value of the form ${HOME} is read from the environment.",
    );
    line(&mut out, "");
    line(&mut out, "# --- the terminal ---");
    field(
        &mut out,
        "The shell a session runs; empty takes the best one this machine was found to have.",
        "shell",
        q(&settings.shell),
    );
    field(
        &mut out,
        "Where a new session starts: \"home\", \"session\", \"fixed\" or \"last\".",
        "start_in",
        named(&settings.start_in),
    );
    field(
        &mut out,
        "The folder a session starts in when start_in is \"fixed\".",
        "start_in_folder",
        q(&settings.start_in_folder),
    );
    field(
        &mut out,
        "A command a file opens in, as in \"code -g {file}:1\"; empty opens it with the system.",
        "editor",
        q(&settings.editor),
    );
    field(
        &mut out,
        "Opens the tabs the window last had, at start.",
        "restore_session",
        b(settings.restore_session).into(),
    );
    field(
        &mut out,
        "Asks before closing a session whose command still runs.",
        "confirm_close",
        b(settings.confirm_close).into(),
    );
    field(
        &mut out,
        "Posts no system notification while on.",
        "do_not_disturb",
        b(settings.do_not_disturb).into(),
    );
    field(
        &mut out,
        "Rows of history a session keeps, 0 to 100000.",
        "scrollback",
        settings.scrollback.to_string(),
    );
    field(
        &mut out,
        "Copies what a drag selects, without a key.",
        "copy_on_select",
        b(settings.copy_on_select).into(),
    );
    field(
        &mut out,
        "A flash on the pane when a program rings the bell.",
        "bell_visual",
        b(settings.bell_visual).into(),
    );
    field(
        &mut out,
        "No blinking, no moving status marks, no bell flash.",
        "reduce_motion",
        b(settings.reduce_motion).into(),
    );
    line(&mut out, "");
    line(&mut out, "# --- how it looks ---");
    field(
        &mut out,
        "The theme; a built-in one's id, or \"custom\" for the colors below.",
        "theme",
        q(&settings.theme),
    );
    field(
        &mut out,
        "The theme while the system is dark; none keeps the one above.",
        "dark_theme",
        q(settings.dark_theme.as_deref().unwrap_or("")),
    );
    field(
        &mut out,
        "The terminal's font; the interface has a size of its own.",
        "font_family",
        q(&settings.font_family),
    );
    field(
        &mut out,
        "Families tried in order for a character the font above lacks.",
        "font_fallbacks",
        format!(
            "[{}]",
            settings
                .font_fallbacks
                .iter()
                .map(|family| q(family))
                .collect::<Vec<_>>()
                .join(", ")
        ),
    );
    field(
        &mut out,
        "The terminal's font size, 6 to 72.",
        "font_size",
        d(settings.font_size),
    );
    field(
        &mut out,
        "The interface's own size in pixels, 10 to 24; the terminal's is separate.",
        "ui_font_size",
        d(settings.ui_font_size),
    );
    field(
        &mut out,
        "A line's height as a multiple of its font's, 1.0 to 2.0.",
        "line_height",
        d(settings.line_height),
    );
    field(
        &mut out,
        "Draws ligatures the font has.",
        "ligatures",
        b(settings.ligatures).into(),
    );
    field(
        &mut out,
        "The cursor's shape: \"block\", \"bar\", \"underline\" or \"hollow\".",
        "cursor_style",
        named(&settings.cursor_style),
    );
    field(
        &mut out,
        "The cursor blinks.",
        "cursor_blink",
        b(settings.cursor_blink).into(),
    );
    field(
        &mut out,
        "How much the window's background shows through, 0.3 to 1.0.",
        "background_opacity",
        d(settings.background_opacity),
    );
    field(
        &mut out,
        "Blurs what is behind the window.",
        "background_blur",
        b(settings.background_blur).into(),
    );
    field(
        &mut out,
        "The terminal's padding in pixels, 0 to 64.",
        "padding_x",
        d(settings.padding_x),
    );
    field(
        &mut out,
        "The same, above and below.",
        "padding_y",
        d(settings.padding_y),
    );
    line(&mut out, "");
    line(&mut out, "# --- the window ---");
    field(
        &mut out,
        "Which the right column shows: \"git\" or \"files\".",
        "panel",
        named(&settings.panel),
    );
    field(
        &mut out,
        "Shows the sessions column.",
        "show_sessions",
        b(settings.show_sessions).into(),
    );
    field(
        &mut out,
        "Shows the right column.",
        "show_panel",
        b(settings.show_panel).into(),
    );
    field(
        &mut out,
        "The language of the interface: \"system\", \"en-US\" or \"zh-CN\".",
        "language",
        named(&settings.language),
    );
    field(
        &mut out,
        "Ely peeks over the window's top edge.",
        "ely",
        b(settings.ely).into(),
    );
    field(
        &mut out,
        "How far in from the pane's left edge she stands, in points.",
        "ely_left",
        d(settings.ely_left),
    );
    field(
        &mut out,
        "Checks for a newer release at start and once a day.",
        "check_updates",
        b(settings.check_updates).into(),
    );
    out
}

pub const FONT_SIZE: RangeInclusive<f32> = 6.0..=72.0;
pub const UI_FONT_SIZE: RangeInclusive<f32> = 10.0..=24.0;
pub const LINE_HEIGHT: RangeInclusive<f32> = 1.0..=2.0;
pub const OPACITY: RangeInclusive<f32> = 0.3..=1.0;
pub const PADDING: RangeInclusive<f32> = 0.0..=64.0;
pub const SCROLLBACK: RangeInclusive<u32> = 0..=100_000;
pub const COLUMN: RangeInclusive<f32> = 160.0..=960.0;
pub const WINDOW: RangeInclusive<f32> = 100.0..=16384.0;
pub const WINDOW_POS: RangeInclusive<f32> = -32768.0..=32768.0;
pub const ELY_LEFT: RangeInclusive<f32> = 0.0..=16384.0;

impl Settings {
    /// Where a new session starts: its strategy's folder when it exists, else the home folder.
    pub fn start_folder(&self, home: &Path, session: Option<&Path>) -> PathBuf {
        let named = match self.start_in {
            StartIn::Home => home.to_path_buf(),
            StartIn::Session => session.unwrap_or(home).to_path_buf(),
            StartIn::Fixed => PathBuf::from(&self.start_in_folder),
            StartIn::Last => self
                .last_folder
                .as_ref()
                .map(PathBuf::from)
                .unwrap_or_default(),
        };
        if named.is_dir() {
            named
        } else {
            if !named.as_os_str().is_empty() {
                log::warn!(
                    "settings: {} is no folder; the session starts in {}",
                    named.display(),
                    home.display()
                );
            }
            home.to_path_buf()
        }
    }

    /// Every value inside its range, every theme built in; the error names the field.
    pub fn validate(&self) -> anyhow::Result<()> {
        within("font_size", self.font_size, &FONT_SIZE)?;
        within("ui_font_size", self.ui_font_size, &UI_FONT_SIZE)?;
        within("line_height", self.line_height, &LINE_HEIGHT)?;
        within("background_opacity", self.background_opacity, &OPACITY)?;
        within("padding_x", self.padding_x, &PADDING)?;
        within("padding_y", self.padding_y, &PADDING)?;
        within("scrollback", self.scrollback, &SCROLLBACK)?;
        within("sessions_width", self.sessions_width, &COLUMN)?;
        within("panel_width", self.panel_width, &COLUMN)?;
        within("window_width", self.window_width, &WINDOW)?;
        within("window_height", self.window_height, &WINDOW)?;
        if let Some(x) = self.window_x {
            within("window_x", x, &WINDOW_POS)?;
        }
        if let Some(y) = self.window_y {
            within("window_y", y, &WINDOW_POS)?;
        }
        within("ely_left", self.ely_left, &ELY_LEFT)?;
        for (field, id) in [
            ("theme", Some(&self.theme)),
            ("dark_theme", self.dark_theme.as_ref()),
        ] {
            if let Some(id) = id {
                anyhow::ensure!(themes::known(id), "{field} is {id:?}, no built-in theme");
            }
        }
        anyhow::ensure!(!self.font_family.is_empty(), "font_family is empty");
        anyhow::ensure!(
            !self.font_fallbacks.iter().any(String::is_empty),
            "font_fallbacks holds an empty family"
        );
        if !self.shell.is_empty() {
            program_path(&self.shell)?;
        }
        Ok(())
    }
}

/// The program `shell` names, as a session starts it: a file at a full path, or the first in `PATH`'s full folders.
pub fn program_path(shell: &str) -> anyhow::Result<String> {
    let folders = env::var_os("PATH").unwrap_or_default();
    find(shell, &folders, cfg!(windows))
}

/// `shell`'s program, or the first one named so in `folders`, listed as `PATH` lists them; with `exe`, as on Windows, a name without an extension means name.exe.
fn find(shell: &str, folders: &OsStr, exe: bool) -> anyhow::Result<String> {
    let path = Path::new(shell);
    // A relative path, or a relative folder, would hang on each session's own folder.
    let found = if path.is_absolute() {
        program(path).then(|| path.to_path_buf())
    } else if path.components().count() == 1 {
        let name = if exe && !shell.contains('.') {
            format!("{shell}.exe")
        } else {
            shell.to_string()
        };
        env::split_paths(folders)
            .filter(|folder| folder.is_absolute())
            .map(|folder| folder.join(&name))
            .find(|file| program(file))
    } else {
        None
    };
    let found = found.with_context(|| format!("shell is {shell:?}, which names no program"))?;
    let text = found.to_str().with_context(|| {
        format!(
            "shell is {shell:?}, found at {}, which is not UTF-8",
            found.display()
        )
    })?;
    Ok(text.to_string())
}

/// Whether `path` is a file the running user may run, as the system answers, ACLs too.
#[cfg(unix)]
fn program(path: &Path) -> bool {
    use std::{ffi::CString, os::unix::ffi::OsStrExt};
    let Ok(name) = CString::new(path.as_os_str().as_bytes()) else {
        return false;
    };
    path.is_file() && unsafe { libc::access(name.as_ptr(), libc::X_OK) } == 0
}

#[cfg(not(unix))]
fn program(path: &Path) -> bool {
    path.is_file()
}

fn within<T: PartialOrd + std::fmt::Display>(
    field: &str,
    value: T,
    range: &RangeInclusive<T>,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        range.contains(&value),
        "{field} is {value}, outside {} to {}",
        range.start(),
        range.end()
    );
    Ok(())
}

#[cfg(test)]
mod tests;
