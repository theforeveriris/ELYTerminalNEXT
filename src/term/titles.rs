/// The titles alacritty saves before it drops the oldest.
const SAVED: usize = 4096;

/// vte keeps 16 parameters of an OSC, its number among them.
const PARTS: usize = 15;

/// The longest OSC 0 or 2 body the tap keeps; a longer one drops, logged.
pub(super) const LONGEST: usize = 65536;

/// The longest title the terminal shows; a longer one is cut, as a window's title has no room for it.
pub const SHOWN: usize = 1024;

/// The title a session shows, read in the output's order as vte and alacritty read it.
#[derive(Default)]
pub(crate) struct Titles {
    /// Empty while the program names none.
    title: String,
    saved: Vec<String>,
}

impl Titles {
    /// Sets the title from an OSC 0 or 2 body past its number, as vte splits it and alacritty joins it.
    pub fn set(&mut self, rest: &[u8]) -> String {
        let parts: Vec<&str> = rest
            .split(|&byte| byte == b';')
            .take(PARTS)
            .filter_map(|part| std::str::from_utf8(part).ok())
            .collect();
        self.title = clean(&parts.join(";"));
        self.title.clone()
    }

    /// Saves the title: CSI 22 t.
    pub fn push(&mut self) {
        if self.saved.len() == SAVED {
            self.saved.remove(0);
        }
        self.saved.push(self.title.clone());
    }

    /// Brings the last title saved back: CSI 23 t; none when none was saved.
    pub fn pop(&mut self) -> Option<String> {
        let title = self.saved.pop()?;
        self.title.clone_from(&title);
        Some(title)
    }

    /// A full reset empties the title and those saved.
    pub fn reset(&mut self) {
        self.title.clear();
        self.saved.clear();
    }
}

/// A title as the window shows it: on one line, with what could disguise it gone, and cut to `SHOWN`.
///
/// A program may write controls a title must not carry: an escape that ends it and sets a color, or a
/// direction control that reorders what the user reads. C0 and C1 controls, the direction controls and
/// the invisible formatting go, as do newlines; joiners stay, as scripts and emoji need them.
pub fn clean(title: &str) -> String {
    let kept: String = title
        .chars()
        .filter(|&c| !crate::sessions::invisible(c))
        .map(|c| match c {
            '\n' | '\r' => ' ',
            c if c.is_control() => ' ',
            c => c,
        })
        .collect();
    let line = kept.trim();
    match line.char_indices().nth(SHOWN) {
        Some((at, _)) => line[..at].trim_end().to_owned(),
        None => line.to_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_title_reads_as_alacritty_joins_it() {
        let mut titles = Titles::default();
        assert_eq!(titles.set(b"  a;b  "), "a;b");
        assert_eq!(titles.set(b""), "");
        let parts: Vec<String> = (1..=17).map(|at| at.to_string()).collect();
        assert_eq!(
            titles.set(parts.join(";").as_bytes()),
            parts[..PARTS].join(";"),
            "vte keeps 16 parameters, the number among them"
        );
    }

    #[test]
    fn a_title_keeps_no_control_that_could_disguise_it() {
        assert_eq!(clean("a\u{1b}[31mb"), "a [31mb", "an escape is no colour");
        assert_eq!(clean("a\u{7}b"), "a b", "BEL");
        assert_eq!(clean("a\nb"), "a b", "a newline ends no line twice");
        assert_eq!(
            clean("\u{202e}gnp.exe"),
            "gnp.exe",
            "a direction control that would reorder it"
        );
        assert_eq!(clean("  a  "), "a", "trimmed");
        assert_eq!(
            clean("\u{4e2d}\u{200d}\u{6587}"),
            "\u{4e2d}\u{200d}\u{6587}"
        );
        let long: String = std::iter::repeat_n('a', SHOWN + 10).collect();
        assert_eq!(clean(&long).chars().count(), SHOWN, "cut to what shows");
    }

    #[test]
    fn a_title_saved_comes_back_and_a_reset_empties_both() {
        let mut titles = Titles::default();
        assert_eq!(titles.pop(), None, "none saved");
        titles.set(b"shell");
        titles.push();
        titles.set(b"vim");
        assert_eq!(titles.pop().as_deref(), Some("shell"));
        assert_eq!(titles.pop(), None);
        titles.push();
        titles.reset();
        assert_eq!(titles.pop(), None, "a reset empties those saved");
        for at in 0..=SAVED {
            titles.set(at.to_string().as_bytes());
            titles.push();
        }
        assert_eq!(titles.saved.len(), SAVED);
        assert_eq!(titles.saved[0], "1", "the oldest went");
    }
}
