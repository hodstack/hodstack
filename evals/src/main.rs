mod base;
mod case;
mod diagnose;
mod grade;
mod judge;
mod report;
mod run;
mod style;

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result};
use clap::Parser;

use crate::case::Case;
use crate::run::{Arm, Precondition, Project, Settings, Step};

#[derive(Parser)]
#[command(
    name = "hod-eval",
    about = "Run the eval cases of the Hodstack skills.",
    disable_version_flag = true
)]
struct Cli {
    #[arg(help = "A skill, or a skill and a case, such as `deps-upgrade/stops-on-a-dirty-tree`")]
    selector: Option<String>,

    #[arg(
        long,
        default_value_t = 8,
        help = "The number of skills that go at the same time"
    )]
    jobs: usize,

    #[arg(long, default_value_t = 1, help = "The number of runs of each case")]
    runs: usize,

    #[arg(
        long,
        default_value = "claude-sonnet-5",
        help = "The model that runs each case"
    )]
    model: String,

    #[arg(
        long,
        default_value = "claude-haiku-4-5",
        help = "The model that weighs each expectation"
    )]
    judge_model: String,

    #[arg(long, default_value = "low", help = "The effort of the model")]
    effort: String,

    #[arg(long, default_value_t = 60, help = "The ceiling of turns of one run")]
    max_turns: usize,

    #[arg(long, help = "The score that a case needs, from 0 to 1")]
    threshold: Option<f64>,

    #[arg(long, help = "Write the result of the suite to this file")]
    json: Option<PathBuf>,

    #[arg(long, help = "Run the arm that holds no skill and report the delta")]
    ablation: bool,

    #[arg(long, help = "Keep the directory of each run")]
    keep_runs: bool,

    #[arg(
        long,
        help = "Run each case, and do not stop at the first run that fails"
    )]
    no_bail: bool,

    #[arg(
        long,
        default_value = "claude-opus-5",
        help = "The model that diagnoses a run that failed"
    )]
    diagnose_model: String,

    #[arg(long, help = "Write no diagnosis for a run that failed")]
    no_diagnose: bool,

    #[arg(
        long,
        help = "Name each case that answers to the selector, and run nothing"
    )]
    list: bool,
}

struct Done {
    case: String,
    arm: Arm,
    run: report::Run,
    diagnosis: Option<PathBuf>,
}

struct Unit {
    case: Arc<Case>,
    arm: Arm,
    index: usize,
    precondition: Precondition,
}

struct Bucket {
    skill: String,
    base: String,
    units: Vec<Unit>,
}

enum Event {
    Along(String, Step, Duration),
    Over(Box<Done>),
}

fn column(buckets: &[Bucket]) -> usize {
    buckets
        .iter()
        .flat_map(|bucket| &bucket.units)
        .map(|unit| tag(&unit.case, unit.arm, unit.index).chars().count())
        .chain(
            buckets
                .iter()
                .flat_map(|bucket| &bucket.units)
                .map(|unit| unit.case.id().chars().count()),
        )
        .max()
        .unwrap_or_default()
        .clamp(24, 64)
}

fn clock(at: Duration) -> String {
    let seconds = at.as_secs_f64();

    match seconds < 10.0 {
        true => format!("{seconds:.1}s"),
        false => format!("{seconds:.0}s"),
    }
}

fn tag(case: &Case, arm: Arm, index: usize) -> String {
    match arm {
        Arm::With => format!("{} #{index}", case.name),
        Arm::Without => format!("{} #{index} without", case.name),
    }
}

fn main() -> ExitCode {
    match work(&Cli::parse()) {
        Ok(code) => code,
        Err(report) => {
            eprintln!("error: {report:#}");

            ExitCode::FAILURE
        }
    }
}

