"""The Crumb has to change whenever the skills do, and nothing may publish one
that did not.

Three layers. The comparisons in crumb_check.py, on small synthetic graphs,
so each kind of drift is named in words a person can act on. The library as it
stands, for the one hand-written thing the Crumb needs per family -- a colour
in each theme -- because a category added without one draws in the fallback
accent and no other check would notice. And a throwaway repository, driven
through the real pre-push hook, because a checker nobody runs at the moment of
publishing guarantees nothing.

Deliberately absent: a check that this checkout's own HEAD passes. The nightly
routine commits skill edits with hooks off and rebuilds the site in one closing
commit, so an intermediate HEAD is stale by design, and only the commit a push
lands on master has to agree with its skills. The hook checks exactly that.
"""

from __future__ import annotations

import copy
import html
import json
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPTS_DIR = Path(__file__).resolve().parent
REPO_ROOT = SCRIPTS_DIR.parent
sys.path.insert(0, str(SCRIPTS_DIR))

import crumb_check  # noqa: E402
from crumb_check import (  # noqa: E402
    GRAPH_JSON, GRAPH_PAGE, LANDING_PAGE, STYLES,
    banner_problems, compare_graphs, page_problems, palette_blocks, palette_problems,
)


def _skill(slug: str, **fields) -> dict:
    node = {"slug": slug, "name": slug, "category": "a", "summary": "", "words": 10,
            "always_on": False, "change_status": None, "change_at": None,
            "trigger": {"refs": {}, "quotes": {}}, "sections": []}
    node.update(fields)
    return node


def _graph() -> dict:
    return {
        "schema": 1,
        "skill_url_template": "/tbaguette-skills/skills/{slug}/",
        "categories": [{"slug": "a", "title": "Family A", "skill_slugs": ["one", "two"]}],
        "skills": [_skill("one"), _skill("two", change_status="new", change_at="2026-09-24T00:00:00+00:00")],
    }


class TestComparingGraphs(unittest.TestCase):
    def test_the_clock_fields_are_not_a_difference(self):
        """change_status and change_at are stamped against the building
        clock; two builds of the same skills an hour apart disagree on them."""
        published = _graph()
        published["skills"][1]["change_status"] = None
        published["skills"][1]["change_at"] = None
        self.assertEqual(compare_graphs(_graph(), published), [])

    def test_a_skill_whose_node_moved_is_named(self):
        published = _graph()
        published["skills"][0]["words"] = 9
        problems = compare_graphs(_graph(), published)
        self.assertEqual(len(problems), 1)
        self.assertIn("out of date", problems[0])
        self.assertIn("one", problems[0])

    def test_a_new_skill_and_a_deleted_one_are_both_named(self):
        fresh = _graph()
        fresh["skills"].append(_skill("three"))
        published = _graph()
        published["skills"].append(_skill("retired"))
        joined = " | ".join(compare_graphs(fresh, published))
        self.assertIn("missing from docs/graph/graph.json: three", joined)
        self.assertIn("no longer skills: retired", joined)

    def test_a_build_for_another_base_path_is_called_that(self):
        """A local build (no --base-path) links every skill to /skills/...,
        which does not exist on the Pages site."""
        published = _graph()
        published["skill_url_template"] = "/skills/{slug}/"
        self.assertTrue(any("another base path" in p for p in compare_graphs(_graph(), published)))

    def test_a_family_added_to_the_catalog_is_reported(self):
        fresh = _graph()
        fresh["categories"].append({"slug": "b", "title": "Family B", "skill_slugs": []})
        problems = compare_graphs(fresh, _graph())
        self.assertTrue(any("1 there, 2 in the skills" in p for p in problems), problems)

    def test_a_reordering_alone_still_counts(self):
        published = _graph()
        published["skills"].reverse()
        self.assertEqual(compare_graphs(_graph(), published),
                         ["docs/graph/graph.json differs from a fresh build in its shape or its order"])


