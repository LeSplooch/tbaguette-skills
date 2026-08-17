"""Integration test for generate.py — the seam content_pipeline.py and
templates.py were each tested against in isolation (against a hand-written
fixture, before either could see the other's real output), now exercised
together against the real, embedded, 87-skill corpus. Catches exactly the
class of contract drift a review of this project flagged as previously
untested: two halves individually correct, never proven correct together.

Usage:
    python3 scripts/test_generate.py
"""

from __future__ import annotations

import os
import re
import shutil
import subprocess
import tempfile
from datetime import datetime, timedelta, timezone
from pathlib import Path

import generate
from checker import Checker

checker = Checker()
check = checker.check


def _run_git(args: list[str], cwd: Path) -> subprocess.CompletedProcess:
    return subprocess.run(
        ["git", *args], cwd=cwd, capture_output=True, text=True, check=True,
    )


def check_changed_skill_slugs() -> None:
    """_changed_skill_slugs() against a real, throwaway git repo — not
    mocked, since the whole point is proving the actual `git status
    --porcelain` parsing handles the real output shapes (a modified
    tracked file, a brand-new untracked directory, and nothing at all for
    a skill nobody touched)."""
    print("_changed_skill_slugs")
    repo = Path(tempfile.mkdtemp(prefix="tbaguette-changed-slugs-test-"))
    try:
        _run_git(["init", "-q"], repo)
        _run_git(["config", "user.email", "test@example.invalid"], repo)
        _run_git(["config", "user.name", "Test"], repo)

        for slug in ("alpha", "beta"):
            skill_dir = repo / "skills" / slug
            skill_dir.mkdir(parents=True)
            (skill_dir / "SKILL.md").write_text(f"---\nname: {slug}\n---\nOriginal.\n", encoding="utf-8")
        _run_git(["add", "-A"], repo)
        _run_git(["commit", "-q", "-m", "initial: alpha and beta"], repo)

        check("clean checkout, nothing touched yet: no changes reported",
              generate._changed_skill_slugs(repo) == {})

        # alpha: modify a tracked file -> "updated"
        (repo / "skills" / "alpha" / "SKILL.md").write_text(
            "---\nname: alpha\n---\nRevised.\n", encoding="utf-8"
        )
        # gamma: a whole new, never-committed directory -> "new"
        gamma_dir = repo / "skills" / "gamma"
        gamma_dir.mkdir()
        (gamma_dir / "SKILL.md").write_text("---\nname: gamma\n---\nBrand new.\n", encoding="utf-8")
        # beta: left alone entirely

        changed = generate._changed_skill_slugs(repo)
        check("modified existing skill is reported as updated, not new",
              changed.get("alpha") == "updated")
        check("brand-new untracked skill directory is reported as new",
              changed.get("gamma") == "new")
        check("untouched skill is absent, not reported as anything",
              "beta" not in changed)
        check("exactly the two touched skills are reported, nothing else",
              set(changed) == {"alpha", "gamma"})

        # A file changed *outside* skills/ (e.g. scripts/generate.py, a real
        # everyday case) must not leak into the result — this function is
        # scoped to skills/ specifically, on purpose.
        (repo / "README.md").write_text("unrelated change\n", encoding="utf-8")
        changed_after_unrelated = generate._changed_skill_slugs(repo)
        check("a change outside skills/ doesn't add anything",
              set(changed_after_unrelated) == {"alpha", "gamma"})

        # This project's own pre-commit hook runs `git add -A` *before*
        # calling generate() — a real-world case, not a hypothetical one:
        # the first real skill shipped through that hook showed "Updated"
        # instead of "New" on the live site, because a staged-added file
        # reads "A " in `git status --porcelain`, not "??". Prove the fix
        # by staging gamma exactly the way the hook does, not just leaving
        # it untracked the way the check above already covers.
        _run_git(["add", "-A"], repo)
        changed_after_staging = generate._changed_skill_slugs(repo)
        check("a *staged* new skill (git add -A, matching the pre-commit "
              "hook's own order of operations) is still reported as new, "
              "not misread as merely updated",
              changed_after_staging.get("gamma") == "new")
        check("a staged modification is still reported as updated",
              changed_after_staging.get("alpha") == "updated")
    finally:
        shutil.rmtree(repo, ignore_errors=True)

    non_repo = Path(tempfile.mkdtemp(prefix="tbaguette-not-a-repo-"))
    try:
        check("outside any git repo: fails open to no changes, not an exception",
              generate._changed_skill_slugs(non_repo) == {})
    finally:
        shutil.rmtree(non_repo, ignore_errors=True)


