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

import locales
from dataclasses import dataclass


@dataclass(frozen=True)
class Strings:
    """Every piece of user-facing chrome text this module used to inline as
    English literals. One frozen dataclass rather than a raw dict so a
    missing/extra key is a Python-level error immediately, not a silent
    KeyError deep in an f-string. (Before the 2026-08-23 i18n revert, this
    exact key set was also what a translated locale's ui.json was checked
    against; that content and its check are gone along with the rest of
    i18n/, but the dataclass's own error-immediacy benefit for the English
    default stands regardless.)

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
    header_updated_value_template: str
    theme_toggle_switch_to_dark: str
    theme_toggle_switch_to_light: str
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
    search_empty_query_open: str
    search_empty_query_close: str
    sentence_end: str
    category_count_singular: str
    category_count_plural: str
    install_frame_label_template: str
    install_note_verified_text: str
    install_note_see_how: str
    install_note_restart_html_template: str
    install_note_session_html_template: str
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
    # {local} and {utc} are filled in by site.js, not by str.format here --
    # only the browser knows the visitor's timezone. One template rather
    # than two glue literals ("… your time · " and " UTC") so a locale can
    # reorder around both values: zh/ja/ko idiomatically lead with the
    # label ("当地时间 {local}"), which no amount of translating the glue
    # in place could express.
    header_updated_value_template="{local} your time · {utc} UTC",
    theme_toggle_switch_to_dark="Switch to dark theme",
    theme_toggle_switch_to_light="Switch to light theme",
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
    search_empty_query_open="“",
    search_empty_query_close="”",
    sentence_end=".",
    category_count_singular="skill",
    category_count_plural="skills",
    install_frame_label_template="Install {brand}’s skills",
    install_note_verified_text=(
        "Only ever touches this folder — verified against your other skills, not "
        "just claimed."
    ),
    install_note_see_how="See how",
    install_note_restart_html_template=(
        "Restart your agent (in Claude Code, <code>/reload-plugins</code>), then "
        "invoke a skill as <code>{brand}:skill-name</code>. On Claude that means "
        "Claude Code specifically — the Claude Desktop app and claude.ai chat "
        "load skills from your account instead of this folder, so cloning here "
        "won’t make them appear there."
    ),
    install_note_session_html_template=(
        "A conversation that was already open when you installed or updated — "
        "including the one you installed from — is still running on the skill "
        "list it started with. Open a new conversation to pick up the latest, or "
        "invoke <code>{brand}:using-tbaguette</code> in the old one."
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
        "The install frame above shows a prompt addressed to Claude, not a "
        "command for you to type — paste it into a Claude Code conversation "
        "and Claude detects your OS and runs the matching command itself. "
        "That command, on every platform, is exactly what follows: a POSIX "
        "command for macOS, Linux, and anyone on Windows already inside WSL "
        "or Git Bash, and a PowerShell command for everyone else on Windows "
        "— which is most Windows users, since PowerShell has been the "
        "default terminal since Windows 10. All three do the exact same "
        "thing, in the same order, for the same reason: check for an "
        "existing clone, <code>git pull</code> if it's there, "
        "<code>git clone</code> if it isn't."
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

# Most tiles the "Fresh from the oven" rail will show at once. See
# _render_fresh_section() for why it is capped rather than exhaustive.
FRESH_RAIL_LIMIT = 12

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
    # data-i18n-updated-template carries the localized glue around the two
    # times site.js computes; it lives on the <time> element rather than on
    # <body> because that's the only element that consumes it, and the two
    # attributes it sits beside are already scoped the same way.
    fallback = escape_html(last_updated_utc.replace("+00:00", "Z")) + " UTC"
    return f"""<p class="site-header__updated">
      <span class="site-header__updated-label">{escape_html(strings.header_updated_label)}</span>
      <time class="site-header__updated-value" datetime="{escape_html(last_updated_utc)}" data-format-updated data-version-url="{base_path}/version.txt" data-i18n-updated-template="{escape_html(strings.header_updated_value_template)}">{fallback}</time>
    </p>"""


def _render_plugin_version(version: str) -> str:
    # A version string is locale-independent and always Latin digits, so it
    # carries dir="ltr" explicitly -- without it the RTL locales reorder
    # "v0.10.0" around the dot. Not a link and deliberately outside the
    # <a class="wordmark">: it is metadata about the product rather than part
    # of the title, and clicking a version number should not navigate home.
    return f'<span class="wordmark-version" dir="ltr">v{escape_html(version)}</span>'


def _render_header(base_path: str = "", last_updated_utc: str = "",
                    *, locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                    path_suffix: str = "", strings: Strings = ENGLISH_STRINGS,
                    plugin_version: str = "") -> str:
    updated_html = _render_updated_time(last_updated_utc, base_path, strings) if last_updated_utc else ""
    version_html = _render_plugin_version(plugin_version) if plugin_version else ""
    return f"""<header class="site-header">
  <div class="site-header__band" aria-hidden="true"></div>
  <div class="container site-header__inner">
    <div class="site-header__brand">
      <a class="wordmark" href="{_locale_url(locale, base_path, '')}">
        {_icon("icon-wheat", css_class="icon wordmark__icon", base_path=base_path)}
        <span class="wordmark__text">TBaguette<span class="wordmark__suffix">&rsquo;s Atelier</span></span>
      </a>
      {version_html}
    </div>
    <div class="site-header__actions">
      {updated_html}
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
                      path_suffix: str = "", strings: Strings = ENGLISH_STRINGS,
                      plugin_version: str = "") -> str:
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
      data-i18n-modal-reload="{escape_html(strings.update_modal_reload_button)}"
      data-i18n-quote-open="{escape_html(strings.search_empty_query_open)}"
      data-i18n-quote-close="{escape_html(strings.search_empty_query_close)}">
