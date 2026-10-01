"""The About page: TBaguette's résumé, laid out as a bakery's day.

    python3 scripts/generate.py     # writes docs/about/index.html with the rest

Everything it says comes from about_profile.py. This module lays that out as
plain, complete HTML, and docs/assets/about.css and about.js only dress and
animate what is already here. The order matters: the page has to be a whole
résumé before the script runs, after it fails, in a search result, on paper,
and for a reader who asks for no motion. So the markup carries every fact, the
script adds a live layer on top (the `about-live` class is the line between the
two), and the print stylesheet reads the plain layer.

The chapters are the stages a loaf goes through, and they are also the
sections a résumé has:

    I    The dough    who, and what the title claims
    II   The pantry   the ingredients: languages, platforms, crafts
    III  The bakery   the projects, as loaves, each with its recipe
    IV   The rise     the same record laid along a timeline you can play
    V    The counter  where to find the rest
"""

from __future__ import annotations

import about_profile as data
import locales
import templates
from templates import (
    ABOUT_PATH, ENGLISH_STRINGS, Strings, _icon, _locale_url, _render_document,
    asset_url, escape_html,
)

_ROMAN = ("I", "II", "III", "IV", "V")
_WORDS = ("zero", "one", "two", "three", "four", "five", "six", "seven", "eight",
          "nine", "ten", "eleven", "twelve")
_MONTHS = ("Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec")

# The seal's ring of words. Repeated to fill the circle; the middle dot is the
# separator, spaced so the ring reads as words rather than as a smear.
_SEAL_TEXT = "MAÎTRE BOULANGER · AUTONOMOUS SOFTWARE · SINCE {since} · "


# Runs the moment the page's root element exists, so the hidden starting
# states of the entrance are in force before the first paint rather than a
# beat after it (a deferred script runs too late, and the finished page would
# flash and then vanish). If about.js never arrives, the class comes off again
# and the plain résumé is what the reader gets.
_GO_LIVE_JS = (
    "(function(){var r=document.querySelector('[data-about]');if(!r)return;"
    "r.classList.add('about-live');"
    "setTimeout(function(){if(!window.TBaguetteAbout)r.classList.remove('about-live')},6000)})();"
)


def _word(n: int) -> str:
    return _WORDS[n] if 0 <= n < len(_WORDS) else str(n)


def _month_label(iso: str) -> str:
    year, month = iso.split("-")
    return f"{_MONTHS[int(month) - 1]} {year}"


def _span_label(first: str, last: str) -> str:
    if first == last:
        return _month_label(first)
    if first[:4] == last[:4]:
        return f"{_MONTHS[int(first[5:]) - 1]}–{_month_label(last)}"
    return f"{_month_label(first)} – {_month_label(last)}"


def _era_years(start: str, end: str, last: str) -> str:
    """An era's years, ended no later than the record is: the last era is
    open, and a label promising a year that has not happened would be one."""
    first_year, last_year = start[:4], min(end[:4], last[:4])
    return first_year if first_year == last_year else f"{first_year}–{last_year}"


def _dom_style(color: int) -> str:
    return f'style="--dom:var(--graph-cat-{color})"'


# ---------------------------------------------------------------------------
# The baguette. One drawing, used in the hero (large, scored by the reader's
# scroll) and, scaled down with N cuts, as each loaf's crown.
# ---------------------------------------------------------------------------

_LOAF_PATH = ("M70 190 C70 128 150 96 290 90 L710 82 C850 80 930 118 930 176 "
              "C930 236 850 262 710 264 L290 262 C150 260 70 246 70 190 Z")

# Centre x, centre y, half-width of each score on the hero loaf. The scores
# lean the way a blade leans; the middle three are the longest.
_SCORES = ((250, 126, 62), (375, 119, 70), (510, 114, 70), (645, 112, 70), (765, 114, 62))


