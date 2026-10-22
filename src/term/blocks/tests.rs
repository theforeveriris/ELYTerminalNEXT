use super::*;
use crate::term::marks::{Click, Mark};

fn at(seconds: u64) -> Instant {
    static BASE: std::sync::OnceLock<Instant> = std::sync::OnceLock::new();
    *BASE.get_or_init(Instant::now) + Duration::from_secs(seconds)
}

#[test]
fn a_command_takes_its_time_and_its_status() {
    let mut blocks = Blocks::default();
    let start = at(0);
    blocks.mark(Mark::Prompt(Click::Arrows), start);
    blocks.mark(Mark::Input, at(1));
    blocks.mark(Mark::Output, at(2));
    blocks.mark(Mark::Done(Some(1)), at(5));
    let block = blocks.current().expect("the block");
    assert_eq!(block.id, 0);
    assert_eq!(block.phase, Phase::Done(Some(1)));
    assert_eq!(block.duration(), Some(Duration::from_secs(3)));
    assert!(block.failed());
    assert_eq!(block.status(), Some(1));
    assert!(!block.running());
}

#[test]
fn a_slow_command_says_when_it_is_done() {
    let start = at(0);
    let block = |status, seconds| Block {
        id: 0,
        phase: Phase::Done(status),
        started: start,
        command_at: Some(start),
        ended: Some(start + Duration::from_secs(seconds)),
        command: None,
        prompt_line: 0,
        output_line: 0,
        input_line: 0,
        input_column: 0,
        end_line: None,
    };
    assert_eq!(completion(&block(Some(0), 9)), None, "too quick");
    assert_eq!(
        completion(&block(Some(0), 12)).as_deref(),
        Some("Command finished in 12.0s")
    );
    assert_eq!(
        completion(&block(Some(2), 61)).as_deref(),
        Some("Command failed with exit 2 after 61.0s")
    );
    assert_eq!(
        completion(&block(None, 30)).as_deref(),
        Some("Command finished in 30.0s")
    );
}

#[test]
fn a_running_command_has_no_duration_yet() {
    let mut blocks = Blocks::default();
    blocks.mark(Mark::Prompt(Click::Arrows), at(0));
    blocks.mark(Mark::Output, at(2));
    let block = blocks.current().expect("the block");
    assert!(block.running());
    assert_eq!(block.duration(), None);
    assert_eq!(block.status(), None);
}

#[test]
fn a_prompt_that_runs_nothing_has_no_duration() {
    let mut blocks = Blocks::default();
    blocks.mark(Mark::Prompt(Click::Arrows), at(0));
    blocks.mark(Mark::Input, at(1));
    blocks.mark(Mark::Done(Some(0)), at(2));
    let block = blocks.current().expect("the block");
    assert_eq!(block.duration(), None);
    assert!(!block.failed());
}

#[test]
fn a_new_prompt_keeps_the_block_that_ended() {
    let mut blocks = Blocks::default();
    blocks.mark(Mark::Prompt(Click::Arrows), at(0));
    blocks.mark(Mark::Output, at(1));
    blocks.mark(Mark::Done(Some(0)), at(2));
    blocks.mark(Mark::Prompt(Click::Arrows), at(3));
    blocks.mark(Mark::Output, at(4));
    assert_eq!(blocks.len(), 1);
    let done = &blocks.done()[0];
    assert_eq!(done.id, 0);
    assert_eq!(done.duration(), Some(Duration::from_secs(1)));
    assert_eq!(blocks.current().expect("the new block").id, 1);
    assert_eq!(blocks.last_finished().expect("the done block").id, 0);
}

#[test]
fn marks_outside_a_block_change_nothing() {
    let mut blocks = Blocks::default();
    blocks.mark(Mark::Input, at(0));
    blocks.mark(Mark::Output, at(1));
    blocks.mark(Mark::Done(Some(0)), at(2));
    assert!(blocks.current().is_none());
    assert!(blocks.done().is_empty());
    assert!(blocks.is_empty());
    assert!(blocks.last().is_none());
}

#[test]
fn a_running_block_that_a_prompt_replaces_is_not_kept() {
    let mut blocks = Blocks::default();
    blocks.mark(Mark::Prompt(Click::Arrows), at(0));
    blocks.mark(Mark::Output, at(1));
    blocks.mark(Mark::Prompt(Click::Arrows), at(2));
    assert!(blocks.done().is_empty(), "no end, no kept block");
    assert_eq!(blocks.current().expect("the new block").id, 1);
}

