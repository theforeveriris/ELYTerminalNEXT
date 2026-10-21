use std::{
    env, fs,
    io::ErrorKind,
    path::{Path, PathBuf},
};

use anyhow::Context as _;
use gpui::{App, Global};

use super::{
    drop::{Quote, Target},
    pty::Launch,
};

pub mod scan;

/// Each script, by its path under the integration folder.
const SCRIPTS: [(&str, &str); 4] = [
    (
        "zsh/.zshenv",
        include_str!("../../assets/shell-integration/zsh/.zshenv"),
    ),
    (
        "bash/elyterminal.bash",
        include_str!("../../assets/shell-integration/bash/elyterminal.bash"),
    ),
    (
        "fish/vendor_conf.d/elyterminal.fish",
        include_str!("../../assets/shell-integration/fish/vendor_conf.d/elyterminal.fish"),
    ),
    (
        "pwsh/elyterminal.ps1",
        include_str!("../../assets/shell-integration/pwsh/elyterminal.ps1"),
    ),
];

/// The folder the scripts live in, written at start.
pub struct ShellIntegration(pub PathBuf);

impl Global for ShellIntegration {}

/// The shells the scan found at start, best first.
pub struct Shells(pub Vec<scan::Found>);

impl Global for Shells {}

impl ShellIntegration {
    /// Writes the scripts into the platform's data folder and keeps its path.
    pub fn init(cx: &mut App) -> anyhow::Result<()> {
        let dir = dirs::data_local_dir()
            .context("shell integration: the platform names no data folder")?
            .join("ELYTerminal")
            .join("shell-integration");
        install(&dir)?;
        let found = scan::cached().to_vec();
        for shell in &found {
            log::info!(
                "shells: {} ({}), {}",
                shell.name,
                shell.command(),
                match shell.confidence {
                    scan::Confidence::Certain => "certain",
                    scan::Confidence::Likely => "likely",
                    scan::Confidence::Guess => "a guess",
                }
            );
        }
        cx.set_global(Shells(found));
        cx.set_global(Self(dir));
        Ok(())
    }
}

/// Writes the scripts into `dir`, replacing any that differ.
pub fn install(dir: &Path) -> anyhow::Result<()> {
    for (name, text) in SCRIPTS {
        let path = dir.join(name);
        match fs::read_to_string(&path) {
            Ok(kept) if kept == text => continue,
            Ok(_) => {}
            Err(error) if matches!(error.kind(), ErrorKind::NotFound | ErrorKind::InvalidData) => {}
            Err(error) => {
                return Err(error)
                    .with_context(|| format!("shell integration: cannot read {}", path.display()));
            }
        }
        let folder = path
            .parent()
            .context("shell integration: a script has no folder")?;
        fs::create_dir_all(folder)
            .with_context(|| format!("shell integration: cannot create {}", folder.display()))?;
        fs::write(&path, text)
            .with_context(|| format!("shell integration: cannot write {}", path.display()))?;
        log::info!("shell integration: wrote {}", path.display());
    }
    Ok(())
}

/// How to start `shell`, or the login shell when empty, in `cwd` so it reports its folder.
pub fn launch(shell: &str, cwd: PathBuf, dir: &Path) -> anyhow::Result<Launch> {
    #[cfg(windows)]
    if shell.is_empty()
        && let Some(found) = scan::best(scan::cached())
    {
        return launch_found(found, cwd, dir);
    }
    if shell.is_empty() {
        return start(default_shell()?, Vec::new(), false, None, cwd, dir);
    }
    // The settings checked this path; the session runs no other.
    let path = crate::settings::program_path(shell).context("shell integration")?;
    start(path, Vec::new(), true, None, cwd, dir)
}

/// How to start the shell the scan found, in `cwd`, with the integration its family reads.
pub fn launch_found(found: &scan::Found, cwd: PathBuf, dir: &Path) -> anyhow::Result<Launch> {
    start(
        found.program.display().to_string(),
        found.args.clone(),
        true,
        Some(found.family),
        cwd,
        dir,
    )
}

