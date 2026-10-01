"""The About page: TBaguette's résumé, laid out as a bakery's day.

    python3 scripts/generate.py     # writes docs/about/index.html with the rest

Everything it says comes from about_profile.py. This module lays that out as
plain, complete HTML, and docs/assets/about.css and about.js only dress and
animate what is already here. The order matters: the page has to be a whole
résumé before the script runs, after it fails, in a search result, on paper,
and for a reader who asks for no motion. So the markup carries every fact,
the script adds a live layer on top (the `about-live` class is the line
between the two), and the print stylesheet reads the plain layer.

The page makes one move and commits to it: the record is a ledger of loaves,
each drawn with as many cuts on its crust as it is large, and above the ledger
sits a timeline that is also its navigator. Scrub or play it and the shelf
fills; click a loaf on it and the ledger opens that entry. Nothing else on the
page competes with that. The sections are the ones a résumé has:

    the opening       who, and what the title claims it will show
    the title         its three words, each answered by the work behind it
    the rules         what the work has in common, led by principle
    the shelf         the timeline and the ledger
    the counter       where the rest can be found
"""

from __future__ import annotations

import about_profile as data
import locales
import templates
from templates import (
    ABOUT_PATH, ENGLISH_STRINGS, Strings, _icon, _locale_url, _render_document,
    asset_url, escape_html,
)

_WORDS = ("zero", "one", "two", "three", "four", "five", "six", "seven", "eight",
          "nine", "ten", "eleven", "twelve")
_MONTHS = ("Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec")

# Runs the moment the page's root element exists, so the entrance's starting
# state is in force from the first paint (a deferred script runs too late).
# The entrance itself is written the safe way round in about.css -- the
# visible state is the resting style and the animation only supplies where it
# comes from -- so if this class never arrives, or the motion never runs, the
# whole page is simply there. If about.js never arrives the class comes off
# again, so the plain résumé is what the reader gets.
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


def _project_link(slug: str, label: str | None = None) -> str:
    """A link to a project's entry in the ledger. A real link, so it can be
    followed, shared and reached with Back, and so it works without script."""
    project = data.project_by_slug()[slug]
    domain = data.domain_by_slug()[project["domain"]]
    return (f'<a class="mention" href="#project={escape_html(slug)}" '
            f'{_dom_style(domain[2])}>{escape_html(label or project["name"])}</a>')


# ---------------------------------------------------------------------------
# The baguette. One drawing in the opening (large, baked on arrival), and the
# same bread in small as each loaf's crown, with as many cuts as its scale.
# ---------------------------------------------------------------------------

_LOAF_PATH = ("M70 190 C70 128 150 96 290 90 L710 82 C850 80 930 118 930 176 "
              "C930 236 850 262 710 264 L290 262 C150 260 70 246 70 190 Z")

# Centre x, centre y, half-width of each score on the large loaf. The scores
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
    return f"""<svg class="loaf-art" viewBox="0 0 1000 340" role="img" aria-label="A baguette, scored, rising in the oven">
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
    <rect x="0" y="60" width="1000" height="230" fill="url(#la-dough)"/>
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


# ---------------------------------------------------------------------------
# Sections
# ---------------------------------------------------------------------------


def _section_head(title: str, lede: str, title_id: str) -> str:
    # The heading carries itself: no kicker above it, no numeral beside it.
    return f"""<header class="part__head">
      <h2 class="part__title" id="{title_id}">{escape_html(title)}</h2>
      <p class="part__lede">{escape_html(lede)}</p>
    </header>"""


def _render_open(base_path: str) -> str:
    letters = "".join(
        f'<span class="open__ch" style="--i:{i}" aria-hidden="true">{escape_html(ch)}</span>'
        for i, ch in enumerate(data.NAME)
    )
    today = ", ".join(
        f'<a href="{escape_html(url)}" rel="noopener">{escape_html(label)}</a>' for label, url in data.OPEN_TODAY
    )
    return f"""<section class="open" id="top" aria-labelledby="about-name" data-about-open>
  <div class="container open__inner">
    <h1 class="open__name" id="about-name" aria-label="{escape_html(data.NAME)}">{letters}</h1>
    <p class="open__role"><span class="open__role-fr">{escape_html(data.TITLE_FR)}</span> {escape_html(data.TITLE_ROLE)} <span class="open__role-of">{escape_html(data.TITLE_OF)}</span> {escape_html(data.TITLE_FIELD)}</p>
    <p class="open__tagline">{escape_html(data.TAGLINE)}</p>
    <p class="open__today"><span>Open today:</span> {today}.</p>
    <div class="open__actions">
      <a class="btn btn--solid" href="#shelf">See the shelf</a>
      <button class="btn" type="button" data-about-print>Save as PDF</button>
      <a class="btn btn--quiet" href="{escape_html(data.GITHUB_URL)}" rel="noopener">@{escape_html(data.HANDLE)} on GitHub</a>
    </div>
  </div>
  <div class="open__stage" data-about-stage>
    <div class="open__loafbox"><div class="open__loaf">{_render_hero_loaf()}</div></div>
  </div>
