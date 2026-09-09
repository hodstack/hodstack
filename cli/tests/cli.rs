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
fn an_unknown_option_is_a_fault_of_use() {
    Command::new(HOD).arg("--nope").assert().code(2);
}

#[test]
fn a_command_without_a_body_fails() {
    Command::new(HOD)
        .args(["completions", "zsh"])
        .assert()
        .failure();
}

#[test]
fn a_second_argument_after_a_skill_is_a_fault_of_use() {
    Command::new(HOD)
        .args(["deps-upgrade", "write a rule"])
        .assert()
        .code(2);
}

#[test]
fn a_skill_that_is_absent_is_a_fault() {
    Command::new(HOD)
        .arg("nope")
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
        fs::read_to_string(dir.path().join(".hod/PROJECT.md")).unwrap(),
        include_str!("../templates/PROJECT.md")
    );
    assert!(dir.path().join(".hod/lock").is_file());
    assert!(
        dir.path().join(".claude/skills/init/SKILL.md").is_file(),
        "the command materialized no skill"
    );
    assert!(dir.path().join(".agents/skills/init/SKILL.md").is_file());
    assert!(
        dir.path()
            .join(".claude/skills/deps-upgrade/SKILL.md")
            .is_file()
    );
    assert!(
        dir.path()
            .join(".agents/skills/deps-upgrade/SKILL.md")
            .is_file()
    );
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

    assert!(agents.contains("## 6. The rules of this project"));
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
            "\n  Kept     .hod/PROJECT.md\n  Created  AGENTS.md\n  Kept     CLAUDE.md\n  Kept     .claude/skills/code-quality\n  Kept     .agents/skills/code-quality\n  Kept     .claude/skills/deps-upgrade\n  Kept     .agents/skills/deps-upgrade\n  Kept     .claude/skills/init\n  Kept     .agents/skills/init\n\n",
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
            "\n  Kept     .hod/PROJECT.md\n  Skipped  AGENTS.md\n           This file is yours. Run `hod update --force` to write over it.\n  Kept     CLAUDE.md\n  Kept     .claude/skills/code-quality\n  Kept     .agents/skills/code-quality\n  Kept     .claude/skills/deps-upgrade\n  Kept     .agents/skills/deps-upgrade\n  Kept     .claude/skills/init\n  Kept     .agents/skills/init\n\n",
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
            "\n  Kept     .hod/PROJECT.md\n  Kept     AGENTS.md\n  Kept     CLAUDE.md\n  Kept     .claude/skills/code-quality\n  Kept     .agents/skills/code-quality\n  Kept     .claude/skills/deps-upgrade\n  Kept     .agents/skills/deps-upgrade\n  Kept     .claude/skills/init\n  Kept     .agents/skills/init\n  Removed  .agents/skills/deploy\n  Removed  .claude/skills/deploy\n\n",
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
            "\nUSER SKILLS\n  deps-upgrade  Raise each dependency of this project to a newer version and keep the tests green.\n  init          Write the intention of this project in `.hod/PROJECT.md`.\n\nMODEL SKILLS\n  code-quality  The rules for the parameters, the properties and the return type of a signature.\n\nPROJECT SKILLS\n  deploy        Deploy this project.\n\n",
        );
}

#[cfg(unix)]
#[expect(clippy::unwrap_used, reason = "a test stops on a fault of its own")]
fn agent(dir: &Path, binary: &str, record: &Path) -> PathBuf {
    use std::os::unix::fs::PermissionsExt as _;

    let bin = dir.join("bin");
    fs::create_dir_all(&bin).unwrap();

    let file = bin.join(binary);
    fs::write(
        &file,
        format!("#!/bin/sh\nprintf '%s' \"$*\" > {}\n", record.display()),
    )
    .unwrap();
    fs::set_permissions(&file, fs::Permissions::from_mode(0o755)).unwrap();

    bin
}

