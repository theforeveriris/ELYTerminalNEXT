use std::{env, fs, path::PathBuf};

use super::*;
use crate::themes::custom::hex_of;

const PLIST: &str = r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Ansi 1 Color</key>
    <dict>
        <key>Blue Component</key>
        <real>0.0</real>
        <key>Green Component</key>
        <real>0.0</real>
        <key>Red Component</key>
        <real>1.0</real>
    </dict>
    <key>Background Color</key>
    <dict>
        <key>Blue Component</key>
        <real>0.0</real>
        <key>Green Component</key>
        <real>0.0</real>
        <key>Red Component</key>
        <real>0.0</real>
    </dict>
    <key>Foreground Color</key>
    <dict>
        <key>Blue Component</key>
        <integer>1</integer>
        <key>Green Component</key>
        <integer>1</integer>
        <key>Red Component</key>
        <integer>1</integer>
    </dict>
    <key>Ansi 12 Color</key>
    <dict>
        <key>Blue Component</key>
        <real>1.0</real>
        <key>Green Component</key>
        <real>0.0</real>
        <key>Red Component</key>
        <real>0.0</real>
    </dict>
</dict>
</plist>
"#;

#[test]
fn an_iterm2_plist_reads_its_colors() {
    let colors = iterm(PLIST).expect("the plist reads");
    assert_eq!(hex_of(colors.background), "#000000");
    assert_eq!(hex_of(colors.foreground), "#ffffff");
    assert_eq!(hex_of(colors.red), "#ff0000");
    assert_eq!(hex_of(colors.bright_blue), "#0000ff");
    assert_eq!(colors.accent, colors.bright_blue);
    assert_eq!(
        colors.cursor,
        Colors::default().cursor,
        "absent stays default"
    );
}

#[test]
fn a_plist_without_any_color_is_an_error() {
    let error = iterm("<plist><dict></dict></plist>").expect_err("no colors");
    assert!(error.to_string().contains("no terminal colors"), "{error}");
}

#[test]
fn a_windows_terminal_scheme_reads_its_colors() {
    let text = r##"{
        "name": "Work",
        "background": "#101010",
        "foreground": "#eeeeee",
        "red": "#ff0000",
        "purple": "#ff00ff",
        "brightPurple": "#aa00aa",
        "brightBlue": "#0000ff"
    }"##;
    let colors = windows_terminal(text).expect("the scheme reads");
    assert_eq!(hex_of(colors.background), "#101010");
    assert_eq!(hex_of(colors.foreground), "#eeeeee");
    assert_eq!(hex_of(colors.red), "#ff0000");
    assert_eq!(hex_of(colors.magenta), "#ff00ff");
    assert_eq!(hex_of(colors.bright_magenta), "#aa00aa");
    let error = windows_terminal("{\"name\":\"empty\"}").expect_err("no colors");
    assert!(error.to_string().contains("no colors"), "{error}");
    assert!(windows_terminal("not json").is_err());
}

#[test]
fn a_ghostty_theme_reads_through_its_own_parser() {
    let text = include_str!("../../../assets/themes/catppuccin-mocha");
    let colors = ghostty(text).expect("the theme reads");
    assert_eq!(hex_of(colors.background), "#1e1e2e");
    assert_eq!(hex_of(colors.red), "#f38ba8");
    assert_eq!(hex_of(colors.bright_blue), "#aeccfc");
    assert_eq!(colors.accent, colors.bright_blue);
}

#[test]
fn a_file_is_read_by_what_it_holds() {
    let root = env::temp_dir().join(format!("elyterminal-theme-{}", std::process::id()));
    fs::create_dir_all(&root).expect("a test folder");
    let ghostty_file = root.join("theme");
    fs::write(
        &ghostty_file,
        include_str!("../../../assets/themes/catppuccin-mocha"),
    )
    .expect("a Ghostty theme");
    assert_eq!(
        hex_of(read(&ghostty_file).expect("the file reads").background),
        "#1e1e2e"
    );
    let plist_file = root.join("theme.itermcolors");
    fs::write(&plist_file, PLIST).expect("an iTerm2 theme");
    assert_eq!(
        hex_of(read(&plist_file).expect("the plist reads").red),
        "#ff0000"
    );
    let json_file: PathBuf = root.join("theme.json");
    fs::write(
        &json_file,
        r##"{"background":"#101010","foreground":"#ffffff"}"##,
    )
    .expect("a scheme");
    assert_eq!(
        hex_of(read(&json_file).expect("the scheme reads").background),
        "#101010"
    );
    fs::remove_dir_all(&root).expect("cleanup");
}
