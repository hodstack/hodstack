use clap::{Parser, Subcommand};

use crate::help;

#[derive(Debug, Parser)]
#[command(
    name = "hod",
    version,
    about = "Hodstack makes coding agents more productive.",
    long_about = None,
    override_usage = "hod <command> [options]",
    disable_help_subcommand = true,
    styles = help::STYLES,
    term_width = 0,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Create AGENTS.md and CLAUDE.md in this directory")]
    Init,

    #[command(about = "Run a skill with a prompt")]
    Run {
        #[arg(help = "The name of the skill")]
        skill: String,

        #[arg(help = "The prompt that the skill receives")]
        prompt: String,
    },

    #[command(about = "List the installed skills")]
    List,

    #[command(about = "Install the newest build of hod")]
    Update {
        #[arg(long, help = "Report the newest build without an installation of it")]
        check: bool,
    },

    #[command(about = "Print a shell completion script")]
    Completions {
        #[arg(help = "The shell that receives the script")]
        shell: clap_complete::Shell,
    },
}
