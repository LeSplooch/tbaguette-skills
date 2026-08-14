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

import content_pipeline
import locales
from dataclasses import dataclass


@dataclass(frozen=True)
class Strings:
    """Every piece of user-facing chrome text this module used to inline as
    English literals. One frozen dataclass rather than a raw dict so a
    missing/extra key is a Python-level error immediately, not a silent
    KeyError deep in an f-string -- and so i18n/<lang>/ui.json's exact key
    set can be checked against dataclasses.fields(Strings) (see
    test_i18n.py's ui.json parity check, Task 4).

    Every _render_* function that used to inline one of these strings now
    takes `strings: Strings = ENGLISH_STRINGS` -- defaulting to English
    keeps every existing call site (all of test_templates.py) passing
    unmodified; only a locale build passes an explicit non-English catalog.

    Templated fields use {name} placeholders (str.format, not
    concatenation) because word order around an interpolated count or name
    varies by language.
    """

    skip_link: str
    header_updated_label: str
    theme_toggle_switch_to_dark: str
    theme_toggle_switch_to_light: str
    language_switcher_label: str
    hero_headline: str
    hero_lede_template: str
    search_label: str
    search_placeholder: str
    search_clear: str
    search_reset_button: str
    search_no_match_prefix: str
    search_no_match_suffix_template: str
    search_status_showing_all_template: str
    search_status_no_match: str
    search_status_partial_template: str
    category_count_singular: str
    category_count_plural: str
    install_frame_label_template: str
    install_note_verified_text: str
    install_note_see_how: str
    install_note_restart_html_template: str
    install_tab_posix_label: str
    install_tab_windows_label: str
    install_tab_aria_label: str
    install_hint_posix: str
    install_hint_powershell: str
    install_copy_label: str
    install_copy_copied: str
    install_copy_aria_label: str
    footer_brand_html_template: str
    footer_categories_label: str
    breadcrumb_home: str
    breadcrumb_aria_label: str
    prevnext_previous: str
    prevnext_next: str
    prevnext_aria_label: str
    see_also_title_template: str
    formidable_stacks_heading: str
    formidable_commands_heading: str
    formidable_craft_floor_heading: str
    change_badge_new: str
    change_badge_updated: str
    index_title_suffix: str
    index_meta_description_template: str
    update_modal_title: str
    update_modal_body: str
    update_modal_reload_button: str
    translation_fallback_banner_template: str


ENGLISH_STRINGS = Strings(
    skip_link="Skip to content",
    header_updated_label="Updated",
    theme_toggle_switch_to_dark="Switch to dark theme",
    theme_toggle_switch_to_light="Switch to light theme",
    language_switcher_label="Language",
    hero_headline="An atelier for the way you build.",
    hero_lede_template=(
        "{skill_count} Claude Code skills for the craft between the ticket and the "
        "commit, across {category_count} categories — findable by name, browsable "
        "below, and written like something a colleague actually handed you. Grown "
        "out of my own projects over time, this collection is created and updated "
        "automatically — and often — by Claude Opus, always its latest version, "
        "as I code."
    ),
    search_label="Search skills",
    search_placeholder="Search by name, summary, or category…",
    search_clear="Clear",
    search_reset_button="Clear search",
    search_no_match_prefix="No skills match",
    search_no_match_suffix_template="to see all {skill_count} again.",
    search_status_showing_all_template="Showing all {count} skills.",
    search_status_no_match="No skills match.",
    search_status_partial_template="{shown} of {total} skills match.",
    category_count_singular="skill",
    category_count_plural="skills",
    install_frame_label_template="Install {brand}’s skills",
    install_note_verified_text=(
        "Only ever touches this folder — verified against your other skills, not "
        "just claimed."
    ),
    install_note_see_how="See how",
    install_note_restart_html_template=(
        "Restart Claude Code (or run <code>/reload-plugins</code>), then invoke a "
        "skill as <code>{brand}:skill-name</code>. This is for Claude Code "
        "specifically — the Claude Desktop app and claude.ai chat load skills "
        "from your account instead of this folder, so cloning here won’t make "
        "them appear there."
    ),
    install_tab_posix_label="macOS / Linux",
    install_tab_windows_label="Windows (PowerShell)",
    install_tab_aria_label="Choose your platform",
    install_hint_posix="Works in bash, zsh, or fish — including WSL and Git Bash on Windows.",
    install_hint_powershell="PowerShell 5.1 or 7+, the Windows default terminal since Windows 10.",
    install_copy_label="Copy",
    install_copy_copied="Copied!",
    install_copy_aria_label="Copy install command",
    footer_brand_html_template=(
        "<strong>{brand_bakery}</strong> is home to {brand_atelier} — the judgment "
        "calls that sit between the ticket and the commit, organized like a proper "
        "bench."
    ),
    footer_categories_label="Categories",
    breadcrumb_home="Home",
    breadcrumb_aria_label="Breadcrumb",
    prevnext_previous="Previous",
    prevnext_next="Next",
    prevnext_aria_label="Skill navigation",
    see_also_title_template="More in {category}",
    formidable_stacks_heading="Stacks",
    formidable_commands_heading="Commands",
    formidable_craft_floor_heading="Craft floor",
    change_badge_new="New",
    change_badge_updated="Updated",
    index_title_suffix="Claude Code skills, organized",
    index_meta_description_template=(
        "{skill_count} Claude Code skills for the craft between the ticket and "
        "the commit, organized into {category_count} categories and cross-linked "
        "for the moment you need one."
    ),
    update_modal_title="New version available",
    update_modal_body="This page has been updated. Reload to see the latest.",
    update_modal_reload_button="Reload",
    translation_fallback_banner_template=(
        "This page hasn’t been translated into {language} yet — showing the "
        "English version."
    ),
)


