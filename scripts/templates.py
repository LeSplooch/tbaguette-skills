"""La Boulangerie TBaguette — page templates.

Stdlib-only HTML string builder for the showcase site. No Jinja, no template
engine, no third-party dependency: every function returns a plain ``str``,
built with f-strings and small composable helpers. Zero install step is a
hard requirement for this project, and this module has none.

Public API — the contract the integration step (``generate.py``) calls once
content is available:

    render_index(categories, skills, base_path="") -> str
    render_skill_page(skill, *, prev_skill, next_skill, siblings, categories,
                       base_path="") -> str

Both are pure functions of their arguments: no file I/O, no network, no
environment reads, no module-level mutable state. Same input always produces
the same output string, so they're predictable to call from a script and
trivial to unit test (see test_templates.py).

``base_path`` exists because this site is served two ways: locally at the
true root (``base_path=""``), and from GitHub Pages as a project site, which
serves at ``https://<user>.github.io/<repo>/`` — a subpath, not the domain
root. Root-relative hrefs like ``/assets/styles.css`` resolve to the wrong
place under a subpath (they'd point at the domain root, dropping the repo
name), so every constructed href/src is prefixed with ``base_path`` instead
of hardcoding a leading slash. ``<base href>`` does not solve this: it only
affects document-relative references, not absolute-path ones — an
absolute-path href like ``/assets/x`` ignores a page's ``<base>`` entirely.

Trust boundary: every raw text field from the content dict (name, summary,
description, category titles, tab titles...) is passed through
``escape_html`` before it touches an f-string. The two exceptions are
``skill["body_html"]`` and each formidable stack/command's ``"html"`` field —
the content pipeline already renders and HTML-escapes those, so they are
injected verbatim; each injection site below is commented to make that trust
boundary visible rather than implicit. The name ``escape_html`` matches
``content_pipeline.py``'s own helper of the same job, deliberately — one
vocabulary for "make this safe to interpolate" across both halves of the
generator.
"""

from html import escape as _escape_html_impl

# ---------------------------------------------------------------------------
# Small internal helpers
# ---------------------------------------------------------------------------

# Category markers cycle through these three line-marks by category index.
# The content schema carries no per-category icon mapping, so this is
# deliberate decorative variety, not an invented semantic assignment.
_CATEGORY_ICONS = ("icon-wheat", "icon-crust", "icon-grain")

_THEME_STORAGE_KEY = "tbaguette-theme"

# Runs synchronously in <head>, before first paint, so the stored or
# OS-preferred theme applies before any pixel is drawn — the alternative is a
# flash of the wrong theme on every reload. Deliberately tiny and dependency
# free; the rest of the theme logic (the toggle button) lives in site.js.
_THEME_BOOTSTRAP_JS = (
    "(function(){try{"
    f"var s=localStorage.getItem('{_THEME_STORAGE_KEY}');"
    "var wantsFlour=s?s==='flour':matchMedia('(prefers-color-scheme: light)').matches;"
    "if(wantsFlour){document.documentElement.setAttribute('data-theme','flour');}"
    "}catch(e){}})();"
)


def escape_html(value: object) -> str:
    """HTML-escape a plain-text value for safe interpolation into markup."""
    return _escape_html_impl(str(value), quote=True)


def _icon(symbol_id: str, *, css_class: str = "icon", base_path: str = "") -> str:
    """A <use>-referenced icon from the shared /assets/icons.svg sprite."""
    return (
        f'<svg class="{css_class}" aria-hidden="true">'
        f'<use href="{base_path}/assets/icons.svg#{symbol_id}"></use></svg>'
    )


def _skill_href(skill: dict, base_path: str = "") -> str:
    return f"{base_path}/skills/{escape_html(skill['slug'])}/"


def _search_haystack(skill: dict) -> str:
    """Precomputed, lowercased text a card matches against, for site.js's
    client-side filter — cheaper and more robust than re-deriving searchable
    text from rendered DOM structure on every keystroke."""
    parts = (
        skill.get("name", ""),
        skill.get("summary", ""),
        skill.get("category_title", ""),
    )
    return escape_html(" ".join(str(p) for p in parts).lower())


def _join(*parts: str) -> str:
    """Join template fragments, dropping empty ones (an empty fragment means
    'this section doesn't apply to this skill' — e.g. no prev/next at a
    category boundary — and should leave no trace in the output)."""
    return "\n".join(p for p in parts if p)


