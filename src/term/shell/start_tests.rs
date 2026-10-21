use std::{env, path::PathBuf};

use super::{launch_found, scan};

fn folder() -> PathBuf {
    env::temp_dir().join(format!("elyterminal-start-{}", std::process::id()))
}

fn found(family: scan::Family, program: &str) -> scan::Found {
    scan::Found::new(
        "a shell",
        family,
        program,
        scan::Source::Known,
        scan::Confidence::Certain,
    )
}

#[test]
fn a_wsl_shell_starts_its_distro_in_the_folder() {
    let mut shell = found(scan::Family::Wsl, r"C:\Windows\System32\wsl.exe");
    shell.args = vec!["-d".into(), "archlinux".into()];
    let launch = launch_found(&shell, folder(), &folder()).expect("a launch");
    let (program, args) = launch.program.expect("a program");
    assert_eq!(program, r"C:\Windows\System32\wsl.exe");
    assert_eq!(&args[..3], ["-d", "archlinux", "--cd"]);
    assert_eq!(Some(&args[3]), scan::wsl_path(&folder()).as_ref());
    assert!(!launch.reports, "no integration inside the distribution");
    assert_eq!(launch.quote, super::Quote::Posix);
}

#[test]
fn powershell_reads_its_integration_and_keeps_its_own_arguments() {
    let mut shell = found(scan::Family::PowerShell, "pwsh.exe");
    shell.args = vec!["-NoProfile".into()];
    let launch = launch_found(&shell, folder(), &folder()).expect("a launch");
    let (_, args) = launch.program.expect("a program");
    assert_eq!(args[0], "-NoProfile");
    assert_eq!(args[1], "-NoLogo");
    assert_eq!(&args[2..4], ["-NoExit", "-ExecutionPolicy"]);
    assert_eq!(args[5], "-Command");
    assert!(args[6].contains("elyterminal.ps1"), "{:?}", args[6]);
    assert!(launch.reports);
    assert_eq!(launch.quote, super::Quote::PowerShell);
}

#[test]
fn a_bash_shell_reads_the_integration_script() {
    let shell = found(scan::Family::Bash, "bash.exe");
    let launch = launch_found(&shell, folder(), &folder()).expect("a launch");
    let (_, args) = launch.program.expect("a program");
    assert_eq!(args[0], "--rcfile");
    if cfg!(windows) {
        assert!(!args[1].contains('\\'), "{:?}", args[1]);
        assert!(args[1].ends_with("bash/elyterminal.bash"), "{:?}", args[1]);
    } else {
        assert!(args[1].ends_with("bash/elyterminal.bash"), "{:?}", args[1]);
    }
    assert!(launch.reports);
    assert_eq!(launch.quote, super::Quote::Posix);
}

#[test]
fn cmd_keeps_its_own_arguments_and_reports_nothing() {
    let mut shell = found(scan::Family::Cmd, "cmd.exe");
    shell.args = vec!["/K".into()];
    let launch = launch_found(&shell, folder(), &folder()).expect("a launch");
    let (_, args) = launch.program.expect("a program");
    assert_eq!(args, ["/K"]);
    assert!(!launch.reports);
    assert_eq!(launch.quote, super::Quote::Cmd);
}

#[cfg(windows)]
#[test]
fn a_windows_path_reads_as_wsl_does() {
    assert_eq!(
        scan::wsl_path(std::path::Path::new(r"C:\Users\me\src")).as_deref(),
        Some("/mnt/c/Users/me/src")
    );
    assert_eq!(
        scan::wsl_path(std::path::Path::new(
            r"\\wsl.localhost\Ubuntu-22.04\home\me"
        ))
        .as_deref(),
        Some("/home/me")
    );
    assert_eq!(
        scan::wsl_path(std::path::Path::new(r"\\server\share")).as_deref(),
        None
    );
    assert_eq!(
        scan::wsl_path(std::path::Path::new(r"C:relative")).as_deref(),
        None
    );
}

#[cfg(unix)]
#[test]
fn a_posix_path_is_already_where_a_distribution_looks() {
    assert_eq!(
        scan::wsl_path(std::path::Path::new("/home/me")).as_deref(),
        Some("/home/me")
    );
    assert_eq!(scan::wsl_path(std::path::Path::new("relative")), None);
}
