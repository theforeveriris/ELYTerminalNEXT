//! What a program asks the terminal about itself, and what it hears back.

use alacritty_terminal::term::TermMode;

use super::mouse::{Format, MouseMode, Tracking};

/// A query the terminal answers: `CSI c`, `CSI > 0 q` or `CSI ? Ps $ p`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Query {
    /// `CSI c`: the primary device attributes.
    Primary,
    /// `CSI > 0 q`: xterm's version, XTVERSION.
    Version,
    /// `CSI ? Ps $ p`: what mode `Ps` is in, DECRQM.
    Mode(u32),
}

/// The name XTVERSION answers with.
pub const NAME: &str = "ELYTerminal";

/// The parameters DA1 answers with: 62 for VT220, 22 for ANSI colour.
///
/// Sixel is 4 and ReGIS is 3, and neither is here: the terminal draws no graphics of its own, so a
/// program that asks never tries to send one.
const PRIMARY: &str = "62;22";

/// What DECRQM says of a mode it knows but never sets: 4, permanently reset.
const NEVER: u8 = 4;

/// What DECRQM says of a mode it knows nothing of: 0.
const UNKNOWN: u8 = 0;

impl Query {
    /// The bytes the terminal answers `self` with, from the grid's modes and the mouse's.
    pub fn answer(self, term: TermMode, mouse: MouseMode) -> String {
        match self {
            Self::Primary => format!("\x1b[?{PRIMARY}c"),
            Self::Version => format!("\x1bP>|{NAME}({})\x1b\\", env!("CARGO_PKG_VERSION")),
            Self::Mode(mode) => format!("\x1b[?{mode};{}$y", state(mode, term, mouse)),
        }
    }
}