# ---------------------------------------------------------------------------
# Shared chrome: document shell, header, footer, theme toggle
# ---------------------------------------------------------------------------


def _render_head(*, title: str, meta_description: str, base_path: str = "") -> str:
    desc = escape_html(meta_description)
    return f"""<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{escape_html(title)}</title>
<meta name="description" content="{desc}">
<meta property="og:type" content="website">
<meta property="og:title" content="{escape_html(title)}">
<meta property="og:description" content="{desc}">
<link rel="icon" type="image/svg+xml" href="{base_path}/assets/favicon.svg">
<link rel="preload" href="{base_path}/assets/fonts/fraunces-variable.woff2" as="font" type="font/woff2" crossorigin>
<link rel="preload" href="{base_path}/assets/fonts/work-sans-variable.woff2" as="font" type="font/woff2" crossorigin>
<link rel="stylesheet" href="{base_path}/assets/styles.css">
<script>{_THEME_BOOTSTRAP_JS}</script>"""


def _render_updated_time(last_updated_utc: str, base_path: str = "") -> str:
    # last_updated_utc is a UTC instant baked in at generation time (see
    # generate.py's own docstring on why it must be the very last step
    # before commit for this to be honest). Rendered here as a plain UTC
    # string so the page still says something true with JS disabled;
    # site.js's initUpdatedTime() replaces the text with both the visitor's
    # local time and UTC once it runs, since the visitor's own timezone
    # can't be known at build time. data-version-url points site.js's
    # initVersionCheck() at the generated docs/version.txt, which always
    # carries this same instant — see scripts/generate.py's _build_into().
    fallback = escape_html(last_updated_utc.replace("+00:00", "Z")) + " UTC"
    return f"""<p class="site-header__updated">
      <span class="site-header__updated-label">Updated</span>
      <time class="site-header__updated-value" datetime="{escape_html(last_updated_utc)}" data-format-updated data-version-url="{base_path}/version.txt">{fallback}</time>
    </p>"""


def _render_header(base_path: str = "", last_updated_utc: str = "") -> str:
    updated_html = _render_updated_time(last_updated_utc, base_path) if last_updated_utc else ""
    return f"""<header class="site-header">
  <div class="site-header__band" aria-hidden="true"></div>
  <div class="container site-header__inner">
    <a class="wordmark" href="{base_path}/">
      {_icon("icon-wheat", css_class="icon wordmark__icon", base_path=base_path)}
      <span class="wordmark__text">TBaguette<span class="wordmark__suffix">&rsquo;s Atelier</span></span>
    </a>
    <div class="site-header__actions">
      {updated_html}
      <button class="theme-toggle" type="button" data-theme-toggle aria-label="Switch to light theme">
        {_icon("icon-sun", css_class="icon theme-toggle__icon theme-toggle__icon--sun", base_path=base_path)}
        {_icon("icon-moon", css_class="icon theme-toggle__icon theme-toggle__icon--moon", base_path=base_path)}
      </button>
    </div>
  </div>
</header>"""


def _render_footer(categories: list[dict], base_path: str = "") -> str:
    links = _join(*(
        f'<li><a href="{base_path}/#{escape_html(cat["slug"])}">{escape_html(cat["title"])}</a></li>'
        for cat in categories
    ))
    return f"""<footer class="site-footer">
  <div class="container site-footer__inner">
    <p class="site-footer__brand"><strong>La Boulangerie TBaguette</strong> is home to
      TBaguette&rsquo;s Atelier — the judgment calls that sit between the ticket and
      the commit, organized like a proper bench.</p>
    <nav aria-label="Categories">
      <p class="site-footer__nav-title">Categories</p>
      <ul class="site-footer__categories">
{links}
      </ul>
    </nav>
  </div>
</footer>"""


def _render_document(*, title: str, meta_description: str, body_class: str,
                      main_html: str, categories: list[dict],
                      base_path: str = "", last_updated_utc: str = "") -> str:
    return f"""<!doctype html>
<html lang="en">
<head>
{_render_head(title=title, meta_description=meta_description, base_path=base_path)}
</head>
<body class="{body_class}">
<a class="skip-link" href="#main">Skip to content</a>
{_render_header(base_path, last_updated_utc)}
<main id="main">
{main_html}
</main>
{_render_footer(categories, base_path)}
<script src="{base_path}/assets/site.js" defer></script>
</body>
</html>
"""