def _render_hero_loaf() -> str:
    scores = []
    for i, (x, y, w) in enumerate(_SCORES):
        inner = w - 8
        # Two groups: the outer one places and tilts the cut (an SVG
        # attribute), the inner one is what the stylesheet opens. A CSS
        # transform on the placing group would replace its attribute, and the
        # cut would fly to the corner of the drawing.
        scores.append(
            f'<g transform="translate({x} {y}) rotate(-17)"><g class="loaf-art__score" style="--i:{i}">'
            f'<path class="loaf-art__cut" d="M{-w} 0 C{-w * 0.64:.0f} -22 {w * 0.64:.0f} -22 {w} 0 '
            f'C{w * 0.64:.0f} 14 {-w * 0.64:.0f} 14 {-w} 0Z"/>'
            f'<path class="loaf-art__ear" d="M{-inner} -1 C{-inner * 0.62:.0f} -17 {inner * 0.62:.0f} -17 {inner} -1 '
            f'C{inner * 0.62:.0f} 7 {-inner * 0.62:.0f} 7 {-inner} -1Z"/></g></g>'
        )
    return f"""<svg class="loaf-art" viewBox="0 0 1000 340" role="img" aria-label="A baguette, scored, rising in the oven" data-about-loaf>
  <defs>
    <linearGradient id="la-crust" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#f3c887"/><stop offset="0.32" stop-color="#dda25c"/>
      <stop offset="0.7" stop-color="#b9772f"/><stop offset="1" stop-color="#6e3f16"/>
    </linearGradient>
    <linearGradient id="la-dough" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#f6e7c8"/><stop offset="0.5" stop-color="#ead2a3"/>
      <stop offset="1" stop-color="#c9a56d"/>
    </linearGradient>
    <linearGradient id="la-ends" x1="0" y1="0" x2="1" y2="0">
      <stop offset="0" stop-color="#000" stop-opacity=".4"/><stop offset=".16" stop-color="#000" stop-opacity="0"/>
      <stop offset=".84" stop-color="#000" stop-opacity="0"/><stop offset="1" stop-color="#000" stop-opacity=".44"/>
    </linearGradient>
    <linearGradient id="la-ear" x1="0" y1="0" x2="0" y2="1">
      <stop offset="0" stop-color="#fff3d6"/><stop offset="1" stop-color="#efc98b"/>
    </linearGradient>
    <radialGradient id="la-shine" cx="0.5" cy="0.5" r="0.5">
      <stop offset="0" stop-color="#fff6df" stop-opacity=".6"/><stop offset="1" stop-color="#fff6df" stop-opacity="0"/>
    </radialGradient>
    <filter id="la-grain" x="0" y="0" width="100%" height="100%">
      <feTurbulence type="fractalNoise" baseFrequency=".9" numOctaves="2" seed="7" result="n"/>
      <feColorMatrix in="n" type="matrix" values="0 0 0 0 .35  0 0 0 0 .18  0 0 0 0 .05  0 0 0 .9 -.28"/>
    </filter>
    <filter id="la-soft"><feGaussianBlur stdDeviation="7"/></filter>
    <clipPath id="la-body"><path d="{_LOAF_PATH}"/></clipPath>
  </defs>
  <ellipse class="loaf-art__shadow" cx="500" cy="298" rx="400" ry="22" filter="url(#la-soft)"/>
  <g clip-path="url(#la-body)">
    <rect class="loaf-art__dough" x="0" y="60" width="1000" height="230" fill="url(#la-dough)"/>
    <rect class="loaf-art__baked" x="0" y="60" width="1000" height="230" fill="url(#la-crust)"/>
    <rect class="loaf-art__grain" x="0" y="60" width="1000" height="230" filter="url(#la-grain)"/>
    <rect x="0" y="60" width="1000" height="230" fill="url(#la-ends)"/>
    <g>{''.join(scores)}</g>
    <ellipse class="loaf-art__shine" cx="420" cy="132" rx="270" ry="48" fill="url(#la-shine)"/>
  </g>
  <path d="{_LOAF_PATH}" fill="none" stroke="#4a2a0e" stroke-opacity=".5" stroke-width="2"/>
</svg>"""


def _render_crown(scale: int) -> str:
    """A loaf's crown: the same bread in small, with as many cuts as the
    project's scale, so size is drawn on the crust and not only said."""
    count = max(1, min(5, scale))
    cuts = "".join(
        f'<path class="crown__cut" d="M{14 + (i + 0.5) * (52 / count) - 3.5:.1f} 16 '
        f'l7 -4.4" style="--i:{i}"/>' for i in range(count)
    )
    return ('<svg class="crown" viewBox="0 0 80 28" aria-hidden="true">'
            '<path class="crown__body" d="M6 15 C6 8 14 5 26 4.6 L56 4 C68 3.8 75 8 75 14 '
            'C75 20.5 68 23.4 56 23.6 L26 23.4 C14 23.2 6 21.6 6 15Z"/>'
            f'{cuts}</svg>')


