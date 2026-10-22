//! The `config.toml` a first run writes: the defaults, each field under a comment saying what it sets.

use super::Settings;

/// `settings` as a `config.toml`, with a comment over each field saying what it sets. The file the settings window writes holds every field and no comments; the one a first run writes should be one the user can read, so each field goes down by hand under a line that says what it does and, where it takes a few values, what they are.
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
    match &settings.dark_theme {
        Some(id) => field(
            &mut out,
            "The theme while the system is dark; say nothing to keep the one above.",
            "dark_theme",
            q(id),
        ),
        None => line(
            &mut out,
            "# dark_theme sets another theme while the system is dark; left out, it keeps the one above.",
        ),
    }
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

#[cfg(test)]
mod tests;
