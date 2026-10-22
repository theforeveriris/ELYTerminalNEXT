//! What a pasted text leaves out, and lines it holds.

/// Pasted text longer than this is shown before it goes.
pub(super) const LONG: usize = 4000;

/// Text pasted as a program reads it: line breaks as a terminal sends them.
pub(super) fn typed(text: &str) -> String {
    text.replace("\r\n", "\r").replace('\n', "\r")
}

/// How many lines `text` becomes when a terminal sends it; a text of blanks runs nothing.
pub(super) fn lines(text: &str) -> usize {
    if text.trim().is_empty() {
        return 0;
    }
    typed(text).matches('\r').count() + 1
}

/// Whether `text` holds a control character worth a look before it goes, or runs past `long`.
pub(super) fn worth_a_look(text: &str, long: usize) -> bool {
    text.chars()
        .any(|ch| ch.is_control() && !matches!(ch, '\r' | '\n' | '\t'))
        || text.chars().count() > long
}

/// `text` with each control character shown as an escape and each line break as a return arrow, cut at `most`.
pub(super) fn preview(text: &str, most: usize) -> String {
    let mut out = String::new();
    for ch in text.chars().take(most) {
        match ch {
            '\r' => {}
            '\n' => out.push_str(" \u{23ce}\n"),
            '\t' => out.push_str("    "),
            ch if ch.is_control() => out.push_str(&format!("\\x{:02x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    if text.chars().count() > most {
        out.push('\u{2026}');
    }
    out
}
