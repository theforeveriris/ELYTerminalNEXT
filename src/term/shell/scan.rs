//! Finding the shells this machine has, and saying why each one is listed.

use std::{
    collections::HashSet,
    env,
    path::{Component, Path, PathBuf, Prefix},
    process::Command,
    sync::OnceLock,
};

/// The kind of program a shell is, which decides how it starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Family {
    PowerShell,
    Cmd,
    Bash,
    Wsl,
    Msys,
    Cygwin,
    Nu,
    Elvish,
    Xonsh,
    Other,
}

impl Family {
    /// The family as a session names it.
    pub fn name(self) -> &'static str {
        match self {
            Self::PowerShell => "PowerShell",
            Self::Cmd => "cmd",
            Self::Bash => "bash",
            Self::Wsl => "WSL",
            Self::Msys => "MSYS2",
            Self::Cygwin => "Cygwin",
            Self::Nu => "nu",
            Self::Elvish => "elvish",
            Self::Xonsh => "xonsh",
            Self::Other => "shell",
        }
    }
}

/// Where the scan learned of a shell.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    /// A path the platform is known to install it at.
    Known,
    /// A folder on `PATH`.
    Path,
    /// An install root the registry named.
    Registry,
    /// A distribution `wsl.exe` listed.
    Wsl,
}

/// How sure the scan is that a shell starts.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Confidence {
    Guess,
    Likely,
    Certain,
}

/// One shell the scan found, ready to launch.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Found {
    /// What the settings call it, such as `Git Bash` or `WSL: archlinux`.
    pub name: String,
    pub family: Family,
    pub program: PathBuf,
    /// Arguments before the folder and integration arguments.
    pub args: Vec<String>,
    pub source: Source,
    pub confidence: Confidence,
}

impl Found {
    /// A shell at `program`, named `name`.
    pub fn new(
        name: impl Into<String>,
        family: Family,
        program: impl Into<PathBuf>,
        source: Source,
        confidence: Confidence,
    ) -> Self {
        Self {
            name: name.into(),
            family,
            program: program.into(),
            args: Vec::new(),
            source,
            confidence,
        }
    }

    /// The command line a log or a diagnostic shows, with no folder.
    pub fn command(&self) -> String {
        let mut out = self.program.display().to_string();
        for arg in &self.args {
            out.push(' ');
            out.push_str(arg);
        }
        out
    }

    /// How good a default it is: a shell the user installed scores over one the system shipped.
    pub fn score(&self) -> u32 {
        let family = match self.family {
            Family::PowerShell => 70,
            Family::Bash => 60,
            Family::Wsl => 50,
            Family::Nu | Family::Elvish | Family::Xonsh => 40,
            Family::Msys | Family::Cygwin => 30,
            Family::Cmd => 20,
            Family::Other => 10,
        };
        let source = match self.source {
            Source::Known => 9,
            Source::Registry => 8,
            Source::Path => 7,
            Source::Wsl => 6,
        };
        let confidence = match self.confidence {
            Confidence::Certain => 5,
            Confidence::Likely => 3,
            Confidence::Guess => 1,
        };
        family
            + source
            + confidence
            + if self.name.contains("PowerShell 7") {
                30
            } else {
                0
            }
    }
}

/// The shell that starts when the settings name none, the best found.
pub fn best(found: &[Found]) -> Option<&Found> {
    found.iter().max_by_key(|shell| shell.score())
}

/// Every shell the scan can find, best first.
pub fn scan() -> Vec<Found> {
    let mut found = Vec::new();
    #[cfg(windows)]
    windows(&mut found);
    #[cfg(unix)]
    unix(&mut found);
    let mut seen = HashSet::new();
    found.retain(|shell| seen.insert((shell.program.clone(), shell.args.clone())));
    found.sort_by(|left, right| {
        right
            .score()
            .cmp(&left.score())
            .then_with(|| left.name.cmp(&right.name))
    });
    found
}

/// The scan's answer, found once and kept for the sessions a window opens.
pub fn cached() -> &'static [Found] {
    static FOUND: OnceLock<Vec<Found>> = OnceLock::new();
    FOUND.get_or_init(scan)
}

