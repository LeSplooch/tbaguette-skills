"""Self-tests for content_pipeline.py.

Run with:
    python3 -m unittest scripts.test_content_pipeline -v
or directly:
    python3 scripts/test_content_pipeline.py

Fixtures are written to look like the real TBaguette skill files rather than
minimal placeholders ("abc", "foo bar") -- several of them are shaped after
real constructs found while surveying the 64-skill corpus (escaped pipes in
a table cell's inline code, a list item that wraps onto a continuation line
with no marker, a paragraph directly followed by a list with no blank line),
because those are exactly the cases a minimal fixture would never exercise.
"""

from __future__ import annotations

import json
import sys
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

from content_pipeline import (  # noqa: E402
    CATEGORIES,
    build_content,
    escape_html,
    humanize_filename,
    make_formidable_link_resolver,
    render_inline_markdown,
    render_markdown_body,
    slugify,
    split_frontmatter,
    strip_title_heading,
    summarize_description,
)

# The repo's own embedded copy — not ~/.claude/skills/TBaguette, which only
# exists on one particular machine. Pointing here means these integration
# tests actually run (rather than silently skip) for anyone who clones the
# repo, since the real corpus they need travels with it.
REAL_SKILLS_ROOT = Path(__file__).resolve().parent.parent / "skills"


# ---------------------------------------------------------------------------
# Fixtures shaped like real skill-file content
# ---------------------------------------------------------------------------

NESTED_LIST_MARKDOWN = """## Triage steps

- Classify the incident by blast radius before doing anything else:
  - **Single tenant** — isolated to one customer's data or account.
  - **Partial** — a feature or region is degraded, most traffic is fine.
  - **Full outage** — the primary user path is down for everyone.
- Page the on-call owner for the affected service.
- Open an incident channel and start the timeline.
"""

BOLD_AND_CODE_SENTENCE = (
    "Never store the **raw session token** in `localStorage`; keep it in an "
    "`httpOnly` cookie so **client-side script** cannot read it at all."
)

FORMIDABLE_STYLE_LINKS_MARKDOWN = """See [stacks/web.md](reference/stacks/web.md) for the web envelope,
[craft-floor.md](reference/craft-floor.md) for the finishing pass, external
guidance at [WCAG contrast](https://www.w3.org/TR/WCAG21/), and a dangling
pointer to [an old doc](reference/retired-notes.md) that no longer exists.
"""

TABLE_WITH_ESCAPED_PIPE_MARKDOWN = (
    "| # | Command | Answers |\n"
    "|---|---|---|\n"
    r"| 5 | `git log --oneline --since=1.week \| wc -l` | how noisy the week was |"
    "\n"
    r'| 6 | guard with `[ -e "$f" ] \|\| continue` | whether the file exists |'
    "\n"
)

FENCED_CODE_WITH_NUMBERED_LOOKING_LINES = (
    "Steps, concretely:\n\n"
    "```\n"
    "1. add new field, optional          deploy\n"
    "2. write both, read old              deploy\n"
    "3. backfill; verify equality\n"
    "```\n\n"
    "Six deploys, and *do not* skip step 3.\n"
)

LIST_WITH_UNMARKED_CONTINUATION_LINE = (
    "1. **Rank the options** worth comparing.\n"
    "2. **Narrow to two** — the current approach and the strongest\n"
    "   alternative, nothing else gets a full writeup.\n"
    "3. **Recommend one,** with the tradeoff named.\n"
)

PARAGRAPH_DIRECTLY_FOLLOWED_BY_LIST = (
    "Two passes, each cheap:\n"
    "1. **Surface** — the obvious map.\n"
    "2. **Spine** — one path followed end to end.\n"
)

SAMPLE_FRONTMATTER_SKILL_MD = """---
name: rotating-credentials
description: Use when a credential has leaked, when an access key has outlived its owner, or when deciding a rotation cadence for a service account.
user-invocable: true
---

# Rotating credentials

## When to use

- A key appears in a log, a ticket, or a screen share.
"""


# ---------------------------------------------------------------------------
# summarize_description
# ---------------------------------------------------------------------------


