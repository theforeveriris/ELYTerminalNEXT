use std::{borrow::Cow, collections::HashMap, path::PathBuf, sync::Arc};

#[cfg(windows)]
use alacritty_terminal::tty;
use alacritty_terminal::{
    event::{Event, EventListener, Notify, OnResize, WindowSize},
    event_loop::{EventLoop, Msg, Notifier},
    sync::FairMutex,
    term::{Config, Term, test::TermSize},
    vte::ansi::CursorShape,
};
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender, unbounded};

use super::{
    device::Query,
    drop::{Quote, Target},
    marks::Mark,
    mouse::MouseMode,
    notify::Ask,
    status::Status,
    tap::Tapped,
};
use crate::settings::{CursorStyle, Settings};

/// What a session hears from its pty.
#[derive(Debug)]
pub enum PtyEvent {
    Term(Event),
    Folder(PathBuf),
    Mouse(MouseMode),
    Mark(Mark),
    /// A program's status, OSC 7501.
    Status(Status),
    /// What a program asked the terminal about itself: `CSI > 0 q`, XTVERSION.
    Device(Query),
    /// A notification or progress, OSC 9, 777 or 99.
    Ask(Ask),
    /// A title a program set, in the output's order: OSC 0 or 2, or the title stack's pop.
    Title(String),
    /// A full reset, `ESC c`.
    Reset,
    /// win32-input-mode set or reset.
    #[cfg(windows)]
    Win32(bool),
}

/// Hands the grid's events to the session; `pid` names the program in logs.
#[derive(Clone)]
pub struct Listener {
    sender: UnboundedSender<PtyEvent>,
    pid: u32,
}

impl EventListener for Listener {
    fn send_event(&self, event: Event) {
        if let Event::ChildExit(status) = &event {
            log::info!("pty {}: the program ended, {status}", self.pid);
        }
        if self.sender.unbounded_send(PtyEvent::Term(event)).is_err() {
            log::debug!("pty {}: an event after its session closed", self.pid);
        }
    }
}

/// A program, the folder it starts in, and variables over the app's own.
#[derive(Clone, Debug)]
pub struct Launch {
    /// None runs the login shell.
    pub program: Option<(String, Vec<String>)>,
    pub cwd: PathBuf,
    pub env: Vec<(String, String)>,
    /// Shell integration makes it report its folder.
    pub reports: bool,
    /// How its shell reads a path dropped on it.
    pub quote: Quote,
    /// Where its shell reads paths: this machine, Git Bash or a WSL distribution.
    pub target: Target,
}

/// A running program on a pseudo-terminal and the grid it draws into.
pub struct Pty {
    pub term: Arc<FairMutex<Term<Listener>>>,
    pub pid: u32,
    pub quote: Quote,
    pub target: Target,
    notifier: Notifier,
    #[cfg(unix)]
    master: std::fs::File,
}

/// What a pty reports, in order.
pub type Events = UnboundedReceiver<PtyEvent>;

#[cfg(unix)]
type Raw = super::unix::Pty;
#[cfg(windows)]
type Raw = tty::Pty;

/// The lines of a program's own output a failed start keeps, enough to say what went wrong.
pub const STDERR: usize = 8;

/// Why a session could not start: what was tried, in what folder, and what the program said. A start that fails used to say nothing a user could act on; this is what they see and what they can copy into an issue — the program and its arguments, the folder, the error, and the first lines the program wrote itself, where a shell says why it will not run.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Failed {
    /// The program and its arguments, as the settings or the scan named them; none runs the login shell.
    pub program: Option<(String, Vec<String>)>,
    /// The folder the session was to start in.
    pub cwd: PathBuf,
    /// What the system said; the error's own words, with its causes.
    pub error: Box<str>,
    /// The first lines the program wrote before it ended.
    pub said: Box<[String]>,
}

impl Failed {
    /// What the user sees, and can copy: every part on its own line.
    pub fn report(&self) -> String {
        let mut lines = Vec::new();
        match &self.program {
            Some((program, args)) if args.is_empty() => lines.push(format!("Program: {program}")),
            Some((program, args)) => lines.push(format!("Program: {program} {}", args.join(" "))),
            None => lines.push("Program: the login shell".into()),
        }
        if self.cwd.as_os_str().is_empty() {
            lines.push("Folder: none given".into());
        } else {
            lines.push(format!("Folder: {}", self.cwd.display()));
        }
        lines.push(format!("Error: {}", self.error));
        for line in &self.said {
            lines.push(line.clone());
        }
        lines.join("\n")
    }
}

