# Hodstack CLI

This directory holds the `hod` program. The program runs the skills. The command `hod run <skill> <prompt>` starts a coding agent with that skill and that prompt.

The `AGENTS.md` file at the top level gives the intention of the project and the rules for this file. This file gives the decisions for this directory.

---

## 1. The function of the program

The `skills` directory holds the material. This directory moves the material to the work. The user runs a command such as `hod run pr:review 1042` in a terminal, in CI, or in a hook. A skill carries a colon on the command line and a hyphen on the disk: the command `hod run pr:review` starts the skill in `skills/skills/pr-review/`.

The user can start each skill with this program. The Agent Plugins standard has no method to show that a skill is for a user or for a model (refer to `skills/AGENTS.md`, section 3). This program is the start method that the standard does not have.

Write the program in Rust. Compile one binary with the name `hod`. Give the binary no dependence on a runtime on the computer of the user.

---

## 2. The commands

The program gives four commands. `init` writes `AGENTS.md` and `CLAUDE.md` into the current directory. `run` starts a skill with a prompt. `list` names each installed skill. `completions` writes a completion script for a shell.

The constants in `src/init.rs` read `../../skills/templates/` with `include_str!`, thus the text of the two files sits in the `skills` directory and the binary carries no dependence on a file on the computer of the user.

The function `init` tests each path before it writes one file, thus the command never replaces the work of the user and never leaves one file of the pair behind.

The function `report` in `src/lib.rs` gives the exit code 0 for a broken pipe. The command `hod | head` closes the output before the program writes each line, thus the program stops without a message and reports success.

Only `init` has a body today. Write `unimplemented!()` in a command without a body, and write `#[expect(clippy::unimplemented, reason = "...")]` above that command. The lint `clippy::unimplemented` carries the level `deny` in `Cargo.toml`, thus the compiler reports the expectation as unfulfilled on the day that the command receives a body. Delete the attribute in the same change.

---

## 3. The layout of the crate

`Cargo.toml` holds `[workspace]` and `[package]` together. One file thus holds the lints for each future crate, and `[profile.release]` takes effect. The key `exclude` names `.agents` and `.claude`, because the vendored skill under `.agents` holds its own `Cargo.toml`.

Keep `main.rs` at one function. Write each command in the library, because a test in `tests/` reaches the library only.

Each file of this crate carries no comment. Refer to the `AGENTS.md` file at the top level, section 4. `Cargo.toml` carries no `missing_docs` lint, thus no lint asks for a documentation comment.

Keep `command` in `src/lib.rs` public. The test in `tests/cli.rs` renders the help screen from that function, thus the snapshot needs no new process.

Give no item the macro `println!`. A function that gives output receives a writer, and the binary gives an `anstream` writer. The lints `clippy::print_stdout` and `clippy::print_stderr` carry the level `deny`, thus this rule has a test. The rule also removes the style when the output is not a terminal, and it lets a test read the output without a new process.

Declare `rust-version = "1.85"`. Edition 2024 and `clap` 4.6 give that number. Do not raise it to the version of the toolchain on your computer.

Do not write a let chain. A let chain needs Rust 1.88, thus it breaks the MSRV. Write two conditions in two `if` statements. The task `cargo make test:msrv` finds this fault.

---

## 4. The help screen

The file `src/cli.rs` holds the text of each command and of each argument in `#[command(about = "...")]` and `#[arg(help = "...")]`. Do not write that text in a `///` comment: `clap` reads the two forms in the same way, and the attribute shows the reader that the text is the output of the program.

The file `src/help.rs` holds the palette and the template. `clap` renders the screen. The template gives each heading, because `{subcommands}` and `{options}` give rows without a heading. Do not use `{all-args}`: that tag writes the headings of `clap`. The template starts with two line breaks, because `clap` removes the first line of the screen when that line is empty.

The weight of the text carries the structure of the screen, thus the screen reads on a light terminal and on a dark terminal. Give a color to a fault only.

The indent of a row and the width of the description column belong to `clap`. The constants sit in `clap_builder/src/output/`, and `clap` gives no method to change them.

The file `tests/snapshots/root.txt` holds the screen. Run `SNAPSHOTS=overwrite cargo test` to write that file again, then read the difference before you commit it.

A test of a command without a body reads the exit code as non zero, not as 101. `[profile.release]` sets `panic = "abort"`, thus a panic gives 134 in a release build.

---

## 5. The quality tools

`Makefile.toml` holds each check. `cargo-make` runs it. Run `cargo make test` before you commit, because that task runs each check that CI runs. Run `cargo make lint` to correct the format and the clippy faults that a machine can correct.

The task `test` depends on ten tasks. `test:lint` runs `cargo fmt --check` and `cargo clippy`. `test:unit` runs each test. `test:docs` builds the documentation and denies each rustdoc warning. `test:audit` reads the RustSec advisory database. `test:deny` reads the advisories, the licenses, the bans and the sources. `test:vet` reads the supply-chain audits. `test:typos` reads each word. `test:machete` finds an unused dependency. `test:coverage` measures the tests. `test:msrv` builds the crate on Rust 1.85.

Each task names the version of the tool that it installs. Raise the version in `Makefile.toml` and in `.github/workflows/ci.yml` together, because the two files must name one version.

The file `deny.toml` holds the licenses that the program accepts. Add a license to `allow` only after you read the terms of that license. The list holds Unicode-3.0 for the crate `unicode-ident`, and `clap_derive` reaches that crate through `syn`.

The file `clippy.toml` lets a test use `unwrap`, `expect`, `panic` and a print macro, because the lints in `Cargo.toml` apply to `tests/` too. Keep `".."` in `doc-valid-idents`: that entry keeps the default list of `clippy`.

`Cargo.toml` gives `multiple_crate_versions` the level `allow`, because the dependency tree of `clap` holds more than one version of some crates.

The task `test:msrv` and the job `MSRV` name the toolchain `1.85.0`. `rust-toolchain.toml` names the stable channel, and the flag `+1.85.0` and the variable in `ci.yml` take precedence over that file.

The directory `supply-chain` holds the files of `cargo-vet`. `config.toml` names four suppliers of audits and holds an exemption for each crate that no supplier audits. Run `cargo vet` after you add a dependency. Run `cargo vet prune` after an import covers a crate, thus the exemption goes away. Do not correct these files by hand. The comment at the top of each file belongs to `cargo vet`. Do not delete it: `cargo vet` fails until `cargo vet fmt` writes it again.

The file `typos.toml` sits at the top of the repository, not in this directory. The task `test:typos` thus runs one directory above this file. That file excludes `supply-chain/`, because `cargo vet` writes the crate names of other suppliers there.

The file `.gitignore` sits at the top of the repository and ignores `/cli/.agents/skills/` and `/cli/.claude/skills/`. The file `skills-lock.json` records the source and the hash of each vendored skill.

The file `.github/workflows/ci.yml` gives one job for each task. The job `Rust` runs the format, the clippy, the documentation, the tests and the release build. The other jobs run one tool each.

---

## 6. Distribution

TBD.

The manifest holds `publish = false`, because this section gives no method. Remove that line when this section gives one.

The two constants in `src/init.rs` read `../../skills/templates/`, and that path leaves the package. Move the two template files into `cli/` before you give this crate a distribution method, or `cargo package` writes a crate that does not build.
