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
        .args(["run", "pr:review", "1042"])
        .assert()
        .failure();
    Command::new(HOD).arg("list").assert().failure();
    Command::new(HOD)
        .args(["completions", "zsh"])
        .assert()
        .failure();
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
fn init_writes_both_files() {
    let dir = tempfile::tempdir().unwrap();
    let mut out = Vec::new();

    let code = hod::init(dir.path(), &mut out).unwrap();

    assert_eq!(code, std::process::ExitCode::SUCCESS);
    assert_eq!(
        fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
        include_str!("../templates/AGENTS.md")
    );
    assert_eq!(
        fs::read_to_string(dir.path().join("CLAUDE.md")).unwrap(),
        include_str!("../templates/CLAUDE.md")
    );
}

#[test]
fn init_keeps_a_file_that_exists_and_writes_nothing() {
    let dir = tempfile::tempdir().unwrap();
    fs::write(dir.path().join("AGENTS.md"), "mine").unwrap();
    let mut out = Vec::new();

    hod::init(dir.path(), &mut out).unwrap();

    assert_eq!(
        fs::read_to_string(dir.path().join("AGENTS.md")).unwrap(),
        "mine"
    );
    assert!(
        !dir.path().join("CLAUDE.md").exists(),
        "the command wrote CLAUDE.md after it kept AGENTS.md"
    );
}
