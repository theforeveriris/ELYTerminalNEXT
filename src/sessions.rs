use std::{
    path::{MAIN_SEPARATOR, Path, PathBuf},
    time::Instant,
};

use ely_gpui_component::{
    buttons::{ButtonVariant, IconButton},
    forms::{Input, TextInput},
    lists::ListItem,
    primitives::{Icon, IconName, Tooltip},
    theme::{ActiveTheme, ControlSize, IconSize},
};
use gpui::{AnyElement, App, Entity, SharedString, Subscription, div, prelude::*, rems};

use crate::{
    term::{Block, Kind, Record, Records, State, Target, TerminalView},
    workspace::{TITLE, Workspace},
};

pub mod orb;

use orb::Look;

/// A program on its own terminal, as the sessions column lists it.
pub struct Session {
    /// Its id, which stays the same as it moves between tabs and closes.
    pub id: u64,
    pub terminal: Entity<TerminalView>,
    /// The program's title; empty while it names none.
    pub title: String,
    /// The name the user gave it; empty leaves the title to its program.
    pub name: String,
    /// The field its row shows while the user renames it.
    pub editing: Option<(Entity<TextInput>, Subscription)>,
    /// The folder it reports, else the one it started in.
    pub folder: PathBuf,
    /// Its shell has integration, or it reported a folder anyway.
    pub reports: bool,
    /// How it was started, which a clone of it runs again.
    pub launch: crate::term::Launch,
    /// A bell rang while it was hidden.
    pub bell: bool,
    /// It started waiting on the user while hidden.
    pub waiting: bool,
    /// The look its mark shows, and since when.
    pub look: Option<(Look, Instant)>,
    pub _events: Subscription,
}

impl Session {
    /// What it is called: the user's name, else its program's title, else the app's name.
    pub fn named(&self) -> &str {
        [self.name.as_str(), self.title.as_str()]
            .into_iter()
            .find(|title| !title.is_empty())
            .unwrap_or(TITLE)
    }

    /// The look its mark takes from its terminal now.
    pub fn looks(&self, cx: &App) -> Option<Look> {
        let view = self.terminal.read(cx);
        let failed = view.ended().is_some_and(|code| code != 0);
        look(view.records().shown().map(|(_, record)| record), failed)
    }

    /// Notes the look its mark takes `now`; true while the mark moves.
    pub fn note(&mut self, now: Instant, cx: &App) -> bool {
        let shows = self.looks(cx);
        if self.look.map(|(shown, _)| shown) != shows {
            self.look = shows.map(|shows| (shows, now));
        }
        self.look
            .is_some_and(|(look, since)| orb::moves(look, now.saturating_duration_since(since)))
    }
}

/// The most records hover lists, and the most characters it shows of each part.
const LISTED: usize = 10;
const SHORT: usize = 120;

/// Whether a session named `name` in `folder` answers `query`, in lower case; an empty query answers all.
pub fn matches(query: &str, name: &str, folder: &str) -> bool {
    query.is_empty() || name.to_lowercase().contains(query) || folder.to_lowercase().contains(query)
}

/// `folder` as a row shows it: the home folder as `~`, on one line.
pub fn shown(folder: &Path, home: Option<&Path>) -> String {
    let shown = match home.and_then(|home| folder.strip_prefix(home).ok()) {
        Some(rest) if rest.as_os_str().is_empty() => "~".into(),
        Some(rest) => format!("~{MAIN_SEPARATOR}{}", rest.display()),
        None => folder.display().to_string(),
    };
    one_line(&shown)
}

/// `text` on one line, as gpui shapes labels: each control character as its picture, `␊` for a newline.
pub fn one_line(text: &str) -> String {
    text.chars()
        .map(|c| match c {
            '\0'..='\x1f' => char::from_u32(0x2400 + c as u32).expect("a control picture"),
            '\x7f' => '\u{2421}',
            c => c,
        })
        .collect()
}

/// What a row's description adds for a block: how long it ran and, when it failed, its status.
pub fn block_note(block: &Block) -> Option<String> {
    let mut note = String::new();
    if let Some(duration) = block.duration() {
        note.push_str(&format!(" · {:.1}s", duration.as_secs_f64()));
    }
    if let Some(status) = block.status().filter(|status| *status != 0) {
        note.push_str(&format!(" · exit {status}"));
    }
    (!note.is_empty()).then_some(note)
}

/// The session shown once session `closed` of `left + 1` is gone, `active` having been shown.
pub fn after_close(active: usize, closed: usize, left: usize) -> usize {
    if closed < active || active == left {
        active - 1
    } else {
        active
    }
}

