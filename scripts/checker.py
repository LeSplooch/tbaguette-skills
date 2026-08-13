"""Shared pass/fail tally for this project's plain assert-based test files.

test_generate.py, test_python_highlight.py, and test_templates.py each used
to define an identical module-level `check()` mutating a `global` counter —
three copies of the same eight lines, and `global` besides. `Checker` is that
logic once, as a small object instead of module state:

    from checker import Checker

    checker = Checker()
    check = checker.check          # existing call sites stay `check(label, condition)`
    ...
    print(f"{checker.total} checks passed.")

test_content_pipeline.py has no need of this — it's unittest-based, with its
own pass/fail accounting. test_install_command.py also doesn't use this: it
deliberately keeps running after a failure to report every scenario's result
in one pass rather than stopping at the first, which is a real behavioral
difference from the fail-fast files above, not just a naming one.
"""

from __future__ import annotations


class Checker:
    """Counts check() calls and raises immediately on the first failure."""

    def __init__(self) -> None:
        self.total = 0

    def check(self, label: str, condition: bool) -> None:
        self.total += 1
        if not condition:
            raise AssertionError(f"FAILED: {label}")
        print(f"  ok  {label}")
