"""Every skill's frontmatter stays inside the Agent Skills platform limits.

`name` and `description` are not this repo's own fields. They are the two
required keys of the Agent Skills format, and the platform documents hard
validation rules for both: `name` at most 64 characters and confined to
lowercase letters, numbers and hyphens; `description` non-empty and at most
1024 characters; neither containing XML tags; `name` avoiding the reserved
words "anthropic" and "claude".

Nothing in this repo used to check any of that, and two descriptions had
drifted past the 1024-character limit by the time anyone measured -- 1107
and 1037. The drift is invisible from inside the repo because the site
renders a description regardless of length and Claude Code loads it anyway,
so the only surface that would ever have complained is the one furthest
from the author: another harness, or the API's own skills upload. Every
manifest this repo ships is another place a limit it never enforced itself
can be enforced against it.

Worth recording, because it nearly became the fix: the first measurement of
this said three skills were over, and it was wrong. The throwaway regex
behind it keyed continuation lines off `\w+:`, which does not match
`user-invocable:` -- so two later frontmatter fields were swallowed into
one skill's description and inflated it past the limit it was actually
under. The parser below keys off `[A-Za-z_][A-Za-z0-9_-]*:` instead. A
measurement worth acting on is worth writing a second one against.

The count is the part that needs watching rather than the charset, because
a description is the one field with continuous pressure to grow: every new
routing trigger someone wants the skill to fire on gets appended to it.
This suite is the thing that says when the next trigger has to displace an
older one instead of joining it.
"""
from __future__ import annotations

import re
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SKILLS_DIR = REPO_ROOT / "skills"

# Platform limits, not house style. Sources:
# https://platform.claude.com/docs/en/agents-and-tools/agent-skills/best-practices
MAX_NAME = 64
MAX_DESCRIPTION = 1024
RESERVED_WORDS = ("anthropic", "claude")

_FRONTMATTER = re.compile(r"\A---\n(.*?)\n---\n", re.S)
_NAME_CHARSET = re.compile(r"\A[a-z0-9]+(?:-[a-z0-9]+)*\Z")
_XML_TAG = re.compile(r"<[A-Za-z/][^>]*>")


def _frontmatter(path: Path) -> dict[str, str]:
    """Pull `name` and `description` out of a SKILL.md's YAML block.

    Deliberately not a YAML parser: this repo is stdlib-only, and the two
    fields it needs are both plain scalars that may wrap across continuation
    lines. A continuation is any line that does not open a new `key:`.
    """
    match = _FRONTMATTER.match(path.read_text(encoding="utf-8"))
    if match is None:
        return {}
    fields: dict[str, list[str]] = {}
    current: str | None = None
    for line in match.group(1).split("\n"):
        key = re.match(r"([A-Za-z_][A-Za-z0-9_-]*):\s?(.*)", line)
        if key and not line.startswith((" ", "\t")):
            current = key.group(1)
            fields[current] = [key.group(2)]
        elif current is not None:
            fields[current].append(line.strip())
    return {k: " ".join(part for part in v if part).strip() for k, v in fields.items()}


def _skills() -> list[tuple[str, dict[str, str]]]:
    out = []
    for directory in sorted(SKILLS_DIR.iterdir()):
        skill_md = directory / "SKILL.md"
        if directory.is_dir() and skill_md.exists():
            out.append((directory.name, _frontmatter(skill_md)))
    return out


class TestSkillFrontmatterLimits(unittest.TestCase):
    def test_every_skill_has_both_required_fields(self):
        missing = [
            slug for slug, fm in _skills()
            if not fm.get("name") or not fm.get("description")
        ]
        self.assertEqual(missing, [], "SKILL.md without a name or a description")

    def test_name_matches_its_directory(self):
        mismatched = [
            f"{slug}: name is {fm.get('name')!r}"
            for slug, fm in _skills() if fm.get("name") != slug
        ]
        self.assertEqual(mismatched, [], "frontmatter name differs from directory name")

    def test_name_within_platform_limits(self):
        bad = []
        for slug, fm in _skills():
            name = fm.get("name", "")
            if len(name) > MAX_NAME:
                bad.append(f"{slug}: {len(name)} chars, limit {MAX_NAME}")
            if not _NAME_CHARSET.match(name):
                bad.append(f"{slug}: not lowercase-hyphen-alphanumeric")
            for word in RESERVED_WORDS:
                if word in name:
                    bad.append(f"{slug}: contains reserved word {word!r}")
        self.assertEqual(bad, [], "name violates a platform constraint")

    def test_description_within_platform_limit(self):
        over = [
            f"{slug}: {len(fm.get('description', ''))} chars, limit {MAX_DESCRIPTION}"
            for slug, fm in _skills()
            if len(fm.get("description", "")) > MAX_DESCRIPTION
        ]
        self.assertEqual(
            over, [],
            "description over the platform's hard limit -- displace an older "
            "routing trigger rather than appending another one",
        )

    def test_no_xml_tags_in_either_field(self):
        bad = [
            f"{slug}: {field}"
            for slug, fm in _skills()
            for field in ("name", "description")
            if _XML_TAG.search(fm.get(field, ""))
        ]
        self.assertEqual(bad, [], "XML tag in frontmatter; the platform rejects it")


if __name__ == "__main__":
    unittest.main(verbosity=2)