/// `CSI > 0 q` in the output, which vte dispatches to no handler, so the tap looks for it itself.
#[derive(Default)]
pub struct Scan {
    state: State,
    first: Option<u32>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum State {
    #[default]
    Ground,
    Escape,
    /// After `CSI`, where `>` may still come.
    Bracket,
    /// After `CSI >`.
    Second,
}

impl Scan {
    /// The version queries in `bytes`, in order.
    pub fn feed(&mut self, bytes: &[u8], found: &mut impl FnMut(Query)) {
        let mut rest = bytes;
        while !rest.is_empty() {
            if self.state == State::Ground {
                let Some(at) = rest.iter().position(|&byte| byte == 0x1b) else {
                    return;
                };
                self.state = State::Escape;
                rest = &rest[at + 1..];
                continue;
            }
            let byte = rest[0];
            rest = &rest[1..];
            self.state = match (self.state, byte) {
                (_, 0x18 | 0x1a) => State::Ground,
                (_, 0x1b) => State::Escape,
                // Mid-sequence, C0 runs and DEL is skipped, as in vte.
                (state, 0x00..=0x1f | 0x7f..=0xff) => state,
                (State::Escape, b'[') => {
                    self.first = Some(0);
                    State::Bracket
                }
                (State::Bracket, b'>') => {
                    self.first = Some(0);
                    State::Second
                }
                (State::Bracket, _) => State::Ground,
                (State::Second, b'0'..=b'9') => {
                    self.first = self.first.map(|first| {
                        first
                            .saturating_mul(10)
                            .saturating_add(u32::from(byte - b'0'))
                    });
                    State::Second
                }
                (State::Second, b';') => State::Second,
                (State::Second, b'q') => {
                    if self.first == Some(0) {
                        found(Query::Version);
                    }
                    State::Ground
                }
                (State::Second, _) => State::Ground,
                (State::Escape, _) => State::Ground,
                (State::Ground, _) => unreachable!("ground scans by position"),
            };
        }
    }
}

/// What DECRQM says of mode `mode`: 1 set, 2 reset, 4 one never set, 0 one unknown to it.
fn state(mode: u32, term: TermMode, mouse: MouseMode) -> u8 {
    let said = |set: bool| u8::from(!set) + 1;
    match mode {
        4 => said(term.contains(TermMode::INSERT)),
        7 => said(term.contains(TermMode::LINE_WRAP)),
        9 => said(mouse.tracking == Tracking::X10),
        20 => said(term.contains(TermMode::LINE_FEED_NEW_LINE)),
        25 => said(term.contains(TermMode::SHOW_CURSOR)),
        1000 => said(mouse.tracking == Tracking::Normal),
        1002 => said(mouse.tracking == Tracking::Button),
        1003 => said(mouse.tracking == Tracking::Any),
        1004 => said(term.contains(TermMode::FOCUS_IN_OUT)),
        1005 => said(mouse.format == Format::Utf8),
        1006 => said(mouse.format == Format::Sgr),
        1015 => said(mouse.format == Format::Urxvt),
        1016 => said(mouse.format == Format::SgrPixels),
        1049 => said(term.contains(TermMode::ALT_SCREEN)),
        2004 => said(term.contains(TermMode::BRACKETED_PASTE)),
        // Mode 9001 is kitty's keyboard; its disambiguation is the part of it DECRQM can see.
        9001 => said(term.contains(TermMode::DISAMBIGUATE_ESC_CODES)),
        // Grapheme clusters, an in-band resize and a synchronized update: known, and none of ours.
        2026 | 2027 | 2048 => NEVER,
        _ => UNKNOWN,
    }
}

/// The reply to `CSI c`, `CSI ? Ps $ p` or `CSI > 0 q` the grid handed `reply`, honestly said.
///
/// The grid answers a device attributes query as VT102, and a mode query from a table that holds
/// almost nothing of what this terminal does; both would have a program take it for less than it is.
/// Their replies are the only `?…c` and `…$y` the grid writes, so each is read back and answered again.
pub fn honest(reply: &str, term: TermMode, mouse: MouseMode) -> Option<String> {
    let rest = reply.strip_prefix("\x1b[?")?;
    let query = match rest.strip_suffix('c') {
        // A secondary device attributes reply, `CSI > 0 ; version ; 1 c`, says no more than ours does.
        Some(_) if rest.starts_with('>') => return None,
        Some(_) => Query::Primary,
        None => {
            let (mode, _) = rest.strip_suffix("$y")?.split_once(';')?;
            Query::Mode(mode.parse().ok()?)
        }
    };
    Some(query.answer(term, mouse))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mouse(tracking: Tracking) -> MouseMode {
        MouseMode {
            tracking,
            ..MouseMode::default()
        }
    }

    /// What the scanner finds in `chunks`.
    fn scan(chunks: &[&[u8]]) -> Vec<Query> {
        let mut scanner = Scan::default();
        let mut found = Vec::new();
        for chunk in chunks {
            scanner.feed(chunk, &mut |query| found.push(query));
        }
        found
    }

    #[test]
    fn da1_names_what_the_terminal_is_and_keeps_what_it_is_not() {
        let answer = Query::Primary.answer(TermMode::empty(), MouseMode::default());
        assert!(
            answer.starts_with("\x1b[?") && answer.ends_with('c'),
            "{answer:?}"
        );
        let params: Vec<&str> = answer[3..answer.len() - 1].split(';').collect();
        assert!(params.contains(&"62"), "VT220: {params:?}");
        assert!(params.contains(&"22"), "ANSI colour: {params:?}");
        assert!(!params.contains(&"4"), "no sixel: {params:?}");
        assert!(!params.contains(&"3"), "no ReGIS: {params:?}");
    }

    #[test]
    fn xtversion_names_the_terminal_and_its_version() {
        let answer = Query::Version.answer(TermMode::empty(), MouseMode::default());
        assert_eq!(
            answer,
            format!("\x1bP>|ELYTerminal({})\x1b\\", env!("CARGO_PKG_VERSION"))
        );
    }

    #[test]
    fn decrqm_says_what_each_mode_is_in() {
        let empty = TermMode::empty();
        let plain = MouseMode::default();
        let said = |mode: u32| Query::Mode(mode).answer(empty, plain);
        assert_eq!(said(2027), "\x1b[?2027;4$y", "no grapheme clusters");
        assert_eq!(said(2048), "\x1b[?2048;4$y", "no in-band resize");
        assert_eq!(said(2026), "\x1b[?2026;4$y", "no synchronized update");
        assert_eq!(said(1234), "\x1b[?1234;0$y", "a mode it knows nothing of");
        assert_eq!(
            Query::Mode(25).answer(TermMode::SHOW_CURSOR, plain),
            "\x1b[?25;1$y"
        );
        assert_eq!(said(25), "\x1b[?25;2$y", "the cursor is hidden");
        let clicked = |tracking: Tracking| Query::Mode(1000).answer(empty, mouse(tracking));
        assert_eq!(clicked(Tracking::Normal), "\x1b[?1000;1$y");
        assert_eq!(
            clicked(Tracking::Any),
            "\x1b[?1000;2$y",
            "1000 is off while 1003 is on"
        );
        assert_eq!(
            Query::Mode(1049).answer(TermMode::ALT_SCREEN, plain),
            "\x1b[?1049;1$y"
        );
        assert_eq!(
            Query::Mode(2004).answer(TermMode::BRACKETED_PASTE, plain),
            "\x1b[?2004;1$y"
        );
        assert_eq!(
            Query::Mode(9001).answer(TermMode::DISAMBIGUATE_ESC_CODES, plain),
            "\x1b[?9001;1$y"
        );
        assert_eq!(
            Query::Mode(1016).answer(
                empty,
                MouseMode {
                    format: Format::SgrPixels,
                    ..MouseMode::default()
                }
            ),
            "\x1b[?1016;1$y"
        );
    }

    #[test]
    fn a_version_query_is_found_across_reads_and_others_are_not() {
        assert_eq!(scan(&[b"\x1b[>0", b"q"]), [Query::Version]);
        assert_eq!(
            scan(&[b"\x1b[>q\x1b[>0;q"]),
            [Query::Version; 2],
            "Ps defaults to 0"
        );
        assert_eq!(
            scan(&[b"\x1b[>0\x00q"]),
            [Query::Version],
            "a control runs inside it"
        );
        assert!(
            scan(&[b"\x1b[>1q\x1b[0q\x1b[q"]).is_empty(),
            "only 0, and only after >"
        );
        assert!(scan(&[b"\x1b[>0\x18q"]).is_empty(), "CAN cancels");
        assert!(scan(&[b"a\x1b[>0"]).is_empty(), "an unfinished query");
    }

    #[test]
    fn the_grid_s_replies_are_answered_again_honestly() {
        let empty = TermMode::empty();
        let plain = MouseMode::default();
        assert_eq!(
            honest("\x1b[?6c", empty, plain).as_deref(),
            Some("\x1b[?62;22c"),
            "VT102 becomes what it is"
        );
        assert_eq!(
            honest("\x1b[?2026;0$y", empty, plain).as_deref(),
            Some("\x1b[?2026;4$y"),
            "a mode the grid's table holds nothing of"
        );
        assert_eq!(
            honest("\x1b[?1000;0$y", empty, mouse(Tracking::Normal)).as_deref(),
            Some("\x1b[?1000;1$y")
        );
        assert_eq!(
            honest("\x1b[>0;1;1c", empty, plain),
            None,
            "a secondary DA stands"
        );
        assert_eq!(
            honest("\x1b[?25;1$y", TermMode::SHOW_CURSOR, plain).as_deref(),
            Some("\x1b[?25;1$y")
        );
        for other in ["\x1b[?6n", "\x1b[31m", "\x1b[?x;1$y", ""] {
            assert_eq!(
                honest(other, empty, plain),
                None,
                "{other:?} is no reply of theirs"
            );
        }
    }
}
