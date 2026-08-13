# TBaguette's Atelier

65 skills for [Claude Code](https://claude.com/claude-code) — judgment calls, code
comprehension, change discipline, testing, debugging, systems design, defensive
security, communication, and tooling. Every one is project-agnostic, stack-agnostic,
and language-agnostic, so the same skill works whether you're in a Rust firmware repo
or a Ruby monolith.

**[Browse them all → lesplooch.github.io/tbaguette-skills](https://lesplooch.github.io/tbaguette-skills/)**

That's the fastest way to see what's actually in here — searchable, organized by
category, every skill readable in full.

## Install

macOS, Linux, or Windows in WSL / Git Bash (bash, zsh, and fish all verified — see below):

```bash
[ -d ~/.claude/skills/TBaguette/.git ] && git -C ~/.claude/skills/TBaguette pull || git clone https://github.com/LeSplooch/tbaguette-skills.git ~/.claude/skills/TBaguette
```

Windows, native PowerShell (5.1 or 7+ — the default terminal since Windows 10):

```powershell
if (Test-Path "$HOME\.claude\skills\TBaguette\.git" -PathType Container) { git -C "$HOME\.claude\skills\TBaguette" pull } else { git clone https://github.com/LeSplooch/tbaguette-skills.git "$HOME\.claude\skills\TBaguette" }
```

Restart Claude Code (or run `/reload-plugins`). It loads as the `TBaguette@skills-dir`
plugin — invoke any skill directly (`TBaguette:formidable`, `TBaguette:knowing-when-to-stop`, ...)
or let them trigger automatically when your situation matches. Run the same command again
later to pull updates in place.

Both commands only ever touch `~/.claude/skills/TBaguette` — neither can alter, merge into,
or overwrite any other skill or plugin you already have. `git clone` refuses outright if that
exact path already exists and isn't empty or a clone of this repo, so a name collision
fails loudly instead of silently overwriting something. `scripts/test_install_command.py`
(stdlib-only Python, no bash required to *run* the test) proves this against four scenarios
(fresh install, re-run, an empty pre-existing directory, and a real collision), then
cross-checks the literal bash command above against every POSIX-ish shell it finds on the
build machine — bash, zsh, fish, and sh. All of it is part of `run_tests.py`, not just
asserted here. The PowerShell command isn't machine-tested the same way (no PowerShell
runtime in this project's build) — it's verified by careful construction against
documented `Test-Path`/`git` behavior instead, which the verify-install page is upfront
about rather than overclaiming. Read the full walkthrough, the exact test source, and a
Command Prompt equivalent too at
[lesplooch.github.io/tbaguette-skills/verify-install](https://lesplooch.github.io/tbaguette-skills/verify-install/).

## What's in it

- **`formidable`** — design craft for every UI stack: web, native mobile, desktop,
  terminal, CLI output, game HUD, embedded/e-ink, XR, email, print, voice, dense data.
- **Judgment and meta** — calibrating confidence, red-teaming your own work, knowing
  when to stop, and `karen-and-the-manager`, a persona-forced pass that refuses to be
  satisfied until it's found everything.
- **Reading code, landing changes, testing, debugging, designing systems, defensive
  security, communicating, environment and tooling** — 8 more categories, 56 more
  skills. Full breakdown in [`CATALOG.md`](CATALOG.md) or, better, on the site above.

## This repo is also the site's source

`docs/` is the generated static site (GitHub Pages serves it straight from there).
`skills/` is the actual plugin content. Edit a skill, then:

```bash
python3 scripts/generate.py --base-path /tbaguette-skills
```

regenerates the whole site — stdlib only, no install step. `python3 scripts/run_tests.py`
runs the full test suite (`python3 -m unittest discover` on its own misses two of the
three test files). Design rationale — the palette, the content schema, why it looks
like a bakery — is in
[`superpowers/specs/2026-08-13-tbaguette-showcase-design.md`](superpowers/specs/2026-08-13-tbaguette-showcase-design.md).

One-time setup after cloning, if you'll be committing:

```bash
git config core.hooksPath .githooks
```

This points git at the tracked pre-commit hook in [`.githooks/`](.githooks/pre-commit),
which regenerates the site before every commit, unconditionally — including a
CSS- or skills-only change, which is still "the site was updated." It exists because
that got missed manually once: a styles-only commit shipped without regenerating,
leaving the header's own "Updated" timestamp pointing at the previous commit instead of
itself. `core.hooksPath` is local to your clone (git only reads `.git/hooks` by
default), so this one command is what actually turns the hook on.