@dataclass(frozen=True)
class VerifyInstallStrings:
    """The verify-install page's ~980 words of prose. Kept as its own
    catalog, separate from Strings, because this content is specific to
    one page rather than site-wide chrome -- matches i18n/<lang>/
    verify-install.json being its own file per the design spec. Every
    *_html field is pre-formatted HTML (contains <code>/<strong> spans),
    injected verbatim per this module's existing trust-boundary
    convention. test_itself_template carries {test_github_url}/
    {test_source_path} placeholders rather than a literal URL, so a
    translator never has to retype (and risk mistyping) that link.
    Likewise, three fields that mention the brand by name outside of a
    literal file path -- four_scenarios_intro_html_template,
    scenario_c_html_template, meta_description_template -- carry a
    {brand} placeholder rather than embedding "TBaguette" directly,
    formatted with the same BRAND_NAME constant Task 3 introduced (see
    this plan's Global Constraints on brand identity). The several
    ~/.claude/skills/TBaguette *path* mentions elsewhere in this class
    (lede_html, what_it_does_html, why_html) are deliberately left as
    plain literals -- a full filesystem path is already exactly as
    untranslatable as the install command's own shell syntax, which the
    same constraint explicitly exempts; splitting a path apart to
    template-splice one path *segment* would add real complexity for a
    string no translator would ever mistake for prose."""

    title: str
    lede_html: str
    what_it_does_heading: str
    what_it_does_html: str
    why_heading: str
    why_html: str
    four_scenarios_heading: str
    four_scenarios_intro_html_template: str
    scenario_a_html: str
    scenario_b_html: str
    scenario_c_html_template: str
    scenario_d_html: str
    scenario_footer_html: str
    every_platform_heading: str
    every_platform_intro_html: str
    posix_intro: str
    powershell_intro_html: str
    powershell_caveat_html: str
    cmd_intro: str
    cmd_caveat_html: str
    test_itself_heading: str
    test_itself_template: str
    run_it_yourself_heading: str
    run_it_yourself_html: str
    breadcrumb_current: str
    meta_description_template: str