#[cfg(unix)]
#[test]
fn a_skill_starts_the_agent_with_its_slash_command() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("record");
    let bin = agent(dir.path(), "claude", &record);

    Command::new(HOD)
        .arg("deps-upgrade")
        .current_dir(dir.path())
        .env("PATH", &bin)
        .env_remove("HOD_AGENT")
        .assert()
        .success();

    assert_eq!(fs::read_to_string(&record).unwrap(), "/deps-upgrade");
}

#[cfg(unix)]
#[test]
fn init_writes_the_files_and_starts_the_agent_with_the_skill_init() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("record");
    let bin = agent(dir.path(), "claude", &record);

    Command::new(HOD)
        .arg("init")
        .current_dir(dir.path())
        .env("PATH", &bin)
        .env_remove("HOD_AGENT")
        .assert()
        .success();

    assert!(dir.path().join("AGENTS.md").is_file());
    assert_eq!(fs::read_to_string(&record).unwrap(), "/init");
}

#[cfg(unix)]
#[test]
fn init_starts_no_agent_after_it_keeps_a_file_that_exists() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("record");
    let bin = agent(dir.path(), "claude", &record);
    fs::write(dir.path().join("AGENTS.md"), "mine").unwrap();

    Command::new(HOD)
        .arg("init")
        .current_dir(dir.path())
        .env("PATH", &bin)
        .env_remove("HOD_AGENT")
        .assert()
        .failure();

    assert!(!record.exists(), "the command started the coding agent");
}

#[cfg(unix)]
#[test]
fn hod_agent_names_the_agent_that_starts() {
    let dir = tempfile::tempdir().unwrap();
    let record = dir.path().join("record");
    agent(dir.path(), "claude", &record);
    let bin = agent(dir.path(), "opencode", &record);

    Command::new(HOD)
        .arg("deps-upgrade")
        .current_dir(dir.path())
        .env("PATH", &bin)
        .env("HOD_AGENT", "opencode")
        .assert()
        .success();

    assert_eq!(
        fs::read_to_string(&record).unwrap(),
        "--prompt /deps-upgrade"
    );
}

#[test]
fn a_name_in_hod_agent_that_no_agent_carries_is_a_fault() {
    Command::new(HOD)
        .arg("deps-upgrade")
        .env("HOD_AGENT", "nope")
        .assert()
        .failure()
        .stderr_eq(
            "error: HOD_AGENT names `nope`; hod starts claude, codex, cursor-agent, opencode, gemini\n",
        );
}

#[cfg(unix)]
#[test]
fn a_computer_without_an_agent_is_a_fault() {
    let dir = tempfile::tempdir().unwrap();

    Command::new(HOD)
        .arg("deps-upgrade")
        .current_dir(dir.path())
        .env("PATH", dir.path())
        .env_remove("HOD_AGENT")
        .assert()
        .failure()
        .stderr_eq(
            "error: no coding agent is on PATH; hod starts claude, codex, cursor-agent, opencode, gemini\n",
        );
}

#[test]
fn check_reports_each_path_and_each_name_that_is_absent() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/lib.rs"), "pub const AGENTS: u8 = 0;").unwrap();
    fs::write(
        dir.path().join("AGENTS.md"),
        "The table `CLIENTS` in `src/lib.rs` names five.\nRead `src/gone.rs` first.\n",
    )
    .unwrap();

    Command::new(HOD)
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout_eq(
            "\n  AGENTS.md:1  `CLIENTS` is absent from `src/lib.rs`\n  AGENTS.md:2  `src/gone.rs` names no file of this project\n\n  2 faults.\n\n",
        );
}

#[test]
fn check_reports_success_when_each_path_and_each_name_exists() {
    let dir = tempfile::tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::write(dir.path().join("src/lib.rs"), "pub const AGENTS: u8 = 0;").unwrap();
    fs::write(
        dir.path().join("AGENTS.md"),
        "The table `AGENTS` in `src/lib.rs` names five.\n",
    )
    .unwrap();

    Command::new(HOD)
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .success()
        .stdout_eq("\n  1 file, 1 path, 1 name. Each one exists.\n\n");
}

