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


# Vendored from hermes-agent's tools/skills_guard.py (`_shell_write_re`,
# `_AGENT_CONFIG_FILES` and friends), read on 2026-09-05. Hermes scans a plugin
# tree before installing it and a single `critical` finding is a `dangerous`
# verdict, which is a hard block -- `--force` does not override it and the only
# escape is the user turning their own scanner off.
#
# The pattern is meant to catch a real persistence attack: `echo evil >
# CLAUDE.md`. It reads any tag-closing `>` as that redirect, so an inline-code
# span wrapping one of these filenames looks like one -- this comment cannot
# show you the sequence without tripping the check itself. That is why the
# library writes agent-config filenames as plain prose rather than in
# backticks. It is a genuinely odd rule and it is load-bearing: five backticked
# filenames across two skills were enough to make TBaguette uninstallable on
# Hermes, silently, with every other check in this repo green.
#
# If Hermes fixes the false positive, delete this check rather than working
# around it further.
_CONFIG_FILE_GROUPS = {
    "agent-config": r"(?:AGENTS\.md|CLAUDE\.md|\.cursorrules|\.clinerules)",
    "Hermes-config": r"\.hermes/(?:config\.yaml|SOUL\.md)",
    "other agents' config": r"\.(?:claude/settings|codex/config)[\w.]*",
}


def _shell_write_re(file_alt: str) -> str:
    return (
        rf'(?:>>|[\w"\'`)\]]\s*>)\s*[~\w./-]*{file_alt}(?!\.?\w)'
        rf'|\bsed\b[^\n]*\s(?:-[A-Za-z]*i[A-Za-z]*|--in-place)\b[^\n]*{file_alt}(?!\.?\w)'
        rf'|\btee\s+(?:-a\s+)?[~\w./"\'-]*{file_alt}(?!\.?\w)'
        rf"|\b(?:cp|mv)\s+[^\s|;&]+\s+[^\n|;&]{{0,40}}?{file_alt}(?!\.?\w)"
    )


MODIFY_VERB_RE = (
    r"(?:\bwrit(?:e|es|ing)\b|\bwritten\b|\bedit(?:s|ed|ing)?\b"
    r"|\bmodif(?:y|ies|ied|ying|ication)s?\b|\bupdat(?:e|es|ed|ing)\b"
    r"|\bappend(?:s|ed|ing)?\b|\bprepend(?:s|ed|ing)?\b"
    r"|\binject(?:s|ed|ing)?\b|\boverwrit(?:e|es|ing)\b|\boverwritten\b"
    r"|\badd\s+to\b|\bchang(?:e|es|ed|ing)\b)"
)


def _prose_modify_re(file_alt: str) -> str:
    """Modification intent aimed at a config file: an imperative-position verb
    (line start or bullet) within 80 comma-free characters of the filename, or a
    mid-line verb behind a directive marker. Descriptive prose misses, which is
    why re-wrapping a line is usually enough to clear a false positive."""
    return (
        rf"^\s*(?:[-*+]\s+|\d+[.)]\s+)?{MODIFY_VERB_RE}[^\n,]{{0,80}}?{file_alt}\b"
        rf"|(?:\byou\s+(?:must|should|need\s+to)\s+|\bplease\s+"
        rf"|\bmake\s+sure\s+(?:to\s+|you\s+)|\bbe\s+sure\s+to\s+)"
        rf"{MODIFY_VERB_RE}[^\n,]{{0,80}}?{file_alt}\b"
    )


# What Hermes' own `_walk` skips. Everything else in the tree is scanned --
# including this file, which is why its comments have to watch their wording.
EXCLUDED_DIRS = {
    ".git", "__pycache__", "node_modules", ".venv", "venv",
    ".mypy_cache", ".pytest_cache", ".ruff_cache", ".tox",
}


def _scannable_files():
    for path in sorted(REPO_ROOT.rglob("*")):
        rel = path.relative_to(REPO_ROOT)
        if path.is_file() and not any(part in EXCLUDED_DIRS for part in rel.parts):
            yield path, rel


def check_scanner_criticals():
    """Nothing in the tree may read to Hermes as a write into a config file.

    Run over the whole repository rather than over `skills/`, because that is
    what Hermes scans: the built pages under `docs/` are the usual offender, but
    an update note, a reference file, or a test comment counts the same.
    """
    print("hermes install scanner (no critical findings)")
    files = [
        (path, rel, path.read_text(encoding="utf-8", errors="replace"))
        for path, rel in _scannable_files()
    ]
    checks = [
        (f"a shell write into {label} files", _shell_write_re(alt), 0)
        for label, alt in _CONFIG_FILE_GROUPS.items()
    ]
    # Only the agent-config group scores prose intent as critical; Hermes' own
    # config is `high` there, because setup docs routinely say "edit config.yaml".
    checks.append(
        ("an instruction to modify agent-config files",
         _prose_modify_re(_CONFIG_FILE_GROUPS["agent-config"]), re.M)
    )
    for label, source, flags in checks:
        pattern = re.compile(source, flags)
        hits = [
            f"{rel}: {m.group(0)[:40]!r}"
            for _, rel, text in files
            for m in pattern.finditer(text)
        ]
        check(
            f"nothing in the tree reads as {label}"
            + (f" -- found {len(hits)}: {hits[:3]}" if hits else ""),
            not hits,
        )


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

    check_scanner_criticals()

    print()
    if failures:
        print(f"{len(failures)} check(s) FAILED:")
        for f in failures:
            print(f"  - {f}")
        sys.exit(1)
    print(f"{passed} checks passed.")


if __name__ == "__main__":
    main()