# ---------------------------------------------------------------------------
# Landing page
# ---------------------------------------------------------------------------


# Clones on first run; pulls in place on every run after, rather than
# erroring on "destination already exists" the way a bare `git clone` does.
# Safe either way: git itself refuses to clone into a non-empty directory it
# doesn't own (proven — see scripts/test_install_command.py), so this can
# never overwrite unrelated content some other skill or plugin happens to
# have at the same path, only ever update its own prior clone.
#
# Verified verbatim (not just "should work in theory") against bash, zsh, and
# fish — see test_install_command.py's check_matches_published_command. WSL
# and Git Bash both run one of those three, so this one string already
# covers every POSIX-shell visitor.
INSTALL_COMMAND = (
    "[ -d ~/.claude/skills/TBaguette/.git ] && "
    "git -C ~/.claude/skills/TBaguette pull || "
    "git clone https://github.com/LeSplooch/tbaguette-skills.git "
    "~/.claude/skills/TBaguette"
)

# The same clone-or-pull logic for native Windows PowerShell (5.1, the
# Windows 10+ default, and 7+) — visitors on Windows who are NOT already in
# WSL or Git Bash need this instead, since `[ -d ... ] && ... || ...` isn't
# PowerShell syntax at all. -PathType Container mirrors POSIX's -d (a
# directory test specifically, not just "something exists here"). Not
# executable in this project's own CI (no PowerShell runtime in the build
# environment — see verify-install page for exactly what is and isn't
# machine-verified for each platform), so correctness here rests on
# Test-Path/git -C/git clone being standard, well-documented behavior rather
# than an automated cross-check the way the POSIX command has.
INSTALL_COMMAND_POWERSHELL = (
    'if (Test-Path "$HOME\\.claude\\skills\\TBaguette\\.git" -PathType Container) '
    '{ git -C "$HOME\\.claude\\skills\\TBaguette" pull } '
    "else "
    '{ git clone https://github.com/LeSplooch/tbaguette-skills.git "$HOME\\.claude\\skills\\TBaguette" }'
)

# Command Prompt (cmd.exe) equivalent — mentioned on the verify-install page
# rather than given equal billing in the install frame's tabs, since a
# developer on Windows who isn't in WSL/Git Bash overwhelmingly has
# PowerShell available (it's been the Windows default since Windows 10) and
# reaches cmd.exe by choice, not necessity. IF EXIST tests for a file or a
# directory — cmd.exe has no clean one-line directory-only test the way -d
# and -PathType Container do — which is loose in principle but never wrong
# in practice here, since a real git checkout's .git is always a directory.
INSTALL_COMMAND_CMD = (
    'if exist "%USERPROFILE%\\.claude\\skills\\TBaguette\\.git" '
    '(git -C "%USERPROFILE%\\.claude\\skills\\TBaguette" pull) '
    "else "
    '(git clone https://github.com/LeSplooch/tbaguette-skills.git "%USERPROFILE%\\.claude\\skills\\TBaguette")'
)

INSTALL_HINT_POSIX = "Works in bash, zsh, or fish — including WSL and Git Bash on Windows."
INSTALL_HINT_POWERSHELL = "PowerShell 5.1 or 7+, the Windows default terminal since Windows 10."


def _render_install_panel(*, panel_id: str, tab_id: str, command: str, hint: str,
                           selected: bool, base_path: str = "") -> str:
    escaped = escape_html(command)
    hidden_attr = "" if selected else " hidden"
    return f"""<div class="tabs__panel install-tabs__panel" role="tabpanel" id="{panel_id}"
             aria-labelledby="{tab_id}" tabindex="0"{hidden_attr}>
          <div class="install">
            <code class="install__command" id="{panel_id}-command">{escaped}</code>
            <button class="install__copy" type="button" data-copy-target="{panel_id}-command"
                    aria-label="Copy install command">
              <span class="install__copy-icons">
                <svg class="icon install__copy-icon install__copy-icon--copy" aria-hidden="true"><use href="{base_path}/assets/icons.svg#icon-copy"></use></svg>
                <svg class="icon install__copy-icon install__copy-icon--check" aria-hidden="true"><use href="{base_path}/assets/icons.svg#icon-check"></use></svg>
              </span>
              <span data-copy-label>Copy</span>
            </button>
          </div>
          <p class="install__hint">{escape_html(hint)}</p>
        </div>"""


