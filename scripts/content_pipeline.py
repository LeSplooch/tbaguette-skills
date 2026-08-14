"""Content-extraction pipeline for the TBaguette skills showcase site.

Reads the 66 skill directories under ``~/.claude/skills/TBaguette/skills/``
(each a ``SKILL.md`` with YAML frontmatter and a markdown body; one skill,
``formidable``, additionally carries a ``reference/`` tree of stack and
command reference files) and produces a single JSON-serializable dict that
matches the locked ``content.json`` schema shared with the site's HTML/CSS/JS
half. See the module-level constant ``CATEGORIES`` for the authoritative
category grouping, and ``build_content`` for the entry point.

Stdlib only, by design: this is the site's only build step and it must not
require an install step.

Run directly to (re)generate ``scripts/content.json`` from the real skills
directory:

    python3 scripts/content_pipeline.py
"""

from __future__ import annotations

import json
import re
from collections.abc import Callable
from pathlib import Path, PurePosixPath

# ---------------------------------------------------------------------------
# The authoritative category grouping. This is fixed by the site's design
# spec, not re-derived from CATALOG.md or any other heuristic -- order here
# is the display order, and skill_slugs order within a category is preserved
# verbatim into the output.
# ---------------------------------------------------------------------------

CATEGORIES: list[dict] = [
    {
        "slug": "ui-and-design",
        "title": "UI and design",
        "skill_slugs": ["formidable"],
    },
    {
        "slug": "judgment-and-meta",
        "title": "Judgment and meta",
        "skill_slugs": [
            "calibrating-confidence",
            "red-teaming-your-own-work",
            "estimating-effort",
            "deciding-reversibility",
            "steelmanning-alternatives",
            "managing-scope-drift",
            "knowing-when-to-stop",
            "karen-and-the-manager",
        ],
    },
    {
        "slug": "reading-code",
        "title": "Reading code",
        "skill_slugs": [
            "orienting-in-unfamiliar-code",
            "tracing-data-flow",
            "code-archaeology",
            "mapping-dependencies",
            "finding-the-seam",
            "reading-specifications",
            "naming-things",
        ],
    },
    {
        "slug": "landing-changes",
        "title": "Landing changes",
        "skill_slugs": [
            "atomic-commits",
            "writing-commit-messages",
            "incremental-migration",
            "refactoring-safely",
            "deleting-code",
            "feature-flagging",
            "resolving-merge-conflicts",
        ],
    },
    {
        "slug": "testing",
        "title": "Testing",
        "skill_slugs": [
            "designing-test-data",
            "property-based-testing",
            "testing-the-untestable",
            "flaky-test-triage",
            "regression-test-from-bug",
            "characterization-testing",
            "choosing-test-scope",
        ],
    },
    {
        "slug": "debugging-and-performance",
        "title": "Debugging and performance",
        "skill_slugs": [
            "reproducing-bugs",
            "bisecting-failures",
            "reading-stack-traces",
            "debugging-concurrency",
            "observing-production-safely",
            "performance-profiling",
            "finding-resource-leaks",
        ],
    },
    {
        "slug": "designing-systems",
        "title": "Designing systems",
        "skill_slugs": [
            "designing-apis",
            "modeling-errors",
            "designing-for-idempotency",
            "choosing-concurrency-model",
            "modeling-state-machines",
            "drawing-boundaries",
            "caching-strategy",
            "schema-evolution",
            "data-migrations",
            "configuration-management",
            "instrumenting-for-observability",
            "rate-limiting-and-backpressure",
        ],
    },
    {
        "slug": "security",
        "title": "Security (defensive)",
        "skill_slugs": [
            "threat-modeling",
            "handling-untrusted-input",
            "secrets-hygiene",
            "auditing-dependencies",
            "least-privilege-design",
        ],
    },
    {
        "slug": "communicating",
        "title": "Communicating",
        "skill_slugs": [
            "writing-durable-docs",
            "writing-adrs",
            "writing-release-notes",
            "writing-postmortems",
            "reviewing-code-deeply",
            "explaining-technical-work",
        ],
    },
    {
        "slug": "environment-and-tooling",
        "title": "Environment and tooling",
        "skill_slugs": [
            "portable-shell-scripting",
            "reproducible-environments",
            "designing-ci-pipelines",
            "upgrading-dependencies",
            "keeping-tbaguette-current",
            "automating-repetition",
        ],
    },
]

