//! A health check on demand: whether the settings file reads, the integration scripts are all there, and each shell the scan found still stands where the scan saw it, put in one place so a user who wonders why a distribution is missing reads the answer rather than the log.

use std::path::Path;

use crate::settings;
use crate::term::shell::scan;

/// The scripts a shell integration installs, by their path under its folder.
pub const SCRIPTS: [&str; 4] = [
    "zsh/.zshenv",
    "bash/elyterminal.bash",
    "fish/vendor_conf.d/elyterminal.fish",
    "pwsh/elyterminal.ps1",
];

/// One line of the report: what was checked, and how it came out.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub what: String,
    pub how: String,
    /// Whether the check found a fault.
    pub fault: bool,
}

/// The report: the settings file, the scripts, and the shells.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Report {
    pub findings: Vec<Finding>,
}

impl Report {
    /// Whether any finding is a fault.
    pub fn faulty(&self) -> bool {
        self.findings.iter().any(|finding| finding.fault)
    }

    /// The report as text, one finding per line, so it copies as it reads: `-` for what stands, `!` for a fault.
    pub fn text(&self) -> String {
        self.findings
            .iter()
            .map(|finding| {
                let mark = if finding.fault { '!' } else { '-' };
                format!("{mark} {}: {}", finding.what, finding.how)
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// The report of this machine: the settings `config` holds, the integration scripts in `scripts`, and each shell the scan found.
pub fn check(config: &Path, scripts: &Path, shells: &[scan::Found]) -> Report {
    let mut report = Report::default();
    let mut say = |what: &str, how: String, fault: bool| {
        report.findings.push(Finding {
            what: what.into(),
            how,
            fault,
        })
    };

    match settings::load_any(config) {
        Ok(settings) => {
            let mut keys: Vec<&String> = settings.extra.keys().collect();
            keys.sort();
            let unknown = match keys.is_empty() {
                true => "no unknown field".to_string(),
                false => format!(
                    "unknown fields kept until the next save: {}",
                    keys.iter()
                        .map(|key| key.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            };
            say(
                "Settings",
                format!("{} reads, {unknown}", config.display()),
                !keys.is_empty(),
            );
        }
        Err(error) => say("Settings", format!("{error:#}"), true),
    }

    let present = SCRIPTS
        .iter()
        .filter(|name| scripts.join(name).is_file())
        .count();
    say(
        "Shell integration",
        format!(
            "{present} of {} scripts in {}",
            SCRIPTS.len(),
            scripts.display()
        ),
        present < SCRIPTS.len(),
    );

    for found in shells {
        match settings::program_path(&found.program.display().to_string()) {
            Ok(program) => say(
                "Shells",
                format!("{}{} at {program}", found.name, version_of(found)),
                false,
            ),
            Err(error) => say(
                "Shells",
                format!("{} does not start: {error:#}", found.name),
                true,
            ),
        }
    }
    if shells.is_empty() {
        say("Shells", "the scan found none".into(), true);
    }
    report
}

/// ` (version)` for a shell that named one, else nothing.
fn version_of(found: &scan::Found) -> String {
    match &found.version {
        Some(version) => format!(" ({version})"),
        None => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("ely-health-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&root).expect("a test folder");
        root
    }

    #[test]
    fn a_report_reads_one_finding_a_line_and_names_its_faults() {
        let report = Report {
            findings: vec![
                Finding {
                    what: "Settings".into(),
                    how: "reads".into(),
                    fault: false,
                },
                Finding {
                    what: "Shells".into(),
                    how: "none".into(),
                    fault: true,
                },
            ],
        };
        assert!(report.faulty());
        assert_eq!(report.text(), "- Settings: reads\n! Shells: none");
        assert!(!Report::default().faulty());
    }

    #[test]
    fn a_version_shows_beside_its_shell_when_it_names_one() {
        let found = |version: Option<&str>| scan::Found {
            name: "PowerShell".into(),
            family: scan::Family::PowerShell,
            program: PathBuf::from("powershell"),
            args: Vec::new(),
            version: version.map(str::to_string),
            source: scan::Source::Path,
            confidence: scan::Confidence::Certain,
        };
        assert_eq!(version_of(&found(None)), "");
        assert_eq!(version_of(&found(Some("5.1"))), " (5.1)");
    }

    #[test]
    fn missing_scripts_and_a_bad_file_are_faults_and_a_good_one_is_not() {
        let root = scratch("report");
        let settings = root.join("settings.json");
        fs::write(&settings, "{}").expect("a settings file");
        let scripts = root.join("scripts");
        let report = check(&settings, &scripts, &[]);
        assert!(report.faulty());
        let text = report.text();
        assert!(
            text.contains("Settings: ") && text.contains("reads"),
            "{text}"
        );
        assert!(text.contains("Shell integration: 0 of 4 scripts"), "{text}");
        assert!(text.contains("Shells: the scan found none"), "{text}");
        fs::create_dir_all(&scripts).expect("the scripts folder");
        for name in SCRIPTS {
            let path = scripts.join(name);
            fs::create_dir_all(path.parent().expect("a folder")).expect("a folder");
            fs::write(path, "script").expect("a script");
        }
        let report = check(&settings, &scripts, &[]);
        let text = report.text();
        assert!(text.contains("Shell integration: 4 of 4 scripts"), "{text}");
        fs::write(&settings, "not json").expect("a bad file");
        let report = check(&settings, &scripts, &[]);
        assert!(
            report.text().contains("Settings: settings: "),
            "{}",
            report.text()
        );
        let _ = fs::remove_dir_all(&root);
    }
}