def _render_seal(base_path: str) -> str:
    ring = _SEAL_TEXT.format(since=data.SINCE)
    return f"""<div class="about-seal" aria-hidden="true">
      <svg viewBox="0 0 200 200" class="about-seal__ring">
        <defs><path id="seal-circle" d="M100 100 m-76 0 a76 76 0 1 1 152 0 a76 76 0 1 1 -152 0"/></defs>
        <circle cx="100" cy="100" r="96" class="about-seal__edge"/>
        <circle cx="100" cy="100" r="58" class="about-seal__edge about-seal__edge--inner"/>
        <text class="about-seal__text"><textPath href="#seal-circle" textLength="474" lengthAdjust="spacing">{escape_html(ring.strip())}</textPath></text>
      </svg>
      {_icon("icon-wheat", css_class="icon about-seal__glyph", base_path=base_path)}
    </div>"""


# ---------------------------------------------------------------------------
# Chapters
# ---------------------------------------------------------------------------


def _chapter_head(index: int, stage: str, title: str, lede: str, title_id: str) -> str:
    return f"""<header class="about-chapter__head" data-reveal>
      <p class="about-chapter__stage"><span class="about-chapter__num" aria-hidden="true">{_ROMAN[index]}</span>{escape_html(stage)}</p>
      <h2 class="about-chapter__title" id="{title_id}">{escape_html(title)}</h2>
      <p class="about-chapter__lede">{escape_html(lede)}</p>
      <svg class="about-cuts" viewBox="0 0 60 12" aria-hidden="true"><path d="M2 10 L12 2"/><path d="M22 10 L32 2"/><path d="M42 10 L52 2"/></svg>
    </header>"""


def _project_button(slug: str, label: str | None = None) -> str:
    project = data.project_by_slug()[slug]
    domain = data.domain_by_slug()[project["domain"]]
    return (f'<button class="evidence" type="button" data-open-project="{escape_html(slug)}" '
            f'{_dom_style(domain[2])}>{escape_html(label or project["name"])}</button>')


def _render_hero(stats: dict[str, int], base_path: str) -> str:
    letters = "".join(
        f'<span class="about-hero__ch" style="--i:{i}" aria-hidden="true">{escape_html(ch)}</span>'
        for i, ch in enumerate(data.NAME)
    )
    stat_items = (
        (stats["projects"], "projects on the shelf"),
        (stats["languages"], "languages"),
        (stats["surfaces"], "surfaces: phone, desk, browser, chat, terminal"),
        (stats["years"], "years on the record"),
    )
    stat_html = "".join(
        f'<li class="about-stats__item" style="--i:{i}"><strong class="about-stats__num" data-count="{n}">{n}</strong>'
        f'<span class="about-stats__label">{escape_html(label)}</span></li>'
        for i, (n, label) in enumerate(stat_items)
    )
    return f"""<section class="about-hero" id="top" aria-labelledby="about-name" data-about-hero>
  <canvas class="about-hero__dust" data-about-dust aria-hidden="true"></canvas>
  <div class="container about-hero__inner">
    <div class="about-hero__copy">
      <p class="about-hero__eyebrow">The résumé of</p>
      <h1 class="about-hero__name" id="about-name" aria-label="{escape_html(data.NAME)}">{letters}</h1>
      <p class="about-hero__role"><span class="about-hero__role-fr">{escape_html(data.TITLE_FR)}</span> {escape_html(data.TITLE_ROLE)} <span class="about-hero__role-of">{escape_html(data.TITLE_OF)}</span> {escape_html(data.TITLE_FIELD)}</p>
      <p class="about-hero__tagline">{escape_html(data.TAGLINE)}</p>
      <div class="about-hero__ctas">
        <a class="about-btn about-btn--solid" href="#bakery">See what&rsquo;s in the oven</a>
        <button class="about-btn" type="button" data-about-print>Save as PDF</button>
        <a class="about-btn about-btn--quiet" href="{escape_html(data.GITHUB_URL)}" rel="noopener">@{escape_html(data.HANDLE)} on GitHub</a>
      </div>
    </div>
  </div>
  <div class="about-hero__stage" data-about-stage>
    <div class="about-hero__loafbox">
      <div class="about-hero__loaf">{_render_hero_loaf()}</div>
    </div>
    <div class="about-hero__steam" aria-hidden="true"><i></i><i></i><i></i></div>
    {_render_seal(base_path)}
  </div>
  <div class="container about-hero__figures">
    <ul class="about-stats" data-about-stats data-reveal aria-label="At a glance">{stat_html}</ul>
  </div>
</section>"""