FORMIDABLE_SLUG = "formidable"
SKILL_FILE_NAME = "SKILL.md"
SUMMARY_MAX_LENGTH = 140

# Small, evidence-based correction table for turning a filename stem into a
# display title (see humanize_filename): built from how these exact acronyms
# are actually capitalized in the stack files' own headings ("Stack: CLI
# output", "Stack: game HUD and in-engine UI", "Stack: terminal / TUI",
# "Stack: XR and spatial"); PDF follows the same convention by analogy.
ACRONYM_WORDS = {"cli": "CLI", "hud": "HUD", "tui": "TUI", "xr": "XR", "pdf": "PDF"}


def _load_json_map(path: Path) -> dict[str, str]:
    """Load a flat {key: translated string} JSON file, or {} if it doesn't
    exist -- a locale is allowed to not yet have this file (phase-1
    sequencing: chrome/descriptions land before every skill body)."""
    if not path.is_file():
        return {}
    return json.loads(path.read_text(encoding="utf-8"))


# ---------------------------------------------------------------------------
# Entry point
# ---------------------------------------------------------------------------


def build_content(
    skills_root: str, *, locale: str | None = None, locale_root: str | None = None
) -> dict:
    """Parse every skill under skills_root and assemble the content.json dict.

    With locale=None (the default), behaves exactly as before -- this is
    the contract every existing caller and test relies on. Passing locale
    additionally requires locale_root (that language's own leaf directory,
    e.g. "i18n/fr", the same way skills_root is already the leaf "skills/"
    directory): every skill file is then looked up under locale_root first,
    falling back to skills_root file-by-file when the translated file
    doesn't exist yet -- see build_plain_skill_entry / build_formidable_skill_entry.

    Raises ValueError if a skill directory isn't covered by CATEGORIES, or if
    a category lists a skill_slug that doesn't exist on disk -- both would
    mean the locked category grouping in this module has drifted from the
    actual skills library, which should stop the build rather than silently
    ship a partial site. Also raises ValueError if locale is given without
    locale_root, or vice versa -- both must be provided together.
    """
    if (locale is None) != (locale_root is None):
        raise ValueError("locale and locale_root must both be given, or neither")

    root = Path(skills_root)
    locale_path = Path(locale_root) if locale_root else None
    translated_descriptions = _load_json_map(locale_path / "descriptions.json") if locale_path else {}
    translated_categories = _load_json_map(locale_path / "categories.json") if locale_path else {}

    category_by_skill_slug = {
        skill_slug: category
        for category in CATEGORIES
        for skill_slug in category["skill_slugs"]
    }

    discovered_slugs = list_skill_slugs(root)

    skills: dict[str, dict] = {}
    for slug in discovered_slugs:
        category = category_by_skill_slug.get(slug)
        if category is None:
            raise ValueError(
                f'skill directory "{slug}" was found under {root} but is not '
                f"listed in any CATEGORIES entry in content_pipeline.py"
            )
        skill_dir = root / slug
        locale_skill_dir = (locale_path / "skills" / slug) if locale_path else None
        fallback_description = translated_descriptions.get(slug)
        if slug == FORMIDABLE_SLUG:
            skills[slug] = build_formidable_skill_entry(
                skill_dir, category,
                locale_skill_dir=locale_skill_dir, fallback_description=fallback_description,
            )
        else:
            skills[slug] = build_plain_skill_entry(
                skill_dir, category,
                locale_skill_dir=locale_skill_dir, fallback_description=fallback_description,
            )

    for category in CATEGORIES:
        for slug in category["skill_slugs"]:
            if slug not in skills:
                raise ValueError(
                    f'category "{category["slug"]}" lists skill_slug "{slug}" '
                    f"but no such directory (with a {SKILL_FILE_NAME}) exists "
                    f"under {root}"
                )

    categories_output = [
        {
            "slug": category["slug"],
            "title": translated_categories.get(category["slug"], category["title"]),
            "skill_slugs": list(category["skill_slugs"]),
        }
        for category in CATEGORIES
    ]

    return {"categories": categories_output, "skills": skills}


def list_skill_slugs(skills_root: Path) -> list[str]:
    """Sorted slugs of every subdirectory of skills_root that has a SKILL.md."""
    return sorted(
        entry.name
        for entry in skills_root.iterdir()
        if entry.is_dir() and (entry / SKILL_FILE_NAME).is_file()
    )


