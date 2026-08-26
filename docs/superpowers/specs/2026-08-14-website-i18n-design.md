# Website i18n: 15 additional languages

Status: approved for planning
Date: 2026-08-14

## Goal

Translate the tbaguette-skills site (`lesplooch.github.io/tbaguette-skills/`)
into 15 languages beyond English, one per country, each language that
country's primary one:

| Code | Language | Country | Direction |
|---|---|---|---|
| zh | Mandarin Chinese (Simplified) | China | ltr |
| es | Spanish | Spain | ltr |
| hi | Hindi | India | ltr |
| ar | Arabic | Egypt | **rtl** |
| pt | Portuguese | Brazil | ltr |
| ru | Russian | Russia | ltr |
| ja | Japanese | Japan | ltr |
| de | German | Germany | ltr |
| fr | French | France | ltr |
| ko | Korean | South Korea | ltr |
| it | Italian | Italy | ltr |
| tr | Turkish | Turkey | ltr |
| vi | Vietnamese | Vietnam | ltr |
| pl | Polish | Poland | ltr |
| id | Indonesian | Indonesia | ltr |

English stays the 16th locale and the default: it keeps living at `/` and
`/skills/<slug>/` exactly as today, so every existing external link, bookmark,
and the current Pages URL keep working unchanged.

## Non-goals

- Translating the skills that install into `~/.claude/skills/` via the
  plugin. This is a *website* translation — `skills/` (the plugin payload)
  is untouched; only its rendering on the site gains language variants.
- Machine-translating at build time. Translations are static files checked
  into the repo, produced ahead of a build, not generated per-request.
- Locale-aware content *differences* (no region-specific skill content) —
  every locale renders the same 66 skills, same categories, same order;
  only the language changes.
- A language switcher that remembers preference via cookies/`Accept-Language`
  redirects. GitHub Pages serves static files with no server logic
  available; the switcher is a plain link list, and each locale's URL is
  the durable, shareable address for that language.

## Repo layout

```
i18n/
  fr/
    ui.json                       # chrome strings (header, search, footer, modal, ...)
    categories.json                # {category_slug: translated title}
    descriptions.json              # {skill_slug: translated description} — stepping
                                    # stone ahead of a full SKILL.md translation; see
                                    # Content pipeline below for precedence
    verify-install.json            # verify-install page's prose fields
    skills/
      naming-things/
        SKILL.md                   # translated frontmatter description + body
      formidable/
        SKILL.md
        reference/
          color.md
          stacks/web.md
          ...
  ar/
    ...
  ... (15 language directories total)
```

English remains solely `skills/` at the repo root — never duplicated into
`i18n/en/`. A translated `SKILL.md` keeps the same filename, same
`## heading` structure, and the same frontmatter `name:` (the slug is a URL
key, not translatable content); only `description:` and the body prose
translate.

Per-file fallback: any `i18n/<lang>/...` path that doesn't exist falls back
to the English source. This is deliberate — the recommended sequencing
(chrome + descriptions for all 15, skill bodies filled in afterward,
language by language) depends on a half-translated locale still rendering a
complete, correct page today, in English, for the parts not yet done.

## Content pipeline (`scripts/content_pipeline.py`)

`build_content(skills_root, locale=None, locale_root=None)` gains two
optional parameters — `locale` is the code (e.g. `"fr"`), `locale_root` is
that language's own directory (`i18n/fr`, not the parent `i18n/`), mirroring
how `skills_root` is already the leaf `skills/` directory rather than its
parent. Locale build behavior:

- `build_plain_skill_entry` and `build_formidable_skill_entry` first look
  for the skill's file(s) under `locale_root`, falling back to `skills_root`
  file-by-file (frontmatter file, and independently each `reference/*.md` /
  `reference/stacks/*.md` file for formidable). Each entry gains
  `"translated": bool` — true only when every one of its files came from
  the locale directory, so a partially-translated formidable page can still
  show an accurate "partially translated" state if we want it later, and
  the simple case (any file missing) reads as untranslated for the banner.