<a class="skip-link" href="#main">{escape_html(strings.skip_link)}</a>
{_render_header(base_path, last_updated_utc, locale=locale, path_suffix=path_suffix, strings=strings, plugin_version=plugin_version)}
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

# Addressed to whichever agent it's pasted into, not to a shell — this is
# what a visitor hands their coding agent instead of running a command
# themselves. It is deliberately harness-adaptive rather than Claude
# Code-only: TBaguette ships manifests for ten harnesses (see PORTING.md),
# and a prompt that assumed Claude Code told everyone else, wrongly, that
# they weren't supported.
#
# The design rule behind the four routes: *state what this repo ships, let
# the agent supply what it knows about itself.* Only Route A's path is a
# fact this project can verify (test_install_command.py proves it against
# four scenarios); the install directory for Codex, Cursor, or Pi is not
# something this file can assert without guessing, and a guessed path
# published on a website is a fabricated one. The agent reading this
# prompt *is* the harness in question and knows its own install mechanism,
# so Route D tells it to discover and confirm rather than inviting it to
# act on a path this prompt invented. "Don't invent a path ... clone
# nothing: tell me what you checked and ask" is the load-bearing sentence.
#
# Routes B and C exist because installs differ in kind, not just in path:
# some harnesses install from a git URL through their own command (and for
# slash commands like Kimi's `/plugins install`, only the human can type
# it, which the agent has to say rather than silently substitute a
# filesystem clone for), and some are a line in a config file the agent
# must not edit unasked — PORTING.md's rule 2, "never edit a user's
# personal files," applied to the install itself. Where this repo already
# documents an exact line for one of them, the prompt publishes that line
# rather than leaving an agent to reconstruct a package spec it would get
# subtly wrong; test_templates.py checks each one against the file that
# documents it, so the prompt is never the only place such a string lives.
#
# The "if you genuinely can't tell which one you are, ask" clause is load-
# bearing too, and was added after asking what an uncertain agent would do
# with this prompt: Route A is the concrete one, with four numbered cases,
# so it reads as the default. An agent on a harness that never looks at
# ~/.claude/skills would clone there, verify the clone, and report success
# — a wholly successful install of nothing.
#
# Branches 1-2 of Route A restate the exact clone-or-pull logic
# INSTALL_COMMAND already encodes and test_install_command.py already
# proves (scenarios A/B/C), just as prose instead of one shell's syntax,
# so it reads correctly no matter which shell the agent's tool actually
# runs. Branch 3 names a real state this project has hit before (a
# directory that already looks like a TBaguette install — CATALOG.md +
# skills/ — but has no .git of its own): worth a specific, non-alarming
# message rather than lumping it into branch 4's generic collision. Branch
# 4 is still the one place this prompt does more than the shell one-liner:
# a bare `git clone` just refuses on a real collision (scenario D) — told
# only "install this," an agent could read that refusal as a problem to
# solve and reach for rm -rf on its own initiative, so branches 3 and 4
# spell out "ask first, don't act" explicitly instead of leaving it
# implicit. The post-install checks go past "the files exist" to confirm
# the clone is actually a working repo and to report back which version
# landed — "verified, not just claimed" applied to what the agent tells
# the visitor, not only to what the shell command itself can't reach.
#
# INSTALL_PROMPT and INSTALL_PROMPT_HINT are deliberately English-only,
# like INSTALL_COMMAND/INSTALL_COMMAND_POWERSHELL/INSTALL_COMMAND_CMD
# above, unlike the rest of this page: the prompt is addressed to an AI
# agent, not to the human visitor, who only ever copy-pastes it verbatim
# rather than reads it closely — an agent follows the original English at
# least as reliably as any translation, and a translation only adds a
# chance of introducing an ambiguity the original doesn't have, for no
# reader who benefits from it. install_copy_aria_label is reused rather
# than given its own new field (it still literally says "install command",
# a small mismatch now that this button copies a prompt instead) to avoid
# a schema change needing a translated value from all 12 shipped locales
# for a single aria-label — worth revisiting in a dedicated pass, not here.
#
# The hint still rules out Claude Desktop by name. A real visitor asked
# where in Claude Desktop to paste this — "a Claude Code conversation"
# read as a location, not as an exclusion of the other product. Claude
# Desktop has no shell tool by default and doesn't read ~/.claude/skills/
# at all (see README's Install section), so pasting the prompt there can't
# do anything useful; worth ruling out here rather than only in the README
# a visitor may never open. Naming Claude Sonnet (Max) sets an expectation
# for what this has actually been exercised against, rather than leaving
# model choice to a visitor with no way to know it matters — a
# recommendation, not a claim that nothing else works.
INSTALL_PROMPT_HINT = "Paste into whichever coding agent you use — it works out which harness it's in and installs the right way for it. It needs a shell tool, which rules out Claude Desktop; on Claude Code it's been exercised against Claude Sonnet (Max)."
INSTALL_PROMPT = """Install (or update) TBaguette — a skills library for coding agents — into whichever agent you are. Use your shell tool.

Before you start: confirm git is available (git --version). If it isn't, tell me and stop — there's nothing else to try.

TBaguette is a single git repo, https://github.com/LeSplooch/tbaguette-skills.git, shipping an integration for each harness it supports (Claude Code, Codex, Cursor, Copilot CLI, Devin, Gemini CLI, Hermes, Kimi Code, OpenCode, Pi). Work out which one you are running in, then take the matching route — you know your own install mechanism better than this prompt does. If you genuinely can't tell which one you are, say so and ask me, rather than falling through to the first route below.

Route A — you read Claude Code's skills directory. Target: ~/.claude/skills/TBaguette (Windows: %USERPROFILE%\\.claude\\skills\\TBaguette). Figure out which case applies:

1. <target>/.git exists — update in place: git -C <target> pull.
2. <target> doesn't exist, or exists and is empty — install fresh:
   git clone https://github.com/LeSplooch/tbaguette-skills.git <target>.
3. <target> exists, has content, isn't a git repo, but contains CATALOG.md and skills/ — this is very likely a previous TBaguette install that lost its own git history. Say that plainly, not a "naming collision", and ask me whether to move it aside and re-clone, rather than doing that yourself.
4. Anything else already at that path — stop. Do not delete or modify it. Tell me there's a naming collision that needs a manual look.

Route B — you install plugins or extensions from a git URL with your own command. Use it, against this same repo: Hermes is `hermes plugins install LeSplooch/tbaguette-skills`, Kimi Code is `/plugins install https://github.com/LeSplooch/tbaguette-skills`, Gemini CLI takes it as an extension. If it's a command only I can type, print me the exact line instead of substituting a filesystem clone for it.

Route C — you load plugins from a config file. Show me the exact entry and ask before editing — never edit my config silently. On OpenCode that entry is "tbaguette-skills@git+https://github.com/LeSplooch/tbaguette-skills.git", added to the "plugin" array in opencode.json.

Route D — none of those. Don't invent a path. Find the directory your harness actually reads: its own dotdir in my home directory, or an existing skills/, plugins/, or extensions/ folder — confirmed to be there, not assumed. Clone-or-pull into a TBaguette-named directory inside it, following Route A's four cases. If nothing you find is clearly right, clone nothing: tell me what you checked and ask.

Whichever route: never delete, move, or overwrite anything to make room. If the clone or pull command itself fails (network, permissions, auth), show me the actual error rather than retrying blindly or guessing why.

After a successful install, verify rather than assume. For a clone you placed yourself:
- <target>/.git exists and git -C <target> rev-parse HEAD succeeds.
- <target>/CATALOG.md exists and <target>/skills/ is non-empty.
- Read <target>/.claude-plugin/plugin.json's "version" field, if present, so you can tell me which version I'm now on.
For a harness-managed install, verify it the way your harness reports installed plugins.

Then tell me what happened (installed fresh, updated, or already current), which version, and the reload step for my agent specifically — restarting it, Claude Code's /reload-plugins, Kimi Code's /new. Skills then invoke as TBaguette:skill-name.

Flag one more thing, because it applies to this very conversation: this session started before the install, so it is still running on the skill list it had at startup, and nothing you just did changes that. Tell me to open a new conversation to pick up the latest — or, if I want to stay in this one, to invoke TBaguette:using-tbaguette here (if you have no skill tool, read skills/using-tbaguette/SKILL.md from the install instead)."""


