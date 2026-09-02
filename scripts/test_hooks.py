"""Tests for the TBaguette plugin's hooks: hooks/hooks.json's and
hooks/hooks-copilot.json's declared shapes, hooks/user-prompt-submit's
per-turn skill-check nudge in both of its output modes, and
hooks/session-start's actual behavior -- that it emits
well-formed additionalContext carrying using-tbaguette's SKILL.md verbatim,
and that keeping-tbaguette-current's step 1 (fetch, compare, working-tree
status) reports correctly across the four states that path can be in.

The update-check scenarios run against throwaway local git repos (a fake
origin and a fake clone of it, both under a temp directory, connected by a
file:// path) rather than the real published repo -- no network dependency,
so this suite runs offline and isn't subject to GitHub being reachable, the
way test_install_command.py's real-clone scenarios are.

Plain assert-based, using the shared Checker (see scripts/checker.py) --
matches test_generate.py, test_templates.py, and test_i18n.py's convention,
not test_content_pipeline.py's unittest one.

Usage:
    python3 scripts/test_hooks.py
"""

from __future__ import annotations

import json
import os
import shutil
import subprocess
import tempfile
from pathlib import Path

from checker import Checker

checker = Checker()
check = checker.check

REPO_ROOT = Path(__file__).resolve().parent.parent
HOOKS_DIR = REPO_ROOT / "hooks"
SESSION_START = HOOKS_DIR / "session-start"
USER_PROMPT_SUBMIT = HOOKS_DIR / "user-prompt-submit"
RUN_HOOK_CMD = HOOKS_DIR / "run-hook.cmd"
HOOKS_COPILOT_JSON = HOOKS_DIR / "hooks-copilot.json"
HOOKS_VSCODE_JSON = REPO_ROOT / "com.github.copilot" / "hooks" / "hooks.json"
HOOKS_CURSOR_JSON = HOOKS_DIR / "hooks-cursor.json"

USING_TBAGUETTE_FIXTURE = (
    "---\nname: using-tbaguette\ndescription: fixture for test_hooks.py\n---\n\n"
    "# using-tbaguette (fixture)\n"
)


# ---------------------------------------------------------------------------
# Fixture helpers
# ---------------------------------------------------------------------------


def install_hook_scripts(root: Path) -> None:
    """Copies session-start and run-hook.cmd into a fixture plugin root.

    Both scripts locate their own plugin root via $0 (dirname of the
    running script), not any environment variable -- deliberately, since
    that's what makes them correct in the real deployment regardless of
    what CLAUDE_PLUGIN_ROOT happens to be set to. It also means a fixture
    needs its own physical copy of the scripts to actually exercise: running
    the repo's real hooks/session-start with an env var pointed elsewhere
    would just re-resolve back to the repo's own real skills/ and .git.
    """
    hooks_dir = root / "hooks"
    hooks_dir.mkdir(parents=True, exist_ok=True)
    for name in ("session-start", "run-hook.cmd"):
        dest = hooks_dir / name
        shutil.copy2(HOOKS_DIR / name, dest)
        dest.chmod(dest.stat().st_mode | 0o111)


def make_plugin_root(parent: Path, name: str) -> Path:
    """A directory shaped like a real plugin root: the one file session-start
    unconditionally reads, plus its own copy of the hook scripts. No .git."""
    root = parent / name
    skill_dir = root / "skills" / "using-tbaguette"
    skill_dir.mkdir(parents=True)
    (skill_dir / "SKILL.md").write_text(USING_TBAGUETTE_FIXTURE, encoding="utf-8")
    install_hook_scripts(root)
    return root


def git(args: list[str], cwd: Path) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["git", *args], cwd=str(cwd), check=True, capture_output=True, text=True
    )


def init_origin(parent: Path, name: str) -> Path:
    """A throwaway git repo standing in for the published repo -- branch
    named "master" explicitly, since session-start's fetch/compare hardcodes
    that name the same way keeping-tbaguette-current's own instructions do.
    Commits the hook scripts along with everything else, so a later `git
    clone` carries a working copy of them into the clone for free."""
    origin = make_plugin_root(parent, name)
    git(["init", "--quiet", "--initial-branch=master"], origin)
    git(["config", "user.email", "test@example.com"], origin)
    git(["config", "user.name", "Test"], origin)
    git(["add", "-A"], origin)
    git(["commit", "--quiet", "-m", "initial"], origin)
    return origin


