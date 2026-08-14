# Claude Install Prompt Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the homepage's two shell-command tabs (macOS/Linux, Windows PowerShell) with a single copy-able prompt addressed to Claude, so a visitor pastes it into a Claude Code conversation instead of hand-running git.

**Architecture:** `_render_install()` in `scripts/templates.py` currently builds a `[data-tabs]` widget around two `_render_install_panel()` calls. It becomes a single `.install` block (same `install__command`/`install__copy` classes, no tabs) rendering a new `INSTALL_PROMPT` constant. `INSTALL_COMMAND`/`INSTALL_COMMAND_POWERSHELL`/`INSTALL_COMMAND_CMD` are untouched — still the tested shell-facing ground truth, still what `README.md` shows and `test_install_command.py` executes. A new test cross-checks `INSTALL_PROMPT`'s embedded facts (repo URL, both target-path forms) against those constants so the two can't silently drift.

**Tech Stack:** Python 3 stdlib only (no template engine, no dependencies), plain HTML/CSS/JS for the generated site.

## Global Constraints

- `INSTALL_COMMAND`, `INSTALL_COMMAND_POWERSHELL`, `INSTALL_COMMAND_CMD` are not modified or removed — only the website's *rendering* of the first two goes away.
- `README.md` is not modified — it keeps showing `INSTALL_COMMAND` verbatim for the GitHub-browsing audience.
- `docs/verify-install/` is not modified.
- The shared `[data-tabs]`/`.tabs` component in `styles.css`/`site.js` is not modified or removed — it's also used by formidable's stack/command tabs (`scripts/templates.py:597`). Only this one *use* of it goes away.
- No new JS test tooling — this project has none by design (stdlib-only, zero install step). Verify JS/CSS changes manually in a browser.
- `python3 scripts/run_tests.py` must be fully green before any commit.
- Stage files by exact name when committing (`git add <path>`), never `git add -A` or `git add .`.
- Regenerate with `python3 scripts/generate.py --base-path /tbaguette-skills` any time `scripts/templates.py` changes, before committing.

---

### Task 1: Replace the two-tab install panel with a single Claude-facing prompt

**Files:**
- Modify: `scripts/templates.py:255-333` (everything from the end of `INSTALL_COMMAND_CMD` through the end of `_render_install`)
- Modify: `scripts/test_templates.py:15-23` (import block) and `:380-449` (install-frame checks inside `main()`)
- Modify: `docs/assets/styles.css` (the `.install-tabs` rule block, currently ~3 rules plus a comment, directly below `.install__hint`)
- Regenerated (not hand-edited): `docs/index.html`, `docs/version.txt` — produced by `scripts/generate.py`

**Interfaces:**
- Consumes: `escape_html()`, `_icon()` — both already defined earlier in `templates.py`, unchanged.
- Produces: `INSTALL_PROMPT: str` and `INSTALL_PROMPT_HINT: str` (new module-level constants in `templates.py`), consumed only by `_render_install()`. `_render_install(base_path: str = "") -> str` keeps its existing signature and single call site (`templates.py:347`, inside the hero renderer) — no caller changes needed.

- [ ] **Step 1: Write the failing test — replace the install-frame checks in `test_templates.py`**

First, update the import block near the top of the file (currently lines 15-23):

```python
from templates import (
    INSTALL_COMMAND,
    INSTALL_COMMAND_CMD,
    INSTALL_COMMAND_POWERSHELL,
    INSTALL_PROMPT,
    INSTALL_TEST_GITHUB_URL,
    escape_html,
    render_index,
    render_skill_page,
    render_verify_install_page,
)
```

(Only change: `INSTALL_PROMPT,` added, alphabetically between `INSTALL_COMMAND_POWERSHELL` and `INSTALL_TEST_GITHUB_URL`.)

Then replace the whole block from the `# Both commands contain && / {}` comment (currently line 380) through the `check("each platform panel names which shells/versions it covers", ...)` line (currently line 449) — everything between `check("has a search input", ...)` and `print(f"  wrote {index_path}")` — with:

