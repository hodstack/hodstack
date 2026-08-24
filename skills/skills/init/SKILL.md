---
name: init
description: "Write the intention of this project in `.hod/PROJECT.md`. Use when the user asks to set up this project for a coding agent, when `.hod/PROJECT.md` holds the text that `hod init` wrote, or when a task needs the command that installs the dependencies, the command that runs the tests or the command that starts the program and `.hod/PROJECT.md` names none of them."
disable-model-invocation: true
---

# Init

Write `.hod/PROJECT.md` from the code of this project. The `AGENTS.md` file of this project tells each model to read that file first, thus the file carries the intention and the commands of the project.

## 1. Read the file

Read `.hod/PROJECT.md`. The file must name four items: the intention of the project, the command that installs the dependencies, the command that runs the tests, and each directory at the top of the project.

Continue with section 2 when the file holds the text that `hod init` wrote. That text asks the reader to write the intention, and it names no command of this project.

Stop when the file names each of the four items. Say that the file is ready, then give the report of section 5. Write no file.

Continue with section 2 for the item that is absent when the file names one item and not each one. Keep the text that the user wrote.

## 2. Read the project

Take each fact from a file of this project. Ask the user for a fact that no file gives, and ask no question that a file answers.

Read the manifest at the top of the project, such as `composer.json`, `package.json`, `Cargo.toml`, `pyproject.toml` or `go.mod`. It gives the name of the project, the dependencies and the scripts. The lock file names the package manager: `pnpm-lock.yaml` gives `pnpm`, and `poetry.lock` gives `poetry`.

Read `README.md`, `CONTRIBUTING.md` and the file of the continuous integration, such as `.github/workflows/ci.yml`. The workflow gives the command that the project runs on each push, thus it gives the test command that the project trusts.

Run the test command one time before you write it in the file. Correct the name when the shell reports that the command is absent. Keep the command when the tests fail, because a test that fails does not make the command wrong.

Name each directory at the top of the project. Read enough of a directory to say what it holds in one line.

## 3. Write the file

Write `.hod/PROJECT.md` in this form. Write one paragraph on one line.

```markdown
# The intention of this project

Pest is a testing framework with a focus on simplicity. A PHP developer uses it to write a test for an application or for a package.

Install the dependencies with `composer install`. Run the tests with `composer test`. Start the program with `./bin/pest`.

- `src/` — the framework: the plugins, the expectation API and the test case
- `bin/` — the executable `pest`
- `tests/` — the tests of the framework, in Pest itself
```

Give the intention one paragraph: what the project does, and who uses it. Write the exact command in `code font`. Give one line to one directory.

Write a fact that the code holds in the code. `.hod/PROJECT.md` holds two items: the intention of the project, and a reason that the code cannot hold.

Write no rule in this file. A rule needs a fault that happened, and each rule of the project sits in one file in `.hod/rules/`.

## 4. Ask the user

Show the text of the file and ask the user to accept it. Name each fact that you took from a file, and name each fact that you guessed.

## 5. Report

Give the path `.hod/PROJECT.md` and say what the file holds now.

Say that a rule of this project goes in a file in `.hod/rules/`, and that a skill of this project goes in a directory in `.hod/skills/`.

Tell the user to run `hod list` to name each skill that this project carries.