ENGLISH_VERIFY_INSTALL_STRINGS = VerifyInstallStrings(
    title="The install command only touches one folder",
    lede_html=(
        "A <code>git clone</code> into <code>~/.claude/skills/TBaguette</code> "
        "sits right next to whatever else already lives under "
        "<code>~/.claude/skills/</code> — other skills, other plugins, things you "
        "already trust. It's a fair question whether installing this one could touch "
        "any of that. It can't. This page is why, and the actual test that proves it, "
        "not a claim restated in different words."
    ),
    what_it_does_heading="What the command does",
    what_it_does_html=(
        "The published command checks for an existing clone before doing anything: "
        "if <code>~/.claude/skills/TBaguette/.git</code> already exists, it runs "
        "<code>git pull</code> in place; otherwise it runs <code>git clone</code>. The "
        "first run installs. Every run after that updates. Neither branch ever "
        "reads or writes anywhere else."
    ),
    why_heading="Why it can't reach anything else",
    why_html=(
        "Both branches are scoped to the single path "
        "<code>~/.claude/skills/TBaguette</code> by construction — a "
        "<code>git clone</code> or <code>git pull</code> targeting that path has no "
        "mechanism to write outside it. The one case worth naming explicitly is a "
        "real collision: something else already sitting at that exact path, that "
        "isn't a clone of this repo. <code>git clone</code> refuses outright when its "
        "target already exists and is non-empty — it does not merge, does not "
        "overwrite, does not ask. It fails loudly and leaves whatever was there "
        "exactly as it was. That refusal is git's own behavior, not something this "
        "project added on top."
    ),
    four_scenarios_heading="Four scenarios, not one",
    four_scenarios_intro_html_template=(
        "A single happy-path test would only prove the command works when "
        "nothing is in its way, which is the one case nobody actually worried "
        "about. The real test runs four scenarios, each against a throwaway "
        "<code>HOME</code> directory seeded with fake sibling skills and plugins "
        "alongside <code>{brand}</code>, checksummed before and after:"
    ),
    scenario_a_html=(
        "<strong>A — fresh install.</strong> Nothing at that path yet. Confirms "
        "the clone succeeds and the siblings are untouched."
    ),
    scenario_b_html=(
        "<strong>B — running it again.</strong> Continues from A. Confirms the "
        "second run updates in place (<code>git pull</code>) instead of erroring "
        "the way a bare <code>git clone</code> would, and that the siblings are "
        "still untouched."
    ),
    scenario_c_html_template=(
        "<strong>C — an empty directory already named <code>{brand}</code>.</strong> "
        "Confirms <code>git clone</code> is willing to use an empty directory "
        "that happens to already exist, and that siblings survive."
    ),
    scenario_d_html=(
        "<strong>D — the real collision.</strong> A non-empty, non-git "
        "directory already sitting at that exact path, with its own unrelated "
        "content. Confirms the command refuses rather than merging into it — "
        "and that both the colliding directory's own content and every sibling "
        "skill survive the refusal."
    ),
    scenario_footer_html=(
        "All twelve checks across those four scenarios pass before this site is "
        "ever deployed — plus twelve more that run the exact command string "
        "published above, word for word, through every POSIX-ish shell this "
        "project's own build machine has installed (bash, zsh, fish, and sh), "
        "rather than trusting that this page's Python reimplementation of it "
        "stayed faithful to what actually ships. <code>run_tests.py</code> runs "
        "all twenty-four alongside everything else, not as a separate manual step "
        "someone has to remember."
    ),
    every_platform_heading="Every platform, not just one",
    every_platform_intro_html=(
        "The install frame above shows two commands, not one: a POSIX command "
        "for macOS, Linux, and anyone on Windows already inside WSL or Git Bash, "
        "and a PowerShell command for everyone else on Windows — which is most "
        "Windows users, since PowerShell has been the default terminal since "
        "Windows 10. Both do the exact same thing, in the same order, for the "
        "same reason: check for an existing clone, <code>git pull</code> if it's "
        "there, <code>git clone</code> if it isn't."
    ),
    posix_intro="The POSIX command is the one machine-verified above, across four shells:",
    powershell_intro_html=(
        "The PowerShell command carries the exact same logic over to "
        "<code>Test-Path</code> and native cmdlets:"
    ),
    powershell_caveat_html=(
        "That one isn't cross-checked by an automated test the way the POSIX "
        "command is — this project's own build has no PowerShell runtime to run "
        "it against — so its correctness rests on <code>Test-Path</code>, "
        "<code>git -C</code>, and <code>git clone</code> being ordinary, "
        "well-documented behavior, not on an executed proof. If that's not good "
        "enough, that's a fair position to hold; the honest answer is \"verified "
        "by careful construction,\" not \"verified,\" for this one specifically."
    ),
    cmd_intro=(
        "And for Command Prompt, which nothing in the install frame targets "
        "but which still exists on every Windows machine:"
    ),
    cmd_caveat_html=(
        "Same logic again, translated to <code>IF EXIST</code> and "
        "<code>%USERPROFILE%</code> — with one honest imprecision. "
        "<code>IF EXIST</code> can't cleanly test \"is this specifically a "
        "directory\" in one line the way <code>-d</code> and "
        "<code>-PathType Container</code> can, so it matches a file or a "
        "directory at that path indiscriminately. That's never actually wrong "
        "here, since a real git checkout's <code>.git</code> is always a "
        "directory, but it's a looser guarantee than the other two, worth naming "
        "rather than glossing over."
    ),
    test_itself_heading="The test itself",
    test_itself_template=(
        'This is <a href="{test_github_url}">{test_source_path}</a>, '
        "unedited — the same file the test suite actually runs, not a "
        "representative excerpt."
    ),
    run_it_yourself_heading="Run it yourself",
    run_it_yourself_html=(
        "Clone the repo and run <code>python3 scripts/test_install_command.py</code> "
        "directly, or <code>python3 scripts/run_tests.py</code> for the full suite "
        "this page's claims are checked against. Standard-library Python only — "
        "no bash required, since it isn't guaranteed to exist on every system this "
        "might run on."
    ),
    breadcrumb_current="Install verification",
    meta_description_template=(
        "How the {brand} install command is verified never to alter, "
        "overwrite, or merge into any other skill or plugin you already "
        "have — with the actual test source, not just a claim."
    ),
)

# Brand identity -- stays literal in every locale (see this plan's Global
# Constraints). Every *_template Strings field that mentions the brand
# names splices one of these in via str.format() rather than embedding the
# name inside the translatable value itself -- the same reasoning as the
# count-interpolated templates above, just for a name that must never
# translate instead of a number that must always agree grammatically.
BRAND_NAME = "TBaguette"
BRAND_ATELIER = "TBaguette&rsquo;s Atelier"
BRAND_BAKERY = "La Boulangerie TBaguette"


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


def _skill_href(skill: dict, base_path: str = "", locale: "locales.Locale" = locales.DEFAULT_LOCALE) -> str:
    return _locale_url(locale, base_path, f"skills/{escape_html(skill['slug'])}/")


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


def _locale_url(locale: "locales.Locale", base_path: str, path_suffix: str) -> str:
    """The one place "English lives at the root, every other locale lives
    under /<code>/" is decided. path_suffix is the part after the locale
    prefix — "" for the index, "skills/<slug>/" for a skill page,
    "verify-install/" for that page — and never itself starts or ends
    with a redundant slash beyond what's shown in the f-strings below."""
    if locale.default:
        return f"{base_path}/{path_suffix}"
    return f"{base_path}/{locale.code}/{path_suffix}"


def _render_language_switcher(
    current_locale: "locales.Locale", base_path: str, path_suffix: str, strings: Strings
) -> str:
    items = _join(*(
        f'<li><a class="language-switcher__link" href="{_locale_url(loc, base_path, path_suffix)}"'
        + (' aria-current="true"' if loc.code == current_locale.code else "")
        + f'>{escape_html(loc.endonym)}</a></li>'
        for loc in locales.LOCALES
    ))
    return f"""<details class="language-switcher">
  <summary class="language-switcher__summary">{escape_html(strings.language_switcher_label)}: {escape_html(current_locale.endonym)}</summary>
  <ul class="language-switcher__list">
{items}
  </ul>
</details>"""


