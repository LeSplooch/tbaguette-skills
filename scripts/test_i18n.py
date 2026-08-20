"""Tests for the site's i18n build: locale registry integrity, per-file
content fallback, description precedence, UI-string key parity, routing
output (hreflang/canonical/switcher), and RTL. Plain assert-based, using
the shared Checker (see scripts/checker.py) -- matches test_generate.py and
test_templates.py's convention, not test_content_pipeline.py's unittest one.

Usage:
    python3 scripts/test_i18n.py
"""

from __future__ import annotations

import dataclasses
import json
import re
import shutil
import tempfile
from pathlib import Path
from unittest import mock

from checker import Checker
import content_pipeline
import generate
import locales
import templates

checker = Checker()
check = checker.check


def check_locale_registry() -> None:
    print("locale registry")
    check(
        f"exactly {locales.EXPECTED_LOCALE_COUNT} locales",
        len(locales.LOCALES) == locales.EXPECTED_LOCALE_COUNT,
    )
    codes = [locale.code for locale in locales.LOCALES]
    check("every locale code is unique", len(codes) == len(set(codes)))
    check(
        "every locale's dir is exactly 'ltr' or 'rtl'",
        all(locale.dir in ("ltr", "rtl") for locale in locales.LOCALES),
    )
    check(
        "exactly one locale is marked default",
        sum(1 for locale in locales.LOCALES if locale.default) == 1,
    )
    check("English is present and is the default locale", locales.DEFAULT_LOCALE.code == "en")
    check(
        "Arabic is the only rtl locale (matches the approved 15-language list)",
        [locale.code for locale in locales.LOCALES if locale.dir == "rtl"] == ["ar"],
    )
    check("get_locale('fr') returns the French entry", locales.get_locale("fr").endonym == "Français")

    threw = False
    try:
        locales.get_locale("xx")
    except KeyError:
        threw = True
    check("get_locale raises KeyError for an unknown code", threw)


def _write_skill(skills_root: Path, slug: str, description: str, body: str) -> None:
    skill_dir = skills_root / slug
    skill_dir.mkdir(parents=True, exist_ok=True)
    (skill_dir / "SKILL.md").write_text(
        f"---\nname: {slug}\ndescription: {description}\n---\n{body}\n", encoding="utf-8"
    )


def check_full_locale_build() -> None:
    print("full locale build (synthetic fixture)")
    tmp_root = Path(tempfile.mkdtemp(prefix="tbaguette-i18n-build-test-"))
    try:
        skills_root = tmp_root / "skills"
        _write_skill(skills_root, "alpha", "English alpha description.", "English alpha body.")
        _write_skill(skills_root, "beta", "English beta description.", "English beta body.")

        fixture_categories = [
            {"slug": "test-cat", "title": "Test Category", "skill_slugs": ["alpha", "beta"]},
        ]

        i18n_root = tmp_root / "i18n"
        fr_dir = i18n_root / "fr"
        fr_dir.mkdir(parents=True)
        ui_json = {field.name: getattr(templates.ENGLISH_STRINGS, field.name)
                   for field in dataclasses.fields(templates.Strings)}
        ui_json["hero_headline"] = "Un atelier pour votre façon de coder."
        (fr_dir / "ui.json").write_text(json.dumps(ui_json), encoding="utf-8")
        (fr_dir / "categories.json").write_text(
            json.dumps({"test-cat": "Catégorie de test"}), encoding="utf-8"
        )
        # Only alpha gets a real translated body -- beta stays English + banner,
        # proving per-skill fallback survives a real generate() run end to end.
        _write_skill(fr_dir / "skills", "alpha", "Description alpha en français.", "Corps alpha en français.")

        # es/ deliberately gets nothing created under it at all -- proves the
        # silent-fallback-to-English path (a locale doesn't need any content
        # on disk yet for the site to build correctly-routed pages for it).

        with mock.patch.object(content_pipeline, "CATEGORIES", fixture_categories), \
             mock.patch.object(generate, "EXPECTED_SKILL_COUNT", 2), \
             mock.patch.object(generate, "_default_i18n_root", lambda root: i18n_root):
            content = generate.generate(tmp_root, skills_root, base_path="")

        docs = tmp_root / "docs"
        check("English index still builds at the root", (docs / "index.html").is_file())
        check("French index builds under docs/fr/", (docs / "fr" / "index.html").is_file())
        check("every other locale's directory also exists (all registered locales always route)",
              all((docs / loc.code).is_dir() for loc in locales.LOCALES if not loc.default))

        fr_index_html = (docs / "fr" / "index.html").read_text(encoding="utf-8")
        check("French index emits dir='ltr' lang='fr'", '<html lang="fr" dir="ltr">' in fr_index_html)
        check("French index's translated hero headline made it through",
              "Un atelier pour votre façon de coder." in fr_index_html)
        check("French index carries all 12 hreflang alternates plus x-default",
              fr_index_html.count('rel="alternate" hreflang="') == 13)

        fr_alpha_html = (docs / "fr" / "skills" / "alpha" / "index.html").read_text(encoding="utf-8")
        check("French alpha page shows the real translated body, no fallback banner",
              "Corps alpha en français" in fr_alpha_html and 'class="translation-banner"' not in fr_alpha_html)

        fr_beta_html = (docs / "fr" / "skills" / "beta" / "index.html").read_text(encoding="utf-8")
        check("French beta page falls back to the English body with a fallback banner",
              "English beta body" in fr_beta_html and 'class="translation-banner"' in fr_beta_html)

        es_index_html = (docs / "es" / "index.html").read_text(encoding="utf-8")
        check("Spanish index (no ui.json on disk at all) silently falls back to English chrome text",
              ">Search skills<" in es_index_html or "Search skills" in es_index_html)
        check("Spanish index is still correctly labeled dir/lang for Spanish despite English chrome text",
              '<html lang="es" dir="ltr">' in es_index_html)

        version_txt = (docs / "version.txt").read_text(encoding="utf-8")
        fr_dt_match = re.search(r'datetime="([^"]+)"', fr_index_html)
        check("version.txt matches the French page's own timestamp (one shared build instant)",
              fr_dt_match is not None and version_txt == fr_dt_match.group(1))

        check("generate() still returns the English content dict (byte-identical-contract)",
              content["categories"][0]["slug"] == "test-cat" and len(content["skills"]) == 2)

        # --locale-scoped build: only fr/ (+ English root) should be touched;
        # es/ must survive untouched from the full build above.
        es_dir_mtime_before = (docs / "es").stat().st_mtime
        with mock.patch.object(content_pipeline, "CATEGORIES", fixture_categories), \
             mock.patch.object(generate, "EXPECTED_SKILL_COUNT", 2), \
             mock.patch.object(generate, "_default_i18n_root", lambda root: i18n_root):
            generate.generate(tmp_root, skills_root, base_path="", only_locale="fr")
        check("a --locale fr build leaves es/ untouched on disk",
              (docs / "es").stat().st_mtime == es_dir_mtime_before)
    finally:
        shutil.rmtree(tmp_root, ignore_errors=True)