# ---------------------------------------------------------------------------
# Per-skill entry builders
# ---------------------------------------------------------------------------


def build_plain_skill_entry(
    skill_dir: Path, category: dict, *,
    locale_skill_dir: Path | None = None, fallback_description: str | None = None,
) -> dict:
    """Build the schema entry for one of the 63 ordinary (non-formidable) skills.

    locale_skill_dir, when given, is checked for a translated SKILL.md
    first; missing it falls back to skill_dir (English) and marks the
    entry translated=False. locale_skill_dir=None (the default, no-locale
    build) always uses skill_dir and is translated=True -- English is
    trivially "in its own language," which is what lets templates.py
    (Task 6) key the fallback banner off this one flag regardless of
    whether a locale build is happening at all.
    """
    if locale_skill_dir is None:
        translated = True
        source_dir = skill_dir
    else:
        translated = (locale_skill_dir / SKILL_FILE_NAME).is_file()
        source_dir = locale_skill_dir if translated else skill_dir

    frontmatter, body = split_frontmatter(read_text(source_dir / SKILL_FILE_NAME))
    description = frontmatter["description"] if translated else (fallback_description or frontmatter["description"])
    body_html = render_markdown_body(strip_title_heading(body))
    return {
        "slug": skill_dir.name,
        "name": frontmatter["name"],
        "category_slug": category["slug"],
        "category_title": category["title"],
        "description": description,
        "summary": summarize_description(description),
        "body_html": body_html,
        "is_formidable": False,
        "translated": translated,
    }


def build_formidable_skill_entry(
    skill_dir: Path, category: dict, *,
    locale_skill_dir: Path | None = None, fallback_description: str | None = None,
) -> dict:
    """Build the schema entry for formidable, including its inlined sub-pages.

    Per-file fallback extends to every reference/*.md and
    reference/stacks/*.md file independently, matched by filename (a
    translated file keeps the exact same name as its English source, per
    the repo layout convention) -- so one stack file can be translated
    while another still shows English. The entry's own translated flag is
    True only when every file (the main SKILL.md plus every reference and
    stacks file) came from locale_skill_dir; any single missing file makes
    the whole page read as untranslated for the fallback banner (Task 6) --
    a coarser signal than per-file, deliberately, since the banner is a
    single page-level notice, not a per-tab one.
    """
    if locale_skill_dir is None:
        main_translated = True
        main_source_dir = skill_dir
    else:
        main_translated = (locale_skill_dir / SKILL_FILE_NAME).is_file()
        main_source_dir = locale_skill_dir if main_translated else skill_dir

    frontmatter, body = split_frontmatter(read_text(main_source_dir / SKILL_FILE_NAME))
    description = frontmatter["description"] if main_translated else (fallback_description or frontmatter["description"])

    reference_dir = skill_dir / "reference"
    stacks_dir = reference_dir / "stacks"
    stack_paths = sorted(stacks_dir.glob("*.md"))
    reference_paths = sorted(p for p in reference_dir.glob("*.md") if p.is_file())

    locale_reference_dir = (locale_skill_dir / "reference") if locale_skill_dir else None
    locale_stacks_dir = (locale_reference_dir / "stacks") if locale_reference_dir else None

    def resolve(english_path: Path, locale_dir: Path | None) -> tuple[Path, bool]:
        if locale_dir is None:
            return english_path, True
        candidate = locale_dir / english_path.name
        return (candidate, True) if candidate.is_file() else (english_path, False)

    all_translated = main_translated
    resolved_stack_paths: list[Path] = []
    for path in stack_paths:
        resolved, ok = resolve(path, locale_stacks_dir)
        resolved_stack_paths.append(resolved)
        all_translated = all_translated and ok
    resolved_reference_paths: list[Path] = []
    for path in reference_paths:
        resolved, ok = resolve(path, locale_reference_dir)
        resolved_reference_paths.append(resolved)
        all_translated = all_translated and ok

    anchor_id_by_filename_stem = {path.stem: f"stack-{path.stem}" for path in stack_paths}
    anchor_id_by_filename_stem.update({path.stem: f"cmd-{path.stem}" for path in reference_paths})
    resolve_relative_link = make_formidable_link_resolver(anchor_id_by_filename_stem)

    body_html = render_markdown_body(
        strip_title_heading(body), resolve_relative_link=resolve_relative_link
    )

    formidable_stacks = [
        render_formidable_subdocument(path, "stack", resolve_relative_link)
        for path in resolved_stack_paths
    ]
    formidable_commands = [
        render_formidable_subdocument(path, "cmd", resolve_relative_link)
        for path in resolved_reference_paths
        if path.stem != "craft-floor"
    ]

    entry = {
        "slug": skill_dir.name,
        "name": frontmatter["name"],
        "category_slug": category["slug"],
        "category_title": category["title"],
        "description": description,
        "summary": summarize_description(description),
        "body_html": body_html,
        "is_formidable": True,
        "formidable_stacks": formidable_stacks,
        "formidable_commands": formidable_commands,
        "translated": all_translated,
    }

    craft_floor_path = reference_dir / "craft-floor.md"
    if craft_floor_path.is_file():
        resolved_craft_floor, _ = resolve(craft_floor_path, locale_reference_dir)
        craft_floor_doc = render_formidable_subdocument(
            resolved_craft_floor, "cmd", resolve_relative_link
        )
        entry["formidable_craft_floor_html"] = craft_floor_doc["html"]

    return entry