def _render_install(base_path: str = "") -> str:
    posix_panel = _render_install_panel(
        panel_id="install-posix", tab_id="tab-install-posix",
        command=INSTALL_COMMAND, hint=INSTALL_HINT_POSIX,
        selected=True, base_path=base_path,
    )
    powershell_panel = _render_install_panel(
        panel_id="install-powershell", tab_id="tab-install-powershell",
        command=INSTALL_COMMAND_POWERSHELL, hint=INSTALL_HINT_POWERSHELL,
        selected=False, base_path=base_path,
    )
    return f"""<div class="install-frame">
  <p class="install-frame__label">
    {_icon("icon-crust", base_path=base_path)}
    Install TBaguette&rsquo;s skills
  </p>
  <div class="install-frame__body">
    <div class="tabs install-tabs" data-tabs data-autoselect-platform="true">
      <div class="tabs__list install-tabs__list" role="tablist" aria-label="Choose your platform">
        <button class="tabs__tab" type="button" role="tab" id="tab-install-posix"
                aria-controls="install-posix" aria-selected="true" tabindex="0"
                data-platform="posix">macOS / Linux</button>
        <button class="tabs__tab" type="button" role="tab" id="tab-install-powershell"
                aria-controls="install-powershell" aria-selected="false" tabindex="-1"
                data-platform="windows">Windows (PowerShell)</button>
      </div>
      <div class="tabs__panels install-tabs__panels">
{posix_panel}
{powershell_panel}
      </div>
    </div>
    <p class="install-frame__note">
      {_icon("icon-check", base_path=base_path)}
      <span>Only ever touches this folder — verified against your other skills, not
      just claimed. <a href="{base_path}/verify-install/">See how</a>.</span>
    </p>
    <p class="install-frame__note">
      {_icon("icon-check", base_path=base_path)}
      <span>Restart Claude Code (or run <code>/reload-plugins</code>), then invoke a
      skill as <code>TBaguette:skill-name</code>. This is for Claude Code specifically —
      the Claude Desktop app and claude.ai chat load skills from your account instead of
      this folder, so cloning here won't make them appear there.</span>
    </p>
  </div>
</div>"""


def _render_hero(skill_count: int, category_count: int, base_path: str = "") -> str:
    lede = (
        f"{skill_count} Claude Code skills for the craft between the ticket and the "
        f"commit, across {category_count} categories — findable by name, browsable "
        "below, and written like something a colleague actually handed you. Grown "
        "out of my own projects over time, this collection is created and updated "
        "automatically — and often — by Claude Opus, always its latest version, "
        "as I code."
    )
    return f"""<section class="hero">
  <div class="container">
    <h1 class="hero__headline">An atelier for the way you build.</h1>
    {_render_install(base_path)}
    <p class="hero__lede">{escape_html(lede)}</p>
    {_render_search_field(base_path)}
  </div>
</section>"""


def _render_search_field(base_path: str = "") -> str:
    return f"""<div class="search" data-search-root>
  <div class="search__field">
    <svg class="icon search__icon" aria-hidden="true"><use href="{base_path}/assets/icons.svg#icon-search"></use></svg>
    <label class="visually-hidden" for="skill-search">Search skills</label>
    <input class="search__input" type="search" id="skill-search" data-search-input
           placeholder="Search by name, summary, or category…" autocomplete="off">
    <button class="search__clear" type="button" data-search-clear hidden>Clear</button>
  </div>
  <p class="search__status" data-search-status aria-live="polite"></p>
</div>"""


def _render_search_empty_state(skill_count: int) -> str:
    return f"""<p class="search__empty container" data-search-empty hidden>
  No skills match <span class="search__empty-query" data-search-empty-query></span>.
  <button type="button" data-search-reset>Clear search</button> to see all {skill_count} again.
</p>"""


def _render_change_badge(status: str | None) -> str:
    """"New"/"Updated" — set on a skill dict by generate.py from a real git
    status check, not guessed here. Empty string (renders as nothing) for
    every skill not touched by the update currently being shipped, which is
    the common case — most page loads carry zero of these."""
    if status not in ("new", "updated"):
        return ""
    label = "New" if status == "new" else "Updated"
    return f'<span class="change-badge change-badge--{status}">{label}</span>'


