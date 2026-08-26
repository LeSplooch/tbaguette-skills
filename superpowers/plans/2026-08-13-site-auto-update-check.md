# Site Auto-Update Check Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** A page from this site notices when a new deploy has landed and prompts the visitor to reload, without losing their scroll position.

**Architecture:** `generate.py` already stamps one UTC instant into every page's header on each run; it now also writes that same instant to a tiny `docs/version.txt`. Every page already carries that instant in a `<time data-format-updated datetime="...">` element; that element gains a `data-version-url` attribute pointing at `version.txt`. Client JS in `site.js` polls `version.txt` every 10-12s (jittered), and on a mismatch shows a reload-only modal that preserves scroll position across the reload.

**Tech Stack:** Python stdlib (`generate.py`/`templates.py`), vanilla JS (`site.js`), hand-authored CSS (`styles.css`). No new dependencies anywhere — matches this project's zero-install-step constraint.

## Global Constraints

- Python: stdlib only, no new imports beyond `re` (already in the standard library) in `test_generate.py`.
- `docs/assets/site.js` and `docs/assets/styles.css` are hand-authored source, not generated — edit them directly at their served path, same as every other change to those files.
- New JS is a 6th/7th `init*()` function in the existing `site.js` IIFE, following its established pattern: a no-op if its markup isn't present, called from the bottom alongside the other five.
- New CSS reuses existing custom properties only (`--space-*`, `--text-*`, `--surface`, `--border-subtle`, `--accent-solid`, `--radius-*`, `--font-display`, `--font-body`, `--duration-micro`, `--ease-out`, `--focus-ring`) — no new tokens.
- `scripts/generate.py` must remain the very last step before a commit (existing project invariant — the pre-commit hook already runs it). Nothing in this plan changes that.
- The modal is reload-only: no dismiss/close/Esc/backdrop-click affordance. This is a deliberate, already-approved product decision, not an oversight to "fix" during implementation.
- Spec: `superpowers/specs/2026-08-13-auto-update-check-design.md`. Read it first if anything below is unclear — this plan implements it exactly, and the two should never disagree.

---

## File Structure

| File | Change |
|---|---|
| `scripts/generate.py` | `_build_into()` writes `docs/version.txt`; `generate()`'s atomic-swap list gains `"version.txt"` |
| `scripts/test_generate.py` | New checks: `version.txt` exists, matches the run's baked-in timestamp, survives a second run |
| `scripts/templates.py` | `_render_updated_time()` gains a `base_path` param and emits `data-version-url` |
| `scripts/test_templates.py` | New checks: `data-version-url` present, correctly `base_path`-prefixed, on both index and skill pages |
| `docs/assets/site.js` | New `showUpdateModal()`, `initVersionCheck()`, `initScrollRestore()`; doc-comment update; two new calls at the bottom |
| `docs/assets/styles.css` | New `.update-modal-overlay` / `.update-modal` / `.update-modal__*` rules |

No new files except this plan and the spec it implements.

---

### Task 1: Generate `docs/version.txt`

