use std::fs;

use crate::settings::{Settings, commented, store};

#[test]
fn a_first_run_s_config_file_reads_back_as_the_defaults() {
    let root = std::env::temp_dir().join(format!("elyterminal-seed-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("a test folder");
    let path = root.join("config.toml");
    let settings = Settings::default();
    let text = commented(&settings);
    fs::write(&path, &text).expect("written");
    // Every field named in the file is one this version knows, and its value reads back.
    let read: Settings = toml::from_str(&text).expect("the seeded file parses");
    assert_eq!(read.shell, settings.shell);
    assert_eq!(read.theme, settings.theme);
    assert_eq!(read.font_family, settings.font_family);
    assert_eq!(read.font_size, settings.font_size);
    assert_eq!(read.scrollback, settings.scrollback);
    assert_eq!(read.start_in, settings.start_in);
    assert_eq!(read.panel, settings.panel);
    assert_eq!(read.cursor_style, settings.cursor_style);
    assert_eq!(read.language, settings.language);
    assert!(
        text.contains("# "),
        "each field carries a comment: {}",
        &text[..200]
    );
    // An existing file is the user's, and seeding leaves it alone.
    let kept = fs::read_to_string(&path).expect("read back");
    store::seeded(&path).expect("seeding an existing file is no error");
    assert_eq!(fs::read_to_string(&path).expect("read back"), kept);
    let _ = fs::remove_dir_all(&root);
}

#[test]
fn a_file_seeded_where_there_was_none_holds_the_defaults() {
    let root = std::env::temp_dir().join(format!("elyterminal-new-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let path = root.join("ELYTerminal").join("config.toml");
    store::seeded(&path).expect("seeded");
    let text = fs::read_to_string(&path).expect("written");
    let read: Settings = toml::from_str(&text).expect("it parses");
    assert_eq!(read, Settings::default(), "the defaults, as a file");
    let _ = fs::remove_dir_all(&root);
}
