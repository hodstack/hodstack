mod cli;
mod help;
mod init;

use std::env;
use std::io::{self, Write as _};
use std::process::ExitCode;

use anstyle::{AnsiColor, Color, Style};
use anyhow::{Context as _, Result};
use clap::{CommandFactory as _, FromArgMatches as _};

use crate::cli::{Cli, Command};

pub use crate::init::init;

const RED: Style = Style::new()
    .fg_color(Some(Color::Ansi(AnsiColor::Red)))
    .bold();

pub fn command() -> clap::Command {
    Cli::command().help_template(help::template())
}

pub fn run() -> Result<ExitCode> {
    let matches = command().get_matches();
    let cli = match Cli::from_arg_matches(&matches) {
        Ok(cli) => cli,
        Err(error) => error.exit(),
    };

    let Some(command) = cli.command else {
        command().print_help()?;
        return Ok(ExitCode::SUCCESS);
    };

    let mut out = anstream::stdout().lock();

    match command {
        Command::Init => {
            let dir = env::current_dir().context("cannot read the current directory")?;
            init(&dir, &mut out)
        }
        Command::Run { skill, prompt } => Ok(run_skill(&skill, &prompt)),
        Command::List => Ok(list()),
        Command::Completions { shell } => Ok(completions(shell)),
    }
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

#[expect(
    clippy::unimplemented,
    reason = "skeleton: `hod run` has no body today"
)]
fn run_skill(_skill: &str, _prompt: &str) -> ExitCode {
    unimplemented!("`hod run` has no body today")
}

#[expect(
    clippy::unimplemented,
    reason = "skeleton: `hod list` has no body today"
)]
fn list() -> ExitCode {
    unimplemented!("`hod list` has no body today")
}

#[expect(
    clippy::unimplemented,
    reason = "skeleton: `hod completions` has no body today"
)]
fn completions(_shell: clap_complete::Shell) -> ExitCode {
    unimplemented!("`hod completions` has no body today")
}
