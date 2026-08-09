# Hodstack

Hodstack makes coding agents more productive. It has two parts: a set of skills and a cli.

The work happens in one repository: `hodstack/hodstack`. It holds two directories:

- **`skills`** — the skills, as text. The directory uses the Agent Plugins standard. One tree supplies Claude Code, Codex, Cursor, Copilot, VS Code and other clients. For the layout, the rules and the package formats, refer to `skills/AGENTS.md`.
- **`cli`** — the `hod` program. The command `hod <skill> <prompt>` starts a coding agent with that skill and that prompt. For the rules, refer to `cli/AGENTS.md`.

One repository, because the two parts change together. A skill and the command that runs it are one change and one tag.

## Specification

These rules apply to this file and to each `AGENTS.md` in a directory. The skills obey the rules in `skills/AGENTS.md`.

Write in [ASD-STE100](https://www.asd-ste100.org) Simplified Technical English: one instruction in one sentence, the imperative, the active voice, one meaning for one word, no contraction, and no synonym for variety. The standard holds the full rules. Do not copy them here. Four items keep their exact form: a quotation from a standard, an identifier in the code, a path or a command, and a name from a different supplier.

Write GitHub flavored Markdown. Put one `#` heading in a file. Write one paragraph on one line, thus a change stays small in `git diff`. Number the sections of a long file, thus a different file can point to "section 3". Put a path, a command and a name from the code in `code font`.

Each task teaches you a fact that these files do not hold. Write the fact in the same change, or the session ends and you lose it. A correction from the user is a rule: put it in a file before you continue the work. The intention of the project goes in this file. A decision about one directory goes in the `AGENTS.md` of that directory. Replace an old rule. Do not add a second rule near it. Delete a rule that the project does not obey. Do not keep a record of what the project stopped doing, in a file or in a directory. Git holds the history.

## Marketing

These rules control the README, the website, the release notes, the announcement and the description in each manifest. The specification above controls the files that the project writes for itself. It does not control the text that the project writes for the public.

The voice is calm and certain. Say what the project does, then stop. Do not say that it is fast, simple, powerful or intelligent. Show the command and let the reader form that opinion. Cut each superlative. Cut each sentence that a competitor can also write about itself.

Write for one person and call that person "you". Put the result first and the mechanism second. Keep the sentences short, and let one sentence stand alone when it carries the idea. A demonstration is the strongest argument: one command in a terminal is worth one paragraph of adjectives.

The care is in the small parts. The alignment of the output, the words in an error message, the space around the text on the page: the reader feels this work before the reader reads one sentence. The project is for people who enjoy their work, thus the text is warm. A dry joke is permitted one time on a page. Do not explain it.

Three limits protect the trust of the reader. Do not promise a feature that does not exist today. Do not give a number without its source. Do not name a different product to make a comparison.