- **Description precedence is separate from body precedence**, which is
  what makes it possible to ship a translated card/meta-description ahead
  of a translated body (phase 1 vs. phase 2 below): resolve, in order, (1)
  the frontmatter `description:` of `i18n/<lang>/skills/<slug>/SKILL.md` if
  that file exists, else (2) `i18n/<lang>/descriptions.json[slug]` if
  present, else (3) the English frontmatter description. `translated`
  (used for the body-fallback banner) still reflects only the body-file
  outcome above — a `descriptions.json`-only translation gets a correctly
  translated card and page `<title>`/meta-description while the body itself
  still renders English with the banner. Once phase 2 adds a real
  `SKILL.md` for that skill, its own frontmatter description wins
  automatically (case 1) and the `descriptions.json` entry becomes inert —
  no cleanup step required, just no longer consulted.
- Category titles come from `i18n/<lang>/categories.json`, falling back to
  the English `CATEGORIES` title per-entry (same per-item fallback
  principle).
- Everything else (slugs, category structure, category order, markdown
  rendering logic, HTML escaping) is unchanged and locale-independent.

`build_content(skills_root)` with no locale arguments must produce
byte-identical output to today — this is what keeps the existing
`test_content_pipeline.py` suite valid unmodified as a regression net.

## UI strings (`scripts/templates.py`)

