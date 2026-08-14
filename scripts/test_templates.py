"""Self-test for templates.py, run before any real content exists.

Renders the locked fixture from the design brief through both public
functions, writes the output to _preview/ so it can be opened in a browser,
and asserts a handful of sanity checks. Stdlib only.

Usage:
    python3 scripts/test_templates.py
"""

from pathlib import Path

from checker import Checker
from templates import (
    ENGLISH_STRINGS,
    INSTALL_COMMAND,
    INSTALL_COMMAND_CMD,
    INSTALL_COMMAND_POWERSHELL,
    INSTALL_TEST_GITHUB_URL,
    escape_html,
    render_index,
    render_skill_page,
    render_verify_install_page,
)

# ---------------------------------------------------------------------------
# The fixture from the original design brief. Only additive edits belong
# here (a new key that doesn't disturb an existing substring check) — this
# is why formidable_craft_floor_html was added directly below rather than
# given its own fixture. Anything that needs deliberately dangerous or
# structurally different input (special characters, a different base_path)
# gets its own small dedicated fixture instead, in check_escaping() and
# check_base_path() below, so a targeted test can't accidentally weaken an
# existing one.
# ---------------------------------------------------------------------------

FIXTURE = {
    "categories": [
        {"slug": "ui-and-design", "title": "UI and design", "skill_slugs": ["formidable"]},
        {"slug": "testing", "title": "Testing", "skill_slugs": ["designing-test-data", "flaky-test-triage"]},
    ],
    "skills": {
        "formidable": {
            "slug": "formidable", "name": "formidable",
            "category_slug": "ui-and-design", "category_title": "UI and design",
            "description": "Use when designing, redesigning, critiquing, auditing, polishing, or hardening any user interface on any stack.",
            "summary": "Design craft for every UI stack, not just web.",
            "body_html": "<h2 id=\"overview\">Overview</h2><p>Design that earns to be called <strong>out-of-distribution</strong> craft.</p>",
            "is_formidable": True,
            "formidable_craft_floor_html": "<p>Load immediately before editing UI.</p>",
            "formidable_stacks": [
                {"id": "stack-web", "title": "Web", "html": "<p>Effectively unlimited color, type, and motion.</p>"},
                {"id": "stack-terminal-tui", "title": "Terminal / TUI", "html": "<p>A grid of character cells.</p>"},
            ],
            "formidable_commands": [
                {"id": "cmd-shape", "title": "Shape", "html": "<p>Decide before you build.</p>"},
                {"id": "cmd-critique", "title": "Critique", "html": "<p>Design review with a verdict.</p>"},
            ],
        },
        "designing-test-data": {
            "slug": "designing-test-data", "name": "designing-test-data",
            "category_slug": "testing", "category_title": "Testing",
            "description": "Use when a test's setup is longer than its assertions, when fixtures are shared across files.",
            "summary": "Builders over shared fixtures; the one-obvious-difference rule.",
            "body_html": "<h2 id=\"overview\">Overview</h2><p>Build test data so the reason a test exists is visible in its setup.</p><table><thead><tr><th>Symptom</th><th>Real cause</th></tr></thead><tbody><tr><td>Order-dependent failures</td><td>Shared mutable fixtures</td></tr></tbody></table>",
            "is_formidable": False,
        },
        "flaky-test-triage": {
            "slug": "flaky-test-triage", "name": "flaky-test-triage",
            "category_slug": "testing", "category_title": "Testing",
            "description": "Use when a test passes on rerun, fails only in CI, fails only when the whole suite runs.",
            "summary": "The cause taxonomy; quarantine with an expiry.",
            "body_html": "<h2 id=\"overview\">Overview</h2><p>Treat flakiness as a defect report, not a nuisance.</p>",
            "is_formidable": False,
        },
    },
}

PREVIEW_DIR = Path(__file__).resolve().parent.parent / "_preview"

checker = Checker()
check = checker.check


def write_preview(filename: str, html: str) -> Path:
    PREVIEW_DIR.mkdir(parents=True, exist_ok=True)
    path = PREVIEW_DIR / filename
    path.write_text(html, encoding="utf-8")
    return path


def check_escaping() -> None:
    """The FIXTURE above deliberately contains no HTML-special characters in
    any plain-text field, so it cannot catch a regression where an escape_html
    call is removed — every substring check in main() would pass identically
    either way. This is a separate, dedicated fixture built specifically to
    prove escaping actually happens: it asserts both that the dangerous raw
    form is absent and that the escaped form is present, and that body_html's
    verbatim-injection contract (pre-rendered HTML is NOT re-escaped) still
    holds at the same time."""
    print("escaping regression check")
    categories = [{"slug": "cat", "title": "Cat", "skill_slugs": ["x"]}]
    skill = {
        "slug": "x",
        "name": 'A & B <em>"quoted"</em>',
        "category_slug": "cat",
        "category_title": "Cat",
        "description": "Uses <script>alert(1)</script> & \"quotes\" & 'apostrophes'.",
        "summary": "Contains <b>bold-looking</b> text & ampersands.",
        "body_html": "<p>pre-rendered, injected verbatim on purpose</p>",
        "is_formidable": False,
    }
    skills = {"x": skill}

    index_html = render_index(categories, skills)
    check("index: raw <em> from a name field never appears unescaped", "<em>" not in index_html)
    check("index: escaped name form is present", "&lt;em&gt;" in index_html)
    check("index: & in a summary field is escaped", "bold-looking&lt;/b&gt; text &amp; ampersands" in index_html)

    page_html = render_skill_page(skill, prev_skill=None, next_skill=None, siblings=[], categories=categories)
    check("page: raw <script> from a description field never appears unescaped", "<script>alert(1)</script>" not in page_html)
    check("page: escaped description form is present", "&lt;script&gt;alert(1)&lt;/script&gt;" in page_html)
    check("page: body_html's own tags are NOT double-escaped (verbatim-injection contract)", "<p>pre-rendered, injected verbatim on purpose</p>" in page_html)
    print("  escaping check passed")


