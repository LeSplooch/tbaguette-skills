"""Tests for the locale-aware build machinery that survived the
2026-08-23 i18n revert: locale registry integrity, the multi-locale build
mechanism (exercised via a synthetic locale set, since the real registry
is English-only), the EXPECTED_LOCALE_COUNT gate, remaining RTL-neutral
CSS, and confirmation that i18n/ and its now-content-free tooling are
actually gone rather than left dormant. Locale-aware *rendering* itself
(routing, fallback banners, RTL dir handling) is still covered in
test_templates.py's check_i18n_* functions, which also switched to
synthetic locales for the same reason. Plain assert-based, using the
shared Checker (see scripts/checker.py) -- matches test_generate.py and
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
    check("get_locale('en') returns the English entry", locales.get_locale("en").endonym == "English")

    threw = False
    try:
        locales.get_locale("xx")
    except KeyError:
        threw = True
    check("get_locale raises KeyError for an unknown code", threw)

    threw_fr = False
    try:
        locales.get_locale("fr")
    except KeyError:
        threw_fr = True
    check("get_locale('fr') also raises KeyError -- proves the 2026-08-23 "
          "i18n revert actually removed it from the registry, not just its "
          "content on disk", threw_fr)


def _write_skill(skills_root: Path, slug: str, description: str, body: str) -> None:
    skill_dir = skills_root / slug
    skill_dir.mkdir(parents=True, exist_ok=True)
    (skill_dir / "SKILL.md").write_text(
        f"---\nname: {slug}\ndescription: {description}\n---\n{body}\n", encoding="utf-8"
    )


def check_full_locale_build() -> None:
    """The real registry is English-only as of the 2026-08-23 i18n revert,
    so this mocks locales.LOCALES (+ the EXPECTED_LOCALE_COUNT gate it must
    match) to a synthetic 3-locale set instead of using the real one --
    the multi-locale build machinery itself wasn't removed, only its
    content and its live consumers (switcher, hreflang), and this is what
    keeps that machinery under real regression coverage for the day it's
    needed again."""
    print("full locale build (synthetic fixture)")
    tmp_root = Path(tempfile.mkdtemp(prefix="tbaguette-i18n-build-test-"))
    fake_locales = (
        locales.DEFAULT_LOCALE,
        locales.Locale(code="fr", hreflang="fr", name="French", endonym="Français", dir="ltr"),
        locales.Locale(code="es", hreflang="es", name="Spanish", endonym="Español", dir="ltr"),
    )
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
             mock.patch.object(locales, "LOCALES", fake_locales), \
             mock.patch.object(generate, "EXPECTED_LOCALE_COUNT", len(fake_locales)), \
             mock.patch.object(generate, "_default_i18n_root", lambda root: i18n_root):
            content = generate.generate(tmp_root, skills_root, base_path="")

        docs = tmp_root / "docs"
        check("English index still builds at the root", (docs / "index.html").is_file())
        check("French index builds under docs/fr/", (docs / "fr" / "index.html").is_file())
        check("every other locale's directory also exists (all registered locales always route)",
              all((docs / loc.code).is_dir() for loc in fake_locales if not loc.default))

        fr_index_html = (docs / "fr" / "index.html").read_text(encoding="utf-8")
        check("French index emits dir='ltr' lang='fr'", '<html lang="fr" dir="ltr">' in fr_index_html)
        check("French index's translated hero headline made it through",
              "Un atelier pour votre façon de coder." in fr_index_html)
        check("French index carries no hreflang alternates -- that block was "
              "removed with the rest of phase 1 in the 2026-08-23 revert",
              'rel="alternate" hreflang="' not in fr_index_html)

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
             mock.patch.object(locales, "LOCALES", fake_locales), \
             mock.patch.object(generate, "EXPECTED_LOCALE_COUNT", len(fake_locales)), \
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
    check("the card-arrow and tab-list [dir=\"rtl\"] overrides are gone -- "
          "removed with the rest of RTL support in the 2026-08-23 revert, "
          "since no locale is ever dir=\"rtl\" anymore",
          '[dir="rtl"]' not in css)

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


def check_i18n_directory_gone() -> None:
    """The 2026-08-23 revert's actual content-removal claim, checked
    directly rather than trusted: no i18n/ directory should exist in the
    real repo at all. (i18n_status.py, which used to report per-locale
    phase-2 coverage, was deleted alongside it -- there is no more
    translation effort for it to report on.)"""
    print("i18n directory removed")
    project_root = Path(__file__).resolve().parent.parent
    check("no i18n/ directory exists in the real repo",
          not (project_root / "i18n").is_dir())
    check("i18n_status.py was removed, not left dormant",
          not (project_root / "scripts" / "i18n_status.py").is_file())


def main() -> None:
    check_locale_registry()
    check_full_locale_build()
    check_locale_count_gate()
    check_rtl_css_pass()
    check_i18n_directory_gone()
    print(f"\n{checker.total} checks passed.")


if __name__ == "__main__":
    main()