def _render_install(base_path: str = "", *,
                     locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                     strings: Strings = ENGLISH_STRINGS) -> str:
    escaped_prompt = escape_html(INSTALL_PROMPT)
    frame_label = strings.install_frame_label_template.format(brand=BRAND_NAME)
    restart_note_html = strings.install_note_restart_html_template.format(brand=BRAND_NAME)
    session_note_html = strings.install_note_session_html_template.format(brand=BRAND_NAME)
    return f"""<div class="install-frame">
  <p class="install-frame__label">
    {_icon("icon-crust", base_path=base_path)}
    {escape_html(frame_label)}
  </p>
  <div class="install-frame__body">
    <div class="install">
      <code class="install__command" id="install-prompt-command">{escaped_prompt}</code>
      <button class="install__copy" type="button" data-copy-target="install-prompt-command"
              aria-label="{escape_html(strings.install_copy_aria_label)}">
        <span class="install__copy-icons">
          <svg class="icon install__copy-icon install__copy-icon--copy" aria-hidden="true"><use href="{base_path}/assets/icons.svg#icon-copy"></use></svg>
          <svg class="icon install__copy-icon install__copy-icon--check" aria-hidden="true"><use href="{base_path}/assets/icons.svg#icon-check"></use></svg>
        </span>
        <span data-copy-label>{escape_html(strings.install_copy_label)}</span>
      </button>
    </div>
    <p class="install__hint">{escape_html(INSTALL_PROMPT_HINT)}</p>
    <p class="install-frame__note">
      {_icon("icon-check", base_path=base_path)}
      <span>{escape_html(strings.install_note_verified_text)} <a href="{_locale_url(locale, base_path, 'verify-install/')}">{escape_html(strings.install_note_see_how)}</a>{escape_html(strings.sentence_end)}</span>
    </p>
    <p class="install-frame__note">
      {_icon("icon-check", base_path=base_path)}
      <span>{restart_note_html}</span>
    </p>
    <p class="install-frame__note">
      {_icon("icon-check", base_path=base_path)}
      <span>{session_note_html}</span>
    </p>
  </div>
</div>"""