def check_locale_count_gate() -> None:
    print("EXPECTED_LOCALE_COUNT gate")
    # generate.EXPECTED_LOCALE_COUNT is a plain int copied from
    # locales.EXPECTED_LOCALE_COUNT at import time (see Step 3 below) --
    # patching locales.EXPECTED_LOCALE_COUNT would NOT affect generate.py's
    # own already-bound copy, so the gate check inside generate() (which
    # references the bare name EXPECTED_LOCALE_COUNT, resolved against
    # generate.py's own module globals) must be patched directly on the
    # generate module instead.
    with mock.patch.object(generate, "EXPECTED_LOCALE_COUNT", 999):
        tmp_root = Path(tempfile.mkdtemp(prefix="tbaguette-i18n-gate-test-"))
        try:
            skills_root = tmp_root / "skills"
            _write_skill(skills_root, "alpha", "English alpha description.", "English alpha body.")
            threw = False
            try:
                generate.generate(tmp_root, skills_root, base_path="")
            except SystemExit:
                threw = True
            check("a mismatched EXPECTED_LOCALE_COUNT refuses to generate", threw)
        finally:
            shutil.rmtree(tmp_root, ignore_errors=True)


def check_rtl_css_pass() -> None:
    print("RTL CSS pass")
    css_path = Path(__file__).resolve().parent.parent / "docs" / "assets" / "styles.css"
    css = css_path.read_text(encoding="utf-8")

    # The code-block line-number gutter is the one deliberate, documented
    # exception -- it displays literal source code, which stays LTR on
    # every page regardless of the surrounding language (see templates.py's
    # dir="ltr" on .code-block, Task 5) -- so its own left/text-align:right
    # are correct as physical properties and must NOT be converted.
    exempt_block_start = css.index(".code-block__line-number {")
    exempt_block_end = css.index("}", exempt_block_start)
    css_outside_exemption = css[:exempt_block_start] + css[exempt_block_end:]

    check("no more than the one documented exemption uses a bare 'padding-left'",
          "padding-left" not in css_outside_exemption)
    check("no bare 'text-align: left' remains outside the exemption",
          "text-align: left" not in css_outside_exemption)
    check("no bare 'text-align: right' remains outside the exemption",
          "text-align: right" not in css_outside_exemption)
    check("the card-arrow hover motion has an explicit [dir=\"rtl\"] override",
          '[dir="rtl"] .card:hover .card__arrow' in css)
    check("...and that override uses the correct transform composition "
          "(scaleX(-1) translateX(3px), not a negated translateX(-3px) which "
          "would move the glyph the wrong screen-space direction under RTL)",
          "scaleX(-1) translateX(3px)" in css)
    check("the tab-list scroll-fade mask has an explicit [dir=\"rtl\"] override",
          '[dir="rtl"] .tabs__list' in css)
    check("...and that override actually points the fade 'to left' on both "
          "-webkit-mask-image and mask-image (not still 'to right')",
          css.count("to left, black") == 2)

    # Code is LTR in every language. Task 5 handled this per call site with
    # dir="ltr" attributes, which by construction only covers the call sites
    # someone thought of -- it missed the homepage install command and every
    # inline <code> inside a translator-authored _html string in i18n/*.json,
    # which reviewing this repo's Python cannot reach at all. One rule on the
    # element covers all of them, plus any <code> not yet written.
    code_rule = css[css.index("\ncode {"):css.index("}", css.index("\ncode {"))]
    check("the base 'code' element rule pins direction: ltr, so no <code> "
          "inherits dir=\"rtl\" from the page (an inline path rendered as "
          "'claude/skills/TBaguette./~' without it)",
          "direction: ltr" in code_rule)
    check("...and isolates it with unicode-bidi: isolate -- what HTML's own "
          "dir attribute maps to, sealing the run in both directions so "
          "neither the Arabic reorders the code nor the code disturbs the "
          "Arabic",
          "unicode-bidi: isolate;" in code_rule)
    check("...and specifically not isolate-override, which would force RTL "
          "text inside a code sample to display left-to-right",
          "isolate-override" not in code_rule)
    check("...and not plaintext, which would infer direction from the first "
          "strong character and so fall back to the paragraph's RTL for a "
          "snippet starting with a neutral (~, / or -) -- the broken case",
          "plaintext" not in code_rule)

    # The rule only helps if it actually reaches the reported offenders, so
    # assert against the templates rather than trusting that a bare `code`
    # selector matches them.
    templates_src = (Path(__file__).resolve().with_name("templates.py")).read_text(encoding="utf-8")
    check("the homepage install command is a <code> element, so the rule "
          "reaches it without a per-call-site attribute (D2: the primary "
          "call to action rendered '] d ... [' for '[ -d ... ]')",
          '<code class="install__command"' in templates_src)
    check("Task 5's dir=\"ltr\" on .code-block is still present -- NOT made "
          "redundant by the new rule, since .code-block renders <div>/<span> "
          "and never a <code> element",
          '<div class="code-block" dir="ltr">' in templates_src)


