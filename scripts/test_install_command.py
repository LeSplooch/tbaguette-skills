"""Proves the published install command (templates.INSTALL_COMMAND) can never
alter, overwrite, or merge into anything a user already has under
~/.claude/skills/ that this repo doesn't own — a user asked "won't this alter
other skills?" and this is the actual answer, re-checked on every run rather
than trusted from memory.

Stdlib only, and deliberately not a shell script (this file replaces what was
originally test_install_command.sh): bash is not guaranteed to exist on every
system this might run on, notably Windows without Git Bash or WSL, and even
where a POSIX shell is present, the coreutils it calls out to differ across
platforms in ways that silently break things — the original script's
sha256sum has no equivalent by that name on macOS (it ships shasum -a 256
instead). Python 3 is a much narrower, more realistic requirement here, since
it's already needed to run scripts/generate.py, and git is already required
to install the plugin in the first place — nothing new to ask a reader to
have.

Four scenarios, each in its own throwaway HOME so a real ~/.claude is never
touched:
  A. fresh install                        — clones cleanly
  B. re-running once already installed     — updates in place (git pull),
                                              does not error
  C. an empty dir already named TBaguette  — clones into it cleanly
  D. a NON-empty, non-git dir already named
     TBaguette (the real collision case)   — refuses, leaves it untouched

Sibling directories (other skills, other plugins) are populated in every
scenario and checksummed before/after to prove they are never touched, not
just assumed to be.

The install() function below reimplements the published command's logic as
direct git subprocess calls rather than shelling out to run the literal
string — the same two git operations, in the same order, gated by the same
condition, but expressed so this file's core guarantee holds even on a
system with no POSIX shell at all. Where a shell *is* available,
check_matches_published_command() additionally runs the literal string from
templates.INSTALL_COMMAND itself, so the claim "the published command is
safe" stays backed by the actual text shown on the site, not only by code
that is merely intended to match it.

Usage:
    python3 scripts/test_install_command.py
"""

from __future__ import annotations

import hashlib
import os
import shutil
import subprocess
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from templates import INSTALL_COMMAND  # noqa: E402

REPO_URL = "https://github.com/LeSplooch/tbaguette-skills.git"

_checks = 0
_fails = 0


def check(condition: bool, label: str) -> None:
    global _checks, _fails
    _checks += 1
    if condition:
        print(f"  ok  {label}")
    else:
        print(f"  FAIL  {label}")
        _fails += 1


def install(home: Path) -> subprocess.CompletedProcess:
    """The published install command's logic, as direct git calls instead of
    a shell string: if ~/.claude/skills/TBaguette/.git already exists, pull
    in place; otherwise clone. Scoped to that single path by construction —
    neither branch has any mechanism to read or write anywhere else."""
    target = home / ".claude" / "skills" / "TBaguette"
    if (target / ".git").is_dir():
        return subprocess.run(
            ["git", "-C", str(target), "pull"],
            capture_output=True, text=True,
        )
    return subprocess.run(
        ["git", "clone", REPO_URL, str(target)],
        capture_output=True, text=True,
    )


def seed_sibling_skills(home: Path) -> None:
    """Fake pre-existing content the install must never touch: a loose
    skill directory and a separate plugin, exactly the two shapes
    ~/.claude/skills/ actually holds (see orienting-in-unfamiliar-code and
    the TBaguette plugin's own layout)."""
    other_skill = home / ".claude" / "skills" / "some-other-skill"
    other_skill.mkdir(parents=True, exist_ok=True)
    (other_skill / "SKILL.md").write_text(
        "precious user content that must survive\n", encoding="utf-8"
    )

    plugin = home / ".claude" / "skills" / "another-plugin"
    (plugin / ".claude-plugin").mkdir(parents=True, exist_ok=True)
    (plugin / ".claude-plugin" / "plugin.json").write_text(
        '{"name": "another-plugin"}\n', encoding="utf-8"
    )
    (plugin / "skills" / "foo").mkdir(parents=True, exist_ok=True)
    (plugin / "skills" / "foo" / "SKILL.md").write_text(
        "more precious user content\n", encoding="utf-8"
    )


def siblings_checksum(home: Path) -> list[tuple[str, str]]:
    """(path relative to home, sha256) for every file under the two sibling
    directories, sorted — needs no external tool at all, unlike the original
    script's `find ... -exec sha256sum {} \\; | sort`, so there is nothing
    here that can be missing or spelled differently across platforms."""
    roots = (
        home / ".claude" / "skills" / "some-other-skill",
        home / ".claude" / "skills" / "another-plugin",
    )
    entries = [
        (str(path.relative_to(home)), hashlib.sha256(path.read_bytes()).hexdigest())
        for root in roots
        for path in root.rglob("*")
        if path.is_file()
    ]
    return sorted(entries)


