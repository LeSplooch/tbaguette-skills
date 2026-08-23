"""Locale registry for the tbaguette-skills site's build.

The single source of truth for which locale(s) the site builds. English
only, as of 2026-08-23: the site briefly carried 11 translations (French,
Spanish, German, Italian, Portuguese, Chinese, Japanese, Korean, Hindi,
Arabic, Turkish) alongside three that never got past an empty switcher
entry (Vietnamese, Polish, Indonesian, dropped 2026-08-15) and one dropped
mid-life after shipping complete (Russian, 2026-08-20). All 11 remaining
translations were removed 2026-08-23 -- the per-skill body translation
effort (i18n/<code>/skills/*) turned out too expensive in tokens to
run, and rather than leave a half-finished translation layer live, the
whole i18n build (chrome strings, descriptions, categories, skill bodies)
was reverted along with it. The history above is worth keeping: re-adding
i18n later means re-running the same procedure documented in
docs/superpowers/plans/2026-08-14-website-i18n.md and
docs/superpowers/specs/2026-08-14-website-i18n-design.md, not reinventing
it -- add Locale entries back, bump EXPECTED_LOCALE_COUNT, populate
i18n/<code>/. scripts/generate.py, scripts/templates.py's language
switcher, and scripts/test_i18n.py all import LOCALES from here rather
than each declaring the list separately.

    python3 scripts/locales.py    # prints the registry as a sanity check
"""

from __future__ import annotations

from dataclasses import dataclass


@dataclass(frozen=True)
class Locale:
    """One locale the site builds.

    code: the URL prefix (e.g. "fr" -> /fr/...) and the i18n/<code>/
        directory name. English's code is "en" but it is never used as a
        URL prefix -- English stays at the true root, see generate.py.
    hreflang: the IETF language tag for <link rel="alternate" hreflang>,
        more specific than `code` where it matters for search engines
        (zh -> zh-Hans, pt -> pt-BR) even though the URL itself stays the
        short `code`.
    name: the English name of the language, for admin/log output.
    endonym: the language's name in its own script, e.g. "Français" -- what
        the language switcher displays, since a switcher listing "Chinese,
        French, German" in English defeats the point of a switcher.
    dir: "ltr" or "rtl", the value of the page's <html dir> attribute.
    default: True for exactly one locale (English) -- the one that builds
        at the site root instead of under /<code>/.
    """

    code: str
    hreflang: str
    name: str
    endonym: str
    dir: str
    default: bool = False


LOCALES: tuple[Locale, ...] = (
    Locale(code="en", hreflang="en", name="English", endonym="English", dir="ltr", default=True),
)

EXPECTED_LOCALE_COUNT = 1

DEFAULT_LOCALE: Locale = next(locale for locale in LOCALES if locale.default)


def get_locale(code: str) -> Locale:
    for locale in LOCALES:
        if locale.code == code:
            return locale
    raise KeyError(f"unknown locale code {code!r}")


def main() -> None:
    for locale in LOCALES:
        marker = " (default)" if locale.default else ""
        print(f"{locale.code:>3}  {locale.hreflang:<8} {locale.name:<12} {locale.endonym:<10} {locale.dir}{marker}")


if __name__ == "__main__":
    main()
