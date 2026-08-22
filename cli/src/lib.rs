mod cli;
mod front;
mod help;
mod init;
mod list;
mod lock;
mod project;
mod skills;
mod sync;
mod update;

use std::env;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::OnceLock;

use anstyle::{AnsiColor, Color, Style};
use anyhow::{Context as _, Result, bail};
use clap::{CommandFactory as _, FromArgMatches as _};

use crate::cli::{Cli, Command};
use crate::project::Project;
use crate::skills::Skill;
use crate::sync::Mode;

pub use crate::init::init;

const RED: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Red)))
    .bold();

pub fn command() -> clap::Command {
    Cli::command()
        .version(version())
        .help_template(help::template())
}

fn version() -> &'static str {
    static VERSION: OnceLock<String> = OnceLock::new();

    VERSION
        .get_or_init(|| {
            let version = env!("CARGO_PKG_VERSION");

            match option_env!("HOD_COMMIT") {
                Some(commit) => format!("{version} ({})", commit.get(..7).unwrap_or(commit)),
                None => version.to_owned(),
            }
        })
        .as_str()
}

pub fn run() -> Result<ExitCode> {
    let matches = command().get_matches();
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };

    let mut out = anstream::stdout().lock();

    let Some(command) = cli.command else {
        let Some((skill, prompt)) = cli.skill.zip(cli.prompt) else {
            command().print_help()?;
            return Ok(ExitCode::SUCCESS);
        };

        let code = start(&skill, &prompt)?;

        out.flush()?;
        update::notice(&mut anstream::stderr().lock());

        return Ok(code);
    };

    let asks = !matches!(
        command,
        Command::Update { .. } | Command::Completions { .. }
    );

    let code = match command {
        Command::Init => init(&here()?, &mut out),
        Command::List => list::list(&Project::new(&here()?), &mut out),
        Command::Update {
            check,
            project,
            force,
        } => refresh(check, project, force, &mut out),
        Command::Completions { shell } => Ok(completions(shell)),
    }?;

    if asks {
        out.flush()?;
        update::notice(&mut anstream::stderr().lock());
    }

    Ok(code)
}

fn here() -> Result<PathBuf> {
    env::current_dir().context("cannot read the current directory")
}

fn mode(check: bool, force: bool) -> Mode {
    if check {
        return Mode::Check;
    }

    if force {
        return Mode::Force;
    }

    Mode::Write
}

fn refresh(check: bool, project: bool, force: bool, out: &mut impl Write) -> Result<ExitCode> {
    let here = Project::new(&here()?);
    let mode = mode(check, force);

    if project {
        if !here.exists() {
            writeln!(out)?;
            writeln!(out, "  This directory has no `.hod`. Run `hod init` first.")?;
            writeln!(out)?;

            return Ok(ExitCode::FAILURE);
        }

        return sync::sync(&here, mode, out);
    }

    let binary = update::update(check, out)?;

    if !here.exists() {
        return Ok(binary);
    }

    let files = sync::sync(&here, mode, out)?;

    Ok(if binary == ExitCode::SUCCESS {
        files
    } else {
        binary
    })
}

#[must_use]
pub fn report(error: &anyhow::Error) -> ExitCode {
    if let Some(error) = error.downcast_ref::<io::Error>() {
        if error.kind() == io::ErrorKind::BrokenPipe {
            return ExitCode::SUCCESS;
        }
    }

    let mut err = anstream::stderr().lock();
    let _ = writeln!(err, "{RED}error{RED:#}: {error:#}");

    ExitCode::FAILURE
}

fn start(name: &str, prompt: &str) -> Result<ExitCode> {
    let project = Project::new(&here()?);

    let Some(skill) = project.skill(name)? else {
        bail!("no skill has the name `{name}`; run `hod list` to name each skill")
    };

    Ok(begin(&skill, prompt))
}

#[expect(
    clippy::unimplemented,
    reason = "skeleton: `hod <skill> <prompt>` has no body today"
)]
fn begin(_skill: &Skill, _prompt: &str) -> ExitCode {
    unimplemented!("`hod <skill> <prompt>` has no body today")
}

#[expect(
    clippy::unimplemented,
    reason = "skeleton: `hod completions` has no body today"
)]
fn completions(_shell: clap_complete::Shell) -> ExitCode {
    unimplemented!("`hod completions` has no body today")
}