</section>"""


def _render_title(stats: dict[str, int]) -> str:
    fmt = {"Years": _word(stats["years"]).capitalize(), "languages": _word(stats["languages"]),
           "surfaces": _word(stats["surfaces"])}
    summary = "".join(f"<p>{escape_html(p)}</p>" for p in data.SUMMARY)
    claims = []
    for word, claim, evidence in data.TITLE_CLAIMS:
        links = "".join(f"<li>{_project_link(slug)}</li>" for slug in evidence)
        claims.append(
            f'<article class="claim"><h3 class="claim__word">{escape_html(word)}</h3>'
            f'<p class="claim__text">{escape_html(claim.format(**fmt))}</p>'
            f'<ul class="claim__work" aria-label="Where it shows">{links}</ul></article>'
        )
    return f"""<section class="part" id="title" aria-labelledby="title-title">
  <div class="container">
    {_section_head("What the title claims", "Three words, each answered by work you can read about below.", "title-title")}
    <div class="summary">{summary}</div>
    <div class="claims">{''.join(claims)}</div>
  </div>
</section>"""


def _render_rules() -> str:
    rules = []
    for name, text, evidence in data.HOUSE_RULES:
        links = "".join(f"<li>{_project_link(slug)}</li>" for slug in evidence)
        rules.append(
            f'<li class="rule"><h3 class="rule__name">{escape_html(name)}</h3>'
            f'<p class="rule__text">{escape_html(text)}</p>'
            f'<ul class="rule__work" aria-label="Where it shows">{links}</ul></li>'
        )
    return f"""<section class="part part--rules" id="rules" aria-labelledby="rules-title">
  <div class="container">
    {_section_head("How the work is made", "What repeats from one project to the next, whatever the language.", "rules-title")}
    <ol class="rules">{''.join(rules)}</ol>
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
    edges = _warp(list(data.PROJECTS), first_year, last_year)
    placed = _place_dots(list(data.PROJECTS), first_year, last_year, edges)
    lane_count = max(lane for _, _, lane in placed) + 1
    domains = data.domain_by_slug()
    dots = []
    for p, x, lane in placed:
        domain = domains[p["domain"]]
        dots.append(
            f'<li class="rise__slot" data-rise-slot data-x="{x:.3f}" data-slug="{escape_html(p["slug"])}" '
            f'style="--x:{x:.2f}%;--lane:{lane};--s:{_dot_size_rem(p["scale"]):.2f}rem;--dom:var(--graph-cat-{domain[2]})">'
            f'<a class="rise__dot" href="#project={escape_html(p["slug"])}">'
            f'<span class="visually-hidden">{escape_html(p["name"])}, {escape_html(_month_label(p["first"]))}</span></a>'
            f'<span class="rise__tip" aria-hidden="true">{escape_html(p["name"])}<em>{escape_html(_month_label(p["first"]))}</em></span></li>'
        )
    ticks = "".join(
        f'<span class="rise__tick" style="--x:{edges[(y - first_year) * 12]:.2f}%">{y}</span>'
        for y in range(first_year, last_year + 1)
    )
    warp = ",".join(f"{e:.3f}" for e in edges)
    return f"""<div class="rise" data-rise data-first-year="{first_year}" data-warp="{warp}">
      <div class="rise__bar">
        <p class="rise__readout" data-rise-readout role="status" aria-live="polite">All {len(data.PROJECTS)} loaves are out.</p>
        <div class="rise__controls">
          <button class="btn btn--solid btn--small" type="button" data-rise-play>Play the years</button>
          <label class="rise__range"><span class="visually-hidden">Scrub through the years</span>
            <input type="range" min="0" max="1000" value="1000" step="1" data-rise-range>
          </label>
        </div>
      </div>
      <div class="rise__scroll" data-rise-scroll><div class="rise__track" data-rise-track style="--lanes:{lane_count}">
        <ul class="rise__dots" data-rise-dots>{''.join(dots)}</ul>
        <div class="rise__axis" aria-hidden="true">{ticks}</div>
      </div></div>
    </div>"""


