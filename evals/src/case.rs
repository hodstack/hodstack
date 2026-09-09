use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result, bail};
use serde::Deserialize;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    #[default]
    LastMessage,
    Trace,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Match {
    #[default]
    Contains,
    NotContains,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Call {
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub input_match: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Grader {
    ToolUsed {
        #[serde(default)]
        tool: Option<String>,
        #[serde(default)]
        input_match: Option<String>,
        #[serde(default = "once")]
        min: usize,
        #[serde(default)]
        max: Option<usize>,
    },
    ToolOrder {
        before: Call,
        after: Call,
    },
    Regex {
        pattern: String,
        #[serde(default)]
        target: Target,
        #[serde(default, rename = "match")]
        verdict: Match,
    },
    FileContent {
        path: String,
        pattern: String,
        #[serde(default, rename = "match")]
        verdict: Match,
    },
    FileExists {
        path: String,
        #[serde(default = "yes")]
        exists: bool,
    },
    GitClean {
        paths: Vec<String>,
    },
    GitDirty {},
    HeadUnmoved {},
}

impl Grader {
    pub fn label(&self) -> String {
        match self {
            Self::ToolUsed {
                tool, input_match, ..
            } => match (tool, input_match) {
                (Some(tool), Some(pattern)) => format!("{tool} ran `{pattern}`"),
                (None, Some(pattern)) => format!("a tool ran `{pattern}`"),
                (Some(tool), None) => format!("{tool} ran"),
                (None, None) => "a tool ran".to_owned(),
            },
            Self::ToolOrder { before, after } => {
                format!("`{}` ran before `{}`", describe(before), describe(after))
            }
            Self::Regex {
                pattern, verdict, ..
            } => format!("the text {} `{pattern}`", says(*verdict)),
            Self::FileContent {
                path,
                pattern,
                verdict,
            } => format!("`{path}` {} `{pattern}`", says(*verdict)),
            Self::FileExists { path, exists } => match exists {
                true => format!("`{path}` exists"),
                false => format!("`{path}` does not exist"),
            },
            Self::GitClean { paths } => format!("git left `{}` alone", paths.join("`, `")),
            Self::GitDirty {} => "the working tree carries a change".to_owned(),
            Self::HeadUnmoved {} => "no commit was written".to_owned(),
        }
    }
}

fn describe(call: &Call) -> String {
    match (&call.input_match, &call.tool) {
        (Some(pattern), _) => pattern.clone(),
        (None, Some(tool)) => tool.clone(),
        (None, None) => "a tool".to_owned(),
    }
}

fn says(verdict: Match) -> &'static str {
    match verdict {
        Match::Contains => "holds",
        Match::NotContains => "does not hold",
    }
}

fn once() -> usize {
    1
}

fn yes() -> bool {
    true
}

fn all() -> f64 {
    1.0
}

fn quarter_hour() -> u64 {
    900
}

fn tools() -> Vec<String> {
    ["Bash", "Read", "Write", "Edit", "Grep", "Glob"]
        .iter()
        .map(|&tool| tool.to_owned())
        .collect()
}

#[derive(Debug, Deserialize)]
struct Header {
    base: String,
    #[serde(default = "all")]
    threshold: f64,
    #[serde(default = "quarter_hour")]
    timeout_seconds: u64,
    #[serde(default = "tools")]
    allowed_tools: Vec<String>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    intent: Option<String>,
    #[serde(default)]
    graders: Vec<Grader>,
}

#[derive(Debug, Clone)]
pub struct Case {
    pub skill: String,
    pub name: String,
    pub dir: PathBuf,
    pub base: String,
    pub threshold: f64,
    pub timeout_seconds: u64,
    pub allowed_tools: Vec<String>,
    pub prompt: Option<String>,
    pub intent: Option<String>,
    pub graders: Vec<Grader>,
    pub expectation: String,
}

impl Case {
    pub fn id(&self) -> String {
        format!("{}/{}", self.skill, self.name)
    }

    pub fn opening(&self) -> String {
        self.prompt
            .clone()
            .unwrap_or_else(|| format!("/{}", self.skill))
    }

    pub fn baseline(&self) -> Option<&str> {
        self.intent.as_deref().or(self.prompt.as_deref())
    }

    pub fn setup(&self) -> PathBuf {
        self.dir.join("setup.sh")
    }

    pub fn fixtures(&self) -> PathBuf {
        self.dir.join("fixtures")
    }
}

pub fn read(dir: &Path, skill: &str, name: &str) -> Result<Case> {
    let file = dir.join("test.md");

    let text =
        fs::read_to_string(&file).with_context(|| format!("cannot read `{}`", file.display()))?;

    let (header, expectation) = split(&text)
        .with_context(|| format!("`{}` carries no `+++` front matter", file.display()))?;

    let header: Header = toml::from_str(header)
        .with_context(|| format!("cannot read the front matter of `{}`", file.display()))?;

    for grader in &header.graders {
        if let Grader::ToolUsed { min, max, .. } = grader {
            if max.is_some_and(|ceiling| ceiling < *min) {
                bail!(
                    "`{}` holds a grader that no run can pass: {} needs {min} call(s) at the least and {} at the most; write `min = 0` to say that the call must not happen",
                    file.display(),
                    grader.label(),
                    max.unwrap_or_default()
                )
            }
        }
    }

    if header.graders.is_empty() && expectation.trim().is_empty() {
        bail!(
            "`{}` carries no grader and no expectation, thus it tests nothing",
            file.display()
        )
    }

    Ok(Case {
        skill: skill.to_owned(),
        name: name.to_owned(),
        dir: dir.to_owned(),
        base: header.base,
        threshold: header.threshold,
        timeout_seconds: header.timeout_seconds,
        allowed_tools: header.allowed_tools,
        prompt: header.prompt,
        intent: header.intent,
        graders: header.graders,
        expectation: expectation.trim().to_owned(),
    })
}

fn split(text: &str) -> Option<(&str, &str)> {
    let rest = text
        .strip_prefix("+++\n")
        .or_else(|| text.strip_prefix("+++\r\n"))?;

    let (header, body) = rest.split_once("\n+++")?;

    Some((header, body))
}

pub fn discover(root: &Path, selector: Option<&str>) -> Result<Vec<Case>> {
    let (wanted_skill, wanted_case) = match selector {
        Some(selector) => match selector.split_once('/') {
            Some((skill, name)) => (Some(skill), Some(name)),
            None => (Some(selector), None),
        },
        None => (None, None),
    };

    let mut cases = Vec::new();

    for skill in directories(root)? {
        let name = leaf(&skill);

        if matches!(wanted_skill, Some(wanted) if wanted != name) {
            continue;
        }

        for case in directories(&skill)? {
            let case_name = leaf(&case);

            if matches!(wanted_case, Some(wanted) if wanted != case_name) {
                continue;
            }

            if !case.join("test.md").is_file() {
                continue;
            }

            cases.push(read(&case, &name, &case_name)?);
        }
    }

    if cases.is_empty() {
        match selector {
            Some(selector) => bail!("no eval case answers to `{selector}`"),
            None => bail!("`{}` holds no eval case", root.display()),
        }
    }

    cases.sort_by_key(Case::id);

    Ok(cases)
}

fn directories(dir: &Path) -> Result<Vec<PathBuf>> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(Vec::new());
    };

    let mut found = Vec::new();

    for entry in entries {
        let path = entry
            .with_context(|| format!("cannot read `{}`", dir.display()))?
            .path();

        if path.is_dir() && !leaf(&path).starts_with('.') {
            found.push(path);
        }
    }

    found.sort();

    Ok(found)
}