def _render_hero(skill_count: int, category_count: int, base_path: str = "", *,
                  fresh_skills: list[dict] | None = None,
                  locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                  strings: Strings = ENGLISH_STRINGS) -> str:
    lede = strings.hero_lede_template.format(skill_count=skill_count, category_count=category_count)
    return f"""<section class="hero">
  <div class="container">
    <h1 class="hero__headline">{escape_html(strings.hero_headline)}</h1>
    {_render_install(base_path, locale=locale, strings=strings)}
    <p class="hero__lede">{escape_html(lede)}</p>
    {_render_fresh_section(fresh_skills or [], base_path)}
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
  {escape_html(strings.search_no_match_prefix)} <span class="search__empty-query" data-search-empty-query></span>{escape_html(strings.sentence_end)}
  <button type="button" data-search-reset>{escape_html(strings.search_reset_button)}</button> {escape_html(suffix)}
</p>"""


def _render_change_badge(status: str | None, at: str | None = None,
                          strings: Strings = ENGLISH_STRINGS) -> str:
    """"New"/"Updated" — set on a skill dict by generate.py from real git
    history, not guessed here. Empty string (renders as nothing) for every
    skill outside the freshness window, which is the common case once a
    repository has any age at all.

    data-fresh-at carries the instant the change landed so site.js can retire
    the badge in the browser. The site is static: without that, a page built
    one hour before the window closes would keep claiming "New" for as long
    as a visitor (or a CDN) held onto it."""
    if status not in ("new", "updated"):
        return ""
    label = strings.change_badge_new if status == "new" else strings.change_badge_updated
    stamp = f' data-fresh-at="{escape_html(at)}"' if at else ""
    return f'<span class="change-badge change-badge--{status}"{stamp}>{escape_html(label)}</span>'


