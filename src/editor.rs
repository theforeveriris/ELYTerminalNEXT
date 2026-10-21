//! Opening a file in the editor the user names.

use std::{
    ffi::OsString,
    path::Path,
    process::{Child, Command},
};

/// Splits a command line into its words, honoring single and double quotes and a backslash.
pub fn words(template: &str) -> Vec<String> {
    let mut words = Vec::new();
    let mut word = String::new();
    let mut quote = None;
    let mut chars = template.chars();
    while let Some(ch) = chars.next() {
        match (quote, ch) {
            (Some(open), close) if close == open => quote = None,
            (Some(_), ch) => word.push(ch),
            (None, '"' | '\'') => quote = Some(ch),
            (None, '\\') => {
                if let Some(ch) = chars.next() {
                    word.push(ch);
                }
            }
            (None, ch) if ch.is_whitespace() => {
                if !word.is_empty() {
                    words.push(std::mem::take(&mut word));
                }
            }
            (None, ch) => word.push(ch),
        }
    }
    if !word.is_empty() {
        words.push(word);
    }
    words
}

/// The program and arguments `template` names for `path`: `{file}` is replaced, else the path is added.
pub fn command(template: &str, path: &Path) -> Option<(String, Vec<OsString>)> {
    let mut words = words(template);
    if words.is_empty() {
        return None;
    }
    let program = words.remove(0);
    let mut args: Vec<OsString> = words.into_iter().map(OsString::from).collect();
    let file = path.display().to_string();
    let mut placed = false;
    for arg in &mut args {
        let shown = arg.to_string_lossy();
        if shown.contains("{file}") {
            *arg = OsString::from(shown.replace("{file}", &file));
            placed = true;
        }
    }
    if !placed {
        args.push(path.as_os_str().to_owned());
    }
    Some((program, args))
}

/// Starts the editor `template` names on `path`.
pub fn open(template: &str, path: &Path) -> std::io::Result<Child> {
    let (program, args) = command(template, path).ok_or_else(|| {
        std::io::Error::new(std::io::ErrorKind::InvalidInput, "no editor command")
    })?;
    Command::new(program).args(args).spawn()
}

#[cfg(test)]
mod tests;