class SummarizeDescriptionTests(unittest.TestCase):
    def test_short_description_passes_through_unchanged(self):
        description = "Use when a request needs a short, honest cache."
        self.assertEqual(summarize_description(description), description)

    def test_description_exactly_at_max_length_passes_through(self):
        description = "x" * 139 + "."  # one "sentence", exactly 140 chars
        self.assertEqual(len(description), 140)
        self.assertEqual(summarize_description(description), description)

    def test_description_over_max_length_with_no_boundary_hard_cuts_with_ellipsis(self):
        description = "x" * 140 + "."  # 141 chars, no space/comma anywhere
        summary = summarize_description(description)
        self.assertLessEqual(len(summary), 140)
        self.assertTrue(summary.endswith("…"))

    def test_long_realistic_description_trims_at_a_clause_boundary(self):
        # Shaped like the real corpus: one long "Use when A, when B, ... or
        # when Z. Covers ..." sentence, deliberately past 140 chars.
        description = (
            "Use when rotating a credential, when a token has leaked into a log "
            "or a ticket, when a service account has broader access than the "
            "task in front of it needs, when an integration asks for more scope "
            "than it will ever call, or when deciding how long a short-lived "
            "credential should live. Covers scoping, rotation cadence, and "
            "blast radius if one leaks."
        )
        summary = summarize_description(description)
        self.assertLessEqual(len(summary), 140)
        self.assertTrue(summary.endswith("…"))
        # The kept text must be an exact, unaltered prefix of the source, and
        # the character right after the cut must be a real boundary -- this
        # is what "no mid-word cut" actually proves, not just eyeballing it.
        core = summary[:-1]
        self.assertTrue(description.startswith(core))
        boundary_char = description[len(core)]
        self.assertIn(boundary_char, (" ", ","))

    def test_long_description_with_no_early_comma_falls_back_to_word_boundary(self):
        description = (
            "Use when the deployment pipeline for the payments service needs a "
            "new canary stage before it is allowed anywhere near production "
            "traffic again after last quarter's outage without any comma "
            "anywhere near the front of this sentence to cut on at all here. "
            "Covers staged rollout and rollback."
        )
        summary = summarize_description(description)
        self.assertLessEqual(len(summary), 140)
        self.assertTrue(summary.endswith("…"))
        core = summary[:-1]
        self.assertTrue(description.startswith(core))
        boundary_char = description[len(core)]
        self.assertIn(boundary_char, (" ", ","))

    def test_all_real_descriptions_stay_within_budget(self):
        if not REAL_SKILLS_ROOT.is_dir():
            self.skipTest("real TBaguette skills directory not present")
        for skill_dir in sorted(REAL_SKILLS_ROOT.iterdir()):
            skill_md = skill_dir / "SKILL.md"
            if not skill_md.is_file():
                continue
            fields, _ = split_frontmatter(skill_md.read_text(encoding="utf-8"))
            with self.subTest(skill=skill_dir.name):
                summary = summarize_description(fields["description"])
                self.assertLessEqual(len(summary), 140)


# ---------------------------------------------------------------------------
# Inline rendering: bold, italic, code, escaping
# ---------------------------------------------------------------------------


class InlineMarkdownTests(unittest.TestCase):
    def test_bold_and_inline_code_render_together(self):
        html = render_inline_markdown(BOLD_AND_CODE_SENTENCE)
        self.assertIn("<strong>raw session token</strong>", html)
        self.assertIn("<code>localStorage</code>", html)
        self.assertIn("<code>httpOnly</code>", html)
        self.assertIn("<strong>client-side script</strong>", html)

    def test_adjacent_bold_and_italic_spans_do_not_cross_eat_asterisks(self):
        html = render_inline_markdown(
            "Use *sparingly*, and only with **explicit approval** from the owner."
        )
        self.assertIn("<em>sparingly</em>", html)
        self.assertIn("<strong>explicit approval</strong>", html)

    def test_angle_bracket_placeholder_text_is_escaped_not_treated_as_html(self):
        # Real construct from reading-stack-traces.md: "*<caller>* passed
        # *<value>* to *<callee>*" -- italic emphasis around literal
        # angle-bracket placeholder text, not an HTML tag.
        html = render_inline_markdown("*<caller>* passed *<value>* to *<callee>*.")
        self.assertIn("<em>&lt;caller&gt;</em>", html)
        self.assertIn("<em>&lt;value&gt;</em>", html)
        self.assertIn("<em>&lt;callee&gt;</em>", html)

    def test_ampersand_and_angle_brackets_are_escaped_in_plain_text(self):
        html = render_inline_markdown("A & B is not the same as A<B or A>B.")
        self.assertEqual(
            html, "A &amp; B is not the same as A&lt;B or A&gt;B."
        )

    def test_escape_html_helper_covers_amp_lt_gt_and_quote(self):
        self.assertEqual(
            escape_html("""<a> & "b" """), "&lt;a&gt; &amp; &quot;b&quot; "
        )


