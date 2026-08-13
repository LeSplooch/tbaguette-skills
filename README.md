# La Boulangerie TBaguette

A static showcase-and-reference site for the [TBaguette](https://github.com) skills
plugin — 64 project-, stack-, and language-agnostic Claude Code skills, plus
`formidable`, a design-craft skill covering every UI stack.

This site doubles as a live test of the skills it showcases: it was built by agents
instructed to actually use `TBaguette:formidable`, `TBaguette:designing-apis`,
`TBaguette:naming-things`, `TBaguette:handling-untrusted-input`, and others while
doing the work, not just describe them.

## Why it looks like a bakery

"Sumptuous" was the brief. This is the "well made" reading of it: a bakery concept
carried through brand, palette, and type — not through puns on every category name or
literal bread clip-art. See `docs/superpowers/specs/2026-08-13-tbaguette-showcase-design.md`
for the full design rationale, locked palette/type values, and content schema.

## Structure

```
index.html              generated — landing page
skills/<slug>/index.html generated — one page per skill, 64 of them
assets/styles.css        hand-authored — the whole design system
assets/site.js           hand-authored — search/filter, theme toggle, tabs
assets/icons.svg         hand-authored — line-mark sprite
scripts/content_pipeline.py  parses ~/.claude/skills/TBaguette/skills/*/SKILL.md → content.json
scripts/templates.py         renders content.json → the HTML pages
scripts/generate.py          orchestrator: runs both, writes the site
```

`index.html` and everything under `skills/` are generated output — each carries a
"do not hand-edit" header comment. The actual source of truth is the skill files
themselves, under `~/.claude/skills/TBaguette/`.

## Regenerating

```bash
python3 scripts/generate.py
```

Stdlib only — no `pip install`, no npm, no build tool. Re-run this after editing any
TBaguette skill to pick up the change.

## Viewing

```bash
python3 -m http.server 8000
```

then open `http://localhost:8000/`. Root-relative asset paths mean it needs to be
served, not opened directly via `file://`.