**Files:**
- Modify: `scripts/generate.py:144-175` (`_build_into`), `scripts/generate.py:214` (`generate`'s swap loop)
- Test: `scripts/test_generate.py`

**Interfaces:**
- Produces: `docs/version.txt`, plain text, content is exactly `last_updated_utc` (e.g. `2026-08-13T19:42:07+00:00`) with no header/prefix — Task 2's client code and Task 3's polling code both depend on this being a bare, directly-comparable string.

- [x] **Step 1: Write the failing checks**

Open `scripts/test_generate.py`. Add `import re` to the imports at the top (alongside the existing `import shutil` etc.).

In `main()`, find this existing block (it reads `index_html` and computes `docs`):

```python
        index_html = (docs / "index.html").read_text(encoding="utf-8")
        check(
            "index mentions the real, current skill count, not a stale hardcoded one",
            f"see all {generate.EXPECTED_SKILL_COUNT} again" in index_html,
        )
        check(
            "index links to the new skill",
            'href="/tbaguette-skills/skills/karen-and-the-manager/"' in index_html,
        )
```

Immediately after it (still before the `print("second run ...")` line), add:

```python
        version_txt_path = docs / "version.txt"
        check("version.txt exists after generation", version_txt_path.exists())
        version_txt_content = version_txt_path.read_text(encoding="utf-8")

        index_dt_match = re.search(r'datetime="([^"]+)"', index_html)
        check("index.html's header carries a parseable datetime attribute",
              index_dt_match is not None)
        check(
            "version.txt's content matches the timestamp baked into index.html, "
            "byte for byte (this is the exact string client JS string-compares "
            "against, so it must carry no GENERATED_HEADER or other prefix)",
            index_dt_match is not None and version_txt_content == index_dt_match.group(1),
        )

        formidable_dt_match = re.search(r'datetime="([^"]+)"', formidable_html)
        check(
            "the same version.txt also matches a skill page's timestamp "
            "(one run, one instant, everywhere)",
            formidable_dt_match is not None and version_txt_content == formidable_dt_match.group(1),
        )
```

Then find the "second run" block:

```python
        print("second run (repeat-safety of the atomic swap)")
        content2 = generate.generate(tmp_root, real_skills_root, base_path="/tbaguette-skills")
        check(
            "second run returns the same skill count",
            len(content2["skills"]) == len(content["skills"]),
        )
        check(
            "no leftover staging or backup directory after two runs",
            not any(p.name.startswith(".docs.") for p in tmp_root.iterdir()),
        )
```

Immediately after it, add:

```python
        check("version.txt still exists after a second run", version_txt_path.exists())
        check(
            "no leftover version.txt backup file after two runs",
            not (tmp_root / ".docs.previous.version.txt").exists(),
        )
```

- [x] **Step 2: Run the tests to verify they fail**

Run: `python3 scripts/test_generate.py`
Expected: FAIL — `version.txt exists after generation` (and everything depending on it) fails, because `generate.py` doesn't write that file yet.

- [x] **Step 3: Implement**

In `scripts/generate.py`, inside `_build_into()`, find the end of the function:

```python
    verify_html = templates.render_verify_install_page(
        highlighted_lines, categories, base_path, last_updated_utc=last_updated_utc,
    )
    _write(output_dir / "verify-install" / "index.html", verify_html)
```

Add immediately after (note: this bypasses the `_write()` helper deliberately — `_write()` prepends `GENERATED_HEADER`, an HTML comment, which would break the byte-for-byte string comparison client JS does against this file):

```python

    # Plain text, no GENERATED_HEADER — client JS in site.js string-compares
    # this directly against the datetime it captured at page load, so this
    # must be exactly last_updated_utc and nothing else.
    (output_dir / "version.txt").write_text(last_updated_utc, encoding="utf-8")
```

Then in `generate()`, find:

```python
    for name in ("index.html", "skills", "verify-install"):
```

Change to:

```python
    for name in ("index.html", "skills", "verify-install", "version.txt"):
```

(No other change needed — the swap loop already handles files and directories generically via `.is_dir()` checks, and `version.txt` is a plain file like `index.html`.)

- [x] **Step 4: Run the tests to verify they pass**

Run: `python3 scripts/test_generate.py`
Expected: PASS, all checks including the new ones.

- [x] **Step 5: Run the full suite and commit**

Run: `python3 scripts/run_tests.py`
Expected: all suites pass.

```bash
git add scripts/generate.py scripts/test_generate.py
git commit -m "Write docs/version.txt alongside every generated page

Gives the upcoming site auto-update check something cheap to poll — a
25-byte file instead of re-fetching a full page just to read one
timestamp out of it."
```

---

### Task 2: Bake the version-check URL into every page

**Files:**
- Modify: `scripts/templates.py:125-141` (`_render_updated_time`, `_render_header`)
- Test: `scripts/test_templates.py`

**Interfaces:**
- Consumes: nothing new — uses the `base_path` already threaded through `_render_header`/`_render_document`.
- Produces: every page's `<time data-format-updated>` element gains `data-version-url="{base_path}/version.txt"`. Task 3's `initVersionCheck()` depends on this exact attribute name and on it living on the same element as `data-format-updated`.

- [x] **Step 1: Write the failing checks**

Open `scripts/test_templates.py`. In `check_header_and_badges()`, find:

```python
    check("header time element is wired for site.js to find and reformat",
          "data-format-updated" in html)
```

Immediately after it, add:

```python
    check("header time element also carries the version-check URL, "
          "base_path-prefixed (empty base_path here, so root-relative)",
          'data-version-url="/version.txt"' in html)
```

In `check_base_path()`, find:

```python
    check("prefixed skill card link", f'href="{base}/skills/formidable/"' in index_html)
```

Immediately after it, add:

```python
    check("version-check URL is base_path-prefixed too", f'data-version-url="{base}/version.txt"' in index_html)
```

And find:

```python
    check("prefixed icon sprite reference", f'{base}/assets/icons.svg#' in page_html)
```

Immediately after it, add:

```python
    check("skill page's version-check URL is base_path-prefixed too",
          f'data-version-url="{base}/version.txt"' in page_html)
```

- [x] **Step 2: Run the tests to verify they fail**

Run: `python3 scripts/test_templates.py`
Expected: FAIL — the three new `data-version-url` checks fail, since `templates.py` doesn't emit that attribute yet.

- [x] **Step 3: Implement**

In `scripts/templates.py`, find:

```python
def _render_updated_time(last_updated_utc: str) -> str:
    # last_updated_utc is a UTC instant baked in at generation time (see
    # generate.py's own docstring on why it must be the very last step
    # before commit for this to be honest). Rendered here as a plain UTC
    # string so the page still says something true with JS disabled;
    # site.js's initUpdatedTime() replaces the text with both the visitor's
    # local time and UTC once it runs, since the visitor's own timezone
    # can't be known at build time.
    fallback = escape_html(last_updated_utc.replace("+00:00", "Z")) + " UTC"
    return f"""<p class="site-header__updated">
      <span class="site-header__updated-label">Updated</span>
      <time class="site-header__updated-value" datetime="{escape_html(last_updated_utc)}" data-format-updated>{fallback}</time>
    </p>"""


def _render_header(base_path: str = "", last_updated_utc: str = "") -> str:
    updated_html = _render_updated_time(last_updated_utc) if last_updated_utc else ""
```

Replace with:

```python
def _render_updated_time(last_updated_utc: str, base_path: str = "") -> str:
    # last_updated_utc is a UTC instant baked in at generation time (see
    # generate.py's own docstring on why it must be the very last step
    # before commit for this to be honest). Rendered here as a plain UTC
    # string so the page still says something true with JS disabled;
    # site.js's initUpdatedTime() replaces the text with both the visitor's
    # local time and UTC once it runs, since the visitor's own timezone
    # can't be known at build time. data-version-url points site.js's
    # initVersionCheck() at the generated docs/version.txt, which always
    # carries this same instant — see scripts/generate.py's _build_into().
    fallback = escape_html(last_updated_utc.replace("+00:00", "Z")) + " UTC"
    return f"""<p class="site-header__updated">
      <span class="site-header__updated-label">Updated</span>
      <time class="site-header__updated-value" datetime="{escape_html(last_updated_utc)}" data-format-updated data-version-url="{base_path}/version.txt">{fallback}</time>
    </p>"""


def _render_header(base_path: str = "", last_updated_utc: str = "") -> str:
    updated_html = _render_updated_time(last_updated_utc, base_path) if last_updated_utc else ""
```

(`base_path` is not HTML-escaped here, matching every other `{base_path}/...` href in this file, e.g. `_render_head`'s `href="{base_path}/assets/favicon.svg"` — it's a trusted internal constant, either `""` or `"/tbaguette-skills"`, never external input.)

- [x] **Step 4: Run the tests to verify they pass**

Run: `python3 scripts/test_templates.py`
Expected: PASS, all checks including the new ones.

- [x] **Step 5: Run the full suite and commit**

Run: `python3 scripts/run_tests.py`
Expected: all suites pass.

```bash
git add scripts/templates.py scripts/test_templates.py
git commit -m "Bake the version-check poll URL into every page's header

Colocated with the existing baked-in build timestamp so the upcoming
client-side poller can read both its starting value and its poll
target off the one element it already knows how to find."
```

---

### Task 3: Client-side polling, reload-only modal, scroll restore

**Files:**
- Modify: `docs/assets/site.js`

**Interfaces:**
- Consumes: `[data-format-updated]` element's `datetime` and `data-version-url` attributes (from Task 2). `toArray()`, already defined earlier in this file.
- Produces: `.update-modal-overlay` / `.update-modal` / `.update-modal__title` / `.update-modal__body` / `.update-modal__reload` DOM structure and class names — Task 4's CSS depends on these exact class names. Also the `tbaguette-reload-scroll-y` sessionStorage key, written by the reload button's click handler and consumed by `initScrollRestore()`.

No automated test — this project has no JS test tooling by design (stdlib-only, zero-install-step). Verified manually in Step 3.

- [x] **Step 1: Update the file's own doc comment**

Open `docs/assets/site.js`. Find the top-of-file comment:

```js
/*
 * TBaguette’s Atelier — site.js
 * Vanilla JS, no dependencies. Five independent features, each a no-op if
 * its markup isn't on the current page:
 *   - theme toggle   (every page)
 *   - search/filter  (landing page only)
 *   - tabs           (formidable's skill page's Stacks/Commands; the
 *                      landing page's install-command platform picker)
 *   - copy install command (landing page only; one button per platform tab)
 *   - header "Updated" time (every page — formats the baked-in UTC instant
 *                             as the visitor's local time)
 * Loaded with `defer`, so the DOM is fully parsed before any of this runs —
 * no DOMContentLoaded wrapper needed.
 */
```

Replace with:

```js
/*
 * TBaguette’s Atelier — site.js
 * Vanilla JS, no dependencies. Seven independent features, each a no-op if
 * its markup isn't on the current page:
 *   - theme toggle   (every page)
 *   - search/filter  (landing page only)
 *   - tabs           (formidable's skill page's Stacks/Commands; the
 *                      landing page's install-command platform picker)
 *   - copy install command (landing page only; one button per platform tab)
 *   - header "Updated" time (every page — formats the baked-in UTC instant
 *                             as the visitor's local time)
 *   - site update check (every page — polls docs/version.txt every 10-12s;
 *                         on a mismatch, shows a reload-only modal)
 *   - post-reload scroll restore (every page — companion to the update
 *                                  check; restores scroll position after
 *                                  the reload it triggered)
 * Loaded with `defer`, so the DOM is fully parsed before any of this runs —
 * no DOMContentLoaded wrapper needed.
 */
```

- [x] **Step 2: Add the new functions**

Find the end of the IIFE — the block of `init*()` calls followed by the closing `})();`:

```js
  initThemeToggle();
  initSearch();
  initInstallCopy();
  initTabs();
  initUpdatedTime();
})();
```

Replace with (new functions above the calls, two new calls added):

```js
  // -------------------------------------------------------------------
  // Site update check — polls a tiny generated file for a newer build,
  // then prompts a reload. Reload-only by design: no dismiss/snooze/Esc/
  // backdrop-click, since the point is to not leave a visitor on a stale
  // page. Scroll position is saved before reloading and restored after,
  // via initScrollRestore() below.
  // -------------------------------------------------------------------

  var SCROLL_RESTORE_KEY = 'tbaguette-reload-scroll-y';
  var UPDATE_POLL_MIN_MS = 10000;
  var UPDATE_POLL_MAX_MS = 12000;

  function showUpdateModal() {
    if (document.querySelector('.update-modal-overlay')) return;

    var overlay = document.createElement('div');
    overlay.className = 'update-modal-overlay';
    overlay.innerHTML =
      '<div class="update-modal" role="alertdialog" aria-modal="true" ' +
      'aria-labelledby="update-modal-title" aria-describedby="update-modal-body">' +
      '<p class="update-modal__title" id="update-modal-title">New version available</p>' +
      '<p class="update-modal__body" id="update-modal-body">This page has been updated. Reload to see the latest.</p>' +
      '<button class="update-modal__reload" type="button">Reload</button>' +
      '</div>';

    // Genuinely modal, not just visually on top: everything already in
    // <body> (skip link, header, main, footer) becomes untabbable and
    // unclickable. The overlay itself is appended after, so it's never
    // included in this pass.
    toArray(document.body.children).forEach(function (el) {
      el.setAttribute('inert', '');
    });
    document.body.appendChild(overlay);

    var reloadButton = overlay.querySelector('.update-modal__reload');
    reloadButton.addEventListener('click', function () {
      try {
        sessionStorage.setItem(SCROLL_RESTORE_KEY, String(window.scrollY));
      } catch (error) {
        // Private browsing / storage disabled: reload still happens, it
        // just won't restore scroll position.
      }
      window.location.reload();
    });
    reloadButton.focus();
  }

  function initVersionCheck() {
    var timeEl = document.querySelector('[data-format-updated]');
    if (!timeEl) return;
    var versionUrl = timeEl.getAttribute('data-version-url');
    var initialVersion = timeEl.getAttribute('datetime');
    if (!versionUrl || !initialVersion || !window.fetch) return;

    function scheduleNext() {
      var delay = UPDATE_POLL_MIN_MS + Math.random() * (UPDATE_POLL_MAX_MS - UPDATE_POLL_MIN_MS);
      setTimeout(poll, delay);
    }

    function poll() {
      // Skip the network call while backgrounded; the timer chain keeps
      // running, so a refocused tab notices within one interval.
      if (document.hidden) {
        scheduleNext();
        return;
      }
      // cache: 'no-store' bypasses the browser's own HTTP cache; the ?t=
      // query param is belt-and-suspenders against any intermediate cache
      // that doesn't honor that — GitHub Pages' CDN headers aren't
      // something this project controls.
      var bustedUrl = versionUrl + (versionUrl.indexOf('?') === -1 ? '?' : '&') + 't=' + Date.now();
      fetch(bustedUrl, { cache: 'no-store' }).then(function (response) {
        if (!response.ok) throw new Error('version check failed: ' + response.status);
        return response.text();
      }).then(function (text) {
        var latest = text.trim();
        if (latest && latest !== initialVersion) {
          showUpdateModal();
        } else {
          scheduleNext();
        }
      }).catch(function () {
        // Offline / transient failure: try again next cycle, no error
        // surfaced to the visitor.
        scheduleNext();
      });
    }

    scheduleNext();
  }

  function initScrollRestore() {
    var saved;
    try {
      saved = sessionStorage.getItem(SCROLL_RESTORE_KEY);
      if (saved !== null) sessionStorage.removeItem(SCROLL_RESTORE_KEY);
    } catch (error) {
      return;
    }
    if (saved === null) return;
    var y = parseInt(saved, 10);
    if (!isNaN(y)) window.scrollTo(0, y);
  }

  initThemeToggle();
  initSearch();
  initInstallCopy();
  initTabs();
  initUpdatedTime();
  initScrollRestore();
  initVersionCheck();
})();
```

- [x] **Step 3: Manually verify**

Start the local preview server (already configured in `.claude/launch.json` as `docs-preview`, `python3 -m http.server 8842 --directory docs`) and open `http://localhost:8842/`.

1. Open the browser console. Confirm no errors on load.
2. In a second terminal, hand-edit `docs/version.txt` — change its content to any different string (e.g. append `x`) and save.
3. Wait up to 12 seconds. Confirm the modal appears: centered, dimmed/blurred background, title "New version available", body text, one "Reload" button.
4. Confirm keyboard Tab cannot reach anything outside the modal (header theme toggle, footer links) while it's open.
5. Scroll down the page before the modal would trigger next time; repeat steps 2-3; click "Reload"; confirm the page reloads and lands back at the same scroll position rather than the top.
6. Revert `docs/version.txt` back to its real generated content afterward (re-run `python3 scripts/generate.py` locally, or restore via `git checkout -- docs/version.txt`) so the working tree isn't left with a hand-edited generated file.

- [x] **Step 4: Commit**

```bash
git add docs/assets/site.js
git commit -m "Add client-side update check: poll, reload-only modal, scroll restore

Polls docs/version.txt every 10-12s (jittered, paused while the tab is
hidden). On a mismatch, shows a modal whose only control is Reload —
no dismiss, by design. Scroll position is saved before reloading and
restored after, so a visitor mid-read on a long page doesn't get
dumped back at the top."
```

---

### Task 4: Modal styling — centered, blurred, theme-aware

**Files:**
- Modify: `docs/assets/styles.css`

**Interfaces:**
- Consumes: class names from Task 3 (`.update-modal-overlay`, `.update-modal`, `.update-modal__title`, `.update-modal__body`, `.update-modal__reload`) and existing design tokens (see Global Constraints).

No automated test — same reasoning as Task 3. Verified manually in Step 2.

- [x] **Step 1: Add the CSS**

Append to `docs/assets/styles.css` (end of file, in its own clearly-labeled section, following the file's existing numbered-section-comment convention):

```css
/* ---------------------------------------------------------------------------
   Update-check modal — reload-only, centered, blurred backdrop
   ------------------------------------------------------------------------- */

.update-modal-overlay {
  position: fixed;
  inset: 0;
  z-index: 1000;
  display: flex;
  align-items: center;
  justify-content: center;
  padding: var(--space-4);
  background: rgb(0 0 0 / 45%);
  backdrop-filter: blur(8px);
  -webkit-backdrop-filter: blur(8px);
}

.update-modal {
  width: 100%;
  max-width: 24rem;
  padding: var(--space-6);
  background: var(--surface);
  border: 1px solid var(--border-subtle);
  border-radius: var(--radius-lg);
  box-shadow: 0 24px 52px -18px rgb(0 0 0 / 45%);
  text-align: center;
}

.update-modal__title {
  margin: 0 0 var(--space-2);
  font-family: var(--font-display);
  font-size: var(--text-4);
  font-weight: 620;
  color: var(--text-primary);
}

.update-modal__body {
  margin: 0 0 var(--space-5);
  color: var(--text-secondary);
}

.update-modal__reload {
  padding: var(--space-3) var(--space-6);
  border: none;
  border-radius: var(--radius-md);
  background: var(--accent-solid);
  color: var(--text-on-accent-solid);
  font-family: var(--font-body);
  font-size: var(--text-2);
  font-weight: 600;
  cursor: pointer;
  transition: background-color var(--duration-micro) var(--ease-out);
}

.update-modal__reload:hover { background: var(--accent-solid-hover); }

.update-modal__reload:focus-visible {
  outline: 2px solid var(--focus-ring);
  outline-offset: 2px;
}
```

(No `@media (prefers-reduced-motion)` guard needed — the only motion here is the `background-color` transition on hover, not an entrance/exit animation. If a future change adds an open/close transition to `.update-modal-overlay`, gate that specific addition behind `prefers-reduced-motion`, matching `prefersReducedMotion()`'s existing use in `site.js`.)

- [x] **Step 2: Manually verify**

With the `docs-preview` server still running (see Task 3, Step 3):

1. Reload `http://localhost:8842/`. Trigger the modal the same way as Task 3 Step 3 (hand-edit `docs/version.txt`, wait up to 12s).
2. Confirm: modal is centered both horizontally and vertically in the viewport at both desktop and mobile widths (use the browser's device toolbar or resize the window).
3. Confirm the backdrop is visibly blurred (page content behind the overlay should be soft/indistinct, not sharp).
4. Toggle the site's light/dark theme control, reload, and re-trigger the modal in both themes — confirm text stays legible (dark text on the light "flour" theme's card, light text on the dark "crust" theme's card) and the Reload button's accent color and hover state both look correct.
5. Revert `docs/version.txt` afterward, same as Task 3 Step 3.

- [x] **Step 3: Run the full test suite one more time, then commit**

Run: `python3 scripts/run_tests.py`
Expected: all suites pass (this task doesn't touch Python, but confirms nothing else regressed).

```bash
git add docs/assets/styles.css
git commit -m "Style the update-check modal: centered, blurred, theme-aware

Reuses existing surface/border/radius/accent tokens rather than
introducing new ones, so it reads as part of the same design system
as the install-command panel, not a bolted-on component."
```
