use std::fs;
use std::path::Path;
use std::process::Command;

use anyhow::{Context as _, Result};
use regex::Regex;

use crate::case::{Call as Wanted, Grader, Match, Target};
use crate::run::{Call, Outcome};

#[derive(Debug, Clone)]
pub struct Verdict {
    pub label: String,
    pub passed: bool,
    pub detail: String,
}

pub fn apply(graders: &[Grader], outcome: &Outcome) -> Result<Vec<Verdict>> {
    graders
        .iter()
        .map(|grader| {
            let (passed, detail) = weigh(grader, outcome)?;

            Ok(Verdict {
                label: grader.label(),
                passed,
                detail,
            })
        })
        .collect()
}

fn weigh(grader: &Grader, outcome: &Outcome) -> Result<(bool, String)> {
    match grader {
        Grader::ToolUsed {
            tool,
            input_match,
            min,
            max,
        } => {
            let found = count(
                &outcome.trace.calls,
                tool.as_deref(),
                input_match.as_deref(),
            )?;

            let enough = found >= *min;
            let spare = max.is_none_or(|ceiling| found <= ceiling);

            Ok((enough && spare, format!("{found} call(s)")))
        }
        Grader::ToolOrder { before, after } => {
            let first = position(&outcome.trace.calls, before)?;
            let second = position(&outcome.trace.calls, after)?;

            match (first, second) {
                (Some(first), Some(second)) => {
                    Ok((first < second, format!("call {first} then call {second}")))
                }
                (None, _) => Ok((false, "the first call never ran".to_owned())),
                (_, None) => Ok((false, "the second call never ran".to_owned())),
            }
        }
        Grader::Regex {
            pattern,
            target,
            verdict,
        } => {
            let text = match target {
                Target::LastMessage => outcome.trace.last_message.clone(),
                Target::Trace => outcome
                    .trace
                    .calls
                    .iter()
                    .map(|call| call.input.as_str())
                    .collect::<Vec<&str>>()
                    .join("\n"),
            };

            let hit = Regex::new(pattern)
                .with_context(|| format!("`{pattern}` is not a pattern"))?
                .is_match(&text);

            Ok((hit == wants(*verdict), told(hit)))
        }
        Grader::FileContent {
            path,
            pattern,
            verdict,
        } => {
            let file = outcome.dir.join(path);

            let Ok(text) = fs::read_to_string(&file) else {
                return Ok((false, format!("`{path}` is absent")));
            };

            let hit = Regex::new(pattern)
                .with_context(|| format!("`{pattern}` is not a pattern"))?
                .is_match(&text);

            Ok((hit == wants(*verdict), told(hit)))
        }
        Grader::FileExists { path, exists } => {
            let there = outcome.dir.join(path).exists();

            Ok((there == *exists, told(there)))
        }
        Grader::GitClean { paths } => {
            let changed = status(&outcome.dir, paths)?;

            Ok((changed.is_empty(), summary(&changed)))
        }
        Grader::GitDirty {} => {
            let changed = status(&outcome.dir, &[])?;

            Ok((!changed.is_empty(), summary(&changed)))
        }
        Grader::HeadUnmoved {} => {
            let before = fs::read_to_string(outcome.dir.join(".eval/head")).unwrap_or_default();
            let now = git(&outcome.dir, &["rev-parse", "HEAD"])?;

            Ok((
                before.trim() == now.trim() && !now.trim().is_empty(),
                format!("HEAD is {}", short(&now)),
            ))
        }
    }
}

fn wants(verdict: Match) -> bool {
    verdict == Match::Contains
}

fn told(hit: bool) -> String {
    match hit {
        true => "found".to_owned(),
        false => "absent".to_owned(),
    }
}

fn short(revision: &str) -> String {
    revision.trim().chars().take(7).collect()
}

fn summary(changed: &[String]) -> String {
    match changed.len() {
        0 => "no change".to_owned(),
        1 => format!("`{}`", changed[0]),
        count => format!("`{}` and {} more", changed[0], count - 1),
    }
}