def _fresh_signed_offset(index: int, count: int) -> int:
    """A tile's position relative to the rail's centre, wrapping the short
    way around rather than counting strictly left-to-right.

    Split evenly around zero — roughly the first half of `shown` lands at
    0, 1, 2, ... to the right, the second half at -1, -2, ... to the left —
    so the initial paint (before any JS ever runs, and forever for visitors
    without it) already reads as a centred fan rather than everything queued
    up on one side. site.js's coverflow step recomputes this exact formula
    at runtime as the active tile advances; keeping the two in sync is why
    it is a plain, restatable rule rather than something baked only here."""
    if count <= 1:
        return 0
    half = count // 2
    return index if index <= half else index - count


# One tile-step of horizontal travel, matching styles.css's own
# `var(--cf-offset) * 140px` and site.js's DRAG_STEP_PX. Restated here for
# the same reason _fresh_signed_offset is: there is no build step joining
# the three, so they are kept deliberately parallel.
FRESH_STEP_PX = 140


def _fresh_group_shift_px(count: int) -> int:
    """How far to slide the whole fan sideways so it sits centred in the
    rail, rather than merely having its *active* tile centred.

    Those are not the same thing whenever the offsets _fresh_signed_offset
    hands out are lopsided, which is most of the time. Four tiles land on
    -1, 0, 1, 2: one card left of centre and two to the right, so the fan
    reaches 295px to the left and 420px to the right and visibly hugs the
    rail's right edge with a gap opening on the left. Five tiles (-2..2)
    are symmetric and need no help; six put their sixth at offset 3, which
    is stacked invisibly behind the 2 and must not drag the centring with
    it -- hence clamping the range to the +-2 that is actually visible.

    Returned in px rather than in offsets so a fractional half-step lands
    as a real translate; the shift depends only on `count`, so it stays
    constant while the rail steps or is dragged, and the fan never jitters
    sideways mid-spin."""
    if count <= 1:
        return 0
    half = count // 2
    hi = min(half, 2)
    lo = max(-(count - 1 - half), -2)
    return round(-(lo + hi) / 2 * FRESH_STEP_PX)


