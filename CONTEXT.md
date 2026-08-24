# Hodstack Context

## 1. The terms

**coding agent**:
The program that `hod` starts with the opening prompt. The table `AGENTS` in `cli/src/agent.rs` names five: `claude`, `codex`, `cursor-agent`, `opencode` and `gemini`. `hod` takes the first one on `PATH`, and the variable `HOD_AGENT` names one in its place.
_Avoid_: agent, AI, assistant, model, client.

**model**:
The reader that obeys the text of an `AGENTS.md` file, of a rule and of a skill. A model reads inside a coding agent, thus a model selects a model skill and a user selects a user skill.
_Avoid_: agent, coding agent, AI, LLM, assistant.

**client**:
A program that reads a skill from a skill directory: Claude Code, Codex, Cursor, Copilot and VS Code. The set of the clients and the set of the coding agents share three members and are not equal: Copilot reads a skill and `hod` cannot start it, and `hod` starts `opencode` and no file of this repository names `opencode` a client.
_Avoid_: coding agent, agent, host, editor, tool.

**skill directory**:
One of the two paths that `hod` writes each installed skill into: `.claude/skills` and `.agents/skills`. The constant `CLIENTS` in `cli/src/project.rs` holds the two.
_Avoid_: client, client directory, target, output directory.

**skill**:
One directory that holds a file with the exact name `SKILL.md`, and that file carries `name` and `description` in its front matter. The type `Skill` in `cli/src/skills.rs` holds the name of that directory and the text of each file below it.
_Avoid_: command, slash command, plugin, prompt.

**user skill**:
A skill that the user selects by its name. Its `SKILL.md` carries `disable-model-invocation: true` and its `agents/openai.yaml` carries `policy.allow_implicit_invocation: false`, thus the field `user` of `Front` in `cli/src/front.rs` is true and `hod list` prints the skill under `USER SKILLS`.
_Avoid_: manual skill, explicit skill, command.

**model skill**:
A skill that the model selects during its work. `hod list` prints it under `MODEL SKILLS`.
_Avoid_: automatic skill, implicit skill, behavior.

**shipped skill**:
A skill that the binary carries. `build.rs` reads the skills tree and writes the table `SKILLS`, and `shipped()` in `cli/src/skills.rs` reads that table.
_Avoid_: built-in skill, bundled skill, default skill, embedded skill.

**project skill**:
A skill in `.hod/skills/` of one project. `local()` in `cli/src/skills.rs` reads it, and `hod list` prints it under `PROJECT SKILLS`.
_Avoid_: local skill, custom skill, own skill.

**installed skill**:
A member of the set that `Project::installed()` in `cli/src/project.rs` gives: each shipped skill, with a project skill of the same name in its place, and each other project skill after them. `hod <skill>` and `sync` read this set.
_Avoid_: available skill, merged skill, resolved skill.

**vendored skill**:
A skill of a different supplier under `cli/.agents/skills/`, with its source and its hash in `cli/skills-lock.json`. This repository reads a vendored skill for its own work, and `hod` neither writes it nor lists it.
_Avoid_: installed skill, project skill, dependency.

**skills tree**:
The directory `skills/skills/` of this repository, with one directory for one skill and no directory between them. It is the source of each shipped skill, and the two skill directories of a project are the destination.
_Avoid_: skill library, catalog, registry, skills folder.

**project**:
A directory that holds `.hod`. The type `Project` in `cli/src/project.rs` holds its root, and `Project::exists()` reads `.hod`.
_Avoid_: workspace, repository, root.

**project file**:
A file that `hod` writes and that the lock records: `AGENTS.md`, `CLAUDE.md`, and each file of each installed skill below each skill directory. `hod update --project` writes this set.
_Avoid_: generated file, managed file, template, artifact.

**intention**:
What the project does and who uses it. The `AGENTS.md` file of this repository holds the intention of Hodstack, and `.hod/PROJECT.md` holds the intention of a user project. `hod` writes the text of `cli/templates/PROJECT.md` into `.hod/PROJECT.md` one time, when the file is absent, and the lock records no sum for it, thus that file belongs to the user after the write.
_Avoid_: description, readme, configuration, context, goal.

**rule**:
One instruction that a project holds for the next task, with the fault that the instruction prevents. A rule of this repository sits in an `AGENTS.md` file, and a rule of a user project sits in a rule file.
_Avoid_: guideline, convention, memory, preference, lesson.

**rule file**:
The file `.hod/rules/<name>.md` of one project, which holds one rule and carries `name` and `description` in its front matter. A model writes it, the type `Rule` in `cli/src/project.rs` reads it, and `agents()` in `cli/src/project.rs` writes one row for it in section 6 of the `AGENTS.md` file of that project.
_Avoid_: rule, note file, memory file.

**lock**:
The file `.hod/lock`, with one line for one project file: the SHA-256 sum of the text that `hod` wrote, then two spaces, then the path. `Lock` in `cli/src/lock.rs` reads it and writes it.
_Avoid_: manifest, index, cache, state file.