/// `error` as a failed start in `cwd` of `program`, with the first lines `said` of its own output.
pub fn failed(
    program: Option<&(String, Vec<String>)>,
    cwd: PathBuf,
    error: &anyhow::Error,
    said: Vec<String>,
) -> Failed {
    Failed {
        program: program.cloned(),
        cwd,
        error: format!("{error:#}").into(),
        said: said.into_iter().take(STDERR).collect(),
    }
}
/// The first `STDERR` lines of `text` that say something, each without its controls and trimmed. A program that will not start writes why to the terminal it was given, and those are the lines worth keeping; controls and empty lines go, so the report reads as the user would read it.
pub fn said(text: &str) -> Vec<String> {
    text.lines()
        .map(|line| {
            line.chars()
                .filter(|&c| !c.is_control() || c == '\t')
                .collect::<String>()
                .trim()
                .to_owned()
        })
        .filter(|line| !line.is_empty())
        .take(STDERR)
        .collect()
}

/// The grid's options from the settings: scrollback, the cursor programs start from, kitty keys.
pub fn config(settings: &Settings) -> Config {
    let shape = match settings.cursor_style {
        CursorStyle::Block => CursorShape::Block,
        CursorStyle::Bar => CursorShape::Beam,
        CursorStyle::Underline => CursorShape::Underline,
        CursorStyle::Hollow => CursorShape::HollowBlock,
    };
    Config {
        scrolling_history: settings.scrollback as usize,
        default_cursor_style: alacritty_terminal::vte::ansi::CursorStyle {
            shape,
            blinking: settings.cursor_blink,
        },
        kitty_keyboard: true,
        ..Config::default()
    }
}

/// Starts `launch` on a pty of `columns` by `lines` cells, each `cell` pixels.
pub fn spawn(
    launch: &Launch,
    (columns, lines): (usize, usize),
    cell: (u16, u16),
    config: Config,
) -> anyhow::Result<(Pty, Events)> {
    let mut env = HashMap::from([
        ("TERM".to_string(), "xterm-256color".to_string()),
        ("COLORTERM".to_string(), "truecolor".to_string()),
    ]);
    if std::env::var_os("LANG").is_none() {
        env.insert("LANG".into(), "en_US.UTF-8".into());
    }
    env.extend(launch.env.iter().cloned());
    let size = fit(columns, lines, cell);
    let raw = start(launch, env, size).map_err(|error| {
        anyhow::anyhow!(
            "pty: cannot start {:?} in {}: {error}",
            launch.program,
            launch.cwd.display()
        )
    })?;
    let pid = child_id(&raw)?;
    #[cfg(unix)]
    let master = raw
        .master()
        .map_err(|error| anyhow::anyhow!("pty {pid}: no copy of the master: {error}"))?;
    log::info!(
        "pty {pid}: started {:?} in {}",
        launch.program,
        launch.cwd.display()
    );
    let (sender, events) = unbounded();
    let listener = Listener {
        sender: sender.clone(),
        pid,
    };
    let grid = TermSize::new(size.num_cols.into(), size.num_lines.into());
    let term = Arc::new(FairMutex::new(Term::new(config, &grid, listener.clone())));
    let event_loop = EventLoop::new(
        term.clone(),
        listener,
        Tapped::new(raw, sender, pid),
        true,
        false,
    )?;
    let notifier = Notifier(event_loop.channel());
    event_loop.spawn();
    Ok((
        Pty {
            term,
            pid,
            quote: launch.quote,
            target: launch.target.clone(),
            notifier,
            #[cfg(unix)]
            master,
        },
        events,
    ))
}

/// Starts `launch` as [`spawn`] does; on failure, why, in words the user can copy.
pub fn opened(launch: &Launch, size: (usize, usize), cell: (u16, u16), config: Config) -> Started {
    match spawn(launch, size, cell, config) {
        Ok(started) => Ok(started),
        Err(error) => Err(failed(
            launch.program.as_ref(),
            launch.cwd.clone(),
            &error,
            Vec::new(),
        )),
    }
}

/// A session's start: its pty and events, or why no session could start.
pub type Started = Result<(Pty, Events), Failed>;

#[cfg(unix)]
fn start(launch: &Launch, env: HashMap<String, String>, size: WindowSize) -> std::io::Result<Raw> {
    super::unix::spawn(launch.program.as_ref(), &launch.cwd, &env, size)
}

#[cfg(windows)]
fn start(launch: &Launch, env: HashMap<String, String>, size: WindowSize) -> std::io::Result<Raw> {
    let options = tty::Options {
        shell: launch
            .program
            .clone()
            .map(|(program, args)| tty::Shell::new(program, args)),
        working_directory: Some(launch.cwd.clone()),
        drain_on_exit: true,
        env,
        escape_args: true,
    };
    tty::new(&options, size, 0)
}

impl Pty {
    /// Sends bytes to the program, as typing does.
    pub fn write(&self, bytes: impl Into<Cow<'static, [u8]>>) {
        self.notifier.notify(bytes);
    }