def _render_dough(stats: dict[str, int]) -> str:
    fmt = {"Years": _word(stats["years"]).capitalize(), "languages": _word(stats["languages"]),
           "surfaces": _word(stats["surfaces"])}
    summary = "".join(f"<p>{escape_html(p)}</p>" for p in data.SUMMARY)

    words = []
    panels = []
    for i, (word, claim, evidence) in enumerate(data.TITLE_CLAIMS):
        pressed = "true" if i == 0 else "false"
        words.append(
            f'<button class="anatomy__word" type="button" id="claim-tab-{i}" aria-expanded="{pressed}" '
            f'aria-controls="claim-{i}" data-claim="{i}">{escape_html(word)}</button>'
        )
        if i == 0:
            words.append(" ")
        elif i == 1:
            words.append(f' <span class="anatomy__sep">{escape_html(data.TITLE_OF)}</span> ')
        buttons = "".join(f"<li>{_project_button(slug)}</li>" for slug in evidence)
        panels.append(
            f'<div class="anatomy__panel" id="claim-{i}" role="region" aria-labelledby="claim-tab-{i}">'
            f'<p class="anatomy__claim">{escape_html(claim.format(**fmt))}</p>'
            f'<ul class="anatomy__evidence" aria-label="Where it shows">{buttons}</ul></div>'
        )
    # Three live words and one connective: the title reads as it is written.
    title_html = "".join(words)

    rules = []
    for i, (name, text, evidence) in enumerate(data.HOUSE_RULES):
        buttons = "".join(f"<li>{_project_button(slug)}</li>" for slug in evidence)
        rules.append(
            f'<li class="rule" style="--i:{i}" data-reveal>'
            f'<h3 class="rule__name"><span class="rule__n" aria-hidden="true">{i + 1}</span>{escape_html(name)}</h3>'
            f'<p class="rule__text">{escape_html(text)}</p>'
            f'<ul class="rule__evidence" aria-label="Where it shows">{buttons}</ul></li>'
        )

    return f"""<section class="about-chapter" id="dough" aria-labelledby="dough-title" data-chapter="Dough">
  <div class="container">
    {_chapter_head(0, "Stage one · Mix", "The dough", "Who this is, and what the title claims, with the work that backs each word.", "dough-title")}
    <div class="dough">
      <div class="dough__summary" data-reveal>{summary}</div>
      <div class="anatomy" data-anatomy data-reveal>
        <p class="anatomy__label">Anatomy of a title <span>(tap a word)</span></p>
        <p class="anatomy__title" data-anatomy-title>{title_html}</p>
        <div class="anatomy__panels">{''.join(panels)}</div>
      </div>
    </div>
    <h3 class="about-sub" data-reveal>House rules</h3>
    <ol class="rules">{''.join(rules)}</ol>
  </div>
</section>"""


def _render_pantry() -> str:
    counts = data.tag_counts()
    domains = data.domain_by_slug()
    tag_domain: dict[str, dict[str, int]] = {}
    for project in data.PROJECTS:
        for tag in project["tags"]:
            tag_domain.setdefault(tag, {}).setdefault(project["domain"], 0)
            tag_domain[tag][project["domain"]] += 1
    groups = []
    for kind, label in (("language", "Languages"), ("platform", "Surfaces"), ("craft", "Crafts")):
        items = []
        for tag, (name, tag_kind) in data.TAGS.items():
            if tag_kind != kind or tag not in counts:
                continue
            main_domain = max(tag_domain[tag].items(), key=lambda kv: (kv[1], kv[0]))[0]
            color = domains[main_domain][2]
            n = counts[tag]
            items.append(
                f'<li><button class="ingredient" type="button" data-tag="{escape_html(tag)}" '
                f'data-weight="{n}" data-kind="{kind}" aria-pressed="false" {_dom_style(color)}>'
                f'<span class="ingredient__name">{escape_html(name)}</span>'
                f'<span class="ingredient__n">{n}</span></button></li>'
            )
        groups.append((label, items))
    shelf = "".join(
        f'<div class="pantry__group" data-kind-group><h3 class="pantry__kind">{escape_html(label)}</h3>'
        f'<ul class="pantry__list">{"".join(items)}</ul></div>'
        for label, items in groups
    )
    return f"""<section class="about-chapter about-chapter--pantry" id="pantry" aria-labelledby="pantry-title" data-chapter="Pantry">
  <div class="container">
    {_chapter_head(1, "Stage two · Proof", "The pantry", "Every ingredient this shelf has used, sized by how many projects it went into. Pick one to see the loaves it is in.", "pantry-title")}
    <div class="pantry" data-pantry data-reveal>
      <div class="pantry__field" data-pantry-field>{shelf}</div>
      <p class="pantry__status" data-pantry-status role="status" aria-live="polite">Nothing picked: all of the shelf is showing.</p>
    </div>
  </div>
</section>"""