**ownership**:
The answer that the lock gives for one path, as `Owner` in `cli/src/lock.rs`: `Absent` for a file that is not there, `Ours` when the sum of the file agrees with the lock, and `Theirs` for each other file. `hod` writes over `Ours`, reports `Skipped` for `Theirs`, and `hod update --force` writes over `Theirs`.
_Avoid_: dirty, modified, conflict, drift.

**sync**:
The pass in `cli/src/sync.rs` that writes each project file, removes each file that the lock holds and the plan of the run does not, and adds two lines to `.gitignore`. `hod init`, `hod update` and `hod update --project` run it.
_Avoid_: update, install, generate, scaffold.

**build**:
One compilation of `hod` that a release carries, named by its commit. The job `build` gives the variable `HOD_COMMIT`, `version.txt` holds the version of the crate and that commit in one line, and `Build` in `cli/src/update.rs` reads that file.
_Avoid_: version, release, edge.

**release**:
One GitHub release of the branch `0.x`, with the tag `edge-<date>-<time>` in UTC and the mark `latest`. One push gives one release, and the version of the crate does not change with it.
_Avoid_: version, tag, edge, nightly.

**opening prompt**:
The text `/<skill>` that `hod` gives the coding agent as its first argument. The enum `Opening` in `cli/src/agent.rs` names the form of that argument: positional, or after a flag.
_Avoid_: command, slash command, first message, opening.

---

## 2. The relationships

- A release carries one build, and that build carries each shipped skill of the skills tree of its commit.
- A project holds its intention in `.hod/PROJECT.md`, each rule file, each project skill and the lock.
- An installed skill is a shipped skill, or a project skill of the same name in its place, or a project skill with a name that no shipped skill has.
- `hod` writes each installed skill into each skill directory of the project, and each client reads one skill directory.
- A skill is a user skill or a model skill, and the property in `SKILL.md` and the property in `agents/openai.yaml` agree.
- A rule file holds one rule, and the `AGENTS.md` file of a project holds one row for each rule file.
- The lock gives the ownership of each project file. `.hod/PROJECT.md` and `.gitignore` carry no line in the lock, thus `hod` writes each of the two one time.
- `hod <skill>` gives the opening prompt to one coding agent, and the model of that coding agent reads the skill.

---

## 3. Flagged ambiguities

- "agent" did four jobs: the program that `hod` starts, the reader of a file, the file `AGENTS.md`, and the directory `.agents/`. Resolved: **coding agent** for the program, **model** for the reader, `AGENTS.md` for the file, and **skill directory** for `.agents/skills`.
- "coding agent" did two of those jobs: the program, and the reader. Resolved: the reader is the **model**. Three sentences name the wrong sense: `AGENTS.md`, section 1, "Each `AGENTS.md` file in this repository is for a coding agent"; `skills/AGENTS.md`, section 4, "A coding agent reads each file of a skill"; `cli/templates/rules.md`, section 1, "This file is for a coding agent".
- `AGENTS.md` names two files: the file that a person writes in this repository, and the file that `hod` writes in a project from `cli/templates/rules.md`. Resolved: "the `AGENTS.md` file of this repository" for the first, "the `AGENTS.md` file of the project" for the second, and the second is a **project file**.
- "intention" names what a project does, and the constant `INTENTION` in `cli/src/project.rs` names the path `.hod/PROJECT.md`. Resolved: **intention** for the text, `.hod/PROJECT.md` for the file.
- "rule" did two jobs: the instruction, and the file that holds one instruction. Resolved: **rule** for the instruction, **rule file** for `.hod/rules/<name>.md`. `cli/templates/rules.md`, section 2, already writes "rule file".
- "client" did two jobs: the program that reads a skill, and the path that `hod` writes a skill into. Resolved: **client** for the program, **skill directory** for the path. The constant `CLIENTS` in `cli/src/project.rs` names two paths, not two clients.
- A skill in `.hod/skills/` carried three names: `local` in `cli/src/skills.rs`, `mine` in `cli/src/list.rs`, and `PROJECT SKILLS` on the screen of `hod list`. Resolved: **project skill**.
- `.agents/skills/` holds two sets with no relation: each installed skill of a user project, and the skill of a different supplier under `cli/.agents/skills/`. Resolved: **skill directory** for the first, **vendored skill** for the second.
- "update" did two jobs: the installation of a newer build, and the write of each project file. Resolved: **update** for the installation, **sync** for the write. `hod update` does the two, and `hod update --project` does the write alone.
- "version" and "build" named one value: the version in `cli/Cargo.toml` stays for many pushes, and the commit names one compilation. Resolved: **version** for the number in the manifest, **build** for the commit.
- "opening" did two jobs in `cli/src/agent.rs`: the enum `Opening` names the form of the argument, and the parameter `opening` holds the text of it. Resolved: **opening prompt** for the text.