#[cfg(windows)]
fn windows(found: &mut Vec<Found>) {
    let program_files = env::var_os("ProgramFiles").map(PathBuf::from);
    let program_files_x86 = env::var_os("ProgramFiles(x86)").map(PathBuf::from);
    let local = env::var_os("LOCALAPPDATA").map(PathBuf::from);
    let system = env::var_os("SystemRoot").map(PathBuf::from);
    if let Some(root) = &program_files {
        known(
            found,
            root.join("PowerShell").join("7").join("pwsh.exe"),
            "PowerShell 7",
            Family::PowerShell,
            Confidence::Certain,
        );
        known(
            found,
            root.join("PowerShell").join("7-preview").join("pwsh.exe"),
            "PowerShell 7 preview",
            Family::PowerShell,
            Confidence::Certain,
        );
    }
    let mut roots: Vec<PathBuf> = [program_files, program_files_x86, local]
        .into_iter()
        .flatten()
        .map(|root| root.join("Git"))
        .collect();
    if let Some(local) = env::var_os("LOCALAPPDATA") {
        roots.push(PathBuf::from(local).join("Programs").join("Git"));
    }
    if let Some(root) = install_root() {
        roots.push(root);
    }
    for root in roots {
        if let Some(shell) = git_bash(&root) {
            found.push(shell);
        }
    }
    if let Some(system) = &system {
        known(
            found,
            system
                .join("System32")
                .join("WindowsPowerShell")
                .join("v1.0")
                .join("powershell.exe"),
            "PowerShell 5.1",
            Family::PowerShell,
            Confidence::Certain,
        );
    }
    if let Some(comspec) = env::var_os("COMSPEC") {
        known(
            found,
            comspec.into(),
            "cmd",
            Family::Cmd,
            Confidence::Certain,
        );
    } else if let Some(system) = &system {
        known(
            found,
            system.join("System32").join("cmd.exe"),
            "cmd",
            Family::Cmd,
            Confidence::Certain,
        );
    }
    let drive = env::var_os("SystemDrive").unwrap_or_else(|| "C:".into());
    for (root, name, family) in [
        ("msys64", "MSYS2", Family::Msys),
        ("msys32", "MSYS2", Family::Msys),
        ("cygwin64", "Cygwin", Family::Cygwin),
        ("cygwin", "Cygwin", Family::Cygwin),
    ] {
        known(
            found,
            PathBuf::from(&drive)
                .join(root)
                .join("bin")
                .join("bash.exe"),
            name,
            family,
            Confidence::Likely,
        );
    }
    for (name, family) in [
        ("nu.exe", Family::Nu),
        ("elvish.exe", Family::Elvish),
        ("xonsh.exe", Family::Xonsh),
    ] {
        if let Some(path) = on_path(name) {
            found.push(Found::new(
                name.trim_end_matches(".exe"),
                family,
                path,
                Source::Path,
                Confidence::Likely,
            ));
        }
    }
    if let Some(wsl) = on_path("wsl.exe") {
        for distro in distros(&wsl) {
            let mut shell = Found::new(
                format!("WSL: {}", distro.name),
                Family::Wsl,
                wsl.clone(),
                Source::Wsl,
                if distro.running() {
                    Confidence::Certain
                } else {
                    Confidence::Likely
                },
            );
            shell.args = vec!["-d".into(), distro.name];
            found.push(shell);
        }
    }
}

#[cfg(unix)]
fn unix(found: &mut Vec<Found>) {
    if let Ok(shell) = super::super::unix::login_shell_path() {
        let family = match Path::new(&shell)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
        {
            "zsh" => Family::Other,
            "bash" => Family::Bash,
            "fish" => Family::Other,
            _ => Family::Other,
        };
        found.push(Found::new(
            "login shell",
            family,
            shell,
            Source::Known,
            Confidence::Certain,
        ));
    }
    for (name, family) in [
        ("zsh", Family::Other),
        ("bash", Family::Bash),
        ("fish", Family::Other),
        ("nu", Family::Nu),
        ("elvish", Family::Elvish),
        ("xonsh", Family::Xonsh),
    ] {
        if let Some(path) = on_path(name) {
            found.push(Found::new(
                name,
                family,
                path,
                Source::Path,
                Confidence::Certain,
            ));
        }
    }
}

/// Adds `program` when it is a file, named and from a known folder.
fn known(
    found: &mut Vec<Found>,
    program: PathBuf,
    name: &str,
    family: Family,
    confidence: Confidence,
) {
    if program.is_file() {
        found.push(Found::new(name, family, program, Source::Known, confidence));
    }
}

/// Git Bash at `root`, when the root holds both its bash and its git and is not the system's WSL launcher.
pub fn git_bash(root: &Path) -> Option<Found> {
    let bash = root.join("bin").join("bash.exe");
    let git = root.join("cmd").join("git.exe");
    if !bash.is_file() || !git.is_file() {
        return None;
    }
    let name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    if name.eq_ignore_ascii_case("system32") {
        return None;
    }
    Some(Found::new(
        "Git Bash",
        Family::Bash,
        bash,
        Source::Registry,
        Confidence::Certain,
    ))
}

/// The install root `HKLM\SOFTWARE\GitForWindows` names, if the registry answers.
#[cfg(windows)]
fn install_root() -> Option<PathBuf> {
    let output = command(
        "reg.exe",
        &["query", r"HKLM\SOFTWARE\GitForWindows", "/v", "InstallPath"],
    )?;
    install_path(&output).map(PathBuf::from)
}