fn work(cli: &Cli) -> Result<ExitCode> {
    let root = root();
    let skills = root
        .parent()
        .context("the eval directory needs a parent")?
        .join("skills/skills");

    let cases = case::discover(&root, cli.selector.as_deref())?;

    if cli.list {
        for case in &cases {
            println!(
                "  {:<58}  {} graders  {} judged  {:.2}",
                case.id(),
                case.graders.len(),
                usize::from(!case.expectation.is_empty()),
                case.threshold
            );
        }

        return Ok(ExitCode::SUCCESS);
    }

    let settings = Arc::new(Settings {
        root: root.clone(),
        cache: root.join(".cache"),
        runs: root.join(".runs"),
        skills,
        model: cli.model.clone(),
        effort: cli.effort.clone(),
        max_turns: cli.max_turns,
        judge_model: cli.judge_model.clone(),
        diagnose_model: match cli.no_diagnose {
            true => None,
            false => Some(cli.diagnose_model.clone()),
        },
        keep_runs: cli.keep_runs,
    });

    for name in bases(&cases) {
        base::materialize(&root.join("bases"), &name, &settings.cache)
            .with_context(|| format!("cannot seed the base `{name}`"))?;
    }

    let buckets = plan(&cases, cli)?;
    let total: usize = buckets.iter().map(|bucket| bucket.units.len()).sum();
    let jobs = cli.jobs.clamp(1, buckets.len());
    let column = column(&buckets);

    eprintln!(
        "\n  {} {}, {} each, {} {} at a time, judged by {}\n",
        cases.len(),
        match cases.len() {
            1 => "case",
            _ => "cases",
        },
        match cli.runs {
            1 => "1 run".to_owned(),
            many => format!("{many} runs"),
        },
        jobs,
        match jobs {
            1 => "skill",
            _ => "skills",
        },
        cli.judge_model
    );

    let queue = Arc::new(Mutex::new(buckets));
    let (sender, receiver) = mpsc::channel();
    let mut workers = Vec::new();
    let began = Instant::now();
    let bailed = Arc::new(AtomicBool::new(false));

    for _ in 0..jobs {
        let queue = Arc::clone(&queue);
        let settings = Arc::clone(&settings);
        let sender = sender.clone();
        let bailed = Arc::clone(&bailed);
        let bail = !cli.no_bail;

        workers.push(thread::spawn(move || {
            while let Some(bucket) = waiting(&queue, &bailed) {
                let opened = Project::open(&bucket.skill, &bucket.base, &settings);

                let mut project = match opened {
                    Ok(project) => project,
                    Err(fault) => {
                        for unit in &bucket.units {
                            let done = broken(&unit.case, unit.arm, unit.index, &fault);

                            sender.send(Event::Over(Box::new(done))).ok();
                        }

                        continue;
                    }
                };

                for unit in bucket.units {
                    if bailed.load(Ordering::Relaxed) {
                        break;
                    }

                    let name = tag(&unit.case, unit.arm, unit.index);
                    let along = sender.clone();
                    let mine = name.clone();
                    let opened = Instant::now();

                    let done = single(&mut project, &unit, &settings, &move |step| {
                        along
                            .send(Event::Along(mine.clone(), step, opened.elapsed()))
                            .ok();
                    });

                    let failed = !done.run.passed;

                    if sender.send(Event::Over(Box::new(done))).is_err() {
                        break;
                    }

                    if bail && failed {
                        bailed.store(true, Ordering::Relaxed);

                        break;
                    }
                }

                project.close();
            }
        }));
    }

    drop(sender);

    let mut finished = 0;
    let mut collected: Vec<Done> = Vec::new();

    for event in receiver {
        match event {
            Event::Along(name, step, at) => eprintln!(
                "  {} {}  {}  {}",
                style::paint(style::DIM, "·"),
                style::paint(style::DIM, &format!("{name:<column$}")),
                style::paint(style::DIM, &format!("{:>7}", clock(at))),
                style::paint(style::DIM, &step.label())
            ),
            Event::Over(done) => {
                finished += 1;

                let mark = match done.run.passed {
                    true => style::paint(style::GREEN, "✓"),
                    false => style::paint(style::RED, "⨯"),
                };

                eprintln!(
                    "  {mark} {}  {}",
                    style::paint(style::BOLD, &format!("{:<column$}", done.case)),
                    style::paint(
                        style::DIM,
                        &format!(
                            "{finished}/{total}  {:.0}s  ${:.2}",
                            done.run.duration_seconds, done.run.cost_usd
                        )
                    )
                );

                if let Some(path) = &done.diagnosis {
                    eprintln!(
                        "    {} {}",
                        style::paint(style::DIM, "↳"),
                        style::paint(style::DIM, &path.display().to_string())
                    );
                }

                collected.push(*done);
            }
        }
    }

    for worker in workers {
        worker.join().ok();
    }

    let diagnosed: Vec<String> = collected
        .iter()
        .filter_map(|done| done.diagnosis.as_ref())
        .map(|path| path.display().to_string())
        .collect();

    let measured = match bailed.load(Ordering::Relaxed) {
        true => measured(&cases, &collected),
        false => cases.clone(),
    };

    let skipped = cases.len() - measured.len();
    let suite = assemble(&measured, collected, cli, began.elapsed().as_secs_f64());
    let table = report::table(&suite, cli.ablation);

    println!("{table}");

    if skipped > 0 {
        eprintln!(
            "  the suite stopped at the first run that failed, thus {skipped} {} did not run\n",
            match skipped {
                1 => "case",
                _ => "cases",
            }
        );
    }

    for path in &diagnosed {
        eprintln!(
            "  {} {}",
            style::paint(style::DIM, "read the diagnosis in"),
            style::paint(style::BOLD, path)
        );
    }

    if !diagnosed.is_empty() {
        eprintln!();
    }

    if let Some(file) = &cli.json {
        report::write(&suite, file)?;
    }

    report::write(&suite, &root.join(".runs/result.json"))?;

    let failed = suite.cases.iter().any(|case| !case.passed);

    Ok(match failed {
        true => ExitCode::FAILURE,
        false => ExitCode::SUCCESS,
    })
}