# ---------------------------------------------------------------------------
# Block rendering: headings, lists (incl. nesting/continuation), tables,
# fenced code, paragraphs
# ---------------------------------------------------------------------------


class HeadingRenderingTests(unittest.TestCase):
    def test_h2_and_h3_get_kebab_case_ids(self):
        html = render_markdown_body("## Two shapes, and which applies\n\n### A sub-point\n\nBody.\n")
        self.assertIn('<h2 id="two-shapes-and-which-applies">', html)
        self.assertIn('<h3 id="a-sub-point">', html)

    def test_duplicate_heading_text_within_one_document_gets_deduped_ids(self):
        markdown = "## Rules\n\nFirst section.\n\n## Rules\n\nSecond section, same title.\n"
        html = render_markdown_body(markdown)
        self.assertIn('id="rules"', html)
        self.assertIn('id="rules-2"', html)

    def test_heading_id_prefix_disambiguates_across_inlined_sections(self):
        # Mirrors formidable: many stack files share a heading verbatim
        # ("Audit hooks"); once inlined onto one page their ids must not
        # collide, or the anchors point at the wrong section.
        web_html = render_markdown_body(
            "## Audit hooks\n\nWeb-specific detail.\n", heading_id_prefix="stack-web-"
        )
        cli_html = render_markdown_body(
            "## Audit hooks\n\nCLI-specific detail.\n", heading_id_prefix="stack-cli-output-"
        )
        self.assertIn('id="stack-web-audit-hooks"', web_html)
        self.assertIn('id="stack-cli-output-audit-hooks"', cli_html)

    def test_heading_with_inline_code_gets_a_clean_slug_and_rendered_code_span(self):
        # Real heading from portable-shell-scripting.md: "`set -e` and its
        # documented holes".
        html = render_markdown_body("## `set -e` and its documented holes\n\nBody.\n")
        self.assertIn('<h2 id="set-e-and-its-documented-holes">', html)
        self.assertIn("<code>set -e</code> and its documented holes", html)

    def test_slugify_strips_apostrophes_rather_than_hyphenating_them(self):
        self.assertEqual(slugify("Amdahl's check"), "amdahls-check")


class ListRenderingTests(unittest.TestCase):
    def test_nested_bullet_list_renders_one_level_deep(self):
        html = render_markdown_body(NESTED_LIST_MARKDOWN)
        self.assertEqual(html.count("<ul>"), 2)  # outer list + one nested list
        self.assertIn("<strong>Single tenant</strong>", html)
        self.assertIn("<strong>Full outage</strong>", html)

    def test_numbered_list_renders_as_ordered_list(self):
        html = render_markdown_body("1. First step.\n2. Second step.\n3. Third step.\n")
        self.assertIn("<ol>", html)
        self.assertEqual(html.count("<li>"), 3)

    def test_unmarked_continuation_line_folds_into_previous_item_not_a_new_one(self):
        html = render_markdown_body(LIST_WITH_UNMARKED_CONTINUATION_LINE)
        self.assertEqual(html.count("<li>"), 3)
        self.assertIn(
            "the strongest alternative, nothing else gets a full writeup.", html
        )

    def test_paragraph_immediately_followed_by_list_with_no_blank_line(self):
        html = render_markdown_body(PARAGRAPH_DIRECTLY_FOLLOWED_BY_LIST)
        self.assertIn("<p>Two passes, each cheap:</p>", html)
        self.assertIn("<ol>", html)
        self.assertEqual(html.count("<li>"), 2)