def check_language_switcher_css() -> None:
    """The language switcher is the one new piece of chrome the i18n work
    adds, and it shipped with literally zero CSS: none of its four class
    names appeared anywhere in styles.css. A native unstyled <details>
    expands in normal flow, so opening it inflated the header from ~140px
    to ~620px and pushed the whole page down, on every page of all 16
    locales.

    Asserting the selectors merely *exist* would repeat the weakness the
    final review called out in Task 9's RTL checks, so each check below
    pins the property that actually does the work -- above all the
    absolute positioning, which is the entire difference between a popover
    and a header that grows by 480px."""
    print("language switcher CSS")
    css_path = Path(__file__).resolve().parent.parent / "docs" / "assets" / "styles.css"
    css = css_path.read_text(encoding="utf-8")

    def rule_body(selector: str) -> str:
        start = css.index(selector + " {")
        return css[start:css.index("}", start)]

    for selector in (".language-switcher", ".language-switcher__summary",
                     ".language-switcher__list", ".language-switcher__link"):
        check(f"{selector} has a rule at all (all four shipped unstyled)",
              selector + " {" in css)

    switcher = rule_body(".language-switcher")
    check("the <details> establishes a containing block, so the panel can "
          "anchor to it rather than to the page",
          "position: relative" in switcher)

    panel = rule_body(".language-switcher__list")
    check("the open panel is OUT OF NORMAL FLOW -- the whole bug was a "
          "16-item list expanding in flow and inflating the header",
          "position: absolute" in panel)
    check("...and is anchored with the logical inset-inline-end, not a "
          "physical right, so /ar/ mirrors without a [dir=\"rtl\"] override",
          "inset-inline-end" in panel and "right:" not in panel)
    check("...and stacks above the header band rather than behind it",
          "z-index" in panel)
    check("...and drops the <ul>'s bullets and default indent",
          "list-style: none" in panel)
    check("...and scrolls rather than running off a short viewport, since "
          "16 entries overflow one",
          "overflow-y: auto" in panel and "max-height" in panel)

    summary = rule_body(".language-switcher__summary")
    check("the summary drops the native disclosure triangle",
          "list-style: none" in summary)
    check("...including on WebKit/Blink, which ignores list-style on a "
          "summary and needs the pseudo-element reset too",
          ".language-switcher__summary::-webkit-details-marker" in css)

    check("the current locale, marked aria-current in the markup, gets a "
          "visible treatment too -- it was announced but indistinguishable",
          ".language-switcher__link[aria-current]" in css)

    check("the switcher's transitions are covered by the reduced-motion "
          "block, like every other transitioned element on the site",
          ".language-switcher__summary, .language-switcher__summary::after"
          in css[css.index("@media (prefers-reduced-motion: reduce)"):])


