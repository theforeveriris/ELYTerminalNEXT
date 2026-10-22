//! The main window's title, as a template composes it from the session's name, the folder it sits in and the shell it runs, since a title is where a user tells one window from another; a template that names none of `{title}`, `{cwd}` or `{shell}` falls back to the session's name alone.

use std::path::Path;

use crate::sessions::shown;

/// The template a fresh settings file carries: the session's name, then where it sits.
pub const DEFAULT: &str = "{title} — {cwd}";

/// `template` with each `{title}`, `{cwd}` and `{shell}` filled in; a variable it does not know is left as written.
pub fn compose(
    template: &str,
    title: &str,
    folder: &Path,
    home: Option<&Path>,
    shell: &str,
) -> String {
    if template.trim().is_empty() {
        return title.to_string();
    }
    let cwd = shown(folder, home);
    let shell = Path::new(shell)
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_string())
        .unwrap_or_else(|| shell.to_string());
    let composed = template
        .replace("{title}", title)
        .replace("{cwd}", &cwd)
        .replace("{shell}", &shell);
    let trimmed = composed.trim();
    if trimmed.is_empty() {
        title.to_string()
    } else {
        trimmed.to_string()
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    #[test]
    fn a_template_names_the_parts_it_asks_for() {
        let home = PathBuf::from("/home/me");
        let folder = PathBuf::from("/home/me/src/api");
        assert_eq!(
            compose(DEFAULT, "cargo", &folder, Some(&home), "/usr/bin/fish"),
            format!("cargo — {}", shown(&folder, Some(&home)))
        );
        assert_eq!(
            compose(
                "{shell}: {title}",
                "vim",
                &folder,
                Some(&home),
                "C:\\bin\\pwsh.exe"
            ),
            "pwsh: vim"
        );
        assert_eq!(
            compose("a window", "cargo", &folder, None, "sh"),
            "a window"
        );
    }

    #[test]
    fn a_template_that_says_nothing_falls_back_to_the_name() {
        assert_eq!(compose("", "cargo", Path::new("/x"), None, "sh"), "cargo");
        assert_eq!(
            compose("   ", "cargo", Path::new("/x"), None, "sh"),
            "cargo"
        );
        assert_eq!(
            compose("{unknown}", "cargo", Path::new("/x"), None, "sh"),
            "{unknown}",
            "a variable it does not know is left as written"
        );
    }
}
