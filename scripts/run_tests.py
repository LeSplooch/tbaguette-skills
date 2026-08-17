"""Runs the complete test suite.

`python3 -m unittest discover` only finds test_content_pipeline.py's
TestCase-based tests — it silently never runs test_templates.py or
test_generate.py, since both use a plain assert-based runner rather than
unittest, and `discover` has no way to know they exist. That gap is exactly
how a real bug shipped once in this repo with a green "OK" on record: the
suite that would have caught it was never actually running. This script is
the one command that runs all three and fails loudly if any of them do, so
"all tests pass" means what it says.

Check counts below are parsed from each suite's own output rather than
hardcoded here — a hardcoded count is exactly the kind of thing that goes
stale the next time a check is added, which happened twice to this file
during the same feature before it got fixed this way instead.

Usage:
    python3 scripts/run_tests.py
"""

from __future__ import annotations

import re
import subprocess
import sys
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent

SUITES = [
    ("content_pipeline.py (unittest)",
     [sys.executable, "-m", "unittest", "test_content_pipeline", "-v"]),
    ("templates.py", [sys.executable, "test_templates.py"]),
    ("python_highlight.py", [sys.executable, "test_python_highlight.py"]),
    ("generate.py integration", [sys.executable, "test_generate.py"]),
    ("i18n", [sys.executable, "test_i18n.py"]),
    ("hooks", [sys.executable, "test_hooks.py"]),
    ("harness manifests", [sys.executable, "test_harness_manifests.py"]),
    # The only suite that hits the network (clones the real, published repo
    # from GitHub several times against throwaway HOME directories) — a
    # flaky connection can fail this one without meaning anything else is
    # broken. Everything it proves is otherwise untested: that the install
    # command on the live site can never alter a user's other skills.
    ("install command safety", [sys.executable, "test_install_command.py"]),
]

# Covers both runner styles in this repo: the plain assert-based one prints
# "N checks passed."; unittest's own -v output prints "Ran N tests in ...".
_COUNT_PATTERNS = (
    re.compile(r"(\d+) checks passed\."),
    re.compile(r"Ran (\d+) tests? in"),
)


def _extract_count(text: str) -> int | None:
    for pattern in _COUNT_PATTERNS:
        match = pattern.search(text)
        if match:
            return int(match.group(1))
    return None


def main() -> int:
    failures = []
    total = 0
    for label, cmd in SUITES:
        print(f"\n{'=' * 70}\n{label}\n{'=' * 70}")
        result = subprocess.run(cmd, cwd=SCRIPTS_DIR, capture_output=True, text=True, check=False)
        print(result.stdout, end="")
        print(result.stderr, end="", file=sys.stderr)

        count = _extract_count(result.stdout + result.stderr)
        if count is None:
            print(f"(could not parse a check count from {label}'s output)")
        else:
            total += count

        if result.returncode != 0:
            failures.append(label)

    print(f"\n{'=' * 70}")
    if failures:
        print(f"FAILED: {len(failures)}/{len(SUITES)} suites failed:")
        for label in failures:
            print(f"  - {label}")
        return 1

    print(f"All {len(SUITES)} suites passed, {total} checks total.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
