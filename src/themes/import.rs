//! Reading theme files other terminals write: iTerm2, Windows Terminal and Ghostty.

use std::{fs, path::Path};

use anyhow::Context as _;
use gpui::{Hsla, Rgba};

use super::custom::Colors;

/// The colors of an iTerm2 `.itermcolors` plist.
pub fn iterm(text: &str) -> anyhow::Result<Colors> {
    let mut colors = Colors::default();
    let mut seen = 0;
    for (name, rgb) in entries(text) {
        let color: Hsla = Rgba {
            r: rgb[0],
            g: rgb[1],
            b: rgb[2],
            a: 1.0,
        }
        .into();
        match name.as_str() {
            "Background Color" => colors.background = color,
            "Foreground Color" => colors.foreground = color,
            "Cursor Color" => colors.cursor = color,
            "Cursor Text Color" => colors.cursor_text = color,
            "Selection Color" => colors.selection_background = color,
            "Selected Text Color" => colors.selection_foreground = color,
            "Ansi 0 Color" => colors.black = color,
            "Ansi 1 Color" => colors.red = color,
            "Ansi 2 Color" => colors.green = color,
            "Ansi 3 Color" => colors.yellow = color,
            "Ansi 4 Color" => colors.blue = color,
            "Ansi 5 Color" => colors.magenta = color,
            "Ansi 6 Color" => colors.cyan = color,
            "Ansi 7 Color" => colors.white = color,
            "Ansi 8 Color" => colors.bright_black = color,
            "Ansi 9 Color" => colors.bright_red = color,
            "Ansi 10 Color" => colors.bright_green = color,
            "Ansi 11 Color" => colors.bright_yellow = color,
            "Ansi 12 Color" => colors.bright_blue = color,
            "Ansi 13 Color" => colors.bright_magenta = color,
            "Ansi 14 Color" => colors.bright_cyan = color,
            "Ansi 15 Color" => colors.bright_white = color,
            _ => continue,
        }
        seen += 1;
    }
    anyhow::ensure!(seen > 0, "the plist holds no terminal colors");
    finish(colors)
}

/// The colors of a Windows Terminal scheme, as JSON.
pub fn windows_terminal(text: &str) -> anyhow::Result<Colors> {
    let value: serde_json::Value = serde_json::from_str(text).context("the scheme is not JSON")?;
    let mut colors = Colors::default();
    let mut seen = 0;
    let mut take = |key: &str, role: &mut Hsla| {
        let Some(text) = value.get(key).and_then(|value| value.as_str()) else {
            return;
        };
        match super::custom::parse_hex(text) {
            Ok(color) => {
                *role = color;
                seen += 1;
            }
            Err(error) => log::warn!("themes: {key} is {text:?}: {error:#}"),
        }
    };
    take("background", &mut colors.background);
    take("foreground", &mut colors.foreground);
    take("cursorColor", &mut colors.cursor);
    take("selectionBackground", &mut colors.selection_background);
    take("black", &mut colors.black);
    take("red", &mut colors.red);
    take("green", &mut colors.green);
    take("yellow", &mut colors.yellow);
    take("blue", &mut colors.blue);
    take("purple", &mut colors.magenta);
    take("cyan", &mut colors.cyan);
    take("white", &mut colors.white);
    take("brightBlack", &mut colors.bright_black);
    take("brightRed", &mut colors.bright_red);
    take("brightGreen", &mut colors.bright_green);
    take("brightYellow", &mut colors.bright_yellow);
    take("brightBlue", &mut colors.bright_blue);
    take("brightPurple", &mut colors.bright_magenta);
    take("brightCyan", &mut colors.bright_cyan);
    take("brightWhite", &mut colors.bright_white);
    anyhow::ensure!(seen > 0, "the scheme holds no colors");
    finish(colors)
}

/// The colors of a Ghostty theme file.
pub fn ghostty(text: &str) -> anyhow::Result<Colors> {
    let theme = super::parse("imported", "Imported", text)?;
    let mut colors = Colors {
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
    };
    colors.accent = colors.bright_blue;
    Ok(colors)
}

/// The colors in the theme file at `path`, by what its text is.
pub fn read(path: &Path) -> anyhow::Result<Colors> {
    let text = fs::read_to_string(path)
        .with_context(|| format!("themes: cannot read {}", path.display()))?;
    let trimmed = text.trim_start();
    if trimmed.starts_with('{') {
        windows_terminal(&text)
    } else if text.contains("<plist") || text.contains("<dict>") {
        iterm(&text)
    } else {
        ghostty(&text)
    }
}

/// The colors a plist and the cursor's text need when the file names none.
fn finish(mut colors: Colors) -> anyhow::Result<Colors> {
    colors.accent = colors.bright_blue;
    Ok(colors)
}

/// Each named color in an iTerm2 plist, with its red, green and blue components.
fn entries(text: &str) -> Vec<(String, [f32; 3])> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(at) = rest.find("<key>") {
        let after = &rest[at + 5..];
        let Some(end) = after.find("</key>") else {
            break;
        };
        let name = after[..end].trim().to_string();
        let after_key = &after[end + 6..];
        let Some(dict_at) = after_key.find("<dict>") else {
            rest = after_key;
            continue;
        };
        let dict = &after_key[dict_at + 6..];
        let Some(dict_end) = dict.find("</dict>") else {
            break;
        };
        let block = &dict[..dict_end];
        if name.ends_with("Color")
            && let (Some(red), Some(green), Some(blue)) = (
                component(block, "Red Component"),
                component(block, "Green Component"),
                component(block, "Blue Component"),
            )
        {
            found.push((name, [red, green, blue]));
        }
        rest = &dict[dict_end + 7..];
    }
    found
}

/// A color component a plist block names, as a real or an integer.
fn component(block: &str, name: &str) -> Option<f32> {
    let at = block.find(&format!("<key>{name}</key>"))?;
    let after = &block[at..];
    for tag in ["<real>", "<integer>"] {
        if let Some(start) = after.find(tag) {
            let end_tag = tag.replace('<', "</");
            let from = start + tag.len();
            let end = after[from..].find(&end_tag)?;
            return after[from..from + end].trim().parse().ok();
        }
    }
    None
}

#[cfg(test)]
mod tests;
