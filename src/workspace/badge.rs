//! The Git branch the shown session's folder sits in, as its header says it.

use std::path::{Path, PathBuf};

/// What a session's header says of Git: which branch its folder sits in.
///
/// Reading `.git/HEAD` is cheap and needs no process; the counts that say a tree is dirty are not,
/// so the badge says which branch and leaves the accounting to the Git panel. A folder outside a
/// work tree says nothing at all, and so does a detached HEAD.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Branch {
    pub name: String,
}

impl Branch {
    /// The branch `folder` holds, read from its nearest `.git` or a gitfile pointing at one.
    pub fn of(folder: &Path) -> Option<Self> {
        let head = find_git(folder)?.join("HEAD");
        let text = std::fs::read_to_string(head).ok()?;
        let name = text
            .trim()
            .strip_prefix("ref: refs/heads/")
            .filter(|name| !name.is_empty())?
            .to_string();
        Some(Self { name })
    }

    /// What the header shows: a branch mark and the branch's name.
    pub fn label(&self) -> String {
        format!("\u{e0a0} {}", self.name)
    }
}

/// The `git` folder for `folder`, walking up from it: a `.git` folder, or the one its gitfile names.
fn find_git(folder: &Path) -> Option<PathBuf> {
    let mut at = Some(folder);
    while let Some(folder) = at {
        let dot = folder.join(".git");
        if dot.is_dir() {
            return Some(dot);
        }
        if dot.is_file()
            && let Some(pointer) = std::fs::read_to_string(&dot)
                .ok()
                .and_then(|text| text.trim().strip_prefix("gitdir: ").map(str::to_string))
        {
            let named = folder.join(pointer);
            if named.is_dir() {
                return Some(named);
            }
        }
        if folder.parent().is_none() {
            break;
        }
        at = folder.parent();
    }
    None
}

#[cfg(test)]
mod tests {
    use std::fs;

    use super::*;

    fn scratch(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("ely-git-badge-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn a_folder_in_a_work_tree_names_its_branch() {
        let root = scratch("branch");
        let git = root.join(".git");
        fs::create_dir_all(git.join("refs").join("heads")).expect("the git folder");
        fs::write(git.join("HEAD"), "ref: refs/heads/feature/paste\n").expect("HEAD");
        let deep = root.join("src").join("term");
        fs::create_dir_all(&deep).expect("a nested folder");
        assert_eq!(
            Branch::of(&deep),
            Some(Branch {
                name: "feature/paste".into()
            }),
            "the nearest git folder is found walking up"
        );
        assert_eq!(
            Branch::of(&deep).map(|branch| branch.label()),
            Some("\u{e0a0} feature/paste".into())
        );
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_detached_head_names_no_branch() {
        let root = scratch("detached");
        let git = root.join(".git");
        fs::create_dir_all(&git).expect("the git folder");
        fs::write(
            git.join("HEAD"),
            "0123456789abcdef0123456789abcdef01234567\n",
        )
        .expect("HEAD");
        assert_eq!(Branch::of(&root), None);
        assert_eq!(Branch::of(&scratch("none")), None, "no work tree, no badge");
        let _ = fs::remove_dir_all(&root);
    }

    #[test]
    fn a_worktree_s_gitfile_names_the_git_folder() {
        let root = scratch("worktree");
        fs::create_dir_all(&root).expect("a test folder");
        let real = root.join("real").join(".git");
        fs::create_dir_all(&real).expect("the real git folder");
        fs::write(real.join("HEAD"), "ref: refs/heads/topic\n").expect("HEAD");
        let branch = root.join("branch");
        fs::create_dir_all(&branch).expect("the worktree");
        let gitfile = format!("gitdir: {}\n", real.display());
        fs::write(branch.join(".git"), gitfile).expect("the gitfile");
        assert_eq!(
            Branch::of(&branch),
            Some(Branch {
                name: "topic".into()
            })
        );
        let _ = fs::remove_dir_all(&root);
    }
}
