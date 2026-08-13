"""Runs the complete test suite.

`python3 -m unittest discover` only finds test_content_pipeline.py's
TestCase-based tests (42 checks) — it silently never runs test_templates.py
(56 checks) or test_generate.py (16 checks), since both use a plain
assert-based runner rather than unittest, and `discover` has no way to know
they exist. That gap is exactly how a real bug shipped once in this repo
with a green "OK" on record: the suite that would have caught it was never
actually running. This script is the one command that runs all three and
fails loudly if any of them do, so "all tests pass" means what it says.

Usage:
    python3 scripts/run_tests.py
"""

from __future__ import annotations

import subprocess
import sys
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent

SUITES = [
    ("content_pipeline.py (unittest, 42 checks)",
     [sys.executable, "-m", "unittest", "test_content_pipeline", "-v"]),
    ("templates.py (56 checks)", [sys.executable, "test_templates.py"]),
    ("generate.py integration (16 checks)", [sys.executable, "test_generate.py"]),
]


def main() -> int:
    failures = []
    for label, cmd in SUITES:
        print(f"\n{'=' * 70}\n{label}\n{'=' * 70}")
        result = subprocess.run(cmd, cwd=SCRIPTS_DIR)
        if result.returncode != 0:
            failures.append(label)

    print(f"\n{'=' * 70}")
    if failures:
        print(f"FAILED: {len(failures)}/{len(SUITES)} suites failed:")
        for label in failures:
            print(f"  - {label}")
        return 1

    print(f"All {len(SUITES)} suites passed.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
