//! A theme of the user's own: the colors of each role, which the settings edit.

use gpui::{Hsla, Rgba};
use serde::{Deserialize, Serialize};

/// The colors a theme holds, each by its name.
pub const ROLES: [&str; 23] = [
    "background",
    "foreground",
    "cursor",
    "cursor_text",
    "selection_background",
    "selection_foreground",
    "black",
    "red",
    "green",
    "yellow",
    "blue",
    "magenta",
    "cyan",
    "white",
    "bright_black",
    "bright_red",
    "bright_green",
    "bright_yellow",
    "bright_blue",
    "bright_magenta",
    "bright_cyan",
    "bright_white",
    "accent",
];

/// The roles by the part of the window they color: the group's message key and its roles.
pub const GROUPS: [(&str, &[&str]); 4] = [
    ("window", &["background", "foreground", "accent"]),
    ("cursor", &["cursor", "cursor_text"]),
    (
        "selection",
        &["selection_background", "selection_foreground"],
    ),
    (
        "ansi",
        &[
            "black",
            "red",
            "green",
            "yellow",
            "blue",
            "magenta",
            "cyan",
            "white",
            "bright_black",
            "bright_red",
            "bright_green",
            "bright_yellow",
            "bright_blue",
            "bright_magenta",
            "bright_cyan",
            "bright_white",
        ],
    ),
];

/// The colors of a theme, in the order the roles name them.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Colors {
    pub background: Hsla,
    pub foreground: Hsla,
    pub cursor: Hsla,
    pub cursor_text: Hsla,
    pub selection_background: Hsla,
    pub selection_foreground: Hsla,
    pub black: Hsla,
    pub red: Hsla,
    pub green: Hsla,
    pub yellow: Hsla,
    pub blue: Hsla,
    pub magenta: Hsla,
    pub cyan: Hsla,
    pub white: Hsla,
    pub bright_black: Hsla,
    pub bright_red: Hsla,
    pub bright_green: Hsla,
    pub bright_yellow: Hsla,
    pub bright_blue: Hsla,
    pub bright_magenta: Hsla,
    pub bright_cyan: Hsla,
    pub bright_white: Hsla,
    /// The color the chrome leans on, as Zed's accent.
    pub accent: Hsla,
}

impl Default for Colors {
    fn default() -> Self {
        Self {
            background: hex(0x1e1e2e),
            foreground: hex(0xcdd6f4),
            cursor: hex(0xf5e0dc),
            cursor_text: hex(0x1e1e2e),
            selection_background: hex(0x585b70),
            selection_foreground: hex(0xcdd6f4),
            black: hex(0x45475a),
            red: hex(0xf38ba8),
            green: hex(0xa6e3a1),
            yellow: hex(0xf9e2af),
            blue: hex(0x89b4fa),
            magenta: hex(0xf5c2e7),
            cyan: hex(0x94e2d5),
            white: hex(0xbac2de),
            bright_black: hex(0x585b70),
            bright_red: hex(0xf38ba8),
            bright_green: hex(0xa6e3a1),
            bright_yellow: hex(0xf9e2af),
            bright_blue: hex(0x89b4fa),
            bright_magenta: hex(0xf5c2e7),
            bright_cyan: hex(0x94e2d5),
            bright_white: hex(0xa6adc8),
            accent: hex(0x89b4fa),
        }
    }
}

impl Colors {
    /// The color of each role, in the order the roles name them.
    pub fn by_role(&self) -> [&Hsla; 23] {
        [
            &self.background,
            &self.foreground,
            &self.cursor,
            &self.cursor_text,
            &self.selection_background,
            &self.selection_foreground,
            &self.black,
            &self.red,
            &self.green,
            &self.yellow,
            &self.blue,
            &self.magenta,
            &self.cyan,
            &self.white,
            &self.bright_black,
            &self.bright_red,
            &self.bright_green,
            &self.bright_yellow,
            &self.bright_blue,
            &self.bright_magenta,
            &self.bright_cyan,
            &self.bright_white,
            &self.accent,
        ]
    }

    /// The color of the role, for the picker to show.
    pub fn get(&self, role: &str) -> Option<Hsla> {
        Some(match role {
            "background" => self.background,
            "foreground" => self.foreground,
            "cursor" => self.cursor,
            "cursor_text" => self.cursor_text,
            "selection_background" => self.selection_background,
            "selection_foreground" => self.selection_foreground,
            "black" => self.black,
            "red" => self.red,
            "green" => self.green,
            "yellow" => self.yellow,
            "blue" => self.blue,
            "magenta" => self.magenta,
            "cyan" => self.cyan,
            "white" => self.white,
            "bright_black" => self.bright_black,
            "bright_red" => self.bright_red,
            "bright_green" => self.bright_green,
            "bright_yellow" => self.bright_yellow,
            "bright_blue" => self.bright_blue,
            "bright_magenta" => self.bright_magenta,
            "bright_cyan" => self.bright_cyan,
            "bright_white" => self.bright_white,
            "accent" => self.accent,
            _ => return None,
        })
    }