def check_base_path() -> None:
    """GitHub Pages serves a project site from a subpath
    (https://<user>.github.io/<repo>/), not the domain root, so every
    root-relative href/src must be prefixed. Confirms both that a non-default
    base_path is honored everywhere and that the default ("") reproduces the
    exact previous root-relative behavior other checks above rely on."""
    print("base_path check")
    categories = FIXTURE["categories"]
    skills = FIXTURE["skills"]
    base = "/tbaguette-skills"
    # last_updated_utc must be passed here, unlike the other renders in this
    # function — _render_header() omits the whole updated-time element
    # (data-version-url along with it) when it's empty, so proving that
    # element's own href is base_path-prefixed needs it present at all.
    iso = "2026-08-13T00:00:00+00:00"

    index_html = render_index(categories, skills, base_path=base, last_updated_utc=iso)
    check("prefixed stylesheet href", f'"{base}/assets/styles.css"' in index_html)
    check("prefixed script src", f'"{base}/assets/site.js"' in index_html)
    check("prefixed skill card link", f'href="{base}/skills/formidable/"' in index_html)
    check("version-check URL is base_path-prefixed too", f'data-version-url="{base}/version.txt"' in index_html)
    check("un-prefixed root-relative form is absent once base_path is set", '"/assets/styles.css"' not in index_html)

    formidable = skills["formidable"]
    page_html = render_skill_page(
        formidable, prev_skill=None, next_skill=None, siblings=[], categories=categories,
        base_path=base, last_updated_utc=iso,
    )
    check("prefixed breadcrumb home link", f'href="{base}/"' in page_html)
    check("prefixed icon sprite reference", f'{base}/assets/icons.svg#' in page_html)
    check("skill page's version-check URL is base_path-prefixed too",
          f'data-version-url="{base}/version.txt"' in page_html)
    print("  base_path check passed")


def check_verify_install_page() -> None:
    """render_verify_install_page against the real, highlighted source of
    test_install_command.py — not a synthetic snippet, since the whole point
    of this page is displaying that exact file. Import is deferred to here
    (rather than the top of the file, next to the other imports) only
    because python_highlight is this module's own sibling under active
    development in the same change; the import itself is otherwise ordinary."""
    print("verify-install page check")
    from python_highlight import highlight_source

    real_source = (Path(__file__).resolve().parent / "test_install_command.py").read_text(encoding="utf-8")
    lines = highlight_source(real_source)
    categories = FIXTURE["categories"]

    html = render_verify_install_page(lines, categories)
    check("looks like a document", "<html" in html)
    check("title names what the page proves", "only touches one folder" in html)
    check("this page's <title> also ends in TBaguette’s Atelier, not the retired name",
          "<title>The install command only touches one folder — TBaguette’s Atelier</title>" in html)
    check("links out to the real file on GitHub as provenance",
          f'href="{INSTALL_TEST_GITHUB_URL}"' in html)
    check("renders one list item per source line",
          html.count('<li><span class="code-block__line-number">')
          == len(real_source.splitlines()))
    check("at least one comment span made it through from the real file",
          'class="tok-comment"' in html)
    check("at least one string span made it through from the real file",
          'class="tok-string"' in html)

    check("explains the PowerShell command exists, correctly escaped",
          escape_html(INSTALL_COMMAND_POWERSHELL) in html)
    check("explains the cmd.exe command exists, correctly escaped",
          escape_html(INSTALL_COMMAND_CMD) in html)
    check("is upfront that the PowerShell command is reasoned, not machine-tested",
          "not on an executed proof" in html)
    check("names the actual shells the POSIX command is cross-checked against",
          "bash, zsh, fish, and sh" in html)
    check("the twenty-four-check total is stated, not left at the stale earlier count",
          "twenty-four" in html and "fifteen" not in html)

    base = "/tbaguette-skills"
    prefixed = render_verify_install_page(lines, categories, base_path=base)
    check("code block content is base_path-independent (no hrefs inside code lines)",
          prefixed.count('class="code-block__line-code"')
          == html.count('class="code-block__line-code"'))
    check("but the page chrome around it is still prefixed like every other page",
          f'"{base}/assets/styles.css"' in prefixed)