def render_formidable_subdocument(
    path: Path, anchor_prefix: str, resolve_relative_link: Callable[[str], str | None]
) -> dict:
    """Render one formidable reference/*.md or reference/stacks/*.md file.

    Returns {"id", "title", "html"} per the schema. id is
    "<anchor_prefix>-<filename stem>" (e.g. "stack-web", "cmd-shape") --
    kebab-case and stable since it's derived straight from the filename.
    title is a humanized version of the filename stem rather than the file's
    own H1 text: the H1s are taglines ("Stack: dense data -- tables,
    dashboards, monitors") that sometimes reorder words relative to the
    filename (data-dense.md) or use a different verb than the command table
    in formidable/SKILL.md (porting.md's H1 leads with "port"), so deriving
    from the filename keeps title and id/anchor consistent everywhere.
    Heading ids inside html are prefixed with this section's own id, because
    all these fragments land on one page (formidable's) and several files
    share heading text verbatim ("Audit hooks", "Rules") -- unprefixed ids
    would collide and anchors would silently point at the wrong section.
    """
    section_id = f"{anchor_prefix}-{path.stem}"
    body = strip_title_heading(read_text(path))
    html = render_markdown_body(
        body,
        heading_id_prefix=f"{section_id}-",
        resolve_relative_link=resolve_relative_link,
    )
    return {"id": section_id, "title": humanize_filename(path.stem), "html": html}


def make_formidable_link_resolver(
    anchor_id_by_filename_stem: dict[str, str],
) -> Callable[[str], str | None]:
    """Build a resolver for formidable's internal relative `.md` links.

    Every such link in the corpus (in formidable/SKILL.md and in
    reference/color.md) points at a reference or stacks file using a
    relative path -- "reference/craft-floor.md", "reference/stacks/web.md",
    "stacks/web.md", "tokens.md", depending which file it's written from.
    Resolving by filename stem alone sidesteps needing to know each
    referencing file's own directory, and is unambiguous here since no
    reference/*.md and reference/stacks/*.md filename collide.
    """

    def resolve(href: str) -> str | None:
        stem = PurePosixPath(href).stem
        anchor_id = anchor_id_by_filename_stem.get(stem)
        return f"#{anchor_id}" if anchor_id else None

    return resolve


# ---------------------------------------------------------------------------
# Frontmatter + title handling
# ---------------------------------------------------------------------------

_FRONTMATTER_FIELD_RE = re.compile(r"^([A-Za-z][A-Za-z0-9_-]*):[ \t]?(.*)$")


def split_frontmatter(markdown_text: str) -> tuple[dict[str, str], str]:
    """Split a SKILL.md's YAML frontmatter from its markdown body.

    Every SKILL.md in this corpus uses flat, single-line, unquoted
    `key: value` frontmatter (verified against all 66 files), so this reads
    just enough YAML to get name/description out without a YAML dependency
    -- it is not a general YAML parser.
    """
    lines = markdown_text.split("\n")
    if not lines or lines[0].strip() != "---":
        raise ValueError('expected a "---" frontmatter delimiter on the first line')
    closing_index = next(
        (i for i in range(1, len(lines)) if lines[i].strip() == "---"), None
    )
    if closing_index is None:
        raise ValueError('frontmatter opened with "---" is never closed')

    fields: dict[str, str] = {}
    for line in lines[1:closing_index]:
        match = _FRONTMATTER_FIELD_RE.match(line)
        if match:
            key, value = match.group(1), match.group(2).strip()
            fields[key] = _strip_matching_quotes(value)

    body = "\n".join(lines[closing_index + 1 :])
    return fields, body


