"""The skill graph: which skill cites which, and from which section.

Built from the *rendered* content rather than from the markdown, on purpose.
content_pipeline already decides what counts as a cross-reference -- a code
span or a bare slug-shaped token naming a skill that exists, never the page's
own slug -- and turns exactly those into ``<a class="skill-link">``. Reading
the links back out of body_html means the graph and the pages can never
disagree about what cites what: an edge exists here if and only if a reader
can click it on the site.

The output is the JSON the /graph/ page fetches (see docs/assets/graph.js),
plus, from banner_summary, the handful of numbers the landing page's
announcement banner carries inline. The JSON's shape, per skill:

    {"slug", "name", "category", "summary", "words", "always_on",
     "change_status", "change_at",
     "trigger":  {"refs": {slug: n}, "quotes": {slug: "..."}},
     "sections": [{"id", "title", "kind", "words", "refs", "quotes",
                   "subs": [{"id", "title", "words", "refs"}]}]}

"trigger" is the frontmatter description -- the one part of a skill an agent
always has loaded, so a citation there is a stronger claim than one in the
body. "kind" is "body" for the SKILL.md's own sections and "reference" for a
reference file rendered on the same page. A section's refs include its
sub-sections' refs; a sub-section keeps its own so the graph can drill in.

Edges are not stored. They are the sum of the refs above, and deriving them
in the browser keeps one source of truth instead of two that must agree.

Stdlib only, like the rest of the generator.
"""

from __future__ import annotations

import re
from html.parser import HTMLParser

# The pipeline renders every cross-reference href through templates.skill_url,
# which always ends ".../skills/<slug>/". Matching the tail rather than the
# whole URL keeps this independent of the base path and the locale prefix.
_SKILL_HREF_RE = re.compile(r"/skills/([a-z0-9]+(?:-[a-z0-9]+)*)/$")

# Elements whose text is one unit of prose for quoting purposes. A citation's
# quote is the sentence around it, and a sentence never crosses one of these.
_BLOCK_TAGS = frozenset({"p", "li", "td", "th", "h2", "h3", "h4", "pre", "blockquote", "dt", "dd"})

# Long enough for the sentence that carries the reason, short enough that a
# tooltip holding one never needs to scroll.
QUOTE_MAX_LENGTH = 240

# A clause this short before a colon is a label that the quote keeps.
LABEL_MAX_WORDS = 3

# The trigger descriptions that make a skill run in every conversation rather
# than when a situation calls for it. Matched on the description's opening,
# which is where every such skill states it.
_ALWAYS_ON_RE = re.compile(r"^Use at the start of every (?:single )?conversation", re.IGNORECASE)

_WORD_RE = re.compile(r"[^\W_]+(?:['’-][^\W_]+)*")
_LABEL_RE = re.compile(r"((?:[^\W_]+(?:['’-][^\W_]+)*)(?:\s+[^\W_]+(?:['’-][^\W_]+)*){0,%d}):\s" % (LABEL_MAX_WORDS - 1))


def skill_slug_from_href(href: str) -> str | None:
    """The slug a skill-link points at, or None for any other href."""
    match = _SKILL_HREF_RE.search(href or "")
    return match.group(1) if match else None


def _count_words(text: str) -> int:
    return len(_WORD_RE.findall(text))


