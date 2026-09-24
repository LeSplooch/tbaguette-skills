"""Tests for scripts/copilot_agents.py and the agents it commits under
com.github.copilot/agents/: that the committed files are exactly what the
renderer produces (an agent that drifted from delegating-tasks-with-review-gates'
templates is a reviewer holding last month's rules), that each one is a
well-formed Copilot agent profile, that the read-only roles cannot edit, and
that a template whose shape changes fails the render instead of shipping a hole.
"""
from __future__ import annotations

import re
import unittest
from unittest import mock

import copilot_agents as ca

# Copilot namespaces plugin agents itself: these are offered as TBaguette:implementer etc.
ROLES = ("implementer", "reviewer", "investigator")


def frontmatter(text: str) -> dict[str, str]:
    head = text.split("\n---\n", 1)[0].removeprefix("---\n")
    return dict(re.match(r"([\w-]+):\s*(.*)", line).groups() for line in head.split("\n") if line.strip())


class TestCommittedAgents(unittest.TestCase):
    def test_committed_agents_match_the_renderer(self):
        self.assertEqual(ca.main(["--check"]), 0,
                         "com.github.copilot/agents/ is stale: run python3 scripts/copilot_agents.py")

    def test_every_role_is_rendered(self):
        self.assertEqual(sorted(ca.render()), sorted(f"{r}.agent.md" for r in ROLES))

    def test_plugin_manifest_points_copilot_cli_at_the_agents(self):
        import json
        manifest = json.loads((ca.ROOT / "plugin.json").read_text(encoding="utf-8"))
        self.assertEqual((ca.ROOT / manifest["agents"]).resolve(), ca.OUT.resolve())

    def test_frontmatter_names_match_files_and_describe_the_role(self):
        for name, text in ca.render().items():
            fm = frontmatter(text)
            self.assertEqual(fm.get("name"), name.removesuffix(".agent.md"))
            self.assertIn("TBaguette", fm.get("description", ""))
            self.assertIn("Does not", fm.get("description", ""), f"{name}: description must state its limits")

    def test_read_only_roles_have_no_edit_tool(self):
        for role in ("reviewer", "investigator"):
            tools = frontmatter(ca.render()[f"{role}.agent.md"])["tools"]
            self.assertNotIn('"edit"', tools, f"{role} must not be able to edit")

    def test_no_role_pins_a_model(self):
        # The controller names a model per dispatch ("Choosing a model for each role");
        # a pinned id that a user's plan does not offer would break the agent outright.
        for name, text in ca.render().items():
            self.assertNotIn("model", frontmatter(text), name)

    def test_template_text_is_embedded_not_paraphrased(self):
        agents = ca.render()
        toks = ca.tokens()
        self.assertIn(toks["{{IMPLEMENTER_DISCIPLINE}}"], agents["implementer.agent.md"])
        self.assertIn(toks["{{REVIEWER_SHARED}}"], agents["reviewer.agent.md"])
        for text in agents.values():
            self.assertIn("Edit those, not this file", text)
            self.assertIsNone(re.search(r"\{\{[A-Z_]+\}\}", text))


class TestTemplateShapeChanges(unittest.TestCase):
    def test_missing_prompt_block_fails_loudly(self):
        with self.assertRaises(ca.TemplateShapeError):
            ca.dispatch_prompt("description: x\nmodel: y", "t")

    def test_implementer_template_without_its_anchor_heading_fails(self):
        real = ca.fenced_blocks

        def cut(text):
            return [b.replace("## You Do Not Dispatch Subagents", "## Renamed") for b in real(text)]

        with mock.patch.object(ca, "fenced_blocks", side_effect=cut):
            with self.assertRaises(ca.TemplateShapeError):
                ca.tokens()


if __name__ == "__main__":
    unittest.main(verbosity=2)