def clone(origin: Path, dest: Path) -> None:
    subprocess.run(
        ["git", "clone", "--quiet", str(origin), str(dest)],
        check=True, capture_output=True, text=True,
    )


def run_session_start(plugin_root: Path, output_format: str | None = None) -> dict:
    script = plugin_root / "hooks" / "session-start"
    argv = ["bash", str(script)]
    if output_format is not None:
        argv.append(output_format)
    result = subprocess.run(
        argv, capture_output=True, text=True, check=False,
    )
    check(f"session-start exits 0 (root={plugin_root.name})", result.returncode == 0)
    if result.returncode != 0:
        print(result.stderr)
    return json.loads(result.stdout)


def context_of(payload: dict) -> str:
    return payload["hookSpecificOutput"]["additionalContext"]


# ---------------------------------------------------------------------------
# hooks.json shape
# ---------------------------------------------------------------------------


def check_hooks_json_shape() -> None:
    print("hooks.json shape")
    data = json.loads((HOOKS_DIR / "hooks.json").read_text(encoding="utf-8"))
    session_start_matchers = data.get("hooks", {}).get("SessionStart", [])
    check("declares exactly one SessionStart matcher group", len(session_start_matchers) == 1)

    matcher_group = session_start_matchers[0]
    check(
        "matcher covers startup, clear, and compact",
        matcher_group.get("matcher") == "startup|clear|compact",
    )

    inner_hooks = matcher_group.get("hooks", [])
    check("exactly one hook command in the group", len(inner_hooks) == 1)

    command = inner_hooks[0]
    check("hook type is command", command.get("type") == "command")
    check("runs through bash", command.get("shell") == "bash")
    check("not async", command.get("async") is False)
    command_str = command.get("command", "")
    check("invokes run-hook.cmd via CLAUDE_PLUGIN_ROOT", "${CLAUDE_PLUGIN_ROOT}/hooks/run-hook.cmd" in command_str)
    check("passes session-start as the script name", "session-start" in command_str)

    # UserPromptSubmit: the per-turn re-assertion of using-tbaguette's rule.
    # SessionStart alone measurably decays -- long sessions would invoke one
    # TBaguette skill or none -- so this group existing is the fix, and a
    # regression that silently dropped it would restore the original bug.
    prompt_groups = data.get("hooks", {}).get("UserPromptSubmit", [])
    check("declares exactly one UserPromptSubmit matcher group", len(prompt_groups) == 1)

    prompt_hooks = prompt_groups[0].get("hooks", []) if prompt_groups else []
    check("exactly one UserPromptSubmit command", len(prompt_hooks) == 1)

    prompt_command = prompt_hooks[0] if prompt_hooks else {}
    check("UserPromptSubmit hook type is command", prompt_command.get("type") == "command")
    check("UserPromptSubmit runs through bash", prompt_command.get("shell") == "bash")
    check(
        "UserPromptSubmit is not async (additionalContext must land before the turn)",
        prompt_command.get("async") is False,
    )
    prompt_command_str = prompt_command.get("command", "")
    check(
        "UserPromptSubmit invokes run-hook.cmd via CLAUDE_PLUGIN_ROOT",
        "${CLAUDE_PLUGIN_ROOT}/hooks/run-hook.cmd" in prompt_command_str,
    )
    check("passes user-prompt-submit as the script name", "user-prompt-submit" in prompt_command_str)


# ---------------------------------------------------------------------------
# File permissions
# ---------------------------------------------------------------------------


def check_executable_bits() -> None:
    print("executable bits")
    check("session-start is executable", os.access(SESSION_START, os.X_OK))
    check("run-hook.cmd is executable", os.access(RUN_HOOK_CMD, os.X_OK))
    check("user-prompt-submit is executable", os.access(USER_PROMPT_SUBMIT, os.X_OK))


# ---------------------------------------------------------------------------
# Basic JSON shape and using-tbaguette content injection
# ---------------------------------------------------------------------------


