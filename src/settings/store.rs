use std::{
    fs,
    io::{ErrorKind, Write},
    path::{Path, PathBuf},
    time::SystemTime,
};

use anyhow::Context as _;
use gpui::{App, Global};

use super::Settings;
use super::write::{is_toml, save};

/// The settings in force and the file they live in; the only writer of that file.
pub struct SettingsStore {
    settings: Settings,
    path: PathBuf,
    /// When the file last changed, as the store last read it.
    modified: Option<SystemTime>,
}

impl Global for SettingsStore {}

impl SettingsStore {
    /// Loads `config.toml`, or `settings.json` when there is no TOML, into the app; a first run writes a `config.toml` of its own, each field with a comment saying what it does.
    pub fn init(cx: &mut App) -> anyhow::Result<()> {
        let folder = dirs::config_dir().context("settings: the platform names no config folder")?;
        let base = folder.join("ELYTerminal");
        let toml = base.join("config.toml");
        let json = base.join("settings.json");
        // CFG-16: the first run writes a config.toml of its own, each field under a comment saying what it sets, so the file the user finds is one they can read and change.
        let path = if !toml.exists() && !json.exists() {
            seeded(&toml)?;
            toml
        } else if toml.exists() {
            toml
        } else {
            json
        };
        let settings = load_any(&path)?;
        fonts(&settings, cx)
            .with_context(|| format!("settings: {} holds a bad value", path.display()))?;
        log::info!("settings: reading {}", path.display());
        cx.set_global(Self {
            settings,
            modified: stamp(&path),
            path,
        });
        Ok(())
    }

    pub fn get(cx: &App) -> &Settings {
        &cx.global::<Self>().settings
    }

    /// Applies `edit`, checks the result and saves it; a bad edit changes nothing.
    pub fn update(cx: &mut App, edit: impl FnOnce(&mut Settings)) -> anyhow::Result<()> {
        let store = cx.global::<Self>();
        let mut next = store.settings.clone();
        edit(&mut next);
        if next == store.settings {
            return Ok(());
        }
        next.validate()?;
        if (&next.font_family, &next.font_fallbacks)
            != (&store.settings.font_family, &store.settings.font_fallbacks)
        {
            fonts(&next, cx)?;
        }
        save(&store.path, &next)?;
        let modified = stamp(&store.path);
        let store = cx.global_mut::<Self>();
        store.settings = next;
        store.modified = modified;
        Ok(())
    }

    /// Loads the file again when another program changed it; true when the settings changed.
    pub fn reload(cx: &mut App) -> anyhow::Result<bool> {
        let (path, before) = {
            let store = cx.global::<Self>();
            (store.path.clone(), store.modified)
        };
        let modified = stamp(&path);
        if modified == before {
            return Ok(false);
        }
        let loaded = load_any(&path).and_then(|settings| {
            fonts(&settings, cx)?;
            Ok(settings)
        });
        match loaded {
            Ok(settings) => {
                let store = cx.global_mut::<Self>();
                store.settings = settings;
                store.modified = modified;
                Ok(true)
            }
            Err(error) => {
                // The file is bad; it is read again when it changes, not on every tick.
                cx.global_mut::<Self>().modified = modified;
                Err(error)
            }
        }
    }
}

/// When `path` last changed, or none when it does not exist.
fn stamp(path: &Path) -> Option<SystemTime> {
    fs::metadata(path).and_then(|meta| meta.modified()).ok()
}

/// Checks the terminal can load its family and each fallback; the error names the field.
fn fonts(settings: &Settings, cx: &App) -> anyhow::Result<()> {
    let fallbacks = settings
        .font_fallbacks
        .iter()
        .map(|family| ("font_fallbacks", family));
    for (field, family) in std::iter::once(("font_family", &settings.font_family)).chain(fallbacks)
    {
        crate::term::load_font(&gpui::font(family.clone()), cx.text_system())
            .with_context(|| field.to_string())?;
    }
    Ok(())
}