#[test]
fn check_reads_the_intention_and_each_rule_of_the_project() {
    let dir = tempfile::tempdir().unwrap();
    hod::init(dir.path(), &mut Vec::new()).unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    fs::create_dir_all(dir.path().join(".hod/rules")).unwrap();
    fs::write(dir.path().join("src/lib.rs"), "").unwrap();
    fs::write(
        dir.path().join(".hod/PROJECT.md"),
        "Run the tests with `src/test.sh`.\n",
    )
    .unwrap();
    fs::write(
        dir.path().join(".hod/rules/one.md"),
        "---\nname: one\n---\n\nRead `src/gone.rs` first.\n",
    )
    .unwrap();

    Command::new(HOD)
        .arg("check")
        .current_dir(dir.path())
        .assert()
        .failure()
        .stdout_eq(
            "\n  .hod/PROJECT.md:1    `src/test.sh` names no file of this project\n  .hod/rules/one.md:5  `src/gone.rs` names no file of this project\n\n  2 faults.\n\n",
        );
}

#[expect(clippy::unwrap_used, reason = "a test stops on a fault of its own")]
fn git(dir: &Path, args: &[&str]) -> String {
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(dir)
        .args(args)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "hod")
        .env("GIT_AUTHOR_EMAIL", "hod@example.com")
        .env("GIT_COMMITTER_NAME", "hod")
        .env("GIT_COMMITTER_EMAIL", "hod@example.com")
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "git {} reports {}:\n{}",
        args.join(" "),
        output.status,
        String::from_utf8_lossy(&output.stderr)
    );

    String::from_utf8(output.stdout).unwrap()
}

#[expect(clippy::unwrap_used, reason = "a test stops on a fault of its own")]
fn repository(dir: &Path) -> (PathBuf, PathBuf) {
    let main = dir.join("main");
    let worktree = dir.join("feature");

    fs::create_dir_all(&main).unwrap();
    git(&main, &["init", "--quiet", "--initial-branch=trunk"]);
    fs::write(main.join("one.txt"), "one\n").unwrap();
    git(&main, &["add", "one.txt"]);
    git(&main, &["commit", "--quiet", "--message", "one"]);

    git(
        &main,
        &[
            "worktree",
            "add",
            "--quiet",
            "-b",
            "feature",
            &worktree.display().to_string(),
        ],
    );
    fs::write(worktree.join("two.txt"), "two\n").unwrap();
    git(&worktree, &["add", "two.txt"]);
    git(&worktree, &["commit", "--quiet", "--message", "two"]);

    (
        fs::canonicalize(main).unwrap(),
        fs::canonicalize(worktree).unwrap(),
    )
}

fn merge(dir: &Path) -> Command {
    Command::new(HOD)
        .arg("worktree:merge")
        .current_dir(dir)
        .env("GIT_CONFIG_GLOBAL", "/dev/null")
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_AUTHOR_NAME", "hod")
        .env("GIT_AUTHOR_EMAIL", "hod@example.com")
        .env("GIT_COMMITTER_NAME", "hod")
        .env("GIT_COMMITTER_EMAIL", "hod@example.com")
}

#[test]
fn worktree_merge_writes_the_branch_into_the_main_checkout_and_removes_the_worktree() {
    let dir = tempfile::tempdir().unwrap();
    let (main, worktree) = repository(dir.path());

    merge(&worktree)
        .stdin("fold feature into trunk\n")
        .assert()
        .success()
        .stdout_eq(format!(
            "\n  Message    Merged   feature → trunk\n  Removed  {}\n\ncd {}\n",
            worktree.display(),
            main.display()
        ));

    assert!(main.join("two.txt").is_file());
    assert!(!worktree.exists(), "the worktree remains");
    assert_eq!(git(&main, &["branch", "--list", "feature"]), "");
    assert_eq!(
        git(&main, &["rev-list", "--count", "HEAD"]).trim(),
        "3",
        "the merge is not a merge commit"
    );
    assert_eq!(
        git(&main, &["log", "-1", "--format=%s"]).trim(),
        "fold feature into trunk"
    );
}