def _render_loaf_card(project: dict) -> str:
    domain = data.domain_by_slug()[project["domain"]]
    tags = " ".join(project["tags"])
    stack = "".join(f'<li>{escape_html(data.TAGS[t][0])}</li>' for t in project["tags"][:4])
    extra = len(project["tags"]) - 4
    if extra > 0:
        stack += f'<li class="loaf__more-chip">+{extra}</li>'
    public = project["vis"] == "public"
    vis = ("Public" if public else "Private repository")
    ai = '<span class="loaf__ai" title="Built in pair with AI coding agents">AI-assisted</span>' if project.get("ai") else ""
    method = "".join(f"<li>{escape_html(m)}</li>" for m in project["method"])
    everything = "".join(f"<li>{escape_html(data.TAGS[t][0])}</li>" for t in project["tags"])
    lineage = (f'<p class="loaf__lineage">{escape_html(project["lineage"])}</p>'
               if project.get("lineage") else "")
    link = (f'<a class="loaf__link" href="{escape_html(project["url"])}" rel="noopener">View it on the web</a>'
            if public and project.get("url") else "")
    repos = project.get("repos", "")
    return f"""<li class="loaf-slot" data-slot>
  <article class="loaf" id="loaf-{escape_html(project["slug"])}" data-project="{escape_html(project["slug"])}" data-domain="{escape_html(project["domain"])}" data-tags="{escape_html(tags)}" data-first="{project["first"]}" data-last="{project["last"]}" data-scale="{project["scale"]}" data-vis="{project["vis"]}" {_dom_style(domain[2])}>
    <div class="loaf__top">
      <p class="loaf__domain">{escape_html(domain[1])}</p>
      {_render_crown(project["scale"])}
    </div>
    <h3 class="loaf__name"><button class="loaf__open" type="button" data-open-project="{escape_html(project["slug"])}" aria-haspopup="dialog">{escape_html(project["name"])}</button></h3>
    <p class="loaf__pitch">{escape_html(project["pitch"])}</p>
    <ul class="loaf__stack" aria-label="Ingredients">{stack}</ul>
    <p class="loaf__meta"><span>{escape_html(_span_label(project["first"], project["last"]))}</span><span class="loaf__vis" data-vis="{project["vis"]}">{vis}</span>{ai}</p>
    <div class="loaf__detail" data-detail>
      {lineage}
      <h4 class="loaf__h">Ingredients</h4>
      <ul class="loaf__all">{everything}</ul>
      <h4 class="loaf__h">Method</h4>
      <ol class="loaf__method">{method}</ol>
      <p class="loaf__kept"><strong>Kept:</strong> {escape_html(_span_label(project["first"], project["last"]))}.{(" Spans " + escape_html(repos) + ".") if repos else ""}</p>
      {link}
    </div>
  </article>
</li>"""


def _render_bakery(base_path: str) -> str:
    domains = data.DOMAINS
    filters = ['<button class="chip is-on" type="button" data-domain-filter="all" aria-pressed="true">All <span>'
               f'{len(data.PROJECTS)}</span></button>']
    for slug, name, color, blurb in domains:
        n = sum(1 for p in data.PROJECTS if p["domain"] == slug)
        filters.append(
            f'<button class="chip" type="button" data-domain-filter="{slug}" aria-pressed="false" '
            f'title="{escape_html(blurb)}" {_dom_style(color)}>{escape_html(name)} <span>{n}</span></button>'
        )
    # Flagships first, newest first within a size: the shelf leads with its
    # best bread, the way a counter does.
    ordered = sorted(data.PROJECTS, key=lambda p: (-p["scale"], tuple(-int(x) for x in p["last"].split("-"))))
    cards = "".join(_render_loaf_card(p) for p in ordered)
    return f"""<section class="about-chapter" id="bakery" aria-labelledby="bakery-title" data-chapter="Bakery">
  <div class="container">
    {_chapter_head(2, "Stage three · Bake", "The bakery", "Every project, as a loaf. The cuts on the crust are its size: one for a sketch, five for a flagship. Open one for its recipe.", "bakery-title")}
    <div class="bakery" data-bakery data-reveal>
      <div class="bakery__bar">
        <div class="bakery__filters" role="group" aria-label="Filter by kind of project">{''.join(filters)}</div>
        <label class="bakery__search"><span class="visually-hidden">Search the projects</span>
          {_icon("icon-search", css_class="icon bakery__search-icon", base_path=base_path)}
          <input type="search" placeholder="Search the shelf" autocomplete="off" data-bakery-search>
        </label>
      </div>
      <p class="bakery__status" data-bakery-status role="status" aria-live="polite">{len(data.PROJECTS)} projects.</p>
      <ul class="bakery__grid" data-bakery-grid>{cards}</ul>
      <p class="bakery__empty" data-bakery-empty hidden>Nothing on the shelf matches. <button type="button" class="linkish" data-bakery-reset>Clear the filters</button></p>
    </div>
  </div>
</section>"""


