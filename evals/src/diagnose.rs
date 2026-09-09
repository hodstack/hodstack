use std::fs;
use std::io::Write as _;
use std::path::Path;
use std::process::{Command, Stdio};

use anyhow::{Context as _, Result, bail};

use crate::case::Case;
use crate::grade::Verdict;
use crate::run::Outcome;

const ROLE: &str = "You read one eval case that failed and you write the prompt that another agent will use to repair it. The working directory holds the project as the agent left it, thus read that project and read the text of the skill in `.claude/skills/` before you answer. Name the exact file and the exact line of each claim. Say which of three things is wrong: the text of the skill, the case, or the harness. Answer with the prompt and with nothing else, and write no preamble.";

const SHAPE: &str = "Write the prompt under these headings, in this order, and write nothing before the first heading or after the last one.\n\n# Repair <the identifier of the case>\n\n## What the case measures\n\n## What the agent did\n\nGive the commands that decided the outcome, and give them in order.\n\n## Where it broke\n\nName the grader that failed, or the part of the expectation that failed, and give the answer that the run gave against the answer that the case wants.\n\n## The cause\n\nSay whether the text of the skill is wrong, the case is wrong, or the harness is wrong. Give the path of the file that carries the fault.\n\n## The change to make\n\nGive the change as an instruction that an agent can obey, with the path of each file that it touches. Give the command that shows that the change worked.";

const EFFORT: &str = "high";
const TOOLS: &str = "Read,Grep,Glob,Bash";
const TURNS: &str = "40";

pub fn about(
    case: &Case,
    outcome: &Outcome,
    verdicts: &[Verdict],
    model: &str,
    skills: &Path,
    into: &Path,
) -> Result<()> {
    let dir = outcome
        .dir
        .with_file_name(format!("{}-{}-diagnosis", case.skill, case.name));

    crate::base::clone(&outcome.dir, &dir)?;

    let answer = ask(&brief(case, outcome, verdicts, skills), model, &dir);

    fs::remove_dir_all(&dir).ok();

    let text = answer?;

    if text.trim().is_empty() {
        bail!("the diagnosis is empty")
    }

    if let Some(parent) = into.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot write `{}`", parent.display()))?;
    }

    fs::write(into, text).with_context(|| format!("cannot write `{}`", into.display()))
}

fn brief(case: &Case, outcome: &Outcome, verdicts: &[Verdict], skills: &Path) -> String {
    let graders: Vec<String> = verdicts
        .iter()
        .map(|verdict| {
            format!(
                "- {} {} — {}",
                match verdict.passed {
                    true => "passed",
                    false => "FAILED",
                },
                verdict.label,
                verdict.detail
            )
        })
        .collect();

    let commands: Vec<String> = outcome
        .trace
        .calls
        .iter()
        .map(|call| format!("- {}: {}", call.tool, call.input))
        .collect();

    format!(
        "{SHAPE}\n\n## The case\n\n`{}`, in `{}`.\n\nThe skill under test is `{}`. Its text ships from `{}`, and the run held a copy at `.claude/skills/{}/SKILL.md`.\n\nThe run took {:.0} seconds and it {}.\n\n## The expectation of the case\n\n{}\n\n## The verdict of each grader\n\n{}\n\n## The report of the agent\n\n{}\n\n## The commands that the agent ran\n\n{}\n",
        case.id(),
        case.dir.display(),
        case.skill,
        skills.join(&case.skill).display(),
        case.skill,
        outcome.elapsed.as_secs_f64(),
        match outcome.timed_out {
            true => "reached the timeout of the case",
            false => "stopped on its own",
        },
        blank(&case.expectation),
        blank(&graders.join("\n")),
        blank(&outcome.trace.last_message),
        blank(&commands.join("\n"))
    )
}

fn blank(text: &str) -> &str {
    match text.trim().is_empty() {
        true => "(nothing)",
        false => text,
    }
}

