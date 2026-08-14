"""Locale registry for the tbaguette-skills site's i18n build.

The single source of truth for which 16 locales (English + 15 translations,
one language per country) the site builds. scripts/generate.py,
scripts/templates.py's language switcher, and scripts/test_i18n.py all
import LOCALES from here rather than each declaring the list separately.

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
    Locale(code="fr", hreflang="fr", name="French", endonym="Français", dir="ltr"),
    Locale(code="es", hreflang="es", name="Spanish", endonym="Español", dir="ltr"),
    Locale(code="de", hreflang="de", name="German", endonym="Deutsch", dir="ltr"),
    Locale(code="it", hreflang="it", name="Italian", endonym="Italiano", dir="ltr"),
    Locale(code="pt", hreflang="pt-BR", name="Portuguese", endonym="Português", dir="ltr"),
    Locale(code="ru", hreflang="ru", name="Russian", endonym="Русский", dir="ltr"),
    Locale(code="zh", hreflang="zh-Hans", name="Chinese", endonym="中文", dir="ltr"),
    Locale(code="ja", hreflang="ja", name="Japanese", endonym="日本語", dir="ltr"),
    Locale(code="ko", hreflang="ko", name="Korean", endonym="한국어", dir="ltr"),
    Locale(code="hi", hreflang="hi", name="Hindi", endonym="हिन्दी", dir="ltr"),
    Locale(code="ar", hreflang="ar", name="Arabic", endonym="العربية", dir="rtl"),
    Locale(code="tr", hreflang="tr", name="Turkish", endonym="Türkçe", dir="ltr"),
    Locale(code="vi", hreflang="vi", name="Vietnamese", endonym="Tiếng Việt", dir="ltr"),
    Locale(code="pl", hreflang="pl", name="Polish", endonym="Polski", dir="ltr"),
    Locale(code="id", hreflang="id", name="Indonesian", endonym="Indonesia", dir="ltr"),
)

EXPECTED_LOCALE_COUNT = 16

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
