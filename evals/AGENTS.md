# Hodstack Evals

This directory holds the eval cases of the skills, and the program that runs them. A case runs a coding agent against a real project and scores what the agent did.

The `AGENTS.md` file at the top level gives the intention of the project and the rules for this file. This file gives the decisions for this directory: the shape of a case, the two kinds of assertion, the bases and the concurrency.

---

## 1. Where a case sits

A case is one directory: `<skill>/<case>/`. The name of the outer directory is the name of the skill, thus the program gives the agent `/<skill>` as its first prompt and needs no field for that prompt. Name the case for the behaviour that it holds, such as `stops-on-a-dirty-tree`, thus the report of a failure reads as a sentence.

A case holds two files and one optional directory:

```
<skill>/<case>/
├── setup.sh      plant the precondition of this case
├── test.md       the graders in the front matter, the expectation in the body
└── fixtures/     the files of this case, written over the base
```

Write no case inside `skills/skills/<name>/`. `build.rs` in `cli/` reads each file of a skill directory into the binary, thus a case in that place travels to each user.

---

## 2. The two kinds of assertion

`test.md` carries TOML front matter between two `+++` lines, and a body.

The front matter holds the graders. A grader reads the trace of the run or the files of the project, and it needs no model: `tool_used`, `tool_order`, `regex`, `file_content`, `file_exists`, `git_clean`, `git_dirty` and `head_unmoved`. `src/case.rs` holds each field of each one.

The body holds one expectation, and a model weighs it.

A run passes when each grader of the case passes and the expectation holds. The score of a case is the part of its runs that passed whole, and `threshold` gives the score that the case needs. The default of `threshold` is 1.0, thus each run must pass. Give a case a lower threshold only when more than one run measures it, such as 0.67 for two runs of three.

Write no count of runs in `test.md`. `--runs` gives that count, and it is 1, thus the suite gives one measurement of each case. Raise it to measure the reliability of a case, and not to raise its score.

**Write the expectation for what the report of the agent shows.** A fact of the file system belongs to a grader. An expectation that names a file that the report does not name fails, because the judge reads the report and the commands, and it does not read the project. This fault is silent: the case fails and the skill is correct.

---

## 3. Write each expectation against what the run observed

Write "each package that the report of the agent named as a minor version was raised". Do not write "`league/csv` went to 9.28.0". The second sentence passes today and fails when the package has a new release. This rule is what lets a case run against a real project and a real registry.

---

## 4. The bases

`bases/<name>/seed.sh` writes one real project. The program runs that file once into `.cache/<name>`, then gives each skill a copy on write clone of it. A clone costs no disk and it costs less than one second, thus a case needs no base of its own.

Write a case against `catalogue`. That base holds every version of every dependency in `packages/`, and it holds the release notes of a version in the `CHANGELOG.md` file of that directory, thus the set of upgrades that a run can find is a fact of the base and not a fact of a registry. The run reaches no network, it takes seconds, and it gives one answer each time. The default of `allowed_tools` holds no `WebFetch` for that reason.

Write a case against `laravel` only to measure that the commands of the skill work against a real framework and a real registry. Keep the number of those cases at one: that base carries the newest version of each dependency, thus it drifts, the workload of a run is whatever the registry offers that week, and a case fails while the skill is correct. Pin a package backwards in the `setup.sh` of that case to give it an upgrade to find, and pin a package that the framework does not constrain, because the dependencies of a framework constrain each other and an old version of one of them does not resolve.

`seed.sh` writes no `.git`. The program runs `git init`, commits the base, then runs `setup.sh`. A `setup.sh` that wants its own change committed commits it, thus a case starts clean unless the case wants a change in the working tree.

The program hides its own files from git in `.git/info/exclude`. Without that step `.eval/` and `.claude/` read as a change in the working tree, and each case of `deps-upgrade` stops at step 1.

The program resets the project with a copy on write clone of the prepared project, and not with `git reset`. Git does not track `vendor/`, thus a `git reset` leaves the raised version of a package on the disk and the next case finds nothing to raise. This fault is silent.

---

## 5. The program

`cargo run --release -- --list` names each case and runs nothing. `cargo run --release --` runs each case of each skill, one run of each, and it runs eight skills at the same time. Give a skill to run one skill, and give `<skill>/<case>` to run one case.

The program stops at the first run that fails, and it writes the diagnosis of that run into `.runs/diagnosis/`. That file is the prompt of the agent that repairs the fault, thus give it to that agent and write no summary of it. Give `--no-bail` to run each case of the suite.

This crate holds its own `[workspace]`. Do not make it a member of the workspace of `cli/`: `cargo vet`, `cargo deny`, `cargo machete` and the MSRV check of `hod` then read the dependencies of this crate, and `cargo package` writes them into the crate that a user installs.

One skill is one unit of work. The program gives one skill one project for one base, runs each case of that skill in that project one after the other, and resets the project between two cases. `--jobs` gives the number of skills that run at the same time, and it is 8. The wall clock holds the install of the dependencies and the turns of the agent, thus the ceiling is memory and the rate limit of the API, and not the number of cores. Raise `--jobs` above 8 only after a run reports no fault of a rate limit.

A precondition is the text of `setup.sh` with the files of `fixtures/` and the arm. The program orders the cases of a skill by precondition and holds the prepared project of the precondition in hand, thus `setup.sh` runs one time for a group of cases that share one precondition, and not one time for each case.