# The timeline's geometry is worked out here, not in the script, so the page
# is right with scripting off and on paper. x is a share of the track's width,
# and a lane is a row to stack in when two dots would still touch. The track
# never gets narrower than _TRACK_REM (the stylesheet holds it there and lets a
# phone scroll), which is what makes the gaps below true in pixels.
#
# Time is not drawn evenly. A month in which something was started is given
# more room than a month in which nothing was, so a record that packs fifteen
# projects into its last four months does not pile them into a single column.
# The axis says so under its readout, and the years are still labelled where
# they fall.
_TRACK_REM = 64.0
_LANE_PITCH_REM = 1.75
_MONTH_BASE = 1       # every month gets this much room...
_MONTH_PER_START = 5  # ...and this much more for each project begun in it


def _month_index(iso: str) -> int:
    year, month = iso.split("-")
    return int(year) * 12 + int(month) - 1


def _dot_size_rem(scale: int) -> float:
    return 0.8 + 0.16 * scale


def _warp(projects: list[dict], first_year: int, last_year: int) -> list[float]:
    """Where each month starts along the track, as a percent: one more entry
    than there are months, the last being 100."""
    start = first_year * 12
    months = (last_year - first_year + 1) * 12
    weights = [float(_MONTH_BASE)] * months
    for project in projects:
        weights[_month_index(project["first"]) - start] += _MONTH_PER_START
    total = sum(weights)
    edges = [0.0]
    for weight in weights:
        edges.append(edges[-1] + weight / total * 100)
    return edges


def _place_dots(projects: list[dict], first_year: int, last_year: int,
                edges: list[float]) -> list[tuple[dict, float, int]]:
    """(project, x percent, lane) for each project. Projects begun in the same
    month share that month's room side by side; after that each goes in the
    lowest lane whose last dot is far enough left not to touch it."""
    start = first_year * 12
    by_month: dict[int, list[dict]] = {}
    for project in sorted(projects, key=lambda p: (p["first"], p["slug"])):
        by_month.setdefault(_month_index(project["first"]), []).append(project)
    positions = []
    for month, group in sorted(by_month.items()):
        left, right = edges[month - start], edges[month - start + 1]
        for j, project in enumerate(group):
            positions.append((project, left + (j + 0.5) / len(group) * (right - left)))
    lanes: list[tuple[float, float]] = []  # (x percent, size rem) of each lane's last dot
    placed = []
    for project, x in positions:
        size = _dot_size_rem(project["scale"])
        for lane, (last_x, last_size) in enumerate(lanes):
            gap = ((size + last_size) / 2 + 0.3) / _TRACK_REM * 100
            if x - last_x >= gap:
                lanes[lane] = (x, size)
                break
        else:
            lanes.append((x, size))
            lane = len(lanes) - 1
        placed.append((project, x, lane))
    return placed