```python
    # <target> inside the prompt is the one part of it that needs HTML
    # escaping (the angle brackets) — checking the escaped form here is a
    # real assertion, not a vacuous one, precisely because of that.
    check("install prompt appears, correctly HTML-escaped",
          escape_html(INSTALL_PROMPT) in index_html)
    check("raw, un-escaped install prompt never appears (would mean escaping "
          "broke, or a <target> placeholder leaked through as a real tag)",
          INSTALL_PROMPT not in index_html)
    check("install frame sits right after the headline, before the lede",
          index_html.index("hero__headline") < index_html.index('id="install-prompt-command"')
          < index_html.index("hero__lede"))
    check("has a copy button wired to the prompt",
          'data-copy-target="install-prompt-command"' in index_html)
    check("install frame is wrapped in its labeled frame",
          index_html.index("install-frame") < index_html.index("Install TBaguette")
          < index_html.index('id="install-prompt-command"'))
    label_start = index_html.index('install-frame__label')
    label_end = index_html.index('</p>', label_start)
    check("frame label itself carries an icon (icon-crust also appears in category "
          "headers elsewhere on the page, so this checks the label's own slice, not "
          "just presence anywhere)",
          '#icon-crust' in index_html[label_start:label_end])
    check("hint tells the visitor to paste this into Claude Code, not run it themselves",
          "Paste into a Claude Code conversation" in index_html)
    check("verification note sits after the prompt and before the lede, inside the frame",
          index_html.index('id="install-prompt-command"') < index_html.index("install-frame__note")
          < index_html.index("hero__lede"))
    check("verification note links to the on-site explanation page, base_path-prefixed",
          'href="/verify-install/"' in index_html)
    check("a second note tells visitors to restart/reload and how to invoke a skill",
          'Restart Claude Code' in index_html and 'TBaguette:skill-name' in index_html)
    check("that note clarifies this is Claude Code-specific, not Desktop app/claude.ai chat "
          "(the actual bug this was written to prevent: a visitor installs correctly but "
          "never sees the skills because they're looking in the wrong product)",
          'Claude Desktop app and claude.ai chat load skills from your account' in index_html)
    check("the Claude-Code-specific note sits after the safety note, before the lede",
          index_html.index("install-frame__note") < index_html.index('Restart Claude Code')
          < index_html.index("hero__lede"))
    check("no platform-picker tabs remain now that there's a single universal prompt",
          'data-autoselect-platform' not in index_html and 'tab-install-posix' not in index_html)

    # --- the prompt's embedded facts must not drift from the tested shell
    # commands (INSTALL_COMMAND covers POSIX + the repo URL, INSTALL_COMMAND_CMD
    # covers the %USERPROFILE%-style Windows path the prompt also uses) ---
    print("install prompt drift check")
    check("prompt's repo URL matches the tested POSIX install command",
          "https://github.com/LeSplooch/tbaguette-skills.git" in INSTALL_PROMPT
          and "https://github.com/LeSplooch/tbaguette-skills.git" in INSTALL_COMMAND)
    check("prompt's POSIX target path matches the tested POSIX install command",
          "~/.claude/skills/TBaguette" in INSTALL_PROMPT
          and "~/.claude/skills/TBaguette" in INSTALL_COMMAND)
    check("prompt's Windows target path matches the tested cmd.exe install command",
          "%USERPROFILE%\\.claude\\skills\\TBaguette" in INSTALL_PROMPT
          and "%USERPROFILE%\\.claude\\skills\\TBaguette" in INSTALL_COMMAND_CMD)
    print(f"  wrote {index_path}")
```

- [ ] **Step 2: Run it to verify it fails**