#[test]
fn worktree_merge_with_keep_keeps_the_worktree_and_the_branch() {
    let dir = tempfile::tempdir().unwrap();
    let (main, worktree) = repository(dir.path());

    merge(&worktree)
        .args(["keep feature", "--keep"])
        .assert()
        .success()
        .stdout_eq(format!(
            "\n  Merged   feature → trunk\n\ncd {}\n",
            main.display()
        ));

    assert_eq!(
        git(&main, &["log", "-1", "--format=%s"]).trim(),
        "keep feature"
    );

    assert!(main.join("two.txt").is_file());
    assert!(worktree.join("two.txt").is_file());
    assert_eq!(
        git(&main, &["branch", "--list", "feature"]).trim(),
        "+ feature"
    );
}

#[test]
fn worktree_merge_from_the_main_checkout_is_a_fault() {
    let dir = tempfile::tempdir().unwrap();
    let (main, _) = repository(dir.path());

    merge(&main).assert().failure().stderr_eq(format!(
        "error: `{}` is not a linked worktree\n",
        main.display()
    ));
}

#[test]
fn worktree_merge_outside_a_repository_is_a_fault() {
    let dir = tempfile::tempdir().unwrap();
    let here = fs::canonicalize(dir.path()).unwrap();

    merge(&here).assert().failure().stderr_eq(format!(
        "error: `{}` is not a linked worktree\n",
        here.display()
    ));
}

#[test]
fn worktree_merge_with_a_detached_head_is_a_fault() {
    let dir = tempfile::tempdir().unwrap();
    let (main, worktree) = repository(dir.path());
    git(&worktree, &["checkout", "--quiet", "--detach"]);

    merge(&worktree).assert().failure().stderr_eq(format!(
        "error: `{}` has no branch; HEAD is detached\n",
        worktree.display()
    ));

    assert!(!main.join("two.txt").exists());
}

#[test]
fn worktree_merge_with_a_change_in_the_worktree_is_a_fault() {
    let dir = tempfile::tempdir().unwrap();
    let (main, worktree) = repository(dir.path());
    fs::write(worktree.join("two.txt"), "more\n").unwrap();

    merge(&worktree).assert().failure().stderr_eq(format!(
        "error: `{}` has a change that no commit holds; commit it or stash it\n",
        worktree.display()
    ));

    assert!(!main.join("two.txt").exists());
}

#[test]
fn worktree_merge_with_a_change_in_the_main_checkout_is_a_fault() {
    let dir = tempfile::tempdir().unwrap();
    let (main, worktree) = repository(dir.path());
    fs::write(main.join("three.txt"), "three\n").unwrap();

    merge(&worktree).assert().failure().stderr_eq(format!(
        "error: `{}` has a change that no commit holds; commit it or stash it\n",
        main.display()
    ));

    assert!(!main.join("two.txt").exists());
}

#[test]
fn worktree_merge_with_a_conflict_names_each_path_and_keeps_the_worktree() {
    let dir = tempfile::tempdir().unwrap();
    let (main, worktree) = repository(dir.path());
    fs::write(worktree.join("one.txt"), "one on feature\n").unwrap();
    git(
        &worktree,
        &["commit", "--quiet", "--all", "--message", "feature"],
    );
    fs::write(main.join("one.txt"), "one on trunk\n").unwrap();
    git(&main, &["commit", "--quiet", "--all", "--message", "trunk"]);

    merge(&worktree).stdin("").assert().failure().stdout_eq(format!(
        "\n  Message    Conflict  one.txt\n\n  `{}` holds no merge. Merge `trunk` into `feature` in `{}`, then run `hod worktree:merge` again.\n\n",
        main.display(),
        worktree.display()
    ));

    assert!(worktree.join("two.txt").is_file());
    assert!(!main.join("two.txt").exists());
    assert!(
        !main.join(".git/MERGE_HEAD").exists(),
        "the merge is not aborted"
    );
    assert_eq!(git(&main, &["status", "--porcelain"]), "");
    assert_eq!(git(&main, &["rev-list", "--count", "HEAD"]).trim(), "2");
}