def _render_skill_card(skill: dict, base_path: str = "") -> str:
    badge = _render_change_badge(skill.get("change_status"))
    return f"""<a class="card" href="{_skill_href(skill, base_path)}" data-search-card data-search-terms="{_search_haystack(skill)}">
  <span class="card__name-row">
    <span class="card__name">{escape_html(skill['name'])}</span>
    {badge}
  </span>
  <p class="card__summary">{escape_html(skill.get('summary', ''))}</p>
  <span class="card__foot">
    <span class="tag">{escape_html(skill.get('category_title', ''))}</span>
    <span class="card__arrow" aria-hidden="true">→</span>
  </span>
</a>"""


def _render_category_section(category: dict, skills: dict, icon_index: int,
                              base_path: str = "") -> str:
    slug = category["slug"]
    skill_slugs = category.get("skill_slugs", [])
    count = len(skill_slugs)
    icon_id = _CATEGORY_ICONS[icon_index % len(_CATEGORY_ICONS)]
    cards = _join(*(
        _render_skill_card(skills[s], base_path) for s in skill_slugs if s in skills
    ))
    noun = "skill" if count == 1 else "skills"
    return f"""<section class="category-section" id="{escape_html(slug)}" data-category-section>
  <div class="container">
    <div class="category-section__head">
      {_icon(icon_id, css_class="icon category-section__icon", base_path=base_path)}
      <h2 class="category-section__title">{escape_html(category['title'])}</h2>
      <span class="tag category-section__count" data-category-count="{count}">{count} {noun}</span>
    </div>
    <div class="card-grid" data-card-grid>
{cards}
    </div>
  </div>
</section>"""


def render_index(categories: list[dict], skills: dict, base_path: str = "",
                  last_updated_utc: str = "") -> str:
    """Full HTML document string for the landing page."""
    sections = _join(*(
        _render_category_section(cat, skills, i, base_path)
        for i, cat in enumerate(categories)
    ))
    skill_count = len(skills)
    category_count = len(categories)
    main_html = _join(
        _render_hero(skill_count, category_count, base_path),
        _render_search_empty_state(skill_count),
        f'<div data-categories>\n{sections}\n</div>',
    )
    return _render_document(
        title="TBaguette’s Atelier — Claude Code skills, organized",
        meta_description=(
            f"{skill_count} Claude Code skills for the craft between the ticket and "
            f"the commit, organized into {category_count} categories and cross-linked "
            "for the moment you need one."
        ),
        body_class="page-index",
        main_html=main_html,
        categories=categories,
        base_path=base_path,
        last_updated_utc=last_updated_utc,
    )


# ---------------------------------------------------------------------------
# Skill page
# ---------------------------------------------------------------------------


def _render_breadcrumb(skill: dict, base_path: str = "") -> str:
    return f"""<nav class="container breadcrumb" aria-label="Breadcrumb">
  <a href="{base_path}/">Home</a>
  <span class="breadcrumb__sep" aria-hidden="true">/</span>
  <a href="{base_path}/#{escape_html(skill.get('category_slug', ''))}">{escape_html(skill.get('category_title', ''))}</a>
  <span class="breadcrumb__sep" aria-hidden="true">/</span>
  <span class="breadcrumb__current" aria-current="page">{escape_html(skill['name'])}</span>
</nav>"""


def _render_skill_head(skill: dict) -> str:
    badge = _render_change_badge(skill.get("change_status"))
    return f"""<div class="skill-article__head">
  <div class="skill-article__title-row">
    <h1 class="skill-article__title">{escape_html(skill['name'])}</h1>
    {badge}
  </div>
  <span class="tag skill-article__tag">{escape_html(skill.get('category_title', ''))}</span>
  <p class="lede">{escape_html(skill.get('description', ''))}</p>
</div>"""


def _render_prose(body_html: str) -> str:
    # body_html is pre-rendered, already-escaped HTML from the content
    # pipeline (h2/h3 with ids, p, ul/ol, table, strong, code, a) — injected
    # verbatim per contract. Do not run it through escape_html.
    return f'<div class="prose">{body_html}</div>'


