//! What a program may put on the clipboard through OSC 52, and what it may not: a program writes without asking, so the terminal is the only place that can say no.

/// The longest text a program may put on the clipboard, in bytes; past it the write is refused.
pub const MOST: usize = 100 * 1024;

/// `text` as a program's OSC 52 write leaves it: the text itself when it fits, and none when it does not.
pub fn keep(text: String) -> Option<String> {
    if text.len() <= MOST {
        return Some(text);
    }
    log::warn!(
        "clipboard: a program asked to write {} bytes, past the {MOST} kept, so it is refused",
        text.len()
    );
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_write_that_fits_is_kept_and_a_long_one_is_refused() {
        assert_eq!(keep("a".into()).as_deref(), Some("a"));
        assert_eq!(
            keep("".into()).as_deref(),
            Some(""),
            "an empty write clears it"
        );
        let edge = "a".repeat(MOST);
        assert_eq!(
            keep(edge).map(|text| text.len()),
            Some(MOST),
            "the limit itself fits"
        );
        assert_eq!(
            keep("a".repeat(MOST + 1)),
            None,
            "one byte past it does not"
        );
        assert_eq!(
            keep("\u{4e2d}".repeat(MOST / 3 + 1)),
            None,
            "the limit counts bytes, as a program writes them"
        );
    }
}
