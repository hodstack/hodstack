# Hodstack

Hodstack makes coding agents more productive. It has two parts: a set of skills and a cli.

The work happens in one repository: `hodstack/hodstack`. It holds two directories:

- **`skills`** — the skills, as text. The directory uses the Agent Plugins standard. One tree supplies Claude Code, Codex, Cursor, Copilot, VS Code and other clients. For the layout, the rules and the package formats, refer to `skills/AGENTS.md`.
- **`cli`** — the `hod` program. The command `hod <skill> <prompt>` starts a coding agent with that skill and that prompt. For the rules, refer to `cli/AGENTS.md`.

Change the skill and the command that runs it in one commit and release them under one tag.

---

## 1. The reader of an `AGENTS.md` file

Each `AGENTS.md` file in this repository is for a coding agent. No user reads it. Write an instruction that an agent obeys during its work, then stop. Do not write an introduction, a conclusion, an argument for a decision that the project made, or a sentence that no agent can obey. Give a reason only when the reason changes the next decision of the agent.

Start each rule with a verb in the imperative. Put the condition before the instruction. Give the exact path, the exact command and the exact name that the agent must use. If a rule needs a test, give the command that does the test.

The public files are different. `README.md`, the website, the release notes and the `description` in each manifest are for a user. Section 3 controls them.

---

## 2. How to write an `AGENTS.md` file

Write in [ASD-STE100](https://www.asd-ste100.org) Simplified Technical English: one instruction in one sentence, the imperative, the active voice, one meaning for one word, no contraction, and no synonym for variety. The standard holds the full rules. Do not copy them here. Four items keep their exact form: a quotation from a standard, an identifier in the code, a path or a command, and a name from a different supplier.

Write GitHub flavored Markdown. Put one `#` heading in a file. Write one paragraph on one line, thus a change stays small in `git diff`. Number the sections of a long file, thus a different file can point to "section 3". Put a path, a command and a name from the code in `code font`.

Write the intention of the project in this file. Write a decision about one directory in the `AGENTS.md` file of that directory. If this file and the `AGENTS.md` file of a directory disagree, obey this file and correct the other file.

Each task teaches you a fact that these files do not hold. Write the fact in the same change, or the session ends and you lose it. A correction from the user is a rule: write it in a file before you continue the work. Replace an old rule. Do not add a second rule near it. Delete a rule that the project does not obey. Do not keep a record of what the project stopped doing, in a file or in a directory. Git holds the history.

---

## 3. How to write the public text

These rules control `README.md`, the website, the release notes, the announcement and the `description` in each manifest. Sections 1 and 2 do not control these files.

The voice is calm and certain. Say what the project does, then stop. Do not say that the project is fast, simple, powerful or intelligent. Show the command and let the reader form that opinion. Cut each superlative. Cut each sentence that a competitor can also write about itself.

Write for one person and call that person "you". Put the result first and the mechanism second. Keep the sentences short, and let one sentence stand alone when it carries the idea. A demonstration is the strongest argument: one command in a terminal is worth one paragraph of adjectives.

The care is in the small parts. The alignment of the output, the words in an error message, the space around the text on the page: the reader feels this work before the reader reads one sentence. The project is for people who enjoy their work, thus the text is warm. Put one dry joke on one page, at the most. Do not explain it.

Obey three limits. Do not promise a feature that does not exist today. Do not give a number without its source. Do not name a different product to make a comparison.
