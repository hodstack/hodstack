use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context as _, Result};
use sha2::{Digest as _, Sha256};

use crate::front::Front;
use crate::skills::{self, Skill};

pub const AGENTS: &str = "AGENTS.md";
pub const CLAUDE: &str = "CLAUDE.md";
pub const HOD: &str = ".hod";
pub const INTENTION: &str = ".hod/project.md";
pub const CLIENTS: [&str; 2] = [".claude/skills", ".agents/skills"];

pub const RULES: &str = include_str!("../templates/rules.md");
pub const SEED: &str = include_str!("../templates/project.md");
pub const IMPORT: &str = include_str!("../templates/CLAUDE.md");

#[derive(Debug)]
pub struct Rule {
    pub file: String,
    pub front: Front,
}

#[derive(Debug)]
pub struct Project {
    root: PathBuf,
}

impl Project {
    pub fn new(root: &Path) -> Self {
        Self {
            root: root.to_path_buf(),
        }
    }

    pub fn exists(&self) -> bool {
        self.root.join(HOD).is_dir()
    }

    pub fn path(&self, relative: &str) -> PathBuf {
        self.root.join(relative)
    }

    pub fn lock(&self) -> PathBuf {
        self.path(".hod/lock")
    }

    pub fn rules(&self) -> Result<Vec<Rule>> {
        let dir = self.path(".hod/rules");
        let Ok(entries) = fs::read_dir(&dir) else {
            return Ok(Vec::new());
        };

        let mut rules = Vec::new();

        for entry in entries {
            let path = entry
                .with_context(|| format!("cannot read `{}`", dir.display()))?
                .path();

            if path.extension().and_then(|extension| extension.to_str()) != Some("md") {
                continue;
            }

            let file = name(&path);
            let stem = file.trim_end_matches(".md").to_owned();
            let text = fs::read_to_string(&path)
                .with_context(|| format!("cannot read `{}`", path.display()))?;

            rules.push(Rule {
                file,
                front: Front::read(&text, &stem),
            });
        }

        rules.sort_by(|one, other| one.file.cmp(&other.file));

        Ok(rules)
    }

    pub fn skills(&self) -> Result<Vec<Skill>> {
        skills::local(&self.path(".hod/skills"))
    }

    pub fn installed(&self) -> Result<Vec<Skill>> {
        let mut skills = skills::shipped();

        for mine in self.skills()? {
            match skills.iter().position(|skill| skill.name == mine.name) {
                Some(at) => skills[at] = mine,
                None => skills.push(mine),
            }
        }

        Ok(skills)
    }

    pub fn skill(&self, name: &str) -> Result<Option<Skill>> {
        let name = name.replace(':', "-");

        Ok(self
            .installed()?
            .into_iter()
            .find(|skill| skill.name == name))
    }
}

pub fn agents(rules: &[Rule]) -> String {
    let mut text = RULES.to_owned();

    if rules.is_empty() {
        return text;
    }

    text.push_str("\n---\n\n## 5. The rules of this project\n\n");
    text.push_str("Read the file of a rule when its subject reaches your task.\n\n");

    for rule in rules {
        text.push_str(&row(rule));
    }

    text
}

fn row(rule: &Rule) -> String {
    let path = format!(".hod/rules/{}", rule.file);

    if rule.front.description.is_empty() {
        return format!("- [{}]({path})\n", rule.front.name);
    }

    format!(
        "- [{}]({path}): {}\n",
        rule.front.name, rule.front.description
    )
}

pub fn sum(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub fn name(path: &Path) -> String {
    path.file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rule(file: &str, name: &str, description: &str) -> Rule {
        Rule {
            file: file.to_owned(),
            front: Front {
                name: name.to_owned(),
                description: description.to_owned(),
                user: false,
            },
        }
    }

    #[test]
    fn the_agents_file_without_a_rule_is_the_template() {
        assert_eq!(agents(&[]), RULES);
    }

    #[test]
    fn each_rule_gives_one_row_with_its_path() {
        let text = agents(&[
            rule(
                "pest-not-phpunit.md",
                "pest-not-phpunit",
                "Write a test with Pest",
            ),
            rule("no-description.md", "no-description", ""),
        ]);

        assert!(text.starts_with(RULES));
        assert!(text.contains("## 5. The rules of this project"));
        assert!(text.contains(
            "- [pest-not-phpunit](.hod/rules/pest-not-phpunit.md): Write a test with Pest\n"
        ));
        assert!(text.contains("- [no-description](.hod/rules/no-description.md)\n"));
    }

    #[test]
    fn a_project_reads_each_rule_in_order() {
        let dir = tempfile::tempdir().unwrap();
        let rules = dir.path().join(".hod/rules");
        fs::create_dir_all(&rules).unwrap();
        fs::write(rules.join("two.md"), "---\nname: two\n---\n").unwrap();
        fs::write(rules.join("one.md"), "---\nname: one\n---\n").unwrap();
        fs::write(rules.join("notes.txt"), "not a rule").unwrap();

        let read = Project::new(dir.path()).rules().unwrap();

        assert_eq!(read.len(), 2);
        assert_eq!(read[0].front.name, "one");
        assert_eq!(read[1].front.name, "two");
    }

    #[test]
    fn a_colon_in_the_name_of_a_skill_is_a_hyphen_on_the_disk() {
        let dir = tempfile::tempdir().unwrap();
        let skill = dir.path().join(".hod/skills/pr-review");
        fs::create_dir_all(&skill).unwrap();
        fs::write(skill.join("SKILL.md"), "---\nname: pr-review\n---\n").unwrap();
        let project = Project::new(dir.path());

        assert_eq!(
            project.skill("pr:review").unwrap().map(|skill| skill.name),
            Some("pr-review".to_owned())
        );
        assert!(project.skill("pr:merge").unwrap().is_none());
    }

    #[test]
    fn a_skill_of_the_project_takes_the_name_of_a_skill_of_the_program() {
        let dir = tempfile::tempdir().unwrap();
        let skill = dir.path().join(".hod/skills/learn");
        fs::create_dir_all(&skill).unwrap();
        fs::write(
            skill.join("SKILL.md"),
            "---\nname: learn\ndescription: Mine.\n---\n",
        )
        .unwrap();

        let found = Project::new(dir.path()).skill("learn").unwrap().unwrap();

        assert_eq!(found.front().description, "Mine.");
    }

    #[test]
    fn a_project_without_a_rules_directory_has_no_rule() {
        let dir = tempfile::tempdir().unwrap();

        assert!(Project::new(dir.path()).rules().unwrap().is_empty());
        assert!(!Project::new(dir.path()).exists());
    }
}
