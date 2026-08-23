use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};
use serde::Serialize;

use crate::grade::Verdict;
use crate::style;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Grader {
    pub label: String,
    pub passed: bool,
    pub detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Run {
    pub run: usize,
    pub score: f64,
    pub passed: bool,
    pub cost_usd: f64,
    pub duration_seconds: f64,
    pub timed_out: bool,
    pub graders: Vec<Grader>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Arms {
    pub with: Vec<Run>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub without: Vec<Run>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Case {
    pub name: String,
    pub threshold: f64,
    pub score: f64,
    pub passed: bool,
    pub arms: Arms,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Aggregates {
    pub pass_rate: f64,
    pub cost_usd: f64,
    pub duration_seconds: f64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Suite {
    pub schema_version: &'static str,
    pub model: String,
    pub judge_model: String,
    pub total_cases: usize,
    pub total_runs: usize,
    pub cases: Vec<Case>,
    pub aggregates: Aggregates,
}

pub fn grader(verdict: &Verdict) -> Grader {
    Grader {
        label: verdict.label.clone(),
        passed: verdict.passed,
        detail: verdict.detail.clone(),
    }
}

pub fn build(model: &str, judge: &str, mut cases: Vec<Case>, wall: f64) -> Suite {
    cases.sort_by(|one, other| one.name.cmp(&other.name));

    let runs: usize = cases
        .iter()
        .map(|case| case.arms.with.len() + case.arms.without.len())
        .sum();

    let passed = cases.iter().filter(|case| case.passed).count();

    let cost = cases
        .iter()
        .flat_map(|case| case.arms.with.iter().chain(case.arms.without.iter()))
        .map(|run| run.cost_usd)
        .sum();

    let rate = match cases.is_empty() {
        true => 1.0,
        false => passed as f64 / cases.len() as f64,
    };

    Suite {
        schema_version: "1",
        model: model.to_owned(),
        judge_model: judge.to_owned(),
        total_cases: cases.len(),
        total_runs: runs,
        cases,
        aggregates: Aggregates {
            pass_rate: rate,
            cost_usd: cost,
            duration_seconds: wall,
        },
    }
}

pub fn rate(runs: &[Run]) -> f64 {
    if runs.is_empty() {
        return 0.0;
    }

    let whole = runs.iter().filter(|run| run.passed).count();

    whole as f64 / runs.len() as f64
}

pub fn write(suite: &Suite, file: &Path) -> Result<()> {
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot write `{}`", parent.display()))?;
    }

    let text = serde_json::to_string_pretty(suite).context("cannot write the result")?;

    fs::write(file, text).with_context(|| format!("cannot write `{}`", file.display()))
}

pub fn table(suite: &Suite, ablation: bool) -> String {
    let mut lines = vec![String::new()];

    for skill in skills(suite) {
        let mine: Vec<&Case> = suite
            .cases
            .iter()
            .filter(|case| group(&case.name) == skill)
            .collect();

        let sound = mine.iter().all(|case| case.passed);

        lines.push(format!(
            "   {}  {}",
            match sound {
                true => style::paint(style::ON_GREEN, " PASS "),
                false => style::paint(style::ON_RED, " FAIL "),
            },
            style::paint(style::DIM, &skill)
        ));

        for case in &mine {
            lines.push(line(case, ablation));
        }

        lines.push(String::new());
    }

    let broken: Vec<&Case> = suite.cases.iter().filter(|case| !case.passed).collect();

    if !broken.is_empty() {
        lines.push(style::paint(style::DIM, &"─".repeat(78)));
        lines.push(String::new());

        for case in broken {
            lines.push(format!(
                "   {}  {} {} {}",
                style::paint(style::ON_RED, " FAILED "),
                group(&case.name),
                style::paint(style::DIM, "›"),
                style::paint(style::BOLD, &spoken(leaf(&case.name)))
            ));

            for verdict in case.arms.with.iter().flat_map(|run| &run.graders) {
                if !verdict.passed {
                    lines.push(format!(
                        "  {} {}",
                        style::paint(style::RED, "⨯"),
                        verdict.label
                    ));
                    lines.push(format!("    {}", style::paint(style::DIM, &verdict.detail)));
                }
            }

            lines.push(String::new());
        }
    }

    let failed = suite.cases.iter().filter(|case| !case.passed).count();
    let passed = suite.total_cases - failed;

    let mut tally = Vec::new();

    if failed > 0 {
        tally.push(style::paint(style::RED, &format!("{failed} failed")));
    }

    if passed > 0 {
        tally.push(style::paint(style::GREEN, &format!("{passed} passed")));
    }

    lines.push(format!(
        "  {}  {} {}",
        style::paint(style::DIM, "Cases:   "),
        tally.join(", "),
        style::paint(style::DIM, &format!("({} total)", suite.total_cases))
    ));
    lines.push(format!(
        "  {}  {} {}",
        style::paint(style::DIM, "Runs:    "),
        suite.total_runs,
        style::paint(style::DIM, &format!("on {}", suite.model))
    ));
    lines.push(format!(
        "  {}  {:.2}s",
        style::paint(style::DIM, "Duration:"),
        suite.aggregates.duration_seconds
    ));
    lines.push(format!(
        "  {}  ${:.2}",
        style::paint(style::DIM, "Cost:    "),
        suite.aggregates.cost_usd
    ));
    lines.push(String::new());

    lines.join("\n")
}

fn line(case: &Case, ablation: bool) -> String {
    let name = spoken(leaf(&case.name));

    let mark = match case.passed {
        true => style::paint(style::GREEN, "✓"),
        false => style::paint(style::RED, "⨯"),
    };

    let graders = case.arms.with.first().map_or(0, |run| run.graders.len());

    let slowest = case
        .arms
        .with
        .iter()
        .map(|run| run.duration_seconds)
        .fold(0.0_f64, f64::max);

    let delta = match ablation && !case.arms.without.is_empty() {
        true => format!("  {:+.2}", case.score - rate(&case.arms.without)),
        false => String::new(),
    };

    let width = 52_usize.saturating_sub(name.chars().count());

    format!(
        "  {mark} {name} {} {}{}",
        style::paint(style::DIM, &".".repeat(width.max(1))),
        style::paint(style::DIM, &format!("{graders} graders  {slowest:>6.2}s")),
        style::paint(style::YELLOW, &delta)
    )
}

fn skills(suite: &Suite) -> Vec<String> {
    let mut found: Vec<String> = suite.cases.iter().map(|case| group(&case.name)).collect();

    found.dedup();

    found
}

fn group(name: &str) -> String {
    name.split_once('/')
        .map_or_else(|| name.to_owned(), |(skill, _)| skill.to_owned())
}

fn leaf(name: &str) -> &str {
    name.split_once('/').map_or(name, |(_, case)| case)
}

fn spoken(name: &str) -> String {
    name.replace('-', " ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(index: usize, score: f64) -> Run {
        Run {
            run: index,
            score,
            passed: score >= 1.0,
            cost_usd: 0.10,
            duration_seconds: 12.0,
            timed_out: false,
            graders: vec![Grader {
                label: "git left `composer.json` alone".to_owned(),
                passed: score >= 1.0,
                detail: "no change".to_owned(),
            }],
        }
    }

    fn case(name: &str, score: f64, passed: bool) -> Case {
        Case {
            name: name.to_owned(),
            threshold: 0.67,
            score,
            passed,
            arms: Arms {
                with: vec![run(1, score), run(2, score)],
                without: Vec::new(),
            },
        }
    }

    #[test]
    fn the_suite_counts_each_case_and_each_run() {
        let suite = build(
            "claude-sonnet-5",
            "claude-haiku-4-5",
            vec![case("b", 1.0, true), case("a", 0.5, false)],
            0.0,
        );

        assert_eq!(suite.total_cases, 2);
        assert_eq!(suite.total_runs, 4);
        assert_eq!(suite.cases[0].name, "a");
        assert!((suite.aggregates.pass_rate - 0.5).abs() < f64::EPSILON);
        assert!((suite.aggregates.cost_usd - 0.4).abs() < 1e-9);
    }

    #[test]
    fn a_case_scores_the_part_of_its_runs_that_passed_whole() {
        let runs = vec![run(1, 1.0), run(2, 0.75), run(3, 1.0)];

        assert!((rate(&runs) - 2.0 / 3.0).abs() < 1e-9);
    }

    #[test]
    fn a_grader_that_fails_in_every_run_scores_nothing() {
        let runs = vec![run(1, 0.8), run(2, 0.8), run(3, 0.8)];

        assert!((rate(&runs) - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn the_duration_of_the_suite_is_the_wall_clock_and_not_the_longest_run() {
        let mut one = case("a", 1.0, true);
        one.arms.with[0].duration_seconds = 30.0;
        one.arms.with[1].duration_seconds = 12.0;

        let suite = build("m", "j", vec![one], 44.0);

        assert!((suite.aggregates.duration_seconds - 44.0).abs() < f64::EPSILON);
    }

    #[test]
    fn the_table_names_each_case_and_marks_the_one_that_failed() {
        let suite = build(
            "m",
            "j",
            vec![case("stops-on-a-dirty-tree", 0.5, false)],
            0.0,
        );
        let table = table(&suite, false);

        assert!(table.contains("FAIL"));
        assert!(table.contains("FAILED"));
        assert!(table.contains("stops on a dirty tree"));
        assert!(table.contains("git left `composer.json` alone"));
        assert!(table.contains("1 failed"));
        assert!(table.contains("(1 total)"));
    }

    #[test]
    fn the_table_gives_the_delta_of_the_two_arms() {
        let mut one = case("raises-a-minor", 1.0, true);
        one.arms.without = vec![run(1, 0.25)];

        let suite = build("m", "j", vec![one], 0.0);

        assert!(table(&suite, true).contains("+1.00"));
        assert!(!table(&suite, false).contains("+1.00"));
    }

    #[test]
    fn a_suite_without_a_case_reads_as_a_pass() {
        let suite = build("m", "j", Vec::new(), 0.0);

        assert!((suite.aggregates.pass_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn the_result_writes_as_json() {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("out/result.json");
        let suite = build("m", "j", vec![case("a", 1.0, true)], 0.0);

        write(&suite, &file).unwrap();

        let text = fs::read_to_string(&file).unwrap();

        assert!(text.contains("\"schemaVersion\": \"1\""));
        assert!(text.contains("\"passRate\""));
        assert!(!text.contains("\"without\""));
    }
}
