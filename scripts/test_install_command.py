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

from templates import INSTALL_COMMAND

REPO_URL = "https://github.com/LeSplooch/tbaguette-skills.git"


class Tally:
    """Accumulates check() results instead of raising on the first failure —
    deliberately different from checker.Checker's fail-fast style (used by
    this project's other plain assert-based test files). This file's whole
    point is comparing results across four independent scenarios, and across
    several shells in the bonus check; stopping at the first failure would
    hide whether everything else still held, which is exactly the question
    a report of "3 of 24 failed, here's which three" answers and a bare
    traceback from the first one doesn't."""

    def __init__(self) -> None:
        self.total = 0
        self.failures = 0

    def check(self, label: str, condition: bool) -> None:
        self.total += 1
        if condition:
            print(f"  ok  {label}")
        else:
            print(f"  FAIL  {label}")
            self.failures += 1


tally = Tally()
check = tally.check


def install(home: Path) -> subprocess.CompletedProcess:
    """The published install command's logic, as direct git calls instead of
    a shell string: if ~/.claude/skills/TBaguette/.git already exists, pull
    in place; otherwise clone. Scoped to that single path by construction —
    neither branch has any mechanism to read or write anywhere else."""
    target = home / ".claude" / "skills" / "TBaguette"
    if (target / ".git").is_dir():
        return subprocess.run(
            ["git", "-C", str(target), "pull"],
            capture_output=True, text=True, check=False,
        )
    return subprocess.run(
        ["git", "clone", REPO_URL, str(target)],
        capture_output=True, text=True, check=False,
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


# Every shell worth naming explicitly rather than just "a POSIX shell": bash
# and zsh cover the macOS/Linux default in every version this century, sh is
# whatever /bin/sh resolves to locally (often bash or dash), and fish is
# included deliberately even though it isn't POSIX-compatible in general —
# fish 3.0+ (2018) added && / || specifically so commands copy-pasted from
# bash/zsh docs would still work, which is exactly this page's use case, and
# it's worth proving rather than assuming.
CANDIDATE_SHELLS = ("bash", "zsh", "fish", "sh")


def check_matches_published_command(workdir: Path) -> None:
    """Best-effort bonus: for every shell in CANDIDATE_SHELLS actually
    present on PATH, runs the literal published command string — not this
    file's reimplementation of it — against its own fresh scenario. A shell
    missing from PATH is skipped with a note, not a failure; on a bare
    Windows system with neither Git Bash nor WSL, every candidate is
    skipped, and that's fine — install() above already proves the same
    guarantee via git directly, with no shell dependency at all. This
    function exists to additionally cross-check the exact string published
    on the site, on whichever real shells happen to be available here."""
    tested_any = False
    for shell_name in CANDIDATE_SHELLS:
        shell = shutil.which(shell_name)
        if shell is None:
            print(f"  (skipped {shell_name}: not on PATH)")
            continue
        tested_any = True

        home = workdir / f"shellcheck-{shell_name}"
        home.mkdir()
        seed_sibling_skills(home)
        before = siblings_checksum(home)

        env = {**os.environ, "HOME": str(home)}
        result = subprocess.run(
            [shell, "-c", INSTALL_COMMAND], env=env, capture_output=True, text=True, check=False,
        )
        check(f"the literal published command exits 0 under {shell_name}",
              result.returncode == 0)
        check(f"...and actually installs something, under {shell_name}",
              (home / ".claude" / "skills" / "TBaguette" / "README.md").is_file())
        check(f"...without touching the sibling skills under {shell_name}, using "
              "the exact string published on the site, not a reimplementation of it",
              before == siblings_checksum(home))

    if not tested_any:
        print("  (skipped entirely: no POSIX-ish shell on PATH — install() above "
              "already verified the same guarantee via git directly, without one)")


def main() -> int:
    workdir = Path(tempfile.mkdtemp(prefix="tbaguette-install-test-"))
    try:
        print("scenario A: fresh install")
        home_a = workdir / "a"
        home_a.mkdir()
        seed_sibling_skills(home_a)
        before_a = siblings_checksum(home_a)
        result = install(home_a)
        check("clone exits 0", result.returncode == 0)
        check("TBaguette content actually present",
              (home_a / ".claude" / "skills" / "TBaguette" / "README.md").is_file())
        check("sibling skills byte-identical after install",
              before_a == siblings_checksum(home_a))

        print("scenario B: re-run when already installed (must update, not error)")
        home_b = home_a  # continues from A, which already installed TBaguette
        before_b = siblings_checksum(home_b)
        result = install(home_b)
        combined_output = (result.stdout or "") + (result.stderr or "")
        check("re-run exits 0 (does not error)", result.returncode == 0)
        check(
            "re-run reports a pull, not a clone-refused error",
            any(marker in combined_output for marker in
                ("Already up to date", "Updating", "Fast-forward")),
        )
        check("sibling skills still byte-identical after re-run",
              before_b == siblings_checksum(home_b))

        print("scenario C: an empty pre-existing TBaguette directory")
        home_c = workdir / "c"
        (home_c / ".claude" / "skills" / "TBaguette").mkdir(parents=True)
        seed_sibling_skills(home_c)
        before_c = siblings_checksum(home_c)
        result = install(home_c)
        check("clone into empty existing dir exits 0", result.returncode == 0)
        check("TBaguette content actually present",
              (home_c / ".claude" / "skills" / "TBaguette" / "README.md").is_file())
        check("sibling skills byte-identical", before_c == siblings_checksum(home_c))

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
        check("refuses (nonzero exit) rather than merging into unrelated content",
              result.returncode != 0)
        check("the colliding directory's own content is untouched",
              hashlib.sha256(marker_file.read_bytes()).hexdigest() == before_marker)
        check("sibling skills byte-identical even in the refusal case",
              before_d == siblings_checksum(home_d))

        print("bonus: the literal published command string, where a shell exists")
        check_matches_published_command(workdir)
    finally:
        shutil.rmtree(workdir, ignore_errors=True)

    print()
    if tally.failures == 0:
        print(f"{tally.total} checks passed.")
        return 0
    print(f"{tally.failures} of {tally.total} checks FAILED.")
    return 1


if __name__ == "__main__":
    sys.exit(main())
