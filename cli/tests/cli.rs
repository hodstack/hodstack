use std::fs;
use std::path::{Path, PathBuf};

use sha2::{Digest as _, Sha256};
use snapbox::cmd::Command;
use snapbox::{assert_data_eq, file};

const HOD: &str = env!("CARGO_BIN_EXE_hod");

const NEWEST: &str = "9.9.9 3f2b1c9d8e7f6a5b4c3d2e1f0a9b8c7d6e5f4a3b";

#[test]
fn the_root_screen_lists_each_command() {
    let mut command = hod::command();

    let screen = command.render_help().to_string();

    assert_data_eq!(screen, file!["snapshots/root.txt"]);
}

#[test]
fn the_root_screen_names_the_path_of_this_binary() {
    let mut command = hod::command();
    let binary = std::env::current_exe().unwrap();

    let screen = command.render_help().to_string();

    assert!(
        screen.trim_end().ends_with(&binary.display().to_string()),
        "the last line of the screen is not the path of this binary:\n{screen}"
    );
}

#[test]
fn the_root_screen_carries_a_style() {
    let mut command = hod::command();

    let screen = command.render_help().ansi().to_string();

    assert!(screen.contains('\u{1b}'), "the screen carries no style");
}

#[test]
fn no_argument_gives_the_root_screen_and_reports_success() {
    Command::new(HOD).assert().success().stderr_eq("");
}

#[test]
fn an_unknown_command_is_a_fault_of_use() {
    Command::new(HOD).arg("nope").assert().code(2);
}

#[test]
fn each_command_without_a_body_fails() {
    Command::new(HOD)
        .args(["learn", "write a rule"])
        .assert()
        .failure();
    Command::new(HOD)
        .args(["completions", "zsh"])
        .assert()
        .failure();
}

#[test]
fn a_skill_that_is_absent_is_a_fault() {
    Command::new(HOD)
        .args(["nope", "a prompt"])
        .assert()
        .failure()
        .stderr_eq("error: no skill has the name `nope`; run `hod list` to name each skill\n");
}

#[expect(clippy::panic, reason = "a test stops when this platform has no build")]
fn target() -> &'static str {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("linux", "aarch64") => "aarch64-unknown-linux-musl",
        ("linux", "x86_64") => "x86_64-unknown-linux-musl",
        ("windows", "x86_64") => "x86_64-pc-windows-msvc",
        (os, arch) => panic!("no build for {os} {arch}"),
    }
}

#[expect(clippy::unwrap_used, reason = "a test stops on a fault of its own")]
fn release(dir: &Path, body: &str, sound: bool) -> String {
    let name = if cfg!(windows) { "hod.exe" } else { "hod" };
    let file = format!("hod-{}.tar.gz", target());
    let staging = dir.join("staging");

    fs::create_dir_all(&staging).unwrap();
    fs::write(staging.join(name), body).unwrap();

    std::process::Command::new("tar")
        .arg("-czf")
        .arg(dir.join(&file))
        .arg("-C")
        .arg(&staging)
        .arg(name)
        .status()
        .unwrap();

    let sum = format!("{:x}", Sha256::digest(fs::read(dir.join(&file)).unwrap()));
    let sum = if sound { sum } else { "0".repeat(64) };

    fs::write(dir.join("checksums.txt"), format!("{sum}  {file}\n")).unwrap();
    fs::write(dir.join("version.txt"), format!("{NEWEST}\n")).unwrap();

    format!("file://{}", dir.display())
}

#[expect(clippy::unwrap_used, reason = "a test stops on a fault of its own")]
fn installed(dir: &Path) -> PathBuf {
    let name = if cfg!(windows) { "hod.exe" } else { "hod" };
    let binary = dir.join(name);

    fs::copy(HOD, &binary).unwrap();

    binary
}

fn update(binary: &Path, home: &Path, address: &str) -> Command {
    Command::new(binary)
        .arg("update")
        .env("HOD_RELEASE_URL", address)
        .env("HOME", home)
        .env("XDG_CACHE_HOME", home.join("cache"))
        .env("LOCALAPPDATA", home.join("cache"))
}

