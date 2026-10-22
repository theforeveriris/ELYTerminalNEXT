//! Where a session's folder is, and how each shell writes it.

use std::path::{Path, PathBuf};

/// A folder a session reports, and who holds it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Location {
    Local(PathBuf),
    Wsl {
        distro: String,
        path: String,
    },
    Ssh {
        host: String,
        user: Option<String>,
        path: String,
    },
    Container {
        name: String,
        path: String,
    },
}

impl Location {
    /// A local folder.
    pub fn local(folder: impl Into<PathBuf>) -> Self {
        Self::Local(folder.into())
    }

    /// The folder on this machine, when it is on this machine.
    pub fn on_disk(&self) -> Option<&Path> {
        match self {
            Self::Local(folder) => Some(folder),
            _ => None,
        }
    }

    /// The folder as a row names it, with its connection in front.
    pub fn display(&self) -> String {
        match self {
            Self::Local(folder) => folder.display().to_string(),
            Self::Wsl { distro, path } => format!("WSL: {distro}:{path}"),
            Self::Ssh { host, user, path } => match user {
                Some(user) => format!("{user}@{host}:{path}"),
                None => format!("{host}:{path}"),
            },
            Self::Container { name, path } => format!("{name}:{path}"),
        }
    }

    /// The Windows path this folder names, when the platform can reach it.
    pub fn windows_path(&self) -> Option<PathBuf> {
        match self {
            Self::Local(folder) => Some(folder.clone()),
            Self::Wsl { distro, path } => wsl_to_windows(path, distro).map(PathBuf::from),
            _ => None,
        }
    }

    /// The folder an OSC 7 `file://host/path` names, when `host` is this machine.
    pub fn from_file_uri(uri: &str, local_host: &str) -> Option<Self> {
        let rest = uri.strip_prefix("file://")?;
        let slash = rest.find('/')?;
        let host = &rest[..slash];
        if !(host.is_empty()
            || host.eq_ignore_ascii_case("localhost")
            || host.eq_ignore_ascii_case(local_host))
        {
            return None;
        }
        let path = percent_decode(&rest.as_bytes()[slash..])?;
        let drive = path
            .as_bytes()
            .get(1..3)
            .is_some_and(|two| two[1] == b':' && two[0].is_ascii_alphabetic());
        Some(Self::Local(PathBuf::from(if cfg!(windows) && drive {
            &path[1..]
        } else {
            &path[..]
        })))
    }
}

impl From<PathBuf> for Location {
    fn from(folder: PathBuf) -> Self {
        Self::Local(folder)
    }
}

/// How a path is written when it is copied.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum As {
    /// As the platform writes it: `C:\Users\me\src`, or `/home/me/src`.
    #[default]
    Native,
    /// As a POSIX shell reads it: `/c/Users/me/src` from `C:\Users\me\src`.
    Posix,
    /// As WSL reads it: `/mnt/c/Users/me/src`.
    Wsl,
    /// Quoted, so a shell that reads it reads one path.
    Quoted,
    /// A `file:` URI, with each byte the URI may not hold escaped.
    Uri,
}

/// `path` as `as` writes it; a form that does not apply to it falls back to the platform's own.
pub fn written(path: &Path, as_: As) -> String {
    let Some(text) = path.to_str() else {
        return path.display().to_string();
    };
    let mapped = match as_ {
        As::Native => None,
        As::Posix => windows_to_posix(text),
        As::Wsl => windows_to_wsl(text),
        As::Quoted => None,
        As::Uri => return uri(text),
    };
    let text = mapped.as_deref().unwrap_or(text);
    match as_ {
        As::Quoted => quote(text),
        _ => text.to_owned(),
    }
}

/// `path` as a POSIX or Windows shell takes it in one piece: what it reads specially is escaped.
fn quote(path: &str) -> String {
    if path.is_empty() {
        return "''".to_string();
    }
    if cfg!(windows) {
        return format!("\"{}\"", path.replace('"', "\\\""));
    }
    let special = path.chars().any(|c| {
        matches!(
            c,
            ' ' | '\''
                | '"'
                | '\\'
                | '$'
                | '`'
                | '*'
                | '?'
                | '('
                | ')'
                | ';'
                | '&'
                | '|'
                | '<'
                | '>'
                | '!'
                | '#'
                | '~'
        )
    });
    if !special {
        return path.to_owned();
    }
    format!("'{}'", path.replace('\'', r"'\''"))
}

