"""Integration test for generate.py — the seam content_pipeline.py and
templates.py were each tested against in isolation (against a hand-written
fixture, before either could see the other's real output), now exercised
together against the real, embedded, 65-skill corpus. Catches exactly the
class of contract drift a review of this project flagged as previously
untested: two halves individually correct, never proven correct together.

Usage:
    python3 scripts/test_generate.py
"""

from __future__ import annotations

import shutil
import tempfile
from pathlib import Path

import generate
from checker import Checker

checker = Checker()
check = checker.check


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
    finally:
        shutil.rmtree(tmp_root, ignore_errors=True)

    print(f"\n{checker.total} checks passed.")


if __name__ == "__main__":
    main()
