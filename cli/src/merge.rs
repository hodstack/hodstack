use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use anstyle::Style;
use anyhow::{Context as _, Result, anyhow, bail};

const BOLD: Style = Style::new().bold();

#[derive(Debug, PartialEq, Eq)]
pub struct Checkout {
    pub path: PathBuf,
    pub branch: Option<String>,
}

impl Checkout {
    fn read(record: &str) -> Result<Self> {
        let mut path = None;
        let mut branch = None;

        for line in record.lines() {
            if let Some(rest) = line.strip_prefix("worktree ") {
                path = Some(PathBuf::from(rest));
            }

            if let Some(rest) = line.strip_prefix("branch refs/heads/") {
                branch = Some(rest.to_owned());
            }
        }

        let path = path.ok_or_else(|| anyhow!("`git worktree list` names no worktree"))?;

        Ok(Self { path, branch })
    }
}

pub fn merge(
    here: &Path,
    keep: bool,
    message: Option<String>,
    input: &mut impl BufRead,
    out: &mut impl Write,
) -> Result<ExitCode> {
    let main = main(here)?;
    let worktree = worktree(here, &main)?;
    let branch = branch(&worktree)?;

    let Some(target) = main.branch.as_deref() else {
        bail!("`{}` has no branch; HEAD is detached", main.path.display())
    };

    clean(&worktree)?;
    clean(&main.path)?;

    writeln!(out)?;

    let message = match message {
        Some(message) => message,
        None => ask(input, out)?,
    };

    let conflicts = combine(&main.path, &branch, message.trim())?;

    if !conflicts.is_empty() {
        return stop(&main.path, &worktree, &branch, target, &conflicts, out);
    }

    writeln!(out, "  {BOLD}Merged{BOLD:#}   {branch} → {target}")?;

    if !keep {
        git(
            &main.path,
            &["worktree", "remove", &worktree.display().to_string()],
        )?;
        git(&main.path, &["branch", "-d", &branch])?;

        writeln!(out, "  {BOLD}Removed{BOLD:#}  {}", worktree.display())?;
    }

    writeln!(out)?;
    writeln!(out, "cd {}", main.path.display())?;

    Ok(ExitCode::SUCCESS)
}

fn ask(input: &mut impl BufRead, out: &mut impl Write) -> Result<String> {
    write!(out, "  {BOLD}Message{BOLD:#}  ")?;
    out.flush()?;

    let mut message = String::new();
    input
        .read_line(&mut message)
        .context("cannot read the message")?;

    Ok(message)
}

pub fn main(here: &Path) -> Result<Checkout> {
    let Ok(list) = git(here, &["worktree", "list", "--porcelain"]) else {
        bail!("`{}` is not a linked worktree", here.display())
    };

    let first = list.split("\n\n").next().unwrap_or_default();
    let mut main = Checkout::read(first)?;

    main.path = real(&main.path);

    Ok(main)
}

fn worktree(here: &Path, main: &Checkout) -> Result<PathBuf> {
    let top = git(here, &["rev-parse", "--show-toplevel"])?;
    let worktree = real(Path::new(top.trim()));

    if worktree == main.path {
        bail!("`{}` is not a linked worktree", here.display())
    }

    Ok(worktree)
}

fn branch(worktree: &Path) -> Result<String> {
    let name = git(worktree, &["rev-parse", "--abbrev-ref", "HEAD"])?;
    let name = name.trim();

    if name == "HEAD" {
        bail!("`{}` has no branch; HEAD is detached", worktree.display())
    }

    Ok(name.to_owned())
}

fn clean(checkout: &Path) -> Result<()> {
    let status = git(checkout, &["status", "--porcelain"])?;

    if !status.trim().is_empty() {
        bail!(
            "`{}` has a change that no commit holds; commit it or stash it",
            checkout.display()
        )
    }

    Ok(())
}

fn combine(main: &Path, branch: &str, message: &str) -> Result<Vec<String>> {
    let mut args = vec!["merge", "--no-ff", "--no-edit"];

    if !message.is_empty() {
        args.extend(["--message", message]);
    }

    args.push(branch);

    let Err(fault) = git(main, &args) else {
        return Ok(Vec::new());
    };

    let conflicts = git(main, &["diff", "--name-only", "--diff-filter=U"])?;
    let conflicts: Vec<String> = conflicts.lines().map(str::to_owned).collect();

    if conflicts.is_empty() {
        return Err(fault.context(format!("cannot merge `{branch}`")));
    }

    git(main, &["merge", "--abort"])?;

    Ok(conflicts)
}

fn stop(
    main: &Path,
    worktree: &Path,
    branch: &str,
    target: &str,
    conflicts: &[String],
    out: &mut impl Write,
) -> Result<ExitCode> {
    for path in conflicts {
        writeln!(out, "  {BOLD}Conflict{BOLD:#}  {path}")?;
    }

    writeln!(out)?;
    writeln!(
        out,
        "  `{}` holds no merge. Merge `{target}` into `{branch}` in `{}`, then run `hod worktree:merge` again.",
        main.display(),
        worktree.display()
    )?;
    writeln!(out)?;

    Ok(ExitCode::FAILURE)
}

pub fn real(path: &Path) -> PathBuf {
    fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

pub fn git(dir: &Path, args: &[&str]) -> Result<String> {
    let mut git = Command::new("git");
    git.arg("-C").arg(dir).args(args);

    let output = match git.output() {
        Ok(output) => output,
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            bail!("this computer has no git")
        }
        Err(error) => return Err(error).context("cannot start git"),
    };

    if output.status.success() {
        return Ok(String::from_utf8_lossy(&output.stdout).into_owned());
    }

    let message = String::from_utf8_lossy(&output.stderr);
    let message = message.trim();
    let message = message.strip_prefix("fatal: ").unwrap_or(message);

    Err(if message.is_empty() {
        anyhow!("git reports {}", output.status)
    } else {
        anyhow!(message.to_owned())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_record_gives_the_path_and_the_branch_of_a_checkout() {
        let checkout =
            Checkout::read("worktree /work/main\nHEAD 3f2b1c9\nbranch refs/heads/0.x\n").unwrap();

        assert_eq!(checkout.path, PathBuf::from("/work/main"));
        assert_eq!(checkout.branch.as_deref(), Some("0.x"));
    }

    #[test]
    fn a_record_without_a_branch_is_detached() {
        let checkout = Checkout::read("worktree /work/main\nHEAD 3f2b1c9\ndetached\n").unwrap();

        assert_eq!(checkout.branch, None);
    }

    #[test]
    fn a_record_without_a_worktree_is_a_fault() {
        assert!(Checkout::read("").is_err());
    }
}