A `Strings` dataclass (or plain dict with a frozen key set) holds every
piece of hardcoded English chrome text currently embedded in f-strings:
header ("Updated", theme toggle labels), hero headline and lede (the
`{skill_count}`/`{category_count}`-interpolated paragraph), search ("Search
skills", placeholder, "No skills match", "Clear search"), footer nav label,
skip link, install panel hints and "Copy"/"Copied!", change badges ("New" /
"Updated"), breadcrumb "Home", prev/next "Previous"/"Next", "More in
{category}", formidable's "Stacks"/"Commands"/"Craft floor" tab-group
headings, the page `<title>` and meta-description templates (also
count-interpolated), verify-install page's ~980 words of prose, and the
update-available modal text that today lives in `site.js` (see below).
Every count-interpolated string is a template with a named placeholder
(`{skill_count}`, not string concatenation around a number) — word order
around a quantity varies by language, so concatenation would silently
produce ungrammatical output in several of the 15.

Brand identity — "TBaguette", "TBaguette's Atelier" — stays literal
in every locale, the same way the install
command's shell syntax does; only the descriptive copy around those names
translates.

Every `_render_*` function that currently inlines English text takes a
`strings: Strings = ENGLISH_STRINGS` keyword parameter, defaulting to the
current English catalog. This keeps every call site in
`test_templates.py` — all ~24KB of it — passing unmodified for the
default-locale case; only the new locale-build call sites pass an explicit
catalog. `ENGLISH_STRINGS` becomes the single source of truth for English
chrome copy (extracted from the current inline strings, not retyped), and
`i18n/<lang>/ui.json` must supply the exact same key set — a missing or
extra key fails the build (see Testing) rather than silently falling back
mid-catalog, since chrome strings are small and fully translated for every
shipped locale from day one, unlike skill bodies.

`_render_document` gains `lang` and `dir` (from the locale registry) and
emits `<html lang="{code}" dir="{ltr|rtl}">` instead of the hardcoded
`lang="en"`, plus a `<link rel="alternate" hreflang="...">` block (one per
locale, computed from the locale registry, plus `x-default` pointing at the
English page) and a self-referencing `<link rel="canonical">`.

## Locale registry (`scripts/locales.py`, new)

```python
LOCALES = [
    {"code": "en", "hreflang": "en", "name": "English", "endonym": "English", "dir": "ltr", "default": True},
    {"code": "fr", "hreflang": "fr", "name": "French", "endonym": "Français", "dir": "ltr"},
    {"code": "zh", "hreflang": "zh-Hans", "name": "Chinese", "endonym": "中文", "dir": "ltr"},
    {"code": "pt", "hreflang": "pt-BR", "name": "Portuguese", "endonym": "Português", "dir": "ltr"},
    {"code": "ar", "hreflang": "ar", "name": "Arabic", "endonym": "العربية", "dir": "rtl"},
    ... # all 16, en included
]
EXPECTED_LOCALE_COUNT = 16
```

`code` is the URL prefix and the `i18n/<code>/` directory name; `hreflang`
is the more specific IETF tag search engines expect (`pt-BR`, `zh-Hans`)
even though the URL itself stays the short `code`. This is the one place
the 15-language list is declared; `generate.py`, the switcher, and the test
suite all read it rather than hardcoding the list a second time.

## Routing and build (`scripts/generate.py`)

- `EXPECTED_LOCALE_COUNT = 16` gate, mirroring the existing
  `EXPECTED_SKILL_COUNT` gate: refuses to build if `locales.py`'s count
  drifts from this constant.
- `generate()` loops over every locale in the registry. English builds
  exactly as it does today, at the existing paths. Each other locale builds
  into `docs/<code>/index.html`, `docs/<code>/skills/<slug>/index.html`,
  `docs/<code>/verify-install/index.html` — the install command's literal
  shell/PowerShell text is never translated (it's code, not prose); only
  its surrounding explanation is.
- The atomic staging-directory swap extends to all 16 top-level output
  paths (`index.html`, `skills/`, `verify-install/`, `version.txt`, and 15×
  `<code>/`) — same all-or-nothing guarantee the module docstring already
  commits to, just wider.
- New `--locale <code>` flag restricts a build to one locale (plus English,
  since English is the fallback source) for fast local iteration while
  translating; omitted (the default, and what CI/the pre-commit hook always
  uses) builds all 16.
- `docs/version.txt` and the update-check flow stay single: one
  build-instant, one version file, shared by every locale (a reload check
  is about "is this deployment stale," not language).

## Language switcher

A `<details>` element in the header (no JS required — matters on a static
site, and keeps the theme toggle's script-optional philosophy intact),
listing all 16 locales by endonym, current one marked
`aria-current="true"`. Each link is a pure slug-preserving prefix swap
computed at build time (`/fr/skills/naming-things/` ↔
`/ar/skills/naming-things/` ↔ `/skills/naming-things/`) — safe because
slugs are shared verbatim across every locale by design (URL question,
already decided).

## `site.js` — shared across all locales, must stop hardcoding English

`site.js` is one asset file referenced by every page regardless of locale;
today it hardcodes "Copied!", "No skills match.", "Switch to light/dark
theme", and the entire update-modal ("New version available" / "This page
has been updated. Reload to see the latest." / "Reload"). These become
`data-i18n-*` attributes rendered by `templates.py` per-locale (e.g.
`data-i18n-copied="Copié !"` on the copy button, `data-i18n-no-match`
on the search status element, `data-i18n-modal-title` /
`data-i18n-modal-body` / `data-i18n-modal-reload` on a `<template>` in the
document, `data-i18n-theme-light` / `data-i18n-theme-dark` on the toggle),
with `site.js` reading from `dataset` instead of the current literals.
Behavior is unchanged for English since the rendered attribute values equal
today's hardcoded strings exactly.

## Untranslated-content fallback banner

Rendered only when `skill["translated"]` is false for a given locale build
(skill body or a formidable reference file is still English while the rest
of the page is in the visitor's language). The banner text itself comes
from `ui.json` (so it renders in the reader's own language — "This page
hasn't been translated into Français yet; showing the English version"),
and the English body content it wraps is marked `lang="en"` so a screen
reader switches voice/pronunciation correctly for that region instead of
mispronouncing English text as if it were French.

## RTL (Arabic)

`docs/assets/styles.css` (1,638 lines) has 18 physical-direction
declarations: 6 `padding-left`, 4 `transform: translateX`, 3 `left:`,
2 `text-align: right`, 2 `text-align: left`, 1 `right:`. It already uses 9
logical properties (`inline-start`/`inline-end` family) elsewhere, so this
is a completion of an existing pattern, not a new one:

- The 6 `padding-left` and the `text-align: left/right` pairs convert to
  `padding-inline-start` / `text-align: start` (or `end`), which mirror
  automatically under `dir="rtl"` with no selector duplication needed.
- The 3 `left:`/1 `right:` positioning declarations and 4 `translateX`
  transforms are layout-critical (dropdown/tooltip offsets, icon
  micro-motion) and get an explicit `[dir="rtl"] { ... }` override block
  rather than a logical-property conversion, since a couple of them are
  paired with non-mirroring sibling values that a blind logical-property
  swap would get wrong.
- Because the layout is flex/grid throughout, the box model itself mirrors
  for free under `dir="rtl"` — this pass only has to fix declarations that
  bypass the box model.

## Testing (`scripts/test_i18n.py`, new — wired into `run_tests.py`)

- `locales.py` has exactly `EXPECTED_LOCALE_COUNT` entries, unique codes,
  every `dir` is `"ltr"` or `"rtl"`, English is present and marked default.
- Every `i18n/<code>/ui.json` has exactly the same key set as
  `ENGLISH_STRINGS` — no missing key, no extra key — checked for whichever
  locale(s) the current build actually includes (all 16 on a full build;
  just the requested code plus English on a `--locale`-scoped one, since
  that build never touches the other locale directories in the first
  place).
- Per-file fallback: a locale directory missing a given skill's `SKILL.md`
  still produces a complete page, in English, for that skill, with
  `translated: false` and the fallback banner present; a locale directory
  present for that skill produces `translated: true` and no banner.
- Description precedence: with only `descriptions.json[slug]` present (no
  `SKILL.md` for that skill), the card/meta-description/title render the
  `descriptions.json` translation while the body still shows English +
  banner (`translated: false`); once an `i18n/<lang>/skills/<slug>/SKILL.md`
  is added, its own frontmatter description wins even if
  `descriptions.json` still has a (now-ignored) entry for that slug.
- `hreflang` block on every generated page lists all 16 locales plus
  `x-default`, and every listed URL 200s within the build output (path
  exists in the generated tree).
- Switcher renders exactly 16 links (15 others + a disabled/current marker
  for the active one), each a correct slug-preserving prefix swap.
- `<html dir="rtl">` is present only for `ar`, and only there; every other
  locale (including English) is `dir="ltr"`.
- `generate.py --locale fr` builds only `docs/fr/` (+ the untouched English
  root) and leaves other locale directories alone; a full run (no flag)
  touches all 16.

Existing suites (`test_content_pipeline.py`, `test_templates.py`,
`test_generate.py`) must keep passing unmodified against the English
default-parameter path — this is the regression net that proves the i18n
work is additive, not a rewrite of the current English site's behavior.

## Sequencing (matches the approved answer: infra + chrome first)

1. **Infra**: `locales.py`, `content_pipeline.py` locale parameters,
   `Strings`/`ui.json` plumbing through `templates.py`, `site.js`
   `data-i18n-*` conversion, routing + hreflang + switcher in `generate.py`,
   RTL CSS pass, `test_i18n.py`. Ships with all 15 `i18n/<lang>/ui.json` +
   `categories.json` + `descriptions.json` + `verify-install.json`
   translated (~75,000 words: chrome + 66 descriptions + verify-install
   page, × 15) and **zero** `i18n/<lang>/skills/*/SKILL.md` files yet —
   every locale is live, navigable, and correctly labeled, cards and page
   titles read in-language via `descriptions.json`, and every skill body
   shows the fallback banner over English text until phase 2 reaches it.
2. **Bodies**: fill `i18n/<lang>/skills/<slug>/SKILL.md` (and formidable's
   `reference/` tree) language by language, skill by skill — streamed in
   over however many follow-up sessions it takes (~1.5M words remaining
   across 66 skills × 15 languages). `scripts/i18n_status.py` (new) reports
   per-locale coverage (`N/66 skills translated`) so this is resumable
   without re-deriving state from scratch each session.

Phase 2 never blocks on being "done" — every regeneration after phase 1
ships whatever fraction is translated so far, correctly falling back for
the rest.

## Known costs

- `docs/` grows from 68 generated files (~1.7MB) to roughly 1,088
  (~27MB) once all bodies are filled — still trivial for GitHub Pages
  (1GB limit), but every full regeneration rewrites all of it, so commits
  carry a larger diff than today's single-locale ones.
- Phase 1 alone is a multi-session translation effort (~75,000 words);
  phase 2 is substantially larger (~1.5M words) and will span many
  sessions over time, tracked via `i18n_status.py` rather than attempted
  in one sitting.
