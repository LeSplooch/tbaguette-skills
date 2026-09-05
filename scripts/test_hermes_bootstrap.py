"""Tests for the Hermes Agent bootstrap (root __init__.py).

This suite exists because of a failure that every other suite in this repo was
structurally unable to see. `hermes plugins install LeSplooch/tbaguette-skills`
aborted outright, and the bootstrap behind it had never run once, while every
manifest here stayed valid and every version stayed in sync. The checks below
are the parts of that integration a repo-local test *can* hold — the ones that
depend on this repository's own files staying in step with each other.

What none of them prove is that Hermes still behaves the way the module's
comments say it does. Those facts came from reading hermes-agent's source on
2026-09-05 (`plugins_cmd.py`, `plugins_discovery.py`, `plugins_loader.py`,
`plugins.py`, `tools/hook_output_spill.py`) and from driving a real install.
Re-check them against a live instance, not against this file.
"""

from __future__ import annotations

import importlib.util
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
SKILL_MD = REPO_ROOT / "skills" / "using-tbaguette" / "SKILL.md"

# Hermes' documented default for hooks.output_spill.max_chars. Past it, a hook's
# context is replaced by a 500-char head, a 500-char tail and a file path.
HERMES_DEFAULT_SPILL_MAX_CHARS = 10_000

failures = []
passed = 0


def check(label, condition):
    global passed
    print(f"  {'ok ' if condition else 'FAIL'} {label}")
    if condition:
        passed += 1
    else:
        failures.append(label)