fn broken(case: &Case, arm: Arm, index: usize, fault: &anyhow::Error) -> Done {
    Done {
        case: case.id(),
        arm,
        diagnosis: None,
        run: report::Run {
            run: index,
            score: 0.0,
            passed: false,
            cost_usd: 0.0,
            duration_seconds: 0.0,
            timed_out: false,
            graders: vec![report::Grader {
                label: "the run".to_owned(),
                passed: false,
                detail: format!("{fault:#}"),
            }],
        },
    }
}

fn single(project: &mut Project, unit: &Unit, settings: &Settings, say: &dyn Fn(Step)) -> Done {
    let case = unit.case.as_ref();
    let arm = unit.arm;
    let index = unit.index;
    let mut diagnosis = None;

    let run = match run::perform(project, case, arm, settings, say) {
        Ok(outcome) => {
            say(Step::Grade);

            let mut verdicts = grade::apply(&case.graders, &outcome).unwrap_or_default();

            say(Step::Judge);

            match judge::weigh(&case.expectation, &outcome, &settings.judge_model) {
                Ok(verdict) => verdicts.push(verdict),
                Err(_) => verdicts.push(grade::Verdict {
                    label: "the expectation".to_owned(),
                    passed: false,
                    detail: "the judge did not answer".to_owned(),
                }),
            }

            let whole = !outcome.timed_out && grade::passed(&verdicts);

            let run = report::Run {
                run: index,
                score: match outcome.timed_out {
                    true => 0.0,
                    false => grade::score(&verdicts),
                },
                passed: whole,
                cost_usd: outcome.trace.cost_usd,
                duration_seconds: outcome.elapsed.as_secs_f64(),
                timed_out: outcome.timed_out,
                graders: verdicts.iter().map(report::grader).collect(),
            };

            if settings.keep_runs {
                project.keep(case, arm, index).ok();
            }

            if let Some(model) = settings.diagnose_model.as_deref().filter(|_| !whole) {
                say(Step::Diagnose);

                let into = settings.runs.join("diagnosis").join(format!(
                    "{}-{}-{}-{index}.md",
                    case.skill,
                    case.name,
                    arm.label()
                ));

                if diagnose::about(case, &outcome, &verdicts, model, &settings.skills, &into)
                    .is_ok()
                {
                    diagnosis = Some(into);
                }
            }

            run
        }
        Err(fault) => return broken(case, arm, index, &fault),
    };

    Done {
        case: case.id(),
        arm,
        run,
        diagnosis,
    }
}

fn assemble(cases: &[Case], collected: Vec<Done>, cli: &Cli, wall: f64) -> report::Suite {
    let mut with: BTreeMap<String, Vec<report::Run>> = BTreeMap::new();
    let mut without: BTreeMap<String, Vec<report::Run>> = BTreeMap::new();

    for done in collected {
        let shelf = match done.arm {
            Arm::With => &mut with,
            Arm::Without => &mut without,
        };

        shelf.entry(done.case).or_default().push(done.run);
    }

    let built = cases
        .iter()
        .map(|case| {
            let id = case.id();
            let mut runs = with.remove(&id).unwrap_or_default();
            let mut baseline = without.remove(&id).unwrap_or_default();

            runs.sort_by_key(|run| run.run);
            baseline.sort_by_key(|run| run.run);

            let score = report::rate(&runs);
            let threshold = cli.threshold.unwrap_or(case.threshold);

            report::Case {
                name: id,
                threshold,
                score,
                passed: score >= threshold,
                arms: report::Arms {
                    with: runs,
                    without: baseline,
                },
            }
        })
        .collect();

    report::build(&cli.model, &cli.judge_model, built, wall)
}