def check_freshness_window() -> None:
    """_recent_skill_commits()/_fresh_skills() against a real throwaway repo
    with real, back-dated commits — the whole point is that a badge expires on
    committer date, and only actual git history proves that parsing works."""
    print("48-hour freshness window")
    repo = Path(tempfile.mkdtemp(prefix="tbaguette-freshness-test-"))
    try:
        _run_git(["init", "-q"], repo)
        _run_git(["config", "user.email", "test@example.invalid"], repo)
        _run_git(["config", "user.name", "Test"], repo)

        now = datetime(2026, 8, 14, 12, 0, 0, tzinfo=timezone.utc)

        # The commits below are deliberately created in an order that has
        # nothing to do with their dates (9d, 9d, 3h, 5h, 10h, 1h, 9d, 49h).
        # Real histories look like this after a rebase, a cherry-pick, an
        # import, clock skew, or two agents committing into one repository.
        # This is not decoration: an earlier version of _recent_skill_commits
        # used `git log --since`, which stops walking at the first old-enough
        # commit, and every check below the "veteran" one failed because half
        # the history was pruned away.

        def commit_at(message: str, when: datetime) -> None:
            stamp = when.isoformat()
            env = {
                **os.environ,
                "GIT_AUTHOR_DATE": stamp,
                "GIT_COMMITTER_DATE": stamp,
            }
            subprocess.run(["git", "add", "-A"], cwd=repo, check=True,
                           capture_output=True, text=True)
            subprocess.run(["git", "commit", "-q", "-m", message], cwd=repo,
                           check=True, capture_output=True, text=True, env=env)

        def write(slug: str, text: str) -> None:
            skill_dir = repo / "skills" / slug
            skill_dir.mkdir(parents=True, exist_ok=True)
            (skill_dir / "SKILL.md").write_text(f"---\nname: {slug}\n---\n{text}\n",
                                                encoding="utf-8")

        # ancient: added 9 days ago, never touched since -> outside the window
        write("ancient", "Original.")
        commit_at("add ancient", now - timedelta(days=9))

        # veteran: added 9 days ago, edited 3 hours ago -> Updated, not New
        write("veteran", "Revised recently.")
        commit_at("edit veteran... ", now - timedelta(days=9))
        write("veteran", "Revised again.")
        commit_at("edit veteran", now - timedelta(hours=3))

        # newborn: added 5 hours ago -> New
        write("newborn", "Brand new.")
        commit_at("add newborn", now - timedelta(hours=5))

        # reborn: added 10 hours ago and edited 1 hour ago -> Updated, since
        # the badge must describe the same event as the timestamp beside it
        write("reborn", "Created.")
        commit_at("add reborn", now - timedelta(hours=10))
        write("reborn", "And edited.")
        commit_at("edit reborn", now - timedelta(hours=1))

        # stale-edge: edited 49 hours ago -> just outside a 48-hour window
        write("stale-edge", "Created long ago.")
        commit_at("add stale-edge", now - timedelta(days=9))
        write("stale-edge", "Edited just outside the window.")
        commit_at("edit stale-edge", now - timedelta(hours=49))

        fresh = generate._fresh_skills(repo, now=now)

        check("a skill untouched for nine days carries no badge at all",
              "ancient" not in fresh)
        check("an edit 49 hours old has already expired — the boundary is real, "
              "not 'roughly two days'",
              "stale-edge" not in fresh)
        check("a long-lived skill edited inside the window reads Updated",
              fresh.get("veteran", {}).get("status") == "updated")
        check("a skill added inside the window reads New",
              fresh.get("newborn", {}).get("status") == "new")
        check("a skill added AND then edited inside the window reads Updated: "
              "the badge describes the same event as the time shown next to "
              "it, and 'New' beside '1 hour ago' would claim it was created "
              "an hour ago",
              fresh.get("reborn", {}).get("status") == "updated")
        check("that skill is stamped with its most recent touch, not the older "
              "commit that created it",
              fresh.get("reborn", {}).get("at", "").startswith("2026-08-14T11:00"))

        # Uncommitted work is what is about to ship: it must show up even
        # though no commit exists to find it by, which is exactly the state
        # this project's pre-commit hook generates in.
        write("ancient", "Edited but not yet committed.")
        write("pending", "Never committed at all.")
        fresh_with_worktree = generate._fresh_skills(repo, now=now)
        check("an uncommitted edit to an otherwise-expired skill brings it back, "
              "because it is the update currently being shipped",
              fresh_with_worktree.get("ancient", {}).get("status") == "updated")
        # newborn was committed as New five hours ago. Editing it now makes
        # the working tree the newest event, and the working tree's event is
        # a modification — so the badge has to follow the timestamp down to
        # "Updated" rather than keeping yesterday's stronger word.
        write("newborn", "Edited after being created.")
        check("an uncommitted edit overrides a committed New: the newest event "
              "is a modification, and the badge tracks the newest event",
              generate._fresh_skills(repo, now=now)["newborn"]["status"] == "updated")
        check("a brand-new uncommitted skill directory reads New",
              fresh_with_worktree.get("pending", {}).get("status") == "new")
        check("uncommitted work is stamped 'now', so it sorts above every "
              "commit already in the window",
              fresh_with_worktree["pending"]["at"] == now.isoformat())

        ordered = generate._fresh_order(
            {slug: {"slug": slug} for slug in fresh_with_worktree},
            fresh_with_worktree,
        )
        check("_fresh_order returns newest change first",
              [s["slug"] for s in ordered][:2] == ["ancient", "pending"]
              or [s["slug"] for s in ordered][:2] == ["pending", "ancient"])
        check("...and the committed ones follow in recency order behind them",
              [s["slug"] for s in ordered][2:] == ["reborn", "veteran", "newborn"])
    finally:
        shutil.rmtree(repo, ignore_errors=True)

    non_repo = Path(tempfile.mkdtemp(prefix="tbaguette-fresh-not-a-repo-"))
    try:
        check("outside any git repo, freshness fails open to nothing fresh "
              "rather than blocking the build",
              generate._fresh_skills(non_repo, now=datetime.now(timezone.utc)) == {})
    finally:
        shutil.rmtree(non_repo, ignore_errors=True)


