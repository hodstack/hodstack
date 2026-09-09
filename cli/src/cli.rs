use clap::{Parser, Subcommand};

use crate::help;

#[derive(Debug, Parser)]
#[command(
    name = "hod",
    version,
    about = "Hodstack makes coding agents more productive.",
    long_about = None,
    override_usage = "hod <skill>\n  hod <command> [options]",
    args_conflicts_with_subcommands = true,
    disable_help_subcommand = true,
    styles = help::STYLES,
    term_width = 0,
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,

    #[arg(value_name = "skill", help = "The name of the skill")]
    pub skill: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    #[command(about = "Write the files of this project, then start the skill init")]
    Init,

    #[command(about = "List the installed skills")]
    List,

    #[command(about = "Report each path and each name in AGENTS.md that is absent")]
    Check,

    #[command(about = "Install the newest build of hod and write the project files")]
    Update {
        #[arg(long, help = "Report each change without a write of it")]
        check: bool,

        #[arg(long, help = "Write the project files without an installation of hod")]
        project: bool,

        #[arg(
            long,
            conflicts_with = "check",
            help = "Write over a project file that this program does not own"
        )]
        force: bool,
    },

    #[command(
        name = "worktree:create",
        about = "Add a linked worktree with a branch in ~/.hod/worktrees"
    )]
    WorktreeCreate {
        #[arg(help = "The name of the branch, instead of two words such as quiet-harbor")]
        branch: Option<String>,
    },

    #[command(
        name = "worktree:merge",
        about = "Merge this worktree into the main checkout, then remove it"
    )]
    WorktreeMerge {
        #[arg(help = "The message of the merge commit, instead of a question")]
        message: Option<String>,

        #[arg(long, help = "Keep the worktree and the branch after the merge")]
        keep: bool,
    },

    #[command(about = "Print a shell completion script")]
    Completions {
        #[arg(help = "The shell that receives the script")]
        shell: clap_complete::Shell,
    },
}