/// Writes `path`, a `config.toml` of the defaults with a comment over each field, when it is not there. A settings file found on the first run is one the user did not write and cannot read, so this one says what each field sets and what else it could be; an existing file is never overwritten, because a user's own is theirs.
pub fn seeded(path: &Path) -> anyhow::Result<()> {
    let settings = Settings::default();
    let lines = crate::settings::commented(&settings);
    let folder = path
        .parent()
        .with_context(|| format!("settings: {} has no folder", path.display()))?;
    fs::create_dir_all(folder)
        .with_context(|| format!("settings: cannot create {}", folder.display()))?;
    // An existing file stays: only a first run writes one, and a crash must not lose the user's.
    match fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .and_then(|mut file| file.write_all(lines.as_bytes()))
    {
        Ok(()) => {
            log::info!("settings: wrote {} with its defaults", path.display());
            Ok(())
        }
        Err(error) if error.kind() == ErrorKind::AlreadyExists => Ok(()),
        Err(error) => {
            Err(error).with_context(|| format!("settings: cannot write {}", path.display()))
        }
    }
}

/// The file's settings; defaults when there is no file. A file that fails to read or check is an error.
pub fn load(path: &Path) -> anyhow::Result<Settings> {
    let text = match super::read_text(path) {
        Ok(text) => text,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            log::info!("settings: no file at {}, defaults", path.display());
            return Ok(Settings::default());
        }
        Err(error) => {
            return Err(error).with_context(|| format!("settings: cannot read {}", path.display()));
        }
    };
    let mut reader = serde_json::Deserializer::from_str(&text);
    let settings: Settings = serde_path_to_error::deserialize(&mut reader).map_err(|error| {
        anyhow::anyhow!(
            "settings: {} does not parse at line {}, column {} (`{}`): {}",
            path.display(),
            error.inner().line(),
            error.inner().column(),
            error.path(),
            error.inner()
        )
    })?;
    reader
        .end()
        .with_context(|| format!("settings: {} does not parse", path.display()))?;
    settings
        .validate()
        .with_context(|| format!("settings: {} holds a bad value", path.display()))?;
    warn_extra(&settings, path);
    log::info!("settings: loaded {}", path.display());
    interpolated(settings, path)
}

/// The settings in `path`, read as TOML when the file is one and JSON otherwise.
pub fn load_any(path: &Path) -> anyhow::Result<Settings> {
    if is_toml(path) {
        load_toml(path)
    } else {
        load(path)
    }
}

/// Expands `${VAR}` in `text`, once, from the environment; one with no value is an error. A path in a settings file is the same on every machine but the user's name and home, so those come from the environment; no substitution runs a command, only reads a variable, and one the environment does not hold names itself in the error.
pub fn expand(text: &str) -> anyhow::Result<String> {
    let mut out = String::with_capacity(text.len());
    let mut rest = text;
    while let Some((before, after)) = rest.split_once("${") {
        let Some((name, tail)) = after.split_once('}') else {
            return Err(anyhow::anyhow!(
                "settings: {text:?} opens a variable and never closes it"
            ));
        };
        if name.is_empty() {
            return Err(anyhow::anyhow!("settings: {text:?} names no variable"));
        }
        out.push_str(before);
        match std::env::var(name) {
            Ok(value) => out.push_str(&value),
            Err(_) => {
                return Err(anyhow::anyhow!(
                    "settings: {text:?} names {name}, which the environment holds no value for"
                ));
            }
        }
        rest = tail;
    }
    out.push_str(rest);
    Ok(out)
}

/// The settings in a TOML file; no file gives the defaults, and a bad one names its line. A file's `include` names others, laid first and over one another, so shared settings live in one place and each file says only what it changes.
pub fn load_toml(path: &Path) -> anyhow::Result<Settings> {
    if !path.exists() {
        log::info!("settings: no file at {}, defaults", path.display());
        return Ok(Settings::default());
    }
    let settings = super::include::read(path)?;
    warn_extra(&settings, path);
    log::info!("settings: loaded {}", path.display());
    interpolated(settings, path)
}