fn ask(prompt: &str, model: &str, dir: &Path) -> Result<String> {
    let mut child = Command::new("claude")
        .arg("-p")
        .arg("--model")
        .arg(model)
        .arg("--effort")
        .arg(EFFORT)
        .arg("--max-turns")
        .arg(TURNS)
        .arg("--allowed-tools")
        .arg(TOOLS)
        .arg("--permission-mode")
        .arg("dontAsk")
        .arg("--system-prompt")
        .arg(ROLE)
        .arg("--disable-slash-commands")
        .current_dir(dir)
        .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1")
        .env("DISABLE_TELEMETRY", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("cannot start the diagnosis")?;

    child
        .stdin
        .take()
        .context("the diagnosis takes no prompt")?
        .write_all(prompt.as_bytes())
        .context("cannot give the prompt to the diagnosis")?;

    let output = child
        .wait_with_output()
        .context("cannot read the diagnosis")?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::{Arm, Trace};
    use std::path::PathBuf;
    use std::time::Duration;

    fn a_case() -> Case {
        let dir = PathBuf::from("/repo/evals/deps-upgrade/raises-the-minor");

        Case {
            skill: "deps-upgrade".to_owned(),
            name: "raises-the-minor".to_owned(),
            dir,
            base: "catalogue".to_owned(),
            threshold: 1.0,
            timeout_seconds: 900,
            allowed_tools: vec!["Bash".to_owned()],
            prompt: None,
            intent: None,
            graders: Vec::new(),
            expectation: "The agent raised the minor.".to_owned(),
        }
    }

    fn an_outcome() -> Outcome {
        Outcome {
            dir: PathBuf::from("/repo/evals/.runs/deps-upgrade-catalogue"),
            trace: Trace {
                calls: vec![crate::run::Call {
                    tool: "Bash".to_owned(),
                    input: r#"{"command":"composer outdated --direct"}"#.to_owned(),
                }],
                last_message: "I raised nothing.".to_owned(),
                cost_usd: 0.1,
                turns: 3,
                failed: false,
            },
            elapsed: Duration::from_secs(42),
            timed_out: false,
        }
    }

    fn verdicts() -> Vec<Verdict> {
        vec![
            Verdict {
                label: "Bash ran `composer outdated --direct`".to_owned(),
                passed: true,
                detail: "found".to_owned(),
            },
            Verdict {
                label: "`composer.json` does not hold `1\\.0\\.0`".to_owned(),
                passed: false,
                detail: "found".to_owned(),
            },
        ]
    }

    #[test]
    fn the_brief_names_the_case_the_skill_and_the_grader_that_failed() {
        let text = brief(
            &a_case(),
            &an_outcome(),
            &verdicts(),
            Path::new("/repo/skills/skills"),
        );

        assert!(text.contains("deps-upgrade/raises-the-minor"));
        assert!(text.contains("/repo/evals/deps-upgrade/raises-the-minor"));
        assert!(text.contains("/repo/skills/skills/deps-upgrade"));
        assert!(text.contains("FAILED `composer.json` does not hold"));
        assert!(text.contains("passed Bash ran `composer outdated --direct`"));
        assert!(text.contains("I raised nothing."));
        assert!(text.contains("composer outdated --direct"));
        assert!(text.contains("The agent raised the minor."));
        assert!(text.contains("stopped on its own"));
    }

    #[test]
    fn the_brief_says_when_the_run_reached_its_timeout() {
        let mut outcome = an_outcome();
        outcome.timed_out = true;

        let text = brief(
            &a_case(),
            &outcome,
            &verdicts(),
            Path::new("/repo/skills/skills"),
        );

        assert!(text.contains("reached the timeout of the case"));
    }

    #[test]
    fn a_brief_without_a_report_reads_as_nothing() {
        let mut outcome = an_outcome();
        outcome.trace.last_message = String::new();
        outcome.trace.calls = Vec::new();

        let text = brief(
            &a_case(),
            &outcome,
            &verdicts(),
            Path::new("/repo/skills/skills"),
        );

        assert!(text.contains("(nothing)"));
    }

    #[test]
    fn the_brief_carries_the_shape_of_the_answer() {
        let text = brief(
            &a_case(),
            &an_outcome(),
            &verdicts(),
            Path::new("/repo/skills/skills"),
        );

        for heading in [
            "## What the case measures",
            "## What the agent did",
            "## Where it broke",
            "## The cause",
            "## The change to make",
        ] {
            assert!(text.contains(heading), "the brief needs `{heading}`");
        }
    }

    #[test]
    fn an_arm_carries_a_label_for_the_name_of_the_file() {
        assert_eq!(Arm::With.label(), "with");
        assert_eq!(Arm::Without.label(), "without");
    }
}
