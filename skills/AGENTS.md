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

```
<top level of the directory>
├── skills/                      # the skills — one level, no subgroups
│   └── learn/
│       ├── SKILL.md
│       ├── agents/openai.yaml   # the Codex metadata of this skill
│       ├── references/*.md
│       └── scripts/*.sh
├── AGENTS.md                    # the rules
└── CLAUDE.md                    # one line: @AGENTS.md
```

Give the name of a skill the group first, such as `pr-review`, when the set needs groups by subject.

The interior directory also has the name `skills`, thus each path has this form: `skills/skills/learn/`. The standard gives that name, and `build.rs` in `cli/` reads that path. Do not change it.

The files that `hod init` writes sit in `cli/templates/`, not in this directory. The crate `hod` reads them with `include_str!`, and `cargo package` writes a crate that does not build when a path leaves `cli/`. Refer to `cli/AGENTS.md`, section 2.

---

## 3. How a skill starts

There are two types of skill.

- A **user skill** controls a sequence of operations. The user selects it by its name.
- A **model skill** holds one part of the discipline. The agent selects it during its work. Write many words in its description that show when to use it.

A user skill can use a model skill. A user skill must not use a different user skill.

The standard has no method to show this difference, and each client gives its own method. Give a user skill `disable-model-invocation: true` in the front matter of `SKILL.md`, for Claude Code. Give it `policy.allow_implicit_invocation: false` in `agents/openai.yaml`, for Codex. Write the two properties together: a skill is a user skill in the two clients or in none.

A different client receives a user skill as a model skill, thus **write the `description` of each user skill for two conditions.** The first condition is a request from the user. The second condition is a selection by a model. A skill must not depend on a request from a user for its correct operation.

With `hod`, the user can start each skill directly. The program is the start method that the standard does not have.

---

## 4. How to write the text of a skill

A coding agent reads each file of a skill. Write an instruction that the agent obeys during its work, then stop. Obey section 1 and section 2 of the `AGENTS.md` file at the top level: the imperative, the active voice, one instruction in one sentence, and the exact path, command and name.

Delete a sentence that says the name of the skill again. Delete a sentence that gives background, such as the history of a tool or the reason that the project made a decision. Delete a sentence that a different file holds, and give the path of that file instead. Give a reason only when the reason changes the next decision of the agent.

---

## 5. Distribution

`build.rs` in `cli/` reads this directory and writes each skill into the binary. A user receives a skill with `hod update`, and `hod` writes it into `.claude/skills/` and `.agents/skills/` of the project. Refer to `cli/AGENTS.md`, section 2.