#[test]
fn worktree_merge_without_a_message_keeps_the_message_of_git() {
    let dir = tempfile::tempdir().unwrap();
    let (main, worktree) = repository(dir.path());

    merge(&worktree).stdin("\n").assert().success();

    assert_eq!(
        git(&main, &["log", "-1", "--format=%s"]).trim(),
        "Merge branch 'feature' into trunk"
    );
}

#[test]
fn worktree_create_adds_a_branch_and_a_worktree_in_the_home_directory() {
    let dir = tempfile::tempdir().unwrap();
    let (main, _) = repository(dir.path());
    let added = dir.path().join(".hod/worktrees/main/fix-help");

    Command::new(HOD)
        .args(["worktree:create", "fix/help"])
        .env("HOME", dir.path())
        .current_dir(&main)
        .assert()
        .success()
        .stdout_eq(format!(
            "\n  Created  {}\n  Branch   fix/help\n\ncd {}\n",
            added.display(),
            added.display()
        ));

    assert!(added.join("one.txt").is_file());
    assert_eq!(
        git(&added, &["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        "fix/help"
    );
    assert_eq!(git(&main, &["status", "--porcelain"]), "");
}

#[test]
fn worktree_create_without_a_branch_names_the_branch_with_two_words() {
    let dir = tempfile::tempdir().unwrap();
    let (main, _) = repository(dir.path());
    let worktrees = dir.path().join(".hod/worktrees/main");

    let output = Command::new(HOD)
        .arg("worktree:create")
        .env("HOME", dir.path())
        .current_dir(&main)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let branch = stdout
        .lines()
        .find_map(|line| line.trim().strip_prefix("Branch   "))
        .unwrap();
    let added = worktrees.join(branch);

    assert!(output.status.success(), "{stdout}");
    assert_eq!(branch.split('-').count(), 2, "{branch}");
    assert!(
        branch.chars().all(|c| c.is_ascii_lowercase() || c == '-'),
        "{branch}"
    );
    assert!(added.join("one.txt").is_file());
    assert_eq!(
        git(&added, &["rev-parse", "--abbrev-ref", "HEAD"]).trim(),
        branch
    );
    assert!(stdout.ends_with(&format!("cd {}\n", added.display())));
}

#[test]
fn worktree_create_takes_a_branch_that_exists() {
    let dir = tempfile::tempdir().unwrap();
    let (main, worktree) = repository(dir.path());
    let added = dir.path().join(".hod/worktrees/main/feature");
    git(
        &main,
        &["worktree", "remove", &worktree.display().to_string()],
    );

    Command::new(HOD)
        .args(["worktree:create", "feature"])
        .env("HOME", dir.path())
        .current_dir(&main)
        .assert()
        .success();

    assert!(added.join("two.txt").is_file());
}

#[test]
fn worktree_create_with_a_branch_that_has_a_worktree_is_a_fault() {
    let dir = tempfile::tempdir().unwrap();
    let (main, _) = repository(dir.path());

    Command::new(HOD)
        .args(["worktree:create", "feature"])
        .env("HOME", dir.path())
        .current_dir(&main)
        .assert()
        .failure()
        .stderr_eq(format!(
            "error: cannot create `{}`: 'feature' is already used by worktree at [..]\n",
            dir.path().join(".hod/worktrees/main/feature").display()
        ));
}
