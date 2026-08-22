use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use anstyle::Style;
use anyhow::{Context as _, Result};

use crate::lock::{Lock, Owner};
use crate::project::{self, CLIENTS, Project, Rule};
use crate::skills::Skill;

const BOLD: Style = Style::new().bold();

const DIM: Style = Style::new().dimmed();

const IGNORED: [&str; 2] = ["/.claude/skills/", "/.agents/skills/"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Check,
    Write,
    Force,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Outcome {
    Kept,
    Created,
    Updated,
    Removed,
    Skipped,
}

#[derive(Debug)]
struct Unit {
    label: String,
    files: Vec<(String, String)>,
}

pub fn sync(project: &Project, mode: Mode, out: &mut impl Write) -> Result<ExitCode> {
    let units = plan(&project.rules()?, &project.installed()?);
    let lock = Lock::read(&project.lock());
    let mut next = Lock::default();
    let mut code = ExitCode::SUCCESS;

    writeln!(out)?;

    if mode == Mode::Check {
        writeln!(out, "  {DIM}This run writes nothing.{DIM:#}")?;
        writeln!(out)?;
    }

    report(out, seed(project, mode)?, project::INTENTION)?;

    for unit in &units {
        let outcome = keep(project, &lock, &mut next, unit, mode)?;

        report(out, outcome, &unit.label)?;

        if outcome == Outcome::Skipped {
            writeln!(
                out,
                "           This file is yours. Run `hod update --force` to write over it."
            )?;

            if mode != Mode::Check {
                code = ExitCode::FAILURE;
            }
        }
    }

    for label in gone(project, &lock, &next, &units, mode)? {
        report(out, Outcome::Removed, &label)?;
    }

    if let Some(outcome) = ignore(project, mode)? {
        report(out, outcome, ".gitignore")?;
    }

    if mode != Mode::Check {
        next.write(&project.lock())?;
    }

    writeln!(out)?;

    Ok(code)
}

fn plan(rules: &[Rule], skills: &[Skill]) -> Vec<Unit> {
    let mut units = vec![
        Unit {
            label: project::AGENTS.to_owned(),
            files: vec![(project::AGENTS.to_owned(), project::agents(rules))],
        },
        Unit {
            label: project::CLAUDE.to_owned(),
            files: vec![(project::CLAUDE.to_owned(), project::IMPORT.to_owned())],
        },
    ];

    for skill in skills {
        for client in CLIENTS {
            units.push(Unit {
                label: format!("{client}/{}", skill.name),
                files: skill
                    .files
                    .iter()
                    .map(|(file, text)| (format!("{client}/{}/{file}", skill.name), text.clone()))
                    .collect(),
            });
        }
    }

    units
}

fn keep(
    project: &Project,
    lock: &Lock,
    next: &mut Lock,
    unit: &Unit,
    mode: Mode,
) -> Result<Outcome> {
    let mut outcome = Outcome::Kept;

    for (file, text) in &unit.files {
        let path = project.path(file);
        let found = fs::read(&path).ok();
        let state = lock.state(file, found.as_deref());

        if state == Owner::Theirs && mode != Mode::Force {
            outcome = outcome.max(Outcome::Skipped);
            continue;
        }

        next.keep(file, project::sum(text.as_bytes()));

        if found.as_deref() == Some(text.as_bytes()) {
            continue;
        }

        outcome = outcome.max(if state == Owner::Absent {
            Outcome::Created
        } else {
            Outcome::Updated
        });

        if mode != Mode::Check {
            write(&path, text)?;
        }
    }

    Ok(outcome)
}

fn gone(
    project: &Project,
    lock: &Lock,
    next: &Lock,
    units: &[Unit],
    mode: Mode,
) -> Result<Vec<String>> {
    let mut labels = Vec::new();

    for file in lock.files() {
        if next.holds(file) {
            continue;
        }

        let path = project.path(file);
        let found = fs::read(&path).ok();

        if lock.state(file, found.as_deref()) != Owner::Ours {
            continue;
        }

        if mode != Mode::Check {
            fs::remove_file(&path)
                .with_context(|| format!("cannot remove `{}`", path.display()))?;
        }

        let label = label(file);

        if units.iter().any(|unit| unit.label == label) {
            continue;
        }

        if !labels.contains(&label) {
            labels.push(label);
        }
    }

    if mode != Mode::Check {
        for label in &labels {
            prune(&project.path(label));
        }
    }

    Ok(labels)
}

fn label(file: &str) -> String {
    for client in CLIENTS {
        if let Some(rest) = file.strip_prefix(&format!("{client}/")) {
            if let Some(name) = rest.split('/').next() {
                return format!("{client}/{name}");
            }
        }
    }

    file.to_owned()
}

fn prune(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();

        if path.is_dir() {
            prune(&path);
        }
    }

    let _ = fs::remove_dir(dir);
}

fn seed(project: &Project, mode: Mode) -> Result<Outcome> {
    let path = project.path(project::INTENTION);

    if path.exists() {
        return Ok(Outcome::Kept);
    }

    if mode != Mode::Check {
        write(&path, project::SEED)?;
    }

    Ok(Outcome::Created)
}

fn ignore(project: &Project, mode: Mode) -> Result<Option<Outcome>> {
    if !project.path(".git").exists() {
        return Ok(None);
    }

    let path = project.path(".gitignore");
    let found = fs::read_to_string(&path).unwrap_or_default();
    let absent: Vec<&str> = IGNORED
        .into_iter()
        .filter(|line| !found.lines().any(|found| found.trim() == *line))
        .collect();

    if absent.is_empty() {
        return Ok(None);
    }

    if mode != Mode::Check {
        let mut text = found.clone();

        if !text.is_empty() && !text.ends_with('\n') {
            text.push('\n');
        }

        if !text.is_empty() {
            text.push('\n');
        }

        for line in absent {
            text.push_str(line);
            text.push('\n');
        }

        write(&path, &text)?;
    }

    Ok(Some(if found.is_empty() {
        Outcome::Created
    } else {
        Outcome::Updated
    }))
}

fn write(path: &Path, text: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot write in `{}`", parent.display()))?;
    }

    fs::write(path, text).with_context(|| format!("cannot write `{}`", path.display()))
}

fn report(out: &mut impl Write, outcome: Outcome, label: &str) -> Result<()> {
    let word = match outcome {
        Outcome::Kept => "Kept",
        Outcome::Created => "Created",
        Outcome::Updated => "Updated",
        Outcome::Removed => "Removed",
        Outcome::Skipped => "Skipped",
    };

    writeln!(out, "  {BOLD}{word:<7}{BOLD:#}  {label}")?;

    Ok(())
}
