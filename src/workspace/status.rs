//! The line under the terminal says what it runs as: where the session sits, its size and its branch.

use super::badge::Branch;

/// What the status line says of the session `id`, at `size` cells: where it sits, its size, and the branch of the folder it is in. The connection comes first, because it is what a user needs before they read a command's output; the size is there for a program that draws to it; the branch last, when there is one.
pub(super) fn of(session: Option<&crate::sessions::Session>, size: (i32, i32)) -> String {
    let mut parts = Vec::new();
    let Some(session) = session else {
        return String::new();
    };
    if let Some(where_) = crate::sessions::connection(&session.launch) {
        parts.push(where_);
    }
    parts.push(format!("{}x{}", size.0, size.1));
    if let Some(branch) = Branch::of(&session.folder) {
        parts.push(branch.label());
    }
    parts.join("  \u{b7}  ")
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;
    use crate::term::Target;

    /// The parts `of` builds, without a workspace: the same rules, on the pieces it reads.
    fn parts(connection: Option<String>, size: (i32, i32), branch: Option<String>) -> String {
        let mut parts = Vec::new();
        parts.extend(connection);
        parts.push(format!("{}x{}", size.0, size.1));
        parts.extend(branch);
        parts.join("  \u{b7}  ")
    }

    #[test]
    fn the_line_says_the_connection_the_size_and_the_branch() {
        let local = parts(None, (120, 40), Some("\u{e0a0} main".into()));
        assert_eq!(local, "120x40  \u{b7}  \u{e0a0} main");
        let remote = parts(Some("ssh: prod".into()), (80, 24), None);
        assert_eq!(remote, "ssh: prod  \u{b7}  80x24");
    }

    #[test]
    fn a_wsl_session_names_its_distribution_before_its_size() {
        let launch = crate::term::Launch {
            program: None,
            cwd: std::env::temp_dir(),
            env: Vec::new(),
            reports: false,
            quote: crate::term::Quote::Posix,
            target: Target::Wsl {
                distro: "Ubuntu-22.04".into(),
            },
        };
        let where_ = crate::sessions::connection(&launch);
        assert_eq!(
            parts(where_, (100, 30), None),
            "WSL: Ubuntu-22.04  \u{b7}  100x30"
        );
    }

    #[test]
    fn a_branch_shows_its_mark_and_a_folder_reports_the_branch_it_holds() {
        let root = std::env::temp_dir().join(format!("ely-status-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let git = root.join(".git");
        fs::create_dir_all(&git).expect("the git folder");
        fs::write(git.join("HEAD"), "ref: refs/heads/main\n").expect("HEAD");
        let branch = Branch::of(&root).map(|branch| branch.label());
        assert_eq!(branch.as_deref(), Some("\u{e0a0} main"));
        assert_eq!(parts(None, (10, 2), branch), "10x2  \u{b7}  \u{e0a0} main");
        let _ = fs::remove_dir_all(&root);
    }
}
