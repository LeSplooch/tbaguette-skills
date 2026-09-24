"""skill_graph.py reads the site's own cross-reference links back out.

The whole design rests on one claim: an edge exists in graph.json exactly
when a reader can click that cross-reference on a skill page. Everything
here either pins down how one rendered document is split into sections, or
checks that claim against the real library, with an instrument that shares
no code with the walker it is checking — a regex over the same HTML.

Usage:
    python3 -m unittest test_skill_graph -v
"""
from __future__ import annotations

import json
import re
import unittest
from pathlib import Path

import content_pipeline
import skill_graph
import templates

REPO_ROOT = Path(__file__).resolve().parent.parent
SKILLS_DIR = REPO_ROOT / "skills"


def _link(slug: str, base: str = "", code: bool = True) -> str:
    inner = f"<code>{slug}</code>" if code else slug
    cls = "skill-link" if code else "skill-link skill-link--bare"
    return f'<a class="{cls}" href="{base}/skills/{slug}/">{inner}</a>'


class SlugFromHrefTests(unittest.TestCase):
    def test_reads_the_slug_whatever_the_base_path_or_locale(self):
        for href in ("/skills/naming-things/", "/tbaguette-skills/skills/naming-things/",
                     "/tbaguette-skills/fr/skills/naming-things/"):
            self.assertEqual(skill_graph.skill_slug_from_href(href), "naming-things")

    def test_anything_else_is_not_a_skill(self):
        for href in ("#overview", "/getting-started/", "https://example.com/skills/x",
                     "/skills/naming-things/SKILL.md", ""):
            self.assertIsNone(skill_graph.skill_slug_from_href(href), href)


class ExtractSectionsTests(unittest.TestCase):
    def test_splits_on_h2_and_nests_h3(self):
        html = (
            '<h2 id="a">First</h2><p>one two three</p>'
            '<h3 id="a-1">Inner</h3><p>four five</p>'
            '<h2 id="b">Second</h2><p>six</p>'
        )
        sections = skill_graph.extract_sections(html)
        self.assertEqual([s["id"] for s in sections], ["a", "b"])
        self.assertEqual(sections[0]["title"], "First")
        self.assertEqual([sub["id"] for sub in sections[0]["subs"]], ["a-1"])
        self.assertEqual(sections[0]["subs"][0]["title"], "Inner")
        # Heading words count toward the section they head.
        self.assertEqual(sections[0]["words"], 1 + 3 + 1 + 2)
        self.assertEqual(sections[0]["subs"][0]["words"], 1 + 2)

    def test_an_empty_opening_is_not_a_section(self):
        sections = skill_graph.extract_sections('<h2 id="a">Only</h2><p>text</p>')
        self.assertEqual(len(sections), 1)

    def test_text_before_the_first_heading_is_the_opening(self):
        sections = skill_graph.extract_sections('<p>Lead words here.</p><h2 id="a">A</h2>')
        self.assertEqual(sections[0]["id"], "")
        self.assertEqual(sections[0]["title"], "Opening")
        self.assertEqual(sections[0]["words"], 3)

    def test_counts_both_link_shapes_and_attributes_them_to_the_sub(self):
        html = (
            f'<h2 id="a">A</h2><p>See {_link("naming-things")} and {_link("naming-things", code=False)}.</p>'
            f'<h3 id="a-x">X</h3><p>Then {_link("atomic-commits")}.</p>'
        )
        section = skill_graph.extract_sections(html)[0]
        self.assertEqual(section["refs"], {"naming-things": 2, "atomic-commits": 1})
        self.assertEqual(section["subs"][0]["refs"], {"atomic-commits": 1})

    def test_ordinary_links_and_code_are_not_citations(self):
        html = '<h2 id="a">A</h2><p><a href="#b">jump</a> and <code>naming-things</code>.</p>'
        self.assertEqual(skill_graph.extract_sections(html)[0]["refs"], {})

    def test_a_citation_in_a_heading_belongs_to_that_heading_section(self):
        html = f'<h2 id="a">A</h2><p>x</p><h2 id="b">Pairs with {_link("naming-things")}</h2><p>y</p>'
        sections = skill_graph.extract_sections(html)
        self.assertEqual(sections[0]["refs"], {})
        self.assertEqual(sections[1]["refs"], {"naming-things": 1})
        self.assertEqual(sections[1]["title"], "Pairs with naming-things")

    def test_the_quote_is_the_sentence_holding_the_mention(self):
        html = (
            f'<h2 id="a">A</h2><p>Unrelated opener. Reach for {_link("naming-things")} '
            f'when a name lies. Unrelated closer.</p>'
        )
        quote = skill_graph.extract_sections(html)[0]["quotes"]["naming-things"]
        self.assertEqual(quote, "Reach for naming-things when a name lies.")

    def test_a_long_sentence_is_trimmed_around_the_mention(self):
        filler = " ".join(["word"] * 120)
        html = f'<h2 id="a">A</h2><p>{filler} {_link("naming-things")} {filler}.</p>'
        quote = skill_graph.extract_sections(html)[0]["quotes"]["naming-things"]
        self.assertIn("naming-things", quote)
        self.assertLessEqual(len(quote), skill_graph.QUOTE_MAX_LENGTH + 2)
        self.assertTrue(quote.startswith("…") and quote.endswith("…"))

    def test_the_first_mention_in_a_section_supplies_its_quote(self):
        html = (
            f'<h2 id="a">A</h2><p>First {_link("naming-things")} here.</p>'
            f'<p>Second {_link("naming-things")} there.</p>'
        )
        section = skill_graph.extract_sections(html)[0]
        self.assertEqual(section["refs"]["naming-things"], 2)
        self.assertEqual(section["quotes"]["naming-things"], "First naming-things here.")

    def test_a_list_item_is_its_own_sentence(self):
        html = f'<h2 id="a">A</h2><ul><li>alpha {_link("naming-things")}</li><li>beta</li></ul>'
        quote = skill_graph.extract_sections(html)[0]["quotes"]["naming-things"]
        self.assertEqual(quote, "alpha naming-things")