#[test]
fn the_oldest_blocks_go_when_the_keep_is_full() {
    let mut blocks = Blocks::default();
    for id in 0..(KEEP as u64 + 2) {
        let base = id * 4;
        blocks.mark(Mark::Prompt(Click::Arrows), at(base));
        blocks.mark(Mark::Output, at(base + 1));
        blocks.mark(Mark::Done(Some(0)), at(base + 2));
    }
    assert_eq!(blocks.len(), KEEP);
    assert_eq!(blocks.done()[0].id, 1, "the first went");
}

#[test]
fn a_command_s_line_is_kept_with_its_block() {
    let mut blocks = Blocks::default();
    blocks.mark(Mark::Prompt(Click::Arrows), at(0));
    blocks.mark(Mark::Input, at(1));
    blocks.typed("cargo test".into(), 0);
    blocks.mark(Mark::Output, at(2));
    blocks.mark(Mark::Done(Some(1)), at(5));
    let done = blocks.current().expect("the block");
    assert_eq!(done.typed(), Some("cargo test"));
    assert_eq!(done.status(), Some(1));
}

#[test]
fn a_summary_counts_what_ran_and_names_the_slowest() {
    let mut blocks = Blocks::default();
    for (command, status, start, end) in [
        ("make", 0, 0, 3),
        ("cargo test", 2, 4, 44),
        ("ls", 0, 45, 46),
    ] {
        blocks.mark(Mark::Prompt(Click::Arrows), at(start));
        blocks.mark(Mark::Input, at(start));
        blocks.typed(command.into(), 0);
        blocks.mark(Mark::Output, at(start));
        blocks.mark(Mark::Done(Some(status)), at(end));
    }
    let summary = blocks.summary();
    assert_eq!(
        summary.ran, 2,
        "the last block is still current: {summary:?}"
    );
    assert_eq!(summary.failed, 1);
    let (slowest, took) = summary.slowest.expect("a slowest");
    assert_eq!(slowest, "cargo test");
    assert_eq!(took, Duration::from_secs(40));
    assert_eq!(blocks.failures(), [(1, "cargo test", Some(2))]);
}

#[test]
fn stepping_walks_the_commands_that_ended_and_can_skip_the_rest() {
    let mut blocks = Blocks::default();
    for (id, status) in [(0, 0), (1, 2), (2, 0)] {
        blocks.mark(Mark::Prompt(Click::Arrows), at(id));
        blocks.typed(format!("command {id}"), 0);
        blocks.mark(Mark::Output, at(id));
        blocks.mark(Mark::Done(Some(status)), at(id + 1));
    }
    blocks.mark(Mark::Prompt(Click::Arrows), at(9));
    assert_eq!(
        blocks.done().len(),
        3,
        "each ended, and the next prompt kept it"
    );
    assert_eq!(blocks.step(None, true, false), Some(0), "the oldest first");
    assert_eq!(blocks.step(Some(0), true, false), Some(1));
    assert_eq!(blocks.step(Some(2), true, false), Some(0), "round the end");
    assert_eq!(
        blocks.step(Some(0), false, false),
        Some(2),
        "back round the start"
    );
    assert_eq!(
        blocks.step(None, true, true),
        Some(1),
        "only the failed one"
    );
    assert_eq!(
        blocks.step(Some(1), true, true),
        Some(1),
        "it is the only one"
    );
    assert_eq!(
        Blocks::default().step(None, true, false),
        None,
        "nothing ran"
    );
}

#[test]
fn an_export_says_what_ran_and_how_it_ended() {
    let mut blocks = Blocks::default();
    blocks.mark(Mark::Prompt(Click::Arrows), at(0));
    blocks.typed("cargo test".into(), 0);
    blocks.mark(Mark::Output, at(0));
    blocks.mark(Mark::Done(Some(2)), at(3));
    blocks.mark(Mark::Prompt(Click::Arrows), at(4));
    blocks.typed("cargo test".into(), 4);
    assert_eq!(export(&blocks, Export::Text), "2\t3.0s\tcargo test");
    let markdown = export(&blocks, Export::Markdown);
    assert!(
        markdown.starts_with("| Command | Exit | Time |"),
        "{markdown}"
    );
    assert!(
        markdown.contains("| `cargo test` | 2 | 3.0s |"),
        "{markdown}"
    );
    let json = export(&blocks, Export::Json);
    assert!(json.starts_with('[') && json.ends_with(']'), "{json}");
    assert!(json.contains(r#""command": "cargo test""#), "{json}");
    assert!(json.contains(r#""exit": 2"#), "{json}");
    assert_eq!(export(&Blocks::default(), Export::Text), "");
    assert_eq!(json_string("a\"b\\c\n"), r#""a\"b\\c\n""#);
}