def _render_hreflang_block(base_path: str, path_suffix: str) -> str:
    alternates = _join(*(
        f'<link rel="alternate" hreflang="{escape_html(loc.hreflang)}" href="{_locale_url(loc, base_path, path_suffix)}">'
        for loc in locales.LOCALES
    ))
    x_default = _locale_url(locales.DEFAULT_LOCALE, base_path, path_suffix)
    return alternates + f'\n<link rel="alternate" hreflang="x-default" href="{x_default}">'


# ---------------------------------------------------------------------------
# Shared chrome: document shell, header, footer, theme toggle
# ---------------------------------------------------------------------------


def _render_head(*, title: str, meta_description: str, base_path: str = "",
                  locale: "locales.Locale" = locales.DEFAULT_LOCALE, path_suffix: str = "") -> str:
    desc = escape_html(meta_description)
    canonical = _locale_url(locale, base_path, path_suffix)
    return f"""<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>{escape_html(title)}</title>
<meta name="description" content="{desc}">
<meta property="og:type" content="website">
<meta property="og:title" content="{escape_html(title)}">
<meta property="og:description" content="{desc}">
<link rel="canonical" href="{canonical}">
{_render_hreflang_block(base_path, path_suffix)}
<link rel="icon" type="image/svg+xml" href="{base_path}/assets/favicon.svg">
<link rel="preload" href="{base_path}/assets/fonts/fraunces-variable.woff2" as="font" type="font/woff2" crossorigin>
<link rel="preload" href="{base_path}/assets/fonts/work-sans-variable.woff2" as="font" type="font/woff2" crossorigin>
<link rel="stylesheet" href="{base_path}/assets/styles.css">
<script>{_THEME_BOOTSTRAP_JS}</script>"""


def _render_updated_time(last_updated_utc: str, base_path: str = "",
                          strings: Strings = ENGLISH_STRINGS) -> str:
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
      <span class="site-header__updated-label">{escape_html(strings.header_updated_label)}</span>
      <time class="site-header__updated-value" datetime="{escape_html(last_updated_utc)}" data-format-updated data-version-url="{base_path}/version.txt">{fallback}</time>
    </p>"""


def _render_header(base_path: str = "", last_updated_utc: str = "",
                    *, locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                    path_suffix: str = "", strings: Strings = ENGLISH_STRINGS) -> str:
    updated_html = _render_updated_time(last_updated_utc, base_path, strings) if last_updated_utc else ""
    return f"""<header class="site-header">
  <div class="site-header__band" aria-hidden="true"></div>
  <div class="container site-header__inner">
    <a class="wordmark" href="{_locale_url(locale, base_path, '')}">
      {_icon("icon-wheat", css_class="icon wordmark__icon", base_path=base_path)}
      <span class="wordmark__text">TBaguette<span class="wordmark__suffix">&rsquo;s Atelier</span></span>
    </a>
    <div class="site-header__actions">
      {updated_html}
      {_render_language_switcher(locale, base_path, path_suffix, strings)}
      <button class="theme-toggle" type="button" data-theme-toggle
              aria-label="{escape_html(strings.theme_toggle_switch_to_light)}"
              data-i18n-theme-light="{escape_html(strings.theme_toggle_switch_to_light)}"
              data-i18n-theme-dark="{escape_html(strings.theme_toggle_switch_to_dark)}">
        {_icon("icon-sun", css_class="icon theme-toggle__icon theme-toggle__icon--sun", base_path=base_path)}
        {_icon("icon-moon", css_class="icon theme-toggle__icon theme-toggle__icon--moon", base_path=base_path)}
      </button>
    </div>
  </div>
</header>"""


def _render_footer(categories: list[dict], base_path: str = "",
                    *, locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                    strings: Strings = ENGLISH_STRINGS) -> str:
    index_url = _locale_url(locale, base_path, "")
    links = _join(*(
        f'<li><a href="{index_url}#{escape_html(cat["slug"])}">{escape_html(cat["title"])}</a></li>'
        for cat in categories
    ))
    brand_html = strings.footer_brand_html_template.format(
        brand_bakery=BRAND_BAKERY, brand_atelier=BRAND_ATELIER
    )
    return f"""<footer class="site-footer">
  <div class="container site-footer__inner">
    <p class="site-footer__brand">{brand_html}</p>
    <nav aria-label="{escape_html(strings.footer_categories_label)}">
      <p class="site-footer__nav-title">{escape_html(strings.footer_categories_label)}</p>
      <ul class="site-footer__categories">
{links}
      </ul>
    </nav>
  </div>
</footer>"""


def _render_document(*, title: str, meta_description: str, body_class: str,
                      main_html: str, categories: list[dict],
                      base_path: str = "", last_updated_utc: str = "",
                      locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                      path_suffix: str = "", strings: Strings = ENGLISH_STRINGS) -> str:
    return f"""<!doctype html>
<html lang="{escape_html(locale.hreflang)}" dir="{escape_html(locale.dir)}">
<head>
{_render_head(title=title, meta_description=meta_description, base_path=base_path, locale=locale, path_suffix=path_suffix)}
</head>
<body class="{body_class}"
      data-i18n-copied="{escape_html(strings.install_copy_copied)}"
      data-i18n-no-match="{escape_html(strings.search_status_no_match)}"
      data-i18n-modal-title="{escape_html(strings.update_modal_title)}"
      data-i18n-modal-body="{escape_html(strings.update_modal_body)}"
      data-i18n-modal-reload="{escape_html(strings.update_modal_reload_button)}">