def check_i18n_status() -> None:
    print("i18n_status")
    import i18n_status

    tmp_root = Path(tempfile.mkdtemp(prefix="tbaguette-i18n-status-test-"))
    try:
        skills_root = tmp_root / "skills"
        _write_skill(skills_root, "alpha", "Alpha description.", "Alpha body.")
        _write_skill(skills_root, "beta", "Beta description.", "Beta body.")
        _write_skill(skills_root, "gamma", "Gamma description.", "Gamma body.")

        i18n_root = tmp_root / "i18n"
        fr_dir = i18n_root / "fr"
        (fr_dir / "skills" / "alpha").mkdir(parents=True)
        (fr_dir / "skills" / "alpha" / "SKILL.md").write_text(
            "---\nname: alpha\ndescription: Alpha en français.\n---\nCorps.\n", encoding="utf-8"
        )
        (fr_dir / "ui.json").write_text("{}", encoding="utf-8")

        fr = locales.get_locale("fr")
        status = i18n_status.locale_status(fr, skills_root, i18n_root)
        check("counts one of three skills translated", status["translated_count"] == 1)
        check("total_count reflects the real skill corpus size", status["total_count"] == 3)
        check("missing_slugs lists exactly the two untranslated skills",
              set(status["missing_slugs"]) == {"beta", "gamma"})
        check("has_ui_json is True (the file exists, even though it's an empty stub here)",
              status["has_ui_json"] is True)
        check("has_categories_json is False (never created for this fixture)",
              status["has_categories_json"] is False)

        es = locales.get_locale("es")
        es_status = i18n_status.locale_status(es, skills_root, i18n_root)
        check("a locale with no i18n/ directory at all reports zero translated, not an error",
              es_status["translated_count"] == 0 and es_status["has_ui_json"] is False)
    finally:
        shutil.rmtree(tmp_root, ignore_errors=True)


def check_real_i18n_content_key_parity() -> None:
    print("real i18n/ content key parity")
    project_root = Path(__file__).resolve().parent.parent
    i18n_root = project_root / "i18n"
    if not i18n_root.is_dir():
        print("  (no i18n/ directory yet -- nothing to check)")
        return

    ui_keys = {f.name for f in dataclasses.fields(templates.Strings)}
    verify_keys = {f.name for f in dataclasses.fields(templates.VerifyInstallStrings)}
    real_skill_slugs = set(content_pipeline.list_skill_slugs(project_root / "skills"))
    real_category_slugs = {c["slug"] for c in content_pipeline.CATEGORIES}

    for locale_dir in sorted(p for p in i18n_root.iterdir() if p.is_dir()):
        code = locale_dir.name

        ui_json_path = locale_dir / "ui.json"
        if ui_json_path.is_file():
            data = json.loads(ui_json_path.read_text(encoding="utf-8"))
            check(f"i18n/{code}/ui.json has exactly Strings' key set (no missing, no extra)",
                  set(data.keys()) == ui_keys)

        verify_json_path = locale_dir / "verify-install.json"
        if verify_json_path.is_file():
            data = json.loads(verify_json_path.read_text(encoding="utf-8"))
            check(f"i18n/{code}/verify-install.json has exactly VerifyInstallStrings' key set",
                  set(data.keys()) == verify_keys)

        descriptions_path = locale_dir / "descriptions.json"
        if descriptions_path.is_file():
            data = json.loads(descriptions_path.read_text(encoding="utf-8"))
            check(f"i18n/{code}/descriptions.json has no unknown skill slugs",
                  set(data.keys()) <= real_skill_slugs)

        categories_path = locale_dir / "categories.json"
        if categories_path.is_file():
            data = json.loads(categories_path.read_text(encoding="utf-8"))
            check(f"i18n/{code}/categories.json has no unknown category slugs",
                  set(data.keys()) <= real_category_slugs)


def main() -> None:
    check_locale_registry()
    check_full_locale_build()
    check_locale_count_gate()
    check_rtl_css_pass()
    check_language_switcher_css()
    check_i18n_status()
    check_real_i18n_content_key_parity()
    print(f"\n{checker.total} checks passed.")


if __name__ == "__main__":
    main()
