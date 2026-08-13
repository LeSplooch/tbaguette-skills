# TBaguette showcase site — design

## Purpose

A static showcase-and-reference site for the 64 skills in the `TBaguette` plugin
(`~/.claude/skills/TBaguette/`), built using the TBaguette skills themselves as a live
test of the library.

## Concept: La Boulangerie TBaguette

An atelier for skills, not a novelty bread site. The bakery concept lives in brand
voice, palette, type, and a small set of restrained line-marks (wheat sheaf, crust
cross-section) — not in puns forced onto every category name. Reference point: a real
high-end bakery's actual identity design (gold foil, warm cream stock, one confident
serif), not clip-art bread.

Distinct from the pre-existing, unrelated `~/Code/sumptuous-ui-showcase` project
(a generic "anti-slop" design-system demo by a different tool). This site borrows
nothing from it directly beyond having independently confirmed what "sumptuous" means
in this user's vocabulary — dark, glassy depth, warm glow, serif+mono type pairing —
then diverges into its own palette, type, and motif so it reads as kin, not a reskin.

## Visual system (locked contract)

**Palette** — dark-first, warm bakery, one disciplined accent:

| Token | Value | Use |
|---|---|---|
| `--crust-950` | `#1a130f` | page background |
| `--crust-900` | `#22190f` | footer / deep alt background |
| `--crumb-850` | `#2b2015` | card surface |
| `--crumb-800` | `#362819` | raised / hover surface |
| `--gold-500` | `#dda25c` | primary accent (used rarely, means something) |
| `--gold-400` | `#e8b876` | accent hover / lighter |
| `--gold-700` | `#a86f34` | accent on light surfaces (AA contrast) |
| `--cream-100` | `#f5ead9` | primary text on dark |
| `--cream-300` | `#cbb79c` | secondary text on dark, warm-tinted, never neutral gray |
| `--ink-900` | `#241a12` | primary text on light |
| `--flour-50` | `#faf3e8` | light theme background |
| `--flour-100` | `#f2e6d2` | light theme card surface |

Light "flour" theme via `[data-theme="flour"]`, toggle in header. Both themes verified
≥4.5:1 body contrast before shipping.

**Type:**

- Display: `Fraunces` (variable, warm/soft character — real pedigree in food and
  editorial branding)
- Body: `Work Sans` (long reference text has to actually read well)
- Mono: `IBM Plex Mono` (skill-name tokens, category tags, code spans — deliberately
  not JetBrains Mono, so this doesn't echo `sumptuous-ui-showcase`)
- Scale ratio ~1.25, spacing scale base 4px (4/8/12/16/24/32/48/64/96)
- Radius: 8 / 14 / 22px — soft, organic, not sharp-corporate, not pill-everything

**Depth/motion:** warm glow on hover is earned here (oven-light, not generic glass) —
one authored hero entrance, lift+glow on cards, nothing louder. Reduced-motion
respected throughout.

**Icons:** single-stroke line marks as category markers only, never decoration filler.

## Structure

- `index.html` — hero, then the 10 categories in `CATALOG.md` order, each a grid of
  skill cards (name, one-line trigger summary, category tag) linking out. Client-side
  search/filter over an index embedded at generation time — no fetch, no CORS issues,
  works opened directly from disk or served.
- `skills/<slug>/index.html` × 64 — full rendered `SKILL.md` body, typeset for real
  reading, breadcrumb, prev/next within category, category siblings.
- `formidable` exception: its 24 reference files (12 stacks + 12 commands) are inlined
  as anchored, tabbed sections on its one page rather than spawning 24 more URLs —
  keeps "a page per skill" intact instead of ballooning to 88 pages. Internal relative
  markdown links inside formidable's own files (e.g. `reference/craft-floor.md`) are
  rewritten to same-page anchors at generation time.
- `assets/styles.css`, `assets/site.js` — hand-authored, not generated.
- `scripts/content_pipeline.py`, `scripts/templates.py`, `scripts/generate.py` —
  the generator. `index.html` and `skills/**` are generated output, marked as such
  with a header comment; the skill files under `~/.claude/skills/TBaguette/` remain
  the single source of truth. Re-running `generate.py` regenerates the site.

## Content contract (schema `content.json`)

```
{
  "categories": [{"slug", "title", "skill_slugs": [...]}, ... 9, in catalog order],
  "skills": {
    "<slug>": {
      "slug", "name", "category_slug", "category_title",
      "description",            // full frontmatter description
      "summary",                 // trimmed teaser, <=140 chars, for cards
      "body_html",                // rendered body, first `#` title line stripped
      "is_formidable": false
    },
    "formidable": {
      ...same base fields..., "is_formidable": true,
      "formidable_stacks":   [{"id", "title", "html"}, ... 12],
      "formidable_commands": [{"id", "title", "html"}, ... 12]
    }
  }
}
```

Markdown subset actually in use across these files (hand-rolled parser, no external
dependency): `#`/`##`/`###` headers, `**bold**`, `` `code` ``, GFM pipe tables,
`-`/`1.` lists (occasionally one level nested), `[text](url)` links (including
formidable's internal relative `.md` links, rewritten per above), paragraphs. No
images, no blockquotes, no raw HTML. Output is HTML-escaped by construction — content
is first-party but the generator treats it as untrusted input on principle
(`TBaguette:handling-untrusted-input`).

## Build approach

Two independent pieces built in parallel against this locked contract, then integrated:
content pipeline (parses skills → `content.json`) and site design/build (CSS, JS, page
templates, self-tested against a small fixture). No client framework, no bundler, no
npm dependency — the generator is the only build step, and its output is plain static
files.

## Out of scope

No deployment or hosting setup (local static files only, deploy-ready if wanted
later). No CMS or live editing — the generator is the update path.

## Verification

Static server via the Browser pane; click through landing, several skill pages
(including formidable's tabbed page and at least one table-heavy page), search/filter,
mobile width, contrast on both themes. Screenshot before calling it done.