class TestThePagesAroundTheGraph(unittest.TestCase):
    LEDE = "How 2 skills lean on each other: 1 cross-references, traced section by section."

    def test_the_page_must_carry_the_summary_a_fresh_build_writes(self):
        stale = '<meta name="description" content="How 2 skills lean on each other: 0 cross-references">'
        self.assertEqual(len(page_problems(stale, self.LEDE)), 1)

    def test_the_summary_is_found_raw_or_escaped(self):
        self.assertEqual(page_problems(f"<p>{self.LEDE}</p>", self.LEDE), [])
        lede = "It’s “quoted” & <b>marked</b>"
        self.assertEqual(page_problems(f'<meta content="{html.escape(lede)}">', lede), [])

    def test_a_missing_page_is_a_problem(self):
        self.assertEqual(page_problems(None, self.LEDE), [f"{GRAPH_PAGE} is missing"])

    def test_the_banner_is_only_compared_while_it_is_up(self):
        banner = {"families": [{"count": 2}], "links": []}
        self.assertEqual(banner_problems("<main>no banner any more</main>", banner), [])

    def test_a_stale_banner_is_named(self):
        banner = {"families": [{"count": 2}, {"count": 1}], "links": [[0, 1, 3]]}
        tag = ('<aside class="graph-banner" data-families="[2,1]" '
               'data-links="[[0,1,2]]">')
        problems = banner_problems(tag, banner)
        self.assertEqual(problems, [f"the graph banner on {LANDING_PAGE} carries stale links"])


class TestThePalette(unittest.TestCase):
    CSS = """
    /* --graph-cat-3: #000; a colour in a comment is not a colour */
    .crumb, .graph-banner { --graph-cat-1: #111; --graph-cat-2: #222; --graph-cat-3: #333; }
    [data-theme='flour'] :is(.crumb, .graph-banner) { --graph-cat-1: #aaa; --graph-cat-2: #bbb; }
    .unrelated { color: red; }
    """

    def test_each_theme_is_read_as_its_own_rule(self):
        blocks = palette_blocks(self.CSS)
        self.assertEqual(blocks, {".crumb, .graph-banner": {1, 2, 3},
                                  "[data-theme='flour'] :is(.crumb, .graph-banner)": {1, 2}})

    def test_a_theme_missing_a_family_names_the_theme_and_the_family(self):
        problems = palette_problems(self.CSS, 3)
        self.assertEqual(len(problems), 1)
        self.assertIn("[data-theme='flour']", problems[0])
        self.assertIn("family 3 of 3", problems[0])

    def test_a_commented_out_colour_does_not_count(self):
        css = ".crumb { --graph-cat-1: #111; /* --graph-cat-2: #222; */ }"
        problems = palette_problems(css, 2)
        self.assertEqual(len(problems), 1, "the commented-out entry was counted as a colour")
        self.assertIn("family 2 of 2", problems[0])

    def test_no_palette_at_all_is_a_problem(self):
        self.assertEqual(palette_problems(".crumb { color: red; }", 1),
                         [f"{STYLES} defines no --graph-cat-N family colours at all"])


class TestTheLibraryAsItStands(unittest.TestCase):
    def test_every_family_in_the_catalog_has_a_colour_in_both_themes(self):
        """The gate a new category meets: the Crumb and the landing banner
        colour families by position, and past the palette's end they fall back
        to the accent without a word."""
        import content_pipeline
        css = (REPO_ROOT / STYLES).read_text(encoding="utf-8")
        self.assertEqual(palette_problems(css, len(content_pipeline.CATEGORIES)), [])
        self.assertGreaterEqual(len(palette_blocks(css)), 2,
                                "expected one palette rule per theme (dark and flour)")


def _git(root: Path, *args: str) -> str:
    result = subprocess.run(["git", *args], cwd=root, capture_output=True, text=True, check=False)
    if result.returncode != 0:
        raise AssertionError(f"git {' '.join(args)} failed: {result.stderr}")
    return result.stdout.strip()


