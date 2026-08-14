"""Per-locale translation coverage report — how much of phase 2 (skill
body translation) is done for each of the 15 non-English locales, so
resuming that work across sessions doesn't require re-deriving state.

    python3 scripts/i18n_status.py
"""

from __future__ import annotations

from pathlib import Path

import locales
from content_pipeline import list_skill_slugs


def _project_root() -> Path:
    return Path(__file__).resolve().parent.parent


def locale_status(locale: locales.Locale, skills_root: Path, i18n_root: Path) -> dict:
    """Coverage for one locale: how many of the real skill corpus's slugs
    have a real translated SKILL.md under i18n/<code>/skills/<slug>/, plus
    which of the four small chrome/description JSON files exist. Works
    correctly even when i18n_root / locale.code doesn't exist on disk at
    all yet (a brand-new, wholly-untranslated locale) -- every check below
    is a plain Path.is_file(), which is False for a missing parent
    directory too, not an exception."""
    slugs = list_skill_slugs(skills_root)
    locale_dir = i18n_root / locale.code
    translated_slugs = [
        slug for slug in slugs
        if (locale_dir / "skills" / slug / "SKILL.md").is_file()
    ]
    return {
        "code": locale.code,
        "name": locale.name,
        "translated_count": len(translated_slugs),
        "total_count": len(slugs),
        "missing_slugs": [slug for slug in slugs if slug not in translated_slugs],
        "has_ui_json": (locale_dir / "ui.json").is_file(),
        "has_categories_json": (locale_dir / "categories.json").is_file(),
        "has_descriptions_json": (locale_dir / "descriptions.json").is_file(),
        "has_verify_install_json": (locale_dir / "verify-install.json").is_file(),
    }


def main() -> None:
    project_root = _project_root()
    skills_root = project_root / "skills"
    i18n_root = project_root / "i18n"

    print(f"{'code':<5}{'chrome':<20}{'skill bodies':<14}name")
    print("-" * 60)
    for locale in locales.LOCALES:
        if locale.default:
            continue
        status = locale_status(locale, skills_root, i18n_root)
        chrome = "/".join([
            "ui" if status["has_ui_json"] else "--",
            "cat" if status["has_categories_json"] else "--",
            "desc" if status["has_descriptions_json"] else "--",
            "verify" if status["has_verify_install_json"] else "--",
        ])
        bodies = f"{status['translated_count']}/{status['total_count']}"
        print(f"{status['code']:<5}{chrome:<20}{bodies:<14}{status['name']}")


if __name__ == "__main__":
    main()
