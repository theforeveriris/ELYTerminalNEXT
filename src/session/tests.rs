use std::{env, fs, path::PathBuf};

use super::*;

fn folder(name: &str) -> PathBuf {
    let folder = env::temp_dir().join(format!("elyterminal-session-{name}-{}", std::process::id()));
    if folder.exists() {
        fs::remove_dir_all(&folder).expect("a stale folder goes");
    }
    fs::create_dir_all(&folder).expect("a test folder");
    folder
}

fn saved(tabs: &[(&str, &str)], active: usize) -> Saved {
    Saved {
        version: VERSION,
        active,
        tabs: tabs
            .iter()
            .map(|(name, cwd)| SavedTab {
                name: name.to_string(),
                cwd: cwd.to_string(),
                collapsed: false,
            })
            .collect(),
    }
}

#[test]
fn a_saved_window_reads_back() {
    let root = folder("round");
    let path = root.join("session.json");
    let mut window = saved(&[("api", "/home/u/api"), ("docs", "/home/u/docs")], 1);
    window.tabs[1].collapsed = true;
    save(&path, &window).expect("saves");
    assert_eq!(load(&path).expect("loads"), window);
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn a_folded_tab_is_open_when_the_file_never_said() {
    let tab: SavedTab = serde_json::from_str(r#"{"name": "api", "cwd": "/home/u/api"}"#)
        .expect("an older file reads");
    assert!(!tab.collapsed, "a tab the file does not fold stays open");
}

#[test]
fn a_missing_file_restores_nothing() {
    let root = folder("missing");
    assert_eq!(load(&root.join("session.json")), None);
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn a_file_of_another_version_or_with_no_tabs_restores_nothing() {
    let root = folder("version");
    let path = root.join("session.json");
    let mut window = saved(&[("one", "/tmp")], 0);
    window.version = 2;
    fs::write(&path, serde_json::to_string(&window).unwrap()).expect("a file");
    assert_eq!(load(&path), None);
    fs::write(&path, r#"{"version":1,"active":0,"tabs":[]}"#).expect("a file");
    assert_eq!(load(&path), None);
    fs::write(&path, "not json").expect("a file");
    assert_eq!(load(&path), None);
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn the_active_tab_stays_within_the_tabs_read() {
    let root = folder("active");
    let path = root.join("session.json");
    fs::write(
        &path,
        r#"{"version":1,"active":9,"tabs":[{"name":"one","cwd":"/tmp"}]}"#,
    )
    .expect("a file");
    assert_eq!(load(&path).expect("loads").active, 0);
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn a_file_keeping_too_many_tabs_is_cut_and_saved_that_way() {
    let root = folder("many");
    let path = root.join("session.json");
    let tabs: Vec<SavedTab> = (0..MAX + 3)
        .map(|at| SavedTab {
            name: format!("tab {at}"),
            cwd: "/tmp".into(),
            collapsed: false,
        })
        .collect();
    let window = Saved {
        version: VERSION,
        active: MAX + 2,
        tabs,
    };
    save(&path, &window).expect("saves");
    let read = load(&path).expect("loads");
    assert_eq!(read.tabs.len(), MAX);
    assert_eq!(read.active, MAX - 1);
    fs::remove_dir_all(&root).expect("cleanup");
}
