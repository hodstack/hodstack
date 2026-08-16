use std::fs;

use snapbox::cmd::Command;
use snapbox::{assert_data_eq, file};

const HOD: &str = env!("CARGO_BIN_EXE_hod");

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