Run: `python3 scripts/test_templates.py`
Expected: `ImportError: cannot import name 'INSTALL_PROMPT' from 'templates'` (it doesn't exist yet).

- [ ] **Step 3: Implement — rewrite the install-frame renderer in `templates.py`**

Replace `INSTALL_HINT_POSIX` and `INSTALL_HINT_POWERSHELL` (currently lines 262-263) with:

```python
INSTALL_PROMPT_HINT = "Paste into a Claude Code conversation — it reads your OS and runs the real commands itself."

# Addressed to Claude, not to a shell — this is what a visitor pastes into a
# Claude Code conversation instead of running a command themselves. Steps
# 1-3 restate the exact clone-or-pull branch INSTALL_COMMAND already encodes
# and test_install_command.py already proves (scenarios A/B/C), just as
# prose instead of one shell's syntax, so it reads correctly no matter which
# shell Claude's tool actually runs. Step 4 is the one place this does more
# than the one-liner: a bare `git clone` just refuses on a real collision
# (scenario D) — told only "install this," an agent could read that refusal
# as a problem to solve and reach for rm -rf on its own initiative, so this
# spells out the refusal explicitly instead of leaving it implicit.
INSTALL_PROMPT = """Install (or update) the TBaguette skills plugin for Claude Code. Use your shell tool:

1. Target directory: ~/.claude/skills/TBaguette (Windows: %USERPROFILE%\\.claude\\skills\\TBaguette).
2. If <target>/.git exists, update in place: git -C <target> pull.
3. Else if <target> doesn't exist, or exists and is empty, install fresh:
   git clone https://github.com/LeSplooch/tbaguette-skills.git <target>.
4. Else (the directory exists, has content, and is not a git repo) — stop.
   Do not delete or modify it. Tell me there's a naming collision at that
   path that needs a manual look.
5. After a successful clone or pull, confirm <target>/CATALOG.md and
   <target>/skills/ both exist, so "it worked" is checked, not assumed.
6. Tell me to restart Claude Code (or run /reload-plugins) — skills then
   invoke as TBaguette:skill-name.
7. This only works for Claude Code. The Claude Desktop app and claude.ai
   chat load skills from my account instead of this folder."""
```

Delete `_render_install_panel()` entirely (currently lines 266-286, the whole function from `def _render_install_panel(` through its closing `</div>"""` and the blank lines before `def _render_install`).

Replace `_render_install()` (currently lines 287-333) with:

```python
def _render_install(base_path: str = "") -> str:
    escaped_prompt = escape_html(INSTALL_PROMPT)
    return f"""<div class="install-frame">
  <p class="install-frame__label">
    {_icon("icon-crust", base_path=base_path)}
    Install TBaguette&rsquo;s skills
  </p>
  <div class="install-frame__body">
    <div class="install">
      <code class="install__command" id="install-prompt-command">{escaped_prompt}</code>
      <button class="install__copy" type="button" data-copy-target="install-prompt-command"
              aria-label="Copy install prompt">
        <span class="install__copy-icons">
          <svg class="icon install__copy-icon install__copy-icon--copy" aria-hidden="true"><use href="{base_path}/assets/icons.svg#icon-copy"></use></svg>
          <svg class="icon install__copy-icon install__copy-icon--check" aria-hidden="true"><use href="{base_path}/assets/icons.svg#icon-check"></use></svg>
        </span>
        <span data-copy-label>Copy</span>
      </button>
    </div>
    <p class="install__hint">{escape_html(INSTALL_PROMPT_HINT)}</p>
    <p class="install-frame__note">
      {_icon("icon-check", base_path=base_path)}
      <span>Only ever touches this folder — verified against your other skills, not
      just claimed. <a href="{base_path}/verify-install/">See how</a>.</span>
    </p>
    <p class="install-frame__note">
      {_icon("icon-check", base_path=base_path)}
      <span>Restart Claude Code (or run <code>/reload-plugins</code>), then invoke a
      skill as <code>TBaguette:skill-name</code>. This is for Claude Code specifically —
      the Claude Desktop app and claude.ai chat load skills from your account instead of
      this folder, so cloning here won't make them appear there.</span>
    </p>
  </div>
</div>"""
```

- [ ] **Step 4: Run it to verify it passes**

Run: `python3 scripts/test_templates.py`
Expected: every `check(...)` prints `  ok  ...`, ends with `N checks passed.` and no traceback.

- [ ] **Step 5: Remove the now-dead `.install-tabs` CSS**

In `docs/assets/styles.css`, delete this comment + rule block (directly after `.install__hint { ... }`):

```css
/* The platform-picker tabs reuse .tabs/.tabs__tab/.tabs__panel wholesale
   (same component as formidable's Stacks/Commands) but sit inside a small,
   already-padded card rather than a full article — .tabs__panels' prose
   sizing (measure-prose max-width, text-3) is the wrong fit here, since a
   panel holds only a one-line command + hint, not running text. */
.install-tabs { margin-top: 0; }
.install-tabs .tabs__panels { margin-top: var(--space-4); }
.install-tabs .tabs__panel { max-width: none; font-size: inherit; line-height: inherit; }
```

Leave everything else in the file untouched — `.tabs`/`.tabs__tab`/`.tabs__panel` (the base, unscoped classes) stay, they're still used by formidable's stack/command tabs.

- [ ] **Step 6: Run the full test suite**

Run: `python3 scripts/run_tests.py`
Expected: all 5 suites pass (this also runs `test_generate.py`, `test_install_command.py`, `test_content_pipeline.py`, `test_python_highlight.py` — none of which reference the install-frame markup, so they should be unaffected, but this is the check that proves it).

- [ ] **Step 7: Regenerate the site**

Run: `python3 scripts/generate.py --base-path /tbaguette-skills`
Then: `git status --short docs/` and `git diff --stat docs/`
Expected: `docs/index.html` changed, `docs/version.txt` changed (timestamp — expected on every regeneration, per the auto-update-check feature). No skill page under `docs/skills/` and no other file should show a diff. If anything else changed, stop and investigate before continuing — that's a sign of an unrelated regression.

- [ ] **Step 8: Verify in a browser**

Start a local server over `docs/` (or use the preview tool) and load the homepage. Confirm:
- The install frame shows one multi-line prompt block, not a tab picker.
- The text is fully legible (wraps, isn't clipped or forced onto one scrolling line).
- Clicking the copy button copies the complete prompt text (paste it somewhere to check) and the button shows its "copied" state.
- The copy button doesn't look visually broken next to a 7-line block (`.install` uses `align-items: stretch` by default, which will make the button match the block's height — confirm this reads fine; if the button looks awkwardly tall/narrow, add `align-items: flex-start` to `.install` in `styles.css`, then repeat Step 6 and Step 7).

- [ ] **Step 9: Commit**

```bash
git add scripts/templates.py scripts/test_templates.py docs/assets/styles.css docs/index.html docs/version.txt
git commit -m "$(cat <<'EOF'
Replace the install command tabs with a single Claude-facing prompt

Visitors now paste one prompt into a Claude Code conversation instead of
hand-running a shell one-liner. The prompt restates the same clone-or-pull
logic INSTALL_COMMAND already encodes and test_install_command.py already
proves, plus an explicit refusal (not a silent one) on a real directory
collision. A new regression test keeps the prompt's embedded facts pinned
to the tested shell commands so the two can't drift apart.
EOF
)"
git push origin master
```

(If the pre-commit hook regenerates `docs/` again and reports additional staged changes, that's expected — let it run, then verify `python3 scripts/run_tests.py` is still green before considering this task done.)

---

### Task 2: Point the local `tending-tbaguette` ship checklist at install-semantics drift

This is a local-only file, not part of the `tbaguette-skills` repo — no git commit, no tests, no regenerate.

**Files:**
- Modify: `~/.claude/skills/tending-tbaguette/SKILL.md` (step 3 of "Ship procedure", currently the bullet list starting "Make the edit:")

**Interfaces:** None — this is prose in a skill file, not code.

- [ ] **Step 1: Add the sweep instruction**

In `~/.claude/skills/tending-tbaguette/SKILL.md`, inside step 3 ("Make the edit:"), after the existing `- **New skill**:` bullet block (which ends with the `.claude-plugin/plugin.json` version-bump line) and before step 4 ("Quality gate:"), add a new sibling bullet:

```markdown
   - **Either way, if this candidate changes install semantics** (the
     target directory, the repo URL, the branch, or the clone/pull logic
     itself) — rare, but when it happens: grep for every place that logic
     is duplicated outside what `run_tests.py` reaches. `README.md`'s
     hand-maintained install block is the first known instance — it
     mirrors `scripts/templates.py`'s `INSTALL_COMMAND` with nothing
     enforcing that it stays in sync. The website's Claude-facing install
     prompt (`INSTALL_PROMPT`, same file) *is* covered by `run_tests.py`'s
     own drift check as of 2026-08-14 — a documentation-only copy like the
     README never will be, since nothing renders it.
```

- [ ] **Step 2: Confirm the edit landed**

Read the file back (or trust the Edit tool's success) and confirm the new bullet sits between the "New skill" block and step 4, at the same indentation level as the other step-3 sub-bullets.
