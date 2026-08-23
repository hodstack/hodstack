use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::{Context as _, Result};
use serde_json::Value;

use crate::base;
use crate::case::Case;

#[derive(Debug, Clone)]
pub enum Step {
    Clone,
    Reset,
    Setup,
    Agent,
    Call(String, String),
    Grade,
    Judge,
    Diagnose,
}

impl Step {
    pub fn label(&self) -> String {
        match self {
            Self::Clone => "clone the base".to_owned(),
            Self::Reset => "reset the project".to_owned(),
            Self::Setup => "run setup.sh".to_owned(),
            Self::Agent => "start the agent".to_owned(),
            Self::Call(tool, what) => format!("{tool}  {what}"),
            Self::Grade => "weigh each grader".to_owned(),
            Self::Judge => "weigh the expectation".to_owned(),
            Self::Diagnose => "diagnose the failure".to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arm {
    With,
    Without,
}

impl Arm {
    pub fn label(self) -> &'static str {
        match self {
            Self::With => "with",
            Self::Without => "without",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Call {
    pub tool: String,
    pub input: String,
}

#[derive(Debug, Default)]
pub struct Trace {
    pub calls: Vec<Call>,
    pub last_message: String,
    pub cost_usd: f64,
    pub turns: usize,
    pub failed: bool,
}

#[derive(Debug)]
pub struct Outcome {
    pub dir: PathBuf,
    pub trace: Trace,
    pub elapsed: Duration,
    pub timed_out: bool,
}

pub struct Settings {
    pub root: PathBuf,
    pub cache: PathBuf,
    pub runs: PathBuf,
    pub skills: PathBuf,
    pub model: String,
    pub effort: String,
    pub max_turns: usize,
    pub judge_model: String,
    pub diagnose_model: Option<String>,
    pub keep_runs: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct Precondition(Vec<u8>);

pub fn precondition(case: &Case, arm: Arm) -> Result<Precondition> {
    let mut bytes = Vec::new();

    bytes.extend_from_slice(arm.label().as_bytes());
    bytes.push(0);
    bytes.extend_from_slice(&fs::read(case.setup()).unwrap_or_default());
    bytes.push(0);

    for (path, content) in base::contents(&case.fixtures())? {
        bytes.extend_from_slice(path.as_bytes());
        bytes.push(0);
        bytes.extend_from_slice(&content);
        bytes.push(0);
    }

    Ok(Precondition(bytes))
}

pub struct Project {
    dir: PathBuf,
    slot: PathBuf,
    base: PathBuf,
    skills: PathBuf,
    held: Option<Precondition>,
}

impl Project {
    pub fn open(skill: &str, base: &str, settings: &Settings) -> Result<Self> {
        let seeded = base::materialize(&settings.root.join("bases"), base, &settings.cache)?;

        Ok(Self {
            dir: settings.runs.join(format!("{skill}-{base}")),
            slot: settings.runs.join(format!("{skill}-{base}.ready")),
            base: seeded,
            skills: settings.skills.clone(),
            held: None,
        })
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn ready(&mut self, case: &Case, arm: Arm, say: &dyn Fn(Step)) -> Result<()> {
        let wanted = precondition(case, arm)?;

        if self.held.as_ref() == Some(&wanted) {
            say(Step::Reset);

            return base::clone(&self.slot, &self.dir);
        }

        self.held = None;

        say(Step::Clone);

        base::clone(&self.base, &self.dir)?;
        base::overlay(&case.fixtures(), &self.dir)?;

        if arm == Arm::With {
            install(case, &self.skills, &self.dir)?;
        }

        fs::create_dir_all(self.dir.join(".eval"))
            .with_context(|| format!("cannot write `{}`", self.dir.join(".eval").display()))?;

        baseline(&self.dir)?;

        say(Step::Setup);

        prepare(case, &self.dir)?;
        remember(&self.dir)?;

        base::clone(&self.dir, &self.slot)?;

        self.held = Some(wanted);

        Ok(())
    }

    pub fn keep(&self, case: &Case, arm: Arm, index: usize) -> Result<()> {
        let kept = self.dir.with_file_name(format!(
            "{}-{}-{}-{index}",
            case.skill,
            case.name,
            arm.label()
        ));

        base::clone(&self.dir, &kept)
    }

    pub fn close(self) {
        fs::remove_dir_all(&self.dir).ok();
        fs::remove_dir_all(&self.slot).ok();
    }
}

pub fn perform(
    project: &mut Project,
    case: &Case,
    arm: Arm,
    settings: &Settings,
    say: &dyn Fn(Step),
) -> Result<Outcome> {
    project.ready(case, arm, say)?;

    let dir = project.dir().to_owned();

    let started = Instant::now();
    let trace_file = dir.join(".eval/trace.jsonl");
    let error_file = dir.join(".eval/stderr.log");

    say(Step::Agent);

    let mut child = spawn(case, arm, &dir, settings, &trace_file, &error_file)?;
    let timed_out = watch(
        &mut child,
        Duration::from_secs(case.timeout_seconds),
        &trace_file,
        say,
    )?;
    let elapsed = started.elapsed();

    let trace = read(&trace_file)?;

    Ok(Outcome {
        dir,
        trace,
        elapsed,
        timed_out,
    })
}

fn install(case: &Case, skills: &Path, dir: &Path) -> Result<()> {
    let from = skills.join(&case.skill);
    let to = dir.join(".claude/skills").join(&case.skill);

    fs::create_dir_all(&to).with_context(|| format!("cannot write `{}`", to.display()))?;

    base::overlay(&from, &to)
}

fn prepare(case: &Case, dir: &Path) -> Result<()> {
    let setup = case.setup();

    if !setup.is_file() {
        return Ok(());
    }

    let log = File::create(dir.join(".setup.log"))
        .with_context(|| format!("cannot write `{}`", dir.join(".setup.log").display()))?;

    let status = Command::new("sh")
        .arg(&setup)
        .current_dir(dir)
        .env("CASE", case.id())
        .stdout(log.try_clone().context("cannot open the setup log")?)
        .stderr(log)
        .status()
        .with_context(|| format!("cannot run `{}`", setup.display()))?;

    anyhow::ensure!(status.success(), "`{}` failed", setup.display());

    fs::remove_file(dir.join(".setup.log")).ok();

    Ok(())
}

const HIDDEN: [&str; 3] = ["/.eval/", "/.claude/", "/.setup.log"];

fn baseline(dir: &Path) -> Result<()> {
    if !dir.join(".git").exists() {
        seed(dir, &["init", "--quiet", "--initial-branch", "main"])?;
    }

    seed(dir, &["config", "user.email", "eval@hodstack.invalid"])?;
    seed(dir, &["config", "user.name", "hod-eval"])?;

    let exclude = dir.join(".git/info/exclude");

    if let Some(parent) = exclude.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot write `{}`", parent.display()))?;
    }

    let mut ignored = fs::read_to_string(&exclude).unwrap_or_default();

    for path in HIDDEN {
        if !ignored.lines().any(|line| line.trim() == path) {
            ignored.push_str(path);
            ignored.push('\n');
        }
    }

    fs::write(&exclude, ignored)
        .with_context(|| format!("cannot write `{}`", exclude.display()))?;

    seed(dir, &["add", "-A"])?;
    seed(
        dir,
        &["commit", "--quiet", "--allow-empty", "-m", "the base"],
    )?;

    Ok(())
}

fn seed(dir: &Path, arguments: &[&str]) -> Result<()> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(dir)
        .output()
        .context("cannot run `git`")?;

    anyhow::ensure!(
        output.status.success(),
        "`git {}` failed: {}",
        arguments.join(" "),
        String::from_utf8_lossy(&output.stderr).trim()
    );

    Ok(())
}

fn remember(dir: &Path) -> Result<()> {
    let output = Command::new("git")
        .args(["rev-parse", "HEAD"])
        .current_dir(dir)
        .output()
        .context("cannot run `git`")?;

    fs::write(dir.join(".eval/head"), output.stdout)
        .with_context(|| format!("cannot write `{}`", dir.join(".eval/head").display()))
}

fn spawn(
    case: &Case,
    arm: Arm,
    dir: &Path,
    settings: &Settings,
    trace: &Path,
    errors: &Path,
) -> Result<Child> {
    let mut command = Command::new("claude");

    let opening = match arm {
        Arm::With => case.opening(),
        Arm::Without => case.baseline().unwrap_or_default().to_owned(),
    };

    command
        .arg("-p")
        .arg(opening)
        .arg("--output-format")
        .arg("stream-json")
        .arg("--verbose")
        .arg("--permission-mode")
        .arg("dontAsk")
        .arg("--allowed-tools")
        .arg(case.allowed_tools.join(","))
        .arg("--model")
        .arg(&settings.model)
        .arg("--effort")
        .arg(&settings.effort)
        .arg("--max-turns")
        .arg(settings.max_turns.to_string())
        .current_dir(dir)
        .env("CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC", "1")
        .env("DISABLE_TELEMETRY", "1")
        .env("DISABLE_ERROR_REPORTING", "1")
        .env("HOD_NO_UPDATE_CHECK", "1")
        .stdin(Stdio::null())
        .stdout(File::create(trace).with_context(|| format!("cannot write `{}`", trace.display()))?)
        .stderr(
            File::create(errors).with_context(|| format!("cannot write `{}`", errors.display()))?,
        );

    if std::env::var_os("ANTHROPIC_API_KEY").is_some() {
        let home = dir.join(".eval/home");

        fs::create_dir_all(&home).with_context(|| format!("cannot write `{}`", home.display()))?;

        command.env("HOME", &home);
        command.env("CLAUDE_CONFIG_DIR", home.join(".claude"));
    }

    command.spawn().context("cannot start `claude`")
}

fn watch(child: &mut Child, limit: Duration, trace: &Path, say: &dyn Fn(Step)) -> Result<bool> {
    let deadline = Instant::now() + limit;
    let mut seen = 0;

    loop {
        seen = follow(trace, seen, say);

        if child.try_wait().context("cannot read the agent")?.is_some() {
            follow(trace, seen, say);

            return Ok(false);
        }

        if Instant::now() >= deadline {
            child.kill().ok();
            child.wait().ok();

            return Ok(true);
        }

        thread::sleep(Duration::from_millis(400));
    }
}

fn follow(trace: &Path, seen: usize, say: &dyn Fn(Step)) -> usize {
    let Ok(text) = fs::read_to_string(trace) else {
        return seen;
    };

    let lines: Vec<&str> = text.lines().collect();

    if lines.len() <= seen {
        return seen;
    }

    for line in &lines[seen..] {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        if event.get("type").and_then(Value::as_str) != Some("assistant") {
            continue;
        }

        for call in calls(&event) {
            say(Step::Call(call.tool.clone(), gist(&call.input)));
        }
    }

    lines.len()
}

fn gist(input: &str) -> String {
    let value: Value = serde_json::from_str(input).unwrap_or(Value::Null);

    let text = ["command", "file_path", "pattern", "path", "description"]
        .iter()
        .find_map(|key| value.get(*key).and_then(Value::as_str))
        .unwrap_or(input);

    let one = text.lines().next().unwrap_or(text).trim();

    match one.char_indices().nth(84) {
        Some((at, _)) => format!("{}…", &one[..at]),
        None => one.to_owned(),
    }
}

fn read(trace: &Path) -> Result<Trace> {
    let Ok(text) = fs::read_to_string(trace) else {
        return Ok(Trace::default());
    };

    let mut found = Trace::default();

    for line in text.lines() {
        let Ok(event) = serde_json::from_str::<Value>(line) else {
            continue;
        };

        match event.get("type").and_then(Value::as_str) {
            Some("assistant") => found.calls.extend(calls(&event)),
            Some("result") => {
                if let Some(text) = event.get("result").and_then(Value::as_str) {
                    found.last_message = text.to_owned();
                }

                found.cost_usd = event
                    .get("total_cost_usd")
                    .and_then(Value::as_f64)
                    .unwrap_or_default();

                found.turns = event
                    .get("num_turns")
                    .and_then(Value::as_u64)
                    .unwrap_or_default() as usize;

                found.failed = event
                    .get("is_error")
                    .and_then(Value::as_bool)
                    .unwrap_or_default();
            }
            _ => {}
        }
    }

    Ok(found)
}

fn calls(event: &Value) -> Vec<Call> {
    let blocks = event
        .get("message")
        .and_then(|message| message.get("content"))
        .or_else(|| event.get("content"))
        .and_then(Value::as_array);

    let Some(blocks) = blocks else {
        return Vec::new();
    };

    blocks
        .iter()
        .filter(|block| block.get("type").and_then(Value::as_str) == Some("tool_use"))
        .filter_map(|block| {
            Some(Call {
                tool: block.get("name")?.as_str()?.to_owned(),
                input: block.get("input").map(Value::to_string).unwrap_or_default(),
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    struct World {
        home: tempfile::TempDir,
        settings: Settings,
    }

    impl World {
        fn new() -> Self {
            let home = tempfile::tempdir().unwrap();
            let at = home.path();

            fs::create_dir_all(at.join("bases/tiny")).unwrap();
            fs::write(
                at.join("bases/tiny/seed.sh"),
                "set -eu\nprintf '{}' > composer.json\nmkdir -p tests\nprintf '<?php' > tests/ExampleTest.php\n",
            )
            .unwrap();

            fs::create_dir_all(at.join("skills/a-skill")).unwrap();
            fs::write(
                at.join("skills/a-skill/SKILL.md"),
                "---\nname: a-skill\n---\n",
            )
            .unwrap();

            let settings = Settings {
                root: at.to_owned(),
                cache: at.join(".cache"),
                runs: at.join(".runs"),
                skills: at.join("skills"),
                model: "claude-sonnet-5".to_owned(),
                effort: "low".to_owned(),
                max_turns: 1,
                judge_model: "claude-haiku-4-5".to_owned(),
                diagnose_model: None,
                keep_runs: false,
            };

            Self { home, settings }
        }

        fn counter(&self) -> PathBuf {
            self.home.path().join("count")
        }

        fn case(&self, name: &str, mark: &str) -> Case {
            let dir = self.home.path().join("cases").join(name);

            fs::create_dir_all(&dir).unwrap();
            fs::write(
                dir.join("test.md"),
                "+++\nbase = \"tiny\"\n+++\n\nIt worked.\n",
            )
            .unwrap();
            fs::write(
                dir.join("setup.sh"),
                format!("set -eu\nprintf {mark} >> {}\n", self.counter().display()),
            )
            .unwrap();

            crate::case::read(&dir, "a-skill", name).unwrap()
        }

        fn project(&self) -> Project {
            Project::open("a-skill", "tiny", &self.settings).unwrap()
        }

        fn setups(&self) -> String {
            fs::read_to_string(self.counter()).unwrap_or_default()
        }
    }

    fn quiet(_step: Step) {}

    #[test]
    fn two_units_of_one_precondition_run_the_setup_one_time() {
        let world = World::new();
        let case = world.case("one", "x");
        let mut project = world.project();

        project.ready(&case, Arm::With, &quiet).unwrap();
        project.ready(&case, Arm::With, &quiet).unwrap();

        assert_eq!(world.setups(), "x");
    }

    #[test]
    fn two_units_of_two_preconditions_run_the_setup_two_times() {
        let world = World::new();
        let one = world.case("one", "x");
        let other = world.case("other", "y");
        let mut project = world.project();

        project.ready(&one, Arm::With, &quiet).unwrap();
        project.ready(&other, Arm::With, &quiet).unwrap();

        assert_eq!(world.setups(), "xy");
    }

    #[test]
    fn a_reset_gives_back_the_bytes_of_the_prepared_project() {
        let world = World::new();
        let case = world.case("one", "x");
        let mut project = world.project();

        project.ready(&case, Arm::With, &quiet).unwrap();

        fs::write(project.dir().join("composer.json"), "raised").unwrap();
        fs::remove_dir_all(project.dir().join("tests")).unwrap();
        fs::write(project.dir().join("NOTES.md"), "a note").unwrap();

        project.ready(&case, Arm::With, &quiet).unwrap();

        assert_eq!(
            fs::read_to_string(project.dir().join("composer.json")).unwrap(),
            "{}"
        );
        assert!(project.dir().join("tests/ExampleTest.php").is_file());
        assert!(!project.dir().join("NOTES.md").exists());
        assert_eq!(world.setups(), "x");
    }

    #[test]
    fn a_reset_gives_back_the_head_that_the_case_started_from() {
        let world = World::new();
        let case = world.case("one", "x");
        let mut project = world.project();

        project.ready(&case, Arm::With, &quiet).unwrap();

        let head = fs::read_to_string(project.dir().join(".eval/head")).unwrap();

        fs::write(project.dir().join("composer.json"), "raised").unwrap();
        seed(project.dir(), &["add", "-A"]).unwrap();
        seed(project.dir(), &["commit", "--quiet", "-m", "the agent"]).unwrap();

        project.ready(&case, Arm::With, &quiet).unwrap();

        let now = Command::new("git")
            .args(["rev-parse", "HEAD"])
            .current_dir(project.dir())
            .output()
            .unwrap();

        assert_eq!(head.trim(), String::from_utf8_lossy(&now.stdout).trim());
    }

    #[test]
    fn a_project_of_the_without_arm_carries_no_skill() {
        let world = World::new();
        let case = world.case("one", "x");
        let mut project = world.project();

        project.ready(&case, Arm::Without, &quiet).unwrap();

        assert!(!project.dir().join(".claude/skills/a-skill").exists());

        project.ready(&case, Arm::With, &quiet).unwrap();

        assert!(
            project
                .dir()
                .join(".claude/skills/a-skill/SKILL.md")
                .is_file()
        );
        assert_eq!(world.setups(), "xx");
    }

    #[test]
    fn a_trace_of_one_case_does_not_reach_the_case_after_it() {
        let world = World::new();
        let case = world.case("one", "x");
        let mut project = world.project();

        project.ready(&case, Arm::With, &quiet).unwrap();

        fs::write(project.dir().join(".eval/trace.jsonl"), "{}").unwrap();

        project.ready(&case, Arm::With, &quiet).unwrap();

        assert!(!project.dir().join(".eval/trace.jsonl").exists());
    }

    #[test]
    fn a_project_that_closes_leaves_no_directory_behind() {
        let world = World::new();
        let case = world.case("one", "x");
        let mut project = world.project();

        project.ready(&case, Arm::With, &quiet).unwrap();

        let dir = project.dir().to_owned();
        let slot = dir.with_file_name("a-skill-tiny.ready");

        assert!(dir.is_dir());
        assert!(slot.is_dir());

        project.close();

        assert!(!dir.exists());
        assert!(!slot.exists());
    }

    #[test]
    fn a_kept_run_holds_the_project_of_that_run() {
        let world = World::new();
        let case = world.case("one", "x");
        let mut project = world.project();

        project.ready(&case, Arm::With, &quiet).unwrap();
        fs::write(project.dir().join("composer.json"), "raised").unwrap();
        project.keep(&case, Arm::With, 1).unwrap();

        let kept = world.settings.runs.join("a-skill-one-with-1");

        assert_eq!(
            fs::read_to_string(kept.join("composer.json")).unwrap(),
            "raised"
        );
    }

    #[test]
    fn one_setup_and_one_arm_give_one_precondition() {
        let world = World::new();
        let one = world.case("one", "x");
        let other = world.case("other", "x");

        assert_eq!(
            precondition(&one, Arm::With).unwrap(),
            precondition(&other, Arm::With).unwrap()
        );
        assert_ne!(
            precondition(&one, Arm::With).unwrap(),
            precondition(&one, Arm::Without).unwrap()
        );
        assert_ne!(
            precondition(&one, Arm::With).unwrap(),
            precondition(&world.case("third", "y"), Arm::With).unwrap()
        );
    }

    #[test]
    fn a_fixture_that_differs_gives_a_precondition_that_differs() {
        let world = World::new();
        let one = world.case("one", "x");
        let other = world.case("other", "x");

        fs::create_dir_all(other.fixtures().join("tests")).unwrap();
        fs::write(other.fixtures().join("tests/BrokenTest.php"), "<?php").unwrap();

        assert_ne!(
            precondition(&one, Arm::With).unwrap(),
            precondition(&other, Arm::With).unwrap()
        );
    }

    fn trace(lines: &[&str]) -> Trace {
        let dir = tempfile::tempdir().unwrap();
        let file = dir.path().join("trace.jsonl");
        fs::write(&file, lines.join("\n")).unwrap();

        read(&file).unwrap()
    }

    #[test]
    fn a_trace_carries_each_call_of_a_tool_in_order() {
        let found = trace(&[
            r#"{"type":"system","subtype":"init"}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"text","text":"reading"},{"type":"tool_use","name":"Read","input":{"file_path":".hod/PROJECT.md"}}]}}"#,
            r#"{"type":"assistant","message":{"content":[{"type":"tool_use","name":"Bash","input":{"command":"composer outdated --direct"}}]}}"#,
            r#"{"type":"result","result":"Three lists follow.","total_cost_usd":0.42,"num_turns":7}"#,
        ]);

        assert_eq!(found.calls.len(), 2);
        assert_eq!(found.calls[0].tool, "Read");
        assert_eq!(found.calls[1].tool, "Bash");
        assert!(found.calls[1].input.contains("composer outdated --direct"));
        assert_eq!(found.last_message, "Three lists follow.");
        assert!((found.cost_usd - 0.42).abs() < f64::EPSILON);
        assert_eq!(found.turns, 7);
        assert!(!found.failed);
    }

    #[test]
    fn a_trace_reads_a_call_that_carries_no_message_wrapper() {
        let found = trace(&[
            r#"{"type":"assistant","content":[{"type":"tool_use","name":"Grep","input":{"pattern":"x"}}]}"#,
        ]);

        assert_eq!(found.calls.len(), 1);
        assert_eq!(found.calls[0].tool, "Grep");
    }

    #[test]
    fn a_line_that_is_not_json_does_not_stop_the_reader() {
        let found = trace(&[
            "not json at all",
            r#"{"type":"result","result":"done","is_error":true}"#,
        ]);

        assert_eq!(found.last_message, "done");
        assert!(found.failed);
    }

    #[test]
    fn a_trace_that_is_absent_gives_an_empty_trace() {
        let dir = tempfile::tempdir().unwrap();

        let found = read(&dir.path().join("nothing.jsonl")).unwrap();

        assert!(found.calls.is_empty());
        assert!(found.last_message.is_empty());
    }

    #[test]
    fn an_agent_that_does_not_stop_is_a_timeout() {
        let mut child = Command::new("sleep")
            .arg("30")
            .stdout(Stdio::null())
            .spawn()
            .unwrap();

        let quiet = |_step: Step| {};

        assert!(
            watch(
                &mut child,
                Duration::from_millis(300),
                &PathBuf::from("nothing.jsonl"),
                &quiet
            )
            .unwrap()
        );
    }

    #[test]
    fn the_tail_reports_each_new_call_once() {
        let dir = tempfile::tempdir().unwrap();
        let trace = dir.path().join("trace.jsonl");
        let seen = std::sync::Mutex::new(Vec::new());

        let say = |step: Step| {
            if let Step::Call(tool, what) = step {
                seen.lock().unwrap().push(format!("{tool} {what}"));
            }
        };

        fs::write(
            &trace,
            "{\"type\":\"assistant\",\"message\":{\"content\":[{\"type\":\"tool_use\",\"name\":\"Bash\",\"input\":{\"command\":\"git status --short\"}}]}}\n",
        )
        .unwrap();

        let mark = follow(&trace, 0, &say);

        assert_eq!(mark, 1);
        assert_eq!(seen.lock().unwrap().len(), 1);
        assert!(seen.lock().unwrap()[0].contains("git status --short"));

        follow(&trace, mark, &say);

        assert_eq!(seen.lock().unwrap().len(), 1);
    }

    #[test]
    fn a_long_command_reads_short_in_the_progress() {
        let long = format!("{{\"command\":\"{}\"}}", "a".repeat(200));

        assert!(gist(&long).ends_with('…'));
        assert!(gist(&long).chars().count() <= 85);
    }

    #[test]
    fn the_baseline_leaves_no_file_of_the_harness_in_the_working_tree() {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path();

        fs::create_dir_all(at.join(".claude/skills/deps-upgrade")).unwrap();
        fs::create_dir_all(at.join(".eval")).unwrap();
        fs::write(
            at.join(".claude/skills/deps-upgrade/SKILL.md"),
            "---\n---\n",
        )
        .unwrap();
        fs::write(at.join(".eval/trace.jsonl"), "{}").unwrap();
        fs::write(at.join("composer.json"), "{}").unwrap();

        baseline(at).unwrap();

        let output = Command::new("git")
            .args(["status", "--porcelain"])
            .current_dir(at)
            .output()
            .unwrap();

        assert_eq!(String::from_utf8_lossy(&output.stdout), "");
    }

    #[test]
    fn the_baseline_commits_the_project_and_leaves_a_head_to_read() {
        let dir = tempfile::tempdir().unwrap();
        fs::create_dir_all(dir.path().join(".eval")).unwrap();
        fs::write(dir.path().join("composer.json"), "{}").unwrap();

        baseline(dir.path()).unwrap();
        remember(dir.path()).unwrap();

        let head = fs::read_to_string(dir.path().join(".eval/head")).unwrap();

        assert_eq!(head.trim().len(), 40);
    }

    #[test]
    fn an_agent_that_stops_is_not_a_timeout() {
        let mut child = Command::new("true").spawn().unwrap();

        let quiet = |_step: Step| {};

        assert!(
            !watch(
                &mut child,
                Duration::from_secs(30),
                &PathBuf::from("nothing.jsonl"),
                &quiet
            )
            .unwrap()
        );
    }
}