def _render_tab_group(*, heading: str, items: list[dict]) -> str:
    if not items:
        return ""
    tabs: list[str] = []
    panels: list[str] = []
    for i, item in enumerate(items):
        item_id = escape_html(item["id"])
        tab_id = f"tab-{item_id}"
        selected = "true" if i == 0 else "false"
        tabindex = "0" if i == 0 else "-1"
        tabs.append(
            f'<button class="tabs__tab" type="button" role="tab" id="{tab_id}" '
            f'aria-controls="{item_id}" aria-selected="{selected}" tabindex="{tabindex}">'
            f'{escape_html(item["title"])}</button>'
        )
        hidden_attr = "" if i == 0 else " hidden"
        # item["html"] is pre-rendered, already-escaped HTML, same contract
        # as body_html above — injected verbatim.
        panels.append(
            f'<div class="tabs__panel" role="tabpanel" id="{item_id}" '
            f'aria-labelledby="{tab_id}" tabindex="0"{hidden_attr}>{item["html"]}</div>'
        )
    return f"""<section class="formidable-extra__group" aria-labelledby="{escape_html(heading.lower())}-heading">
  <h2 id="{escape_html(heading.lower())}-heading" class="formidable-extra__subtitle">{escape_html(heading)}</h2>
  <div class="tabs" data-tabs>
    <div class="tabs__list" role="tablist" aria-label="{escape_html(heading)}">
      {"".join(tabs)}
    </div>
    <div class="tabs__panels">
      {"".join(panels)}
    </div>
  </div>
</section>"""


def _render_craft_floor(skill: dict) -> str:
    # Not a tab: craft-floor.md is the quality bar, not a command, and links
    # inside formidable's own body/stack/command content already point at
    # #cmd-craft-floor (the id the content pipeline's link-rewriter produces
    # for any reference/*.md mention, on the same convention as the command
    # tabs) — this id must exist on the page or those links are dead.
    html = skill.get("formidable_craft_floor_html")
    if not html:
        return ""
    return f"""<section class="formidable-extra__group" id="cmd-craft-floor" aria-labelledby="craft-floor-heading">
  <h2 id="craft-floor-heading" class="formidable-extra__subtitle">Craft floor</h2>
  <div class="prose">{html}</div>
</section>"""


def _render_formidable_extras(skill: dict) -> str:
    if not skill.get("is_formidable"):
        return ""
    groups = _join(
        _render_tab_group(heading="Stacks", items=skill.get("formidable_stacks") or []),
        _render_tab_group(heading="Commands", items=skill.get("formidable_commands") or []),
        _render_craft_floor(skill),
    )
    if not groups:
        return ""
    return f'<div class="formidable-extra">{groups}</div>'


def _render_prevnext_link(skill: dict | None, *, direction: str, base_path: str = "") -> str:
    if skill is None:
        return ""
    label = "Previous" if direction == "prev" else "Next"
    modifier = " prevnext__link--next" if direction == "next" else ""
    name = escape_html(skill["name"])
    name_html = f"← {name}" if direction == "prev" else f"{name} →"
    return f"""<a class="prevnext__link{modifier}" href="{_skill_href(skill, base_path)}">
  <span class="prevnext__label">{label}</span>
  <span class="prevnext__name">{name_html}</span>
</a>"""


def _render_prevnext(prev_skill: dict | None, next_skill: dict | None,
                      base_path: str = "") -> str:
    if prev_skill is None and next_skill is None:
        return ""
    links = _join(
        _render_prevnext_link(prev_skill, direction="prev", base_path=base_path),
        _render_prevnext_link(next_skill, direction="next", base_path=base_path),
    )
    return f'<nav class="container prevnext" aria-label="Skill navigation">{links}</nav>'


def _render_see_also(skill: dict, siblings: list[dict], base_path: str = "") -> str:
    others = [s for s in siblings if s.get("slug") != skill.get("slug")]
    if not others:
        return ""
    items = _join(*(
        f"""<li><a class="see-also__link" href="{_skill_href(s, base_path)}">
      <span class="see-also__name">{escape_html(s['name'])}</span>
      <span class="see-also__summary">{escape_html(s.get('summary', ''))}</span>
    </a></li>"""
        for s in others
    ))
    return f"""<div class="container see-also">
  <h2 class="see-also__title">More in {escape_html(skill.get('category_title', ''))}</h2>
  <ul class="see-also__list">
{items}
  </ul>
</div>"""


