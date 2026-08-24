---
name: deps-upgrade
description: "Raise each dependency of this project to a newer version and keep the tests green. Use when the user asks to upgrade the dependencies, to raise one package to a new version, to bump each version in a manifest, or to remove an old version from a lock file."
disable-model-invocation: true
---

# Deps Upgrade

Raise each dependency of this project to a newer version, one step at a time, and stop at the first step that breaks the tests.

## 1. Read the project

Read `.hod/PROJECT.md` for the command that installs the dependencies and the command that runs the tests.

Find each manifest at the top of the project. Section 3 names the command of each manifest.

Run `git status --short`. Stop when a file carries a change, and ask the user to commit that change first.

## 2. Take the baseline

Write no test during this work. A test that you write now covers the code that you changed, thus it shows no regression.

Run the test command that `.hod/PROJECT.md` names before you change a file. Decide this step from the output of that command only. Read no test file to decide it, and judge no test by its value, because a test that the framework wrote is a test and a test that asserts a constant is a test.

When a test fails, stop and raise no package. Name each test that failed from the output of the command. Say that the failure came before this work, thus the tests can show no regression. Give the report of section 8, and say that you raised no package, because the first list is empty. Ask the user nothing and write no commit.

When the test command does not exist, or when the command runs zero tests, tell the user that you found no test and that no test can show a regression, then ask the user to continue or to stop. Wait for the answer.

## 3. The command of each manifest

| Manifest | Report | Raise one package | Align the manifest |
| --- | --- | --- | --- |
| `composer.json` | `composer outdated --direct` | `composer require <package>:^<version> --with-all-dependencies` | `composer bump` |
| `package.json` | `npm outdated` | `npm install <package>@^<version>` | `npm update --save` |
| `Cargo.toml` | `cargo upgrade --dry-run --incompatible` | `cargo add <package>@<version>` | `cargo upgrade` |
| `pyproject.toml` | `uv tree --outdated` | `uv add <package>@<version>` | `uv-bump` |
| `go.mod` | `go list -m -u all` | `go get <package>@<version>` | `go mod tidy` |

Install the tool of a command that the computer does not carry: `cargo install cargo-edit` gives `cargo upgrade`, and `uv tool install uv-bump` gives `uv-bump`.

Read the lock file of the project and use the package manager that wrote it, such as `pnpm` for `pnpm-lock.yaml` and `poetry` for `poetry.lock`.

## 4. Sort the work

Run the report command of each manifest. Write one list of each dependency that has a newer version.

Split the list in two groups. The first group holds each new minor version and each new patch version. The second group holds each new major version.

## 5. Raise the first group in one step

Raise each package of the first group, then run the tests. Ask the user nothing, because a minor version and a patch version carry no breaking change.

Report each package of the first group with the version before and the version after, and say that you ran the tests. Report this work also when a major version of section 6 stops the work later.

## 6. Raise one major version at a time

Read the release notes of the package between the two versions. The notes name a member in a qualified form, such as `Class::method()`, and the code of the project calls that member in a different form, such as `$object->method()`, thus search for the bare name of each class, each method, each function and each option that the notes name, and not for the qualified string of the notes. A search of each bare name over the code of the project decides this step, and `packages/` and `vendor/` hold no code of the project.

When no breaking change touches the project, raise the package and run the tests. When a test fails after a raise of a major version that you made without a question, the search missed a breaking change, thus return the manifest and the lock file with `git checkout --`, change no file of the code, and give the question of the next paragraph for that package.

When a breaking change touches the project, give the user the name of the package, the version that the project holds and the newer version, each breaking change and each file that the change touches, give the report of section 8 for the work that you finished, then ask the user to continue or to skip the package. Wait for the answer. The work stops until the answer arrives, thus the report comes before the question.

After the user continues, raise the package, apply each change that the notes name, then run the tests.

Return the manifest and the lock file with `git checkout --` after a step that the user approved, that fails, and that you cannot correct, then continue with the next package. This rule does not cover a raise of a major version that you made without a question.

## 7. Align the manifest with the lock file

Run the align command of section 3 for each manifest after the last package, then run the tests. The align command writes the installed version into the manifest, thus a manifest that holds `^1.0` against an installed `1.9.3` holds `^1.9.3` after this step.

Return the manifest and the lock file with `git checkout --` when a test fails.

## 8. Report

Write no commit. Leave each change in the working tree.

Give three lists: each package that you raised with the version before and the version after, each package that you kept with the version that the project holds, the newer version, and the reason, and each file of the code that you changed. Name the last change of this work, say that you ran the tests after that change, and say that those tests passed, in one sentence of its own, whenever the first list names one package. The align command of section 7 is the last change when that command ran. The three lists and that sentence are the report of section 8.

Give the report of section 8 in the last message of each answer. An answer that asks the user a question carries the report, and an answer that stops the work early carries the report. Put the report first and the question last, in that one message. Write no other sentence about a test run in that message, because two sentences about two test runs hide which run came last.
