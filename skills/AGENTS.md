# Hodstack Skills

This directory holds the skills, as text. It uses the Agent Plugins standard, thus one tree supplies Claude Code, Codex, Cursor, Copilot, VS Code and other clients.

The `AGENTS.md` file at the top level gives the intention of the project and the rules for this file. This file gives the decisions for this directory: the layout, the rules, the set of skills, the package formats and the steps to add a skill.

---

## 1. The package format: Agent Plugins 1.0.0

Obey [Agent Plugins 1.0.0](https://github.com/agentplugins/agent-plugins-spec). ChatGPT, Codex, Cursor, GitHub Copilot, Kiro and VS Code read this standard. Claude Code reads its own format.

Two rules control the tree:

1. **The search of `skills/` is not recursive.** Each immediate subdirectory that contains a file with the exact name `SKILL.md` is one skill. A client that obeys the standard cannot see a directory below that level.
2. **Put material for one client in a namespace with a reverse domain name.** Use `extensions["com.example.client"]` in `plugin.json`, or a `com.example.client/` directory at the top level.

Rule 1 prevents the form `skills/<group>/<name>/`. Keep each skill one level below `skills/`.

Before you change the layout, test these three statements against the text of the standard: the search of `skills/` is not recursive; material for one client goes in a namespace with a reverse domain name; the skill contract does not carry `disable-model-invocation`.

---

## 2. The tree

```
<top level of the directory>
├── plugin.json                  # Agent Plugins manifest ($schema, name: "hodstack", version,
│                                #   description, author, homepage, repository, license,
│                                #   keywords, extensions)
├── skills/                      # the skills — one level, no subgroups
│   └── pr-review/
│       ├── SKILL.md
│       ├── references/*.md
│       └── scripts/*.sh
├── templates/                   # the files that `hod init` writes
│   ├── AGENTS.md
│   └── CLAUDE.md
├── .claude-plugin/plugin.json   # the Claude Code channel (a list of the skills)
├── .agents/                     # writing-skills.md, invocation-model.md
├── AGENTS.md                    # the rules
├── CLAUDE.md                    # one line: @AGENTS.md
├── CONTEXT.md                   # the terms for this directory
├── README.md
└── LICENSE
```

Do not make a directory for a subject. If the set needs groups by subject, put them in `keywords`, in the README, or in the name of the skill. Do not put them in the path.

The interior directory also has the name `skills`, thus each path has this form: `skills/skills/pr-review/`. The standard makes the interior name necessary. Do not change it.

The directory `templates` holds the two files that `hod init` writes into the directory of a user. Write no fact about Hodstack in them: they must work in any project. The crate `hod` reads them with `include_str!`, thus a change to one of them takes effect on the next build of the program.

---

## 3. How a skill starts

There are two types of skill.

- A **user skill** controls a sequence of operations. The user selects it by its name.
- A **model skill** holds one part of the discipline. The agent selects it during its work. Write many words in its description that show when to use it.

A user skill can use a model skill. A user skill must not use a different user skill.

The standard has no method to show this difference. The property `disable-model-invocation: true` is a Claude Code property, and the skill contract in the standard (refer to [agentskills.io](https://agentskills.io/specification)) does not carry it, thus a different client receives a user skill as a model skill.

Therefore: **write the `description` of each user skill for two conditions.** The first condition is a request from the user. The second condition is a selection by a model. A skill must not depend on a request from a user for its correct operation. The file `.agents/writing-skills.md` holds this rule.

With `hod`, the user can start each skill directly. The program is the start method that the standard does not have.

---

## 4. How to write the text of a skill

A coding agent reads each file of a skill. Write an instruction that the agent obeys during its work, then stop. Obey section 1 and section 2 of the `AGENTS.md` file at the top level: the imperative, the active voice, one instruction in one sentence, and the exact path, command and name.

Delete a sentence that says the name of the skill again. Delete a sentence that gives background, such as the history of a tool or the reason that the project made a decision. Delete a sentence that a different file holds, and give the path of that file instead. Give a reason only when the reason changes the next decision of the agent.

---

## 5. The rules for the manifest

- Give `name` a value that agrees with `[a-z0-9.-]+`. Use 1 to 64 characters. Start and end the value with a letter or a number. Do not put two `-` characters together or two `.` characters together. The name `hodstack` obeys these rules.
- The schema of `plugin.json` is closed (`additionalProperties: false`). Use these properties only: `$schema`, `name`, `version`, `description`, `author`, `homepage`, `repository`, `license`, `keywords` and `extensions`. Put all other data in `extensions.<reverse domain name>`. Only `$schema` and `name` are necessary, thus a minimum manifest has two lines.
- Start each path in the configuration with `./`. Keep each path in the top level of the plugin. Do not use a symbolic link that goes out of it.
- Use a semantic version number in `version`. The clients use it to find a new version.
- The `mcp.json` file is optional. Do not add it until there is a reason.

---

## 6. Distribution

TBD.