def _quote_around(text: str, offset: int, length: int, max_length: int = QUOTE_MAX_LENGTH) -> str:
    """The sentence of ``text`` containing the span at ``offset``, trimmed to
    ``max_length`` around that span with an ellipsis on whichever side lost
    words. Sentence ends are ". ", "? ", "! " and "; " -- a semicolon joins
    two claims often enough in this corpus that stopping there keeps the
    quote about the one that holds the citation. A colon cuts the quote only
    after a clause longer than ``LABEL_MAX_WORDS``; a short label stays."""
    start = 0
    for match in re.finditer(r"[.?!;:](?=\s)", text[:offset]):
        # A colon after a short label ("Not for:", "Use when:") is part of the
        # claim, not its start: cutting there turns a redirect away from a
        # skill into what reads as an endorsement of it.
        if match.group() == ":" and _count_words(text[start:match.start()]) <= LABEL_MAX_WORDS:
            continue
        start = match.end()
    end_match = re.search(r"[.?!;](?=\s|$)", text[offset + length:])
    end = offset + length + end_match.end() if end_match else len(text)
    raw = text[start:end]
    mention_at = offset - start - (len(raw) - len(raw.lstrip()))
    # A clause cut at a semicolon or colon ends on that mark, which reads as
    # a sentence left hanging; the quote stands better without it.
    sentence = raw.strip().rstrip(";:,")
    if len(sentence) <= max_length:
        return sentence
    # A kept label ("Not for:") is spent from the budget up front so a trim
    # that drops it can put it back, rather than opening on "…for:".
    label_match = _LABEL_RE.match(sentence)
    label = label_match.group(1) + ":" if label_match and label_match.end() < offset - start else ""
    if label:
        max_length -= len(label) + 2
    # Centre the window on the mention, then snap both edges to word breaks.
    half = (max_length - length) // 2
    lo = max(0, mention_at - half)
    hi = min(len(sentence), lo + max_length)
    lo = max(0, hi - max_length)
    if lo > 0:
        space = sentence.find(" ", lo)
        lo = space + 1 if 0 <= space < mention_at else lo
    if hi < len(sentence):
        space = sentence.rfind(" ", mention_at + length, hi)
        hi = space if space > 0 else hi
    lead = ""
    if lo > 0:
        lead = label + " …" if label and lo > len(label) else "…"
    return lead + sentence[lo:hi].strip() + ("…" if hi < len(sentence) else "")


class _Block:
    __slots__ = ("parts", "mentions", "section", "sub")

    def __init__(self, section: dict, sub: dict | None) -> None:
        self.parts: list[str] = []
        self.mentions: list[tuple[int, int, str]] = []
        self.section = section
        self.sub = sub

    @property
    def length(self) -> int:
        return sum(len(p) for p in self.parts)


class _SectionWalker(HTMLParser):
    """Splits one rendered document into h2 sections and h3 sub-sections,
    counting words and recording each skill-link inside them."""

    def __init__(self, *, opening_title: str, kind: str, opening_id: str = "") -> None:
        super().__init__(convert_charrefs=True)
        self.kind = kind
        self.sections: list[dict] = []
        self._section = self._new_section(opening_id, opening_title)
        self._sub: dict | None = None
        self._heading_level: int | None = None
        self._heading_target: dict | None = None
        self._heading_parts: list[str] = []
        self._blocks: list[_Block] = []
        self._link_slug: str | None = None
        self._link_start: int = 0

    def _new_section(self, section_id: str, title: str) -> dict:
        section = {"id": section_id, "title": title, "kind": self.kind, "words": 0,
                   "refs": {}, "quotes": {}, "subs": []}
        self.sections.append(section)
        return section

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        attributes = dict(attrs)
        if tag in ("h2", "h3"):
            heading_id = attributes.get("id") or ""
            if tag == "h2":
                self._section = self._new_section(heading_id, "")
                self._sub = None
                self._heading_target = self._section
            else:
                self._sub = {"id": heading_id, "title": "", "words": 0, "refs": {}}
                self._section["subs"].append(self._sub)
                self._heading_target = self._sub
            self._heading_level = int(tag[1])
            self._heading_parts = []
        if tag in _BLOCK_TAGS:
            self._blocks.append(_Block(self._section, self._sub))
        if tag == "a" and "skill-link" in (attributes.get("class") or "").split():
            slug = skill_slug_from_href(attributes.get("href") or "")
            if slug:
                self._link_slug = slug
                self._link_start = self._blocks[-1].length if self._blocks else 0
                self._record_ref(slug)

    def handle_endtag(self, tag: str) -> None:
        if tag == "a" and self._link_slug is not None:
            if self._blocks:
                block = self._blocks[-1]
                block.mentions.append((self._link_start, block.length - self._link_start, self._link_slug))
            self._link_slug = None
        if tag in ("h2", "h3") and self._heading_level == int(tag[1]):
            if self._heading_target is not None:
                self._heading_target["title"] = " ".join("".join(self._heading_parts).split())
            self._heading_level = None
            self._heading_target = None
        if tag in _BLOCK_TAGS and self._blocks:
            self._flush_block(self._blocks.pop())

    def handle_data(self, data: str) -> None:
        if self._heading_level is not None:
            self._heading_parts.append(data)
        words = _count_words(data)
        self._section["words"] += words
        if self._sub is not None:
            self._sub["words"] += words
        if self._blocks:
            self._blocks[-1].parts.append(data)

    def _record_ref(self, slug: str) -> None:
        refs = self._section["refs"]
        refs[slug] = refs.get(slug, 0) + 1
        if self._sub is not None:
            sub_refs = self._sub["refs"]
            sub_refs[slug] = sub_refs.get(slug, 0) + 1

    def _flush_block(self, block: _Block) -> None:
        if not block.mentions:
            return
        text = "".join(block.parts).replace("\n", " ")
        quotes = block.section["quotes"]
        for offset, length, slug in block.mentions:
            if slug not in quotes:
                quotes[slug] = _quote_around(text, offset, length)

    def close(self) -> None:
        super().close()
        while self._blocks:
            self._flush_block(self._blocks.pop())
        # The stretch before the first heading only counts as a section when
        # it says something; most skills open straight into their first h2.
        if self.sections and self.sections[0]["words"] == 0 and not self.sections[0]["refs"]:
            self.sections.pop(0)