def _render_entry(project: dict) -> str:
    domain = data.domain_by_slug()[project["domain"]]
    tags = " ".join(project["tags"])
    stack = "".join(f"<li>{escape_html(data.TAGS[t][0])}</li>" for t in project["tags"])
    public = project["vis"] == "public"
    facts = [f'<span class="entry__family">{escape_html(domain[1])}</span>',
             "Public" if public else "Private repository"]
    if project.get("ai"):
        facts.append("Built in pair with AI coding agents")
    method = "".join(f"<li>{escape_html(m)}</li>" for m in project["method"])
    lineage = (f'<p class="entry__lineage">{escape_html(project["lineage"])}</p>'
               if project.get("lineage") else "")
    repos = project.get("repos", "")
    kept = f'Kept {escape_html(_span_label(project["first"], project["last"]))}.'
    if repos:
        kept += f" Spans {escape_html(repos)}."
    link = (f'<p class="entry__link"><a href="{escape_html(project["url"])}" rel="noopener">Open it on the web</a></p>'
            if public and project.get("url") else "")
    return f"""<li class="entry-slot" data-slot>
  <article class="entry" id="project={escape_html(project["slug"])}" data-project="{escape_html(project["slug"])}" data-domain="{escape_html(project["domain"])}" data-tags="{escape_html(tags)}" data-first="{project["first"]}" data-last="{project["last"]}" data-scale="{project["scale"]}" data-vis="{project["vis"]}" {_dom_style(domain[2])}>
    <div class="entry__glyph">{_render_crown(project["scale"])}<p class="entry__dates">{escape_html(_span_label(project["first"], project["last"]))}</p></div>
    <div class="entry__main">
      <h4 class="entry__name">{escape_html(project["name"])}</h4>
      <p class="entry__pitch">{escape_html(project["pitch"])}</p>
      <p class="entry__facts">{' · '.join(facts)}</p>
      <ul class="entry__stack" aria-label="Ingredients">{stack}</ul>
      <details class="entry__more" data-more>
        <summary>How it&rsquo;s made</summary>
        <div class="entry__body">
          {lineage}
          <ol class="entry__method">{method}</ol>
          <p class="entry__kept">{kept}</p>
          {link}
        </div>
      </details>
    </div>
  </article>
</li>"""