def check_header_and_badges() -> None:
    """Dedicated fixture: the main FIXTURE's skills carry no change_status
    key at all, matching the common case (most page loads carry zero
    badges) — deliberately not exercising this path, which is exactly why
    it needs its own small fixture rather than piggybacking on FIXTURE."""
    print("header datetime + change-badge check")
    categories = [{"slug": "cat", "title": "Cat", "skill_slugs": ["fresh", "revised", "untouched"]}]
    base_skill = {
        "category_slug": "cat", "category_title": "Cat",
        "description": "d", "summary": "s", "body_html": "<p>x</p>", "is_formidable": False,
    }
    skills = {
        "fresh": {**base_skill, "slug": "fresh", "name": "fresh", "change_status": "new"},
        "revised": {**base_skill, "slug": "revised", "name": "revised", "change_status": "updated"},
        "untouched": {**base_skill, "slug": "untouched", "name": "untouched"},
    }

    iso = "2026-08-13T19:42:07+00:00"
    html = render_index(categories, skills, last_updated_utc=iso)

    check("header carries the baked-in datetime attribute", f'datetime="{iso}"' in html)
    check("header's no-JS fallback text is a real, readable UTC string",
          "2026-08-13T19:42:07Z UTC" in html)
    check("header time element is wired for site.js to find and reformat",
          "data-format-updated" in html)
    check("header time element also carries the version-check URL, "
          "base_path-prefixed (empty base_path here, so root-relative)",
          'data-version-url="/version.txt"' in html)
    check("wordmark carries its decorative wheat mark", "#icon-wheat" in html)
    check("wordmark reads TBaguette's Atelier, not just TBaguette",
          '<span class="wordmark__text">TBaguette<span class="wordmark__suffix">&rsquo;s Atelier</span></span>' in html)
    check("document <title> is TBaguette’s Atelier, not the retired La Boulangerie form",
          "<title>TBaguette’s Atelier — Claude Code skills, organized</title>" in html
          and "La Boulangerie TBaguette —" not in html)
    check("footer still names La Boulangerie TBaguette as the place, without repeating "
          "\"atelier\" awkwardly now that the title itself is TBaguette's Atelier",
          "La Boulangerie TBaguette</strong> is home to TBaguette&rsquo;s Atelier" in html)

    check("exactly two change-badges rendered (fresh and revised only, not untouched)",
          html.count('class="change-badge') == 2)
    check('"new" skill gets the New badge',
          'change-badge change-badge--new">New</span>' in html)
    check('"updated" skill gets the Updated badge',
          'change-badge change-badge--updated">Updated</span>' in html)

    no_time_html = render_index(categories, skills)
    check("with no last_updated_utc passed, the updated-time element is omitted "
          "entirely rather than left blank",
          "site-header__updated" not in no_time_html)

    fresh_page_html = render_skill_page(
        skills["fresh"], prev_skill=None, next_skill=None, siblings=[], categories=categories,
    )
    check("the skill's own page shows the same badge next to its title",
          'change-badge change-badge--new">New</span>' in fresh_page_html)
    check("badge sits inside the title row specifically, not the tag/description area",
          fresh_page_html.index("skill-article__title-row")
          < fresh_page_html.index("change-badge")
          < fresh_page_html.index("skill-article__tag"))
    check("a skill page's own <title> ends in TBaguette’s Atelier too",
          "<title>fresh — Cat — TBaguette’s Atelier</title>" in fresh_page_html)


def check_i18n_document_shell() -> None:
    """Task 3's own coverage: the document shell threads locale/strings
    through lang/dir, the hreflang/canonical block, and the language
    switcher. Deliberately does not exercise _render_hero/_render_breadcrumb/
    etc. — those keep their pre-locale signatures until Task 4."""
    print("i18n document shell check")
    import locales

    fr = locales.get_locale("fr")
    ar = locales.get_locale("ar")

    html_en = render_index(FIXTURE["categories"], FIXTURE["skills"], base_path="")
    check("default render_index still emits lang='en' dir='ltr' (no regression)",
          '<html lang="en" dir="ltr">' in html_en)

    html_fr = render_index(
        FIXTURE["categories"], FIXTURE["skills"], base_path="", locale=fr,
    )
    check("French index emits lang='fr' dir='ltr'", '<html lang="fr" dir="ltr">' in html_fr)
    check("French index's canonical link points at /fr/",
          '<link rel="canonical" href="/fr/">' in html_fr)
    check("French index carries an hreflang alternate for every one of the 16 locales, "
          "plus one more for x-default (17 total 'rel=\"alternate\" hreflang=' tags, since "
          "x-default's own <link> also matches that prefix)",
          html_fr.count('rel="alternate" hreflang="') == 17)
    check("French index carries an x-default hreflang pointing at the English root",
          'hreflang="x-default" href="/">' in html_fr)
    check("French index's own hreflang entry uses the plain 'fr' tag (not a region variant)",
          'hreflang="fr" href="/fr/">' in html_fr)
    check("French index's Portuguese hreflang entry uses the region-specific 'pt-BR' tag",
          'hreflang="pt-BR" href="/pt/">' in html_fr)

    html_ar = render_index(FIXTURE["categories"], FIXTURE["skills"], base_path="", locale=ar)
    check("Arabic index emits dir='rtl'", '<html lang="ar" dir="rtl">' in html_ar)

    html_skill_fr = render_skill_page(
        FIXTURE["skills"]["designing-test-data"],
        prev_skill=None, next_skill=None, siblings=[],
        categories=FIXTURE["categories"], base_path="", locale=fr,
    )
    check("French skill page's canonical points at its own /fr/skills/<slug>/ path",
          '<link rel="canonical" href="/fr/skills/designing-test-data/">' in html_skill_fr)
    check("French skill page's English hreflang alternate points at the un-prefixed root path",
          'hreflang="en" href="/skills/designing-test-data/">' in html_skill_fr)

    html_skill_base_path = render_skill_page(
        FIXTURE["skills"]["designing-test-data"],
        prev_skill=None, next_skill=None, siblings=[],
        categories=FIXTURE["categories"], base_path="/tbaguette-skills", locale=fr,
    )
    check("base_path prefixes every locale URL in the hreflang block, not just the current one",
          'hreflang="en" href="/tbaguette-skills/skills/designing-test-data/">' in html_skill_base_path)

    check("language switcher lists all 16 locales by endonym",
          html_en.count('class="language-switcher__link"') == 16)
    check("language switcher's French entry links to /fr/",
          '<a class="language-switcher__link" href="/fr/"' in html_en)
    check("language switcher marks the current locale with aria-current",
          'aria-current="true"' in html_en)