<a class="skip-link" href="#main">{escape_html(strings.skip_link)}</a>
{_render_header(base_path, last_updated_utc, locale=locale, path_suffix=path_suffix, strings=strings)}
<main id="main">
{main_html}
</main>
{_render_footer(categories, base_path, locale=locale, strings=strings)}
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


def _render_install_panel(*, panel_id: str, tab_id: str, command: str, hint: str,
                           selected: bool, base_path: str = "", strings: Strings = ENGLISH_STRINGS) -> str:
    escaped = escape_html(command)
    hidden_attr = "" if selected else " hidden"
    return f"""<div class="tabs__panel install-tabs__panel" role="tabpanel" id="{panel_id}"
             aria-labelledby="{tab_id}" tabindex="0"{hidden_attr}>
          <div class="install">
            <code class="install__command" id="{panel_id}-command">{escaped}</code>
            <button class="install__copy" type="button" data-copy-target="{panel_id}-command"
                    aria-label="{escape_html(strings.install_copy_aria_label)}">
              <span class="install__copy-icons">
                <svg class="icon install__copy-icon install__copy-icon--copy" aria-hidden="true"><use href="{base_path}/assets/icons.svg#icon-copy"></use></svg>
                <svg class="icon install__copy-icon install__copy-icon--check" aria-hidden="true"><use href="{base_path}/assets/icons.svg#icon-check"></use></svg>
              </span>
              <span data-copy-label>{escape_html(strings.install_copy_label)}</span>
            </button>
          </div>
          <p class="install__hint">{escape_html(hint)}</p>
        </div>"""


def _render_install(base_path: str = "", *,
                     locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                     strings: Strings = ENGLISH_STRINGS) -> str:
    posix_panel = _render_install_panel(
        panel_id="install-posix", tab_id="tab-install-posix",
        command=INSTALL_COMMAND, hint=strings.install_hint_posix,
        selected=True, base_path=base_path, strings=strings,
    )
    powershell_panel = _render_install_panel(
        panel_id="install-powershell", tab_id="tab-install-powershell",
        command=INSTALL_COMMAND_POWERSHELL, hint=strings.install_hint_powershell,
        selected=False, base_path=base_path, strings=strings,
    )
    frame_label = strings.install_frame_label_template.format(brand=BRAND_NAME)
    restart_note_html = strings.install_note_restart_html_template.format(brand=BRAND_NAME)
    return f"""<div class="install-frame">
  <p class="install-frame__label">
    {_icon("icon-crust", base_path=base_path)}
    {escape_html(frame_label)}
  </p>
  <div class="install-frame__body">
    <div class="tabs install-tabs" data-tabs data-autoselect-platform="true">
      <div class="tabs__list install-tabs__list" role="tablist" aria-label="{escape_html(strings.install_tab_aria_label)}">
        <button class="tabs__tab" type="button" role="tab" id="tab-install-posix"
                aria-controls="install-posix" aria-selected="true" tabindex="0"
                data-platform="posix">{escape_html(strings.install_tab_posix_label)}</button>
        <button class="tabs__tab" type="button" role="tab" id="tab-install-powershell"
                aria-controls="install-powershell" aria-selected="false" tabindex="-1"
                data-platform="windows">{escape_html(strings.install_tab_windows_label)}</button>
      </div>
      <div class="tabs__panels install-tabs__panels">
{posix_panel}
{powershell_panel}
      </div>
    </div>
    <p class="install-frame__note">
      {_icon("icon-check", base_path=base_path)}
      <span>{escape_html(strings.install_note_verified_text)} <a href="{_locale_url(locale, base_path, 'verify-install/')}">{escape_html(strings.install_note_see_how)}</a>.</span>
    </p>
    <p class="install-frame__note">
      {_icon("icon-check", base_path=base_path)}
      <span>{restart_note_html}</span>
    </p>
  </div>
</div>"""


def _render_hero(skill_count: int, category_count: int, base_path: str = "", *,
                  locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                  strings: Strings = ENGLISH_STRINGS) -> str:
    lede = strings.hero_lede_template.format(skill_count=skill_count, category_count=category_count)
    return f"""<section class="hero">
  <div class="container">
    <h1 class="hero__headline">{escape_html(strings.hero_headline)}</h1>
    {_render_install(base_path, locale=locale, strings=strings)}
    <p class="hero__lede">{escape_html(lede)}</p>
    {_render_search_field(base_path, strings)}
  </div>
</section>"""


def _render_search_field(base_path: str = "", strings: Strings = ENGLISH_STRINGS) -> str:
    return f"""<div class="search" data-search-root>
  <div class="search__field">
    <svg class="icon search__icon" aria-hidden="true"><use href="{base_path}/assets/icons.svg#icon-search"></use></svg>
    <label class="visually-hidden" for="skill-search">{escape_html(strings.search_label)}</label>
    <input class="search__input" type="search" id="skill-search" data-search-input
           placeholder="{escape_html(strings.search_placeholder)}" autocomplete="off">
    <button class="search__clear" type="button" data-search-clear hidden>{escape_html(strings.search_clear)}</button>
  </div>
  <p class="search__status" data-search-status aria-live="polite"
     data-i18n-showing-all-template="{escape_html(strings.search_status_showing_all_template)}"
     data-i18n-partial-template="{escape_html(strings.search_status_partial_template)}"></p>
</div>"""