def extract_sections(html: str, *, kind: str = "body", opening_title: str = "Opening",
                     opening_id: str = "") -> list[dict]:
    """Every h2 section of one rendered document, in reading order."""
    walker = _SectionWalker(opening_title=opening_title, kind=kind, opening_id=opening_id)
    walker.feed(html or "")
    walker.close()
    return walker.sections


def _reference_documents(skill: dict) -> list[tuple[str, str, str]]:
    """(anchor id, title, html) for each reference file rendered on this
    skill's page, in the order the page shows them."""
    documents = []
    for group_label, key in (("Stack", "formidable_stacks"), ("Command", "formidable_commands"),
                             ("Reference", "reference_sections")):
        for item in skill.get(key) or []:
            documents.append((item["id"], f"{group_label} · {item['title']}", item["html"]))
    craft_floor = skill.get("formidable_craft_floor_html")
    if craft_floor:
        documents.append(("cmd-craft-floor", "Craft floor", craft_floor))
    return documents


def build_skill_node(skill: dict) -> dict:
    sections = extract_sections(skill.get("body_html", ""))
    for anchor_id, title, html in _reference_documents(skill):
        # One reference file becomes one section; its own h2s become that
        # section's subs, so the graph keeps both the file and its parts.
        parts = extract_sections(html, kind="reference", opening_title=title, opening_id=anchor_id)
        section = {"id": anchor_id, "title": title, "kind": "reference", "words": 0,
                   "refs": {}, "quotes": {}, "subs": []}
        for part in parts:
            section["words"] += part["words"]
            for slug, count in part["refs"].items():
                section["refs"][slug] = section["refs"].get(slug, 0) + count
            for slug, quote in part["quotes"].items():
                section["quotes"].setdefault(slug, quote)
            if part["id"] != anchor_id:
                section["subs"].append({"id": part["id"], "title": part["title"],
                                        "words": part["words"], "refs": part["refs"]})
        sections.append(section)

    trigger = extract_sections(skill.get("description_html", ""), kind="trigger",
                               opening_title="When to use")
    description = skill.get("description", "")
    trigger_refs = trigger[0]["refs"] if trigger else {}
    trigger_quotes = trigger[0]["quotes"] if trigger else {}
    return {
        "slug": skill["slug"],
        "name": skill["name"],
        "category": skill["category_slug"],
        "summary": skill.get("summary", ""),
        "words": sum(s["words"] for s in sections),
        "always_on": bool(_ALWAYS_ON_RE.match(description)),
        "change_status": skill.get("change_status"),
        "change_at": skill.get("change_at"),
        "trigger": {"refs": trigger_refs, "quotes": trigger_quotes},
        "sections": sections,
    }


def build_graph(content: dict, *, skill_url_template: str) -> dict:
    """The whole graph document for one locale's content dict.

    skill_url_template holds a literal "{slug}" -- the browser fills it in,
    so the JSON carries one URL pattern instead of one hundred and one URLs."""
    categories = [
        {"slug": c["slug"], "title": c["title"], "skill_slugs": list(c["skill_slugs"])}
        for c in content["categories"]
    ]
    order = [slug for c in categories for slug in c["skill_slugs"]]
    skills = [build_skill_node(content["skills"][slug]) for slug in order]
    return {
        "schema": 1,
        "skill_url_template": skill_url_template,
        "categories": categories,
        "skills": skills,
    }