#[test]
fn update_writes_the_newest_build_over_this_one() {
    let home = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let address = release(dir.path(), "the newest build", true);
    let binary = installed(home.path());

    update(&binary, home.path(), &address)
        .assert()
        .success()
        .stdout_eq("\n  Updated  [..] → 9.9.9 (3f2b1c9)\n           [..]\n\n");

    assert_eq!(fs::read_to_string(&binary).unwrap(), "the newest build");
    assert_eq!(
        fs::read_to_string(home.path().join("cache/hod/update")).unwrap(),
        format!("{NEWEST}\n")
    );
}

#[test]
fn update_keeps_this_build_when_the_checksum_does_not_agree() {
    let home = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let address = release(dir.path(), "the newest build", false);
    let binary = installed(home.path());
    let before = fs::read(&binary).unwrap();

    update(&binary, home.path(), &address)
        .assert()
        .failure()
        .stderr_eq(
            format!(
                "error: cannot install 9.9.9 (3f2b1c9): the checksum of hod-{}.tar.gz does not agree with checksums.txt\n",
                target()
            ),
        );

    assert_eq!(fs::read(&binary).unwrap(), before);
}

#[test]
fn update_check_names_the_newest_build_and_writes_nothing() {
    let home = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let address = release(dir.path(), "the newest build", true);
    let binary = installed(home.path());
    let before = fs::read(&binary).unwrap();

    update(&binary, home.path(), &address)
        .arg("--check")
        .assert()
        .success()
        .stdout_eq("\n  Newest   9.9.9 (3f2b1c9)\n           Run `hod update` to install it.\n\n");

    assert_eq!(fs::read(&binary).unwrap(), before);
}

#[test]
fn update_reports_a_release_that_is_absent() {
    let home = tempfile::tempdir().unwrap();
    let dir = tempfile::tempdir().unwrap();
    let address = format!("file://{}", dir.path().join("nothing").display());
    let binary = installed(home.path());
    let before = fs::read(&binary).unwrap();

    update(&binary, home.path(), &address).assert().failure();

    assert_eq!(fs::read(&binary).unwrap(), before);
}

#[test]
fn init_writes_the_files_of_this_program_and_the_seed_of_the_user() {
    let dir = tempfile::tempdir().unwrap();
    let mut out = Vec::new();

    let code = hod::init(dir.path(), &mut out).unwrap();

    assert_eq!(code, std::process::ExitCode::SUCCESS);
    assert_eq!(
        fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
        include_str!("../templates/rules.md")
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap(),
        include_str!("../templates/CLAUDE.md")
    );
    assert_eq!(
        fs::read_to_string(dir.path().join(".hod/project.md")).unwrap(),
        include_str!("../templates/project.md")
    );
    assert!(dir.path().join(".hod/lock").is_file());
    assert!(
        dir.path().join(".claude/skills/learn/SKILL.md").is_file(),
        "the command materialized no skill"
    );
    assert!(dir.path().join(".agents/skills/learn/SKILL.md").is_file());
}

#[test]
fn init_keeps_a_file_that_exists_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "mine").unwrap();
    let mut out = Vec::new();

    let code = hod::init(dir.path(), &mut out).unwrap();

    assert_eq!(code, std::process::ExitCode::FAILURE);
    assert_eq!(
        fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
        "mine"
    );
    assert!(
        !dir.path().join("CLAUDE.md").exists(),
        "the command wrote CLAUDE.md after it kept AGENTS.md"
    );
    assert!(
        !dir.path().join(".hod").exists(),
        "the command wrote .hod after it kept AGENTS.md"
    );
}

#[test]
fn the_agents_file_names_each_rule_of_the_project() {
    let dir = tempfile::tempdir().unwrap();
    let rules = dir.path().join(".hod/rules");
    fs::create_dir_all(&rules).unwrap();
    fs::write(
        rules.join("queue-worker-restart.md"),
        "---\nname: queue-worker-restart\ndescription: Restart the queue worker.\n---\n",
    )
    .unwrap();
    let mut out = Vec::new();

    hod::init(dir.path(), &mut out).unwrap();

    let agents = fs::read_to_string(dir.path().join("AGENTS.md")).unwrap();

    assert!(agents.contains("## 5. The rules of this project"));
    assert!(agents.contains(
        "- [queue-worker-restart](.hod/rules/queue-worker-restart.md): Restart the queue worker.\n"
    ));
}

