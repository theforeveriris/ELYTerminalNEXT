use std::{env, fs, path::PathBuf};

use super::*;

/// A fresh folder under the temp dir.
fn folder(name: &str) -> PathBuf {
    let folder = env::temp_dir().join(format!("elyterminal-scan-{name}-{}", std::process::id()));
    if folder.exists() {
        fs::remove_dir_all(&folder).expect("a stale folder goes");
    }
    fs::create_dir_all(&folder).expect("a test folder");
    folder
}

#[test]
fn git_bash_needs_both_its_bash_and_its_git() {
    let root = folder("git");
    fs::create_dir_all(root.join("bin")).expect("bin");
    fs::write(root.join("bin").join("bash.exe"), "").expect("bash");
    assert_eq!(git_bash(&root), None, "no git yet");
    fs::create_dir_all(root.join("cmd")).expect("cmd");
    fs::write(root.join("cmd").join("git.exe"), "").expect("git");
    let found = git_bash(&root).expect("Git Bash");
    assert_eq!(found.name, "Git Bash");
    assert_eq!(found.family, Family::Bash);
    assert_eq!(found.program, root.join("bin").join("bash.exe"));
    fs::remove_dir_all(&root).expect("cleanup");
}

#[test]
fn the_systems_wsl_launcher_is_not_git_bash() {
    let base = folder("wsl-launcher");
    let root = base.join("System32");
    fs::create_dir_all(root.join("bin")).expect("bin");
    fs::create_dir_all(root.join("cmd")).expect("cmd");
    fs::write(root.join("bin").join("bash.exe"), "").expect("bash");
    fs::write(root.join("cmd").join("git.exe"), "").expect("git");
    assert_eq!(git_bash(&root), None);
    fs::remove_dir_all(&base).expect("cleanup");
}

#[test]
fn the_registry_answer_names_the_install_root() {
    let answer = "\r\nHKEY_LOCAL_MACHINE\\SOFTWARE\\GitForWindows\r\n    InstallPath    REG_SZ    C:\\Program Files\\Git\r\n\r\n";
    assert_eq!(
        install_path(answer).as_deref(),
        Some(r"C:\Program Files\Git")
    );
    assert_eq!(install_path("no value here"), None);
    assert_eq!(install_path("InstallPath REG_SZ   "), None);
}

#[test]
fn wsl_distros_read_from_an_english_or_chinese_table() {
    let english = "  NAME            STATE           VERSION\r\n* Ubuntu-22.04    Stopped         2\r\n  archlinux       Running         2\r\n";
    let distros = distros_from(english);
    assert_eq!(distros.len(), 2);
    assert_eq!(distros[0].name, "Ubuntu-22.04");
    assert!(distros[0].default);
    assert!(!distros[0].running());
    assert_eq!(distros[0].version, Some(2));
    assert_eq!(distros[1].name, "archlinux");
    assert!(distros[1].running());
    let chinese = "  名称            状态            版本\r\n* Ubuntu-22.04    已停止          2\r\n  archlinux       运行中          2\r\n";
    let distros = distros_from(chinese);
    assert_eq!(distros.len(), 2, "the header is not a distro");
    assert_eq!(distros[1].state, "运行中");
    assert_eq!(distros[1].version, Some(2));
}

#[test]
fn a_table_without_a_version_still_names_its_distros() {
    let distros = distros_from("Ubuntu-22.04 Running\narchlinux Stopped\n");
    assert_eq!(distros.len(), 2);
    assert_eq!(distros[0].name, "Ubuntu-22.04");
    assert_eq!(distros[0].state, "Running");
    assert_eq!(distros[0].version, None);
}

#[test]
fn wsl_s_utf16_answer_reads_as_text() {
    let ascii = "Ubuntu-22.04\r\n"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    assert_eq!(utf16le(&ascii), "Ubuntu-22.04\r\n");
    let chinese = "名称\r\n"
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    assert_eq!(utf16le(&chinese), "名称\r\n");
    assert_eq!(utf16le(b"plain utf-8"), "plain utf-8");
}

#[test]
fn a_shell_s_command_names_the_arguments_it_starts_with() {
    let mut shell = Found::new(
        "WSL: archlinux",
        Family::Wsl,
        r"C:\Windows\System32\wsl.exe",
        Source::Wsl,
        Confidence::Certain,
    );
    shell.args = vec!["-d".into(), "archlinux".into()];
    assert_eq!(shell.command(), r"C:\Windows\System32\wsl.exe -d archlinux");
}

#[test]
fn powershell_seven_outranks_the_system_shell_and_cmd_comes_last() {
    let found = vec![
        Found::new(
            "cmd",
            Family::Cmd,
            "cmd.exe",
            Source::Known,
            Confidence::Certain,
        ),
        Found::new(
            "PowerShell 5.1",
            Family::PowerShell,
            "powershell.exe",
            Source::Known,
            Confidence::Certain,
        ),
        Found::new(
            "PowerShell 7",
            Family::PowerShell,
            "pwsh.exe",
            Source::Known,
            Confidence::Certain,
        ),
        Found::new(
            "Git Bash",
            Family::Bash,
            "bash.exe",
            Source::Registry,
            Confidence::Certain,
        ),
    ];
    let best = best(&found).expect("one shell");
    assert_eq!(best.name, "PowerShell 7");
    let mut sorted = found.clone();
    sorted.sort_by_key(|shell| std::cmp::Reverse(shell.score()));
    assert_eq!(sorted[0].name, "PowerShell 7");
    assert_eq!(sorted[3].name, "cmd");
}

#[test]
fn a_duplicate_program_is_listed_once() {
    let mut found = vec![
        Found::new(
            "bash",
            Family::Bash,
            "bash.exe",
            Source::Known,
            Confidence::Certain,
        ),
        Found::new(
            "bash",
            Family::Bash,
            "bash.exe",
            Source::Path,
            Confidence::Likely,
        ),
    ];
    let mut seen = HashSet::new();
    found.retain(|shell| seen.insert((shell.program.clone(), shell.args.clone())));
    assert_eq!(found.len(), 1);
}
