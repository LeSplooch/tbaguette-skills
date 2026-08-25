"""Every skill a skill points at must exist.

A cross-reference in this library is not decoration: the site turns it into
a link (see content_pipeline.make_skill_mention_resolver), and an agent
reading the skill treats it as a routing instruction. A reference to a skill
that isn't here therefore fails twice over -- it renders as dead plain text
for a human, and sends an agent looking for something that was never
installed.

That happened: `tracing-data-flow` shipped a "Not for: ... (systematic-debugging)"
pointer for months. `systematic-debugging` is Superpowers' name for what this
library calls `diagnosing-before-fixing`, and it survived the parity work
because it was written bare rather than in backticks -- so a grep for
backticked skill names never saw it, and nothing else was looking.

The rule enforced here is deliberately narrow, because skill names and
ordinary hyphenated English are the same shape ("red-green-refactor",
"rate-limited", "copy-on-write" are all prose): a parenthetical whose
*entire* content is a slug-shaped token, optionally prefixed by "see", is a
skill reference and must resolve. That matches 120 real references across the
corpus and needs only the two exemptions below.
"""
from __future__ import annotations

import re
import unittest
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SKILLS_DIR = REPO_ROOT / "skills"

_REFERENCE = re.compile(r"\((?:see\s+)?`?([a-z][a-z0-9]*(?:-[a-z0-9]+)+)`?\)")
_FENCED = re.compile(r"```.*?```", re.S)

# Parentheticals that match the shape but are ordinary prose, not references.
# Keep this list short: every addition is a place the guard stops looking.
NOT_SKILL_REFERENCES = {
    "e-ink",        # formidable/reference/stacks/embedded-display.md, a display type
    "state-level",  # threat-modeling/SKILL.md, an attacker tier
}

# Skills belonging to other plugins, referenced on purpose to show a handoff.
FOREIGN_SKILLS = {"using-superpowers"}


def _skill_slugs() -> set[str]:
    return {p.name for p in SKILLS_DIR.iterdir() if p.is_dir()}


class TestSkillReferencesResolve(unittest.TestCase):
    def test_every_parenthetical_reference_names_a_real_skill(self):
        known = _skill_slugs() | FOREIGN_SKILLS | NOT_SKILL_REFERENCES
        dangling = []
        for path in sorted(SKILLS_DIR.rglob("*.md")):
            text = _FENCED.sub("", path.read_text(encoding="utf-8"))
            for match in _REFERENCE.finditer(text):
                if match.group(1) not in known:
                    dangling.append(f"{path.relative_to(REPO_ROOT)}: ({match.group(1)})")
        self.assertEqual(dangling, [], "reference(s) to a skill that does not exist")

    def test_no_skill_directory_is_unreachable_from_another_skill(self):
        """A skill nothing links to is one a reader only finds by already
        knowing its name.

        This is deliberately zero-tolerance, so a newly added skill turns the
        suite red until some existing skill points at it. That is the intended
        forcing function, not a bug in the test: a skill worth adding is a
        skill some neighbour's "Not for:" line should be redirecting to."""
        slugs = _skill_slugs()
        bare = re.compile(r"(?<![\w/-])([a-z0-9]+(?:-[a-z0-9]+)+)(?![\w/-])")
        incoming = {slug: 0 for slug in slugs}
        for source in slugs:
            text = "\n".join(
                p.read_text(encoding="utf-8")
                for p in sorted((SKILLS_DIR / source).rglob("*.md"))
            )
            text = re.sub(r"^---.*?^---\n", "", text, flags=re.S | re.M)
            for target in set(bare.findall(text)) | set(re.findall(r"`([a-z0-9-]+)`", text)):
                if target in incoming and target != source:
                    incoming[target] += 1
        self.assertEqual(sorted(s for s, n in incoming.items() if n == 0), [])

    def test_the_exemption_lists_stay_honest(self):
        """An exemption that becomes a real skill is a silenced check."""
        slugs = _skill_slugs()
        for exempt in NOT_SKILL_REFERENCES | FOREIGN_SKILLS:
            with self.subTest(exempt=exempt):
                self.assertNotIn(exempt, slugs)


if __name__ == "__main__":
    unittest.main(verbosity=2)