def load_plugin_module():
    """Load the root __init__.py by path.

    It cannot be imported by name: the package would be `tbaguette-skills`,
    and a hyphen is not a Python identifier. Hermes loads it the same way,
    via spec_from_file_location.
    """
    spec = importlib.util.spec_from_file_location(
        "tbaguette_hermes_plugin", REPO_ROOT / "__init__.py"
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class FakeContext:
    """Stands in for Hermes' PluginContext, with its two real constraints:
    register_skill takes a pathlib.Path (a str raises AttributeError there and
    hermes silently disables the whole plugin), and it rejects a name carrying
    the namespace separator, because it applies the namespace itself."""

    def __init__(self):
        self.skills = {}
        self.hooks = {}

    def register_skill(self, name, path, description="", frontmatter=None):
        assert isinstance(path, Path), f"register_skill needs a Path, got {type(path)}"
        assert ":" not in name, f"Hermes rejects a namespaced skill name: {name!r}"
        assert path.is_file(), f"no SKILL.md at {path}"
        self.skills[f"TBaguette:{name}"] = path

    def register_hook(self, event, callback):
        self.hooks[event] = callback


def main():
    plugin = load_plugin_module()
    print("hermes bootstrap")

    # --- the section split has to stay exhaustive -------------------------
    # A new `## ` section in SKILL.md is a decision: does it ride in the
    # injection, or is it left for skill_view? Neither answer is safe to reach
    # by default, so the test fails until someone picks one.
    body = re.sub(r"^---\n[\s\S]*?\n---\n", "", SKILL_MD.read_text(encoding="utf-8"))
    headings = {
        part.split("\n", 1)[0][3:].strip()
        for part in re.split(r"(?m)^(?=## )", body)[1:]
    }
    classified = set(plugin.BOOTSTRAP_SECTIONS) | set(plugin.OMITTED_SECTIONS)
    check(
        "every SKILL.md section is either carried or deliberately omitted "
        f"(unclassified: {sorted(headings - classified)})",
        headings <= classified,
    )
    check(
        "the plugin names no section SKILL.md no longer has "
        f"(stale: {sorted(classified - headings)})",
        classified <= headings,
    )

    # --- the size guarantee ----------------------------------------------
    skills_dir = plugin._skills_dir()
    bootstrap = plugin._build_bootstrap(skills_dir)
    check(
        f"bootstrap fits under Hermes' default spill limit "
        f"({len(bootstrap)} <= {HERMES_DEFAULT_SPILL_MAX_CHARS})",
        len(bootstrap) <= HERMES_DEFAULT_SPILL_MAX_CHARS,
    )
    check(
        "...with the margin the module reserves, so a small edit to SKILL.md "
        "does not land right on the line",
        len(bootstrap) <= plugin._spill_budget(),
    )

    # A tightened hooks.output_spill.max_chars must shed content, never spill.
    # Spilling the *minimal* bootstrap is the worst case there is: what is left
    # by then is all load-bearing.
    original = plugin._spill_budget
    try:
        for limit in (8_000, 6_000, 4_000, 3_000):
            plugin._spill_budget = lambda lim=limit: max(lim - plugin.SPILL_MARGIN, 1_500)
            trimmed = plugin._build_bootstrap(skills_dir)
            check(
                f"a max_chars of {limit} sheds sections instead of spilling "
                f"({len(trimmed)} chars)",
                len(trimmed) <= limit,
            )
    finally:
        plugin._spill_budget = original

    # --- what the injected text must and must not say --------------------
    # The bug this replaced was not the truncation. It was that the truncated
    # text still said "do not try to load using-tbaguette again" -- forbidding
    # the one recovery available -- while 95% of the skill was missing.
    check(
        "the bootstrap never forbids loading the skill it is summarizing",
        "do not try to load" not in bootstrap.lower(),
    )
    check(
        "it says how to read the rest",
        'skill_view("TBaguette:using-tbaguette")' in bootstrap,
    )
    check(
        "it names what it left out rather than dropping it silently",
        "left out of this injection" in bootstrap,
    )
    for omitted, reason in plugin.OMITTED_SECTIONS.items():
        check(f"...naming {omitted!r} and why", omitted in bootstrap and reason in bootstrap)
    check(
        "it carries the namespace rule, which is what makes a lookup resolve",
        "`TBaguette:` prefix is not decoration" in bootstrap,
    )
    check(
        "it carries this harness's tool mapping",
        "Hermes Agent Tool Mapping" in bootstrap,
    )
    check("it is framed for the model", bootstrap.startswith("<EXTREMELY_IMPORTANT>"))
    check("...and closed", bootstrap.rstrip().endswith("</EXTREMELY_IMPORTANT>"))

    # --- the per-turn nudge ----------------------------------------------
    # Session start alone is measurably not enough (~95% injected, ~42% of
    # sessions ever invoking a skill), which is why every harness with a
    # per-turn seam gets one. Hermes' is pre_llm_call on turns after the first.
    check(
        "the nudge names Hermes' own invocation form, not the Skill tool",
        'skill_view("TBaguette:<skill-name>")' in plugin.NUDGE
        and "Skill tool" not in plugin.NUDGE,
    )
    check(
        "the nudge is short enough to ride every turn",
        len(plugin.NUDGE) < 600,
    )

    # --- registration ----------------------------------------------------
    ctx = FakeContext()
    plugin.register(ctx)
    on_disk = {p.name for p in (REPO_ROOT / "skills").iterdir() if (p / "SKILL.md").is_file()}
    check(
        f"every skill on disk is registered ({len(ctx.skills)} of {len(on_disk)})",
        {f"TBaguette:{name}" for name in on_disk} == set(ctx.skills),
    )
    check("the pre_llm_call hook is registered", "pre_llm_call" in ctx.hooks)

    hook = ctx.hooks["pre_llm_call"]
    first = hook(is_first_turn=True, user_message="hi", conversation_history=None)
    later = hook(is_first_turn=False, user_message="hi", conversation_history=[1, 2])
    check("first turn gets the bootstrap", first["context"] == bootstrap)
    check("every later turn gets the nudge", later["context"] == plugin.NUDGE)
    check(
        "no turn gets nothing -- the seam fires once per user turn, so a None "
        "here is a turn with no reminder at all",
        first.get("context") and later.get("context"),
    )

    print()
    if failures:
        print(f"{len(failures)} check(s) FAILED:")
        for f in failures:
            print(f"  - {f}")
        sys.exit(1)
    print(f"{passed} checks passed.")


if __name__ == "__main__":
    main()
