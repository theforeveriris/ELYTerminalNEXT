use alacritty_terminal::{
    index::{Boundary, Point},
    term::{Term, cell::Flags},
};
use ely_gpui_component::terminal::{Target, links};

use super::prompts;

/// The schemes a link may open: a web page, a mail or a file the platform opens; no `javascript:` or `data:`.
pub const SCHEMES: [&str; 4] = ["http", "https", "mailto", "file"];

/// Whether `link` is one the user may open: a scheme of its own, and one the terminal lets through.
pub fn allowed(link: &str) -> bool {
    let Some((scheme, _)) = scheme(link) else {
        log::debug!("links: {link:?} names no scheme");
        return false;
    };
    let ok = SCHEMES.iter().any(|one| scheme.eq_ignore_ascii_case(one));
    if !ok {
        log::warn!(
            "links: {scheme:?} is none of {:?}, so it opens not",
            SCHEMES
        );
    }
    ok
}

/// `link`'s scheme and what follows it; none when it names no scheme, or is not UTF-8 after it.
fn scheme(link: &str) -> Option<(&str, &str)> {
    let (scheme, rest) = link.split_once(':')?;
    let scheme = scheme.trim();
    if scheme.is_empty()
        || scheme
            .chars()
            .any(|c| !c.is_ascii_alphanumeric() && !matches!(c, '+' | '-' | '.'))
    {
        return None;
    }
    Some((scheme, rest))
}

/// The OSC 8 target at `point`, else a URL in its wrapped line, when it is one the user may open; a prompt's own link is no target.
pub fn link_at<T>(term: &Term<T>, mut point: Point) -> Option<String> {
    let grid = term.grid();
    if grid[point].flags.contains(Flags::WIDE_CHAR_SPACER) {
        point.column -= 1;
    }
    if let Some(link) = grid[point]
        .hyperlink()
        .filter(|link| link.uri() != prompts::LINK)
    {
        return allowed(link.uri()).then(|| link.uri().to_string());
    }
    let end = term.line_search_right(point);
    let mut cell = term.line_search_left(point);
    let (mut text, mut at) = (String::new(), 0);
    loop {
        if cell == point {
            at = text.len();
        }
        let spacer = Flags::WIDE_CHAR_SPACER | Flags::LEADING_WIDE_CHAR_SPACER;
        if !grid[cell].flags.intersects(spacer) {
            text.push(grid[cell].c);
            text.extend(grid[cell].zerowidth().into_iter().flatten());
        }
        if cell == end {
            break;
        }
        cell = cell.add(term, Boundary::Grid, 1);
    }
    links(&text)
        .into_iter()
        .find_map(|(bytes, target)| match target {
            Target::Url(url) if bytes.contains(&at) && allowed(&url) => Some(url.to_string()),
            _ => None,
        })
}

#[cfg(test)]
mod tests {
    use alacritty_terminal::{
        event::VoidListener,
        index::{Column, Line},
        term::{Config, test::TermSize},
        vte::ansi::Processor,
    };

    use super::*;

    fn link(bytes: &str, line: i32, column: usize) -> Option<String> {
        let mut term = Term::new(Config::default(), &TermSize::new(12, 4), VoidListener);
        let mut parser: Processor = Processor::new();
        parser.advance(&mut term, bytes.as_bytes());
        link_at(&term, Point::new(Line(line), Column(column)))
    }

    #[test]
    fn a_link_opens_only_what_the_user_may_open() {
        for ok in [
            "https://ely.dev/x",
            "http://ely.dev",
            "HTTPS://ELY.DEV",
            "mailto:a@b.c",
            "file:///a/b",
        ] {
            assert!(allowed(ok), "{ok}");
        }
        for no in [
            "javascript:alert(1)",
            "data:text/html,a",
            "vbscript:x",
            "ely.dev",
            ":x",
            "1a:",
            "",
        ] {
            assert!(!allowed(no), "{no}");
        }
    }

    #[test]
    fn a_link_the_terminal_refuses_is_no_target() {
        let text = "a \x1b]8;;javascript:alert(1)\x1b\\x\x1b]8;;\x1b\\ b";
        assert_eq!(link(text, 0, 3), None);
        let data = "\x1b]8;;data:text/html,a\x1b\\x\x1b]8;;\x1b\\";
        assert_eq!(link(data, 0, 0), None);
    }

    #[test]
    fn an_osc_8_link_names_its_target() {
        let text = "a \x1b]8;;https://ely.dev/x\x1b\\ely\x1b]8;;\x1b\\ b";
        assert_eq!(link(text, 0, 3).as_deref(), Some("https://ely.dev/x"));
        assert_eq!(link(text, 0, 0), None);
    }

    #[test]
    fn a_prompt_s_own_link_is_no_target() {
        let text = "\x1b]8;;\x7f\x1b\\$ https://ely.dev\x1b]8;;\x1b\\";
        assert_eq!(link(text, 0, 0), None);
        assert_eq!(link(text, 0, 4).as_deref(), Some("https://ely.dev"));
    }

    #[test]
    fn a_url_ends_before_its_sentence_does() {
        assert_eq!(link("(a.io/x).", 0, 2), None, "no scheme, no link");
        let text = "(http://a.b)";
        assert_eq!(link(text, 0, 4).as_deref(), Some("http://a.b"));
        assert_eq!(link(text, 0, 11), None);
    }

    #[test]
    fn a_url_keeps_its_combining_marks() {
        let text = "https://e.test/cafe\u{301}/x";
        assert_eq!(link(text, 1, 1).as_deref(), Some(text));
    }

    #[test]
    fn a_url_runs_across_wrapped_rows_and_wide_cells() {
        let text = "\u{4e2d} https://ely.dev/abc";
        assert_eq!(link(text, 1, 3).as_deref(), Some("https://ely.dev/abc"));
        assert_eq!(link(text, 0, 3).as_deref(), Some("https://ely.dev/abc"));
        assert_eq!(link(text, 0, 1), None);
        let wide = "file://a/\u{4e2d} x";
        assert_eq!(
            link(wide, 0, 10).as_deref(),
            Some("file://a/\u{4e2d}"),
            "a wide cell's spacer is its char"
        );
    }
}