    /// Fits the grid and tells the program its new size; both get the same counts.
    pub fn resize(&mut self, (columns, lines): (usize, usize), cell: (u16, u16)) {
        let size = fit(columns, lines, cell);
        self.notifier.on_resize(size);
        self.term
            .lock()
            .resize(TermSize::new(size.num_cols.into(), size.num_lines.into()));
    }
}

#[cfg(unix)]
impl Pty {
    /// The process group the program runs in the foreground.
    pub fn foreground(&self) -> std::io::Result<i32> {
        let group = rustix_openpty::rustix::termios::tcgetpgrp(&self.master)?;
        Ok(group.as_raw_nonzero().get())
    }
}

#[cfg(all(test, unix))]
impl Pty {
    /// Whether the program reads raw keys, canonical mode off.
    pub(crate) fn reads_keys(&self) -> bool {
        use rustix_openpty::rustix::termios::{LocalModes, tcgetattr};
        let modes = tcgetattr(&self.master).expect("the pty's modes");
        !modes.local_modes.contains(LocalModes::ICANON)
    }
}

impl Drop for Pty {
    fn drop(&mut self) {
        if self.notifier.0.send(Msg::Shutdown).is_err() {
            log::debug!("pty {}: the event loop had already stopped", self.pid);
        } else {
            log::info!("pty {}: closing", self.pid);
        }
    }
}

#[cfg(unix)]
fn child_id(pty: &Raw) -> anyhow::Result<u32> {
    Ok(pty.id())
}

#[cfg(windows)]
fn child_id(pty: &Raw) -> anyhow::Result<u32> {
    pty.child_watcher()
        .pid()
        .map(u32::from)
        .ok_or_else(|| anyhow::anyhow!("pty: Windows gave the program no id"))
}

/// A pty's size, at least 2 by 1; counts past what the kernel holds are cut and logged.
pub fn fit(columns: usize, lines: usize, cell: (u16, u16)) -> WindowSize {
    let cells = |count: usize, side: u16| {
        let most = (i16::MAX as u16).min(u16::MAX / side.max(1));
        let kept = u16::try_from(count).unwrap_or(u16::MAX).min(most);
        if usize::from(kept) < count {
            log::error!("pty: {count} cells of {side} px pass what a pty holds; {kept} kept");
        }
        kept
    };
    WindowSize {
        num_lines: cells(lines.max(1), cell.1),
        num_cols: cells(columns.max(2), cell.0),
        cell_width: cell.0,
        cell_height: cell.1,
    }
}

#[cfg(all(test, unix))]
mod tests;

#[cfg(test)]
mod failed_tests {
    use super::*;

    #[test]
    fn a_failed_start_says_what_it_tried_and_where() {
        let program = Some(("/bin/sh".to_string(), vec!["-l".to_string()]));
        let failed = failed(
            program.as_ref(),
            PathBuf::from("/home/u/src"),
            &anyhow::anyhow!("no such file"),
            Vec::new(),
        );
        assert_eq!(
            failed.report(),
            "Program: /bin/sh -l\nFolder: /home/u/src\nError: no such file"
        );
        let login = super::failed(None, PathBuf::new(), &anyhow::anyhow!("no shell"), vec![]);
        assert_eq!(
            login.report(),
            "Program: the login shell\nFolder: none given\nError: no shell"
        );
    }

    #[test]
    fn the_lines_a_program_wrote_come_with_the_report() {
        let lines = said("sh: \u{1b}[31mno such shell\u{1b}[0m\r\n\r\n  1. try again\n");
        assert_eq!(lines, ["sh: [31mno such shell[0m", "1. try again"]);
        let program = ("sh".to_string(), Vec::<String>::new());
        let failed = failed(
            Some(&program),
            PathBuf::from("/tmp"),
            &anyhow::anyhow!("it ended at once"),
            lines,
        );
        assert!(
            failed.report().ends_with("1. try again"),
            "{}",
            failed.report()
        );
        assert!(said("\r\n\n").is_empty(), "nothing said");
        let long: String = (1..=40).map(|at| format!("{at}\n")).collect();
        assert_eq!(said(&long).len(), STDERR, "the first {STDERR} lines");
    }
}

#[cfg(test)]
mod scrollback {
    use alacritty_terminal::{event::VoidListener, grid::Dimensions, vte::ansi::Processor};

    use super::*;

    #[test]
    fn the_setting_caps_history_and_a_change_trims_it() {
        let settings = Settings {
            scrollback: 5,
            ..Settings::default()
        };
        let mut term = Term::new(config(&settings), &TermSize::new(10, 3), VoidListener);
        let mut parser: Processor = Processor::new();
        let lines: String = (1..=20).map(|line| format!("{line}\r\n")).collect();
        parser.advance(&mut term, lines.as_bytes());
        assert_eq!(term.grid().history_size(), 5);
        term.set_options(config(&Settings {
            scrollback: 2,
            ..settings
        }));
        assert_eq!(term.grid().history_size(), 2);
    }
}
