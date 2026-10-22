/// A semantic prompt mark a program sends, OSC 133.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    /// A prompt starts: A, or N, saying how its line takes a click.
    Prompt(Click),
    /// The prompt ends and input starts: B, or I.
    Input,
    /// Input ends and the command's output starts: C.
    Output,
    /// The command ended, with the status it gave: D.
    Done(Option<i32>),
}

/// How the line a prompt reads takes a click, as its start's `click_events` says.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Click {
    /// Arrows the terminal counts.
    Arrows,
    /// A mouse press at the click's place on the screen, `click_events=1`.
    Absolute,
    /// A mouse press, its row counted from the prompt's first, `click_events=2`.
    Relative,
}

/// The mark in an OSC 133 body past `133;`; none for one the terminal takes no action on.
///
/// VS Code's shell integration writes the same marks under OSC 633, so both read alike. The options
/// a mark may carry 鈥?`aid` to pair a prompt with its command, `cl` and `cmdline_url` describing the
/// line, `k` naming which prompt, `err` explaining a failure 鈥?are read past: what the terminal acts
/// on is the letter and, for `D`, the status.
pub(crate) fn read(body: &[u8]) -> Option<Mark> {
    let shown = || String::from_utf8_lossy(body);
    let Some((&letter, rest)) = body.split_first() else {
        log::warn!("pty: OSC 133 without a mark");
        return None;
    };
    let options = match rest {
        [] => rest,
        [b';', options @ ..] => options,
        _ => {
            log::warn!("pty: OSC 133;{} is no mark", shown());
            return None;
        }
    };
    match letter {
        b'A' | b'N' => Some(Mark::Prompt(click(options))),
        b'B' | b'I' => Some(Mark::Input),
        b'C' => Some(Mark::Output),
        b'D' => Some(Mark::Done(status(options))),
        // L's fresh line needs the cursor, unseen before the parser; P adds nothing to A.
        b'L' | b'P' => {
            log::debug!("pty: OSC 133;{} takes no action here", shown());
            None
        }
        _ => {
            log::warn!("pty: OSC 133;{} is no mark", shown());
            None
        }
    }
}

/// How A's options say its line takes a click; arrows without `click_events`.
fn click(options: &[u8]) -> Click {
    let value = options
        .split(|&byte| byte == b';')
        .find_map(|option| option.strip_prefix(b"click_events="));
    match value {
        None | Some(b"0") => Click::Arrows,
        Some(b"1") => Click::Absolute,
        Some(b"2") => Click::Relative,
        Some(other) => {
            log::warn!(
                "pty: OSC 133;A with click_events={}, no such mode, so arrows",
                String::from_utf8_lossy(other)
            );
            Click::Arrows
        }
    }
}

/// D's status, its first option; none when the program gave none or no number.
fn status(options: &[u8]) -> Option<i32> {
    let first = options.split(|&byte| byte == b';').next()?;
    if first.is_empty() {
        return None;
    }
    let status = std::str::from_utf8(first)
        .ok()
        .and_then(|text| text.parse().ok());
    if status.is_none() {
        log::warn!(
            "pty: OSC 133;D with a status that is no number ({})",
            String::from_utf8_lossy(first)
        );
    }
    status
}

/// The command id an OSC 133 `aid` names, so a prompt and its command can be paired.
pub(crate) fn aid(options: &[u8]) -> Option<&str> {
    options.split(|&byte| byte == b';').find_map(|option| {
        let rest = option.strip_prefix(b"aid=")?;
        std::str::from_utf8(rest).ok().filter(|id| !id.is_empty())
    })
}

/// Whether a `D` carries the shell's `err`, which says why a command failed in its own words.
pub(crate) fn failed_with(options: &[u8]) -> Option<&str> {
    options.split(|&byte| byte == b';').find_map(|option| {
        let rest = option.strip_prefix(b"err=")?;
        std::str::from_utf8(rest)
            .ok()
            .filter(|line| !line.is_empty())
    })
}