fn count(calls: &[Call], tool: Option<&str>, pattern: Option<&str>) -> Result<usize> {
    let pattern = compile(pattern)?;

    Ok(calls
        .iter()
        .filter(|call| tool.is_none_or(|wanted| call.tool == wanted))
        .filter(|call| pattern.as_ref().is_none_or(|it| it.is_match(&call.input)))
        .count())
}

fn position(calls: &[Call], wanted: &Wanted) -> Result<Option<usize>> {
    let pattern = compile(wanted.input_match.as_deref())?;

    Ok(calls.iter().position(|call| {
        wanted.tool.as_ref().is_none_or(|it| &call.tool == it)
            && pattern.as_ref().is_none_or(|it| it.is_match(&call.input))
    }))
}

fn compile(pattern: Option<&str>) -> Result<Option<Regex>> {
    match pattern {
        Some(pattern) => {
            Ok(Some(Regex::new(pattern).with_context(|| {
                format!("`{pattern}` is not a pattern")
            })?))
        }
        None => Ok(None),
    }
}

fn status(dir: &Path, paths: &[String]) -> Result<Vec<String>> {
    let mut arguments = vec!["status", "--porcelain"];

    if !paths.is_empty() {
        arguments.push("--");
    }

    for path in paths {
        arguments.push(path);
    }

    let text = git(dir, &arguments)?;

    Ok(text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| line.get(3..).unwrap_or(line).to_owned())
        .collect())
}

fn git(dir: &Path, arguments: &[&str]) -> Result<String> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(dir)
        .output()
        .context("cannot run `git`")?;

    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

pub fn passed(verdicts: &[Verdict]) -> bool {
    verdicts.iter().all(|verdict| verdict.passed)
}

