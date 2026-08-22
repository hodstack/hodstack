---
name: deps-upgrade
description: "Raise each dependency of this project to a newer version and keep the tests green. Use when the user asks to upgrade the dependencies, to raise one package to a new version, to bump each version in a manifest, or to remove an old version from a lock file."
disable-model-invocation: true
---

# Deps Upgrade

Raise each dependency of this project to a newer version, one step at a time, and stop at the first step that breaks the tests.

## 1. Read the project

Read `.hod/project.md` for the command that installs the dependencies and the command that runs the tests.

Find each manifest at the top of the project. Section 3 names the command of each manifest.

Run `git status --short`. Stop when a file carries a change, and ask the user to commit that change first.

## 2. Take the baseline

Write no test during this work. A test that you write now covers the code that you changed, thus it shows no regression.

When the project has no test, tell the user that you found no test and that no test can show a regression, then ask the user to continue or to stop. Wait for the answer.

Run the tests before you change a file. Stop when a test fails, and report that the failure came before this work.

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

## 6. Raise one major version at a time

Read the release notes of the package between the two versions. Search the project for each breaking change that the notes name.

When no breaking change touches the project, raise the package and run the tests.

When a breaking change touches the project, give the user the name of the package, its two versions, each breaking change and each file that the change touches, then ask the user to continue or to skip the package. Wait for the answer.

After the user continues, raise the package, apply each change that the notes name, then run the tests.

Return the manifest and the lock file with `git checkout --` after a step that fails and that you cannot correct, then continue with the next package.

## 7. Align the manifest with the lock file

Run the align command of section 3 for each manifest after the last package, then run the tests. The align command writes the installed version into the manifest, thus a manifest that holds `^1.0` against an installed `1.9.3` holds `^1.9.3` after this step.

Return the manifest and the lock file with `git checkout --` when a test fails.

## 8. Report

Write no commit. Leave each change in the working tree.

Give three lists: each package that you raised with its two versions, each package that you kept with the reason, and each file of the code that you changed.