def _strip_matching_quotes(value: str) -> str:
    if len(value) >= 2 and value[0] == value[-1] and value[0] in ("'", '"'):
        return value[1:-1]
    return value


def strip_title_heading(body: str) -> str:
    """Remove the body's leading `# Title` line (the page template supplies its own)."""
    lines = body.split("\n")
    index = 0
    while index < len(lines) and lines[index].strip() == "":
        index += 1
    if index < len(lines) and lines[index].lstrip().startswith("# "):
        return "\n".join(lines[index + 1 :])
    return body


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


# ---------------------------------------------------------------------------
# Summary (card teaser) trimming
# ---------------------------------------------------------------------------

_SENTENCE_START_RE = re.compile(r"^.*?[.!?](?=\s|$)")


def summarize_description(description: str, max_length: int = SUMMARY_MAX_LENGTH) -> str:
    """Trim a frontmatter description to a clean, <=140-char teaser.

    Every description in this corpus is a single long "Use when A, when B,
    ... or when Z. Covers ..." sentence (all 66 run well past 140 characters,
    the shortest is 339), so this always has real trimming to do. Strategy:
    take the first sentence; if it still doesn't fit, cut at the last comma
    (clause boundary) within budget, or fall back to the last word boundary
    -- either way stopping only at a real boundary, never mid-word -- and
    mark the cut with an ellipsis.
    """
    normalized = " ".join(description.split())
    sentence_match = _SENTENCE_START_RE.match(normalized)
    first_sentence = sentence_match.group(0) if sentence_match else normalized

    if len(first_sentence) <= max_length:
        return first_sentence

    budget = max_length - 1  # reserve one character for the ellipsis
    truncated = first_sentence[:budget]
    comma_cut = truncated.rfind(",")
    space_cut = truncated.rfind(" ")

    if comma_cut > budget * 0.3:
        clipped = first_sentence[:comma_cut]
    elif space_cut > 0:
        clipped = first_sentence[:space_cut]
    else:
        clipped = truncated

    return clipped.rstrip(" ,;:") + "…"


# ---------------------------------------------------------------------------
# Slugs and titles
# ---------------------------------------------------------------------------


def slugify(text: str) -> str:
    """Kebab-case a piece of plain text for use as an HTML id fragment."""
    normalized = text.lower().replace("'", "").replace("’", "")
    slug = re.sub(r"[^a-z0-9]+", "-", normalized).strip("-")
    return slug or "section"


def humanize_filename(filename_stem: str) -> str:
    """Turn a kebab-case filename stem into a clean display title.

    "web" -> "Web", "cli-output" -> "CLI Output". See ACRONYM_WORDS for the
    evidence behind which words get uppercased instead of merely capitalized.
    """
    words = filename_stem.split("-")
    return " ".join(ACRONYM_WORDS.get(word, word.capitalize()) for word in words)


# ---------------------------------------------------------------------------
# HTML escaping -- applied to every extracted text run before it is wrapped
# in a tag, on the handling-untrusted-input principle that a generator
# should not trust its input implicitly even when the input is first-party.
# ---------------------------------------------------------------------------


def escape_html(text: str) -> str:
    return (
        text.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
    )


# ---------------------------------------------------------------------------
# Inline markdown: bold, italic, code, links
# ---------------------------------------------------------------------------

# Ordered so `**bold**` is matched before `*italic*` can claim one of its
# asterisks, and inline `code` before either (code spans are never
# re-parsed for markup inside them). Bold/italic content is disallowed from
# containing a literal `*` -- true of every real instance in this corpus and
# what keeps adjacent spans (`**a** and **b**`) from merging into one match.
_INLINE_TOKEN_RE = re.compile(
    r"`(?P<code>[^`]+)`"
    r"|\*\*(?P<bold>[^*]+)\*\*"
    r"|\*(?P<italic>[^*]+)\*"
    r"|\[(?P<link_text>[^\]]*)\]\((?P<link_href>[^)]*)\)"
)