/// A loaded file's settings with each `${VAR}` in a path or command expanded from the environment.
fn interpolated(mut settings: Settings, path: &Path) -> anyhow::Result<Settings> {
    for (field, value) in [
        ("shell", &mut settings.shell),
        ("editor", &mut settings.editor),
        ("start_in_folder", &mut settings.start_in_folder),
    ] {
        if value.contains("${") {
            *value = expand(value)
                .with_context(|| format!("settings: {} holds a bad {field}", path.display()))?;
        }
    }
    if let Some(last) = &settings.last_folder
        && last.contains("${")
    {
        settings.last_folder =
            Some(expand(last).with_context(|| {
                format!("settings: {} holds a bad last_folder", path.display())
            })?);
    }
    Ok(settings)
}

/// Logs each field the file names that this version does not know.
fn warn_extra(settings: &Settings, path: &Path) {
    for key in settings.extra.keys() {
        log::warn!(
            "settings: {} names an unknown field {key:?}; it is kept until the next save",
            path.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::CursorStyle;

    fn folder(name: &str) -> PathBuf {
        let folder =
            std::env::temp_dir().join(format!("elyterminal-{name}-{}", std::process::id()));
        if folder.exists() {
            fs::remove_dir_all(&folder).expect("a stale test folder goes");
        }
        folder
    }

    #[test]
    fn no_file_gives_defaults() {
        let path = folder("none").join("settings.json");
        assert_eq!(load(&path).expect("defaults"), Settings::default());
    }

    #[test]
    fn a_stamp_follows_the_file_it_names() {
        let folder = folder("stamp");
        fs::create_dir_all(&folder).expect("a test folder");
        let path = folder.join("settings.json");
        assert_eq!(stamp(&path), None, "no file, no stamp");
        fs::write(&path, "{}").expect("a file");
        let when = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(10);
        fs::File::options()
            .write(true)
            .open(&path)
            .expect("the file")
            .set_modified(when)
            .expect("a stamp");
        assert_eq!(stamp(&path), Some(when));
        fs::remove_dir_all(folder).expect("cleanup");
    }

    #[test]
    fn a_toml_file_loads_and_saves_itself() {
        let folder = folder("toml");
        let path = folder.join("config.toml");
        let settings = Settings {
            font_size: 15.0,
            theme: "catppuccin-latte".into(),
            ..Settings::default()
        };
        save(&path, &settings).expect("saves TOML");
        let text = fs::read_to_string(&path).expect("the file");
        assert!(text.contains("font_size = 15"), "{text}");
        assert!(text.contains("[custom_colors]"), "{text}");
        assert_eq!(load_toml(&path).expect("loads"), settings);
        assert_eq!(load_any(&path).expect("loads"), settings);
        fs::remove_dir_all(folder).expect("cleanup");
    }

    #[test]
    fn a_bad_toml_file_says_where_it_went_wrong() {
        let folder = folder("bad-toml");
        let path = folder.join("config.toml");
        fs::create_dir_all(&folder).expect("a folder");
        fs::write(&path, "font_size = [\n").expect("a file");
        let error = format!("{:#}", load_toml(&path).expect_err("a bad value"));
        assert!(error.contains("does not parse"), "{error}");
        assert!(error.contains("line"), "{error}");
        assert!(error.contains("config.toml"), "{error}");
        fs::remove_dir_all(folder).expect("cleanup");
    }

    #[test]
    fn an_unknown_field_is_read_warned_and_dropped_on_the_next_save() {
        let folder = folder("unknown");
        let path = folder.join("settings.json");
        fs::create_dir_all(&folder).expect("a folder");
        fs::write(
            &path,
            r##"{"font_size":14.0,"future_flag":true,"custom_colors":{"background":"#000000"}}"##,
        )
        .expect("a file");
        let settings = load(&path).expect("loads");
        assert_eq!(settings.font_size, 14.0);
        assert!(settings.extra.contains_key("future_flag"));
        save(&path, &settings).expect("saves");
        let text = fs::read_to_string(&path).expect("the file");
        assert!(!text.contains("future_flag"), "{text}");
        assert!(load(&path).expect("loads").extra.is_empty());
        fs::remove_dir_all(folder).expect("cleanup");
    }

    #[test]
    fn a_saved_file_loads_back_and_leaves_no_temp() {
        let folder = folder("round");
        let path = folder.join("settings.json");
        let settings = Settings {
            font_size: 15.5,
            cursor_style: CursorStyle::Bar,
            dark_theme: Some("catppuccin-latte".into()),
            ..Settings::default()
        };
        save(&path, &settings).expect("saves");
        assert_eq!(load(&path).expect("loads"), settings);
        assert!(!path.with_extension("json.tmp").exists());
        fs::remove_dir_all(folder).expect("cleanup");
    }

    #[test]
    fn overlapping_saves_leave_one_whole_file() {
        let folder = folder("overlap");
        let path = folder.join("settings.json");
        let writers: Vec<_> = (0..4)
            .map(|writer| {
                let path = path.clone();
                std::thread::spawn(move || {
                    for round in 0..50 {
                        let settings = Settings {
                            scrollback: writer * 1000 + round,
                            ..Settings::default()
                        };
                        save(&path, &settings).expect("each save lands");
                    }
                })
            })
            .collect();
        for writer in writers {
            writer.join().expect("a writer finishes");
        }
        let settings = load(&path).expect("the file is whole");
        assert_eq!(settings.scrollback % 1000, 49);
        let left: Vec<_> = fs::read_dir(&folder).expect("folder").collect();
        assert_eq!(left.len(), 1, "only settings.json remains");
        fs::remove_dir_all(folder).expect("cleanup");
    }

    #[test]
    fn a_missing_field_takes_its_default() {
        let folder = folder("partial");
        let path = folder.join("settings.json");
        fs::create_dir_all(&folder).expect("folder");
        fs::write(&path, r#"{ "font_size": 16.0 }"#).expect("write");
        let settings = load(&path).expect("loads");
        assert_eq!(settings.font_size, 16.0);
        assert_eq!(settings.theme, Settings::default().theme);
        fs::remove_dir_all(folder).expect("cleanup");
    }

    #[test]
    fn a_bad_file_is_an_error_naming_it() {
        let folder = folder("bad");
        let path = folder.join("settings.json");
        fs::create_dir_all(&folder).expect("folder");
        fs::write(&path, "{ font_size: ").expect("write");
        let error = load(&path).expect_err("does not parse");
        assert!(format!("{error:#}").contains("does not parse"));
        assert!(error.to_string().contains("line 1, column"), "{error}");
        assert!(error.to_string().contains("settings.json"));
        let named = |json: &str, field: &str| {
            fs::write(&path, json).expect("write");
            let error = format!("{:#}", load(&path).expect_err(json));
            assert!(error.contains("settings.json"), "{error}");
            assert!(error.contains(field), "{error}");
        };
        named(r#"{ "cursor_style": "beam" }"#, "`cursor_style`");
        named(r#"{ "scrollback": -1 }"#, "`scrollback`");
        named(r#"{ "font_size": "large" }"#, "`font_size`");
        named(r#"{ "font_size": 12 } x"#, "trailing characters");
        fs::remove_dir_all(folder).expect("cleanup");
    }

    #[test]
    fn a_bad_value_is_an_error_naming_the_field() {
        let folder = folder("range");
        let path = folder.join("settings.json");
        fs::create_dir_all(&folder).expect("folder");
        fs::write(&path, r#"{ "background_opacity": 0.1 }"#).expect("write");
        let error = load(&path).expect_err("too clear");
        assert!(format!("{error:#}").contains("background_opacity is 0.1"));
        fs::remove_dir_all(folder).expect("cleanup");
    }
}
