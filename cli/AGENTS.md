# Hodstack CLI

This directory holds the `hod` program. The program runs the skills. The command `hod <skill>` starts the coding agent of the user with `/<skill>` as its first prompt.

The `AGENTS.md` file at the top level gives the intention of the project and the rules for this file. This file gives the decisions for this directory.

---

## 1. The function of the program

The `skills` directory holds the material. This directory moves the material to the work. The user runs a command such as `hod pr-review` in a terminal. A skill carries a hyphen on the command line and on the disk: the command `hod pr-review` starts the skill in `skills/skills/pr-review/`. Write no colon in the name of a skill, because each client reads the name of the directory and a slash command holds no colon.

The user can start each skill with this program. The Agent Plugins standard has no method to show that a skill is for a user or for a model (refer to `skills/AGENTS.md`, section 3). This program is the start method that the standard does not have.

Write the program in Rust. Compile one binary with the name `hod`. Give the binary no dependence on a runtime on the computer of the user.

---

## 2. The commands

The program gives four commands and one skill. `init` writes the files of a project into the current directory, then starts the coding agent with the skill `init`. `list` names each skill. `update` installs the newest build over this one and writes the project files again, `update --project` writes those files alone, `update --check` reports each change without a write, and `update --force` writes over a file that the program does not own. `completions` writes a completion script for a shell. A first argument that is not one of these four names is a skill, thus a skill cannot take the name of a command. The skill `init` is the one exception, because the command `init` starts it, and a test in `src/skills.rs` reads the two rules.

`build.rs` reads `../skills/skills/` and writes a table of `include_str!` into `OUT_DIR`, thus the binary carries each skill and the skill of a release agrees with the program of that release. That directory is absent in the crate that `cargo package` writes, thus `cargo make publish` copies the tree to `cli/skills/` and `build.rs` reads that copy. `Cargo.toml` names `/skills/**`, `/build.rs` and `/templates/*.md` in the key `include`. Write a new file that the binary reads into that key too.

`.hod/lock` holds the sum of each file that the program wrote. The program writes over a file when the lock records it with the sum that the file still has, and it reports `Skipped` for each other file, thus a command never replaces the work of the user. `init` refuses each directory that holds `AGENTS.md` or `CLAUDE.md`, because the text of that file belongs to `.hod/PROJECT.md`.

The function `report` in `src/lib.rs` gives the exit code 0 for a broken pipe. The command `hod | head` closes the output before the program writes each line, thus the program stops without a message and reports success.

`init`, `update` and `hod <skill>` have a body today. Write `unimplemented!()` in a command without a body, and write `#[expect(clippy::unimplemented, reason = "...")]` above that command. The lint `clippy::unimplemented` carries the level `deny` in `Cargo.toml`, thus the compiler reports the expectation as unfulfilled on the day that the command receives a body. Delete the attribute in the same change.

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

The file `.gitignore` sits at the top of the repository and ignores `/cli/.agents/skills/`, `/cli/.claude/skills/` and `/cli/skills/`. The file `skills-lock.json` records the source and the hash of each vendored skill.

The file `.github/workflows/ci.yml` gives one job for each task. The job `Rust` runs the format, the clippy, the documentation, the tests and the release build. The other jobs run one tool each.

The organization `hodstack` refuses an action that a tag or a branch names. Write the full commit SHA of the action after each `uses:` in `.github/workflows/`, then write the version in a comment after the SHA, because Dependabot reads that comment and raises the SHA with it. Give `dtolnay/rust-toolchain` the SHA of the tag `v1` and name the toolchain in the input `toolchain`, because the action reads the toolchain from the ref when that input is absent.

---

## 6. Distribution

The branch `0.x` is the release. The project writes no tag for a version today. The workflow `.github/workflows/release.yml` starts when the workflow `CI` reports success on that branch, thus a push gives new artifacts and a build that fails a check reaches no user.

The workflow writes one release for each push. The tag is `edge-<date>-<time>` in UTC, and the release carries the mark `latest`. The address of each artifact thus stays the same: `https://github.com/hodstack/hodstack/releases/latest/download/hod-<target>.tar.gz`. `install.sh`, `install.ps1` and `npm/install.js` hold that address. Do not write the version into a file name.

The repository holds the setting for an immutable release. A published release thus accepts no new file and no change, and one tag serves one release. Give each release a new tag, and do not use the tag of a release that exists: GitHub keeps the name of that tag for ever, and the name `edge` is spent. Do not give a release the mark `prerelease`: the address `releases/latest/` passes a release with that mark.

The step `Delete each release after the tenth` keeps ten releases. GitHub accepts the deletion of an immutable release, and it keeps the name of the tag.

The job `build` gives five targets: `aarch64-apple-darwin`, `x86_64-apple-darwin`, `x86_64-unknown-linux-musl`, `aarch64-unknown-linux-musl` and `x86_64-pc-windows-msvc`. Linux uses musl, because musl gives a static binary and the binary thus has no dependence on the glibc version of the computer of the user.

Each archive carries the binary and `LICENSE.md` in the format `tar.gz`, for each target. Windows 10 and later carry `tar.exe`, thus one format serves each installer.

The job `publish` writes `checksums.txt` from each archive. Each installer reads that file and stops when the sum does not agree. Keep that test in a new installer. The job writes `version.txt` too, with the version of the crate and the full commit in one line, and `hod update` reads that file to find the newest build.

The function `version` in `src/lib.rs` reads the variable `HOD_COMMIT` with `option_env!`. The job `build` gives that variable the commit, thus `hod --version` names the build. The variable is absent in a local build, thus the snapshot in `tests/snapshots/root.txt` holds the version alone.

The job `npm` publishes the directory `npm/` with the same version and the dist-tag `edge`. The package downloads the binary from the newest release, thus the version of the package names the build that published it and not the build that the user receives.

The job publishes the same files three times, with the names `hodstack`, `@hodstack/cli` and `@hodstack/hod`. `npm pkg set name=...` writes each name before each publication. The name `hod` on npm belongs to a different supplier. The two names with the prefix `@hodstack/` need the organization `hodstack` on npm.

The dist-tag `latest` holds the first version, because npm gives that tag to the first version of a new package. The job moves the tag `edge` only. The package downloads the newest release at the installation, thus each dist-tag gives the newest binary. Move the tag `latest` by hand at the first release that you announce to a user.

The job `npm` needs the secret `NPM_TOKEN`. The job reads that secret through a variable in `env`, because the context `secrets` does not reach the key `if` of a step. The job without its secret does no step and reports success.

The three installers sit at the top of the repository: `install.sh`, `install.ps1` and `npm/`. They install the binary of this directory, thus this section controls them. Write no comment in them. Refer to the `AGENTS.md` file at the top level, section 4.

The job `publish` writes `install.sh` and `install.ps1` to each release. `README.md` gives the address `releases/latest/download/install.sh` to the user, thus the project needs no website to install the binary.

`install.sh` reads the variable `HOD_RELEASE_URL`. Give that variable a `file://` address to test the installer without a release.

The crate `hod` goes to crates.io by hand. Run `cargo make publish:test` before each publication, then `cargo make publish`. The two tasks copy the skills into `cli/skills/` and give `cargo` the flag `--allow-dirty`, because that copy is not in git. No workflow publishes the crate, because the branch `0.x` gives many builds for one version and crates.io accepts one version one time.

`README.md` in this directory is the page of the crate on crates.io. The lint `clippy::cargo_common_metadata` asks for the key `readme`, and a path that leaves `cli/` does not reach the package. Keep this file short and give the address of the repository. Obey the `AGENTS.md` file at the top level, section 3: a user reads this file.