/// Session `id`'s row: its title and folder, the mark of its status, a bell while one rang unseen, a button that closes it.
pub fn row(
    workspace: &Entity<Workspace>,
    id: u64,
    session: &Session,
    active: bool,
    home: Option<&Path>,
    now: Instant,
    cx: &App,
) -> ListItem {
    let terminal = session.terminal.read(cx);
    let pid = terminal.pid() as usize;
    let mut description = shown(&session.folder, home);
    // CONN-18: which machine or distribution the session sits on, when it is not this one.
    if let Some(where_) = connection(&session.launch) {
        description = format!("{where_} \u{b7} {description}");
    }
    if let Some(note) = terminal.blocks().last_finished().and_then(block_note) {
        description.push_str(&note);
    }
    let (select, close) = (workspace.clone(), workspace.clone());
    let editing = session.editing.is_some();
    let item = ListItem::new(("session", pid), session.named().to_string())
        .description(description)
        .selected(active)
        .strong(session.bell || session.waiting)
        // A press in the field it renames with stays the field's.
        .on_click(
            move |event, window, cx| match (editing, event.click_count()) {
                (true, _) => {}
                (false, 2..) => select.update(cx, |this, cx| this.rename(id, window, cx)),
                (false, _) => select.update(cx, |this, cx| this.select(id, window, cx)),
            },
        )
        .trailing(
            IconButton::new(("close-session", pid), IconName::X)
                .variant(ButtonVariant::Ghost)
                .size(ControlSize::Sm)
                .tooltip(crate::settings::language::message(
                    cx,
                    "workspace.close_session",
                ))
                .on_click(move |_, window, cx| {
                    cx.stop_propagation();
                    close.update(cx, |this, cx| this.request_close(id, window, cx))
                }),
        );
    let item = match &session.editing {
        Some((input, _)) => item.editor(editor(workspace, session, input)),
        None => item,
    };
    let mark = mark(session, now, cx);
    let bell = session
        .bell
        .then(|| Icon::new(IconName::Bell).color(cx.theme().colors.warning));
    if mark.is_none() && bell.is_none() {
        return item;
    }
    item.leading(
        div()
            .flex()
            .items_center()
            .gap_1()
            .children(mark)
            .children(bell),
    )
}

/// Where a session sits, when it is not this machine: its WSL distribution, or the host it sshs to. The launch says how the session started, so a WSL shell names its distribution and an ssh one the host in its arguments, rather than one guessed from output.
pub fn connection(launch: &crate::term::Launch) -> Option<String> {
    match &launch.target {
        Target::Wsl { distro } if distro != "default" => Some(format!("WSL: {distro}")),
        Target::Wsl { .. } => Some("WSL".into()),
        _ => {
            let (program, args) = launch.program.as_ref()?;
            let name = std::path::Path::new(program).file_stem()?.to_string_lossy();
            if !name.eq_ignore_ascii_case("ssh") {
                return None;
            }
            let host = args.iter().rev().find(|arg| !arg.starts_with('-'))?;
            Some(format!("ssh: {host}"))
        }
    }
}

/// The field a session's row renames it with; Escape drops what it holds.
fn editor(
    workspace: &Entity<Workspace>,
    session: &Session,
    input: &Entity<TextInput>,
) -> impl IntoElement {
    let (workspace, id) = (workspace.clone(), session.terminal.entity_id());
    div()
        .on_key_down(move |event, window, cx| {
            if event.keystroke.key == "escape" {
                cx.stop_propagation();
                workspace.update(cx, |this, cx| this.end_rename(id, false, window, cx));
            }
        })
        .child(Input::new(input).size(ControlSize::Sm))
}

/// What a mark shows: the most pressing record's state, else a failed last command.
pub fn look(record: Option<&Record>, failed: bool) -> Option<Look> {
    Some(
        match (record.map(|record| (record.state, record.progress)), failed) {
            (Some((State::Blocked, _)), _) => Look::Waiting,
            (Some((State::Error, _)), _) => Look::Error,
            (Some((State::Working, progress)), _) => Look::Working(progress),
            (Some((State::Done, _)), _) => Look::Done,
            (_, true) => Look::Error,
            (Some((State::Idle, _)), false) => Look::Idle,
            (None, false) => return None,
        },
    )
}