class TableRenderingTests(unittest.TestCase):
    def test_simple_table_renders_thead_and_tbody(self):
        markdown = (
            "| Symptom | Real cause |\n"
            "|---|---|\n"
            "| Bisect lands on a huge commit | Boundaries drawn by time of day |\n"
        )
        html = render_markdown_body(markdown)
        self.assertIn("<table>", html)
        self.assertIn("<thead>", html)
        self.assertIn("<tbody>", html)
        self.assertEqual(html.count("<th>"), 2)
        self.assertEqual(html.count("<td>"), 2)

    def test_table_cell_with_bold_renders_inline_markup(self):
        markdown = (
            "| Approach | Notes |\n"
            "|---|---|\n"
            "| Range pin | Needs a **documented default** |\n"
        )
        html = render_markdown_body(markdown)
        self.assertIn("<td>Needs a <strong>documented default</strong></td>", html)

    def test_escaped_pipe_in_table_cell_code_span_keeps_correct_column_count(self):
        html = render_markdown_body(TABLE_WITH_ESCAPED_PIPE_MARKDOWN)
        self.assertEqual(html.count("<th>"), 3)
        self.assertEqual(html.count("<td>"), 6)  # two body rows x 3 columns
        # The escaped pipes must survive as literal `|` characters inside the
        # rendered <code> span, not as extra table columns.
        self.assertIn("git log --oneline --since=1.week | wc -l", html)
        self.assertIn("<code>[ -e &quot;$f&quot; ] || continue</code>", html)


class FencedCodeBlockTests(unittest.TestCase):
    def test_fenced_code_block_is_literal_not_a_list_or_markup(self):
        html = render_markdown_body(FENCED_CODE_WITH_NUMBERED_LOOKING_LINES)
        self.assertIn("<pre><code>", html)
        self.assertNotIn("<ol>", html)
        self.assertIn("1. add new field, optional          deploy", html)
        # Markup outside the fence still renders normally.
        self.assertIn("<em>do not</em>", html)

    def test_fenced_code_block_content_is_html_escaped(self):
        markdown = "```\nif a < b && c > d:\n```\n"
        html = render_markdown_body(markdown)
        self.assertIn("a &lt; b &amp;&amp; c &gt; d", html)

    def test_language_tag_after_fence_marker_is_discarded_not_rendered(self):
        markdown = "```sh\necho hi\n```\n"
        html = render_markdown_body(markdown)
        self.assertIn("<pre><code>echo hi</code></pre>", html)
        self.assertNotIn("sh\n", html)


class ParagraphRenderingTests(unittest.TestCase):
    def test_blank_line_separated_text_becomes_two_paragraphs(self):
        html = render_markdown_body("First paragraph.\n\nSecond paragraph.\n")
        self.assertEqual(html.count("<p>"), 2)


# ---------------------------------------------------------------------------
# formidable-style relative link resolution
# ---------------------------------------------------------------------------


class FormidableLinkResolutionTests(unittest.TestCase):
    def setUp(self):
        self.resolve = make_formidable_link_resolver(
            {"web": "stack-web", "craft-floor": "cmd-craft-floor"}
        )

    def test_relative_link_resolves_to_matching_anchor(self):
        html = render_markdown_body(
            FORMIDABLE_STYLE_LINKS_MARKDOWN, resolve_relative_link=self.resolve
        )
        self.assertIn('href="#stack-web"', html)
        self.assertIn('href="#cmd-craft-floor"', html)

    def test_absolute_url_passes_through_unchanged(self):
        html = render_markdown_body(
            FORMIDABLE_STYLE_LINKS_MARKDOWN, resolve_relative_link=self.resolve
        )
        self.assertIn('href="https://www.w3.org/TR/WCAG21/"', html)

    def test_unresolvable_relative_link_drops_href_but_keeps_text(self):
        html = render_markdown_body(
            FORMIDABLE_STYLE_LINKS_MARKDOWN, resolve_relative_link=self.resolve
        )
        self.assertIn("an old doc", html)
        self.assertNotIn("retired-notes", html)

    def test_no_resolver_provided_drops_relative_links_but_keeps_absolute(self):
        html = render_markdown_body(FORMIDABLE_STYLE_LINKS_MARKDOWN)
        self.assertIn('href="https://www.w3.org/TR/WCAG21/"', html)
        self.assertNotIn("reference/stacks/web.md", html)
        self.assertIn("stacks/web.md", html)  # visible text survives