def render_inline_markdown(
    text: str, resolve_relative_link: Callable[[str], str | None] | None = None
) -> str:
    """Render bold/italic/code/link markdown within a single run of text.

    Plain text runs between matches are HTML-escaped; recognized markup is
    wrapped in the corresponding tag. Bold and italic content is rendered
    recursively (so `` **`code`** `` still gets a real <code> span); code
    span content never is, since code is verbatim by definition.
    """
    pieces: list[str] = []
    cursor = 0
    for match in _INLINE_TOKEN_RE.finditer(text):
        if match.start() > cursor:
            pieces.append(escape_html(text[cursor : match.start()]))

        if match.group("code") is not None:
            pieces.append(f"<code>{escape_html(match.group('code'))}</code>")
        elif match.group("bold") is not None:
            inner = render_inline_markdown(match.group("bold"), resolve_relative_link)
            pieces.append(f"<strong>{inner}</strong>")
        elif match.group("italic") is not None:
            inner = render_inline_markdown(match.group("italic"), resolve_relative_link)
            pieces.append(f"<em>{inner}</em>")
        else:
            pieces.append(
                _render_markdown_link(
                    match.group("link_text"), match.group("link_href"), resolve_relative_link
                )
            )
        cursor = match.end()

    if cursor < len(text):
        pieces.append(escape_html(text[cursor:]))
    return "".join(pieces)


def _render_markdown_link(
    link_text: str,
    href: str,
    resolve_relative_link: Callable[[str], str | None] | None,
) -> str:
    rendered_text = render_inline_markdown(link_text, resolve_relative_link)
    if href.startswith(("http://", "https://")):
        return f'<a href="{escape_html(href)}">{rendered_text}</a>'

    resolved_href = resolve_relative_link(href) if resolve_relative_link else None
    if resolved_href is None:
        # Unresolvable relative link: keep the visible text, drop the href
        # rather than emit a broken link.
        return rendered_text
    return f'<a href="{escape_html(resolved_href)}">{rendered_text}</a>'


def _plain_text_for_slug(raw_heading_text: str) -> str:
    """Strip inline markup characters so a heading's id reflects its words, not its syntax."""
    return raw_heading_text.replace("`", "").replace("*", "")


# ---------------------------------------------------------------------------
# Block-level markdown: headings, fenced code, tables, lists, paragraphs
# ---------------------------------------------------------------------------

_HEADING_RE = re.compile(r"^(#{2,3})\s+(.*)$")
_FENCE_RE = re.compile(r"^```")
_LIST_ITEM_RE = re.compile(r"^(\s*)(-|\d+\.)\s+(.*)$")
_TABLE_ROW_RE = re.compile(r"^\s*\|.*\|\s*$")
_TABLE_SEPARATOR_RE = re.compile(r"^\s*\|[\s:|-]+\|\s*$")
_UNESCAPED_PIPE_RE = re.compile(r"(?<!\\)\|")


def render_markdown_body(
    markdown_text: str,
    *,
    heading_id_prefix: str = "",
    resolve_relative_link: Callable[[str], str | None] | None = None,
) -> str:
    """Render a markdown body (without its leading `# Title`) to HTML.

    Covers the subset actually used across these files: ## / ### headings
    (each given a kebab-case id, deduplicated within this one call),
    **bold**, `code`, GFM pipe tables, `-` and `1.` lists (with one level of
    nesting, and unmarked indented continuation lines folded into the
    preceding item), `[text](url)` links, and paragraphs. Also handles two
    constructs seen in this corpus but outside that stated subset -- fenced
    code blocks and single-asterisk italics -- converted as sensibly as
    possible rather than left to crash the parser (see the module's final
    report for detail on where these show up).

    heading_id_prefix is prepended to every heading id produced by this
    call, so that when several of these bodies are rendered onto the same
    page (formidable's stacks and commands, all inlined together) their
    internal anchors can't collide even when their heading text is
    identical (several stack files share an "Audit hooks" heading verbatim).
    """
    lines = markdown_text.split("\n")
    used_heading_ids: set[str] = set()
    blocks: list[str] = []
    index = 0
    total = len(lines)

    while index < total:
        line = lines[index]
        if line.strip() == "":
            index += 1
            continue

        if _HEADING_RE.match(line):
            html, index = _consume_heading(
                lines, index, heading_id_prefix, used_heading_ids, resolve_relative_link
            )
        elif _FENCE_RE.match(line.strip()):
            html, index = _consume_fenced_code_block(lines, index)
        elif _is_table_start(lines, index):
            html, index = _consume_table(lines, index, resolve_relative_link)
        elif _LIST_ITEM_RE.match(line):
            html, index = _consume_list(lines, index, resolve_relative_link)
        else:
            html, index = _consume_paragraph(lines, index, resolve_relative_link)
        blocks.append(html)

    return "\n".join(blocks)


