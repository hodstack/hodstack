use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use anyhow::{Context as _, Result};

use crate::project;

#[derive(Debug, Default)]
pub struct Lock {
    files: BTreeMap<String, String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Owner {
    Absent,
    Ours,
    Theirs,
}

impl Lock {
    pub fn read(path: &Path) -> Self {
        let Ok(text) = fs::read_to_string(path) else {
            return Self::default();
        };

        let mut files = BTreeMap::new();

        for line in text.lines() {
            if let Some((sum, file)) = line.split_once("  ") {
                files.insert(file.to_owned(), sum.to_owned());
            }
        }

        Self { files }
    }

    pub fn write(&self, path: &Path) -> Result<()> {
        let mut text = String::new();

        for (file, sum) in &self.files {
            let _ = writeln!(text, "{sum}  {file}");
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("cannot write in `{}`", parent.display()))?;
        }

        fs::write(path, text).with_context(|| format!("cannot write `{}`", path.display()))
    }

    pub fn state(&self, file: &str, found: Option<&[u8]>) -> Owner {
        let Some(bytes) = found else {
            return Owner::Absent;
        };

        match self.files.get(file) {
            Some(sum) if *sum == project::sum(bytes) => Owner::Ours,
            _ => Owner::Theirs,
        }
    }

    pub fn keep(&mut self, file: &str, sum: String) {
        self.files.insert(file.to_owned(), sum);
    }

    pub fn holds(&self, file: &str) -> bool {
        self.files.contains_key(file)
    }

    pub fn files(&self) -> impl Iterator<Item = &str> {
        self.files.keys().map(String::as_str)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_lock_that_does_not_exist_holds_no_file() {
        let lock = Lock::read(Path::new("/hod/no/such/lock"));

        assert_eq!(lock.files().count(), 0);
        assert_eq!(lock.state("AGENTS.md", Some(b"one")), Owner::Theirs);
        assert_eq!(lock.state("AGENTS.md", None), Owner::Absent);
    }

    #[test]
    fn a_file_with_the_sum_of_the_lock_belongs_to_this_program() {
        let mut lock = Lock::default();
        lock.keep("AGENTS.md", project::sum(b"one"));

        assert_eq!(lock.state("AGENTS.md", Some(b"one")), Owner::Ours);
        assert_eq!(lock.state("AGENTS.md", Some(b"two")), Owner::Theirs);
        assert!(lock.holds("AGENTS.md"));
    }

    #[test]
    fn a_lock_reads_the_file_that_it_wrote() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(".hod/lock");
        let mut lock = Lock::default();
        lock.keep("AGENTS.md", project::sum(b"one"));
        lock.keep(".claude/skills/init/SKILL.md", project::sum(b"two"));

        lock.write(&path).unwrap();
        let read = Lock::read(&path);

        assert_eq!(
            read.files().collect::<Vec<_>>(),
            vec![".claude/skills/init/SKILL.md", "AGENTS.md"]
        );
        assert_eq!(read.state("AGENTS.md", Some(b"one")), Owner::Ours);
    }
}
