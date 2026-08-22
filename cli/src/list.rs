use std::io::Write;
use std::process::ExitCode;

use anyhow::Result;

use crate::project::Project;
use crate::skills::{self, Skill};

pub fn list(project: &Project, out: &mut impl Write) -> Result<ExitCode> {
    let shipped = skills::shipped();
    let mine = project.skills()?;

    writeln!(out)?;

    if shipped.is_empty() && mine.is_empty() {
        writeln!(out, "  No skill is installed. Run `hod update`.")?;
        writeln!(out)?;

        return Ok(ExitCode::SUCCESS);
    }

    let width = width(shipped.iter().chain(mine.iter()));

    group(
        out,
        "USER SKILLS",
        shipped.iter().filter(|skill| skill.front().user),
        width,
    )?;
    group(
        out,
        "MODEL SKILLS",
        shipped.iter().filter(|skill| !skill.front().user),
        width,
    )?;
    group(out, "PROJECT SKILLS", mine.iter(), width)?;

    Ok(ExitCode::SUCCESS)
}

fn sentence(description: &str) -> String {
    match description.split_once(". ") {
        Some((first, _)) => format!("{first}."),
        None => description.to_owned(),
    }
}

fn width<'a>(skills: impl Iterator<Item = &'a Skill>) -> usize {
    skills
        .map(|skill| skill.name.len())
        .max()
        .unwrap_or(0)
        .max(11)
}

fn group<'a>(
    out: &mut impl Write,
    heading: &str,
    skills: impl Iterator<Item = &'a Skill>,
    width: usize,
) -> Result<()> {
    let mut wrote = false;

    for skill in skills {
        if !wrote {
            writeln!(out, "{heading}")?;
            wrote = true;
        }

        let front = skill.front();

        writeln!(
            out,
            "  {:width$}  {}",
            front.name,
            sentence(&front.description)
        )?;
    }

    if wrote {
        writeln!(out)?;
    }

    Ok(())
}
