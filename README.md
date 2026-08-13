# TBaguette

65 skills for [Claude Code](https://claude.com/claude-code) — judgment calls, code
comprehension, change discipline, testing, debugging, systems design, defensive
security, communication, and tooling. Every one is project-agnostic, stack-agnostic,
and language-agnostic, so the same skill works whether you're in a Rust firmware repo
or a Ruby monolith.

**[Browse them all → lesplooch.github.io/tbaguette-skills](https://lesplooch.github.io/tbaguette-skills/)**

That's the fastest way to see what's actually in here — searchable, organized by
category, every skill readable in full.

## Install

```bash
[ -d ~/.claude/skills/TBaguette/.git ] && git -C ~/.claude/skills/TBaguette pull || git clone https://github.com/LeSplooch/tbaguette-skills.git ~/.claude/skills/TBaguette
```

Restart Claude Code (or run `/reload-plugins`). It loads as the `TBaguette@skills-dir`
plugin — invoke any skill directly (`TBaguette:formidable`, `TBaguette:knowing-when-to-stop`, ...)
or let them trigger automatically when your situation matches. Run the same command again
later to pull updates in place.

This only ever touches `~/.claude/skills/TBaguette` — it cannot alter, merge into, or
overwrite any other skill or plugin you already have. `git clone` refuses outright if that
exact path already exists and isn't empty or a clone of this repo, so a name collision
fails loudly instead of silently overwriting something. `scripts/test_install_command.sh`
proves this against four scenarios (fresh install, re-run, an empty pre-existing directory,
and a real collision) — it's part of `run_tests.py`, not just asserted here.

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