def check_session_start_basic_shape() -> None:
    print("session-start: JSON shape and using-tbaguette content")
    with tempfile.TemporaryDirectory() as tmp:
        plugin_root = make_plugin_root(Path(tmp), "plugin")
        payload = run_session_start(plugin_root)

        check("top-level key is hookSpecificOutput", list(payload.keys()) == ["hookSpecificOutput"])
        hook_output = payload["hookSpecificOutput"]
        check("hookEventName is SessionStart", hook_output.get("hookEventName") == "SessionStart")

        ctx = hook_output.get("additionalContext", "")
        check(
            # $(cat ...) strips trailing newlines (bash command substitution,
            # not a session-start bug) -- compare with that same trim applied
            # to the expected side.
            "additionalContext contains using-tbaguette's SKILL.md verbatim",
            USING_TBAGUETTE_FIXTURE.rstrip("\n") in ctx,
        )
        check(
            "additionalContext flags a plugin root with no .git as not a clone",
            "not a git clone" in ctx,
        )


def check_run_hook_cmd_unix_passthrough() -> None:
    print("run-hook.cmd: Unix passthrough")
    with tempfile.TemporaryDirectory() as tmp:
        plugin_root = make_plugin_root(Path(tmp), "plugin")
        run_hook_cmd = plugin_root / "hooks" / "run-hook.cmd"
        result = subprocess.run(
            ["bash", str(run_hook_cmd), "session-start"],
            capture_output=True, text=True, check=False,
        )
        check("run-hook.cmd exits 0", result.returncode == 0)
        payload = json.loads(result.stdout)
        check(
            "output matches session-start's own shape",
            payload["hookSpecificOutput"]["hookEventName"] == "SessionStart",
        )


# ---------------------------------------------------------------------------
# keeping-tbaguette-current step 1, across all four states
# ---------------------------------------------------------------------------


def check_update_check_same_sha() -> None:
    print("update check: clone matches origin (up to date)")
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        origin = init_origin(tmp_path, "origin")
        cloned = tmp_path / "clone"
        clone(origin, cloned)

        ctx = context_of(run_session_start(cloned))
        check("reports same (up to date)", "same (up to date)" in ctx)
        # A bare "dirty" substring search would false-fail here: the
        # instructional prose always mentions "dirty" as one of the two
        # branches it explains, regardless of actual tree state. Anchor on
        # the specific status line instead.
        check("reports the working tree as clean", "- Working tree: clean" in ctx)
        check("tells Claude not to re-fetch", "do not re-fetch" in ctx)


def check_update_check_update_available() -> None:
    print("update check: origin has moved ahead (update available)")
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        origin = init_origin(tmp_path, "origin")
        cloned = tmp_path / "clone"
        clone(origin, cloned)

        (origin / "README.md").write_text("origin content, updated\n", encoding="utf-8")
        git(["add", "-A"], origin)  # README.md is new, not tracked -- `-a` alone wouldn't pick it up
        git(["commit", "--quiet", "-m", "second commit"], origin)
        origin_head = git(["rev-parse", "HEAD"], origin).stdout.strip()

        ctx = context_of(run_session_start(cloned))
        check("reports different (update available)", "different (update available)" in ctx)
        check("reports the new origin/master SHA", origin_head in ctx)
        check("tells Claude to follow the remaining steps", "remaining steps" in ctx)


def check_update_check_dirty_tree() -> None:
    print("update check: local changes present (dirty)")
    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        origin = init_origin(tmp_path, "origin")
        cloned = tmp_path / "clone"
        clone(origin, cloned)
        (cloned / "README.md").write_text("hand-edited\n", encoding="utf-8")

        ctx = context_of(run_session_start(cloned))
        check("reports dirty even though HEAD still matches origin", "dirty (local changes present)" in ctx)
        check("still reports the SHA comparison alongside it", "same (up to date)" in ctx)


def check_update_check_no_git() -> None:
    print("update check: not a git clone")
    with tempfile.TemporaryDirectory() as tmp:
        plugin_root = make_plugin_root(Path(tmp), "plugin")
        ctx = context_of(run_session_start(plugin_root))
        check("reports not a git clone, nothing to do", "not a git clone" in ctx)
        check("no SHA-comparison language leaks in", "up to date" not in ctx and "update available" not in ctx)


# ---------------------------------------------------------------------------
# UserPromptSubmit: the per-turn skill-check nudge
# ---------------------------------------------------------------------------