/// `path` as a `file:` URI; on Windows, `C:\a` is `file:///C:/a`.
fn uri(path: &str) -> String {
    let text = path.replace('\\', "/");
    let text = if cfg!(windows) {
        let drive = text.as_bytes().get(1) == Some(&b':');
        match drive {
            true => format!("/{text}"),
            false => text,
        }
    } else {
        text
    };
    let mut out = String::from("file://");
    for byte in text.bytes() {
        match byte {
            // What a URI may hold unescaped, and what the user reads best unescaped.
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' | b'/' | b':' => {
                out.push(char::from(byte))
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}
pub fn percent_decode(bytes: &[u8]) -> Option<String> {
    let mut out = Vec::with_capacity(bytes.len());
    let mut at = 0;
    while at < bytes.len() {
        let byte = bytes[at];
        let hex = bytes
            .get(at + 1..at + 3)
            .and_then(|two| std::str::from_utf8(two).ok())
            .and_then(|two| u8::from_str_radix(two, 16).ok());
        match (byte, hex) {
            (b'%', Some(value)) => {
                out.push(value);
                at += 3;
            }
            (byte, _) => {
                out.push(byte);
                at += 1;
            }
        }
    }
    String::from_utf8(out).ok()
}

/// A Windows path as WSL reads it: `C:\x` is `/mnt/c/x`, and a WSL share is its own folder.
pub fn to_wsl(path: &Path) -> Option<String> {
    windows_to_wsl(path.to_str()?)
}

/// A Windows path string as WSL reads it.
pub fn windows_to_wsl(text: &str) -> Option<String> {
    for share in [r"\\wsl.localhost\", r"\\wsl$\"] {
        if let Some(rest) = text.strip_prefix(share) {
            let (_, folder) = rest.split_once('\\')?;
            return Some(format!("/{}", folder.replace('\\', "/")));
        }
    }
    let (drive, rest) = split_drive(text)?;
    Some(match rest.is_empty() {
        true => format!("/mnt/{drive}"),
        false => format!("/mnt/{drive}/{}", rest.replace('\\', "/")),
    })
}

/// A folder WSL reports as the Windows path to it, `\\wsl.localhost\<distro>\...` for its own.
pub fn wsl_to_windows(path: &str, distro: &str) -> Option<String> {
    if let Some(rest) = path.strip_prefix("/mnt/") {
        let (drive, tail) = rest.split_once('/').unwrap_or((rest, ""));
        if drive.len() == 1 && drive.as_bytes()[0].is_ascii_alphabetic() {
            let drive = drive.to_ascii_uppercase();
            return Some(match tail.is_empty() {
                true => format!("{drive}:\\"),
                false => format!("{drive}:\\{}", tail.replace('/', "\\")),
            });
        }
    }
    if path.starts_with('/') {
        let folder = path.trim_start_matches('/').replace('/', "\\");
        return Some(format!(r"\\wsl.localhost\{distro}\{folder}"));
    }
    None
}

/// A Windows path as a Git Bash shell reads it: `C:\x` is `/c/x`.
pub fn windows_to_posix(text: &str) -> Option<String> {
    let (drive, rest) = split_drive(text)?;
    Some(match rest.is_empty() {
        true => format!("/{drive}"),
        false => format!("/{drive}/{}", rest.replace('\\', "/")),
    })
}

/// A Git Bash path as Windows reads it: `/c/x` is `C:\x`; `/usr/bin` names no drive.
pub fn posix_to_windows(path: &str) -> Option<String> {
    let rest = path.strip_prefix('/')?;
    let (drive, tail) = rest.split_once('/').unwrap_or((rest, ""));
    if drive.len() != 1 || !drive.as_bytes()[0].is_ascii_alphabetic() {
        return None;
    }
    let drive = drive.to_ascii_uppercase();
    Some(match tail.is_empty() {
        true => format!("{drive}:\\"),
        false => format!("{drive}:\\{}", tail.replace('/', "\\")),
    })
}

/// A Windows drive path's lower-case drive and what follows it.
fn split_drive(text: &str) -> Option<(char, &str)> {
    let bytes = text.as_bytes();
    if bytes.len() < 2 || bytes[1] != b':' || !bytes[0].is_ascii_alphabetic() {
        return None;
    }
    let drive = (bytes[0] as char).to_ascii_lowercase();
    let rest = match bytes.get(2) {
        None => "",
        Some(b'\\') | Some(b'/') => &text[3..],
        Some(_) => return None,
    };
    if rest.contains(':') || rest.starts_with("//") {
        return None;
    }
    Some((drive, rest))
}

#[cfg(test)]
mod tests;