/// The `InstallPath` a `reg query` answer holds.
pub fn install_path(output: &str) -> Option<String> {
    output.lines().find_map(|line| {
        let (name, value) = line.split_once("REG_SZ")?;
        name.trim()
            .eq_ignore_ascii_case("InstallPath")
            .then(|| value.trim().to_string())
            .filter(|value| !value.is_empty())
    })
}

/// A distribution `wsl.exe -l -v` lists.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Distro {
    pub name: String,
    pub state: String,
    pub version: Option<u8>,
    pub default: bool,
}

impl Distro {
    /// Whether it is running now, as WSL spells the state.
    pub fn running(&self) -> bool {
        self.state.eq_ignore_ascii_case("running")
    }
}

/// The distributions `wsl.exe` runs, which lists the default, then asks for its answer.
#[cfg(windows)]
fn distros(wsl: &Path) -> Vec<Distro> {
    let Some(bytes) = command_bytes(wsl, &["-l", "-v"]) else {
        return Vec::new();
    };
    distros_from(&utf16le(&bytes))
}

/// The distributions in `wsl -l -v`'s answer, whatever language its header speaks.
pub fn distros_from(text: &str) -> Vec<Distro> {
    text.lines()
        .filter_map(|line| {
            let mut parts = line.split_whitespace().peekable();
            let mut default = false;
            let mut first = parts.next()?;
            if first == "*" {
                default = true;
                first = parts.next()?;
            }
            let rest: Vec<&str> = parts.collect();
            if rest.is_empty() {
                return None;
            }
            if first.eq_ignore_ascii_case("name") || first.eq_ignore_ascii_case("名称") {
                return None;
            }
            let version = rest.last().and_then(|version| version.parse::<u8>().ok());
            let state = match version {
                Some(_) if rest.len() >= 2 => rest[..rest.len() - 1].join(" "),
                Some(_) => String::new(),
                None => rest.join(" "),
            };
            if state.is_empty() && version.is_none() {
                return None;
            }
            Some(Distro {
                name: first.to_string(),
                state,
                version,
                default,
            })
        })
        .collect()
}

/// `bytes` as text: WSL answers in UTF-16LE, other programs in the platform's own.
pub fn utf16le(bytes: &[u8]) -> String {
    let zeros = bytes.chunks_exact(2).filter(|pair| pair[1] == 0).count();
    let utf16 = bytes.len() > 1 && bytes.len() % 2 == 0 && zeros * 4 >= bytes.len();
    if !utf16 {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let units: Vec<u16> = bytes
        .chunks_exact(2)
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    String::from_utf16_lossy(&units)
}

/// The first file named `name` in the settings' `PATH`, as `program_path` would find it.
fn on_path(name: &str) -> Option<PathBuf> {
    let folders = env::var_os("PATH")?;
    env::split_paths(&folders)
        .filter(|folder| folder.is_absolute())
        .map(|folder| folder.join(name))
        .find(|file| file.is_file())
}

/// A Windows path as WSL reads it: `C:\x` is `/mnt/c/x`, and a WSL share is its own folder.
pub fn wsl_path(path: &Path) -> Option<String> {
    let text = path.to_str()?;
    for share in [r"\\wsl.localhost\", r"\\wsl$\"] {
        if let Some(rest) = text.strip_prefix(share) {
            let (_, folder) = rest.split_once('\\')?;
            return Some(format!("/{}", folder.replace('\\', "/")));
        }
    }
    let mut parts = path.components();
    let Some(Component::Prefix(prefix)) = parts.next() else {
        return path.is_absolute().then(|| forward(path));
    };
    let Prefix::Disk(letter) = prefix.kind() else {
        return None;
    };
    let mut out = format!("/mnt/{}", letter.to_ascii_lowercase() as char);
    let mut rooted = false;
    for part in parts {
        match part {
            Component::RootDir => rooted = true,
            Component::Normal(name) => {
                out.push('/');
                out.push_str(&name.to_string_lossy());
            }
            _ => return None,
        }
    }
    rooted.then_some(out)
}

/// `path` with the separators a POSIX shell on Windows reads.
pub fn forward(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

/// What `program` writes to its standard output, as text, or none when it does not start.
#[cfg(windows)]
fn command(program: &str, args: &[&str]) -> Option<String> {
    command_bytes(program, args).map(|bytes| utf16le(&bytes))
}

/// What `program` writes to its standard output, or none when it does not start.
#[cfg(windows)]
fn command_bytes(program: impl AsRef<std::ffi::OsStr>, args: &[&str]) -> Option<Vec<u8>> {
    let output = Command::new(program).args(args).output().ok()?;
    output.status.success().then_some(output.stdout)
}

#[cfg(test)]
mod tests;