class TestAgainstARealRepository(unittest.TestCase):
    """What the pre-push hook runs, on commits made the three ways that
    matter: rebuilt, not rebuilt, and rebuilt again."""

    @classmethod
    def setUpClass(cls):
        cls._tmp = tempfile.TemporaryDirectory(prefix="crumb-check-test-")
        root = cls.root = Path(cls._tmp.name)
        shutil.copytree(REPO_ROOT / "scripts", root / "scripts",
                        ignore=shutil.ignore_patterns("__pycache__"))
        shutil.copytree(REPO_ROOT / "skills", root / "skills")
        (root / ".githooks").mkdir()
        shutil.copy2(REPO_ROOT / ".githooks" / "pre-push", root / ".githooks" / "pre-push")
        (root / STYLES).parent.mkdir(parents=True)
        shutil.copy2(REPO_ROOT / STYLES, root / STYLES)
        for args in (("init", "--quiet"), ("config", "user.email", "crumb@example.invalid"),
                     ("config", "user.name", "crumb test"), ("config", "commit.gpgsign", "false"),
                     ("config", "core.hooksPath", "/dev/null")):
            _git(root, *args)
        _git(root, "add", "-A")
        _git(root, "commit", "--quiet", "-m", "sources")

        cls.built = crumb_check.build_at(_git(root, "rev-parse", "HEAD"), root)
        cls.rebuilt = cls._commit_site("site")

        # A skill gains a citation and is committed with no rebuild -- the
        # shape of every source-only commit, and of a hook that never ran.
        first = cls.built["graph"]["skills"][0]
        cited = set(first["trigger"]["refs"])
        for section in first["sections"]:
            cited |= set(section["refs"])
        target = next(s["slug"] for s in cls.built["graph"]["skills"]
                      if s["slug"] != first["slug"] and s["slug"] not in cited)
        cls.edited, cls.target = first["slug"], target
        skill_md = root / "skills" / first["slug"] / "SKILL.md"
        skill_md.write_text(skill_md.read_text(encoding="utf-8")
                            + f"\n\nSee also `{target}`.\n", encoding="utf-8")
        _git(root, "add", "-A")
        _git(root, "commit", "--quiet", "-m", "skill only")
        cls.stale = _git(root, "rev-parse", "HEAD")

        cls.built = crumb_check.build_at(cls.stale, root)
        cls.rebuilt_again = cls._commit_site("site again")

    @classmethod
    def _commit_site(cls, message: str) -> str:
        root, built = cls.root, cls.built
        (root / GRAPH_JSON).parent.mkdir(parents=True, exist_ok=True)
        (root / GRAPH_JSON).write_text(json.dumps(built["graph"]), encoding="utf-8")
        (root / GRAPH_PAGE).write_text(
            f'<meta name="description" content="{html.escape(built["lede"])}">', encoding="utf-8")
        banner = built["banner"]
        families = json.dumps([f["count"] for f in banner["families"]])
        links = json.dumps(banner["links"])
        (root / LANDING_PAGE).write_text(
            f'<aside class="graph-banner" data-families="{html.escape(families)}" '
            f'data-links="{html.escape(links)}"></aside>', encoding="utf-8")
        _git(root, "add", "-A")
        _git(root, "commit", "--quiet", "-m", message)
        return _git(root, "rev-parse", "HEAD")

    @classmethod
    def tearDownClass(cls):
        cls._tmp.cleanup()

    def _push(self, *lines: str) -> subprocess.CompletedProcess:
        return subprocess.run(
            [str(self.root / ".githooks" / "pre-push"), "origin", "https://example.invalid/repo.git"],
            cwd=self.root, input="".join(line + "\n" for line in lines),
            capture_output=True, text=True, check=False,
        )

    def test_a_rebuilt_commit_passes(self):
        for sha in (self.rebuilt, self.rebuilt_again):
            self.assertEqual(crumb_check.check(sha, root=self.root)[1], [])

    def test_a_commit_that_skipped_the_rebuild_names_the_skill_and_the_pages(self):
        problems = crumb_check.check(self.stale, root=self.root)[1]
        joined = " | ".join(problems)
        self.assertIn(f"out of date in {GRAPH_JSON}: {self.edited}", joined)
        self.assertIn(GRAPH_PAGE, joined, "one citation more changes the page's own count")

    def test_a_commit_before_the_graph_existed_has_nothing_to_check(self):
        first = _git(self.root, "rev-list", "--max-parents=0", "HEAD")
        self.assertEqual(crumb_check.check(first, root=self.root)[1], [])

    def test_the_hook_refuses_to_publish_the_stale_crumb(self):
        result = self._push(f"HEAD {self.stale} refs/heads/master {'0' * 40}")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("refused", result.stderr)
        self.assertIn(self.edited, result.stderr)

    def test_the_hook_lets_the_rebuilt_commit_through(self):
        result = self._push(f"HEAD {self.rebuilt_again} refs/heads/master {self.stale}")
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_the_hook_ignores_every_branch_but_master(self):
        result = self._push(f"refs/heads/wip {self.stale} refs/heads/wip {'0' * 40}")
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_the_hook_ignores_a_deletion(self):
        result = self._push(f"(delete) {'0' * 40} refs/heads/master {self.stale}")
        self.assertEqual(result.returncode, 0, result.stderr)

    def test_the_hook_checks_every_ref_line_it_is_given(self):
        """The second line must still reach the loop after the checker ran
        for the first -- a child reading the hook's stdin would swallow it."""
        result = self._push(f"HEAD {self.rebuilt_again} refs/heads/master {'0' * 40}",
                            f"HEAD {self.stale} refs/heads/master {'0' * 40}")
        self.assertNotEqual(result.returncode, 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