# ---------------------------------------------------------------------------
# Frontmatter parsing + title stripping
# ---------------------------------------------------------------------------


class FrontmatterAndTitleTests(unittest.TestCase):
    def test_split_frontmatter_extracts_name_and_description_verbatim(self):
        fields, body = split_frontmatter(SAMPLE_FRONTMATTER_SKILL_MD)
        self.assertEqual(fields["name"], "rotating-credentials")
        self.assertEqual(
            fields["description"],
            "Use when a credential has leaked, when an access key has outlived "
            "its owner, or when deciding a rotation cadence for a service account.",
        )
        self.assertTrue(body.lstrip().startswith("# Rotating credentials"))

    def test_strip_title_heading_removes_only_the_leading_h1(self):
        _, body = split_frontmatter(SAMPLE_FRONTMATTER_SKILL_MD)
        stripped = strip_title_heading(body)
        self.assertNotIn("# Rotating credentials", stripped)
        self.assertIn("## When to use", stripped)

    def test_humanize_filename_uppercases_known_acronyms(self):
        self.assertEqual(humanize_filename("web"), "Web")
        self.assertEqual(humanize_filename("cli-output"), "CLI Output")
        self.assertEqual(humanize_filename("native-mobile"), "Native Mobile")
        self.assertEqual(humanize_filename("xr-spatial"), "XR Spatial")
        self.assertEqual(humanize_filename("terminal-tui"), "Terminal TUI")


# ---------------------------------------------------------------------------
# End-to-end integration against the real skills library
# ---------------------------------------------------------------------------


@unittest.skipUnless(REAL_SKILLS_ROOT.is_dir(), "real TBaguette skills directory not present")
class BuildContentIntegrationTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.content = build_content(str(REAL_SKILLS_ROOT))

    def test_finds_exactly_65_skills(self):
        self.assertEqual(len(self.content["skills"]), 65)

    def test_ten_categories_in_the_locked_order(self):
        actual_slugs = [c["slug"] for c in self.content["categories"]]
        expected_slugs = [c["slug"] for c in CATEGORIES]
        self.assertEqual(actual_slugs, expected_slugs)
        self.assertEqual(len(actual_slugs), 10)

    def test_category_skill_slug_counts_sum_to_65(self):
        total = sum(len(c["skill_slugs"]) for c in self.content["categories"])
        self.assertEqual(total, 65)

    def test_formidable_entry_has_stacks_commands_and_craft_floor(self):
        formidable = self.content["skills"]["formidable"]
        self.assertTrue(formidable["is_formidable"])
        self.assertEqual(len(formidable["formidable_stacks"]), 12)
        self.assertEqual(len(formidable["formidable_commands"]), 11)
        self.assertIn("formidable_craft_floor_html", formidable)

        stack_ids = [s["id"] for s in formidable["formidable_stacks"]]
        command_ids = [c["id"] for c in formidable["formidable_commands"]]
        self.assertEqual(len(stack_ids), len(set(stack_ids)))
        self.assertEqual(len(command_ids), len(set(command_ids)))
        self.assertNotIn("cmd-craft-floor", command_ids)
        self.assertNotIn("<h1", formidable["body_html"])

    def test_non_formidable_skill_has_no_formidable_only_fields(self):
        skill = self.content["skills"]["atomic-commits"]
        self.assertFalse(skill["is_formidable"])
        self.assertNotIn("formidable_stacks", skill)
        self.assertNotIn("formidable_commands", skill)

    def test_every_skill_body_html_has_no_leading_h1(self):
        for slug, skill in self.content["skills"].items():
            with self.subTest(skill=slug):
                self.assertNotIn("<h1", skill["body_html"])

    def test_every_summary_within_length_budget(self):
        for slug, skill in self.content["skills"].items():
            with self.subTest(skill=slug):
                self.assertLessEqual(len(skill["summary"]), 140)

    def test_content_is_fully_json_serializable(self):
        serialized = json.dumps(self.content)
        self.assertGreater(len(serialized), 0)


if __name__ == "__main__":
    unittest.main(verbosity=2)
