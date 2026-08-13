# Site auto-update check — design

## Purpose

The showcase site (`docs/`, served by GitHub Pages directly from `master`) has no
push mechanism today — a visitor who loads a page keeps that snapshot until they
manually reload, even if a new deploy landed seconds later. This adds a client-side
check that notices a new deploy and prompts the visitor to reload, so a page left
open stays current without a manual refresh.

## Detection mechanism

`scripts/generate.py` already computes one `last_updated_utc` instant per run and
stamps it into every page's header (`_render_updated_time()` in `templates.py`,
via the shared `_render_document()` → `_render_header()` chain all three page
types go through). This instant becomes the version signal:

- `_build_into()` writes one more small file alongside `index.html`, the skill
  pages, and `verify-install/index.html`: `docs/version.txt`, containing exactly
  `last_updated_utc` as plain text, nothing else.
- `generate()`'s atomic swap already promotes `("index.html", "skills",
  "verify-install")` from the staging directory as independent renames; add
  `"version.txt"` as a fourth entry so it's covered by the same atomicity and
  repeat-run safety as everything else `generate()` owns.
- `_render_updated_time()`'s existing `<time data-format-updated
  datetime="...">` element gains one more attribute: `data-version-url="{base
  path}/version.txt"`. Client JS reads both `datetime` (the value captured at
  load) and `data-version-url` (where to poll) off the one element it already
  queries — no new template plumbing beyond this.

`version.txt` is a generated file, in the same "never hand-edit, `generate.py`
owns it" category as `index.html` / `skills/` / `verify-install/`.

## Client-side polling

New sixth feature in `docs/assets/site.js`, following the file's existing
pattern exactly: one more `init*()` function, a no-op if its markup isn't
present, called alongside the other five at the bottom of the IIFE. The
top-of-file doc comment listing the five features gets a sixth line.

- Interval: random, re-rolled every cycle, uniformly distributed in [10, 12]
  seconds — not a fixed 10s or 12s tick.
- Each cycle: if `document.hidden`, skip the network call and reschedule —
  keeps the timer chain alive without spending requests on a backgrounded tab.
  Worst case, a refocused tab notices within one interval.
- Otherwise: `fetch(versionUrl + '?t=' + Date.now(), { cache: 'no-store' })`.
  The query param is belt-and-suspenders against any intermediate cache that
  doesn't honor `cache: 'no-store'` semantics — GitHub Pages' CDN headers
  aren't something this project controls. (Observed this session: once a Pages
  build shows `status: "built"` via the API, a fresh fetch already serves
  current content, so no extra propagation delay is expected beyond the build
  itself completing.)
- Compare the trimmed response text to the value captured at load.
  - Non-empty and different from the load-time value → stop polling, show the
    modal (once — a `document.querySelector('.update-modal')` guard prevents
    a duplicate).
  - Anything else — empty response, same value, or a fetch failure
    (offline, transient) → reschedule, no error surfaced.

## Modal — reload-only

Per explicit decision: no dismiss/close action. Reload is the only control.

- Structure: full-viewport fixed overlay, flex-centered, dimmed scrim with
  `backdrop-filter: blur(...)`; browsers without `backdrop-filter` support
  still get the plain dimmed scrim (progressive enhancement, not a blocker).
- Card reuses the existing design system's surface/border/shadow/radius
  tokens (the same family `.install-frame` already uses), not new ones.
- Accessibility: `role="alertdialog"`, `aria-modal="true"`,
  `aria-labelledby`/`aria-describedby` pointing at the title/body text, focus
  moves to the Reload button on open. The header/main/footer regions get the
  HTML `inert` attribute while the modal is open, so background content is
  genuinely unreachable by Tab or click — not just visually covered.
- No backdrop-click-to-close, no Esc-to-close — consistent with "reload-only"
  being a deliberate choice, not an oversight.
- Copy (draft, not locked): title "New version available", body "This page
  has been updated. Reload to see the latest.", button "Reload".

## Reload + scroll restore

Requested explicitly: reloading shouldn't dump the visitor back at the top of
a long page.

- On click: `sessionStorage.setItem(key, String(window.scrollY))`, then
  `window.location.reload()`.
- On the next load: a small `initScrollRestore()` reads that key, if present
  scrolls to it and immediately removes the key — so it applies only to the
  reload it was saved for, never a later unrelated refresh.
- This runs as a normal deferred `init*()` call in `site.js`, not as a
  synchronous inline `<head>` script the way the existing theme-flash fix
  (`_THEME_BOOTSTRAP_JS`) does. That inline-script trick exists because the
  theme flash happens on *every* load; scroll restore only fires on the rare
  update-triggered reload, so a barely-perceptible scroll snap (if any) is an
  accepted trade against adding a second inline-bootstrap-script precedent to
  the codebase.

## Testing

- **Python (generate.py / templates.py):** real, automated coverage, same
  rigor as the rest of this project's test suite.
  - `test_generate.py`: `version.txt` is written, its content equals the run's
    `last_updated_utc`, it survives the atomic swap and a second/repeat run.
  - `test_templates.py`: `data-version-url` is present on the rendered output
    of all three page types (index, skill page, verify-install) and is
    correctly `base_path`-prefixed, both with and without a `base_path`.
- **JS (site.js / styles.css):** no automated coverage — this project has zero
  JS test tooling by design (stdlib-only, "zero install step is a hard
  requirement," per `templates.py`'s own docstring), and adding one for a
  single feature would be disproportionate. Verified manually instead: build
  locally, hand-edit `docs/version.txt` to simulate a new deploy, confirm the
  modal appears and blurs correctly in both the `crust` and `flour` themes,
  confirm keyboard focus can't escape the modal, confirm Reload restores
  scroll position.

## Out of scope

- No dismiss/snooze affordance (explicit decision).
- No automated JS test suite (matches existing project constraints).
- No change to how often the site itself is actually rebuilt/deployed — this
  only affects how a page that's already open notices a deploy that already
  happened.
