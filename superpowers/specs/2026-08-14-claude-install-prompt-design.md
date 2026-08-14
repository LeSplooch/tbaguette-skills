# Claude install prompt — design

## Purpose

The homepage's install frame currently gives a human two shell one-liners
(macOS/Linux, Windows PowerShell) to paste into their own terminal. This
replaces that with a single prompt addressed to Claude instead of to a
shell — something a visitor pastes straight into a Claude Code conversation,
so Claude runs the install itself rather than the visitor hand-running a
command.

This isn't just a delivery-format swap. A raw shell one-liner is inert text
that either runs correctly or fails loudly (that's exactly what
`test_install_command.py` proves). A prompt handed to an LLM is interpreted,
which means it can be *more* robust than the one-liner — most importantly,
it can spell out what to do when the target directory exists but isn't a
git repo, a state this project has already seen in the wild (a local
install that was never a clone, silently breaking every future `git pull`
against it). The published one-liner just fails in that case, which is
safe but unhelpful. The prompt can name the situation explicitly and tell
Claude to stop and report it rather than improvise — still safe, but
no longer silent.

## Current state

`_render_install()` in `scripts/templates.py` builds an `install-frame`
containing a two-tab `[data-tabs]` widget (`_render_install_panel()` called
once per platform), each tab holding a `<code class="install__command">`
+ copy button pair. The two command strings are `INSTALL_COMMAND` (POSIX)
and `INSTALL_COMMAND_POWERSHELL`, both independently tested by
`test_install_command.py` across four collision scenarios and, where a
shell exists on the build machine, executed verbatim. Below the tabs, two
`install-frame__note` paragraphs cover (1) the safety guarantee with a link
to `/verify-install/`, and (2) the restart/reload step and the Claude
Code-only caveat.

## The install-frame markup

`_render_install()` is rewritten to emit one panel instead of two, and the
now-unused tab machinery around it is deleted rather than left in place:

- **Removed:** `_render_install_panel()`, `INSTALL_HINT_POSIX`,
  `INSTALL_HINT_POWERSHELL`, and the `.install-tabs` CSS rules in
  `styles.css` (the shared `.tabs`/`[data-tabs]` component itself stays —
  it's also used by formidable's stack/command tabs).
- **Added:** `INSTALL_PROMPT` (see below) and `INSTALL_PROMPT_HINT`, a
  single hint string: `"Paste into a Claude Code conversation — it reads
  your OS and runs the real commands itself."`
- The `<code>` block keeps the `install__command` class (already
  `white-space: pre-wrap; overflow-wrap: anywhere`, so multi-line content
  needs no CSS change) and a new id, `install-prompt-command`; the copy
  button's `data-copy-target` follows it. No changes needed to `site.js` —
  its copy logic already just reads `textContent` off whatever element
  `data-copy-target` names.
- `INSTALL_COMMAND`, `INSTALL_COMMAND_POWERSHELL`, and
  `INSTALL_COMMAND_CMD` are **not** touched or removed — they stay as the
  tested, shell-facing ground truth. `test_install_command.py` still
  imports and executes them; `README.md` still shows `INSTALL_COMMAND`
  verbatim for the GitHub-browsing audience. Only the *website's* rendering
  of them goes away.
- The two `install-frame__note` paragraphs are unchanged. They're still
  accurate, and the redundancy with the prompt's own step 6/7 (both the
  page and the prompt tell the visitor to restart/reload) is intentional —
  the note stays visible on the page regardless of what Claude actually
  relays back in chat.

## The prompt content

```
Install (or update) the TBaguette skills plugin for Claude Code. Use your shell tool:

1. Target directory: ~/.claude/skills/TBaguette (Windows: %USERPROFILE%\.claude\skills\TBaguette).
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
   chat load skills from my account instead of this folder.
```

Step-by-step rationale:

- **Steps 1–3** are the exact same clone-or-pull branch `INSTALL_COMMAND`
  already encodes and `test_install_command.py` already proves (scenarios
  A/B/C) — restated as prose instead of shell syntax so it reads correctly
  regardless of which shell Claude's tool actually runs, rather than
  picking one syntax (bash vs. PowerShell vs. cmd) and hoping it matches.
- **Step 4** is the one place this prompt does more than the one-liner:
  the raw `git clone` just refuses on a real collision (scenario D). Told
  only "install this," an agent could read a refusal as a problem to solve
  and reach for `rm -rf` on its own initiative. This step forecloses that
  explicitly, matching this project's own rule against unprompted deletion.
- **Step 5** turns "verified, not just claimed" — the existing note's own
  language — into something Claude actually does, not just a page visitors
  read.
- **Steps 6–7** restate the two existing notes so the information survives
  even if a visitor only reads what Claude says back, not the page itself.

## Keeping it in sync

Two layers, so "the prompt is still accurate" isn't a hope:

1. **Regression test** (`test_templates.py`): a new check that
   `INSTALL_PROMPT` and `INSTALL_COMMAND`/`INSTALL_COMMAND_POWERSHELL`
   agree — the repo URL and both target-path forms in the prompt must
   appear verbatim in the corresponding command constants. This is a
   same-file cross-check, not a hardcoded third copy in the test, so it
   fails the moment the two drift instead of trusting them to stay
   manually in sync.
2. **`tending-tbaguette`'s ship checklist** (the local-only meta-skill at
   `~/.claude/skills/tending-tbaguette/SKILL.md` — confirmed not part of
   this repo, so this edit is a direct file change with no commit) gets a
   new sweep instruction next to its existing "sweep for every other place
   the skill count is written down" step: when a candidate changes install
   semantics (target path, repo URL, branch, or the clone/pull logic
   itself), grep for every place that logic is duplicated *outside* what
   `run_tests.py` reaches. `README.md`'s hand-maintained install block is
   the first known instance of this gap — it mirrors `INSTALL_COMMAND`
   today with nothing enforcing that it keeps doing so.

## Testing

- **`test_templates.py`:** remove the now-obsolete dual-tab assertions
  (tab ids, `aria-selected`, both commands' escaped/unescaped presence,
  the POSIX/PowerShell hint text) and replace with: the frame label still
  reads "Install TBaguette's skills"; exactly one `install__command` block
  inside `install-frame`; `INSTALL_PROMPT` appears HTML-escaped and its raw
  form does not; the copy button's `data-copy-target` matches the block's
  id; both `install-frame__note` paragraphs are still present and in the
  same relative order as today. Plus the new cross-check described above.
- **Manual:** load the regenerated homepage, confirm the prompt block
  renders as legible multi-line text (not squashed to one scrolling line),
  and confirm the copy button copies the full multi-line text correctly —
  this project has no JS test tooling by design (stdlib-only), consistent
  with how the existing update-check modal was verified.

## Out of scope

- `README.md` — keeps showing the raw shell command; different audience
  (GitHub browsers comfortable running a command themselves), not touched
  by this change.
- `docs/verify-install/` — still documents/proves the shell command's
  safety; not extended to also narrate or prove the prompt.
- `INSTALL_COMMAND`, `INSTALL_COMMAND_POWERSHELL`, `INSTALL_COMMAND_CMD` —
  unchanged, still the tested shell-facing ground truth the prompt's own
  facts are checked against.
- No automated execution of the prompt through a live agent as part of the
  test suite — the regression test checks the facts embedded in the prompt
  text, not that a real Claude instance would behave correctly given it,
  matching this project's stdlib-only, deterministic testing philosophy.
