# TBaguette's Atelier

93 skills for [Claude Code](https://claude.com/claude-code) — judgment calls, code
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

New to it? **[Getting started](https://lesplooch.github.io/tbaguette-skills/getting-started/)**
is the first session in the order you actually hit it: the reload, why the conversation you
installed from is the one that won't pick it up, how to prove it landed, six skills worth
trying on purpose, and what to check when nothing seems to be happening.

This installs for **Claude Code** specifically. The general Claude Desktop app and claude.ai
chat don't read `~/.claude/skills/` at all — they load whatever skills are enabled on your
claude.ai account instead, synced separately. Cloning this repo won't make these skills show
up there; enable them from **Customize** in the Desktop app sidebar or the skills settings on
claude.ai instead.

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

### Other agents

The commands above are the Claude Code install. The Atelier isn't Claude Code-only,
though: this same repo ships an integration for Codex, Cursor, Copilot CLI, Devin,
Gemini CLI, Hermes, Kimi Code, OpenCode, and Pi — see [`PORTING.md`](PORTING.md)
for what each one loads and how, plus [`README.opencode.md`](README.opencode.md)
and [`README.kimi.md`](README.kimi.md) for the two with enough surface to need
their own page.

Those installs differ in kind, not just in path — a plugin command here, an
extension install there, a line in a config file somewhere else — so rather than
ten recipes, the [site's install box](https://lesplooch.github.io/tbaguette-skills/)
carries one prompt you paste into whichever agent you use. It works out which
harness it's in and takes the matching route: clone-or-pull into Claude Code's
skills directory, hand off to the harness's own install command, show you the
config line and ask before touching it, or — when it can't identify a directory
its harness actually reads — stop and ask, rather than inventing a path. Only the
Claude Code path is one this repo can machine-verify, which is exactly why the
prompt is written to discover the rest instead of asserting them.

## What's in it

- **`formidable`** — design craft for every UI stack: web, native mobile, desktop,
  terminal, CLI output, game HUD, embedded/e-ink, XR, email, print, voice, dense data.
- **Judgment and meta** — calibrating confidence, red-teaming your own work, knowing
  when to stop, and `karen-and-the-manager`, a persona-forced pass that refuses to be
  satisfied until it's found everything.
- **Reading code, landing changes, testing, debugging, designing systems, defensive
  security, communicating, environment and tooling** — 8 more categories, 75 more
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

No setup step after cloning. `python3 scripts/run_tests.py` wires the pre-commit
hook itself the first time you run it, and says so when it does.

The hook lives in [`.githooks/`](.githooks/pre-commit) and regenerates the site before
every commit, unconditionally — including a CSS- or skills-only change, which is still
"the site was updated." It exists because that got missed manually once: a styles-only
commit shipped without regenerating, leaving the header's own "Updated" timestamp
pointing at the previous commit instead of itself.

It needs wiring because git will not read it otherwise: hooks come from `.git/hooks` by
default, and `core.hooksPath` — the setting that redirects it — is local to each clone
and cannot be committed. That is deliberate on git's part, since a repository able to
install its own hooks is a repository that runs code on `git clone`. Nothing can change
that; what [`scripts/githooks.py`](scripts/githooks.py) does is make the wiring happen
the first time you run anything here, rather than the first time somebody remembers.
It never overwrites a `core.hooksPath` you set yourself, and to do it by hand instead:

```bash
git config core.hooksPath .githooks
```

## Licence

TBaguette's Atelier is free software under the
[GNU General Public License, version 2](LICENSE). Copyright © 2026 LeSplooch.

That choice is load-bearing rather than decorative. The library is only as good as
what it absorbs from real work, so the terms are the ones that keep improvements
flowing back instead of accumulating privately in a hundred separate installs. If you
change a skill and pass that changed copy on to anyone, GPLv2 requires you to pass on
the source under these same terms.

Upstream contribution is a step further than the licence compels, and it is how this
repo expects to be used anyway: the `tending-tbaguette` skill turns a lesson learned
while using the Atelier into a pull request here, so it reaches everyone else the next
time `keeping-tbaguette-current` runs.

This program is distributed in the hope that it will be useful, but WITHOUT ANY
WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A
PARTICULAR PURPOSE. See the GNU General Public License for more details.
