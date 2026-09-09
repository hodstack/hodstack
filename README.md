# Hodstack

Hodstack makes coding agents more productive.

## Installation

On macOS and Linux:

```sh
curl -fsSL https://github.com/hodstack/hodstack/releases/latest/download/install.sh | sh
```

On Windows:

```powershell
irm https://github.com/hodstack/hodstack/releases/latest/download/install.ps1 | iex
```

With npm:

```sh
npm install --global hodstack@edge
```

Then set up your project:

```sh
hod init
```

`hod init` writes the files, then opens your coding agent to fill in `.hod/PROJECT.md`.

## Your project

`hod init` writes `AGENTS.md`, and `hod` keeps writing it. Your own words go in `.hod/`:

```
AGENTS.md            hod writes this
.hod/lock            hod writes this
.hod/PROJECT.md      what this project is
.hod/rules/*.md      one rule per file, listed in AGENTS.md
.hod/skills/*/       your own skills
```

Tell your agent when it gets something wrong. It writes the rule in `.hod/rules/`, and the next session reads it.

Those files quote paths and names from your code. Rename one and the sentence stays behind:

```sh
hod check
```

`hod check` reads every path and every name you wrote in code font, and reports the ones your project no longer holds. It exits non-zero, so CI can run it.

## Your skills

Run a skill by its name:

```sh
hod deps-upgrade
```

`hod` opens the coding agent it finds on your PATH — `claude`, `codex`, `cursor-agent`, `opencode` or `gemini` — and hands it `/deps-upgrade`. Set `HOD_AGENT` to name a different one.

`hod list` names every skill you have.

## Your worktrees

Give each piece of work its own branch and its own directory:

```sh
hod worktree:create fix-help
```

`hod worktree:create` adds a linked worktree in `~/.hod/worktrees/<project>/`, with the branch you name. Leave the name out and it names the branch for you, such as `quiet-harbor`. Its last line is a `cd` into that directory.

When the work is done, fold it in from that worktree:

```sh
hod worktree:merge "fix: help"
```

`hod worktree:merge` merges the branch of the worktree you stand in into the branch of your main checkout, then removes the worktree and the branch. Leave the message out and it asks you for one. Press enter to keep the message of git. It stops before the merge when either checkout has uncommitted changes. When the two branches conflict, it undoes the merge, names each conflicted file, and keeps the worktree. Pass `--keep` to keep the worktree and the branch.

Its last line is a `cd` into your main checkout, because the directory you stood in is gone.

## Updates

`hod` tells you when a newer build exists. One command installs it, whichever way you installed `hod`, and writes the files it owns again:

```sh
hod update
```

Your files stay yours. `hod` writes over a file only when the file is still the one it wrote, and it tells you which files it kept.

Set `HOD_NO_UPDATE_CHECK=1` to keep the notice quiet.

## Sponsors

We cannot thank our sponsors enough for their incredible support in funding Hodstack's development. Their contributions have been instrumental in making Hodstack the best it can be. For those who are interested in becoming a sponsor, please visit Nuno Maduro's Sponsor page at **[github.com/sponsors/nunomaduro](https://github.com/sponsors/nunomaduro)**.

- **[CMS Max](https://cmsmax.com/?ref=pestphp)**
- **[PhpStorm](https://jb.gg/nuno)**
- **[CodeRabbit](https://coderabbit.link/nunomaduro)**
- **[SerpApi](https://serpapi.com/?ref=nunomaduro)**
- **[Typesense](https://typesense.org/?ref=nunomaduro)**
- **[Bento](https://bentonow.com/?ref=nunomaduro)**
- **[Pixel](https://wearepixel.com.au/?ref=nunomaduro)**
- **[Redberry](https://redberry.international/laravel-development/?ref=nunomaduro)**

Hodstack is an open-sourced software licensed under the **[MIT license](https://opensource.org/licenses/MIT)**.
