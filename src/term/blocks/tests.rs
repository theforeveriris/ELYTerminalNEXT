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
