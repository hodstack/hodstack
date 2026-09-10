# Hodstack Skills

This directory holds the skills, as text. Each skill obeys the Agent Skills format of the Agent Plugins standard, thus `hod` writes one tree into the project of the user and Claude Code, Codex, Cursor, Copilot, VS Code and other clients read it.

The `AGENTS.md` file at the top level gives the intention of the project and the rules for this file. This file gives the decisions for this directory: the layout, the rules, the start of a skill and its distribution.

---

## 1. The format of a skill: Agent Plugins 1.0.0

Obey the Agent Skills format of [Agent Plugins 1.0.0](https://github.com/agentplugins/agent-plugins-spec). Each skill is one directory with a file with the exact name `SKILL.md`. That file carries `name` and `description` in its front matter, and `name` agrees with the name of the directory. A test in `cli/src/skills.rs` reads each rule of this file about a skill that the binary carries, and `build.rs` stops the build for a directory in `skills/skills/` without a `SKILL.md`. Run `cargo test` in `cli/`.

Two rules control the tree:

1. **Keep each skill one level below `skills/`.** `hod` writes one flat directory into the project of the user, and the search of a directory of skills is not recursive in each client. The form `skills/<group>/<name>/` thus gives no group to the user.
2. **Put material for one client in the place that the client reads.** Codex reads `agents/openai.yaml` in the directory of the skill.

The hyphen carries the group. The command `hod pr-review` starts the skill in `skills/skills/pr-review/`. Refer to `cli/AGENTS.md`, section 1.

---

## 2. The tree

```text
<top level of the directory>
├── skills/                      # the skills — one level, no subgroups
│   └── init/
│       ├── SKILL.md
│       ├── agents/openai.yaml   # the Codex metadata of this skill
│       ├── references/*.md
│       └── scripts/*.sh
├── AGENTS.md                    # the rules
└── CLAUDE.md                    # one line: @AGENTS.md
```

Give the name of a skill the group first, such as `pr-review`, when the set needs groups by subject.

The interior directory also has the name `skills`, thus each path has this form: `skills/skills/init/`. The standard gives that name, and `build.rs` in `cli/` reads that path. Do not change it.

The files that `hod init` writes sit in `cli/templates/`, not in this directory. The crate `hod` reads them with `include_str!`, and `cargo package` writes a crate that does not build when a path leaves `cli/`. Refer to `cli/AGENTS.md`, section 2.

---

## 3. How a skill starts

There are two types of skill.

- A **user skill** controls a sequence of operations. The user selects it by its name.
- A **model skill** holds the rules of one subject. The agent selects it during its work. Section 4 gives its form.

A user skill can use a model skill. A user skill must not use a different user skill.

Write the sentence `Call the Skill tool with "<name>".` in a skill that uses a different skill. Each client gives the model a tool for a skill, and this sentence names that tool, thus the model calls it. Do not write `/<name>`: the model reads that form as text, and a slash command belongs to one client. Do not write the path of a file of a different skill: `hod` writes each skill in one flat directory, and a path that leaves the directory of the skill breaks.

Name one skill in one sentence. Two skills need two sentences, because the tool takes one skill in one call.

Put material that two skills read in the skill that owns it. The second skill calls the first skill, and it does not read a file of that skill.

Write no call to a user skill. No skill reaches a user skill, thus write an instruction for the user instead, such as "Tell the user to run `hod deps-upgrade`.".

The standard has no method to show this difference, and each client gives its own method. Give a user skill `disable-model-invocation: true` in the front matter of `SKILL.md`, for Claude Code. Give it `policy.allow_implicit_invocation: false` in `agents/openai.yaml`, for Codex. Write the two properties together: a skill is a user skill in the two clients or in none.

A different client receives a user skill as a model skill, thus **write the `description` of each user skill for two conditions.** The first condition is a request from the user. The second condition is a selection by a model. A skill must not depend on a request from a user for its correct operation.

With `hod`, the user can start each skill directly. The program is the start method that the standard does not have.

---

## 4. How to write the text of a skill

A coding agent reads each file of a skill. Write an instruction that the agent obeys during its work, then stop. Obey section 1 and section 2 of the `AGENTS.md` file at the top level: the imperative, the active voice, one instruction in one sentence, and the exact path, command and name.

Delete a sentence that says the name of the skill again. Delete a sentence that gives background, such as the history of a tool or the reason that the project made a decision. Delete a sentence that a different file holds, and give the path of that file instead. Give a reason only when the reason changes the next decision of the agent.

Give each model skill one form. The test `a_model_skill_holds_its_rules_in_one_form` in `cli/src/skills.rs` reads that form, and `cargo test` in `cli/` runs it. Start the `description` with `The rules for <subject>.`, then write `Use when` and each decision of the agent that the rules control, because the model selects the skill by that text alone. Start the body with `Obey each rule of this file each time that you <write the subject> and each time that you <change the subject>.`. Give one rule one numbered section, and write the heading as the rule, in the imperative. Name the last section `Report`, and write there each fact that the agent names in its last message, because the judge of an eval case reads that message and not the project. End the body with the two sentences that the test names. Write an eval case in `evals/<skill>/` for each rule that you add, thus a change to the text that breaks that rule fails a test. Refer to `evals/AGENTS.md`.

---

## 5. Distribution

`build.rs` in `cli/` reads this directory and writes each skill into the binary. A user receives a skill with `hod update`, and `hod` writes it into `.claude/skills/` and `.agents/skills/` of the project. Refer to `cli/AGENTS.md`, section 2.