def check_matches_published_command(workdir: Path) -> None:
    """Best-effort bonus check: on any system with a POSIX shell on PATH,
    also runs the literal published command string — not this file's
    reimplementation of it — against one fresh scenario. Skipped, not
    failed, where no such shell exists (a bare Windows system with neither
    Git Bash nor WSL); install() above already proves the same guarantee
    there via git directly, which is the whole reason it's written that
    way rather than as a wrapper around a shell string."""
    shell = shutil.which("bash") or shutil.which("sh")
    if shell is None:
        print("  (skipped: no POSIX shell on PATH — install() above already "
              "verified the same guarantee via git directly, without one)")
        return

    home = workdir / "shellcheck"
    home.mkdir()
    seed_sibling_skills(home)
    before = siblings_checksum(home)

    env = {**os.environ, "HOME": str(home)}
    result = subprocess.run(
        [shell, "-c", INSTALL_COMMAND], env=env, capture_output=True, text=True,
    )
    check(result.returncode == 0,
          f"the literal published command also exits 0 under {Path(shell).name}")
    check((home / ".claude" / "skills" / "TBaguette" / "README.md").is_file(),
          "...and actually installs something")
    check(before == siblings_checksum(home),
          "...without touching the sibling skills, using the exact string "
          "published on the site, not a reimplementation of it")


def main() -> int:
    workdir = Path(tempfile.mkdtemp(prefix="tbaguette-install-test-"))
    try:
        print("scenario A: fresh install")
        home_a = workdir / "a"
        home_a.mkdir()
        seed_sibling_skills(home_a)
        before_a = siblings_checksum(home_a)
        result = install(home_a)
        check(result.returncode == 0, "clone exits 0")
        check((home_a / ".claude" / "skills" / "TBaguette" / "README.md").is_file(),
              "TBaguette content actually present")
        check(before_a == siblings_checksum(home_a),
              "sibling skills byte-identical after install")

        print("scenario B: re-run when already installed (must update, not error)")
        home_b = home_a  # continues from A, which already installed TBaguette
        before_b = siblings_checksum(home_b)
        result = install(home_b)
        combined_output = (result.stdout or "") + (result.stderr or "")
        check(result.returncode == 0, "re-run exits 0 (does not error)")
        check(
            any(marker in combined_output for marker in
                ("Already up to date", "Updating", "Fast-forward")),
            "re-run reports a pull, not a clone-refused error",
        )
        check(before_b == siblings_checksum(home_b),
              "sibling skills still byte-identical after re-run")

        print("scenario C: an empty pre-existing TBaguette directory")
        home_c = workdir / "c"
        (home_c / ".claude" / "skills" / "TBaguette").mkdir(parents=True)
        seed_sibling_skills(home_c)
        before_c = siblings_checksum(home_c)
        result = install(home_c)
        check(result.returncode == 0, "clone into empty existing dir exits 0")
        check((home_c / ".claude" / "skills" / "TBaguette" / "README.md").is_file(),
              "TBaguette content actually present")
        check(before_c == siblings_checksum(home_c), "sibling skills byte-identical")

        print("scenario D: a non-empty, non-git TBaguette directory (real collision)")
        home_d = workdir / "d"
        target_d = home_d / ".claude" / "skills" / "TBaguette"
        target_d.mkdir(parents=True)
        marker_file = target_d / "dont-touch-me.txt"
        marker_file.write_text(
            "unrelated content some other tool put here\n", encoding="utf-8"
        )
        seed_sibling_skills(home_d)
        before_marker = hashlib.sha256(marker_file.read_bytes()).hexdigest()
        before_d = siblings_checksum(home_d)
        result = install(home_d)
        check(result.returncode != 0,
              "refuses (nonzero exit) rather than merging into unrelated content")
        check(hashlib.sha256(marker_file.read_bytes()).hexdigest() == before_marker,
              "the colliding directory's own content is untouched")
        check(before_d == siblings_checksum(home_d),
              "sibling skills byte-identical even in the refusal case")

        print("bonus: the literal published command string, where a shell exists")
        check_matches_published_command(workdir)
    finally:
        shutil.rmtree(workdir, ignore_errors=True)

    print()
    if _fails == 0:
        print(f"{_checks} checks passed.")
        return 0
    print(f"{_fails} of {_checks} checks FAILED.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