def check_i18n_content_links_and_strings() -> None:
    """Task 4's own coverage: unlike check_i18n_document_shell above (which
    only exercises the document shell), this confirms locale actually
    reaches content-level links -- cards, breadcrumb, prev/next, see-also --
    that Task 3 deliberately left on their pre-locale signatures."""
    import locales

    fr = locales.get_locale("fr")
    fr_strings = ENGLISH_STRINGS  # a real translated Strings isn't built until Task 12; reuse
                                   # ENGLISH_STRINGS here to isolate this test to *routing*
                                   # (does a card/breadcrumb/prevnext link land under /fr/?),
                                   # not translation content, which Task 12 covers separately.

    html_fr = render_index(FIXTURE["categories"], FIXTURE["skills"], base_path="", locale=fr, strings=fr_strings)
    check("a card on the French index links into /fr/skills/<slug>/, not the English root",
          'href="/fr/skills/designing-test-data/"' in html_fr)
    check("French index's install-frame 'See how' link points at the French "
          "verify-install page, not the English one",
          'href="/fr/verify-install/"' in html_fr)

    html_skill_fr = render_skill_page(
        FIXTURE["skills"]["designing-test-data"],
        prev_skill=FIXTURE["skills"]["formidable"], next_skill=FIXTURE["skills"]["flaky-test-triage"],
        siblings=[FIXTURE["skills"]["designing-test-data"], FIXTURE["skills"]["flaky-test-triage"]],
        categories=FIXTURE["categories"], base_path="", locale=fr, strings=fr_strings,
    )
    check("French skill page's breadcrumb Home link points at /fr/",
          '<a href="/fr/">' in html_skill_fr)
    check("French skill page's prev link points into /fr/skills/",
          '/fr/skills/formidable/' in html_skill_fr)
    check("French skill page's see-also link points into /fr/skills/",
          '/fr/skills/flaky-test-triage/' in html_skill_fr)

    html_en_badge = render_index(FIXTURE["categories"], {
        **FIXTURE["skills"],
        "formidable": {**FIXTURE["skills"]["formidable"], "change_status": "new"},
    }, base_path="")
    check("default-English 'New' badge text still renders (no regression)",
          '>New<' in html_en_badge)
    check("default-English index's install-frame 'See how' link is still unprefixed "
          "(no regression)",
          'href="/verify-install/"' in html_en_badge)


def check_i18n_verify_install_page() -> None:
    import locales
    from python_highlight import highlight_source

    fr = locales.get_locale("fr")
    ar = locales.get_locale("ar")
    highlighted = highlight_source("x = 1\n")

    html_en = render_verify_install_page(highlighted, FIXTURE["categories"], base_path="")
    check("default verify-install page still shows the real English heading (no regression)",
          "The install command only touches one folder" in html_en)
    check("default verify-install page has exactly 5 dir='ltr' occurrences -- the <html> tag "
          "itself (English is ltr) plus the 4 code-block/pre elements this task adds "
          "(the code-block div and 3 command <pre> blocks)",
          html_en.count('dir="ltr"') == 5)

    html_ar = render_verify_install_page(
        highlighted, FIXTURE["categories"], base_path="", locale=ar,
    )
    check("Arabic verify-install page's own <html> tag is dir='rtl', not dir='ltr'",
          '<html lang="ar" dir="rtl">' in html_ar)
    check("...but its code blocks still force dir='ltr' -- exactly 4 occurrences, which since "
          "the <html> tag itself is dir='rtl' here can only be coming from the code-block/pre "
          "elements, proving they're locale-independent rather than accidentally inherited from "
          "an English-only page",
          html_ar.count('dir="ltr"') == 4)

    html_fr = render_verify_install_page(
        highlighted, FIXTURE["categories"], base_path="", locale=fr,
    )
    check("French verify-install page's canonical points at /fr/verify-install/",
          '<link rel="canonical" href="/fr/verify-install/">' in html_fr)
    check("French verify-install page's breadcrumb Home link points at /fr/",
          '<a href="/fr/">' in html_fr)