/// `path` with `args`, integration scripts from `dir`, in `cwd`; `explicit` false runs the login shell.
fn start(
    path: String,
    mut args: Vec<String>,
    explicit: bool,
    family: Option<scan::Family>,
    cwd: PathBuf,
    dir: &Path,
) -> anyhow::Result<Launch> {
    let name = Path::new(&path)
        .file_stem()
        .map(|stem| stem.to_string_lossy().to_lowercase())
        .unwrap_or_default();
    let family = family.unwrap_or(match name.as_str() {
        "pwsh" | "powershell" => scan::Family::PowerShell,
        "cmd" => scan::Family::Cmd,
        "bash" => scan::Family::Bash,
        "wsl" => scan::Family::Wsl,
        _ => scan::Family::Other,
    });
    let mut launch = Launch {
        program: explicit.then(|| (path.clone(), args.clone())),
        cwd: cwd.clone(),
        env: Vec::new(),
        reports: true,
        quote: match family {
            scan::Family::PowerShell => Quote::PowerShell,
            scan::Family::Cmd => Quote::Cmd,
            _ => Quote::Posix,
        },
        target: match family {
            scan::Family::Wsl => Target::Wsl {
                distro: args
                    .iter()
                    .position(|arg| arg == "-d")
                    .and_then(|at| args.get(at + 1))
                    .cloned()
                    .unwrap_or_else(|| "default".into()),
            },
            scan::Family::Bash | scan::Family::Msys | scan::Family::Cygwin if cfg!(windows) => {
                Target::GitBash
            }
            _ => Target::Local,
        },
    };
    let script = |name: &str| {
        let path = dir.join(name);
        if cfg!(windows) {
            scan::forward(&path)
        } else {
            path.display().to_string()
        }
    };
    let folder = if cfg!(windows) {
        scan::forward(dir)
    } else {
        dir.display().to_string()
    };
    match family {
        scan::Family::Wsl => {
            launch.reports = false;
            match scan::wsl_path(&cwd) {
                Some(folder) => {
                    args.extend(["--cd".into(), folder]);
                    launch.program = Some((path.clone(), args));
                }
                None => log::warn!(
                    "shell integration: {} is no WSL folder; wsl starts in its own home",
                    cwd.display()
                ),
            }
        }
        scan::Family::Bash | scan::Family::Msys | scan::Family::Cygwin => {
            args.extend(["--rcfile".into(), script("bash/elyterminal.bash")]);
            launch.program = Some((path.clone(), args));
            if cfg!(target_os = "macos") {
                launch.env.push(("ELY_BASH_LOGIN".into(), "1".into()));
            }
        }
        scan::Family::PowerShell => {
            let file = script("pwsh/elyterminal.ps1").replace('\'', "''");
            args.extend([
                "-NoLogo".into(),
                "-NoExit".into(),
                "-ExecutionPolicy".into(),
                "Bypass".into(),
                "-Command".into(),
                format!(
                    "try {{ . '{file}' }} catch {{ [Console]::Error.WriteLine(\"ELYTerminal: shell integration failed: $_\"); exit 1 }}"
                ),
            ]);
            launch.program = Some((path.clone(), args));
        }
        scan::Family::Cmd => {
            launch.reports = false;
            log::info!("shell integration: none for {path}; its folder goes unreported");
        }
        scan::Family::Nu | scan::Family::Elvish | scan::Family::Xonsh | scan::Family::Other => {
            match name.as_str() {
                "zsh" => {
                    if let Ok(zdotdir) = env::var("ZDOTDIR") {
                        launch.env.push(("ELY_ZSH_ZDOTDIR".into(), zdotdir));
                    }
                    launch.env.push(("ZDOTDIR".into(), script("zsh")));
                }
                "fish" => {
                    let previous = env::var("XDG_DATA_DIRS").unwrap_or_default();
                    let rest = if previous.is_empty() {
                        "/usr/local/share:/usr/share"
                    } else {
                        &previous
                    };
                    let data = format!("{folder}:{rest}");
                    launch.env.push(("XDG_DATA_DIRS".into(), data));
                    launch.env.push(("ELY_FISH_XDG_DATA_DIRS".into(), previous));
                }
                _ => {
                    launch.reports = false;
                    log::info!("shell integration: none for {path}; its folder goes unreported")
                }
            }
        }
    }
    log::info!("shell integration: {path} as {name}");
    Ok(launch)
}

#[cfg(unix)]
fn default_shell() -> anyhow::Result<String> {
    Ok(super::unix::login_shell_path()?)
}

#[cfg(windows)]
fn default_shell() -> anyhow::Result<String> {
    Ok("powershell.exe".into())
}

#[cfg(all(test, unix))]
mod tests;

#[cfg(test)]
mod start_tests;
