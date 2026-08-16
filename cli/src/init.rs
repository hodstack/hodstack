use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::ExitCode;

use anstyle::Style;
use anyhow::{Context as _, Result};

const BOLD: Style = Style::new().bold();

const CLAUDE: &str = include_str!("../templates/CLAUDE.md");
const AGENTS: &str = include_str!("../templates/AGENTS.md");

const FILES: [(&str, &str); 2] = [("AGENTS.md", AGENTS), ("CLAUDE.md", CLAUDE)];

pub fn init(dir: &Path, out: &mut impl Write) -> Result<ExitCode> {
    for (name, _) in FILES {
        let path = dir.join(name);
        let exists = path
            .try_exists()
            .with_context(|| format!("cannot read `{}`", path.display()))?;

        if exists {
            writeln!(out, "  {name} already exists. Nothing was written.")?;
            return Ok(ExitCode::FAILURE);
        }
    }

    writeln!(out)?;
    for (name, content) in FILES {
        let path = dir.join(name);
        fs::write(&path, content).with_context(|| format!("cannot write `{}`", path.display()))?;
        writeln!(out, "  {BOLD}Created{BOLD:#}  {name}")?;
    }
    writeln!(out)?;

    Ok(ExitCode::SUCCESS)
}
