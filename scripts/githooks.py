"""Point this clone's git at .githooks/, because git will not do it itself.

git never executes hooks that arrive with a clone, and that is deliberate: a
repository able to install its own hooks would be a repository that runs
arbitrary code on `git clone`. So `.githooks/pre-commit` travels with the repo
and does nothing at all until `core.hooksPath` points at it -- and that setting
is local to each clone and cannot be committed.

For this repo the consequence is not cosmetic. The hook regenerates `docs/`
before every commit, and `docs/` *is* the published site: GitHub Pages serves
it straight off master, so pushing is publishing. An unwired clone commits a
skill change with a stale site attached and publishes that, which is the exact
failure the hook was added for -- a styles.css-only commit that shipped without
regenerating, leaving the header's "Updated" time pointing at the commit
before it.

Nothing here can make git wire itself. What it can do is make the wiring happen
the first time a contributor runs anything, since every entry point in this repo
is `python3 scripts/<something>.py` and there is no install step to hang it off.
The call is idempotent, scoped to the clone with `--local`, announced only when
it actually changes something, and it never overwrites a hooksPath somebody set
on purpose.
"""

from __future__ import annotations

import subprocess
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
HOOKS_DIRNAME = ".githooks"


def _git(repo_root: Path, *args: str) -> tuple[int, str]:
    result = subprocess.run(
        ["git", *args], cwd=repo_root, capture_output=True, text=True, check=False
    )
    return result.returncode, result.stdout.strip()


def ensure_wired(repo_root: Path | None = None, announce=print) -> str:
    """Set core.hooksPath to .githooks unless it is already set to something.

    Returns one of: "wired" (we just set it), "already-wired", "custom"
    (someone pointed it elsewhere -- left alone), "no-hooks-dir",
    "not-a-git-checkout". Never raises: a contributor running the test suite
    from a tarball should get their tests, not a traceback about git.
    """
    root = Path(repo_root) if repo_root is not None else REPO_ROOT

    if not (root / HOOKS_DIRNAME).is_dir():
        return "no-hooks-dir"

    code, toplevel = _git(root, "rev-parse", "--show-toplevel")
    if code != 0 or not toplevel:
        return "not-a-git-checkout"
    # Only ever touch the checkout this file actually lives in.
    if Path(toplevel).resolve() != root.resolve():
        return "not-a-git-checkout"

    code, current = _git(root, "config", "--local", "--get", "core.hooksPath")
    if code == 0 and current:
        # Compare resolved directories, not strings. An absolute path pointing
        # at this same .githooks is wired, and reporting it as foreign is how
        # the first version of this function libelled its own repo.
        configured = Path(current)
        if not configured.is_absolute():
            configured = root / configured
        try:
            same = configured.resolve() == (root / HOOKS_DIRNAME).resolve()
        except OSError:
            same = False
        if same:
            return "already-wired"
        announce(
            f"note: core.hooksPath is {current!r}, not {HOOKS_DIRNAME!r} — leaving it alone. "
            f"This clone will not run {HOOKS_DIRNAME}/pre-commit, so docs/ is not "
            f"regenerated automatically before a commit."
        )
        return "custom"

    code, _ = _git(root, "config", "--local", "core.hooksPath", HOOKS_DIRNAME)
    if code != 0:
        return "not-a-git-checkout"
    announce(
        f"wired core.hooksPath -> {HOOKS_DIRNAME} for this clone (git cannot do this "
        f"itself; see scripts/githooks.py). docs/ now regenerates before every commit."
    )
    return "wired"


if __name__ == "__main__":
    print(ensure_wired())