def _render_fresh_tile(skill: dict, base_path: str = "", index: int = 0, count: int = 1) -> str:
    """One rail tile — the same card content as the grid below
    (_render_skill_card): name + badge, summary, category tag and arrow.
    It carries the "card" class alongside "fresh__tile" deliberately, so it
    inherits that look for free (background, padding, hover lift, the
    New/Updated badge treatment) instead of duplicating it; "fresh__tile" is
    only what adds the rail's own positioning on top.

    data-fresh-at sits on the tile itself, so site.js removing a stale
    element takes the whole tile rather than leaving a headless entry
    behind.

    --cf-offset is inert everywhere that doesn't reference it via var(), so
    this markup renders identically whether or not the coverflow rule in
    styles.css ends up applying — a reduced-motion, narrow, or forced-colors
    visitor gets the same tile, just laid out by the flat rail instead."""
    at = skill.get("change_at")
    stamp = f' data-fresh-at="{escape_html(at)}"' if at else ""
    # English label deliberately, same as this whole rail's title/tag/nav
    # text below (_render_fresh_section) -- this feature merged in from a
    # concurrent branch that predates this site's i18n work, and localizing
    # it needs its own dedicated pass across every already-shipped locale's
    # ui.json rather than a rushed field addition mid-merge.
    badge = _render_change_badge(skill.get("change_status"), at)
    offset = _fresh_signed_offset(index, count)
    # draggable="false": Firefox's native "drag this link" gesture would
    # otherwise compete with site.js's own click-and-drag-to-spin handling
    # for the same pointer gesture. Chrome/Safari take styles.css's
    # -webkit-user-drag: none for the same thing; Firefox doesn't honour
    # that property at all, but does honour the HTML attribute -- so both
    # are needed, neither is redundant.
    return f"""<a class="card fresh__tile" href="{_skill_href(skill, base_path)}"{stamp} draggable="false" style="--cf-offset: {offset}">
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


def _render_fresh_section(fresh_skills: list[dict], base_path: str = "") -> str:
    """The "Fresh from the oven" rail, above the search field.

    Renders nothing at all when nothing is fresh. An empty state here would be
    a section whose entire job is to say it has no job — worse than the space
    it would occupy, and the categories below already answer "what is there".
    Its tiles are the same cards the grid below renders (see
    _render_fresh_tile) — this rail is a preview of that grid, not a second,
    differently-designed one.

    Capped at FRESH_RAIL_LIMIT because a burst of activity (or, on a young
    repository, its whole history) can put every skill inside the window at
    once, and a rail listing everything is just the site again in miniature.
    The cap only bounds this rail: the badges themselves are exhaustive, so
    anything trimmed here is still marked in the grid below.

    Each tile carries its own --cf-offset (see _render_fresh_tile) — a plain
    custom property, not scoped to any media query itself, so a no-coverflow
    visitor (reduced motion, a narrow viewport, forced-colors, or no JS at
    all) gets exactly the flat scrolling rail this section has always
    rendered. The coverflow only forms where styles.css's gated rule opts in
    and reads it."""
    if not fresh_skills:
        return ""
    shown = fresh_skills[:FRESH_RAIL_LIMIT]
    count = len(shown)
    tiles = _join(*(
        _render_fresh_tile(skill, base_path, index=i, count=count)
        for i, skill in enumerate(shown)
    ))
    # hidden by default: only initFreshCoverflow() ever un-hides this, and
    # only once it has confirmed the exact same three conditions styles.css
    # gates the coverflow behind. A no-JS visitor, or one who fails any of
    # those conditions, never sees a control for a carousel that isn't
    # turning — one tile can't be stepped through at all, so it's skipped
    # even then.
    nav = f"""<div class="fresh__nav" data-fresh-nav hidden>
    <button type="button" class="fresh__nav-btn" data-fresh-prev aria-label="Show previous skill">&lsaquo;</button>
    <button type="button" class="fresh__nav-btn" data-fresh-next aria-label="Show next skill">&rsaquo;</button>
  </div>""" if count > 1 else ""
    return f"""<section class="fresh" aria-labelledby="fresh-title" data-fresh-section>
  <div class="fresh__head">
    {_icon("icon-grain", css_class="icon fresh__icon", base_path=base_path)}
    <h2 class="fresh__title" id="fresh-title">Fresh from the oven</h2>
    <span class="tag fresh__tag">Last 48 hours</span>
    {nav}
  </div>
  <div class="fresh__rail" data-fresh-coverflow style="--cf-shift: {_fresh_group_shift_px(count)}px">
{tiles}
  </div>