def check_i18n_fallback_banner() -> None:
    import locales

    fr = locales.get_locale("fr")
    untranslated_skill = {**FIXTURE["skills"]["designing-test-data"], "translated": False}
    translated_skill = {**FIXTURE["skills"]["designing-test-data"], "translated": True}

    html_untranslated = render_skill_page(
        untranslated_skill, prev_skill=None, next_skill=None, siblings=[],
        categories=FIXTURE["categories"], base_path="", locale=fr,
    )
    check("an untranslated skill's page shows the fallback banner",
          'class="translation-banner"' in html_untranslated)
    check("an untranslated skill's body is marked lang='en' so a screen reader "
          "doesn't mispronounce English text as French",
          'class="prose" lang="en"' in html_untranslated)
    check("...and carries an explicit dir='ltr' alongside it, since HTML infers "
          "direction from dir alone and never from lang",
          'class="prose" lang="en" dir="ltr"' in html_untranslated)

    # The direction override only matters on an RTL page, which is where it
    # was missing: all 66 skill bodies are still English-only, so before this
    # every /ar/skills/* page laid English out as an RTL paragraph and threw
    # each trailing colon and period to the start of its visual line.
    ar = locales.get_locale("ar")
    html_ar_untranslated = render_skill_page(
        untranslated_skill, prev_skill=None, next_skill=None, siblings=[],
        categories=FIXTURE["categories"], base_path="", locale=ar,
    )
    check("on an RTL page the English fallback body still forces dir='ltr', so it "
          "does not inherit dir='rtl' from the <html> element",
          'class="prose" lang="en" dir="ltr"' in html_ar_untranslated)
    check("...on a page whose own <html> element really is dir='rtl'",
          '<html lang="ar" dir="rtl">' in html_ar_untranslated)

    # formidable's extras are fallback English too, and sit *outside*
    # _render_prose -- fixing only the main body left 21 of 24 elements in
    # the craft floor and 20 of 31 in the first tab panel still laid out RTL
    # on /ar/skills/formidable/. Found by looking at the page, not the diff.
    untranslated_formidable = {**FIXTURE["skills"]["formidable"], "translated": False}
    html_ar_formidable = render_skill_page(
        untranslated_formidable, prev_skill=None, next_skill=None, siblings=[],
        categories=FIXTURE["categories"], base_path="", locale=ar,
    )
    check("formidable's tab panels carry the same fallback lang/dir as the body",
          'tabindex="0" lang="en" dir="ltr">' in html_ar_formidable
          or 'tabindex="0" hidden lang="en" dir="ltr">' in html_ar_formidable)
    check("...both the visible first panel and the hidden ones",
          'tabindex="0" lang="en" dir="ltr">' in html_ar_formidable
          and 'tabindex="0" hidden lang="en" dir="ltr">' in html_ar_formidable)
    check("formidable's craft-floor prose carries it too",
          html_ar_formidable.count('<div class="prose" lang="en" dir="ltr">') == 2)
    check("the translated section headings are NOT swept into the override -- "
          "they come from Strings and really are in the page's language",
          'class="formidable-extra__subtitle" lang="en"' not in html_ar_formidable
          and 'class="formidable-extra" lang="en"' not in html_ar_formidable)

    html_ar_formidable_translated = render_skill_page(
        {**FIXTURE["skills"]["formidable"], "translated": True},
        prev_skill=None, next_skill=None, siblings=[],
        categories=FIXTURE["categories"], base_path="", locale=ar,
    )
    check("a fully translated formidable page gets no override on its panels "
          "either -- translated content follows the page's own direction",
          'lang="en" dir="ltr"' not in html_ar_formidable_translated)

    html_translated = render_skill_page(
        translated_skill, prev_skill=None, next_skill=None, siblings=[],
        categories=FIXTURE["categories"], base_path="", locale=fr,
    )
    check("a translated skill's page shows no fallback banner",
          'class="translation-banner"' not in html_translated)
    check("a translated skill's body carries no lang override",
          'class="prose" lang="en"' not in html_translated)
    check("...and no direction override either -- a translated body is in the "
          "page's own language and must follow the page's own direction",
          'class="prose"><' in html_translated or 'class="prose">' in html_translated)
    check("...specifically, no dir attribute is emitted on it",
          'class="prose" dir=' not in html_translated)

    html_en = render_index(FIXTURE["categories"], FIXTURE["skills"], base_path="")
    html_default_skill_page = render_skill_page(
        FIXTURE["skills"]["designing-test-data"], prev_skill=None, next_skill=None, siblings=[],
        categories=FIXTURE["categories"], base_path="",
    )
    check("the fixture's own English skills (translated key absent, defaults True) "
          "never show a banner on the default-locale build",
          'class="translation-banner"' not in html_default_skill_page)


def check_i18n_multiword_heading_id() -> None:
    """_render_tab_group's heading id must come from a real slugify, not a
    bare .lower() -- English/French both happen to have single-word Stacks/
    Commands headings today, so .lower() alone never showed the bug, but a
    genuinely multi-word translated heading would leave a literal space in
    the id, which is invalid HTML and breaks aria-labelledby (a
    space-separated id list would misparse one space-containing id as two
    nonexistent ones)."""
    import dataclasses
    import re

    multiword_strings = dataclasses.replace(
        ENGLISH_STRINGS, formidable_stacks_heading="Multi Word Heading"
    )
    html = render_skill_page(
        FIXTURE["skills"]["formidable"], prev_skill=None, next_skill=None, siblings=[],
        categories=FIXTURE["categories"], base_path="", strings=multiword_strings,
    )
    check("a multi-word translated heading produces a space-free HTML id "
          "(otherwise aria-labelledby, a space-separated id list, would "
          "misparse a single id containing an embedded space as two ids)",
          'id="multi-word-heading-heading"' in html)

    aria_match = re.search(r'aria-labelledby="(multi[^"]*)"', html)
    check("...and the tablist section's aria-labelledby value pointing at "
          "that heading contains no literal space character",
          aria_match is not None and " " not in aria_match.group(1))