def check_user_prompt_submit() -> None:
    print("user-prompt-submit: per-turn skill-check nudge")
    # Unlike session-start, this script reads nothing from its plugin root --
    # the nudge is static text -- so the repo's real copy is what runs here,
    # no fixture root needed.
    result = subprocess.run(
        ["bash", str(USER_PROMPT_SUBMIT)],
        capture_output=True,
        text=True,
    )
    check("user-prompt-submit exits 0", result.returncode == 0)
    check("writes nothing to stderr", result.stderr == "")

    payload = json.loads(result.stdout)
    hook_output = payload.get("hookSpecificOutput", {})
    check("top-level key is hookSpecificOutput", "hookSpecificOutput" in payload)
    check("hookEventName is UserPromptSubmit", hook_output.get("hookEventName") == "UserPromptSubmit")

    ctx = hook_output.get("additionalContext", "")
    check("names the Skill tool invocation form", "TBaguette:<skill-name>" in ctx)
    check("points back at using-tbaguette for the full rule", "using-tbaguette" in ctx)
    # The two rationalizations that the measured misses ran on: treating a
    # question as not-a-task, and treating small work as beneath a skill.
    check("closes the 'just a question' loophole", "Questions" in ctx)
    check("closes the 'too small' loophole", "too small" in ctx)
    # This fires on every single user turn, so its size is a recurring cost,
    # not a one-off. A ceiling here is what keeps a well-meaning edit from
    # quietly turning a nudge back into a full SKILL.md re-injection.
    check(f"stays short enough for per-turn cost ({len(ctx)} bytes)", len(ctx) < 600)

    wrapped = subprocess.run(
        ["bash", str(RUN_HOOK_CMD), "user-prompt-submit"],
        capture_output=True,
        text=True,
    )
    check("run-hook.cmd dispatches to it", wrapped.returncode == 0)
    check(
        "wrapper output matches the script's own",
        json.loads(wrapped.stdout) == payload,
    )


# ---------------------------------------------------------------------------
# Copilot CLI: a second hook config, and the same two scripts in a second mode
# ---------------------------------------------------------------------------


def check_hooks_copilot_json_shape() -> None:
    print("hooks-copilot.json shape")
    data = json.loads(HOOKS_COPILOT_JSON.read_text(encoding="utf-8"))
    # Copilot CLI requires this and requires it to be 1; Claude Code's own
    # hooks.json has no such key, which is the shortest proof the two files
    # are not interchangeable however similar they look.
    check("declares version 1", data.get("version") == 1)

    hooks = data.get("hooks", {})
    check("uses Copilot's camelCase event names, not Claude Code's PascalCase",
          set(hooks) == {"sessionStart", "userPromptSubmitted"})

    for event, script in (("sessionStart", "session-start"),
                          ("userPromptSubmitted", "user-prompt-submit")):
        entries = hooks.get(event, [])
        check(f"{event} declares exactly one hook", len(entries) == 1)
        entry = entries[0] if entries else {}
        check(f"{event} type is command", entry.get("type") == "command")
        # Both are required, not belt-and-braces: Copilot picks by platform, so
        # a missing powershell key is a Windows install with no bootstrap.
        for shell in ("bash", "powershell"):
            command = entry.get(shell, "")
            check(f"{event} declares a {shell} command", bool(command))
            check(f"{event}'s {shell} command runs run-hook.cmd",
                  "run-hook.cmd" in command)
            check(f"{event}'s {shell} command names {script}",
                  script in command)
            # The argument is the entire difference between emitting Copilot's
            # envelope and emitting Claude Code's. Drop it and the hook still
            # runs, still exits 0, and delivers nothing Copilot can read --
            # a silent no-op, which is the worst shape a bootstrap failure
            # can take.
            check(f"{event}'s {shell} command passes the copilot format argument",
                  command.rstrip().endswith("copilot"))
            check(f"{event}'s {shell} command resolves the plugin root",
                  "PLUGIN_ROOT" in command)
        check(f"{event} bounds its runtime", isinstance(entry.get("timeoutSec"), int))


def check_session_start_copilot_shape() -> None:
    print("session-start: Copilot output envelope")
    with tempfile.TemporaryDirectory() as tmp:
        plugin_root = make_plugin_root(Path(tmp), "plugin")
        payload = run_session_start(plugin_root, "copilot")

        # Copilot's sessionStart reads a bare additionalContext. Claude Code's
        # nesting would parse as JSON and carry nothing.
        check("top-level key is additionalContext, unnested",
              list(payload.keys()) == ["additionalContext"])
        ctx = payload["additionalContext"]
        check("carries using-tbaguette's SKILL.md verbatim",
              USING_TBAGUETTE_FIXTURE.rstrip("\n") in ctx)
        check("still carries the update-check block",
              "not a git clone" in ctx)
        # The one sentence that is genuinely per-harness. Copilot has no Skill
        # tool, so naming one here would send the model looking for something
        # it cannot find, on the line whose whole job is reaching the other 95
        # skills.
        check("names the slash-command form Copilot actually has",
              "/TBaguette:<skill-name>" in ctx)
        check("does not name a Skill tool Copilot does not have",
              "use the 'Skill' tool" not in ctx)
        check("points at the Copilot tool mapping",
              "references/copilot-tools.md" in ctx)


