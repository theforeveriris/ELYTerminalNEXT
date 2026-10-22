use super::card::{lines, preview, typed, worth_a_look};

#[test]
fn a_paste_sends_each_line_break_carriage_first() {
    assert_eq!(typed("one\ntwo"), "one\rtwo");
    assert_eq!(typed("one\r\ntwo"), "one\rtwo");
    assert_eq!(typed("one\rtwo"), "one\rtwo");
}

#[test]
fn the_lines_a_paste_holds_count_its_breaks() {
    assert_eq!(lines("one"), 1);
    assert_eq!(lines("one\ntwo\nthree"), 3);
    assert_eq!(lines("one\r\ntwo"), 2);
    assert_eq!(lines(""), 0);
    assert_eq!(lines("\n"), 0, "a text of blanks runs nothing");
    assert_eq!(lines("one\n"), 2);
}

#[test]
fn a_paste_worth_a_look_holds_an_escape_or_runs_long() {
    assert!(!worth_a_look("ls -la\n", 100));
    assert!(worth_a_look("rm -rf /\u{1b}[2J", 100), "an escape");
    assert!(worth_a_look("a\u{7}b", 100), "a bell");
    assert!(worth_a_look(&"x".repeat(101), 100), "long");
    assert!(
        !worth_a_look("\u{4e2d}\u{6587}", 100),
        "wide text is no control"
    );
}

#[test]
fn a_preview_shows_escapes_and_line_ends() {
    assert_eq!(preview("a\nb", 10), "a \u{23ce}\nb");
    assert_eq!(preview("a\u{1b}b", 10), "a\\x1bb");
    assert_eq!(preview("abc", 2), "ab\u{2026}");
    assert_eq!(preview("\tx", 10), "    x");
}