</section>"""


def _render_skill_card(skill: dict, base_path: str = "",
                        locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                        strings: Strings = ENGLISH_STRINGS) -> str:
    badge = _render_change_badge(skill.get("change_status"), skill.get("change_at"), strings)
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
                  fresh_skills: list[dict] | None = None,
                  locale: "locales.Locale" = locales.DEFAULT_LOCALE,
                  strings: Strings = ENGLISH_STRINGS,
                  plugin_version: str = "") -> str:
    """Full HTML document string for the landing page. fresh_skills arrives
    already ordered newest-first from generate.py, which is the only place
    that can know a change's real timestamp."""
    sections = _join(*(
        _render_category_section(cat, skills, i, base_path, locale, strings)
        for i, cat in enumerate(categories)
    ))
    skill_count = len(skills)
    category_count = len(categories)
    main_html = _join(
        _render_hero(skill_count, category_count, base_path,
                     fresh_skills=fresh_skills, locale=locale, strings=strings),
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
        plugin_version=plugin_version,
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
    badge = _render_change_badge(skill.get("change_status"), skill.get("change_at"), strings)
    return f"""<div class="skill-article__head">
  <div class="skill-article__title-row">
    <h1 class="skill-article__title">{escape_html(skill['name'])}</h1>
    {badge}
  </div>
  <span class="tag skill-article__tag">{escape_html(skill.get('category_title', ''))}</span>
  <p class="lede">{escape_html(skill.get('description', ''))}</p>
</div>"""


def _render_prose(body_html: str, *, lang: str | None = None,
                   text_dir: str | None = None) -> str:
    # body_html is pre-rendered, already-escaped HTML from the content
    # pipeline — injected verbatim per contract, same as elsewhere in this
    # module. lang and text_dir, when given, override the ambient page
    # language *and text direction* for this one block (used when the body is
    # an untranslated English fallback on a non-English page — see
    # _render_translation_fallback_banner).
    #
    # Both are needed, and text_dir is not inferable from lang: HTML derives
    # text direction only from an explicit dir attribute, never from lang. On
    # an RTL page the fallback body would otherwise inherit <html dir="rtl">
    # and lay out English as an RTL paragraph — every trailing colon, period
    # and parenthesis jumps to the start of its visual line ("Core
    # principles:" renders with the colon leading). Since all 66 skill bodies
    # are still English-only, that was every /ar/skills/* page.
    #
    # text_dir is its own parameter rather than being implied by lang's
    # presence so that a future fallback in some other language carries that
    # language's own direction instead of a hardcoded "ltr".
    return f'<div class="prose"{_fallback_attrs(lang, text_dir)}>{body_html}</div>'


def _fallback_attrs(lang: str | None, text_dir: str | None) -> str:
    """The lang/dir pair marking a block as untranslated fallback content.

    Shared by every container that can hold a fallback body: the main prose
    block, formidable's per-tab panels, and its craft-floor section. They all
    need it for the same reason and must not drift apart -- the craft floor
    and the tab panels are English too, and were laying out under dir="rtl"
    on /ar/skills/formidable/ (21 of 24 elements in the craft floor, 20 of 31
    in the first tab panel) even after the main body was fixed.
    """
    attrs = ""
    if lang:
        attrs += f' lang="{escape_html(lang)}"'
    if text_dir:
        attrs += f' dir="{escape_html(text_dir)}"'
    return attrs


def _render_translation_fallback_banner(locale: "locales.Locale", strings: Strings) -> str:
    message = strings.translation_fallback_banner_template.format(language=locale.endonym)
    return f'<p class="translation-banner" role="note">{escape_html(message)}</p>'


def _render_tab_group(*, group_id: str, heading: str, items: list[dict],
                       lang: str | None = None, text_dir: str | None = None) -> str:
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
        # as body_html above — injected verbatim. The lang/dir pair goes on
        # the panel, not on the group, because the group's heading comes from
        # Strings and really is translated — only the panel body is fallback.
        panels.append(
            f'<div class="tabs__panel" role="tabpanel" id="{item_id}" '
            f'aria-labelledby="{tab_id}" tabindex="0"{hidden_attr}'
            f'{_fallback_attrs(lang, text_dir)}>{item["html"]}</div>'
        )
    # heading_id comes from the caller's stable, translation-independent
    # group_id -- never from the heading text. Two earlier attempts derived
    # it from the heading and both broke on real translated content:
    # heading.lower() left a literal space in the id for any multi-word
    # heading (invalid HTML, and aria-labelledby is a space-separated id
    # list, so one space-containing id misparses as two nonexistent ones),
    # and content_pipeline.slugify() -- whose [^a-z0-9]+ strip is ASCII-only
    # -- collapsed an entirely non-Latin heading to the empty string and
    # fell through to its literal "section" fallback. Since both of
    # formidable's groups do that in zh/ja/ko/ru/hi/ar, BOTH landed on
    # id="section-heading" on the same page: a duplicate id, with the
    # Commands panel's aria-labelledby resolving to the Stacks heading.
    # An id is a machine identifier, so it gets a machine-chosen name;
    # heading stays display-only. group_id is caller-supplied plain ASCII
    # (see _render_formidable_extras), so it needs no escaping or slugging.
    heading_id = f"{group_id}-heading"
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


def _render_craft_floor(skill: dict, strings: Strings = ENGLISH_STRINGS, *,
                         lang: str | None = None, text_dir: str | None = None) -> str:
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
  <div class="prose"{_fallback_attrs(lang, text_dir)}>{html}</div>
</section>"""


def _render_formidable_extras(skill: dict, strings: Strings = ENGLISH_STRINGS, *,
                               lang: str | None = None, text_dir: str | None = None) -> str:
    if not skill.get("is_formidable"):
        return ""
    groups = _join(
        # group_id values are stable English keys, deliberately identical
        # across every locale: they are HTML ids and anchor targets, not
        # display text. Their headings are translated; these are not.
        _render_tab_group(group_id="stacks",
                          heading=strings.formidable_stacks_heading,
                          items=skill.get("formidable_stacks") or [],
                          lang=lang, text_dir=text_dir),
        _render_tab_group(group_id="commands",
                          heading=strings.formidable_commands_heading,
                          items=skill.get("formidable_commands") or [],
                          lang=lang, text_dir=text_dir),
        _render_craft_floor(skill, strings, lang=lang, text_dir=text_dir),
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
    # ← (U+2190) and → (U+2192) do NOT carry the Unicode Bidi_Mirrored
    # property, so — unlike brackets and parentheses — the bidi algorithm
    # never flips them for an RTL run. The glyph has to be swapped here.
    #
    # Only the glyph, not the position: bidi already places the arrow on the
    # correct side unaided (measured on /ar/, "{name} →" renders with the
    # arrow to the *left* of the name, which is where "forward" belongs when
    # reading right-to-left). What is wrong is purely which way it points —
    # an RTL reader advances leftward, so "next" must point ← and "previous"
    # must point →, the mirror of the LTR pairing. Keeping each arrow in its
    # existing logical slot preserves that already-correct placement.
    #
    # Swapping the character rather than mirroring it with CSS
    # transform: scaleX(-1) — the approach used for .card__arrow — because
    # transform does not apply to non-replaced inline elements at all
    # (verified in-browser: translateX(50px) on an inline span moves it 0px).
    # .card__arrow only works that way because .card__foot is a flex
    # container, which blockifies it; .prevnext__name is a plain block, so
    # the same rule here would need a new span *plus* display: inline-block,
    # and would leave the DOM text saying the opposite of what is drawn.
    rtl = locale.dir == "rtl"
    back_arrow, forward_arrow = ("→", "←") if rtl else ("←", "→")
    name_html = f"{back_arrow} {name}" if direction == "prev" else f"{name} {forward_arrow}"
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
                       strings: Strings = ENGLISH_STRINGS,
                       plugin_version: str = "") -> str:
    """Full HTML document string for one skill's page (siblings = other skills
    in the same category, for a 'see also' list; categories = full category
    list, for nav)."""
    is_translated = skill.get("translated", True)
    banner = "" if is_translated else _render_translation_fallback_banner(locale, strings)
    # The untranslated fallback body is the default locale's own content, so
    # it carries that locale's language *and* direction — both read off
    # DEFAULT_LOCALE rather than hardcoded, so they cannot drift apart.
    fallback_locale = locales.DEFAULT_LOCALE
    body_lang = None if is_translated else fallback_locale.code
    body_dir = None if is_translated else fallback_locale.dir
    article = (
        '<article class="container skill-article">'
        + _render_skill_head(skill, strings)
        + banner
        + _render_prose(skill.get("body_html", ""), lang=body_lang, text_dir=body_dir)
        + _render_formidable_extras(skill, strings, lang=body_lang, text_dir=body_dir)
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
        plugin_version=plugin_version,
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
                                verify_strings: VerifyInstallStrings = ENGLISH_VERIFY_INSTALL_STRINGS,
                                plugin_version: str = "") -> str:
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
        plugin_version=plugin_version,
    )
