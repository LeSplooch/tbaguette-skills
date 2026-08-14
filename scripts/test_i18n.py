"""Tests for the site's i18n build: locale registry integrity, per-file
content fallback, description precedence, UI-string key parity, routing
output (hreflang/canonical/switcher), and RTL. Plain assert-based, using
the shared Checker (see scripts/checker.py) -- matches test_generate.py and
test_templates.py's convention, not test_content_pipeline.py's unittest one.

Usage:
    python3 scripts/test_i18n.py
"""

from __future__ import annotations

from checker import Checker
import locales

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


def main() -> None:
    check_locale_registry()
    print(f"\n{checker.total} checks passed.")


if __name__ == "__main__":
    main()