def render_skill_page(skill: dict, *, prev_skill: dict | None, next_skill: dict | None,
                       siblings: list[dict], categories: list[dict],
                       base_path: str = "", last_updated_utc: str = "") -> str:
    """Full HTML document string for one skill's page (siblings = other skills
    in the same category, for a 'see also' list; categories = full category
    list, for nav)."""
    article = (
        '<article class="container skill-article">'
        + _render_skill_head(skill)
        + _render_prose(skill.get("body_html", ""))
        + _render_formidable_extras(skill)
        + "</article>"
    )
    main_html = _join(
        _render_breadcrumb(skill, base_path),
        article,
        _render_prevnext(prev_skill, next_skill, base_path),
        _render_see_also(skill, siblings, base_path),
    )
    name = skill.get("name", "")
    category_title = skill.get("category_title", "")
    summary = skill.get("summary") or skill.get("description", "")
    return _render_document(
        title=f"{name} — {category_title} — TBaguette’s Atelier",
        meta_description=summary,
        body_class="page-skill",
        main_html=main_html,
        categories=categories,
        base_path=base_path,
        last_updated_utc=last_updated_utc,
    )


# ---------------------------------------------------------------------------
# Install verification page
# ---------------------------------------------------------------------------

INSTALL_TEST_SOURCE_PATH = "scripts/test_install_command.py"
INSTALL_TEST_GITHUB_URL = (
    "https://github.com/LeSplooch/tbaguette-skills/blob/master/"
    + INSTALL_TEST_SOURCE_PATH
)


def _render_code_block(highlighted_lines: list[str]) -> str:
    rows = _join(*(
        f'<li><span class="code-block__line-number"></span>'
        f'<span class="code-block__line-code">{line or " "}</span></li>'
        for line in highlighted_lines
    ))
    return f"""<div class="code-block">
  <div class="code-block__scroll">
    <ol class="code-block__lines">
{rows}
    </ol>
  </div>
</div>"""