    /// The color of the role, set by the picker.
    pub fn set(&mut self, role: &str, color: Hsla) -> bool {
        match role {
            "background" => self.background = color,
            "foreground" => self.foreground = color,
            "cursor" => self.cursor = color,
            "cursor_text" => self.cursor_text = color,
            "selection_background" => self.selection_background = color,
            "selection_foreground" => self.selection_foreground = color,
            "black" => self.black = color,
            "red" => self.red = color,
            "green" => self.green = color,
            "yellow" => self.yellow = color,
            "blue" => self.blue = color,
            "magenta" => self.magenta = color,
            "cyan" => self.cyan = color,
            "white" => self.white = color,
            "bright_black" => self.bright_black = color,
            "bright_red" => self.bright_red = color,
            "bright_green" => self.bright_green = color,
            "bright_yellow" => self.bright_yellow = color,
            "bright_blue" => self.bright_blue = color,
            "bright_magenta" => self.bright_magenta = color,
            "bright_cyan" => self.bright_cyan = color,
            "bright_white" => self.bright_white = color,
            "accent" => self.accent = color,
            _ => return false,
        }
        true
    }

    /// The 16 ANSI colors, in the order a terminal numbers them.
    pub fn ansi(&self) -> [Hsla; 16] {
        [
            self.black,
            self.red,
            self.green,
            self.yellow,
            self.blue,
            self.magenta,
            self.cyan,
            self.white,
            self.bright_black,
            self.bright_red,
            self.bright_green,
            self.bright_yellow,
            self.bright_blue,
            self.bright_magenta,
            self.bright_cyan,
            self.bright_white,
        ]
    }

    /// `theme`'s colors as the user's own, so an edit starts from what is on the screen rather than from a fixed default.
    pub fn from_theme(theme: &super::TerminalTheme) -> Self {
        let ansi = theme.ansi;
        Self {
            background: theme.background,
            foreground: theme.foreground,
            cursor: theme.cursor,
            cursor_text: theme.cursor_text,
            selection_background: theme.selection_background,
            selection_foreground: theme.selection_foreground.unwrap_or(theme.foreground),
            black: ansi[0],
            red: ansi[1],
            green: ansi[2],
            yellow: ansi[3],
            blue: ansi[4],
            magenta: ansi[5],
            cyan: ansi[6],
            white: ansi[7],
            bright_black: ansi[8],
            bright_red: ansi[9],
            bright_green: ansi[10],
            bright_yellow: ansi[11],
            bright_blue: ansi[12],
            bright_magenta: ansi[13],
            bright_cyan: ansi[14],
            bright_white: ansi[15],
            // The chrome's accent follows the theme's blue, as the built-in palettes do.
            accent: ansi[4],
        }
    }

    /// The colors as `#rrggbb`, which a theme file holds.
    pub fn to_hex(&self) -> Vec<(String, String)> {
        ROLES
            .iter()
            .zip(self.by_role())
            .map(|(role, color)| ((*role).to_string(), hex_of(*color)))
            .collect()
    }

    /// The colors from `#rrggbb` pairs; a bad one names the role that failed.
    pub fn from_hex(pairs: &[(String, String)]) -> anyhow::Result<Self> {
        let mut colors = Self::default();
        for (role, value) in pairs {
            let color =
                parse_hex(value).with_context(|| format!("the color of {role}, {value:?}"))?;
            anyhow::ensure!(colors.set(role, color), "no role {role:?} in a theme");
        }
        Ok(colors)
    }
}

/// A color as `#rrggbb`.
pub fn hex_of(color: Hsla) -> String {
    let rgba: Rgba = color.into();
    let channel = |value: f32| (value.clamp(0.0, 1.0) * 255.0).round() as u8;
    format!(
        "#{:02x}{:02x}{:02x}",
        channel(rgba.r),
        channel(rgba.g),
        channel(rgba.b)
    )
}

/// A color from `#rrggbb`.
pub fn parse_hex(text: &str) -> anyhow::Result<Hsla> {
    let digits = text
        .trim()
        .strip_prefix('#')
        .ok_or_else(|| anyhow::anyhow!("`{text}` is not #rrggbb"))?;
    anyhow::ensure!(digits.len() == 6, "`{text}` is not #rrggbb");
    let value = u32::from_str_radix(digits, 16)?;
    let channel = |shift: u32| ((value >> shift) & 0xff) as f32 / 255.0;
    Ok(Rgba {
        r: channel(16),
        g: channel(8),
        b: channel(0),
        a: 1.0,
    }
    .into())
}

