use std::env;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{self, ExitCode};
use std::time::{SystemTime, UNIX_EPOCH};

use anstyle::Style;
use anyhow::{Context as _, Result, anyhow};

use crate::merge::{self, git};
use crate::project;

const BOLD: Style = Style::new().bold();
const WORKTREES: &str = ".hod/worktrees";
const FIRST: [&str; 32] = [
    "amber", "bold", "brave", "bright", "calm", "clear", "cool", "crisp", "deep", "early", "fair",
    "fast", "fond", "free", "glad", "gold", "keen", "kind", "light", "lucky", "mild", "neat",
    "plain", "proud", "quick", "quiet", "rare", "sharp", "soft", "still", "warm", "wise",
];
const SECOND: [&str; 32] = [
    "anchor", "bay", "birch", "brook", "cedar", "cliff", "cloud", "coast", "comet", "coral",
    "dune", "ember", "fern", "field", "fjord", "flame", "grove", "harbor", "heron", "island",
    "lake", "maple", "meadow", "moss", "oak", "otter", "pine", "reef", "ridge", "river", "stone",
    "tide",
];

pub fn create(here: &Path, branch: Option<String>, out: &mut impl Write) -> Result<ExitCode> {
    let main = merge::main(here)?;
    let home = home()?;
    let branch = branch.unwrap_or_else(|| name(&home, &main.path));
    let branch = branch.as_str();
    let path = place(&home, &main.path, branch);

    if exists(&main.path, branch) {
        git(
            &main.path,
            &[
                "worktree",
                "add",
                "--quiet",
                &path.display().to_string(),
                branch,
            ],
        )
    } else {
        git(
            &main.path,
            &[
                "worktree",
                "add",
                "--quiet",
                "-b",
                branch,
                &path.display().to_string(),
            ],
        )
    }
    .with_context(|| format!("cannot create `{}`", path.display()))?;

    writeln!(out)?;
    writeln!(out, "  {BOLD}Created{BOLD:#}  {}", path.display())?;
    writeln!(out, "  {BOLD}Branch{BOLD:#}   {branch}")?;
    writeln!(out)?;
    writeln!(out, "cd {}", path.display())?;

    Ok(ExitCode::SUCCESS)
}

fn home() -> Result<PathBuf> {
    env::var_os("HOME")
        .or_else(|| env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .ok_or_else(|| anyhow!("neither `HOME` nor `USERPROFILE` names the home directory"))
}

fn name(home: &Path, main: &Path) -> String {
    let mut seed = seed();

    loop {
        let branch = format!(
            "{}-{}",
            FIRST[seed % FIRST.len()],
            SECOND[(seed / FIRST.len()) % SECOND.len()]
        );

        if !exists(main, &branch) && !place(home, main, &branch).exists() {
            return branch;
        }

        seed += 1;
    }
}

fn seed() -> usize {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.subsec_nanos())
        .unwrap_or_default();

    nanos as usize ^ process::id() as usize
}

fn place(home: &Path, main: &Path, branch: &str) -> PathBuf {
    home.join(WORKTREES)
        .join(project::name(main))
        .join(branch.replace('/', "-"))
}

fn exists(main: &Path, branch: &str) -> bool {
    git(
        main,
        &[
            "rev-parse",
            "--verify",
            "--quiet",
            &format!("refs/heads/{branch}"),
        ],
    )
    .is_ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_worktree_sits_in_the_home_directory_under_the_project_and_the_branch() {
        assert_eq!(
            place(
                Path::new("/home/me"),
                Path::new("/work/hodstack"),
                "feature"
            ),
            PathBuf::from("/home/me/.hod/worktrees/hodstack/feature")
        );
        assert_eq!(
            place(
                Path::new("/home/me"),
                Path::new("/work/hodstack"),
                "fix/help"
            ),
            PathBuf::from("/home/me/.hod/worktrees/hodstack/fix-help")
        );
    }
}
