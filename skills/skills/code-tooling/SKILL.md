---
name: code-tooling
description: "The rules for each check of a project. Use when you finish a change, when a check of this project fails, when you add an ignore, an entry of a baseline or a suppression, when you change the level of a static analyser, when you add a path to an exclusion, when a test that your change touches is skipped, when you add a check to this project, and when the code carries `@phpstan-ignore`, `@psalm-suppress`, `@ts-ignore`, `@ts-expect-error`, `eslint-disable`, `type: ignore`, `noqa`, `nolint`, `rubocop:disable`, `#[allow(`, `@SuppressWarnings` or `@Suppress`, and when a file that you touch carries one of those markers or a marker of a check of this project."
---

# Code Tooling

Obey each rule of this file each time that you finish a change and each time that a check of this project fails.

A check is a command that gives a fault of the code back: the static analyser, the tests, the coverage of the types, the formatter and the checker of the typos.

## 1. Run each check of this project

Read `.hod/PROJECT.md` for the command of each check. Run each of those commands after your last change and before your report. Read the file that each command runs for the level of that check and for the marker that it reads.

Run the command of the project, and not the binary of the tool. The command of the project carries the configuration, the paths and the arguments of this project, thus the result of it agrees with the result of the pipeline.

Read the scripts of the manifest of the project for a check that `.hod/PROJECT.md` does not name: `composer.json`, `package.json`, `Makefile`, `Cargo.toml` and `pyproject.toml`. Tell the user to run `hod init` when `.hod/PROJECT.md` names no check.

## 2. Weaken no check to make it pass

Correct the code that a check names.

Lower no level of a check. Add no path to an exclusion. Add no entry to a baseline. Mark no test skipped, delete no assertion of a test, and delete no test.

A check that passes because you turned it off gives the user a wrong report. Change the configuration of a check for one purpose: to raise the level of that check, in the form of section 4.

## 3. Write no suppression

Write no suppression of a fault of a static analyser and no suppression of a fault of a linter. Each item of this table turns one rule off for one line, and the fault stays in the code.

Delete each suppression that a file of your change already holds, and correct the code that gave the fault under it. A suppression that you leave in a file that you touch is a suppression that you wrote. Read each file of your change for a marker of this table and for the marker of a check of this project, after your last change and before your report. A file that your change does not touch stays under section 5.

| The language | The suppression |
| --- | --- |
| PHP | `@phpstan-ignore`, `@phpstan-ignore-next-line`, `@psalm-suppress` |
| TypeScript | `@ts-ignore`, `@ts-expect-error`, `eslint-disable` |
| Java | `@SuppressWarnings` |
| Kotlin | `@Suppress` |
| C# | `#pragma warning disable` |
| Swift | `// swiftlint:disable` |
| Rust | `#[allow(...)]` |
| Python | `# type: ignore`, `# noqa` |
| Go | `//nolint` |
| Ruby | `# rubocop:disable` |

A check that this project holds carries a marker of its own, such as a comment that turns one rule of that check off for one line. Read the code of that check for the marker that it reads, and treat that marker as a row of this table.

Keep one suppression for one condition: the analyser cannot read the type of a library, because that library builds its methods at run time or reaches the code of a different language. Give that suppression the one line that needs it, name the rule that it turns off, and name the library in the sentence of your report.

Turn the report of an unmatched suppression on, such as `reportUnmatchedIgnoredErrors: true` for PHPStan. A suppression that the code no longer needs then fails the check.

## 4. Set each check at its highest level

Set the highest level of this table in a check that you add to this project.

| The tool | The highest level |
| --- | --- |
| PHPStan | `level: max` |
| Psalm | `errorLevel="1"` |
| TypeScript | `"strict": true` in `tsconfig.json` |
| mypy | `strict = true` |
| The coverage of the types | the minimum at 100 |

Ask the user before you raise a level that this project holds today. Give the command, the level today, the level that you propose and the count of the faults that the raise gives. A raise gives a fault in a file that your task does not touch, thus the raise and your task are two changes.

## 5. Stop when a check fails for a reason outside your change

Read the fault and read your change. The fault came before your work when no line of your change touches the file and the rule that the fault names.

Give the user the command, the output of that command and that reason. Ask the user to correct that fault first or to continue with it, and wait for the answer. Turn no check off to continue.

## 6. Report

Name each check that you ran, and give the command and the result of each one.

Name each fault that a check gave, and name the line of the code that you corrected for it.

Name each file of your change, and say for each one that it held a suppression or that it held none.

Name each suppression that you deleted, give the line of the code that you corrected for it, and give the type or the value that you wrote there. Name each suppression that you kept, give the one line of it and name the library that needs it.

Name each level that you raised, and give the value before and the value after.

Name each check that failed for a reason outside your change, and give the command and the output of it.

Say that you weakened no check.

Write this report in the last message of your answer. Write no sentence for a rule of this file that your work did not touch.