/// The session's mark: its look's dots in its state's color; hover lists the records.
fn mark(session: &Session, now: Instant, cx: &App) -> Option<AnyElement> {
    let (look, since) = session.look?;
    let theme = cx.theme();
    let colors = &theme.colors;
    let color = match look {
        Look::Working(_) => colors.info,
        Look::Waiting => colors.warning,
        Look::Error => colors.danger,
        Look::Done => colors.success,
        Look::Idle => colors.fg_muted,
    };
    let pid = session.terminal.read(cx).pid() as usize;
    // Hover reads the records as they stand, so an open one follows them, and scrolls within the window.
    let watched = session.terminal.downgrade();
    let tip = Tooltip::rich(move |window, cx| {
        let lines = watched.upgrade().map_or_else(Vec::new, |terminal| {
            let view = terminal.read(cx);
            let failed: Vec<&str> = view
                .blocks()
                .failures()
                .into_iter()
                .map(|(_, command, _)| command)
                .collect();
            hover(
                view.records(),
                view.ended().filter(|&code| code != 0),
                &failed,
            )
        });
        let room = window.viewport_size().height - rems(3.).to_pixels(window.rem_size());
        div()
            .id("session-hover")
            .max_h(room)
            .overflow_y_scroll()
            .flex()
            .flex_col()
            .children(lines)
            .into_any_element()
    });
    // Every mark takes a 24-pixel box, so the title stays put as the state changes.
    Some(
        div()
            .id(("session-status", pid))
            .size(theme.icon_size(IconSize::Xl))
            .flex()
            .items_center()
            .justify_center()
            .hoverable_tooltip(tip)
            .child(orb::lattice(look, color, now, since))
            .into_any_element(),
    )
}

/// A session's hover: each record it holds, at most `LISTED`, then how many more, then what its last command and its failures came to.
pub fn hover(records: &Records, failed: Option<i32>, failures: &[&str]) -> Vec<SharedString> {
    let mut lines: Vec<SharedString> = records
        .iter()
        .take(LISTED)
        .map(|(id, record)| line(records, id, record).into())
        .collect();
    let more = records.iter().count().saturating_sub(LISTED);
    if more > 0 {
        lines.push(format!("{more} more").into());
    }
    if let Some(code) = failed {
        lines.push(format!("Last command exited with {code}").into());
    }
    // STAT-19: the commands that failed in this session, newest last, so a row says how many.
    match failures.len() {
        0 => {}
        1 => lines.push(format!("1 command failed: {}", failures[0]).into()),
        count => {
            lines.push(format!("{count} commands failed").into());
            for command in failures.iter().take(LISTED) {
                lines.push(format!("  {command}").into());
            }
        }
    }
    lines
}

/// A record as hover and notifications say it: its state, then its app, title and message.
pub fn line(records: &Records, id: &str, record: &Record) -> String {
    let parts = [
        records.app(id),
        record.title.as_deref(),
        record.msg.as_deref(),
    ];
    let said: Vec<String> = parts.into_iter().flatten().map(plain).collect();
    let state = label(record);
    if said.is_empty() {
        state
    } else {
        format!("{state}: {}", said.join(" · "))
    }
}

/// A record's state, as hover names it.
fn label(record: &Record) -> String {
    match (record.state, record.kind, record.progress) {
        (State::Idle, ..) => "Idle".into(),
        (State::Working, _, Some(value)) => format!("Working, {value}%"),
        (State::Working, ..) => "Working".into(),
        (State::Blocked, Some(Kind::Permission), _) => "Waiting for permission".into(),
        (State::Blocked, Some(Kind::Question), _) => "Waiting for an answer".into(),
        (State::Blocked, Some(Kind::Auth), _) => "Waiting for a login".into(),
        (State::Blocked, None, _) => "Waiting".into(),
        (State::Done, ..) => "Done".into(),
        (State::Error, ..) => "Error".into(),
    }
}

/// A program's text as hover shows it: on one line, 120 characters at most, without what could disguise it.
pub fn plain(text: &str) -> String {
    let kept: String = text.chars().filter(|&c| !invisible(c)).collect();
    let line = one_line(&kept);
    match line.char_indices().nth(SHORT) {
        Some((at, _)) => format!("{}…", &line[..at]),
        None => line,
    }
}

/// C1 controls, direction controls and invisible formatting; joiners stay, as scripts and emoji need them.
pub fn invisible(c: char) -> bool {
    matches!(
        c,
        '\u{80}'..='\u{9f}'
            | '\u{ad}'
            | '\u{61c}'
            | '\u{180e}'
            | '\u{200b}'
            | '\u{200e}'
            | '\u{200f}'
            | '\u{202a}'..='\u{202e}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{2069}'
            | '\u{feff}'
            | '\u{fff9}'..='\u{fffb}'
            | '\u{e0000}'..='\u{e007f}'
    )
}

#[cfg(test)]
mod tests;
