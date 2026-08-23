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

## Your project

`hod init` writes `AGENTS.md`, and `hod` keeps writing it. Your own words go in `.hod/`:

```
AGENTS.md            hod writes this
.hod/lock            hod writes this
.hod/PROJECT.md      what this project is
.hod/rules/*.md      one rule per file, listed in AGENTS.md
.hod/skills/*/       your own skills
```

Ask your agent to run `/learn` when it gets something wrong. It writes the rule, and the next session reads it.

## Your skills

Run a skill by its name:

```sh
hod deps-upgrade
```

`hod` opens the coding agent it finds on your PATH — `claude`, `codex`, `cursor-agent`, `opencode` or `gemini` — and hands it `/deps-upgrade`. Set `HOD_AGENT` to name a different one.

`hod list` names every skill you have.

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