def run_prompt_hook(stdin: str) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["bash", str(USER_PROMPT_SUBMIT), "copilot"],
        input=stdin, capture_output=True, text=True, check=False,
    )


def check_user_prompt_submit_copilot() -> None:
    print("user-prompt-submit: Copilot modifiedPrompt mode")
    prompt = "fix the login bug"
    result = run_prompt_hook(json.dumps({
        "sessionId": "s", "timestamp": 0, "cwd": "/tmp", "prompt": prompt,
    }))
    check("exits 0", result.returncode == 0)
    payload = json.loads(result.stdout)
    check("returns modifiedPrompt, the only field this event accepts",
          list(payload.keys()) == ["modifiedPrompt"])
    modified = payload["modifiedPrompt"]
    check("the user's own prompt survives, last and intact",
          modified.endswith(prompt))
    check("the nudge is prepended, delimited so it does not read as the user",
          modified.startswith("<TBAGUETTE_SKILL_CHECK>"))
    check("nudge names Copilot's invocation form", "/TBaguette:<skill-name>" in modified)
    check("nudge does not name Claude Code's Skill tool", "Skill tool" not in modified)
    check("closes the 'just a question' loophole", "Questions" in modified)
    check("closes the 'too small' loophole", "too small" in modified)

    # This is the assertion the whole branch exists for. Unlike Claude Code's
    # additionalContext, this hook holds the user's prompt in its hands: a
    # quoting bug here does not weaken a reminder, it rewrites what the person
    # typed. Anything a real prompt can contain has to come back byte for byte.
    nasty = 'say "hi"\nline 2\ttabbed\\backslash C:\\Users é 🥖 {"json":true}'
    payload = json.loads(run_prompt_hook(json.dumps({"prompt": nasty})).stdout)
    check("a prompt full of quotes, newlines, tabs, backslashes, unicode and "
          "JSON round-trips byte for byte",
          payload["modifiedPrompt"].endswith(nasty))

    # Every way this can go wrong ends the same way: no modification. Copilot
    # reads {} as "leave the prompt alone", so a lost nudge costs one turn's
    # reminder and nothing else.
    for label, stdin in (
        ("stdin is not JSON at all", "not json"),
        ("stdin is empty", ""),
        ("payload carries no prompt", '{"sessionId": "s"}'),
        ("prompt is not a string", '{"prompt": 42}'),
        ("prompt is null", '{"prompt": null}'),
        ("stdin is JSON but not an object", '["prompt"]'),
    ):
        result = run_prompt_hook(stdin)
        check(f"{label}: exits 0", result.returncode == 0)
        check(f"{label}: emits an empty object, modifying nothing",
              json.loads(result.stdout) == {})

    wrapped = subprocess.run(
        ["bash", str(RUN_HOOK_CMD), "user-prompt-submit", "copilot"],
        input=json.dumps({"prompt": prompt}), capture_output=True, text=True,
    )
    check("run-hook.cmd forwards the format argument and stdin both",
          json.loads(wrapped.stdout)["modifiedPrompt"].endswith(prompt))