class BuildSkillNodeTests(unittest.TestCase):
    def _skill(self, **extra):
        skill = {
            "slug": "alpha", "name": "alpha", "category_slug": "c",
            "description": "Use when x.", "description_html": "Use when x.",
            "summary": "Use when x.", "body_html": '<h2 id="one">One</h2><p>a b</p>',
        }
        skill.update(extra)
        return skill

    def test_reference_files_become_reference_sections_with_their_h2s_as_subs(self):
        skill = self._skill(reference_sections=[{
            "id": "ref-deep", "title": "Deep",
            "html": f'<p>Intro.</p><h2 id="ref-deep-part">Part</h2><p>See {_link("beta")}.</p>',
        }])
        node = skill_graph.build_skill_node(skill)
        self.assertEqual([s["kind"] for s in node["sections"]], ["body", "reference"])
        ref = node["sections"][1]
        self.assertEqual(ref["id"], "ref-deep")
        self.assertEqual(ref["title"], "Reference · Deep")
        self.assertEqual(ref["refs"], {"beta": 1})
        self.assertEqual([sub["id"] for sub in ref["subs"]], ["ref-deep-part"])
        self.assertEqual(node["words"], 2 + 1 + 1 + 1 + 2)

    def test_the_trigger_description_is_read_separately_from_the_body(self):
        skill = self._skill(description_html=f"Use when naming; pairs with {_link('beta')}.")
        node = skill_graph.build_skill_node(skill)
        self.assertEqual(node["trigger"]["refs"], {"beta": 1})
        self.assertTrue(all("beta" not in s["refs"] for s in node["sections"]))

    def test_always_on_is_read_off_the_description(self):
        self.assertTrue(skill_graph.build_skill_node(
            self._skill(description="Use at the start of every conversation, always."))["always_on"])
        self.assertFalse(skill_graph.build_skill_node(self._skill())["always_on"])


