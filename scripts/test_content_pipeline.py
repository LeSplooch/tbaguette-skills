"""Self-tests for content_pipeline.py.

Run with:
    python3 -m unittest scripts.test_content_pipeline -v
or directly:
    python3 scripts/test_content_pipeline.py

Fixtures are written to look like the real TBaguette skill files rather than
minimal placeholders ("abc", "foo bar") -- several of them are shaped after
real constructs found while surveying the 66-skill corpus (escaped pipes in
a table cell's inline code, a list item that wraps onto a continuation line
with no marker, a paragraph directly followed by a list with no blank line),
because those are exactly the cases a minimal fixture would never exercise.
"""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import content_pipeline
from content_pipeline import (
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

# Real, committed Chinese content -- not a synthetic fixture. Bug A (CJK
# punctuation-blind summarize_description) was only ever surfaced by this
# real content in the first place: a hand-written ASCII-only fixture has no
# way to exercise a full-width 。／，／、／； character at all. Reading the
# live file directly (rather than hand-copying a string into this module)
# means these tests stay honest as the content evolves and there's no
# transcription step that could silently drift from the real string.
REAL_ZH_DESCRIPTIONS_PATH = Path(__file__).resolve().parent.parent / "i18n" / "zh" / "descriptions.json"

# Real, committed Arabic content, for exactly the same reason. Arabic's own
# clause punctuation (، U+060C, ؛ U+061B) is Unicode-distinct from the ASCII
# marks and appears nowhere in a hand-typed English fixture, so only the real
# corpus exercises it -- the same lesson the CJK set above already taught.
REAL_AR_DESCRIPTIONS_PATH = Path(__file__).resolve().parent.parent / "i18n" / "ar" / "descriptions.json"


def _real_zh_description(slug: str) -> str:
    data = json.loads(REAL_ZH_DESCRIPTIONS_PATH.read_text(encoding="utf-8"))
    return data[slug]


def _real_ar_descriptions() -> dict:
    return json.loads(REAL_AR_DESCRIPTIONS_PATH.read_text(encoding="utf-8"))


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

    # -- CJK punctuation (Bug A) -------------------------------------------
    #
    # Chinese uses full-width sentence-final punctuation (。！？) and has no
    # spaces between words at all, so the ASCII-only _SENTENCE_START_RE and
    # the rfind(",")/rfind(" ") clause-cut logic can't see any of the real
    # boundaries in Chinese prose. Every fixture below is copied live from
    # the real, committed i18n/zh/descriptions.json via _real_zh_description
    # rather than a hand-typed placeholder, per this project's own repeated
    # lesson that synthetic fixtures miss what real translated content
    # actually triggers.

    def test_cjk_sentence_fitting_within_budget_returns_first_sentence_unchanged(self):
        # testing-the-untestable's zh description: the first
        # ideographic-full-stop-terminated sentence is 138 chars (fits the
        # 140 budget), but the full two-sentence string runs to 160. The
        # pre-fix regex never matches a CJK string at all (no ASCII
        # .!?), so it always fell through to treating the *entire*
        # normalized string as "the first sentence" -- silently including
        # the second "Covers ..."-equivalent sentence whenever the whole
        # thing happened to still fit under 140. The fix must recognize 。
        # as a real sentence boundary and stop there.
        if not REAL_ZH_DESCRIPTIONS_PATH.is_file():
            self.skipTest("real i18n/zh/descriptions.json not present")
        description = _real_zh_description("testing-the-untestable")
        summary = summarize_description(description)
        self.assertLessEqual(len(summary), 140)
        self.assertFalse(summary.endswith("…"))
        first_sentence_end = description.index("。") + 1
        self.assertEqual(summary, description[:first_sentence_end])

    def test_cjk_description_needing_clause_cut_stops_at_a_real_cjk_boundary(self):
        # tracing-data-flow's zh description is the concrete bug report:
        # pre-fix, this collapses to ~20 characters, because the
        # regex/rfind calls only recognize ASCII . ! ? , and space, so they
        # latch onto the single incidental ASCII space inside "为 null" --
        # a Latin loanword, not a real word boundary -- since it's the
        # earliest "boundary" of any kind in the whole string. The real
        # Chinese clause marks (、，；) that a human would actually cut at
        # appear much later and were invisible to the old code entirely.
        if not REAL_ZH_DESCRIPTIONS_PATH.is_file():
            self.skipTest("real i18n/zh/descriptions.json not present")
        description = _real_zh_description("tracing-data-flow")
        summary = summarize_description(description)
        self.assertLessEqual(len(summary), 140)
        self.assertTrue(summary.endswith("…"))
        core = summary[:-1]
        self.assertTrue(description.startswith(core))
        boundary_char = description[len(core)]
        self.assertIn(boundary_char, (" ", ",", "，", "、", "；"))
        # The bug's concrete, reported failure mode was collapsing to ~20
        # characters -- this is the sharpest possible regression guard for
        # it (the fixed value is 139).
        self.assertGreater(len(summary), 100)

    def test_cjk_description_with_only_semicolons_before_budget_still_finds_a_clause(self):
        # atomic-commits' first sentence uses only ； as a clause separator
        # anywhere near the front (no ，or 、 appears that early) -- real
        # content that specifically exercises the semicolon branch. If the
        # fix omitted ； from the clause-boundary set, this description
        # would fall through to the ASCII-space fallback and cut on the
        # incidental space inside the Latin loanword "diff", the same bug
        # shape as tracing-data-flow, just less severe.
        if not REAL_ZH_DESCRIPTIONS_PATH.is_file():
            self.skipTest("real i18n/zh/descriptions.json not present")
        description = _real_zh_description("atomic-commits")
        summary = summarize_description(description)
        self.assertLessEqual(len(summary), 140)
        self.assertTrue(summary.endswith("…"))
        core = summary[:-1]
        self.assertTrue(description.startswith(core))
        boundary_char = description[len(core)]
        self.assertIn(boundary_char, (" ", ",", "，", "、", "；"))
        self.assertFalse(
            summary.endswith("diff…"),
            "must not cut on the incidental space inside an English loanword",
        )

    # -- Arabic punctuation -------------------------------------------------
    #
    # Arabic writes its clause punctuation with ، (U+060C) and ؛ (U+061B),
    # not the ASCII , and ; -- distinct code points, mirrored glyphs. Before
    # they were recognized, the clause-cut branch found nothing in any of the
    # 66 real Arabic descriptions and every teaser fell through to the
    # word-boundary fallback, stopping mid-thought. All fixtures below come
    # from the live i18n/ar/descriptions.json, per the CJK lesson above.

    ARABIC_COMMA = "،"
    ARABIC_SEMICOLON = "؛"

    def test_arabic_corpus_uses_arabic_clause_marks_and_no_ascii_comma(self):
        # The premise the fix rests on. If a future re-translation started
        # using ASCII commas this would no longer be the bug being fixed, and
        # the assertions below would be testing something else by accident.
        if not REAL_AR_DESCRIPTIONS_PATH.is_file():
            self.skipTest("real i18n/ar/descriptions.json not present")
        corpus = _real_ar_descriptions()
        self.assertEqual(len(corpus), 66)
        self.assertEqual(
            [slug for slug, text in corpus.items() if "," in text], [],
            "no real Arabic description should contain an ASCII comma",
        )
        with_arabic_comma = [s for s, t in corpus.items() if self.ARABIC_COMMA in t]
        self.assertGreater(len(with_arabic_comma), 50)

    def test_arabic_description_cuts_at_a_real_arabic_clause_boundary(self):
        # calibrating-confidence is the concrete report: pre-fix this stopped
        # on the bare connective "أو" ("or"), reading as an unfinished
        # sentence. The cut must land on a real clause mark instead.
        if not REAL_AR_DESCRIPTIONS_PATH.is_file():
            self.skipTest("real i18n/ar/descriptions.json not present")
        description = _real_ar_descriptions()["calibrating-confidence"]
        summary = summarize_description(description)
        self.assertLessEqual(len(summary), 140)
        self.assertTrue(summary.endswith("…"))
        core = summary[:-1]
        self.assertTrue(description.startswith(core))
        self.assertEqual(description[len(core)], self.ARABIC_COMMA)
        self.assertFalse(
            core.rstrip().endswith("أو"),
            "must not stop on the dangling connective 'أو' (or)",
        )

    def test_arabic_semicolon_alone_is_enough_to_find_a_clause(self):
        # drawing-boundaries reaches the budget with ؛ as the only clause
        # mark in the window -- the Arabic counterpart of the ；-only case the
        # CJK tests cover, and the reason ؛ cannot be dropped from the set.
        # It is not a lone example: 10 of the 66 entries cut on a ؛, and all
        # 10 change if it is removed.
        if not REAL_AR_DESCRIPTIONS_PATH.is_file():
            self.skipTest("real i18n/ar/descriptions.json not present")
        description = _real_ar_descriptions()["drawing-boundaries"]
        truncated = " ".join(description.split())[:139]
        self.assertNotIn(self.ARABIC_COMMA, truncated,
                         "fixture must exercise the semicolon branch specifically")
        self.assertIn(self.ARABIC_SEMICOLON, truncated)
        summary = summarize_description(description)
        core = summary[:-1]
        self.assertEqual(description[len(core)], self.ARABIC_SEMICOLON)

    def test_no_arabic_teaser_strands_a_clause_mark_against_the_ellipsis(self):
        # The rstrip set has to track the cut set. When it didn't, 6 of the 66
        # teasers rendered a stray ، immediately before the … .
        if not REAL_AR_DESCRIPTIONS_PATH.is_file():
            self.skipTest("real i18n/ar/descriptions.json not present")
        offenders = []
        for slug, description in _real_ar_descriptions().items():
            summary = summarize_description(description)
            core = summary[:-1] if summary.endswith("…") else summary
            if core and core[-1] in ",;" + self.ARABIC_COMMA + self.ARABIC_SEMICOLON:
                offenders.append(slug)
        self.assertEqual(offenders, [])

    def test_every_real_arabic_teaser_stays_within_budget(self):
        if not REAL_AR_DESCRIPTIONS_PATH.is_file():
            self.skipTest("real i18n/ar/descriptions.json not present")
        for slug, description in _real_ar_descriptions().items():
            with self.subTest(skill=slug):
                self.assertLessEqual(len(summarize_description(description)), 140)


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
        self.assertIn('<pre class="prose-code-block"><code>', html)
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
        self.assertIn('<pre class="prose-code-block"><code>echo hi</code></pre>', html)
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

    def test_finds_exactly_66_skills(self):
        self.assertEqual(len(self.content["skills"]), 66)

    def test_ten_categories_in_the_locked_order(self):
        actual_slugs = [c["slug"] for c in self.content["categories"]]
        expected_slugs = [c["slug"] for c in CATEGORIES]
        self.assertEqual(actual_slugs, expected_slugs)
        self.assertEqual(len(actual_slugs), 10)

    def test_category_skill_slug_counts_sum_to_66(self):
        total = sum(len(c["skill_slugs"]) for c in self.content["categories"])
        self.assertEqual(total, 66)

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


class LocaleBuildTests(unittest.TestCase):
    def test_locale_build_falls_back_to_english_per_file(self):
        """A locale directory missing a skill's SKILL.md still produces a
        complete page in English for that skill, translated=False; a locale
        directory that has the file produces translated=True."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            skills_root = tmp_path / "skills"
            locale_root = tmp_path / "i18n" / "xx"

            alpha_dir = skills_root / "alpha"
            alpha_dir.mkdir(parents=True)
            (alpha_dir / "SKILL.md").write_text(
                "---\nname: alpha\ndescription: English alpha description.\n---\nEnglish alpha body.\n",
                encoding="utf-8",
            )

            beta_dir = skills_root / "beta"
            beta_dir.mkdir(parents=True)
            (beta_dir / "SKILL.md").write_text(
                "---\nname: beta\ndescription: English beta description.\n---\nEnglish beta body.\n",
                encoding="utf-8",
            )

            # Only beta gets a real translated SKILL.md in the locale dir.
            locale_beta_dir = locale_root / "skills" / "beta"
            locale_beta_dir.mkdir(parents=True)
            (locale_beta_dir / "SKILL.md").write_text(
                "---\nname: beta\ndescription: XX beta description.\n---\nXX beta body.\n",
                encoding="utf-8",
            )

            categories = [{"slug": "test-cat", "title": "Test", "skill_slugs": ["alpha", "beta"]}]
            with mock.patch.object(content_pipeline, "CATEGORIES", categories):
                content = content_pipeline.build_content(
                    str(skills_root), locale="xx", locale_root=str(locale_root)
                )

            alpha = content["skills"]["alpha"]
            self.assertFalse(alpha["translated"])
            self.assertEqual(alpha["description"], "English alpha description.")
            self.assertIn("English alpha body", alpha["body_html"])

            beta = content["skills"]["beta"]
            self.assertTrue(beta["translated"])
            self.assertEqual(beta["description"], "XX beta description.")
            self.assertIn("XX beta body", beta["body_html"])

    def test_locale_build_description_precedence(self):
        """descriptions.json translates a card/title ahead of a full
        SKILL.md; once a real translated SKILL.md exists, its own
        frontmatter description wins even if descriptions.json still has
        an entry for that slug."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            skills_root = tmp_path / "skills"
            locale_root = tmp_path / "i18n" / "xx"

            for slug in ("gamma", "delta"):
                skill_dir = skills_root / slug
                skill_dir.mkdir(parents=True)
                (skill_dir / "SKILL.md").write_text(
                    f"---\nname: {slug}\ndescription: English {slug} description.\n---\nEnglish body.\n",
                    encoding="utf-8",
                )

            locale_root.mkdir(parents=True)
            (locale_root / "descriptions.json").write_text(
                '{"gamma": "XX gamma description (from descriptions.json).", '
                '"delta": "XX delta description (should be ignored, SKILL.md wins)."}',
                encoding="utf-8",
            )
            locale_delta_dir = locale_root / "skills" / "delta"
            locale_delta_dir.mkdir(parents=True)
            (locale_delta_dir / "SKILL.md").write_text(
                "---\nname: delta\ndescription: XX delta description (from its own SKILL.md).\n---\nXX delta body.\n",
                encoding="utf-8",
            )

            categories = [{"slug": "test-cat", "title": "Test", "skill_slugs": ["gamma", "delta"]}]
            with mock.patch.object(content_pipeline, "CATEGORIES", categories):
                content = content_pipeline.build_content(
                    str(skills_root), locale="xx", locale_root=str(locale_root)
                )

            gamma = content["skills"]["gamma"]
            self.assertFalse(gamma["translated"])
            self.assertEqual(gamma["description"], "XX gamma description (from descriptions.json).")

            delta = content["skills"]["delta"]
            self.assertTrue(delta["translated"])
            self.assertEqual(delta["description"], "XX delta description (from its own SKILL.md).")

    def test_locale_build_category_title_fallback(self):
        """categories.json translates a category title; a category missing
        from categories.json falls back to the English title."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            skills_root = tmp_path / "skills"
            locale_root = tmp_path / "i18n" / "xx"
            for slug in ("epsilon",):
                skill_dir = skills_root / slug
                skill_dir.mkdir(parents=True)
                (skill_dir / "SKILL.md").write_text(
                    f"---\nname: {slug}\ndescription: English description.\n---\nBody.\n",
                    encoding="utf-8",
                )
            locale_root.mkdir(parents=True)
            (locale_root / "categories.json").write_text(
                '{"translated-cat": "XX Translated Title"}', encoding="utf-8"
            )

            categories = [
                {"slug": "translated-cat", "title": "English Translated Cat", "skill_slugs": ["epsilon"]},
            ]
            with mock.patch.object(content_pipeline, "CATEGORIES", categories):
                content = content_pipeline.build_content(
                    str(skills_root), locale="xx", locale_root=str(locale_root)
                )
            self.assertEqual(content["categories"][0]["title"], "XX Translated Title")

    def test_locale_build_formidable_with_mixed_per_file_fallback(self):
        """Formidable's per-file fallback extends to every reference/*.md and
        reference/stacks/*.md file independently. The entry's translated flag is
        False when even one file is untranslated; each fragment's HTML reflects
        whether that specific file came from the locale dir or fell back to English."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            skills_root = tmp_path / "skills"
            locale_root = tmp_path / "i18n" / "xx"

            # Create English formidable skill with multiple reference files and stacks
            formidable_dir = skills_root / "formidable"
            formidable_dir.mkdir(parents=True)
            (formidable_dir / "SKILL.md").write_text(
                "---\nname: formidable\ndescription: English formidable description.\n---\nEnglish formidable body.\n",
                encoding="utf-8",
            )

            reference_dir = formidable_dir / "reference"
            reference_dir.mkdir(parents=True)
            (reference_dir / "shape.md").write_text(
                "## Shape\nEnglish shape content.\n",
                encoding="utf-8",
            )
            (reference_dir / "craft-floor.md").write_text(
                "## Craft Floor\nEnglish craft floor content.\n",
                encoding="utf-8",
            )

            stacks_dir = reference_dir / "stacks"
            stacks_dir.mkdir(parents=True)
            (stacks_dir / "web.md").write_text(
                "## Web\nEnglish web stack content.\n",
                encoding="utf-8",
            )
            (stacks_dir / "terminal.md").write_text(
                "## Terminal\nEnglish terminal stack content.\n",
                encoding="utf-8",
            )

            # Mixed-translation case: only web.md is translated; everything
            # else (SKILL.md, shape.md, terminal.md, craft-floor.md)
            # falls back to English.
            locale_formidable_dir = locale_root / "skills" / "formidable"
            locale_formidable_dir.mkdir(parents=True)
            locale_reference_dir = locale_formidable_dir / "reference"
            locale_reference_dir.mkdir(parents=True)
            locale_stacks_dir = locale_reference_dir / "stacks"
            locale_stacks_dir.mkdir(parents=True)

            # Only translate one stack file
            (locale_stacks_dir / "web.md").write_text(
                "## Web\nXX web stack content.\n",
                encoding="utf-8",
            )
            # Intentionally do not create stack-terminal.md, cmd-shape.md, or
            # craft-floor.md in the locale dir to test fallback.

            categories = [{"slug": "test-cat", "title": "Test", "skill_slugs": ["formidable"]}]
            with mock.patch.object(content_pipeline, "CATEGORIES", categories):
                content = content_pipeline.build_content(
                    str(skills_root), locale="xx", locale_root=str(locale_root)
                )

            formidable = content["skills"]["formidable"]

            # Overall entry should be translated=False because not all files are
            # translated (the resolve() closure's all_translated flag is False
            # after any file fails the is_file() check).
            self.assertFalse(formidable["translated"])

            # Main SKILL.md fell back to English
            self.assertEqual(formidable["description"], "English formidable description.")
            self.assertIn("English formidable body", formidable["body_html"])

            # Each stack fragment's HTML reflects its own translation state
            web_stack = next(
                (s for s in formidable["formidable_stacks"] if s["id"] == "stack-web"),
                None,
            )
            self.assertIsNotNone(web_stack)
            self.assertIn("XX web stack content", web_stack["html"])

            terminal_stack = next(
                (s for s in formidable["formidable_stacks"] if s["id"] == "stack-terminal"),
                None,
            )
            self.assertIsNotNone(terminal_stack)
            self.assertIn("English terminal stack content", terminal_stack["html"])

            # Command fragments also reflect their translation state
            shape_cmd = next(
                (c for c in formidable["formidable_commands"] if c["id"] == "cmd-shape"),
                None,
            )
            self.assertIsNotNone(shape_cmd)
            self.assertIn("English shape content", shape_cmd["html"])

            # craft-floor is optional but if present should also fall back
            self.assertIn("formidable_craft_floor_html", formidable)
            self.assertIn("English craft floor content", formidable["formidable_craft_floor_html"])

    def test_locale_build_formidable_fully_translated(self):
        """When all formidable files are translated (SKILL.md, every
        reference/*.md, every reference/stacks/*.md including craft-floor if
        present), the entry's translated flag is True and all fragments show
        translated content."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            skills_root = tmp_path / "skills"
            locale_root = tmp_path / "i18n" / "xx"

            # Create English formidable skill
            formidable_dir = skills_root / "formidable"
            formidable_dir.mkdir(parents=True)
            (formidable_dir / "SKILL.md").write_text(
                "---\nname: formidable\ndescription: English formidable description.\n---\nEnglish formidable body.\n",
                encoding="utf-8",
            )

            reference_dir = formidable_dir / "reference"
            reference_dir.mkdir(parents=True)
            (reference_dir / "shape.md").write_text(
                "## Shape\nEnglish shape content.\n",
                encoding="utf-8",
            )
            (reference_dir / "craft-floor.md").write_text(
                "## Craft Floor\nEnglish craft floor content.\n",
                encoding="utf-8",
            )

            stacks_dir = reference_dir / "stacks"
            stacks_dir.mkdir(parents=True)
            (stacks_dir / "web.md").write_text(
                "## Web\nEnglish web stack content.\n",
                encoding="utf-8",
            )

            # Fully translate all files: SKILL.md, shape.md, web.md, craft-floor.md
            locale_formidable_dir = locale_root / "skills" / "formidable"
            locale_formidable_dir.mkdir(parents=True)
            (locale_formidable_dir / "SKILL.md").write_text(
                "---\nname: formidable\ndescription: XX formidable description.\n---\nXX formidable body.\n",
                encoding="utf-8",
            )

            locale_reference_dir = locale_formidable_dir / "reference"
            locale_reference_dir.mkdir(parents=True)
            (locale_reference_dir / "shape.md").write_text(
                "## Shape\nXX shape content.\n",
                encoding="utf-8",
            )
            (locale_reference_dir / "craft-floor.md").write_text(
                "## Craft Floor\nXX craft floor content.\n",
                encoding="utf-8",
            )

            locale_stacks_dir = locale_reference_dir / "stacks"
            locale_stacks_dir.mkdir(parents=True)
            (locale_stacks_dir / "web.md").write_text(
                "## Web\nXX web stack content.\n",
                encoding="utf-8",
            )

            categories = [{"slug": "test-cat", "title": "Test", "skill_slugs": ["formidable"]}]
            with mock.patch.object(content_pipeline, "CATEGORIES", categories):
                content = content_pipeline.build_content(
                    str(skills_root), locale="xx", locale_root=str(locale_root)
                )

            formidable = content["skills"]["formidable"]

            # All files are translated, so overall entry should be translated=True
            # (the resolve() closure's all_translated flag remains True after all
            # files pass the is_file() check).
            self.assertTrue(formidable["translated"])

            # Main SKILL.md is translated
            self.assertEqual(formidable["description"], "XX formidable description.")
            self.assertIn("XX formidable body", formidable["body_html"])

            # All stack fragments show translated content
            web_stack = next(
                (s for s in formidable["formidable_stacks"] if s["id"] == "stack-web"),
                None,
            )
            self.assertIsNotNone(web_stack)
            self.assertIn("XX web stack content", web_stack["html"])

            # All command fragments show translated content
            shape_cmd = next(
                (c for c in formidable["formidable_commands"] if c["id"] == "cmd-shape"),
                None,
            )
            self.assertIsNotNone(shape_cmd)
            self.assertIn("XX shape content", shape_cmd["html"])

            # craft-floor also shows translated content
            self.assertIn("formidable_craft_floor_html", formidable)
            self.assertIn("XX craft floor content", formidable["formidable_craft_floor_html"])

    def test_default_build_has_no_locale_regressions(self):
        """build_content(skills_root) with no locale args still returns
        translated=True for every skill (English is trivially "in its own
        language") and is otherwise unaffected -- the byte-identical-output
        contract this whole feature is built on."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            skills_root = tmp_path / "skills"
            skill_dir = skills_root / "solo"
            skill_dir.mkdir(parents=True)
            (skill_dir / "SKILL.md").write_text(
                "---\nname: solo\ndescription: Solo description.\n---\nSolo body.\n",
                encoding="utf-8",
            )
            categories = [{"slug": "test-cat", "title": "Test", "skill_slugs": ["solo"]}]
            with mock.patch.object(content_pipeline, "CATEGORIES", categories):
                content = content_pipeline.build_content(str(skills_root))
            self.assertTrue(content["skills"]["solo"]["translated"])

    def test_locale_build_translates_per_skill_category_title_too(self):
        """categories.json's translation must reach each skill's own
        category_title field, not just the top-level categories list --
        this is what the card tag, breadcrumb, see-also heading, and page
        <title> actually read."""
        with tempfile.TemporaryDirectory() as tmp:
            tmp_path = Path(tmp)
            skills_root = tmp_path / "skills"
            locale_root = tmp_path / "i18n" / "xx"

            skill_dir = skills_root / "zeta"
            skill_dir.mkdir(parents=True)
            (skill_dir / "SKILL.md").write_text(
                "---\nname: zeta\ndescription: English zeta description.\n---\nBody.\n",
                encoding="utf-8",
            )
            locale_root.mkdir(parents=True)
            (locale_root / "categories.json").write_text(
                '{"test-cat": "XX Translated Category"}', encoding="utf-8"
            )

            categories = [{"slug": "test-cat", "title": "English Test Cat", "skill_slugs": ["zeta"]}]
            with mock.patch.object(content_pipeline, "CATEGORIES", categories):
                content = content_pipeline.build_content(
                    str(skills_root), locale="xx", locale_root=str(locale_root)
                )
            self.assertEqual(content["categories"][0]["title"], "XX Translated Category")
            self.assertEqual(
                content["skills"]["zeta"]["category_title"], "XX Translated Category",
            )


if __name__ == "__main__":
    unittest.main(verbosity=2)
