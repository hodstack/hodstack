use std::env;
use std::ffi::OsString;
use std::path::PathBuf;
use std::process::{Command, ExitCode};

use anyhow::{Context as _, Result, bail};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Opening {
    Positional,
    Flag(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Agent {
    pub binary: &'static str,
    opening: Opening,
}

const AGENTS: [Agent; 5] = [
    Agent {
        binary: "claude",
        opening: Opening::Positional,
    },
    Agent {
        binary: "codex",
        opening: Opening::Positional,
    },
    Agent {
        binary: "cursor-agent",
        opening: Opening::Positional,
    },
    Agent {
        binary: "opencode",
        opening: Opening::Flag("--prompt"),
    },
    Agent {
        binary: "gemini",
        opening: Opening::Flag("-i"),
    },
];

pub fn find() -> Result<Agent> {
    if let Some(named) = env::var_os("HOD_AGENT") {
        let named = named.to_string_lossy().into_owned();

        let Some(agent) = AGENTS.iter().find(|agent| agent.binary == named) else {
            bail!("HOD_AGENT names `{named}`; hod starts {}", known())
        };

        return Ok(*agent);
    }

    let Some(agent) = AGENTS.iter().find(|agent| on_path(agent.binary).is_some()) else {
        bail!("no coding agent is on PATH; hod starts {}", known())
    };

    Ok(*agent)
}

pub fn start(agent: Agent, opening: &str) -> Result<ExitCode> {
    let mut command = Command::new(agent.binary);

    match agent.opening {
        Opening::Positional => command.arg(opening),
        Opening::Flag(flag) => command.arg(flag).arg(opening),
    };

    let status = command
        .status()
        .with_context(|| format!("cannot start `{}`", agent.binary))?;

    let Some(code) = status.code() else {
        return Ok(ExitCode::FAILURE);
    };

    Ok(match u8::try_from(code) {
        Ok(code) => ExitCode::from(code),
        Err(_) => ExitCode::FAILURE,
    })
}

fn known() -> String {
    let names: Vec<&str> = AGENTS.iter().map(|agent| agent.binary).collect();

    names.join(", ")
}

fn on_path(binary: &str) -> Option<PathBuf> {
    let path = env::var_os("PATH")?;

    for dir in env::split_paths(&path) {
        for name in names(binary) {
            let file = dir.join(name);

            if file.is_file() {
                return Some(file);
            }
        }
    }

    None
}

#[cfg(windows)]
fn names(binary: &str) -> Vec<OsString> {
    env::var("PATHEXT")
        .unwrap_or_else(|_| ".EXE;.CMD;.BAT".to_owned())
        .split(';')
        .map(|extension| OsString::from(format!("{binary}{extension}")))
        .collect()
}

#[cfg(not(windows))]
fn names(binary: &str) -> Vec<OsString> {
    vec![OsString::from(binary)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_agent_carries_a_name_in_the_fault_of_a_name_that_is_absent() {
        let message = known();

        for agent in AGENTS {
            assert!(
                message.contains(agent.binary),
                "the fault does not name `{}`",
                agent.binary
            );
        }
    }

    #[test]
    fn a_binary_that_is_absent_is_not_on_the_path() {
        assert!(on_path("a-binary-that-no-computer-holds").is_none());
    }

    #[test]
    fn the_first_agent_of_the_table_is_claude() {
        assert_eq!(AGENTS[0].binary, "claude");
        assert_eq!(AGENTS[0].opening, Opening::Positional);
    }
}
