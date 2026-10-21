//! Writing a theme in Ghostty's `key = value` format.

use crate::themes::{
    TerminalTheme,
    custom::{Colors, hex_of},
};

/// The colors `theme` wears, as the custom theme's fields.
pub fn colors(theme: &TerminalTheme) -> Colors {
    Colors {
        background: theme.background,
        foreground: theme.foreground,
        cursor: theme.cursor,
        cursor_text: theme.cursor_text,
        selection_background: theme.selection_background,
        selection_foreground: theme.selection_foreground.unwrap_or(theme.foreground),
        black: theme.ansi[0],
        red: theme.ansi[1],
        green: theme.ansi[2],
        yellow: theme.ansi[3],
        blue: theme.ansi[4],
        magenta: theme.ansi[5],
        cyan: theme.ansi[6],
        white: theme.ansi[7],
        bright_black: theme.ansi[8],
        bright_red: theme.ansi[9],
        bright_green: theme.ansi[10],
        bright_yellow: theme.ansi[11],
        bright_blue: theme.ansi[12],
        bright_magenta: theme.ansi[13],
        bright_cyan: theme.ansi[14],
        bright_white: theme.ansi[15],
        accent: theme.ansi[12],
    }
}

/// `colors` as a Ghostty theme file, one palette line for each of the sixteen.
pub fn ghostty(colors: &Colors) -> String {
    let ansi = colors.by_role();
    let mut out = String::new();
    for (index, color) in ansi[6..22].iter().enumerate() {
        out.push_str(&format!("palette = {index}={}\n", hex_of(**color)));
    }
    out.push_str(&format!("background = {}\n", hex_of(colors.background)));
    out.push_str(&format!("foreground = {}\n", hex_of(colors.foreground)));
    out.push_str(&format!("cursor-color = {}\n", hex_of(colors.cursor)));
    out.push_str(&format!("cursor-text = {}\n", hex_of(colors.cursor_text)));
    out.push_str(&format!(
        "selection-background = {}\n",
        hex_of(colors.selection_background)
    ));
    out.push_str(&format!(
        "selection-foreground = {}\n",
        hex_of(colors.selection_foreground)
    ));
    out
}

#[cfg(test)]
mod tests;