/// The command id and error an OSC 133 mark carries, for a session that pairs a prompt with its
/// command and keeps why it failed: `Some` for a D with `err`, and the `aid` both marks may name.
pub(crate) fn options(body: &[u8]) -> (Option<String>, Option<String>) {
    let Some((&letter, rest)) = body.split_first() else {
        return (None, None);
    };
    let options = rest
        .strip_prefix(b";")
        .filter(|_| letter == b'A' || letter == b'D');
    match (letter, options) {
        (b'A' | b'D', Some(options)) => (
            aid(options).map(str::to_owned),
            (letter == b'D')
                .then(|| failed_with(options).map(str::to_owned))
                .flatten(),
        ),
        _ => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_mark_reads_with_its_options_as_ghostty_reads_it() {
        assert_eq!(read(b"A"), Some(Mark::Prompt(Click::Arrows)));
        assert_eq!(
            read(b"A;click_events=1"),
            Some(Mark::Prompt(Click::Absolute))
        );
        assert_eq!(
            read(b"A;aid=3;click_events=2"),
            Some(Mark::Prompt(Click::Relative))
        );
        assert_eq!(read(b"A;click_events=0"), Some(Mark::Prompt(Click::Arrows)));
        assert_eq!(read(b"A;cl=line"), Some(Mark::Prompt(Click::Arrows)));
        assert_eq!(read(b"N;aid=7"), Some(Mark::Prompt(Click::Arrows)));
        assert_eq!(read(b"B"), Some(Mark::Input));
        assert_eq!(read(b"I"), Some(Mark::Input));
        assert_eq!(read(b"C;cmdline_url=false"), Some(Mark::Output));
        assert_eq!(read(b"D;1"), Some(Mark::Done(Some(1))));
        assert_eq!(read(b"D;127;aid=foo"), Some(Mark::Done(Some(127))));
        assert_eq!(read(b"D;-1"), Some(Mark::Done(Some(-1))));
        assert_eq!(read(b"D"), Some(Mark::Done(None)));
        assert_eq!(read(b"D;"), Some(Mark::Done(None)));
        assert_eq!(read(b"D;abc"), Some(Mark::Done(None)));
    }

    #[test]
    fn a_mark_the_terminal_takes_no_action_on_reads_as_none() {
        for body in [&b""[..], b"L", b"P;k=s", b"AB", b"Q", b"a"] {
            assert_eq!(read(body), None, "{}", String::from_utf8_lossy(body));
        }
    }

    #[test]
    fn a_mark_s_aid_and_its_error_are_read_for_the_session() {
        assert_eq!(
            options(b"A;aid=3;cl=line"),
            (Some("3".into()), None),
            "a prompt names its command"
        );
        assert_eq!(
            options(b"D;2;aid=3;err=no such file"),
            (Some("3".into()), Some("no such file".into()))
        );
        assert_eq!(options(b"D;0"), (None, None), "no aid, no error");
        assert_eq!(options(b"B;aid=3"), (None, None), "only A and D carry them");
        assert_eq!(options(b""), (None, None));
    }

    #[test]
    fn the_options_a_mark_carries_are_read_past_and_the_ones_it_needs_are_kept() {
        assert_eq!(aid(b"aid=12;cl=line"), Some("12"));
        assert_eq!(aid(b"cl=line"), None, "no aid");
        assert_eq!(aid(b"aid="), None, "an empty one names nothing");
        assert_eq!(aid(b"aid=\xff"), None, "not UTF-8");
        assert_eq!(failed_with(b"2;err=no such file"), Some("no such file"));
        assert_eq!(failed_with(b"2;err="), None);
        assert_eq!(failed_with(b"2"), None);
        // Every option the shell integrations write is tolerated, and the mark still reads.
        assert_eq!(
            read(b"A;aid=3;cl=line;cmdline_url=false"),
            Some(Mark::Prompt(Click::Arrows))
        );
        assert_eq!(read(b"D;2;aid=3;err=it failed"), Some(Mark::Done(Some(2))));
        assert_eq!(read(b"C;cmdline_url=false"), Some(Mark::Output));
    }
}