def _is_table_start(lines: list[str], index: int) -> bool:
    return (
        _TABLE_ROW_RE.match(lines[index]) is not None
        and index + 1 < len(lines)
        and _TABLE_SEPARATOR_RE.match(lines[index + 1]) is not None
    )


def _is_block_start(lines: list[str], index: int) -> bool:
    """Whether lines[index] begins a new heading/fence/table/list block.

    Used so a paragraph or list-continuation scan stops at the right line
    even when the source has no blank line separating it from what follows
    (both patterns occur for real in this corpus: a lead-in sentence
    directly followed by a list, and mid-item wrapped continuation text).
    """
    line = lines[index]
    return bool(
        _HEADING_RE.match(line)
        or _FENCE_RE.match(line.strip())
        or _is_table_start(lines, index)
        or _LIST_ITEM_RE.match(line)
    )


def _consume_heading(
    lines: list[str],
    index: int,
    heading_id_prefix: str,
    used_heading_ids: set[str],
    resolve_relative_link: Callable[[str], str | None] | None,
) -> tuple[str, int]:
    match = _HEADING_RE.match(lines[index])
    level = len(match.group(1))
    raw_text = match.group(2).strip()

    slug = slugify(_plain_text_for_slug(raw_text))
    heading_id = _dedupe_id(f"{heading_id_prefix}{slug}", used_heading_ids)
    inner_html = render_inline_markdown(raw_text, resolve_relative_link)

    tag = f"h{level}"
    return f'<{tag} id="{heading_id}">{inner_html}</{tag}>', index + 1


def _dedupe_id(candidate: str, used_ids: set[str]) -> str:
    unique_id = candidate
    suffix = 2
    while unique_id in used_ids:
        unique_id = f"{candidate}-{suffix}"
        suffix += 1
    used_ids.add(unique_id)
    return unique_id


def _consume_fenced_code_block(lines: list[str], index: int) -> tuple[str, int]:
    """Consume a ``` ... ``` block. Not in the spec's stated subset, but real

    (5 of the 66 files use one, e.g. bisecting-failures.md's bisect script);
    rendered as a literal, HTML-escaped <pre><code> block with no inline
    markdown processing inside it, so a code sample's own `*` or backticks
    can't be misread as formatting -- this matters concretely, since one
    fenced block (incremental-migration.md) contains "1. 2. 3." -style
    lines that must NOT become a rendered <ol>.
    """
    total = len(lines)
    closing_index = index + 1
    while closing_index < total and not _FENCE_RE.match(lines[closing_index].strip()):
        closing_index += 1
    code_text = "\n".join(lines[index + 1 : closing_index])
    next_index = closing_index + 1 if closing_index < total else closing_index
    return (
        f'<pre class="prose-code-block"><code>{escape_html(code_text)}</code></pre>',
        next_index,
    )


def _consume_table(
    lines: list[str], index: int, resolve_relative_link: Callable[[str], str | None] | None
) -> tuple[str, int]:
    header_cells = _split_table_row(lines[index])
    body_index = index + 2  # skip the header row and the |---|---| separator
    total = len(lines)
    body_rows: list[list[str]] = []
    while body_index < total and _TABLE_ROW_RE.match(lines[body_index]):
        body_rows.append(_split_table_row(lines[body_index]))
        body_index += 1

    header_html = "".join(
        f"<th>{render_inline_markdown(cell, resolve_relative_link)}</th>" for cell in header_cells
    )
    body_html = "".join(
        "<tr>"
        + "".join(f"<td>{render_inline_markdown(cell, resolve_relative_link)}</td>" for cell in row)
        + "</tr>"
        for row in body_rows
    )
    table_html = f"<table><thead><tr>{header_html}</tr></thead><tbody>{body_html}</tbody></table>"
    return table_html, body_index


