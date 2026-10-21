//! Asking each shell which version it is.

use std::process::Command;

use super::scan::{Family, Found};

/// Asks each shell for its version, in place; one that cannot answer keeps none.
pub fn versions(found: &mut [Found]) {
    for shell in found {
        shell.version = probe(shell);
        log::info!(
            "shells: {} says its version is {:?}",
            shell.name,
            shell.version
        );
    }
}

/// The command that makes `found` name its version, where its family has one.
fn version_args(family: Family) -> Option<&'static [&'static str]> {
    match family {
        Family::PowerShell => Some(&[
            "-NoProfile",
            "-Command",
            "$PSVersionTable.PSVersion.ToString()",
        ]),
        Family::Cmd => Some(&["/C", "ver"]),
        _ => Some(&["--version"]),
    }
}

/// What `found` says its version is, in one short line.
pub fn probe(found: &Found) -> Option<String> {
    let args = version_args(found.family)?;
    let output = match Command::new(&found.program).args(args).output() {
        Ok(output) if output.status.success() => output,
        Ok(output) => {
            log::debug!(
                "shells: {} answers its version command with {}",
                found.name,
                output.status
            );
            return None;
        }
        Err(error) => {
            log::debug!("shells: {} does not answer: {error}", found.name);
            return None;
        }
    };
    version_in(&String::from_utf8_lossy(&output.stdout))
}

/// The version in a version command's answer, cut to one readable token.
pub fn version_in(text: &str) -> Option<String> {
    let line = text.lines().map(str::trim).find(|line| !line.is_empty())?;
    let lower = line.to_ascii_lowercase();
    let version = if let Some(at) = lower.find("[version ") {
        line[at + 9..].split(']').next()?
    } else if let Some(at) = lower.find("version") {
        line[at + 7..]
            .trim_start_matches([':', ' ', 'v'])
            .split_whitespace()
            .next()?
    } else {
        let mut words = line.split_whitespace();
        let first = words.next()?;
        if first.chars().all(|ch| ch.is_ascii_alphabetic()) {
            words.next().unwrap_or(first)
        } else {
            first
        }
    };
    let version = version
        .trim_matches(|ch: char| ch.is_ascii_punctuation() && !matches!(ch, '.' | '-' | '(' | ')'));
    (!version.is_empty()).then(|| version.chars().take(40).collect())
}

#[cfg(test)]
mod tests;