def main() -> None:
    real_project_root = Path(__file__).resolve().parent.parent
    real_skills_root = real_project_root / "skills"

    tmp_root = Path(tempfile.mkdtemp(prefix="tbaguette-generate-test-"))
    try:
        print(f"building into throwaway root: {tmp_root}")
        content = generate.generate(tmp_root, real_skills_root, base_path="/tbaguette-skills")

        check(
            "returned exactly the expected skill count",
            len(content["skills"]) == generate.EXPECTED_SKILL_COUNT,
        )

        docs = tmp_root / "docs"
        check("docs/index.html exists", (docs / "index.html").exists())
        check(
            "docs/index.html carries the generated-file header",
            (docs / "index.html").read_text(encoding="utf-8").startswith(generate.GENERATED_HEADER),
        )

        skill_dirs = sorted(p.name for p in (docs / "skills").iterdir() if p.is_dir())
        check(
            f"exactly {generate.EXPECTED_SKILL_COUNT} skill page directories",
            len(skill_dirs) == generate.EXPECTED_SKILL_COUNT,
        )

        for slug in ("formidable", "karen-and-the-manager", "knowing-when-to-stop"):
            check(f"{slug}'s page exists", (docs / "skills" / slug / "index.html").exists())

        formidable_html = (docs / "skills" / "formidable" / "index.html").read_text(encoding="utf-8")
        check(
            "formidable's page has the base_path-prefixed stylesheet link",
            '"/tbaguette-skills/assets/styles.css"' in formidable_html,
        )
        check(
            "formidable's craft-floor anchor resolves end to end (the earlier bug fix, under real data)",
            'id="cmd-craft-floor"' in formidable_html and '#cmd-craft-floor"' in formidable_html,
        )
        check(
            "formidable's page has all 23 tab panels (12 stacks + 11 commands)",
            formidable_html.count('role="tabpanel"') == 23,
        )
        check(
            "formidable's page carries a well-formed last-updated <time> element "
            "(exact value is wall-clock and untestable here, but the element must exist)",
            '<time class="site-header__updated-value"' in formidable_html
            and 'datetime="' in formidable_html,
        )

        karen_html = (docs / "skills" / "karen-and-the-manager" / "index.html").read_text(encoding="utf-8")
        check("the new skill's own trigger description made it into its page", "never satisfied" in karen_html)
        check(
            "the new skill's prev/next nav places it after knowing-when-to-stop",
            "knowing-when-to-stop" in karen_html,
        )

        index_html = (docs / "index.html").read_text(encoding="utf-8")
        check(
            "index mentions the real, current skill count, not a stale hardcoded one",
            f"see all {generate.EXPECTED_SKILL_COUNT} again" in index_html,
        )
        check(
            "index links to the new skill",
            'href="/tbaguette-skills/skills/karen-and-the-manager/"' in index_html,
        )

        version_txt_path = docs / "version.txt"
        check("version.txt exists after generation", version_txt_path.exists())
        version_txt_content = version_txt_path.read_text(encoding="utf-8")

        index_dt_match = re.search(r'datetime="([^"]+)"', index_html)
        check("index.html's header carries a parseable datetime attribute",
              index_dt_match is not None)
        check(
            "version.txt's content matches the timestamp baked into index.html, "
            "byte for byte (this is the exact string client JS string-compares "
            "against, so it must carry no GENERATED_HEADER or other prefix)",
            index_dt_match is not None and version_txt_content == index_dt_match.group(1),
        )

        formidable_dt_match = re.search(r'datetime="([^"]+)"', formidable_html)
        check(
            "the same version.txt also matches a skill page's timestamp "
            "(one run, one instant, everywhere)",
            formidable_dt_match is not None and version_txt_content == formidable_dt_match.group(1),
        )

        # A second run proves the atomic-swap machinery is safe to repeat,
        # not just safe to run once — this is the exact property that
        # protects a real "edit a skill, rerun generate.py" workflow.
        print("second run (repeat-safety of the atomic swap)")
        content2 = generate.generate(tmp_root, real_skills_root, base_path="/tbaguette-skills")
        check(
            "second run returns the same skill count",
            len(content2["skills"]) == len(content["skills"]),
        )
        check(
            "no leftover staging or backup directory after two runs",
            not any(p.name.startswith(".docs.") for p in tmp_root.iterdir()),
        )
        check("version.txt still exists after a second run", version_txt_path.exists())
        check(
            "no leftover version.txt backup file after two runs",
            not (tmp_root / ".docs.previous.version.txt").exists(),
        )
    finally:
        shutil.rmtree(tmp_root, ignore_errors=True)

    check_changed_skill_slugs()
    check_freshness_window()

    print(f"\n{checker.total} checks passed.")


if __name__ == "__main__":
    main()
