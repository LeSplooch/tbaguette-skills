"""Keeps CATALOG.md honest against content_pipeline.CATEGORIES.

CATEGORIES is the authoritative grouping: the site is built from it, and
build_content already raises when a skill directory isn't covered by it or
when a category names a slug that doesn't exist on disk. CATALOG.md is the
hand-maintained human mirror of that same grouping -- and until this suite
existed, *nothing* compared the two. A skill could be filed under
"Testing" on the site and "Landing changes" in CATALOG.md indefinitely,
and every check in the repo would still be green.

That gap mattered more once creating a *new* category became a documented
move rather than a thing nobody did: a new category is two registrations,
and the second one is the one that gets forgotten.

The `†` marker in CATALOG.md means "not a TBaguette skill, listed only to
show where a TBaguette skill hands off to a neighbour" -- those rows are
deliberately absent from CATEGORIES, so they're skipped here rather than
reported as drift.
"""
from __future__ import annotations

import re
import sys
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPTS_DIR.parent
sys.path.insert(0, str(SCRIPTS_DIR))

from content_pipeline import CATEGORIES  # noqa: E402

CATALOG_PATH = REPO_ROOT / "CATALOG.md"
SKILLS_DIR = REPO_ROOT / "skills"

# A CATALOG.md table row: | `slug` | prose |  -- with an optional trailing †
# on the skill cell marking a non-TBaguette neighbour.
_ROW = re.compile(r"^\|\s*`([a-z0-9-]+)`\s*(†?)\s*\|")


def _parse_catalog() -> list[tuple[str, list[str], list[str]]]:
    """[(heading title, own skill slugs, foreign †-marked slugs)], in file order."""
    sections: list[tuple[str, list[str], list[str]]] = []
    title: str | None = None
    own: list[str] = []
    foreign: list[str] = []
    for line in CATALOG_PATH.read_text(encoding="utf-8").splitlines():
        if line.startswith("## "):
            if title is not None:
                sections.append((title, own, foreign))
            title, own, foreign = line[3:].strip(), [], []
            continue
        match = _ROW.match(line)
        if match and title is not None:
            (foreign if match.group(2) else own).append(match.group(1))
    if title is not None:
        sections.append((title, own, foreign))
    return sections


class TestCatalogMatchesCategories(unittest.TestCase):
    def setUp(self):
        self.sections = _parse_catalog()

    def test_headings_match_categories_in_order(self):
        """Display order on the site is CATEGORIES order; CATALOG.md must read
        the same way, or a browser and a reader disagree about the library."""
        self.assertEqual(
            [title for title, _, _ in self.sections],
            [category["title"] for category in CATEGORIES],
        )

    def test_each_category_lists_exactly_its_skills(self):
        by_title = {title: own for title, own, _ in self.sections}
        for category in CATEGORIES:
            with self.subTest(category=category["title"]):
                self.assertIn(category["title"], by_title)
                self.assertEqual(by_title[category["title"]], list(category["skill_slugs"]))

    def test_no_skill_is_listed_twice(self):
        listed = [slug for _, own, _ in self.sections for slug in own]
        duplicates = sorted({slug for slug in listed if listed.count(slug) > 1})
        self.assertEqual(duplicates, [])

    def test_foreign_skills_are_not_tbaguette_skills(self):
        """A † row claims the skill lives outside this repo. If one grows a
        directory under skills/, the marker is a lie and the skill is missing
        from CATEGORIES -- which build_content would then refuse to build."""
        for title, _, foreign in self.sections:
            for slug in foreign:
                with self.subTest(category=title, skill=slug):
                    self.assertFalse((SKILLS_DIR / slug).is_dir())

    def test_every_prose_skill_count_matches_reality(self):
        """The skill count is written in prose in eight hand-maintained places
        and nothing checked any of them.

        Found by a subagent walking the ship path for a hypothetical 93rd
        skill: it listed every file it would have to hand-edit and noted none
        were gated. The versioned-manifest suite validates those files\' names,
        paths and versions and deliberately never reads their descriptions.
        A first pass at this test guarded only .claude-plugin/plugin.json --
        the count is in seven manifests and README.md."""
        words = {60: "Sixty", 70: "Seventy", 80: "Eighty", 90: "Ninety"}
        ones = ["", "-one", "-two", "-three", "-four", "-five",
                "-six", "-seven", "-eight", "-nine"]
        on_disk = len([p for p in SKILLS_DIR.iterdir() if p.is_dir()])
        tens, unit = (on_disk // 10) * 10, on_disk % 10
        self.assertIn(tens, words, f"spell-out table needs extending for {on_disk}")
        spelled, digits = words[tens] + ones[unit], str(on_disk)

        # Every file carrying the count in prose, and the form it uses.
        for rel, form in [
            (".claude-plugin/plugin.json", spelled),
            (".claude-plugin/marketplace.json", spelled),
            (".codex-plugin/plugin.json", spelled),
            (".cursor-plugin/plugin.json", spelled),
            (".devin-plugin/plugin.json", spelled),
            (".kimi-plugin/plugin.json", spelled),
            ("gemini-extension.json", spelled),
            ("README.md", digits + " skills"),
        ]:
            text = (REPO_ROOT / rel).read_text(encoding="utf-8")
            with self.subTest(file=rel):
                self.assertIn(
                    form, text,
                    f"{rel} does not contain {form!r}; {on_disk} skills are on disk",
                )
                # A stale count must not also still be present.
                for stale in (on_disk - 1, on_disk + 1):
                    st, su = (stale // 10) * 10, stale % 10
                    if st in words:
                        self.assertNotIn(words[st] + ones[su] + " ", text)

    def test_stated_skill_count_matches_reality(self):
        """CATALOG.md opens with 'N skills'. N is written by hand."""
        # Only the preamble, so a body line that happens to start "12 skills,"
        # cannot satisfy the check the header is supposed to satisfy.
        preamble = CATALOG_PATH.read_text(encoding="utf-8").split("## ", 1)[0]
        match = re.search(r"^(\d+) skills,", preamble, re.M)
        self.assertIsNotNone(match, "CATALOG.md has no 'N skills,' opening line")
        on_disk = len([p for p in SKILLS_DIR.iterdir() if p.is_dir()])
        self.assertEqual(int(match.group(1)), on_disk)


if __name__ == "__main__":
    unittest.main(verbosity=2)