def _render_search_empty_state(skill_count: int, strings: Strings = ENGLISH_STRINGS) -> str:
    suffix = strings.search_no_match_suffix_template.format(skill_count=skill_count)
    return f"""<p class="search__empty container" data-search-empty hidden>
  {escape_html(strings.search_no_match_prefix)} <span class="search__empty-query" data-search-empty-query></span>.
  <button type="button" data-search-reset>{escape_html(strings.search_reset_button)}</button> {escape_html(suffix)}
</p>"""


def _render_change_badge(status: str | None, strings: Strings = ENGLISH_STRINGS) -> str:
    """"New"/"Updated" — set on a skill dict by generate.py from a real git
    status check, not guessed here. Empty string (renders as nothing) for
    every skill not touched by the update currently being shipped, which is
    the common case — most page loads carry zero of these."""
    if status not in ("new", "updated"):
        return ""
    label = strings.change_badge_new if status == "new" else strings.change_badge_updated
    return f'<span class="change-badge change-badge--{status}">{escape_html(label)}</span>'


def _render_skill_card(skill: dict, base_path: str = "",
                        locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                        strings: Strings = ENGLISH_STRINGS) -> str:
    badge = _render_change_badge(skill.get("change_status"), strings)
    return f"""<a class="card" href="{_skill_href(skill, base_path, locale)}" data-search-card data-search-terms="{_search_haystack(skill)}">
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
                              base_path: str = "", locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                              strings: Strings = ENGLISH_STRINGS) -> str:
    slug = category["slug"]
    skill_slugs = category.get("skill_slugs", [])
    count = len(skill_slugs)
    icon_id = _CATEGORY_ICONS[icon_index % len(_CATEGORY_ICONS)]
    cards = _join(*(
        _render_skill_card(skills[s], base_path, locale, strings) for s in skill_slugs if s in skills
    ))
    noun = strings.category_count_singular if count == 1 else strings.category_count_plural
    return f"""<section class="category-section" id="{escape_html(slug)}" data-category-section>
  <div class="container">
    <div class="category-section__head">
      {_icon(icon_id, css_class="icon category-section__icon", base_path=base_path)}
      <h2 class="category-section__title">{escape_html(category['title'])}</h2>
      <span class="tag category-section__count" data-category-count="{count}"
            data-i18n-singular="{escape_html(strings.category_count_singular)}"
            data-i18n-plural="{escape_html(strings.category_count_plural)}">{count} {escape_html(noun)}</span>
    </div>
    <div class="card-grid" data-card-grid>
{cards}
    </div>
  </div>
</section>"""


def render_index(categories: list[dict], skills: dict, base_path: str = "",
                  last_updated_utc: str = "", *,
                  locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                  strings: Strings = ENGLISH_STRINGS) -> str:
    """Full HTML document string for the landing page."""
    sections = _join(*(
        _render_category_section(cat, skills, i, base_path, locale, strings)
        for i, cat in enumerate(categories)
    ))
    skill_count = len(skills)
    category_count = len(categories)
    main_html = _join(
        _render_hero(skill_count, category_count, base_path, locale=locale, strings=strings),
        _render_search_empty_state(skill_count, strings),
        f'<div data-categories>\n{sections}\n</div>',
    )
    title = f"TBaguette’s Atelier — {strings.index_title_suffix}"
    meta_description = strings.index_meta_description_template.format(
        skill_count=skill_count, category_count=category_count
    )
    return _render_document(
        title=title,
        meta_description=meta_description,
        body_class="page-index",
        main_html=main_html,
        categories=categories,
        base_path=base_path,
        last_updated_utc=last_updated_utc,
        locale=locale,
        path_suffix="",
        strings=strings,
    )


# ---------------------------------------------------------------------------
# Skill page
# ---------------------------------------------------------------------------


def _render_breadcrumb(skill: dict, base_path: str = "",
                        locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                        strings: Strings = ENGLISH_STRINGS) -> str:
    index_url = _locale_url(locale, base_path, "")
    return f"""<nav class="container breadcrumb" aria-label="{escape_html(strings.breadcrumb_aria_label)}">
  <a href="{index_url}">{escape_html(strings.breadcrumb_home)}</a>
  <span class="breadcrumb__sep" aria-hidden="true">/</span>
  <a href="{index_url}#{escape_html(skill.get('category_slug', ''))}">{escape_html(skill.get('category_title', ''))}</a>
  <span class="breadcrumb__sep" aria-hidden="true">/</span>
  <span class="breadcrumb__current" aria-current="page">{escape_html(skill['name'])}</span>
</nav>"""


def _render_skill_head(skill: dict, strings: Strings = ENGLISH_STRINGS) -> str:
    badge = _render_change_badge(skill.get("change_status"), strings)
    return f"""<div class="skill-article__head">
  <div class="skill-article__title-row">
    <h1 class="skill-article__title">{escape_html(skill['name'])}</h1>
    {badge}
  </div>
  <span class="tag skill-article__tag">{escape_html(skill.get('category_title', ''))}</span>
  <p class="lede">{escape_html(skill.get('description', ''))}</p>