def check_i18n_quote_glyphs() -> None:
    """Task 12's own coverage: the search empty-state's quote marks around
    the visitor's live query used to be hardcoded English/typewriter-curly
    quotes ('“'/'”') in site.js regardless of locale. The fix threads
    them through Strings -> data-i18n-quote-open/-close body attributes,
    same pattern as data-i18n-copied/-no-match/-modal-* just above them in
    _render_document.

    escape_html is Python's stdlib html.escape(quote=True), which only
    escapes & < > " ' -- it has no notion of HTML named entities like
    &ldquo;/&rdquo; and leaves any other Unicode character, curly quotes
    included, completely untouched. So the *correct* expectation for the
    English default is the literal '“'/'”' characters appearing
    verbatim in the attribute value, not an &ldquo;/&rdquo; entity form."""
    import dataclasses

    html_en = render_index(FIXTURE["categories"], FIXTURE["skills"], base_path="")
    check("default English render's data-i18n-quote-open is the literal "
          "left curly quote (“), unescaped -- escape_html doesn't turn "
          "arbitrary Unicode into named entities, only & < > \" '",
          'data-i18n-quote-open="“"' in html_en)
    check("default English render's data-i18n-quote-close is the literal "
          "right curly quote (”), unescaped",
          'data-i18n-quote-close="”"' in html_en)

    custom_strings = dataclasses.replace(
        ENGLISH_STRINGS,
        search_empty_query_open="TEST-OPEN",
        search_empty_query_close="TEST-CLOSE",
    )
    html_custom = render_index(
        FIXTURE["categories"], FIXTURE["skills"], base_path="", strings=custom_strings,
    )
    check("a Strings instance with custom open/close quote values renders "
          "its exact open value into data-i18n-quote-open",
          'data-i18n-quote-open="TEST-OPEN"' in html_custom)
    check("...and its exact close value into data-i18n-quote-close",
          'data-i18n-quote-close="TEST-CLOSE"' in html_custom)


def check_i18n_sentence_end() -> None:
    """_render_install (the "verified... See how." note) and
    _render_search_empty_state (the 'No skills match "x".' empty state)
    each had a bare ASCII '.' sitting in the f-string literal itself,
    outside any Strings field, applied unconditionally regardless of
    locale. Same bug class as check_i18n_quote_glyphs above (f8b726cc),
    same fix shape: thread a new Strings field (sentence_end) through both
    call sites instead of hardcoding the literal.

    This was invisible for the six Latin/Cyrillic-script locales shipped
    so far -- a period is correct sentence-final punctuation for all of
    French/Spanish/German/Italian/Portuguese/Russian -- but wrong for
    Chinese, which ends a declarative sentence with the full-width
    ideographic full stop 。instead, producing a half-width "." visibly
    glued onto otherwise full-width-punctuated Chinese prose.

    The source-text checks below read templates.py's own source and fail
    with a clean, readable assertion before the fix. The render-based
    checks further down would instead crash on a raw TypeError before the
    fix (dataclasses.replace(ENGLISH_STRINGS, sentence_end=...) can't set
    a field that doesn't exist yet on Strings) -- ordered second on
    purpose, so the source-text checks are what actually reports red
    first.
    """
    import dataclasses

    templates_src = Path(__file__).resolve().with_name("templates.py").read_text(encoding="utf-8")
    check("_render_install's install_note_see_how line no longer hardcodes "
          "a bare '.' right after </a> (must come from strings.sentence_end "
          "instead)",
          "{escape_html(strings.install_note_see_how)}</a>.</span>" not in templates_src)
    check("_render_search_empty_state's line no longer hardcodes a bare "
          "'.' right after the search-query </span>",
          "data-search-empty-query></span>." not in templates_src)

    html_en = render_index(FIXTURE["categories"], FIXTURE["skills"], base_path="")
    check("default English render still ends the install note with a "
          "literal '.' after \"See how\" (sentence_end defaults to '.', so "
          "English output is unchanged)",
          "See how</a>." in html_en)
    check("default English render still ends the search empty state with "
          "a literal '.' right after the query span",
          "data-search-empty-query></span>." in html_en)

    custom_strings = dataclasses.replace(ENGLISH_STRINGS, sentence_end="★")
    html_custom = render_index(
        FIXTURE["categories"], FIXTURE["skills"], base_path="", strings=custom_strings,
    )
    check("a Strings instance with a custom sentence_end renders it at the "
          "install note site instead of a hardcoded '.'",
          "See how</a>★" in html_custom and "See how</a>." not in html_custom)
    check("...and at the search empty-state site too",
          "data-search-empty-query></span>★" in html_custom
          and "data-search-empty-query></span>." not in html_custom)


