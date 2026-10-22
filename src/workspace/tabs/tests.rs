use std::path::PathBuf;

use super::*;

fn launch(cwd: &str) -> Launch {
    Launch {
        program: Some(("/bin/sh".into(), vec!["-l".into()])),
        cwd: PathBuf::from(cwd),
        env: vec![("ELY".into(), "ok".into())],
        reports: true,
        quote: Quote::Posix,
        target: Target::Local,
    }
}

#[test]
fn a_clone_runs_the_same_program_in_the_folder_its_original_reports() {
    let started = launch("/home/me");
    let clone = clone_of(&started, &PathBuf::from("/home/me/src/api"));
    assert_eq!(clone.cwd, PathBuf::from("/home/me/src/api"));
    assert_eq!(clone.program, started.program, "the same program");
    assert_eq!(clone.env, started.env, "the same variables");
    assert_eq!(clone.reports, started.reports);
    assert_eq!(clone.quote, started.quote);
    assert_eq!(clone.target, started.target);
    assert_eq!(
        started.cwd,
        PathBuf::from("/home/me"),
        "the original stands"
    );
}
