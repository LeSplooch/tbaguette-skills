"""Tests for the TBaguette plugin's SessionStart hook: hooks/hooks.json's
declared shape, and hooks/session-start's actual behavior -- that it emits
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
RUN_HOOK_CMD = HOOKS_DIR / "run-hook.cmd"

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


def run_session_start(plugin_root: Path) -> dict:
    script = plugin_root / "hooks" / "session-start"
    result = subprocess.run(
        ["bash", str(script)], capture_output=True, text=True, check=False,
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


# ---------------------------------------------------------------------------
# File permissions
# ---------------------------------------------------------------------------


def check_executable_bits() -> None:
    print("executable bits")
    check("session-start is executable", os.access(SESSION_START, os.X_OK))
    check("run-hook.cmd is executable", os.access(RUN_HOOK_CMD, os.X_OK))


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


def main() -> None:
    check_hooks_json_shape()
    check_executable_bits()
    check_session_start_basic_shape()
    check_run_hook_cmd_unix_passthrough()
    check_update_check_same_sha()
    check_update_check_update_available()
    check_update_check_dirty_tree()
    check_update_check_no_git()

    print(f"\n{checker.total} checks passed.")


if __name__ == "__main__":
    main()
