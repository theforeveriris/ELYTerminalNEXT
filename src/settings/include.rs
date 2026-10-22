//! Laying one settings file over another: a project's settings over the user's, a host's over both. A file names the files it builds on with `include`, a string or a list of them, each relative to itself, and those are laid first across each other before the file's own fields go over theirs; a file that comes home to one already laid stops there, so a loop of includes ends instead of running on.

use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::Context as _;

use super::Settings;

/// `settings` laid over `under`: each field `settings` names wins, and one it leaves out keeps `under`'s. The merge goes field by field because a settings file may say only what it changes, and a value left at its default stands for "not named", though an empty string or an empty list is taken as named.
pub fn over(under: &mut Settings, settings: &Settings) {
    let defaults = Settings::default();
    let set = |now: &str, was: &str, default: &str| now != default || was == default;
    if set(&settings.shell, &under.shell, &defaults.shell) {
        under.shell = settings.shell.clone();
    }
    if set(&settings.editor, &under.editor, &defaults.editor) {
        under.editor = settings.editor.clone();
    }
    if set(
        &settings.start_in_folder,
        &under.start_in_folder,
        &defaults.start_in_folder,
    ) {
        under.start_in_folder = settings.start_in_folder.clone();
    }
    if set(&settings.theme, &under.theme, &defaults.theme) {
        under.theme = settings.theme.clone();
    }
    if settings.dark_theme.is_some() {
        under.dark_theme = settings.dark_theme.clone();
    }
    if set(
        &settings.font_family,
        &under.font_family,
        &defaults.font_family,
    ) {
        under.font_family = settings.font_family.clone();
    }
    if !settings.font_fallbacks.is_empty() {
        under.font_fallbacks = settings.font_fallbacks.clone();
    }
    if settings.font_size != defaults.font_size || under.font_size == defaults.font_size {
        under.font_size = settings.font_size;
    }
    if settings.scrollback != defaults.scrollback || under.scrollback == defaults.scrollback {
        under.scrollback = settings.scrollback;
    }
    if settings.language != defaults.language {
        under.language = settings.language;
    }
    under.extra.remove("include");
}

/// The files a TOML document's `include` names, in order: one string or a list of them.
pub fn includes(document: &toml::Table) -> Vec<String> {
    match document.get("include") {
        Some(toml::Value::String(one)) => vec![one.clone()],
        Some(toml::Value::Array(many)) => many
            .iter()
            .filter_map(|value| value.as_str().map(str::to_string))
            .collect(),
        Some(_) => {
            log::warn!("settings: include is neither a string nor a list, so it names none");
            Vec::new()
        }
        None => Vec::new(),
    }
}

/// The file at `path` read as settings, each file `include` names read first and laid under it.
pub fn read(path: &Path) -> anyhow::Result<Settings> {
    let mut seen = Vec::new();
    lay(path, &mut seen)
}

fn lay(path: &Path, seen: &mut Vec<PathBuf>) -> anyhow::Result<Settings> {
    let read = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut settings = Settings::default();
    if seen.contains(&read) {
        log::debug!(
            "settings: {} is included again, so it is read no more",
            path.display()
        );
        return Ok(settings);
    }
    seen.push(read.clone());
    let text = fs::read_to_string(path)
        .with_context(|| format!("settings: cannot read {}", path.display()))?;
    let document: toml::Table = toml::from_str(&text)
        .map_err(|error| anyhow::anyhow!("settings: {} does not parse: {error}", path.display()))?;
    let folder = path.parent().unwrap_or(Path::new("."));
    for named in includes(&document) {
        let included = lay(&folder.join(&named), seen).with_context(|| {
            format!(
                "settings: {} includes {named}, which cannot be read",
                path.display()
            )
        })?;
        over(&mut settings, &included);
    }
    let own: Settings = document
        .clone()
        .try_into()
        .map_err(|error| anyhow::anyhow!("settings: {} does not parse: {error}", path.display()))?;
    own.validate()
        .with_context(|| format!("settings: {} holds a bad value", path.display()))?;
    over(&mut settings, &own);
    Ok(settings)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("ely-include-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("a test folder");
        root
    }

    fn write(folder: &Path, name: &str, text: &str) -> PathBuf {
        let path = folder.join(name);
        fs::write(&path, text).expect("a settings file");
        path
    }

    #[test]
    fn a_file_lays_its_own_over_the_ones_it_names() {
        let root = scratch("over");
        write(
            &root,
            "base.toml",
            "font_size = 15.0\ntheme = \"catppuccin-latte\"\nscrollback = 2000\n",
        );
        let path = write(
            &root,
            "config.toml",
            "include = \"base.toml\"\nfont_size = 20.0\n",
        );
        let settings = read(&path).expect("layered settings");
        assert_eq!(settings.font_size, 20.0, "its own wins");
        assert_eq!(settings.theme, "catppuccin-latte", "the base's stands");
        assert_eq!(settings.scrollback, 2000);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn the_last_listed_name_lays_over_the_earlier_ones() {
        let root = scratch("order");
        write(
            &root,
            "a.toml",
            "font_size = 9.0\ntheme = \"catppuccin-latte\"\n",
        );
        write(&root, "b.toml", "font_size = 11.0\n");
        let path = write(&root, "config.toml", "include = [\"a.toml\", \"b.toml\"]\n");
        let settings = read(&path).expect("layered settings");
        assert_eq!(settings.font_size, 11.0, "the later file wins");
        assert_eq!(
            settings.theme, "catppuccin-latte",
            "and keeps what the earlier set"
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_file_that_comes_home_to_itself_ends_instead_of_running_on() {
        let root = scratch("loop");
        write(&root, "a.toml", "include = \"b.toml\"\nfont_size = 8.0\n");
        write(
            &root,
            "b.toml",
            "include = \"a.toml\"\ntheme = \"catppuccin-latte\"\n",
        );
        let path = root.join("a.toml");
        let settings = read(&path).expect("a loop ends");
        assert_eq!(settings.theme, "catppuccin-latte");
        assert_eq!(settings.font_size, 8.0);
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_name_that_cannot_be_read_says_which_file_named_it() {
        let root = scratch("missing");
        let path = write(&root, "config.toml", "include = \"gone.toml\"\n");
        let error = format!("{:#}", read(&path).expect_err("no such file"));
        assert!(error.contains("config.toml includes gone.toml"), "{error}");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn an_include_of_no_name_is_no_include() {
        let document: toml::Table = toml::from_str("include = 3\n").expect("parses");
        assert!(includes(&document).is_empty());
        let document: toml::Table =
            toml::from_str("include = [\"a\", 2, \"b\"]\n").expect("parses");
        assert_eq!(
            includes(&document),
            ["a", "b"],
            "the names count, the rest not"
        );
    }
}