fn leaf(path: &Path) -> String {
    path.file_name()
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    const CASE: &str = r#"+++
base = "laravel"
threshold = 0.5

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "composer outdated --direct"

[[graders]]
type = "tool_used"
tool = "Bash"
input_match = "git commit"
min = 0
max = 0

[[graders]]
type = "git_clean"
paths = ["composer.json"]
+++

The agent asked the user nothing.
"#;

    fn parse(text: &str) -> Case {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.md"), text).unwrap();

        read(dir.path(), "deps-upgrade", "a-case").unwrap()
    }

    #[test]
    fn a_case_carries_its_graders_and_its_expectation() {
        let case = parse(CASE);

        assert_eq!(case.id(), "deps-upgrade/a-case");
        assert_eq!(case.opening(), "/deps-upgrade");
        assert_eq!(case.base, "laravel");
        assert_eq!(case.graders.len(), 3);
        assert_eq!(case.expectation, "The agent asked the user nothing.");
    }

    #[test]
    fn a_case_with_a_prompt_gives_that_prompt_and_no_name_of_a_skill() {
        let case = parse(
            "+++\nbase = \"laravel\"\nprompt = \"Write a function that reads a file.\"\n+++\n\nIt worked.\n",
        );

        assert_eq!(case.opening(), "Write a function that reads a file.");
        assert_eq!(case.baseline(), Some("Write a function that reads a file."));
    }

    #[test]
    fn a_case_without_a_field_takes_the_default_of_that_field() {
        let case = parse("+++\nbase = \"laravel\"\n+++\n\nIt worked.\n");

        assert_eq!(case.timeout_seconds, 900);
        assert!(case.allowed_tools.contains(&"Bash".to_owned()));
    }

    #[test]
    fn a_grader_that_no_run_can_pass_is_a_fault() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("test.md"),
            "+++\nbase = \"laravel\"\n\n[[graders]]\ntype = \"tool_used\"\ntool = \"Bash\"\ninput_match = \"git commit\"\nmax = 0\n+++\n\nIt worked.\n",
        )
        .unwrap();

        let fault = read(dir.path(), "deps-upgrade", "impossible").unwrap_err();

        assert!(format!("{fault:#}").contains("no run can pass"));
    }

    #[test]
    fn a_grader_that_denies_a_call_passes_with_a_floor_of_zero() {
        let case = parse(
            "+++\nbase = \"laravel\"\n\n[[graders]]\ntype = \"tool_used\"\ntool = \"Bash\"\ninput_match = \"git commit\"\nmin = 0\nmax = 0\n+++\n\nIt worked.\n",
        );

        assert_eq!(case.graders.len(), 1);
    }

    #[test]
    fn a_case_that_tests_nothing_is_a_fault() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.md"), "+++\nbase = \"laravel\"\n+++\n").unwrap();

        assert!(read(dir.path(), "deps-upgrade", "empty").is_err());
    }

    #[test]
    fn a_file_without_front_matter_is_a_fault() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join("test.md"), "The agent worked.\n").unwrap();

        assert!(read(dir.path(), "deps-upgrade", "bare").is_err());
    }

    #[test]
    fn each_grader_gives_a_label_for_the_report() {
        for grader in parse(CASE).graders {
            assert!(!grader.label().is_empty());
        }
    }
}