def check_i18n_prevnext_arrows() -> None:
    """← (U+2190) and → (U+2192) lack the Unicode Bidi_Mirrored property, so
    the bidi algorithm never flips them the way it flips brackets. On an RTL
    page the prev/next arrows therefore kept pointing the LTR way round --
    "next" drawn as → when a right-to-left reader advances leftward.

    Bidi does already place each arrow on the correct side unaided, so only
    the glyph is swapped, never the slot it sits in. A CSS
    transform: scaleX(-1) (the .card__arrow approach) is not usable here:
    transform does not apply to non-replaced inline elements, and
    .prevnext__name is a plain block, not the flex container that blockifies
    .card__arrow.

    Invisible to the RTL *CSS* pass in test_i18n.py, which can only see CSS
    properties -- this is a literal character in the markup."""
    import locales

    dtd = FIXTURE["skills"]["designing-test-data"]
    ftt = FIXTURE["skills"]["flaky-test-triage"]

    def render(locale):
        return (
            render_skill_page(dtd, prev_skill=None, next_skill=ftt, siblings=[],
                              categories=FIXTURE["categories"], base_path="", locale=locale),
            render_skill_page(ftt, prev_skill=dtd, next_skill=None, siblings=[],
                              categories=FIXTURE["categories"], base_path="", locale=locale),
        )

    en_next, en_prev = render(locales.DEFAULT_LOCALE)
    check("LTR: the next link points → (unchanged)",
          'class="prevnext__name">flaky-test-triage →<' in en_next)
    check("LTR: the prev link points ← (unchanged)",
          'class="prevnext__name">← designing-test-data<' in en_prev)

    fr_next, fr_prev = render(locales.get_locale("fr"))
    check("an already-shipped LTR locale is byte-for-byte unaffected by the "
          "swap (French next link still →)",
          'class="prevnext__name">flaky-test-triage →<' in fr_next)
    check("...and French prev still ←",
          'class="prevnext__name">← designing-test-data<' in fr_prev)

    ar_next, ar_prev = render(locales.get_locale("ar"))
    check("RTL: the next link points ← -- a right-to-left reader advances "
          "leftward, so that is the forward direction",
          'class="prevnext__name">flaky-test-triage ←<' in ar_next)
    check("RTL: ...and carries no → anywhere in its prev/next nav",
          "→" not in ar_next[ar_next.index('class="container prevnext"'):])
    check("RTL: the prev link points → -- backward, for the same reason",
          'class="prevnext__name">→ designing-test-data<' in ar_prev)
    check("RTL: ...and carries no ← anywhere in its prev/next nav",
          "←" not in ar_prev[ar_prev.index('class="container prevnext"'):])

    check("the arrow stays in its existing logical slot on RTL (before the "
          "name for prev, after it for next) -- bidi already puts it on the "
          "correct side, so only the glyph needed mirroring",
          'class="prevnext__name">→ designing-test-data<' in ar_prev
          and 'class="prevnext__name">flaky-test-triage ←<' in ar_next)