class RealLibraryTests(unittest.TestCase):
    """The claim itself, against every skill that ships."""

    @classmethod
    def setUpClass(cls):
        base = "/tbaguette-skills"
        cls.content = content_pipeline.build_content(
            str(SKILLS_DIR), resolve_skill_link=lambda slug: templates.skill_url(slug, base))
        cls.graph = skill_graph.build_graph(
            cls.content, skill_url_template=templates.skill_url("{slug}", base))

    def test_every_skill_appears_once_in_catalog_order(self):
        expected = [slug for c in self.content["categories"] for slug in c["skill_slugs"]]
        self.assertEqual([s["slug"] for s in self.graph["skills"]], expected)

    def test_every_citation_names_a_real_skill_and_never_the_citer(self):
        known = {s["slug"] for s in self.graph["skills"]}
        for (source, target) in skill_graph.edge_weights(self.graph):
            self.assertIn(target, known)
            self.assertNotEqual(source, target)

    def test_the_graph_holds_exactly_the_links_the_pages_render(self):
        # An independent instrument: every skill-link href in everything a
        # skill page renders, by regex rather than by the walker's parser.
        pattern = re.compile(r'<a class="skill-link[^"]*" href="[^"]*/skills/([a-z0-9-]+)/"')
        from_pages: dict[tuple[str, str], int] = {}
        for slug, skill in self.content["skills"].items():
            parts = [skill.get("description_html", ""), skill.get("body_html", ""),
                     skill.get("formidable_craft_floor_html", "")]
            for key in ("reference_sections", "formidable_stacks", "formidable_commands"):
                parts += [item["html"] for item in skill.get(key) or []]
            for target in pattern.findall("".join(parts)):
                from_pages[(slug, target)] = from_pages.get((slug, target), 0) + 1
        self.assertEqual(skill_graph.edge_weights(self.graph), from_pages)

    def test_the_url_template_builds_the_same_urls_as_the_site(self):
        template = self.graph["skill_url_template"]
        self.assertEqual(template.replace("{slug}", "naming-things"),
                         templates.skill_url("naming-things", "/tbaguette-skills"))

    def test_every_quote_mentions_the_skill_it_is_quoting_for(self):
        for skill in self.graph["skills"]:
            for section in [skill["trigger"]] + skill["sections"]:
                for target, quote in section["quotes"].items():
                    self.assertIn(target, quote, f"{skill['slug']} → {target}")

    def test_the_banner_summary_agrees_with_the_graph(self):
        summary = skill_graph.banner_summary(self.graph)
        weights = skill_graph.edge_weights(self.graph)
        self.assertEqual(summary["pair_count"], len(weights))
        self.assertEqual(summary["skill_count"], len(self.graph["skills"]))
        self.assertEqual(summary["mutual_count"],
                         sum(1 for (a, b) in weights if (b, a) in weights) // 2)
        self.assertEqual([f["count"] for f in summary["families"]],
                         [len(c["skill_slugs"]) for c in self.graph["categories"]])
        # Family links are unordered pairs of distinct families, each once.
        pairs = [(a, b) for a, b, _ in summary["links"]]
        self.assertTrue(all(a < b for a, b in pairs))
        self.assertEqual(len(pairs), len(set(pairs)))
        # Every cross-family citation lands in exactly one link.
        family = {s["slug"]: s["category"] for s in self.graph["skills"]}
        cross = sum(n for (a, b), n in weights.items() if family[a] != family[b])
        self.assertEqual(sum(w for _, _, w in summary["links"]), cross)
        self.assertGreaterEqual(summary["max_steps"], 1)

    def test_the_banner_summary_notices_an_unreachable_skill(self):
        graph = {"categories": [{"slug": "c", "title": "C", "skill_slugs": ["a", "b"]}],
                 "skills": [
                     {"slug": "a", "category": "c", "trigger": {"refs": {}},
                      "sections": [{"refs": {"b": 1}}]},
                     {"slug": "b", "category": "c", "trigger": {"refs": {}}, "sections": []},
                 ]}
        summary = skill_graph.banner_summary(graph)
        self.assertFalse(summary["reachable"])
        self.assertEqual(summary["max_steps"], 1)

    def test_the_document_stays_small_enough_to_fetch_on_a_click(self):
        # The dialog fetches this the first time Graph is clicked. Well past
        # this and the fetch is noticeable; the likeliest way to get there is
        # quoting far more than one sentence per citation.
        size = len(json.dumps(self.graph, ensure_ascii=False, separators=(",", ":")).encode())
        self.assertLess(size, 900_000)


if __name__ == "__main__":
    unittest.main()