def check_hooks_vscode_json_shape() -> None:
    print("com.github.copilot/hooks/hooks.json shape")
    data = json.loads(HOOKS_VSCODE_JSON.read_text(encoding="utf-8"))
    check("declares version 1", data.get("version") == 1)

    hooks = data.get("hooks", {})
    # VS Code publishes PascalCase event names and, unlike the CLI, will not
    # be told where its hook file is -- it derives the path from the manifest's
    # format. So this file is only ever reached as VS Code, and only ever
    # needs VS Code's spelling.
    check("uses the PascalCase names VS Code documents",
          set(hooks) == {"SessionStart", "UserPromptSubmit"})

    for event, script in (("SessionStart", "session-start"),
                          ("UserPromptSubmit", "user-prompt-submit")):
        entries = hooks.get(event, [])
        check(f"{event} declares exactly one hook", len(entries) == 1)
        entry = entries[0] if entries else {}
        check(f"{event} type is command", entry.get("type") == "command")
        for shell in ("bash", "powershell"):
            command = entry.get(shell, "")
            check(f"{event} declares a {shell} command", bool(command))
            check(f"{event}'s {shell} command names {script}", script in command)
            check(f"{event}'s {shell} command passes the vscode format argument",
                  command.rstrip().endswith("vscode"))
            # VS Code is documented as setting CLAUDE_PLUGIN_ROOT, so it is
            # worth falling back through on the surface where PLUGIN_ROOT is
            # least certain.
            check(f"{event}'s {shell} command falls back through CLAUDE_PLUGIN_ROOT",
                  "CLAUDE_PLUGIN_ROOT" in command)
        check(f"{event} bounds its runtime", isinstance(entry.get("timeoutSec"), int))


def check_session_start_vscode_shape() -> None:
    print("session-start: VS Code output envelope")
    with tempfile.TemporaryDirectory() as tmp:
        plugin_root = make_plugin_root(Path(tmp), "plugin")
        payload = run_session_start(plugin_root, "vscode")

        # VS Code names its events the Claude Code way but runs Copilot's hook
        # engine, and publishes nothing about which stdout shape it reads. Both
        # are emitted so either reader finds its key. Injection happens once per
        # session, so the duplicated payload costs a session, not a turn.
        check("emits both envelopes rather than betting on one",
              sorted(payload.keys()) == ["additionalContext", "hookSpecificOutput"])
        check("hookEventName is VS Code's PascalCase spelling",
              payload["hookSpecificOutput"].get("hookEventName") == "SessionStart")
        check("the two envelopes carry the same context, not two versions of it",
              payload["additionalContext"] == payload["hookSpecificOutput"]["additionalContext"])

        ctx = payload["additionalContext"]
        check("carries using-tbaguette's SKILL.md verbatim",
              USING_TBAGUETTE_FIXTURE.rstrip("\n") in ctx)
        check("names the slash-command form, not a Skill tool",
              "/TBaguette:<skill-name>" in ctx and "use the 'Skill' tool" not in ctx)


def check_user_prompt_submit_vscode() -> None:
    print("user-prompt-submit: VS Code mode")
    result = subprocess.run(
        ["bash", str(USER_PROMPT_SUBMIT), "vscode"],
        input=json.dumps({"prompt": "anything"}),
        capture_output=True, text=True, check=False,
    )
    check("exits 0", result.returncode == 0)
    payload = json.loads(result.stdout)

    # The asymmetry with session-start is deliberate and is the point of this
    # check. There, emitting both shapes is free. Here, a reader honoring both
    # would rewrite the user's prompt AND add the nudge -- so this branch takes
    # only the shape that cannot touch the prompt. If the guess is wrong the
    # nudge is lost, which is what every failure path here already costs.
    check("emits only the non-destructive envelope",
          list(payload.keys()) == ["hookSpecificOutput"])
    check("never returns modifiedPrompt on this surface",
          "modifiedPrompt" not in result.stdout)
    hook_output = payload["hookSpecificOutput"]
    check("hookEventName is UserPromptSubmit",
          hook_output.get("hookEventName") == "UserPromptSubmit")

    ctx = hook_output.get("additionalContext", "")
    check("carries Copilot's nudge, not Claude Code's",
          "/TBaguette:<skill-name>" in ctx and "Skill tool" not in ctx)
    check("closes the 'just a question' loophole", "Questions" in ctx)
    check("closes the 'too small' loophole", "too small" in ctx)
    check(f"stays short enough for per-turn cost ({len(ctx)} bytes)", len(ctx) < 600)


