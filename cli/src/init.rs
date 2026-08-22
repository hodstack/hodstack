use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use anyhow::{Context as _, Result};

use crate::project::{self, Project};
use crate::sync::{self, Mode};

pub fn init(dir: &Path, out: &mut impl Write) -> Result<ExitCode> {
    let project = Project::new(dir);

    for name in [project::AGENTS, project::CLAUDE] {
        let path = project.path(name);
        let exists = path
            .try_exists()
            .with_context(|| format!("cannot read `{}`", path.display()))?;

        if exists {
            writeln!(out, "  {name} already exists. Nothing was written.")?;
            writeln!(
                out,
                "  Move its text to `{}`, then run `hod init` again.",
                project::INTENTION
            )?;

            return Ok(ExitCode::FAILURE);
        }
    }

    sync::sync(&project, Mode::Write, out)
}