/// A color from a number, as a role's default.
fn hex(value: u32) -> Hsla {
    let channel = |shift: u32| ((value >> shift) & 0xff) as f32 / 255.0;
    Rgba {
        r: channel(16),
        g: channel(8),
        b: channel(0),
        a: 1.0,
    }
    .into()
}

use anyhow::Context as _;

/// A theme the user named and kept: the name it shows and the colors it holds.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct SavedTheme {
    /// The id the settings name it by, `saved-…`, which no other saved theme shares.
    pub id: String,
    pub name: String,
    pub colors: Colors,
}

impl SavedTheme {
    /// `name` over `colors`, with an id no theme in `taken` holds; a repeated name takes the next number.
    pub fn new(name: &str, colors: Colors, taken: &[SavedTheme]) -> Self {
        let base = slug(name);
        let mut id = format!("saved-{base}");
        let mut count = 2;
        while taken.iter().any(|theme| theme.id == id) {
            id = format!("saved-{base}-{count}");
            count += 1;
        }
        Self {
            id,
            name: name.trim().to_string(),
            colors,
        }
    }
}

/// `name` as an id: letters and digits kept, anything else joining them with a dash.
fn slug(name: &str) -> String {
    let mut out = String::new();
    for c in name.chars() {
        if c.is_alphanumeric() {
            out.extend(c.to_lowercase());
        } else if !out.ends_with('-') {
            out.push('-');
        }
    }
    let slug = out.trim_matches('-');
    match slug.is_empty() {
        true => "theme".to_string(),
        false => slug.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_saved_theme_takes_its_name_and_never_another_s_id() {
        let colors = Colors::default();
        let first = SavedTheme::new("My Theme", colors.clone(), &[]);
        assert_eq!(first.id, "saved-my-theme");
        assert_eq!(first.name, "My Theme");
        let again = SavedTheme::new("My Theme", colors.clone(), std::slice::from_ref(&first));
        assert_eq!(
            again.id, "saved-my-theme-2",
            "a repeated name takes the next number"
        );
        let third = SavedTheme::new("My Theme", colors, &[first.clone(), again.clone()]);
        assert_eq!(third.id, "saved-my-theme-3");
        let unnamed = SavedTheme::new("  ", Colors::default(), &[]);
        assert_eq!(unnamed.id, "saved-theme", "an empty name still has an id");
        assert_eq!(
            SavedTheme::new("主题 A", Colors::default(), &[]).id,
            "saved-主题-a"
        );
    }

    #[test]
    fn the_user_s_colors_come_from_a_theme_to_edit() {
        let theme = super::super::find("catppuccin-mocha").expect("a built-in theme");
        let colors = Colors::from_theme(theme);
        assert_eq!(colors.background, theme.background);
        assert_eq!(colors.foreground, theme.foreground);
        assert_eq!(colors.blue, theme.ansi[4]);
        assert_eq!(colors.bright_white, theme.ansi[15]);
        assert_eq!(colors.accent, theme.ansi[4], "the accent follows the blue");
        assert_eq!(
            colors.ansi(),
            theme.ansi,
            "the sixteen colors are kept as they are"
        );
    }

    #[test]
    fn every_role_has_a_color_and_a_name() {
        let colors = Colors::default();
        for role in ROLES {
            assert!(colors.get(role).is_some(), "{role} has no color");
        }
        assert_eq!(colors.by_role().len(), ROLES.len());
    }

    #[test]
    fn a_color_set_by_its_role_reads_back() {
        let mut colors = Colors::default();
        let red = parse_hex("#ff0000").unwrap();
        assert!(colors.set("background", red));
        assert_eq!(colors.get("background"), Some(red));
        assert!(!colors.set("no-such-role", red), "no such role");
    }

    #[test]
    fn the_ansi_colors_come_in_their_order() {
        let mut colors = Colors::default();
        for (at, role) in ROLES.iter().enumerate().take(16) {
            let color = parse_hex(&format!("#{:06x}", at)).unwrap();
            colors.set(role, color);
        }
        let ansi = colors.ansi();
        assert_eq!(ansi[0], colors.black);
        assert_eq!(ansi[9], colors.bright_red);
        assert_eq!(ansi[15], colors.bright_white);
    }

    #[test]
    fn the_colors_survive_a_round_trip_through_hex() {
        let colors = Colors::default();
        let pairs = colors.to_hex();
        assert_eq!(pairs.len(), ROLES.len());
        assert_eq!(Colors::from_hex(&pairs).unwrap(), colors);
    }

    #[test]
    fn a_bad_color_names_its_role() {
        let error = Colors::from_hex(&[("background".into(), "nope".into())]).unwrap_err();
        assert!(
            format!("{error:#}").contains("background"),
            "the error names the role: {error:#}"
        );
    }
}