# ---------------------------------------------------------------------------
# The file on disk: the same graph, packed
# ---------------------------------------------------------------------------
#
# graph.json ships inside the plugin folder, and Anthropic's plugin directory
# holds any text file over 256 KiB for a reviewer. Written out as build_graph
# returns it, the graph was 344 KiB -- nearly all of it the same seven keys
# spelled out on 1,127 sections, 3,000 empty {} and [], and a skill's slug
# written out every time another skill cited it. Schema 2 is the same data
# with those taken out, and nothing else:
#
#   - a skill, a section and a subsection are arrays in a fixed field order,
#     with trailing fields that hold their default left off;
#   - a citation is [skill index, count] or [skill index, count, quote] --
#     the quote rides on its citation, since a section only ever quotes a
#     skill it also cites (pack refuses a graph where that stops holding);
#   - a heading's id is written only when it is not slugify(title), which is
#     how content_pipeline makes nearly all of them;
#   - a family is its index; a skill's name only when it is not its slug.
#
# unpack_graph turns it back into exactly what build_graph made, and the
# tests hold both it and graph.js's own decoder to that on the real library.
# Every other reader -- the build, crumb_check, the tests -- works on the
# unpacked graph; only the file and the browser see this form.

PACKED_SCHEMA = 2

_KINDS = ("body", "reference")


def slugify_heading(title: str) -> str:
    """content_pipeline.slugify, restated here so this module stays free of
    it and graph.js can mirror it line for line."""
    normalized = title.lower().replace("'", "").replace("’", "")
    slug = re.sub(r"[^a-z0-9]+", "-", normalized).strip("-")
    return slug or "section"


def _trim(fields: list, defaults: list) -> list:
    """Drop trailing fields that equal their default. The defaults line up
    with the *end* of fields."""
    fields = list(fields)
    offset = len(fields) - len(defaults)
    while len(fields) > offset and fields[-1] == defaults[len(fields) - 1 - offset]:
        fields.pop()
    return fields


def pack_graph(graph: dict) -> dict:
    """build_graph's graph in the compact form graph.json is written in."""
    slugs = [skill["slug"] for skill in graph["skills"]]
    index = {slug: i for i, slug in enumerate(slugs)}
    families = [c["slug"] for c in graph["categories"]]

    def citations(refs: dict, quotes: dict, where: str) -> list:
        stray = set(quotes) - set(refs)
        if stray:
            raise ValueError(f"{where} quotes {sorted(stray)} without citing them; "
                             "schema 2 carries a quote on its citation")
        return [[index[slug], count] + ([quotes[slug]] if slug in quotes else [])
                for slug, count in refs.items()]

    def heading_id(node: dict):
        return None if node["id"] == slugify_heading(node["title"]) else node["id"]

    skills = []
    for skill in graph["skills"]:
        where = skill["slug"]
        sections = []
        for section in skill["sections"]:
            subs = [_trim([sub["title"], sub["words"],
                           citations(sub["refs"], {}, where), heading_id(sub)],
                          [[], None])
                    for sub in section["subs"]]
            sections.append(_trim(
                [section["title"], _KINDS.index(section["kind"]), section["words"],
                 citations(section["refs"], section["quotes"], where), subs, heading_id(section)],
                [[], [], None]))
        skills.append(_trim(
            [skill["slug"], families.index(skill["category"]), skill["summary"], skill["words"],
             citations(skill["trigger"]["refs"], skill["trigger"]["quotes"], where), sections,
             skill["always_on"], skill["change_status"], skill["change_at"],
             None if skill["name"] == skill["slug"] else skill["name"]],
            [False, None, None, None]))
    return {
        "schema": PACKED_SCHEMA,
        "skill_url_template": graph["skill_url_template"],
        "categories": [[c["slug"], c["title"], [index[s] for s in c["skill_slugs"]]]
                       for c in graph["categories"]],
        "skills": skills,
    }