</div>"""


def _render_prose(body_html: str, *, lang: str | None = None) -> str:
    # body_html is pre-rendered, already-escaped HTML from the content
    # pipeline — injected verbatim per contract, same as elsewhere in this
    # module. lang, when given, overrides the ambient page language for
    # this one block (used when the body is an untranslated English
    # fallback on a non-English page — see _render_translation_fallback_banner).
    lang_attr = f' lang="{escape_html(lang)}"' if lang else ""
    return f'<div class="prose"{lang_attr}>{body_html}</div>'


def _render_translation_fallback_banner(locale: "locales.Locale", strings: Strings) -> str:
    message = strings.translation_fallback_banner_template.format(language=locale.endonym)
    return f'<p class="translation-banner" role="note">{escape_html(message)}</p>'


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
    # heading_id via content_pipeline.slugify() rather than a bare
    # heading.lower(): .lower() only ever worked because English's and
    # French's Stacks/Commands headings happen to be single words --
    # a genuinely multi-word translated heading would leave a literal
    # space in the id, which is invalid HTML and breaks aria-labelledby
    # (a space-separated id list misparses one space-containing id as two
    # nonexistent ones). slugify() already produces an ASCII-safe,
    # hyphen-joined, no-space string, so it needs no escape_html() around
    # it, unlike the old .lower() call.
    heading_id = f"{content_pipeline.slugify(heading)}-heading"
    return f"""<section class="formidable-extra__group" aria-labelledby="{heading_id}">
  <h2 id="{heading_id}" class="formidable-extra__subtitle">{escape_html(heading)}</h2>
  <div class="tabs" data-tabs>
    <div class="tabs__list" role="tablist" aria-label="{escape_html(heading)}">
      {"".join(tabs)}
    </div>
    <div class="tabs__panels">
      {"".join(panels)}
    </div>
  </div>
</section>"""


def _render_craft_floor(skill: dict, strings: Strings = ENGLISH_STRINGS) -> str:
    # Not a tab: craft-floor.md is the quality bar, not a command, and links
    # inside formidable's own body/stack/command content already point at
    # #cmd-craft-floor (the id the content pipeline's link-rewriter produces
    # for any reference/*.md mention, on the same convention as the command
    # tabs) — this id must exist on the page or those links are dead.
    html = skill.get("formidable_craft_floor_html")
    if not html:
        return ""
    return f"""<section class="formidable-extra__group" id="cmd-craft-floor" aria-labelledby="craft-floor-heading">
  <h2 id="craft-floor-heading" class="formidable-extra__subtitle">{escape_html(strings.formidable_craft_floor_heading)}</h2>
  <div class="prose">{html}</div>
</section>"""


def _render_formidable_extras(skill: dict, strings: Strings = ENGLISH_STRINGS) -> str:
    if not skill.get("is_formidable"):
        return ""
    groups = _join(
        _render_tab_group(heading=strings.formidable_stacks_heading, items=skill.get("formidable_stacks") or []),
        _render_tab_group(heading=strings.formidable_commands_heading, items=skill.get("formidable_commands") or []),
        _render_craft_floor(skill, strings),
    )
    if not groups:
        return ""
    return f'<div class="formidable-extra">{groups}</div>'


def _render_prevnext_link(skill: dict | None, *, direction: str, base_path: str = "",
                           locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                           strings: Strings = ENGLISH_STRINGS) -> str:
    if skill is None:
        return ""
    label = strings.prevnext_previous if direction == "prev" else strings.prevnext_next
    modifier = " prevnext__link--next" if direction == "next" else ""
    name = escape_html(skill["name"])
    name_html = f"← {name}" if direction == "prev" else f"{name} →"
    return f"""<a class="prevnext__link{modifier}" href="{_skill_href(skill, base_path, locale)}">
  <span class="prevnext__label">{escape_html(label)}</span>
  <span class="prevnext__name">{name_html}</span>
</a>"""


def _render_prevnext(prev_skill: dict | None, next_skill: dict | None, base_path: str = "",
                      locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                      strings: Strings = ENGLISH_STRINGS) -> str:
    if prev_skill is None and next_skill is None:
        return ""
    links = _join(
        _render_prevnext_link(prev_skill, direction="prev", base_path=base_path, locale=locale, strings=strings),
        _render_prevnext_link(next_skill, direction="next", base_path=base_path, locale=locale, strings=strings),
    )
    return f'<nav class="container prevnext" aria-label="{escape_html(strings.prevnext_aria_label)}">{links}</nav>'


def _render_see_also(skill: dict, siblings: list[dict], base_path: str = "",
                      locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                      strings: Strings = ENGLISH_STRINGS) -> str:
    others = [s for s in siblings if s.get("slug") != skill.get("slug")]
    if not others:
        return ""
    items = _join(*(
        f"""<li><a class="see-also__link" href="{_skill_href(s, base_path, locale)}">
      <span class="see-also__name">{escape_html(s['name'])}</span>
      <span class="see-also__summary">{escape_html(s.get('summary', ''))}</span>
    </a></li>"""
        for s in others
    ))
    title = strings.see_also_title_template.format(category=skill.get('category_title', ''))
    return f"""<div class="container see-also">
  <h2 class="see-also__title">{escape_html(title)}</h2>
  <ul class="see-also__list">
{items}
  </ul>