def _render_rise() -> str:
    first = min(p["first"] for p in data.PROJECTS)
    last = max(p["last"] for p in data.PROJECTS)
    first_year, last_year = int(first[:4]), int(last[:4])
    start = first_year * 12
    edges = _warp(list(data.PROJECTS), first_year, last_year)
    placed = _place_dots(list(data.PROJECTS), first_year, last_year, edges)
    lane_count = max(lane for _, _, lane in placed) + 1
    domains = data.domain_by_slug()
    dots = []
    for p, x, lane in placed:
        domain = domains[p["domain"]]
        dots.append(
            f'<li class="rise__dot-slot" data-rise-slot data-x="{x:.3f}" data-first="{p["first"]}" data-slug="{escape_html(p["slug"])}" '
            f'style="--x:{x:.2f}%;--lane:{lane};--s:{_dot_size_rem(p["scale"]):.2f}rem;--dom:var(--graph-cat-{domain[2]})">'
            f'<button class="rise__dot" type="button" data-open-project="{escape_html(p["slug"])}">'
            f'<span class="visually-hidden">{escape_html(p["name"])}, {escape_html(_month_label(p["first"]))}</span></button>'
            f'<span class="rise__tip" aria-hidden="true">{escape_html(p["name"])}<em>{escape_html(_month_label(p["first"]))}</em></span></li>'
        )
    ticks = "".join(
        f'<span class="rise__tick" style="--x:{edges[(y - first_year) * 12]:.2f}%">{y}</span>'
        for y in range(first_year, last_year + 1)
    )
    eras = []
    for i, (era_start, era_end, name, text) in enumerate(data.ERAS):
        x_from = edges[max(0, _month_index(era_start) - start)]
        x_to = edges[min(len(edges) - 1, _month_index(era_end) - start + 1)]
        eras.append(
            f'<li class="era" data-era data-x-from="{x_from:.3f}" data-x-to="{x_to:.3f}" style="--i:{i}">'
            f'<p class="era__when">{escape_html(_era_years(era_start, era_end, last))}</p>'
            f'<h3 class="era__name">{escape_html(name)}</h3><p class="era__text">{escape_html(text)}</p></li>'
        )
    warp = ",".join(f"{e:.3f}" for e in edges)
    return f"""<section class="about-chapter about-chapter--rise" id="rise" aria-labelledby="rise-title" data-chapter="Rise">
  <div class="container">
    {_chapter_head(3, "Stage four · Rise", "The long rise", "The same record along its own time. Scrub through the years, or press play, and watch the shelf fill.", "rise-title")}
    <div class="rise" data-rise data-first-year="{first_year}" data-warp="{warp}" data-reveal>
      <div class="rise__readout" data-rise-readout role="status" aria-live="polite">All {len(data.PROJECTS)} loaves are out.</div>
      <p class="rise__note">Time is stretched where there was more to bake: a busy month gets more room than a quiet one.</p>
      <div class="rise__scroll" data-rise-scroll><div class="rise__track" data-rise-track style="--lanes:{lane_count}">
        <div class="rise__axis" aria-hidden="true">{ticks}</div>
        <ul class="rise__dots" data-rise-dots>{''.join(dots)}</ul>
      </div></div>
      <div class="rise__controls">
        <button class="about-btn about-btn--solid" type="button" data-rise-play>Play the years</button>
        <label class="rise__range"><span class="visually-hidden">Scrub through the years</span>
          <input type="range" min="0" max="1000" value="1000" step="1" data-rise-range>
        </label>
      </div>
      <ol class="eras">{''.join(eras)}</ol>
    </div>
  </div>
</section>"""


def _render_counter(base_path: str, locale: "locales.Locale", skill_count: int) -> str:
    home = _locale_url(locale, base_path, "")
    atelier = data.project_by_slug()["tbaguette-atelier"]
    skills = f"{skill_count} skills for AI agents" if skill_count else "Skills for AI agents"
    return f"""<section class="about-chapter about-chapter--counter" id="counter" aria-labelledby="counter-title" data-chapter="Counter">
  <div class="container">
    {_chapter_head(4, "Stage five · Serve", "The counter", "The rest is a conversation. Here is where to start it.", "counter-title")}
    <div class="counter" data-reveal>
      <ul class="counter__links">
        <li><a class="counter__link" href="{escape_html(data.GITHUB_URL)}" rel="noopener"><span class="counter__k">GitHub</span><span class="counter__v">@{escape_html(data.HANDLE)}</span></a></li>
        <li><a class="counter__link" href="{escape_html(data.SITE_URL)}" rel="noopener"><span class="counter__k">The Atelier</span><span class="counter__v">{escape_html(skills)}</span></a></li>
        <li><a class="counter__link" href="{escape_html(atelier["url"])}" rel="noopener"><span class="counter__k">Source</span><span class="counter__v">this very site, on GitHub</span></a></li>
      </ul>
      <p class="counter__note">Most of the projects above live in private repositories, which is why their cards describe the work instead of linking to it.</p>
      <p class="counter__colophon">Baked by hand in HTML, CSS and plain JavaScript: no framework, no tracker. Built in a session with Claude Code; the facts come from reading the repositories, not from memory.</p>
      <p class="counter__actions"><button class="about-btn" type="button" data-about-print>Save this as a PDF</button> <a class="about-btn about-btn--quiet" href="#top">Back to the oven door</a> <a class="about-btn about-btn--quiet" href="{home}">The Atelier</a></p>
    </div>
  </div>
</section>"""


