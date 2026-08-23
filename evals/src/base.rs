use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context as _, Result, bail};

const SEEDED: &str = ".seeded";

pub fn materialize(bases: &Path, name: &str, cache: &Path) -> Result<PathBuf> {
    let dir = cache.join(name);

    if dir.join(SEEDED).is_file() {
        return Ok(dir);
    }

    let seed = bases.join(name).join("seed.sh");

    if !seed.is_file() {
        bail!(
            "the base `{name}` needs `{}`, and that file is absent",
            seed.display()
        )
    }

    if dir.exists() {
        fs::remove_dir_all(&dir).with_context(|| format!("cannot clear `{}`", dir.display()))?;
    }

    fs::create_dir_all(&dir).with_context(|| format!("cannot write `{}`", dir.display()))?;

    let status = Command::new("sh")
        .arg(&seed)
        .current_dir(&dir)
        .status()
        .with_context(|| format!("cannot run `{}`", seed.display()))?;

    if !status.success() {
        bail!("`{}` failed", seed.display())
    }

    fs::write(dir.join(SEEDED), name)
        .with_context(|| format!("cannot write `{}`", dir.join(SEEDED).display()))?;

    Ok(dir)
}

pub fn clone(from: &Path, to: &Path) -> Result<()> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("cannot write `{}`", parent.display()))?;
    }

    if to.exists() {
        fs::remove_dir_all(to).with_context(|| format!("cannot clear `{}`", to.display()))?;
    }

    for arguments in attempts() {
        let status = Command::new("cp")
            .args(arguments)
            .arg(from)
            .arg(to)
            .status()
            .context("cannot run `cp`")?;

        if status.success() {
            fs::remove_file(to.join(SEEDED)).ok();

            return Ok(());
        }

        if to.exists() {
            fs::remove_dir_all(to).with_context(|| format!("cannot clear `{}`", to.display()))?;
        }
    }

    bail!("cannot copy `{}` to `{}`", from.display(), to.display())
}

fn attempts() -> Vec<Vec<&'static str>> {
    match std::env::consts::OS {
        "macos" => vec![vec!["-Rc"], vec!["-R"]],
        _ => vec![vec!["-a", "--reflink=auto"], vec!["-a"]],
    }
}

pub fn overlay(from: &Path, to: &Path) -> Result<()> {
    if !from.is_dir() {
        return Ok(());
    }

    let entries =
        fs::read_dir(from).with_context(|| format!("cannot read `{}`", from.display()))?;

    for entry in entries {
        let path = entry
            .with_context(|| format!("cannot read `{}`", from.display()))?
            .path();

        let Some(name) = path.file_name() else {
            continue;
        };

        let target = to.join(name);

        if path.is_dir() {
            fs::create_dir_all(&target)
                .with_context(|| format!("cannot write `{}`", target.display()))?;

            overlay(&path, &target)?;

            continue;
        }

        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)
                .with_context(|| format!("cannot write `{}`", parent.display()))?;
        }

        fs::copy(&path, &target).with_context(|| format!("cannot write `{}`", target.display()))?;
    }

    Ok(())
}

pub fn contents(dir: &Path) -> Result<Vec<(String, Vec<u8>)>> {
    let mut found = Vec::new();

    gather(dir, dir, &mut found)?;

    found.sort_by(|one, other| one.0.cmp(&other.0));

    Ok(found)
}