pub fn score(verdicts: &[Verdict]) -> f64 {
    if verdicts.is_empty() {
        return 1.0;
    }

    let passed = verdicts.iter().filter(|verdict| verdict.passed).count();

    passed as f64 / verdicts.len() as f64
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::run::Trace;
    use std::path::PathBuf;
    use std::time::Duration;

    fn outcome(calls: &[(&str, &str)], last: &str, dir: PathBuf) -> Outcome {
        Outcome {
            dir,
            trace: Trace {
                calls: calls
                    .iter()
                    .map(|(tool, input)| Call {
                        tool: (*tool).to_owned(),
                        input: (*input).to_owned(),
                    })
                    .collect(),
                last_message: last.to_owned(),
                cost_usd: 0.0,
                turns: 0,
                failed: false,
            },
            elapsed: Duration::ZERO,
            timed_out: false,
        }
    }

    fn one(grader: Grader, outcome: &Outcome) -> bool {
        apply(&[grader], outcome).unwrap()[0].passed
    }

    #[test]
    fn a_tool_that_ran_with_the_pattern_passes() {
        let it = outcome(
            &[("Bash", r#"{"command":"composer outdated --direct"}"#)],
            "",
            PathBuf::new(),
        );

        assert!(one(
            Grader::ToolUsed {
                tool: Some("Bash".to_owned()),
                input_match: Some("composer outdated --direct".to_owned()),
                min: 1,
                max: None,
            },
            &it
        ));
    }

    #[test]
    fn a_grader_without_a_tool_reads_the_call_of_any_tool() {
        let it = outcome(
            &[("Bash", r#"{"command":"cat .hod/project.md"}"#)],
            "",
            PathBuf::new(),
        );

        assert!(one(
            Grader::ToolUsed {
                tool: None,
                input_match: Some("project\\.md".to_owned()),
                min: 1,
                max: None,
            },
            &it
        ));
    }

    #[test]
    fn an_order_without_a_tool_reads_the_call_of_any_tool() {
        let it = outcome(
            &[
                ("Read", r#"{"file_path":".hod/project.md"}"#),
                ("Bash", r#"{"command":"composer outdated --direct"}"#),
            ],
            "",
            PathBuf::new(),
        );

        assert!(one(
            Grader::ToolOrder {
                before: Wanted {
                    tool: None,
                    input_match: Some("project\\.md".to_owned()),
                },
                after: Wanted {
                    tool: None,
                    input_match: Some("composer outdated".to_owned()),
                },
            },
            &it
        ));
    }

    #[test]
    fn a_tool_that_ran_more_than_the_ceiling_fails() {
        let it = outcome(
            &[
                ("Bash", r#"{"command":"git commit -m one"}"#),
                ("Bash", r#"{"command":"git status"}"#),
            ],
            "",
            PathBuf::new(),
        );

        assert!(!one(
            Grader::ToolUsed {
                tool: Some("Bash".to_owned()),
                input_match: Some("git commit".to_owned()),
                min: 0,
                max: Some(0),
            },
            &it
        ));
    }

    #[test]
    fn a_tool_that_never_ran_passes_a_ceiling_of_none() {
        let it = outcome(&[("Read", "{}")], "", PathBuf::new());

        assert!(one(
            Grader::ToolUsed {
                tool: Some("Bash".to_owned()),
                input_match: Some("git commit".to_owned()),
                min: 0,
                max: Some(0),
            },
            &it
        ));
    }

    #[test]
    fn a_call_before_a_second_call_passes_the_order() {
        let it = outcome(
            &[
                ("Bash", r#"{"command":"composer require acme/one"}"#),
                ("Bash", r#"{"command":"composer bump"}"#),
            ],
            "",
            PathBuf::new(),
        );

        let grader = Grader::ToolOrder {
            before: Wanted {
                tool: Some("Bash".to_owned()),
                input_match: Some("composer require".to_owned()),
            },
            after: Wanted {
                tool: Some("Bash".to_owned()),
                input_match: Some("composer bump".to_owned()),
            },
        };

        assert!(one(grader, &it));
    }

    #[test]
    fn a_call_after_a_second_call_fails_the_order() {
        let it = outcome(
            &[
                ("Bash", r#"{"command":"composer bump"}"#),
                ("Bash", r#"{"command":"composer require acme/one"}"#),
            ],
            "",
            PathBuf::new(),
        );

        let grader = Grader::ToolOrder {
            before: Wanted {
                tool: Some("Bash".to_owned()),
                input_match: Some("composer require".to_owned()),
            },
            after: Wanted {
                tool: Some("Bash".to_owned()),
                input_match: Some("composer bump".to_owned()),
            },
        };

        assert!(!one(grader, &it));
    }

    #[test]
    fn an_order_that_needs_a_call_that_never_ran_fails() {
        let it = outcome(
            &[("Bash", r#"{"command":"composer bump"}"#)],
            "",
            PathBuf::new(),
        );

        let grader = Grader::ToolOrder {
            before: Wanted {
                tool: Some("Bash".to_owned()),
                input_match: Some("composer require".to_owned()),
            },
            after: Wanted {
                tool: Some("Bash".to_owned()),
                input_match: Some("composer bump".to_owned()),
            },
        };

        assert!(!one(grader, &it));
    }

    #[test]
    fn a_pattern_that_the_last_message_does_not_hold_passes_not_contains() {
        let it = outcome(&[], "I raised each package.", PathBuf::new());

        assert!(one(
            Grader::Regex {
                pattern: "commit".to_owned(),
                target: Target::LastMessage,
                verdict: Match::NotContains,
            },
            &it
        ));
    }

    #[test]
    fn a_file_that_holds_the_pattern_passes() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("composer.json"), r#"{"acme/one":"^2.0"}"#).unwrap();

        let it = outcome(&[], "", dir.path().to_owned());

        assert!(one(
            Grader::FileContent {
                path: "composer.json".to_owned(),
                pattern: r#""acme/one":"\^2"#.to_owned(),
                verdict: Match::Contains,
            },
            &it
        ));
    }

    #[test]
    fn a_file_that_is_absent_fails_whatever_the_pattern_wants() {
        let dir = tempfile::tempdir().unwrap();
        let it = outcome(&[], "", dir.path().to_owned());

        for verdict in [Match::Contains, Match::NotContains] {
            assert!(!one(
                Grader::FileContent {
                    path: "composer.json".to_owned(),
                    pattern: "acme".to_owned(),
                    verdict,
                },
                &it
            ));
        }
    }

    #[test]
    fn a_file_that_the_agent_did_not_write_passes_a_denial() {
        let dir = tempfile::tempdir().unwrap();
        let it = outcome(&[], "", dir.path().to_owned());

        assert!(one(
            Grader::FileExists {
                path: "tests/Upgrade.php".to_owned(),
                exists: false,
            },
            &it
        ));
    }

    fn repository() -> tempfile::TempDir {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path();

        for arguments in [
            vec!["init", "--initial-branch", "main"],
            vec!["config", "user.email", "eval@example.com"],
            vec!["config", "user.name", "eval"],
        ] {
            Command::new("git")
                .args(&arguments)
                .current_dir(at)
                .output()
                .unwrap();
        }

        fs::write(at.join("composer.json"), "{}").unwrap();

        Command::new("git")
            .args(["add", "."])
            .current_dir(at)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "-m", "seed"])
            .current_dir(at)
            .output()
            .unwrap();

        fs::create_dir_all(at.join(".eval")).unwrap();
        fs::write(at.join(".git/info/exclude"), "/.eval/\n").unwrap();

        let head = git(at, &["rev-parse", "HEAD"]).unwrap();
        fs::write(at.join(".eval/head"), head).unwrap();

        dir
    }

    #[test]
    fn a_file_that_git_did_not_see_change_is_clean() {
        let dir = repository();
        let it = outcome(&[], "", dir.path().to_owned());

        assert!(one(
            Grader::GitClean {
                paths: vec!["composer.json".to_owned()],
            },
            &it
        ));
        assert!(!one(Grader::GitDirty {}, &it));
    }

    #[test]
    fn a_file_that_the_agent_changed_is_not_clean() {
        let dir = repository();
        fs::write(dir.path().join("composer.json"), r#"{"acme/one":"^2.0"}"#).unwrap();

        let it = outcome(&[], "", dir.path().to_owned());

        assert!(!one(
            Grader::GitClean {
                paths: vec!["composer.json".to_owned()],
            },
            &it
        ));
        assert!(one(Grader::GitDirty {}, &it));
    }

    #[test]
    fn a_repository_without_a_new_commit_keeps_its_head() {
        let dir = repository();
        let it = outcome(&[], "", dir.path().to_owned());

        assert!(one(Grader::HeadUnmoved {}, &it));
    }

    #[test]
    fn a_repository_with_a_new_commit_moved_its_head() {
        let dir = repository();
        fs::write(dir.path().join("composer.json"), "changed").unwrap();

        for arguments in [vec!["add", "."], vec!["commit", "-m", "raise"]] {
            Command::new("git")
                .args(&arguments)
                .current_dir(dir.path())
                .output()
                .unwrap();
        }

        let it = outcome(&[], "", dir.path().to_owned());

        assert!(!one(Grader::HeadUnmoved {}, &it));
    }

    #[test]
    fn the_score_is_the_part_of_the_graders_that_passed() {
        let verdicts = vec![
            Verdict {
                label: "one".to_owned(),
                passed: true,
                detail: String::new(),
            },
            Verdict {
                label: "two".to_owned(),
                passed: false,
                detail: String::new(),
            },
            Verdict {
                label: "three".to_owned(),
                passed: true,
                detail: String::new(),
            },
            Verdict {
                label: "four".to_owned(),
                passed: true,
                detail: String::new(),
            },
        ];

        assert!((score(&verdicts) - 0.75).abs() < f64::EPSILON);
        assert!((score(&[]) - 1.0).abs() < f64::EPSILON);
    }
}