fn project(dir: &Path) -> Command {
    Command::new(HOD)
        .args(["update", "--project"])
        .current_dir(dir)
}

#[test]
fn update_project_writes_the_files_of_this_program_again() {
    let dir = tempfile::tempdir().unwrap();
    hod::init(dir.path(), &mut Vec::new()).unwrap();
    let agents = dir.path().join("AGENTS.md");
    fs::remove_file(&agents).unwrap();

    project(dir.path())
        .assert()
        .success()
        .stdout_eq(
            "\n  Kept     .hod/project.md\n  Created  AGENTS.md\n  Kept     CLAUDE.md\n  Kept     .claude/skills/learn\n  Kept     .agents/skills/learn\n\n",
        );

    assert_eq!(
        fs::read_to_string(&agents).unwrap(),
        include_str!("../templates/rules.md")
    );
}

#[test]
fn update_project_keeps_a_file_that_the_user_wrote() {
    let dir = tempfile::tempdir().unwrap();
    hod::init(dir.path(), &mut Vec::new()).unwrap();
    let agents = dir.path().join("AGENTS.md");
    fs::write(&agents, "mine").unwrap();

    project(dir.path())
        .assert()
        .failure()
        .stdout_eq(
            "\n  Kept     .hod/project.md\n  Skipped  AGENTS.md\n           This file is yours. Run `hod update --force` to write over it.\n  Kept     CLAUDE.md\n  Kept     .claude/skills/learn\n  Kept     .agents/skills/learn\n\n",
        );

    assert_eq!(fs::read_to_string(&agents).unwrap(), "mine");

    project(dir.path()).arg("--force").assert().success();

    assert_ne!(fs::read_to_string(&agents).unwrap(), "mine");
}

#[test]
fn update_project_removes_a_skill_that_this_program_does_not_carry() {
    let dir = tempfile::tempdir().unwrap();
    hod::init(dir.path(), &mut Vec::new()).unwrap();
    let mine = dir.path().join(".hod/skills/deploy");
    fs::create_dir_all(&mine).unwrap();
    fs::write(mine.join("SKILL.md"), "---\nname: deploy\n---\n").unwrap();

    project(dir.path()).assert().success();

    assert!(dir.path().join(".claude/skills/deploy/SKILL.md").is_file());

    fs::remove_dir_all(&mine).unwrap();

    project(dir.path())
        .assert()
        .success()
        .stdout_eq(
            "\n  Kept     .hod/project.md\n  Kept     AGENTS.md\n  Kept     CLAUDE.md\n  Kept     .claude/skills/learn\n  Kept     .agents/skills/learn\n  Removed  .agents/skills/deploy\n  Removed  .claude/skills/deploy\n\n",
        );

    assert!(!dir.path().join(".claude/skills/deploy").exists());
}

#[test]
fn update_project_outside_a_project_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();

    project(dir.path())
        .assert()
        .failure()
        .stdout_eq("\n  This directory has no `.hod`. Run `hod init` first.\n\n");

    assert!(!dir.path().join("AGENTS.md").exists());
}

#[test]
fn list_names_each_skill_of_the_program_and_of_the_project() {
    let dir = tempfile::tempdir().unwrap();
    hod::init(dir.path(), &mut Vec::new()).unwrap();
    let mine = dir.path().join(".hod/skills/deploy");
    fs::create_dir_all(&mine).unwrap();
    fs::write(
        mine.join("SKILL.md"),
        "---\nname: deploy\ndescription: Deploy this project.\n---\n",
    )
    .unwrap();

    Command::new(HOD)
        .arg("list")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout_eq(
            "\nUSER SKILLS\n  learn        Write one rule for this project in `.hod/rules/`.\n\nPROJECT SKILLS\n  deploy       Deploy this project.\n\n",
        );
}