fn plan(cases: &[Case], cli: &Cli) -> Result<Vec<Bucket>> {
    let mut buckets: Vec<Bucket> = Vec::new();

    for case in cases {
        let shared = Arc::new(case.clone());
        let mut arms = vec![Arm::With; cli.runs];

        if cli.ablation && case.baseline().is_some() {
            arms.push(Arm::Without);
        }

        for (place, arm) in arms.into_iter().enumerate() {
            let unit = Unit {
                case: Arc::clone(&shared),
                arm,
                index: match arm {
                    Arm::With => place + 1,
                    Arm::Without => 1,
                },
                precondition: run::precondition(case, arm)?,
            };

            match buckets
                .iter_mut()
                .find(|bucket| bucket.skill == case.skill && bucket.base == case.base)
            {
                Some(bucket) => bucket.units.push(unit),
                None => buckets.push(Bucket {
                    skill: case.skill.clone(),
                    base: case.base.clone(),
                    units: vec![unit],
                }),
            }
        }
    }

    for bucket in &mut buckets {
        bucket.units.sort_by(|one, other| {
            one.precondition
                .cmp(&other.precondition)
                .then_with(|| one.case.name.cmp(&other.case.name))
                .then(one.index.cmp(&other.index))
        });
    }

    Ok(buckets)
}

fn measured(cases: &[Case], collected: &[Done]) -> Vec<Case> {
    cases
        .iter()
        .filter(|case| collected.iter().any(|done| done.case == case.id()))
        .cloned()
        .collect()
}

fn waiting(queue: &Mutex<Vec<Bucket>>, bailed: &AtomicBool) -> Option<Bucket> {
    match bailed.load(Ordering::Relaxed) {
        true => None,
        false => queue.lock().ok()?.pop(),
    }
}

fn bases(cases: &[Case]) -> Vec<String> {
    let mut names: Vec<String> = cases.iter().map(|case| case.base.clone()).collect();

    names.sort();
    names.dedup();

    names
}