def _split_table_row(line: str) -> list[str]:
    """Split a `| a | b |` row into cells, honoring `\\|` as a literal pipe.

    Two real rows in this corpus put a shell pipeline inside a table cell's
    inline code (e.g. `` `git log ... \\| sort \\| uniq -c` ``) and rely on
    the backslash-escape to keep it from being read as extra columns; a
    naive split('|') breaks those rows into the wrong number of cells.
    """
    trimmed = line.strip().removeprefix("|").removesuffix("|")
    cells = _UNESCAPED_PIPE_RE.split(trimmed)
    return [cell.strip().replace("\\|", "|") for cell in cells]


def _consume_list(
    lines: list[str], index: int, resolve_relative_link: Callable[[str], str | None] | None
) -> tuple[str, int]:
    """Consume a run of `-`/`1.` list-item lines (with up to one level of nesting).

    An indented line that does NOT itself start a new item is folded into
    the text of the preceding item rather than treated as a parse error --
    real for one item in formidable/reference/calm.md, whose item 4 wraps
    onto a continuation line with no marker of its own.
    """
    total = len(lines)
    entries: list[list] = []  # each entry: [indent, marker, text]
    cursor = index
    while cursor < total:
        line = lines[cursor]
        if line.strip() == "":
            break
        if _HEADING_RE.match(line) or _FENCE_RE.match(line.strip()) or _is_table_start(lines, cursor):
            break

        item_match = _LIST_ITEM_RE.match(line)
        if item_match:
            indent = len(item_match.group(1))
            marker = item_match.group(2)
            text = item_match.group(3).strip()
            entries.append([indent, marker, text])
        elif entries:
            entries[-1][2] += " " + line.strip()
        else:
            break
        cursor += 1

    html, _ = _render_list_entries(entries, 0, resolve_relative_link)
    return html, cursor


def _render_list_entries(
    entries: list[list],
    start: int,
    resolve_relative_link: Callable[[str], str | None] | None,
) -> tuple[str, int]:
    """Recursively render entries[start:] at entries[start]'s indent level.

    Returns (html, next_index) where next_index is the first entry at a
    shallower indent than this group (or len(entries) if none remains) --
    the same shape a recursive-descent parser returns so the caller can
    keep walking sibling items after a nested group closes.
    """
    base_indent = entries[start][0]
    is_ordered = entries[start][1][0].isdigit()
    tag = "ol" if is_ordered else "ul"

    parts = [f"<{tag}>"]
    cursor = start
    while cursor < len(entries) and entries[cursor][0] == base_indent:
        item_html = render_inline_markdown(entries[cursor][2], resolve_relative_link)
        cursor += 1
        if cursor < len(entries) and entries[cursor][0] > base_indent:
            nested_html, cursor = _render_list_entries(entries, cursor, resolve_relative_link)
            item_html += nested_html
        parts.append(f"<li>{item_html}</li>")
    parts.append(f"</{tag}>")
    return "".join(parts), cursor


def _consume_paragraph(
    lines: list[str], index: int, resolve_relative_link: Callable[[str], str | None] | None
) -> tuple[str, int]:
    total = len(lines)
    paragraph_lines = [lines[index].strip()]
    cursor = index + 1
    while cursor < total and lines[cursor].strip() != "" and not _is_block_start(lines, cursor):
        paragraph_lines.append(lines[cursor].strip())
        cursor += 1

    text = " ".join(paragraph_lines)
    return f"<p>{render_inline_markdown(text, resolve_relative_link)}</p>", cursor


# ---------------------------------------------------------------------------
# Script entry point
# ---------------------------------------------------------------------------


def _default_skills_root() -> Path:
    # The repo embeds its own copy of the skill set at <repo_root>/skills/, so
    # a clone builds standalone with no dependency on any particular machine's
    # ~/.claude install. scripts/ is a direct child of the repo root.
    return Path(__file__).resolve().parent.parent / "skills"


def _output_path() -> Path:
    return Path(__file__).resolve().parent / "content.json"


def main() -> None:
    skills_root = _default_skills_root()
    content = build_content(str(skills_root))
    output_path = _output_path()
    output_path.write_text(
        json.dumps(content, indent=2, ensure_ascii=False) + "\n", encoding="utf-8"
    )
    print(f"Wrote {output_path} ({len(content['skills'])} skills)")


if __name__ == "__main__":
    main()
