use super::*;
use crate::term::shell::scan::{Confidence, Family, Found, Source};

#[test]
fn a_version_answer_cuts_to_one_token() {
    assert_eq!(
        version_in("5.1.19041.5007\r\n").as_deref(),
        Some("5.1.19041.5007")
    );
    assert_eq!(
        version_in("GNU bash, version 5.2.37(1)-release (x86_64-pc-linux-gnu)").as_deref(),
        Some("5.2.37(1)-release")
    );
    assert_eq!(
        version_in("Microsoft Windows [Version 10.0.22631.4169]").as_deref(),
        Some("10.0.22631.4169")
    );
    assert_eq!(
        version_in("WSL version: 2.5.9.0\nKernel version: 6.6").as_deref(),
        Some("2.5.9.0")
    );
    assert_eq!(version_in("nu 0.101.0").as_deref(), Some("0.101.0"));
    assert_eq!(version_in("   \n"), None);
}

#[test]
fn a_version_probe_asks_the_program_its_family_names() {
    let mut shell = Found::new(
        "PowerShell",
        Family::PowerShell,
        "pwsh.exe",
        Source::Path,
        Confidence::Likely,
    );
    assert_eq!(
        version_args(shell.family),
        Some(
            &[
                "-NoProfile",
                "-Command",
                "$PSVersionTable.PSVersion.ToString()"
            ][..]
        )
    );
    shell.version = version_in("7.4.1");
    assert_eq!(shell.version.as_deref(), Some("7.4.1"));
    assert_eq!(version_args(Family::Cmd), Some(&["/C", "ver"][..]));
    assert_eq!(version_args(Family::Bash), Some(&["--version"][..]));
}