fn gather(root: &Path, dir: &Path, into: &mut Vec<(String, Vec<u8>)>) -> Result<()> {
    let Ok(entries) = fs::read_dir(dir) else {
        return Ok(());
    };

    for entry in entries {
        let path = entry
            .with_context(|| format!("cannot read `{}`", dir.display()))?
            .path();

        if path.is_dir() {
            gather(root, &path, into)?;

            continue;
        }

        let name = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");

        let bytes = fs::read(&path).with_context(|| format!("cannot read `{}`", path.display()))?;

        into.push((name, bytes));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_clone_carries_each_file_of_the_base() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("from");
        fs::create_dir_all(from.join("app")).unwrap();
        fs::write(from.join("composer.json"), "{}").unwrap();
        fs::write(from.join("app/User.php"), "<?php").unwrap();
        fs::write(from.join(SEEDED), "laravel").unwrap();

        let to = dir.path().join("runs/one");
        clone(&from, &to).unwrap();

        assert_eq!(fs::read_to_string(to.join("composer.json")).unwrap(), "{}");
        assert_eq!(
            fs::read_to_string(to.join("app/User.php")).unwrap(),
            "<?php"
        );
    }

    #[test]
    fn a_clone_does_not_carry_the_mark_of_the_base() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("from");
        fs::create_dir_all(&from).unwrap();
        fs::write(from.join(SEEDED), "laravel").unwrap();

        let to = dir.path().join("runs/one");
        clone(&from, &to).unwrap();

        assert!(!to.join(SEEDED).exists());
    }

    #[test]
    fn an_overlay_writes_over_the_file_of_the_base() {
        let dir = tempfile::tempdir().unwrap();
        let base = dir.path().join("base");
        fs::create_dir_all(base.join("tests")).unwrap();
        fs::write(base.join("composer.json"), "old").unwrap();
        fs::write(base.join("tests/Kept.php"), "kept").unwrap();

        let fixtures = dir.path().join("fixtures");
        fs::create_dir_all(fixtures.join("tests")).unwrap();
        fs::write(fixtures.join("composer.json"), "new").unwrap();
        fs::write(fixtures.join("tests/Added.php"), "added").unwrap();

        overlay(&fixtures, &base).unwrap();

        assert_eq!(
            fs::read_to_string(base.join("composer.json")).unwrap(),
            "new"
        );
        assert_eq!(
            fs::read_to_string(base.join("tests/Kept.php")).unwrap(),
            "kept"
        );
        assert_eq!(
            fs::read_to_string(base.join("tests/Added.php")).unwrap(),
            "added"
        );
    }

    #[test]
    fn an_overlay_that_is_absent_writes_nothing() {
        let dir = tempfile::tempdir().unwrap();

        overlay(&dir.path().join("nothing"), dir.path()).unwrap();
    }

    #[test]
    fn a_clone_over_a_directory_that_holds_a_file_drops_that_file() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("from");
        fs::create_dir_all(&from).unwrap();
        fs::write(from.join("composer.json"), "pinned").unwrap();

        let to = dir.path().join("to");
        fs::create_dir_all(to.join("vendor")).unwrap();
        fs::write(to.join("composer.json"), "raised").unwrap();
        fs::write(to.join("vendor/installed.php"), "raised").unwrap();
        fs::write(to.join("NOTES.md"), "left over").unwrap();

        clone(&from, &to).unwrap();

        assert_eq!(
            fs::read_to_string(to.join("composer.json")).unwrap(),
            "pinned"
        );
        assert!(!to.join("NOTES.md").exists());
        assert!(!to.join("vendor").exists());
        assert!(!to.join("from").exists());
    }

    #[test]
    fn a_clone_brings_back_a_file_that_the_run_deleted() {
        let dir = tempfile::tempdir().unwrap();
        let from = dir.path().join("from");
        fs::create_dir_all(from.join("tests")).unwrap();
        fs::write(from.join("tests/ExampleTest.php"), "<?php").unwrap();

        let to = dir.path().join("to");
        clone(&from, &to).unwrap();
        fs::remove_dir_all(to.join("tests")).unwrap();
        clone(&from, &to).unwrap();

        assert_eq!(
            fs::read_to_string(to.join("tests/ExampleTest.php")).unwrap(),
            "<?php"
        );
    }

    #[test]
    fn the_contents_of_a_tree_read_in_one_order_whatever_the_file_system_gives() {
        let dir = tempfile::tempdir().unwrap();
        let at = dir.path();
        fs::create_dir_all(at.join("tests/Feature")).unwrap();
        fs::write(at.join("composer.json"), "one").unwrap();
        fs::write(at.join("tests/Feature/BrokenTest.php"), "two").unwrap();

        let found = contents(at).unwrap();

        assert_eq!(
            found,
            vec![
                ("composer.json".to_owned(), b"one".to_vec()),
                ("tests/Feature/BrokenTest.php".to_owned(), b"two".to_vec()),
            ]
        );
    }

    #[test]
    fn the_contents_of_a_tree_that_is_absent_are_empty() {
        let dir = tempfile::tempdir().unwrap();

        assert!(contents(&dir.path().join("nothing")).unwrap().is_empty());
    }

    #[test]
    fn a_base_without_a_seed_is_a_fault() {
        let dir = tempfile::tempdir().unwrap();

        assert!(materialize(dir.path(), "laravel", &dir.path().join("cache")).is_err());
    }

    #[test]
    fn a_base_runs_its_seed_once() {
        let dir = tempfile::tempdir().unwrap();
        let bases = dir.path().join("bases/laravel");
        fs::create_dir_all(&bases).unwrap();
        fs::write(
            bases.join("seed.sh"),
            "printf x >> $(dirname $0)/../../count\n",
        )
        .unwrap();

        let cache = dir.path().join("cache");
        let counter = dir.path().join("count");

        materialize(&dir.path().join("bases"), "laravel", &cache).unwrap();
        materialize(&dir.path().join("bases"), "laravel", &cache).unwrap();

        assert_eq!(fs::read_to_string(&counter).unwrap(), "x");
    }
}