def unpack_graph(data: dict) -> dict:
    """graph.json as read back from disk, in build_graph's form. A schema-1
    file (every commit before the packed form) is already in it."""
    if data.get("schema") != PACKED_SCHEMA:
        return data
    slugs = [skill[0] for skill in data["skills"]]
    families = [c[0] for c in data["categories"]]

    def field(fields: list, i: int, default):
        return fields[i] if i < len(fields) else default

    def citations(packed: list) -> tuple[dict, dict]:
        refs, quotes = {}, {}
        for entry in packed:
            refs[slugs[entry[0]]] = entry[1]
            if len(entry) > 2:
                quotes[slugs[entry[0]]] = entry[2]
        return refs, quotes

    def heading_id(fields: list, i: int, title: str) -> str:
        explicit = field(fields, i, None)
        return slugify_heading(title) if explicit is None else explicit

    skills = []
    for s in data["skills"]:
        trigger_refs, trigger_quotes = citations(s[4])
        sections = []
        for sec in s[5]:
            refs, quotes = citations(field(sec, 3, []))
            sections.append({
                "id": heading_id(sec, 5, sec[0]), "title": sec[0], "kind": _KINDS[sec[1]],
                "words": sec[2], "refs": refs, "quotes": quotes,
                "subs": [{"id": heading_id(sub, 3, sub[0]), "title": sub[0], "words": sub[1],
                          "refs": citations(field(sub, 2, []))[0]}
                         for sub in field(sec, 4, [])],
            })
        name = field(s, 9, None)
        skills.append({
            "slug": s[0], "name": s[0] if name is None else name,
            "category": families[s[1]], "summary": s[2], "words": s[3],
            "always_on": field(s, 6, False), "change_status": field(s, 7, None),
            "change_at": field(s, 8, None),
            "trigger": {"refs": trigger_refs, "quotes": trigger_quotes},
            "sections": sections,
        })
    return {
        "schema": 1,
        "skill_url_template": data["skill_url_template"],
        "categories": [{"slug": c[0], "title": c[1], "skill_slugs": [slugs[i] for i in c[2]]}
                       for c in data["categories"]],
        "skills": skills,
    }


def edge_weights(graph: dict) -> dict[tuple[str, str], int]:
    """(citing slug, cited slug) -> number of mentions, trigger included. The
    browser derives the same thing; this copy exists for the tests and for
    the page's own summary line."""
    weights: dict[tuple[str, str], int] = {}
    for skill in graph["skills"]:
        sources = [skill["trigger"]["refs"]] + [s["refs"] for s in skill["sections"]]
        for refs in sources:
            for slug, count in refs.items():
                key = (skill["slug"], slug)
                weights[key] = weights.get(key, 0) + count
    return weights


def banner_summary(graph: dict) -> dict:
    """The few numbers, and the family-level picture, the landing page's
    announcement banner draws -- small enough to ride in the page itself, so
    the banner never fetches graph.json for a teaser.

    families: one entry per category, in catalog order, with its skill count.
    links: [a, b, citations] for every pair of families with any citation
    between them, either way, a < b. max_steps is the longest of the
    shortest walks along citations; reachable says whether every skill can
    reach every other, which is what makes max_steps worth a sentence."""
    weights = edge_weights(graph)
    category_index = {c["slug"]: i for i, c in enumerate(graph["categories"])}
    family_of = {s["slug"]: category_index[s["category"]] for s in graph["skills"]}
    links: dict[tuple[int, int], int] = {}
    for (source, target), count in weights.items():
        a, b = family_of[source], family_of[target]
        if a == b:
            continue
        key = (min(a, b), max(a, b))
        links[key] = links.get(key, 0) + count
    mutual = sum(1 for (a, b) in weights if a < b and (b, a) in weights)

    out: dict[str, list[str]] = {}
    for (source, target) in weights:
        out.setdefault(source, []).append(target)
    slugs = [s["slug"] for s in graph["skills"]]
    max_steps, reachable = 0, True
    for start in slugs:
        dist = {start: 0}
        frontier = [start]
        while frontier:
            nxt = []
            for node in frontier:
                for target in out.get(node, ()):
                    if target not in dist:
                        dist[target] = dist[node] + 1
                        nxt.append(target)
            frontier = nxt
        if len(dist) < len(slugs):
            reachable = False
        max_steps = max(max_steps, max(dist.values()))

    return {
        "skill_count": len(slugs),
        "pair_count": len(weights),
        "mutual_count": mutual,
        "max_steps": max_steps,
        "reachable": reachable,
        "families": [{"index": i, "title": c["title"], "count": len(c["skill_slugs"])}
                     for i, c in enumerate(graph["categories"])],
        "links": [[a, b, w] for (a, b), w in sorted(links.items())],
    }