</div>"""


def render_skill_page(skill: dict, *, prev_skill: dict | None, next_skill: dict | None,
                       siblings: list[dict], categories: list[dict],
                       base_path: str = "", last_updated_utc: str = "",
                       locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                       strings: Strings = ENGLISH_STRINGS) -> str:
    """Full HTML document string for one skill's page (siblings = other skills
    in the same category, for a 'see also' list; categories = full category
    list, for nav)."""
    is_translated = skill.get("translated", True)
    banner = "" if is_translated else _render_translation_fallback_banner(locale, strings)
    body_lang = None if is_translated else "en"
    article = (
        '<article class="container skill-article">'
        + _render_skill_head(skill, strings)
        + banner
        + _render_prose(skill.get("body_html", ""), lang=body_lang)
        + _render_formidable_extras(skill, strings)
        + "</article>"
    )
    main_html = _join(
        _render_breadcrumb(skill, base_path, locale, strings),
        article,
        _render_prevnext(prev_skill, next_skill, base_path, locale, strings),
        _render_see_also(skill, siblings, base_path, locale, strings),
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
        locale=locale,
        path_suffix=f"skills/{skill['slug']}/",
        strings=strings,
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
    return f"""<div class="code-block" dir="ltr">
  <div class="code-block__scroll">
    <ol class="code-block__lines">
{rows}
    </ol>
  </div>
</div>"""


def render_verify_install_page(highlighted_lines: list[str], categories: list[dict],
                                base_path: str = "", last_updated_utc: str = "",
                                *, locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                                strings: Strings = ENGLISH_STRINGS,
                                verify_strings: VerifyInstallStrings = ENGLISH_VERIFY_INSTALL_STRINGS) -> str:
    """Full HTML document for the page linked from the install frame's
    "See how" — the actual explanation plus the actual test source,
    syntax-highlighted. highlighted_lines is pre-rendered HTML per line
    (see python_highlight.py), injected verbatim, same verbatim-injection
    contract as skill body_html elsewhere in this module. The three
    literal install-command strings (POSIX/PowerShell/cmd.exe) are never
    translated — only verify_strings' surrounding prose is."""
    v = verify_strings
    breadcrumb = f"""<nav class="container breadcrumb" aria-label="{escape_html(strings.breadcrumb_aria_label)}">
  <a href="{_locale_url(locale, base_path, '')}">{escape_html(strings.breadcrumb_home)}</a>
  <span class="breadcrumb__sep" aria-hidden="true">/</span>
  <span class="breadcrumb__current" aria-current="page">{escape_html(v.breadcrumb_current)}</span>
</nav>"""

    test_itself_html = v.test_itself_template.format(
        test_github_url=INSTALL_TEST_GITHUB_URL, test_source_path=INSTALL_TEST_SOURCE_PATH
    )
    four_scenarios_intro_html = v.four_scenarios_intro_html_template.format(brand=BRAND_NAME)
    scenario_c_html = v.scenario_c_html_template.format(brand=BRAND_NAME)
    meta_description = v.meta_description_template.format(brand=BRAND_NAME)

    article = f"""<article class="container skill-article">
  <div class="skill-article__head">
    <h1 class="skill-article__title">{escape_html(v.title)}</h1>
    <p class="lede">{v.lede_html}</p>
  </div>

  <div class="prose">
    <h2 id="what-it-does">{escape_html(v.what_it_does_heading)}</h2>
    <p>{v.what_it_does_html}</p>

    <h2 id="why-it-cant-reach-anything-else">{escape_html(v.why_heading)}</h2>
    <p>{v.why_html}</p>

    <h2 id="four-scenarios">{escape_html(v.four_scenarios_heading)}</h2>
    <p>{four_scenarios_intro_html}</p>
    <ul>
      <li>{v.scenario_a_html}</li>
      <li>{v.scenario_b_html}</li>
      <li>{scenario_c_html}</li>
      <li>{v.scenario_d_html}</li>
    </ul>
    <p>{v.scenario_footer_html}</p>

    <h2 id="every-platform">{escape_html(v.every_platform_heading)}</h2>
    <p>{v.every_platform_intro_html}</p>
    <p>{escape_html(v.posix_intro)}</p>
    <pre class="prose-code-block" dir="ltr"><code>{escape_html(INSTALL_COMMAND)}</code></pre>
    <p>{v.powershell_intro_html}</p>
    <pre class="prose-code-block" dir="ltr"><code>{escape_html(INSTALL_COMMAND_POWERSHELL)}</code></pre>
    <p>{v.powershell_caveat_html}</p>
    <p>{escape_html(v.cmd_intro)}</p>
    <pre class="prose-code-block" dir="ltr"><code>{escape_html(INSTALL_COMMAND_CMD)}</code></pre>
    <p>{v.cmd_caveat_html}</p>

    <h2 id="the-test-itself">{escape_html(v.test_itself_heading)}</h2>
    <p>{test_itself_html}</p>
  </div>

  {_render_code_block(highlighted_lines)}

  <div class="prose">
    <h2 id="run-it-yourself">{escape_html(v.run_it_yourself_heading)}</h2>
    <p>{v.run_it_yourself_html}</p>
  </div>
</article>"""

    main_html = _join(breadcrumb, article)
    return _render_document(
        title=f"{v.title} — TBaguette’s Atelier",
        meta_description=meta_description,
        body_class="page-skill",
        main_html=main_html,
        categories=categories,
        base_path=base_path,
        last_updated_utc=last_updated_utc,
        locale=locale,
        path_suffix="verify-install/",
        strings=strings,
    )