def check_hooks_cursor_json_shape() -> None:
    print("hooks-cursor.json shape")
    data = json.loads(HOOKS_CURSOR_JSON.read_text(encoding="utf-8"))
    check("declares version 1", data.get("version") == 1)

    hooks = data.get("hooks", {})
    # postToolUse, not beforeSubmitPrompt. Cursor has a per-prompt hook and its
    # output schema is {continue, user_message} -- `continue` gates the turn and
    # `user_message` is addressed to the human. There is no door to the model on
    # that event at all, and only sessionStart and postToolUse accept
    # additional_context. Wiring the per-prompt one would look right in a diff
    # and deliver nothing.
    check("uses the two events that actually accept additional_context",
          set(hooks) == {"sessionStart", "postToolUse"})

    for event, script, fmt in (("sessionStart", "session-start", "cursor"),
                               ("postToolUse", "user-prompt-submit", "cursor")):
        entries = hooks.get(event, [])
        check(f"{event} declares exactly one hook", len(entries) == 1)
        entry = entries[0] if entries else {}
        command = entry.get("command", "")
        check(f"{event} runs run-hook.cmd", "run-hook.cmd" in command)
        check(f"{event} names {script}", script in command)
        # Without the argument this emits Claude Code's envelope, which is
        # exactly the bug that left Cursor with a hook that ran and delivered
        # nothing for as long as this integration has existed.
        check(f"{event} passes the cursor format argument",
              command.rstrip().endswith(fmt))
        check(f"{event} bounds its runtime", isinstance(entry.get("timeout"), int))


def check_session_start_cursor_shape() -> None:
    print("session-start: Cursor output envelope")
    with tempfile.TemporaryDirectory() as tmp:
        plugin_root = make_plugin_root(Path(tmp), "plugin")
        payload = run_session_start(plugin_root, "cursor")

        # Flat and snake_case. Close enough to Copilot's to read as a typo,
        # far enough from Claude Code's to be dropped in silence.
        check("top-level key is additional_context, flat and snake_case",
              list(payload.keys()) == ["additional_context"])
        ctx = payload["additional_context"]
        check("carries using-tbaguette's SKILL.md verbatim",
              USING_TBAGUETTE_FIXTURE.rstrip("\n") in ctx)
        # Cursor's tool surface is Claude Code-compatible, so unlike Copilot it
        # really does have a Skill tool and the default wording is correct.
        check("keeps Claude Code's Skill-tool wording, which Cursor can follow",
              "use the 'Skill' tool" in ctx)


def check_user_prompt_submit_cursor() -> None:
    print("user-prompt-submit: Cursor postToolUse throttle")
    with tempfile.TemporaryDirectory() as tmp:
        env = dict(os.environ, TMPDIR=tmp)

        def call(conversation_id: str = "conv-1") -> dict:
            result = subprocess.run(
                ["bash", str(USER_PROMPT_SUBMIT), "cursor"],
                input=json.dumps({"conversation_id": conversation_id,
                                  "hook_event_name": "postToolUse"}),
                capture_output=True, text=True, check=False, env=env,
            )
            check_quiet = result.returncode == 0
            check(f"exits 0 (conversation={conversation_id})", check_quiet)
            return json.loads(result.stdout)

        first = call()
        check("speaks up on the first tool call of a conversation",
              list(first.keys()) == ["additional_context"])
        ctx = first["additional_context"]
        check("the nudge is delimited so it does not read as the user",
              ctx.startswith("<TBAGUETTE_SKILL_CHECK>"))
        check("names the Skill tool, which Cursor has", "Skill tool" in ctx)
        check("closes the 'just a question' loophole", "Questions" in ctx)

        # postToolUse fires per tool result, not per turn -- emitting on every
        # one would put the nudge in front of every file read. The throttle is
        # what makes riding this event affordable rather than merely possible.
        emitted = [i for i in range(2, 21) if call() != {}]
        check("stays quiet between re-assertions rather than firing on every "
              "tool result", emitted == [10, 20])

        # A second conversation is counted separately, or a long session would
        # silence a short one that started beside it.
        other = call("conv-2")
        check("each conversation gets its own counter",
              list(other.keys()) == ["additional_context"])


def main() -> None:
    check_hooks_json_shape()
    check_hooks_copilot_json_shape()
    check_hooks_vscode_json_shape()
    check_hooks_cursor_json_shape()
    check_executable_bits()
    check_user_prompt_submit()
    check_user_prompt_submit_copilot()
    check_user_prompt_submit_vscode()
    check_user_prompt_submit_cursor()
    check_session_start_basic_shape()
    check_session_start_copilot_shape()
    check_session_start_vscode_shape()
    check_session_start_cursor_shape()
    check_run_hook_cmd_unix_passthrough()
    check_update_check_same_sha()
    check_update_check_update_available()
    check_update_check_dirty_tree()
    check_update_check_no_git()

    print(f"\n{checker.total} checks passed.")


if __name__ == "__main__":
    main()