def main() -> None:
    categories = FIXTURE["categories"]
    skills = FIXTURE["skills"]

    # --- render_index -----------------------------------------------------
    print("render_index")
    index_html = render_index(categories, skills)
    index_path = write_preview("index.html", index_html)
    check("non-empty", len(index_html) > 0)
    check("looks like a document", "<html" in index_html)
    check("references the shared stylesheet root-relatively", '"/assets/styles.css"' in index_html)
    check("references the shared script root-relatively", '"/assets/site.js"' in index_html)
    for cat in categories:
        check(f"contains category title {cat['title']!r}", cat["title"] in index_html)
    for slug, skill in skills.items():
        check(f"contains skill name {slug!r}", skill["name"] in index_html)
        check(f"contains skill summary for {slug!r}", skill["summary"] in index_html)
        check(f"links to /skills/{slug}/", f'href="/skills/{slug}/"' in index_html)
    check("has a search input", 'data-search-input' in index_html)
    # Both commands contain && / {} , which escape_html correctly turns into
    # entity forms — checking for the raw form here would either fail
    # (proving nothing) or, worse, pass by accident if escaping were ever
    # silently disabled. Checking the escaped form catches that regression
    # directly.
    check("POSIX install command appears, correctly HTML-escaped",
          escape_html(INSTALL_COMMAND) in index_html)
    check("raw, un-escaped POSIX command never appears (would mean escaping broke)",
          INSTALL_COMMAND not in index_html)
    check("PowerShell install command appears, correctly HTML-escaped",
          escape_html(INSTALL_COMMAND_POWERSHELL) in index_html)
    check("raw, un-escaped PowerShell command never appears",
          INSTALL_COMMAND_POWERSHELL not in index_html)
    check("install frame sits right after the headline, before the lede",
          index_html.index("hero__headline") < index_html.index('id="install-posix-command"')
          < index_html.index("hero__lede"))
    check("has a copy button wired to the POSIX command",
          'data-copy-target="install-posix-command"' in index_html)
    check("has a separate copy button wired to the PowerShell command",
          'data-copy-target="install-powershell-command"' in index_html)
    check("install frame is wrapped in its labeled frame",
          index_html.index("install-frame") < index_html.index("Install TBaguette")
          < index_html.index('id="install-posix-command"'))
    label_start = index_html.index('install-frame__label')
    label_end = index_html.index('</p>', label_start)
    check("frame label itself carries an icon (icon-crust also appears in category "
          "headers elsewhere on the page, so this checks the label's own slice, not "
          "just presence anywhere)",
          '#icon-crust' in index_html[label_start:label_end])
    check("verification note sits after both commands and before the lede, inside the frame",
          index_html.index('id="install-powershell-command"') < index_html.index("install-frame__note")
          < index_html.index("hero__lede"))
    check("verification note links to the on-site explanation page, base_path-prefixed",
          'href="/verify-install/"' in index_html)
    check("a second note tells visitors to restart/reload and how to invoke a skill",
          'Restart Claude Code' in index_html and 'TBaguette:skill-name' in index_html)
    check("that note clarifies this is Claude Code-specific, not Desktop app/claude.ai chat "
          "(the actual bug this was written to prevent: a visitor installs correctly but "
          "never sees the skills because they're looking in the wrong product)",
          'Claude Desktop app and claude.ai chat load skills from your account' in index_html)
    check("the Claude-Code-specific note sits after the safety note, before the lede",
          index_html.index("install-frame__note") < index_html.index('Restart Claude Code')
          < index_html.index("hero__lede"))

    # --- platform picker: two tabs, POSIX shown by default, PowerShell hidden ---
    print("install platform picker")
    posix_tab_start = index_html.index('id="tab-install-posix"')
    posix_tab = index_html[posix_tab_start:index_html.index('</button>', posix_tab_start)]
    check("POSIX tab starts selected", 'aria-selected="true"' in posix_tab)
    check("POSIX tab is tagged for the auto-select logic to recognize as the non-Windows option",
          'data-platform="posix"' in posix_tab)

    ps_tab_start = index_html.index('id="tab-install-powershell"')
    ps_tab = index_html[ps_tab_start:index_html.index('</button>', ps_tab_start)]
    check("Windows tab starts unselected", 'aria-selected="false"' in ps_tab)
    check("Windows tab is tagged for the auto-select logic to find",
          'data-platform="windows"' in ps_tab)

    ps_panel_start = index_html.index('id="install-powershell"')
    ps_panel = index_html[ps_panel_start:index_html.index('id="install-powershell-command"')]
    check("PowerShell panel starts hidden (JS-driven auto-select or a click reveals it)",
          "hidden" in ps_panel)
    posix_panel_start = index_html.index('id="install-posix"')
    posix_panel = index_html[posix_panel_start:index_html.index('id="install-posix-command"')]
    check("POSIX panel does NOT start hidden — correct even with JS disabled",
          "hidden" not in posix_panel)

    check("group opts into platform auto-selection", 'data-autoselect-platform="true"' in index_html)
    check("each platform panel names which shells/versions it covers",
          "Works in bash, zsh, or fish" in index_html and "PowerShell 5.1 or 7" in index_html)
    print(f"  wrote {index_path}")

    # --- render_skill_page: formidable (the interesting one) --------------
    # No prev/next, no siblings: formidable is alone in its category in this
    # fixture, exactly like the real content it stands in for. This is a
    # real empty state, not a hypothetical one, so it's worth its own file.
    print("render_skill_page (formidable)")
    formidable = skills["formidable"]
    formidable_html = render_skill_page(
        formidable, prev_skill=None, next_skill=None, siblings=[], categories=categories
    )
    skill_path = write_preview("skill.html", formidable_html)
    check("non-empty", len(formidable_html) > 0)
    check("looks like a document", "<html" in formidable_html)
    check("contains the fixture skill name", "formidable" in formidable_html)
    check("contains the full description (trigger text)", formidable["description"] in formidable_html)
    check("injects body_html verbatim (h2#overview)", 'id="overview"' in formidable_html)
    check("injects body_html verbatim (strong)", "<strong>out-of-distribution</strong>" in formidable_html)
    for item in formidable["formidable_stacks"] + formidable["formidable_commands"]:
        check(f"contains tab panel id {item['id']!r}", f'id="{item["id"]}"' in formidable_html)
        check(f"contains tab title {item['title']!r}", item["title"] in formidable_html)
    check("first stack tab starts selected", 'aria-controls="stack-web" aria-selected="true"' in formidable_html)
    check("second stack tab starts unselected", 'aria-controls="stack-terminal-tui" aria-selected="false"' in formidable_html)
    check("no prev/next nav when both are None", 'class="container prevnext"' not in formidable_html)
    check("no see-also section with zero siblings", 'class="container see-also"' not in formidable_html)
    check("craft floor section id exists (the anchor formidable's own links target)", 'id="cmd-craft-floor"' in formidable_html)
    check("craft floor heading present", "Craft floor" in formidable_html)
    check("craft floor content injected verbatim", "Load immediately before editing UI" in formidable_html)
    print(f"  wrote {skill_path}")

    # --- render_skill_page: the two-skill "testing" category --------------
    # Extra coverage beyond the required two files: exercises a table inside
    # body_html, and both directions of prev/next (one skill has next-only,
    # the other has prev-only) plus a populated see-also list.
    print("render_skill_page (testing category, both directions)")
    dtd = skills["designing-test-data"]
    ftt = skills["flaky-test-triage"]
    testing_siblings = [dtd, ftt]

    dtd_html = render_skill_page(
        dtd, prev_skill=None, next_skill=ftt, siblings=testing_siblings, categories=categories
    )
    dtd_path = write_preview("skill-testing-1.html", dtd_html)
    check("table from body_html is present", "<table>" in dtd_html)
    check("next-only: has a next link", 'prevnext__link--next' in dtd_html)
    check("next-only: has no prev label", ">Previous<" not in dtd_html)
    check("see-also excludes self, includes sibling", "flaky-test-triage" in dtd_html)
    print(f"  wrote {dtd_path}")

    ftt_html = render_skill_page(
        ftt, prev_skill=dtd, next_skill=None, siblings=testing_siblings, categories=categories
    )
    ftt_path = write_preview("skill-testing-2.html", ftt_html)
    check("prev-only: has a previous label", ">Previous<" in ftt_html)
    check("prev-only: has no next-modifier link", 'prevnext__link--next' not in ftt_html)
    check("see-also excludes self, includes sibling", "designing-test-data" in ftt_html)
    print(f"  wrote {ftt_path}")

    check_escaping()
    check_base_path()
    check_verify_install_page()
    check_header_and_badges()
    check_i18n_document_shell()
    check_i18n_content_links_and_strings()
    check_i18n_verify_install_page()
    check_i18n_fallback_banner()
    check_i18n_multiword_heading_id()
    check_i18n_quote_glyphs()
    check_i18n_sentence_end()
    check_i18n_prevnext_arrows()

    print(f"\n{checker.total} checks passed.")
    print(f"Preview files written to {PREVIEW_DIR}")
    print("Serve from the project root (e.g. `python3 -m http.server 8000`) and open:")
    print("  /_preview/index.html")
    print("  /_preview/skill.html")
    print("  /_preview/skill-testing-1.html")
    print("  /_preview/skill-testing-2.html")


if __name__ == "__main__":
    main()