def _render_shelf(base_path: str) -> str:
    stats = data.stats()
    count = len(data.PROJECTS)
    lede = (f"{_word(count).capitalize()} projects over {_word(stats['years'])} years, in "
            f"{_word(stats['languages'])} languages, on {_word(stats['surfaces'])} surfaces. "
            "The cuts on each loaf’s crust are its size: one for a sketch, five for a flagship. "
            "On the timeline, time is stretched where there was more to bake.")
    filters = [f'<button class="chip" type="button" data-domain-filter="all" aria-pressed="true">All <span>{count}</span></button>']
    for slug, name, color, blurb in data.DOMAINS:
        n = sum(1 for p in data.PROJECTS if p["domain"] == slug)
        filters.append(
            f'<button class="chip chip--family" type="button" data-domain-filter="{slug}" aria-pressed="false" '
            f'title="{escape_html(blurb)}" {_dom_style(color)}>{escape_html(name)} <span>{n}</span></button>'
        )
    counts = data.tag_counts()
    groups = []
    for kind, label in (("language", "Languages"), ("platform", "Surfaces"), ("craft", "Crafts")):
        chips = "".join(
            f'<li><button class="chip" type="button" data-tag="{tag}" aria-pressed="false">'
            f'{escape_html(name)} <span>{counts[tag]}</span></button></li>'
            for tag, (name, tag_kind) in data.TAGS.items() if tag_kind == kind and tag in counts
        )
        groups.append(f'<div class="ingredients__group"><p class="ingredients__label">{label}</p><ul class="ingredients__list">{chips}</ul></div>')

    # The ledger is grouped by era, newest first, with each group's projects
    # newest first too: a career read as a trajectory, not a heap of cards.
    last = max(p["last"] for p in data.PROJECTS)
    eras = []
    for i, (start, end, name, text) in reversed(list(enumerate(data.ERAS))):
        members = sorted((p for p in data.PROJECTS if start <= p["first"] <= end),
                         key=lambda p: (p["first"], p["scale"], p["slug"]), reverse=True)
        if not members:
            continue
        entries = "".join(_render_entry(p) for p in members)
        eras.append(
            f'<section class="era" data-era-group aria-labelledby="era-{i}">'
            f'<header class="era__head"><h3 class="era__name" id="era-{i}">{escape_html(name)} '
            f'<span class="era__years">{escape_html(_era_years(start, end, last))}</span></h3>'
            f'<p class="era__text">{escape_html(text)}</p></header>'
            f'<ol>{entries}</ol></section>'
        )
    return f"""<section class="part part--shelf" id="shelf" aria-labelledby="shelf-title">
  <div class="container">
    {_section_head("The shelf", lede, "shelf-title")}
    {_render_rise()}
    <div class="shelf" data-shelf>
      <div class="shelf__bar">
        <div class="shelf__filters" role="group" aria-label="Filter by kind of project">{''.join(filters)}</div>
        <label class="shelf__search"><span class="visually-hidden">Search the projects</span>
          {_icon("icon-search", css_class="icon shelf__search-icon", base_path=base_path)}
          <input type="search" placeholder="Search the shelf" autocomplete="off" data-search>
        </label>
      </div>
      <details class="ingredients" data-ingredients>
        <summary>Filter by ingredient</summary>
        <div class="ingredients__groups">{''.join(groups)}</div>
      </details>
      <p class="shelf__status" data-status role="status" aria-live="polite">{count} projects.</p>
      <div data-ledger>{''.join(eras)}</div>
      <p class="shelf__empty" data-empty hidden>Nothing on the shelf matches. <button type="button" class="linkish" data-reset>Clear the filters</button></p>
    </div>
  </div>
</section>"""


def _render_counter(base_path: str, locale: "locales.Locale", skill_count: int) -> str:
    home = _locale_url(locale, base_path, "")
    atelier = data.project_by_slug()["tbaguette-atelier"]
    skills = f"{skill_count} skills for AI agents" if skill_count else "Skills for AI agents"
    return f"""<section class="part" id="counter" aria-labelledby="counter-title">
  <div class="container">
    {_section_head("The counter", "The rest is a conversation. Here is where to start it.", "counter-title")}
    <ul class="counter__links">
      <li><a href="{escape_html(data.GITHUB_URL)}" rel="noopener">@{escape_html(data.HANDLE)} on GitHub</a></li>
      <li><a href="{escape_html(data.SITE_URL)}" rel="noopener">TBaguette’s Atelier: {escape_html(skills)}</a></li>
      <li><a href="{escape_html(atelier["url"])}" rel="noopener">The source of this site</a></li>
      <li><a href="{home}">Back to the skills</a></li>
    </ul>
    <p class="counter__note">Most of the projects above live in private repositories, which is why their entries describe the work instead of linking to it. The public ones say so. Forks and unmodified templates are not listed.</p>
    <p class="counter__colophon">Written in plain HTML, CSS and JavaScript: no framework, no tracker. Built in a session with Claude Code; the facts come from reading the repositories, not from memory.</p>
  </div>
</section>"""


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
{_render_open(base_path)}
{_render_title(stats)}
{_render_rules()}
{_render_shelf(base_path)}
{_render_counter(base_path, locale, skill_count)}
</div>
<script src="{asset_url("about.js", base_path)}" defer></script>"""
    description = (f"{data.NAME}, {data.TITLE_FR} {data.TITLE_ROLE} {data.TITLE_OF} "
                   f"{data.TITLE_FIELD}: {stats['projects']} projects in {stats['languages']} "
                   f"languages since {data.SINCE}, as a ledger of loaves.")
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