fn root() -> PathBuf {
    if let Some(named) = std::env::var_os("HOD_EVAL_ROOT") {
        return PathBuf::from(named);
    }

    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

    match manifest.join("bases").is_dir() {
        true => manifest,
        false => std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn a_case(root: &std::path::Path, skill: &str, name: &str, base: &str, setup: &str) -> Case {
        let dir = root.join(skill).join(name);

        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join("test.md"),
            format!("+++\nbase = \"{base}\"\nintent = \"Raise them.\"\n+++\n\nIt worked.\n"),
        )
        .unwrap();
        std::fs::write(dir.join("setup.sh"), setup).unwrap();

        case::read(&dir, skill, name).unwrap()
    }

    fn cli(arguments: &[&str]) -> Cli {
        let mut all = vec!["hod-eval"];
        all.extend_from_slice(arguments);

        Cli::parse_from(all)
    }

    #[test]
    fn one_skill_gives_one_bucket() {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path();

        let cases = vec![
            a_case(at, "deps-upgrade", "one", "laravel", "pin"),
            a_case(at, "deps-upgrade", "other", "laravel", "pin"),
            a_case(at, "init", "third", "laravel", "pin"),
        ];

        let buckets = plan(&cases, &cli(&[])).unwrap();

        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].skill, "deps-upgrade");
        assert_eq!(buckets[0].units.len(), 2);
        assert_eq!(buckets[1].skill, "init");
        assert_eq!(buckets[1].units.len(), 1);
    }

    #[test]
    fn one_skill_of_two_bases_gives_one_bucket_for_one_base() {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path();

        let cases = vec![
            a_case(at, "deps-upgrade", "one", "laravel", "pin"),
            a_case(at, "deps-upgrade", "other", "cargo", "pin"),
        ];

        let buckets = plan(&cases, &cli(&[])).unwrap();

        assert_eq!(buckets.len(), 2);
        assert_eq!(buckets[0].base, "laravel");
        assert_eq!(buckets[1].base, "cargo");
    }

    #[test]
    fn the_units_of_one_precondition_sit_next_to_each_other() {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path();

        let cases = vec![
            a_case(at, "deps-upgrade", "one", "laravel", "pin the csv"),
            a_case(at, "deps-upgrade", "other", "laravel", "break a test"),
            a_case(at, "deps-upgrade", "third", "laravel", "pin the csv"),
        ];

        let buckets = plan(&cases, &cli(&[])).unwrap();
        let units = &buckets[0].units;

        assert_eq!(units.len(), 3);
        assert_eq!(setups(units), 2);
    }

    #[test]
    fn each_run_of_one_case_reads_as_one_precondition() {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path();

        let cases = vec![a_case(at, "deps-upgrade", "one", "laravel", "pin")];

        let buckets = plan(&cases, &cli(&["--runs", "3"])).unwrap();
        let units = &buckets[0].units;

        assert_eq!(units.len(), 3);
        assert_eq!(setups(units), 1);
        assert_eq!(
            units.iter().map(|unit| unit.index).collect::<Vec<usize>>(),
            vec![1, 2, 3]
        );
    }

    #[test]
    fn the_arm_that_holds_no_skill_reads_as_its_own_precondition() {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path();

        let cases = vec![a_case(at, "deps-upgrade", "one", "laravel", "pin")];

        let buckets = plan(&cases, &cli(&["--ablation"])).unwrap();
        let units = &buckets[0].units;

        assert_eq!(units.len(), 2);
        assert_eq!(setups(units), 2);
        assert!(units.iter().any(|unit| unit.arm == Arm::Without));
    }

    #[test]
    fn the_column_fits_the_widest_name_of_the_run() {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path();

        let cases = vec![
            a_case(at, "deps-upgrade", "one", "laravel", "pin"),
            a_case(
                at,
                "deps-upgrade",
                "asks-before-the-major-that-touches-the-project",
                "laravel",
                "pin",
            ),
        ];

        let buckets = plan(&cases, &cli(&["--ablation"])).unwrap();
        let wide = tag(&buckets[0].units[0].case, Arm::Without, 1);

        assert!(column(&buckets) >= wide.chars().count().min(64));
        assert!(column(&buckets) <= 64);
    }

    #[test]
    fn a_column_of_no_unit_still_reads() {
        assert_eq!(column(&[]), 24);
    }

    #[test]
    fn the_clock_of_a_step_reads_short_under_ten_seconds_and_whole_above() {
        assert_eq!(clock(Duration::from_millis(0)), "0.0s");
        assert_eq!(clock(Duration::from_millis(540)), "0.5s");
        assert_eq!(clock(Duration::from_millis(9940)), "9.9s");
        assert_eq!(clock(Duration::from_secs(12)), "12s");
        assert_eq!(clock(Duration::from_secs(108)), "108s");
    }

    #[test]
    fn a_suite_that_stopped_reports_the_cases_that_ran() {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path();

        let cases = vec![
            a_case(at, "deps-upgrade", "one", "laravel", "pin"),
            a_case(at, "deps-upgrade", "other", "laravel", "pin"),
        ];

        let collected = vec![broken(
            &cases[0],
            Arm::With,
            1,
            &anyhow::anyhow!("the agent stopped"),
        )];

        let ran = measured(&cases, &collected);

        assert_eq!(ran.len(), 1);
        assert_eq!(ran[0].name, "one");
    }

    #[test]
    fn a_queue_gives_no_bucket_after_the_suite_stopped() {
        let dir = tempfile::tempdir().unwrap();
        let cases = vec![a_case(dir.path(), "deps-upgrade", "one", "laravel", "pin")];
        let queue = Mutex::new(plan(&cases, &cli(&[])).unwrap());
        let bailed = AtomicBool::new(false);

        assert!(waiting(&queue, &bailed).is_some());

        let queue = Mutex::new(plan(&cases, &cli(&[])).unwrap());
        bailed.store(true, Ordering::Relaxed);

        assert!(waiting(&queue, &bailed).is_none());
    }

    fn setups(units: &[Unit]) -> usize {
        units
            .windows(2)
            .filter(|pair| pair[0].precondition != pair[1].precondition)
            .count()
            + 1
    }
}
