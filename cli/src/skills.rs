use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};

use crate::front::Front;
use crate::project;

#[derive(Debug)]
struct Built {
    name: &'static str,
    files: &'static [(&'static str, &'static str)],
}

include!(concat!(env!("OUT_DIR"), "/skills.rs"));

#[derive(Debug, Clone)]
pub struct Skill {
    pub name: String,
    pub files: Vec<(String, String)>,
}

impl Skill {
    pub fn front(&self) -> Front {
        let text = self
            .files
            .iter()
            .find(|(file, _)| file == "SKILL.md")
            .map_or("", |(_, text)| text.as_str());

        Front::read(text, &self.name)
    }
}

pub fn shipped() -> Vec<Skill> {
    SKILLS
        .iter()
        .map(|built| Skill {
            name: built.name.to_owned(),
            files: built
                .files
                .iter()
                .map(|(file, text)| ((*file).to_owned(), (*text).to_owned()))
                .collect(),
        })
        .collect()
}

pub fn local(dir: &Path) -> Result<Vec<Skill>> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(Vec::new());
    };

    let mut skills = Vec::new();

    for entry in entries {
        let path = entry
            .with_context(|| format!("cannot read `{}`", dir.display()))?
            .path();

        if !path.join("SKILL.md").is_file() {
            continue;
        }

        skills.push(Skill {
            name: project::name(&path),
            files: walk(&path, &path)?,
        });
    }

    skills.sort_by(|one, other| one.name.cmp(&other.name));

    Ok(skills)
}

fn walk(skill: &Path, dir: &Path) -> Result<Vec<(String, String)>> {
    let entries = fs::read_dir(dir).with_context(|| format!("cannot read `{}`", dir.display()))?;

    let mut files = Vec::new();

    for entry in entries {
        let path = entry
            .with_context(|| format!("cannot read `{}`", dir.display()))?
            .path();

        if project::name(&path).starts_with('.') {
            continue;
        }

        if path.is_dir() {
            files.extend(walk(skill, &path)?);
            continue;
        }

        let text = fs::read_to_string(&path)
            .with_context(|| format!("cannot read `{}`", path.display()))?;

        files.push((relative(skill, &path), text));
    }

    files.sort_by(|one, other| one.0.cmp(&other.0));

    Ok(files)
}

fn relative(skill: &Path, file: &Path) -> String {
    file.strip_prefix(skill)
        .unwrap_or(file)
        .to_string_lossy()
        .replace('\\', "/")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::front::unquote;

    fn metadata(skill: &Skill) -> &str {
        skill
            .files
            .iter()
            .find(|(file, _)| file == "agents/openai.yaml")
            .map_or_else(
                || panic!("the skill `{}` holds no `agents/openai.yaml`", skill.name),
                |(_, text)| text.as_str(),
            )
    }

    fn value<'a>(text: &'a str, key: &str) -> Option<&'a str> {
        text.lines()
            .filter_map(|line| line.trim().split_once(':'))
            .find(|(name, _)| *name == key)
            .map(|(_, value)| unquote(value.trim()))
    }

    #[test]
    fn each_skill_carries_a_name_that_agrees_with_its_directory() {
        for skill in shipped() {
            assert_eq!(
                skill.front().name,
                skill.name,
                "the front matter of `{}` names a different skill",
                skill.name
            );
        }
    }

    #[test]
    fn each_skill_carries_a_description() {
        for skill in shipped() {
            assert!(
                !skill.front().description.is_empty(),
                "the skill `{}` carries no description, thus no client names it",
                skill.name
            );
        }
    }

    #[test]
    fn each_skill_carries_the_metadata_of_codex() {
        for skill in shipped() {
            let metadata = metadata(&skill);

            for key in ["display_name", "short_description"] {
                let value = value(metadata, key);

                assert!(
                    value.is_some_and(|value| !value.is_empty()),
                    "the skill `{}` gives `{key}` no value in `agents/openai.yaml`",
                    skill.name
                );
            }
        }
    }

    #[test]
    fn a_user_skill_carries_the_two_properties_of_a_user_skill() {
        for skill in shipped() {
            let implicit = value(metadata(&skill), "allow_implicit_invocation");

            assert_eq!(
                skill.front().user,
                implicit == Some("false"),
                "the skill `{}` is a user skill in one client and a model skill in the other",
                skill.name
            );
        }
    }

    #[test]
    fn no_skill_takes_the_name_of_a_command() {
        let command = crate::command();
        let names: Vec<&str> = command
            .get_subcommands()
            .map(clap::Command::get_name)
            .collect();

        for skill in shipped() {
            assert!(
                !names.contains(&skill.name.as_str()),
                "the skill `{}` takes the name of a command, thus `hod {}` cannot reach it",
                skill.name,
                skill.name
            );
        }
    }

    #[test]
    fn a_directory_without_a_skill_gives_no_skill() {
        let dir = tempfile::tempdir().unwrap();

        assert!(local(dir.path()).unwrap().is_empty());
        assert!(local(&dir.path().join("nothing")).unwrap().is_empty());
    }

    #[test]
    fn a_local_skill_carries_each_file_and_its_front_matter() {
        let dir = tempfile::tempdir().unwrap();
        let skill = dir.path().join("deploy");
        fs::create_dir_all(skill.join("references")).unwrap();
        fs::write(
            skill.join("SKILL.md"),
            "---\nname: deploy\ndescription: Deploy this project\n---\n",
        )
        .unwrap();
        fs::write(skill.join("references/hosts.md"), "one").unwrap();

        let skills = local(dir.path()).unwrap();

        assert_eq!(skills.len(), 1);
        assert_eq!(skills[0].name, "deploy");
        assert_eq!(skills[0].front().description, "Deploy this project");
        assert_eq!(
            skills[0].files,
            vec![
                ("SKILL.md".to_owned(), skills[0].files[0].1.clone()),
                ("references/hosts.md".to_owned(), "one".to_owned()),
            ]
        );
    }
}
