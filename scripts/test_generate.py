"""Integration test for generate.py — the seam content_pipeline.py and
templates.py were each tested against in isolation (against a hand-written
fixture, before either could see the other's real output), now exercised
together against the real, embedded, 90-skill corpus. Catches exactly the
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
import templates
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


def check_skill_links_end_to_end(docs: Path, base_path: str) -> None:
    """Every skill cross-reference on the built site, checked against the pages
    that actually exist beside it. The unit tests cover the linking rules; this
    covers the wiring — base_path, the update-notes path, and the claim that no
    link is dead."""
    print("skill cross-reference links")
    slugs = {p.name for p in (docs / "skills").iterdir() if p.is_dir()}
    total = 0
    broken: list[str] = []
    unprefixed: list[str] = []
    for page in sorted(docs.rglob("index.html")):
        for href in re.findall(
            r'class="skill-link[^"]*" href="([^"]*)"', page.read_text(encoding="utf-8")
        ):
            total += 1
            if base_path and not href.startswith(base_path + "/"):
                unprefixed.append(f"{page.name}: {href}")
            if href.strip("/").split("/")[-1] not in slugs:
                broken.append(f"{page.relative_to(docs)}: {href}")

    check(f"the built site carries skill cross-reference links ({total} of them)",
          total > 300)
    check(f"every one points at a skill page that exists — no dead cross-"
          f"references (first offenders: {broken[:3]})", not broken)
    check(f"every one carries the deployment base path, so they resolve on the "
          f"published site and not only when served from a domain root "
          f"(first offenders: {unprefixed[:3]})", not unprefixed)

    index = (docs / "index.html").read_text(encoding="utf-8")
    # Scoped to the notes section and slug-agnostic on purpose. This used to
    # assert one hardcoded slug appeared linked somewhere on the page, which
    # made it a test of *which entries are currently newest*: UPDATE_NOTES_LIMIT
    # is 6, so shipping a seventh entry silently pushed the named skill off the
    # rendered set and failed a check about linking that linking had not broken.
    notes_section = re.search(r'<ul class="notes__bullets">.*?</section>', index, re.S)
    check("the update notes render at all", notes_section is not None)
    notes_links = re.findall(
        r'class="skill-link[^"]*" href="([^"]*)"', notes_section.group(0) if notes_section else ""
    )
    check(f"the update notes link their skill mentions too ({len(notes_links)} linked)",
          any(href.strip("/").split("/")[-1] in slugs for href in notes_links))
    # Same brittleness as the check above, and it bit for the same reason: this
    # asserted the literal "<code>UPDATES.md</code>", which only holds while the
    # entry that happens to mention UPDATES.md is inside UPDATE_NOTES_LIMIT.
    # Three entries shipped in one day pushed it out and reddened a check about
    # linking that linking had not broken. Assert the property instead.
    notes_html = notes_section.group(0) if notes_section else ""
    code_spans = re.findall(r"<code>([^<]+)</code>", notes_html)
    non_skill_spans = [c for c in code_spans if c not in slugs]
    check(f"...but a code span in those notes that is not a skill stays plain "
          f"({len(non_skill_spans)} of {len(code_spans)} are non-skill) — a bare "
          f"`UPDATES.md` linked to a nonexistent /skills/UPDATES.md/ until the "
          f"notes path got the same known-slug filter build_content already used",
          non_skill_spans and not re.search(
              r'class="skill-link[^"]*" href="[^"]*/skills/(?:' +
              "|".join(re.escape(c) for c in non_skill_spans) + r')/"', notes_html))

    own = (docs / "skills" / "naming-things" / "index.html").read_text(encoding="utf-8")
    check("a page never links to itself",
          f'class="skill-link" href="{base_path}/skills/naming-things/"' not in own)

    # formidable is the page with two link systems on it at once, which is the
    # only place they could collide. Its reference panels cross-reference each
    # other by *filename* ("harden.md"), which resolves to an on-page anchor;
    # its body cross-references another *skill* by slug, which resolves to that
    # skill's page. Measured, not assumed: the panels contain no slug mentions
    # at all, so the real check is that adding slug linking left the 6 anchor
    # links untouched.
    formidable = (docs / "skills" / "formidable" / "index.html").read_text(encoding="utf-8")
    check("formidable's body links the one other skill it names",
          f'class="skill-link" href="{base_path}/skills/knowing-when-to-stop/"' in formidable)
    check("`tokens` in that same body stays plain — it names formidable's own "
          "tokens command, not a skill, and there is no skill by that name",
          "<code>tokens</code>" in formidable
          and f'{base_path}/skills/tokens/' not in formidable)
    anchors = re.findall(r'href="#((?:cmd|stack)-[a-z-]+)"', formidable)
    ids = set(re.findall(r'id="([^"]+)"', formidable))
    dangling = sorted({a for a in anchors if a not in ids})
    check(f"the reference panels' own relative-.md links all still resolve to "
          f"real on-page ids ({len(anchors)} of them), undisturbed by the new "
          f"slug linking (dangling: {dangling[:3]})",
          len(anchors) > 20 and not dangling)


def check_update_notes_source() -> None:
    """_update_notes() against a real file on disk. The parser's own grammar is
    covered in test_content_pipeline; what matters here is the policy generate.py
    layers on top — missing is silent, malformed stops the build."""
    print("_update_notes")
    root = Path(tempfile.mkdtemp(prefix="tbaguette-update-notes-test-"))
    try:
        check("no UPDATES.md at all: no notes, no error",
              generate._update_notes(root) == [])

        path = root / generate.UPDATE_NOTES_FILENAME
        path.write_text(
            "# Update notes\n\n## 2026-08-24 — A change\n- It changed.\n",
            encoding="utf-8",
        )
        entries = generate._update_notes(root)
        check("a well-formed file parses into entries",
              len(entries) == 1 and entries[0]["date"] == "2026-08-24"
              and entries[0]["notes"] == ["It changed."])

        # The failure that actually happens: appending the new entry to the
        # bottom of the file instead of the top. Nothing about the resulting
        # page looks broken -- it just quietly shows a months-old entry as the
        # latest news, which is worse than not shipping the section at all.
        path.write_text(
            "## 2026-08-20 — Older\n- A.\n\n## 2026-08-24 — Newer\n- B.\n",
            encoding="utf-8",
        )
        failed = False
        try:
            generate._update_notes(root)
        except SystemExit as error:
            failed = str(generate.UPDATE_NOTES_FILENAME) in str(error)
        check("an out-of-order file stops the build, naming the file, rather "
              "than shipping a stale entry as the newest one", failed)

        path.write_text("## Unreleased\n- A.\n", encoding="utf-8")
        failed = False
        try:
            generate._update_notes(root)
        except SystemExit:
            failed = True
        check("a heading that is not a date stops the build too — the site has "
              "nowhere to put an undated entry", failed)
    finally:
        shutil.rmtree(root, ignore_errors=True)


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
        check(
            "a project root with no UPDATES.md builds a normal site with no "
            "update-notes section — a clone that has never shipped a change "
            "has nothing to say, and that is not a build failure",
            "data-update-notes" not in index_html,
        )

        gs_path = docs / "getting-started" / "index.html"
        check("docs/getting-started/index.html exists after a real build", gs_path.exists())
        gs_html = gs_path.read_text(encoding="utf-8") if gs_path.exists() else ""
        check(
            "...and carries the generated-file header, so nobody hand-edits a page "
            "the next build overwrites",
            gs_html.startswith(generate.GENERATED_HEADER),
        )
        check(
            "its skill-count sentence is interpolated from the real catalog, which "
            "is the number this build actually shipped",
            f"memorize {generate.EXPECTED_SKILL_COUNT} names" in gs_html,
        )
        check(
            "the landing page reaches it from all three agreed entry points "
            "(header nav, install frame, footer)",
            index_html.count('href="/tbaguette-skills/getting-started/"') == 3,
        )
        check(
            "and so does a skill page's header, on a real build rather than a fixture",
            'href="/tbaguette-skills/getting-started/"' in karen_html,
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
        # ...and it doubles as the run that proves UPDATES.md reaches the page,
        # since the first one deliberately had no such file to read.
        print("second run (repeat-safety of the atomic swap, now with UPDATES.md)")
        real_updates = real_project_root / generate.UPDATE_NOTES_FILENAME
        expected_notes = generate._update_notes(real_project_root)
        shutil.copyfile(real_updates, tmp_root / generate.UPDATE_NOTES_FILENAME)
        content2 = generate.generate(tmp_root, real_skills_root, base_path="/tbaguette-skills")

        index_html2 = (docs / "index.html").read_text(encoding="utf-8")
        check(
            "this repository's own UPDATES.md parses and reaches the built page",
            "data-update-notes" in index_html2 and len(expected_notes) > 0,
        )
        check(
            "the newest entry in the file is the one the page shows expanded",
            f'datetime="{expected_notes[0]["date"]}"' in index_html2,
        )
        check(
            "the page shows no more entries than the cap allows, however long "
            "the file has grown — the newest is rendered twice on purpose, "
            "once on the page and once at the top of the archive dialog, so "
            "the dialog reads as a whole record rather than a remainder",
            index_html2.count('<li class="notes__entry">')
            == min(len(expected_notes), templates.UPDATE_NOTES_LIMIT) + 1,
        )
        check(
            "the notes land between the fresh rail and the search field, "
            "under real data and not just in the template's own fixtures",
            index_html2.index("data-update-notes") < index_html2.index("data-search-root"),
        )

        # Deferred to here rather than run after the first build: half of what
        # it checks lives in the update notes, and the first build deliberately
        # had no UPDATES.md to read.
        check_skill_links_end_to_end(docs, "/tbaguette-skills")
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
    check_update_notes_source()

    print(f"\n{checker.total} checks passed.")


if __name__ == "__main__":
    main()