def render_verify_install_page(highlighted_lines: list[str], categories: list[dict],
                                base_path: str = "", last_updated_utc: str = "") -> str:
    """Full HTML document for the page linked from the install frame's
    "See how" — the actual explanation plus the actual test source,
    syntax-highlighted. highlighted_lines is pre-rendered HTML per line
    (see python_highlight.py), injected verbatim, same verbatim-injection
    contract as skill body_html elsewhere in this module."""
    breadcrumb = f"""<nav class="container breadcrumb" aria-label="Breadcrumb">
  <a href="{base_path}/">Home</a>
  <span class="breadcrumb__sep" aria-hidden="true">/</span>
  <span class="breadcrumb__current" aria-current="page">Install verification</span>
</nav>"""

    article = f"""<article class="container skill-article">
  <div class="skill-article__head">
    <h1 class="skill-article__title">The install command only touches one folder</h1>
    <p class="lede">A <code>git clone</code> into <code>~/.claude/skills/TBaguette</code>
    sits right next to whatever else already lives under
    <code>~/.claude/skills/</code> — other skills, other plugins, things you
    already trust. It's a fair question whether installing this one could touch
    any of that. It can't. This page is why, and the actual test that proves it,
    not a claim restated in different words.</p>
  </div>

  <div class="prose">
    <h2 id="what-it-does">What the command does</h2>
    <p>The published command checks for an existing clone before doing anything:
    if <code>~/.claude/skills/TBaguette/.git</code> already exists, it runs
    <code>git pull</code> in place; otherwise it runs <code>git clone</code>. The
    first run installs. Every run after that updates. Neither branch ever
    reads or writes anywhere else.</p>

    <h2 id="why-it-cant-reach-anything-else">Why it can't reach anything else</h2>
    <p>Both branches are scoped to the single path
    <code>~/.claude/skills/TBaguette</code> by construction — a
    <code>git clone</code> or <code>git pull</code> targeting that path has no
    mechanism to write outside it. The one case worth naming explicitly is a
    real collision: something else already sitting at that exact path, that
    isn't a clone of this repo. <code>git clone</code> refuses outright when its
    target already exists and is non-empty — it does not merge, does not
    overwrite, does not ask. It fails loudly and leaves whatever was there
    exactly as it was. That refusal is git's own behavior, not something this
    project added on top.</p>

    <h2 id="four-scenarios">Four scenarios, not one</h2>
    <p>A single happy-path test would only prove the command works when
    nothing is in its way, which is the one case nobody actually worried
    about. The real test runs four scenarios, each against a throwaway
    <code>HOME</code> directory seeded with fake sibling skills and plugins
    alongside <code>TBaguette</code>, checksummed before and after:</p>
    <ul>
      <li><strong>A — fresh install.</strong> Nothing at that path yet. Confirms
      the clone succeeds and the siblings are untouched.</li>
      <li><strong>B — running it again.</strong> Continues from A. Confirms the
      second run updates in place (<code>git pull</code>) instead of erroring
      the way a bare <code>git clone</code> would, and that the siblings are
      still untouched.</li>
      <li><strong>C — an empty directory already named <code>TBaguette</code>.</strong>
      Confirms <code>git clone</code> is willing to use an empty directory
      that happens to already exist, and that siblings survive.</li>
      <li><strong>D — the real collision.</strong> A non-empty, non-git
      directory already sitting at that exact path, with its own unrelated
      content. Confirms the command refuses rather than merging into it —
      and that both the colliding directory's own content and every sibling
      skill survive the refusal.</li>
    </ul>
    <p>All twelve checks across those four scenarios pass before this site is
    ever deployed — plus twelve more that run the exact command string
    published above, word for word, through every POSIX-ish shell this
    project's own build machine has installed (bash, zsh, fish, and sh),
    rather than trusting that this page's Python reimplementation of it
    stayed faithful to what actually ships. <code>run_tests.py</code> runs
    all twenty-four alongside everything else, not as a separate manual step
    someone has to remember.</p>

    <h2 id="every-platform">Every platform, not just one</h2>
    <p>The install frame above shows two commands, not one: a POSIX command
    for macOS, Linux, and anyone on Windows already inside WSL or Git Bash,
    and a PowerShell command for everyone else on Windows — which is most
    Windows users, since PowerShell has been the default terminal since
    Windows 10. Both do the exact same thing, in the same order, for the
    same reason: check for an existing clone, <code>git pull</code> if it's
    there, <code>git clone</code> if it isn't.</p>
    <p>The POSIX command is the one machine-verified above, across four
    shells:</p>
    <pre class="prose-code-block"><code>{escape_html(INSTALL_COMMAND)}</code></pre>
    <p>The PowerShell command carries the exact same logic over to
    <code>Test-Path</code> and native cmdlets:</p>
    <pre class="prose-code-block"><code>{escape_html(INSTALL_COMMAND_POWERSHELL)}</code></pre>
    <p>That one isn't cross-checked by an automated test the way the POSIX
    command is — this project's own build has no PowerShell runtime to run
    it against — so its correctness rests on <code>Test-Path</code>,
    <code>git -C</code>, and <code>git clone</code> being ordinary,
    well-documented behavior, not on an executed proof. If that's not good
    enough, that's a fair position to hold; the honest answer is "verified
    by careful construction," not "verified," for this one specifically.</p>
    <p>And for Command Prompt, which nothing in the install frame targets
    but which still exists on every Windows machine:</p>
    <pre class="prose-code-block"><code>{escape_html(INSTALL_COMMAND_CMD)}</code></pre>
    <p>Same logic again, translated to <code>IF EXIST</code> and
    <code>%USERPROFILE%</code> — with one honest imprecision.
    <code>IF EXIST</code> can't cleanly test "is this specifically a
    directory" in one line the way <code>-d</code> and
    <code>-PathType Container</code> can, so it matches a file or a
    directory at that path indiscriminately. That's never actually wrong
    here, since a real git checkout's <code>.git</code> is always a
    directory, but it's a looser guarantee than the other two, worth naming
    rather than glossing over.</p>

    <h2 id="the-test-itself">The test itself</h2>
    <p>This is <a href="{INSTALL_TEST_GITHUB_URL}">{INSTALL_TEST_SOURCE_PATH}</a>,
    unedited — the same file the test suite actually runs, not a
    representative excerpt.</p>
  </div>

  {_render_code_block(highlighted_lines)}

  <div class="prose">
    <h2 id="run-it-yourself">Run it yourself</h2>
    <p>Clone the repo and run <code>python3 scripts/test_install_command.py</code>
    directly, or <code>python3 scripts/run_tests.py</code> for the full suite
    this page's claims are checked against. Standard-library Python only —
    no bash required, since it isn't guaranteed to exist on every system this
    might run on.</p>
  </div>
</article>"""

    main_html = _join(breadcrumb, article)
    return _render_document(
        title="The install command only touches one folder — TBaguette’s Atelier",
        meta_description=(
            "How the TBaguette install command is verified never to alter, "
            "overwrite, or merge into any other skill or plugin you already "
            "have — with the actual test source, not just a claim."
        ),
        body_class="page-skill",
        main_html=main_html,
        categories=categories,
        base_path=base_path,
        last_updated_utc=last_updated_utc,
    )