def _render_recipe_dialog() -> str:
    return """<dialog class="recipe" data-recipe aria-labelledby="recipe-title">
  <div class="recipe__card" data-recipe-card>
    <button class="recipe__close" type="button" data-recipe-close aria-label="Close the recipe">&times;</button>
    <p class="recipe__domain" data-recipe-domain></p>
    <h2 class="recipe__title" id="recipe-title" data-recipe-title></h2>
    <p class="recipe__pitch" data-recipe-pitch></p>
    <div class="recipe__body">
      <section class="recipe__col" aria-labelledby="recipe-ing">
        <h3 class="recipe__h" id="recipe-ing">Ingredients</h3>
        <ul class="recipe__chips" data-recipe-stack></ul>
        <h3 class="recipe__h">Crust</h3>
        <div class="recipe__crown" data-recipe-crown></div>
        <p class="recipe__small" data-recipe-meta></p>
      </section>
      <section class="recipe__col" aria-labelledby="recipe-method">
        <h3 class="recipe__h" id="recipe-method">Method</h3>
        <ol class="recipe__steps" data-recipe-steps></ol>
        <p class="recipe__lineage" data-recipe-lineage></p>
      </section>
    </div>
    <footer class="recipe__foot">
      <button class="recipe__nav" type="button" data-recipe-prev aria-label="Previous loaf">&larr; Previous</button>
      <a class="recipe__link" data-recipe-link hidden rel="noopener"></a>
      <button class="recipe__nav" type="button" data-recipe-next aria-label="Next loaf">Next &rarr;</button>
    </footer>
  </div>
</dialog>"""


def _render_rail() -> str:
    stops = (("dough", "Dough"), ("pantry", "Pantry"), ("bakery", "Bakery"),
             ("rise", "Rise"), ("counter", "Counter"))
    items = "".join(
        f'<li><a class="about-rail__stop" href="#{slug}" data-rail-stop="{slug}"><span>{label}</span></a></li>'
        for slug, label in stops
    )
    return f"""<nav class="about-rail" aria-label="Chapters" data-about-rail>
  <ol class="about-rail__list">{items}</ol>
  <div class="about-rail__line" aria-hidden="true"><i data-rail-fill></i><svg class="about-rail__loaf" viewBox="0 0 24 24"><rect x="2.6" y="8.3" width="18.8" height="7.4" rx="3.7" transform="rotate(-45 12 12)"/></svg></div>
</nav>"""


def render_about_page(categories: list[dict], base_path: str = "",
                      last_updated_utc: str = "", *, skill_count: int = 0,
                      locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                      strings: Strings = ENGLISH_STRINGS,
                      plugin_version: str = "") -> str:
    """Full HTML document for /about/. Complete with scripting off; about.js
    adds the live layer (see the module docstring)."""
    stats = data.stats()
    main_html = f"""<div class="about" data-about>
<script>{_GO_LIVE_JS}</script>
<div class="about-progress" aria-hidden="true"><i></i></div>
{_render_rail()}
{_render_hero(stats, base_path)}
{_render_dough(stats)}
{_render_pantry()}
{_render_bakery(base_path)}
{_render_rise()}
{_render_counter(base_path, locale, skill_count)}
{_render_recipe_dialog()}
</div>
<script src="{asset_url("about.js", base_path)}" defer></script>"""
    description = (f"{data.NAME}, {data.TITLE_FR} {data.TITLE_ROLE} {data.TITLE_OF} "
                   f"{data.TITLE_FIELD}: {stats['projects']} projects in {stats['languages']} "
                   f"languages since {data.SINCE}, laid out as a bakery’s day.")
    return _render_document(
        title=f"{strings.about_page_title} — {templates.BRAND_ATELIER_TEXT}",
        meta_description=description,
        body_class="page-about",
        main_html=main_html,
        categories=categories,
        base_path=base_path,
        last_updated_utc=last_updated_utc,
        locale=locale,
        path_suffix=ABOUT_PATH,
        strings=strings,
        plugin_version=plugin_version,
        stylesheets=("about.css",),
    )
