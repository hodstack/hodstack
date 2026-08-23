use std::io::Write as _;
use std::process::{Command, Stdio};

use anyhow::{Context as _, Result};
use serde_json::Value;

use crate::grade::Verdict;
use crate::run::Outcome;

const ROLE: &str = "You weigh one expectation against the report of a coding agent. Answer only with the schema. Answer `false` when the report gives no evidence for one part of the expectation. Do not reward an intention: the expectation holds when the agent did the work, not when the agent said that it would.";

const SCHEMA: &str = r#"{"type":"object","properties":{"passed":{"type":"boolean"},"reason":{"type":"string"}},"required":["passed","reason"],"additionalProperties":false}"#;

const EFFORT: &str = "low";

pub fn weigh(expectation: &str, outcome: &Outcome, model: &str) -> Result<Verdict> {
    if expectation.trim().is_empty() {
        return Ok(Verdict {
            label: "the expectation".to_owned(),
            passed: true,
            detail: "no expectation".to_owned(),
        });
    }

    let answer = ask(&brief(expectation, outcome), model)?;

    let Some(answer) = answer else {
        return Ok(Verdict {
            label: shorten(expectation),
            passed: false,
            detail: "the judge gave no verdict".to_owned(),
        });
    };

    Ok(Verdict {
        label: shorten(expectation),
        passed: answer
            .get("passed")
            .and_then(Value::as_bool)
            .unwrap_or_default(),
        detail: answer
            .get("reason")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    })
}

fn brief(expectation: &str, outcome: &Outcome) -> String {
    let commands: Vec<String> = outcome
        .trace
        .calls
        .iter()
        .map(|call| format!("- {}: {}", call.tool, call.input))
        .collect();

    format!(
        "## The expectation\n\n{expectation}\n\n## The report of the agent\n\n{}\n\n## The commands that the agent ran\n\n{}\n",
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

fn shorten(expectation: &str) -> String {
    let line = expectation.split('\n').next().unwrap_or(expectation).trim();

    match line.char_indices().nth(72) {
        Some((at, _)) => format!("{}…", &line[..at]),
        None => line.to_owned(),
    }
}

fn ask(prompt: &str, model: &str) -> Result<Option<Value>> {
    let mut child = Command::new("claude")
        .arg("-p")
        .arg("--model")
        .arg(model)
        .arg("--effort")
        .arg(EFFORT)
        .arg("--output-format")
        .arg("json")
        .arg("--json-schema")
        .arg(SCHEMA)
        .arg("--permission-mode")
        .arg("dontAsk")
        .arg("--system-prompt")
        .arg(ROLE)
        .arg("--disable-slash-commands")
        .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1")
        .env("DISABLE_TELEMETRY", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .context("cannot start the judge")?;

    child
        .stdin
        .take()
        .context("the judge takes no prompt")?
        .write_all(prompt.as_bytes())
        .context("cannot give the prompt to the judge")?;

    let output = child.wait_with_output().context("cannot read the judge")?;
    let text = String::from_utf8_lossy(&output.stdout);

    Ok(answer(&text))
}

fn answer(text: &str) -> Option<Value> {
    let event: Value = serde_json::from_str(text.trim()).ok()?;

    if let Some(structured) = event.get("structured_output") {
        if structured.get("passed").is_some() {
            return Some(structured.clone());
        }
    }

    let inner = event.get("result").and_then(Value::as_str)?;

    let parsed: Value = serde_json::from_str(inner.trim()).ok()?;

    parsed.get("passed").is_some().then_some(parsed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::{Call, Trace};
    use std::path::PathBuf;
    use std::time::Duration;

    fn outcome(last: &str) -> Outcome {
        Outcome {
            dir: PathBuf::new(),
            trace: Trace {
                calls: vec![Call {
                    tool: "Bash".to_owned(),
                    input: r#"{"command":"composer outdated --direct"}"#.to_owned(),
                }],
                last_message: last.to_owned(),
                cost_usd: 0.0,
                turns: 0,
                failed: false,
            },
            elapsed: Duration::ZERO,
            timed_out: false,
        }
    }

    #[test]
    fn the_brief_carries_the_expectation_the_report_and_each_command() {
        let brief = brief("It asked nothing.", &outcome("I raised two packages."));

        assert!(brief.contains("It asked nothing."));
        assert!(brief.contains("## The expectation"));
        assert!(brief.contains("I raised two packages."));
        assert!(brief.contains("composer outdated --direct"));
    }

    #[test]
    fn a_report_that_is_empty_reads_as_nothing() {
        let brief = brief("It stopped.", &outcome("   "));

        assert!(brief.contains("(nothing)"));
    }

    #[test]
    fn an_expectation_without_a_verdict_needs_no_judge() {
        let verdict = weigh("  ", &outcome(""), "claude-haiku-4-5").unwrap();

        assert!(verdict.passed);
    }

    #[test]
    fn a_long_expectation_gives_a_short_label() {
        let label = shorten(
            "The agent raised every package that its own report listed as a minor version and it asked the user nothing at all.",
        );

        assert!(label.ends_with('…'));
        assert!(label.chars().count() <= 73);
    }

    #[test]
    fn the_judge_reads_a_structured_answer() {
        let found =
            answer(r#"{"type":"result","structured_output":{"passed":true,"reason":"it did"}}"#);

        assert_eq!(found.unwrap().get("passed").unwrap(), &Value::Bool(true));
    }

    #[test]
    fn the_judge_reads_an_answer_that_arrives_as_text() {
        let found = answer(r#"{"type":"result","result":"{\"passed\":false,\"reason\":\"no\"}"}"#);

        assert_eq!(found.unwrap().get("passed").unwrap(), &Value::Bool(false));
    }

    #[test]
    fn a_judge_that_says_nothing_useful_gives_no_answer() {
        assert!(answer("not json").is_none());
        assert!(answer(r#"{"type":"result","result":"I think it passed."}"#).is_none());
    }
}
