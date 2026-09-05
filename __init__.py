"""Hermes Agent plugin entry point for TBaguette's Atelier.

This file and its `plugin.yaml` sit at the repository root rather than in a
`.hermes-plugin/` subdirectory, and that is not a style choice — Hermes looks
for both in exactly one place, the plugin directory root:

- `hermes_cli/plugins_cmd.py:_native_manifest_file` checks only
  `<plugin_dir>/plugin.yaml` (or `.yml`). Anything nested is invisible.
- `hermes_cli/plugins_loader.py:_load_directory_module` loads
  `<plugin_dir>/__init__.py`, where `<plugin_dir>` is the directory the
  manifest was found in. The manifest and the module cannot be separated.

The nesting also had a second, louder cost. With no native manifest at the
root, `_read_manifest_for_install` falls through to the portable Agent Plugins
reader for the root `plugin.json` we ship for GitHub Copilot — and that reader
enforces a lowercase name (`^[a-z0-9][a-z0-9.-]*[a-z0-9]$`). "TBaguette" fails
it, so `hermes plugins install LeSplooch/tbaguette-skills` aborted outright
rather than degrading. A native `plugin.yaml` here is checked first and
shadows the portable manifest for Hermes only; no other harness reads it.
"""

import os
import re
from pathlib import Path

BOOTSTRAP_MARKER = "TBaguette:using-tbaguette bootstrap for hermes"

# Re-asserted on every turn after the first. Deliberately not the whole
# SKILL.md: the first turn already injected that, and every skill's trigger
# description is registered with Hermes' own loader. All this has to do is put
# the check back in front of the model before it starts responding.
#
# Session start alone is measurably not enough — across real sessions the
# start-of-session notice landed ~95% of the time while only ~42% of
# substantive sessions ever invoked a skill, because a single message at
# position zero loses against a long context however forcefully it is worded.
# Wording is kept in step with `hooks/user-prompt-submit`, which does the same
# job on the harnesses that expose a per-prompt shell hook.
#
# `pre_llm_call` is the right seam for it: Hermes runs it once per user turn
# (agent/turn_context.py builds the turn, passing is_first_turn), not once per
# LLM call inside the agentic loop, so this needs no throttling.
NUDGE = (
    "<TBAGUETTE_SKILL_CHECK>\n"
    "Before responding: does a TBaguette skill cover this turn? Questions and "
    '"too small to bother" count — see TBaguette:using-tbaguette. If one '
    "plausibly applies, invoke it — `skill_view(\"TBaguette:<skill-name>\")`. "
    "Triggers are already registered with your skill loader; drop the skill if "
    "it does not fit once you are in it.\n"
    "</TBAGUETTE_SKILL_CHECK>"
)


def _skills_dir() -> str:
    """Locate the stock skills/ tree.

    `hermes plugins install LeSplooch/tbaguette-skills` clones the whole repo
    into the plugin directory, so `skills/` is this module's sibling. The
    parent-directory candidate covers a flattened install that copied the
    plugin files one level down.

    Raises loudly when neither matches — a bootstrap that silently skips is how
    a broken install masquerades as a working one.
    """
    here = os.path.dirname(os.path.realpath(__file__))
    candidates = (
        os.path.realpath(os.path.join(here, "skills")),
        os.path.realpath(os.path.join(here, "..", "skills")),
    )
    for cand in candidates:
        if os.path.isfile(os.path.join(cand, "using-tbaguette", "SKILL.md")):
            return cand
    raise RuntimeError(
        "TBaguette plugin: cannot find the skills/ tree "
        f"(looked at {candidates}). Reinstall with "
        "`hermes plugins install LeSplooch/tbaguette-skills --enable`."
    )


# Hermes spills any single hook's context to disk once it passes
# `hooks.output_spill.max_chars` — 10,000 by default, enabled by default
# (tools/hook_output_spill.py) — and replaces it with a 500-char head, a
# 500-char tail and a path. The whole using-tbaguette SKILL.md plus this
# harness's tool mapping came to ~12,400 chars, so the bootstrap that actually
# reached the model was a stub that still carried the sentence "already loaded
# ... do not try to load using-tbaguette again". Truncation is survivable; a
# truncation that forbids the recovery is not.
#
# So the first-turn injection is assembled to fit, with headroom, and the
# sections left out are named rather than dropped in silence.
DEFAULT_SPILL_MAX_CHARS = 10_000  # tools/hook_output_spill.DEFAULT_MAX_CHARS
SPILL_MARGIN = 600  # room for the omission sentence the trim itself lengthens


def _spill_budget() -> int:
    """The size the injection has to come in under.

    Read from Hermes' own resolved config where possible, because
    `hooks.output_spill.max_chars` is a user setting and a hardcoded 10,000
    would silently spill for anyone who tightened it. Falls back to the
    documented default: this is a private import, and a plugin that refused to
    bootstrap because an internal moved would be a worse failure than a
    slightly stale constant.
    """
    try:
        from tools.hook_output_spill import get_spill_config

        config = get_spill_config()
        if not config.get("enabled", True):
            return 10**9
        limit = int(config.get("max_chars") or DEFAULT_SPILL_MAX_CHARS)
    except Exception:
        limit = DEFAULT_SPILL_MAX_CHARS
    return max(limit - SPILL_MARGIN, 1_500)

# Which of SKILL.md's `## ` sections ride in the injection, and which are left
# for `skill_view`. This is an explicit list rather than a greedy fill because
# the three omissions below are omitted for being *wrong here*, not merely for
# being long — dropping them makes the Hermes bootstrap more accurate, not just
# smaller. test_hermes_bootstrap.py fails if SKILL.md's section set stops
# matching this one, so a new section forces the choice to be made again
# instead of silently defaulting either way.
BOOTSTRAP_SECTIONS = (
    "The rule",
    "A long response needs a second check, not only the first",
    "When the work is bigger than one response",
    "Red flags",
    "Alongside other plugins",
)
OMITTED_SECTIONS = {
    # Its premise is Claude Code's: that every skill's trigger description is
    # already in the context listing. On Hermes it is not — register_skill
    # entries are explicit loads, absent from <available_skills> — so the
    # section's advice (raise a listing budget, set skillOverrides) has no
    # referent here. The Hermes-accurate version is one sentence, below.
    "The listing may be shorter than the library": "does not apply on Hermes",
    # Points at the per-harness reference files. This bootstrap appends the
    # Hermes one inline, so following the pointer would be a round trip to
    # something already in front of you.
    "Platform adaptation": "the Hermes mapping is included below",
    # Describes a block that Claude Code's session-start hook attaches. Hermes
    # attaches none, so there is nothing here to act on.
    "Automatic update check": "Claude Code only",
}


def _strip_frontmatter(content: str) -> str:
    match = re.match(r"^---\n[\s\S]*?\n---\n([\s\S]*)$", content)
    return (match.group(1) if match else content).strip()


def _split_sections(body: str) -> tuple:
    """Split a SKILL.md body into (preamble, {heading: section_text})."""
    parts = re.split(r"(?m)^(?=## )", body)
    preamble = parts[0].strip()
    sections = {}
    for part in parts[1:]:
        heading = part.split("\n", 1)[0][3:].strip()
        sections[heading] = part.strip()
    return preamble, sections


def _build_bootstrap(skills_dir: str) -> str:
    with open(
        os.path.join(skills_dir, "using-tbaguette", "SKILL.md"),
        encoding="utf-8",
    ) as f:
        preamble, sections = _split_sections(_strip_frontmatter(f.read()))

    tools_path = os.path.join(
        skills_dir, "using-tbaguette", "references", "hermes-tools.md"
    )
    with open(tools_path, encoding="utf-8") as f:
        tool_mapping = f.read().strip()

    # BOOTSTRAP_SECTIONS is in SKILL.md order, which is also roughly descending
    # value, so trimming from the end sheds the cheapest first. The loop is what
    # makes the budget a guarantee rather than a hope: SKILL.md and the tool
    # mapping are both edited far more often than this file, and a budget that
    # is only checked by a test in this repo would still spill on a user's
    # machine the moment one of them grew.
    budget = _spill_budget()
    chosen = [h for h in BOOTSTRAP_SECTIONS if h in sections]
    tools = tool_mapping
    while True:
        kept = [preamble] + [sections[h] for h in chosen]
        # Anything SKILL.md grew since this list was written is named too, so
        # the pointer stays true without this file having to be edited first.
        left_out = [h for h in sections if h not in chosen]
        body = _assemble(kept, left_out, skills_dir, tools)
        if len(body) <= budget:
            return body
        if chosen:
            chosen.pop()
        elif tools:
            # Last to go, and only against a budget tightened well below
            # Hermes' default: the mapping is the one part of this that is
            # Hermes-specific, so it is worth more here than any prose. What
            # remains after this is the standing rule and how to load the rest,
            # which is the least that can be called a bootstrap.
            tools = ""
        else:
            return body


def _assemble(kept, left_out, skills_dir, tool_mapping) -> str:
    return (
        f"<EXTREMELY_IMPORTANT>\n"
        f"{BOOTSTRAP_MARKER}\n\n"
        f"You have TBaguette.\n\n"
        f"Most of the using-tbaguette skill is included below and is already "
        f"loaded for this Hermes session. Follow it now.\n\n"
        + "\n\n".join(kept)
        + f"\n\n## Loading TBaguette Skills on Hermes\n\n"
        f"TBaguette's {len(_stock_skill_names(skills_dir))} skills are "
        f"registered with Hermes' native skill loader: invoke one with "
        f'`skill_view("TBaguette:skill-name")` (for example '
        f'`skill_view("TBaguette:naming-things")`). The `TBaguette:` prefix is '
        f"not decoration — Hermes derives it from the plugin name, and the "
        f"bare name will not resolve.\n\n"
        f"These are explicit loads: unlike a Claude Code skill listing, they "
        f"are **not** in `<available_skills>`, so nothing puts a skill's "
        f"trigger description in front of you unasked. `skills_list` is how "
        f"you see what is there, and reaching for it is the Hermes equivalent "
        f"of reading the listing before concluding that nothing covers a "
        f"question. `CATALOG.md` in the skills directory has the long "
        f"descriptions.\n\n"
        f"These sections of using-tbaguette were left out of this injection to "
        f"keep it under Hermes' hook-output limit"
        + (
            " — "
            + "; ".join(
                f"*{h}* ({OMITTED_SECTIONS[h]})"
                if h in OMITTED_SECTIONS
                else f"*{h}*"
                for h in left_out
            )
            if left_out
            else ""
        )
        + f". Read the whole thing with "
        f'`skill_view("TBaguette:using-tbaguette")` whenever you want it; '
        f"nothing here says not to.\n\n"
        f"If a lookup returns 'not found', read the skill file directly "
        f"instead:\n"
        f'`read_file("{skills_dir}/skill-name/SKILL.md")`\n\n'
        f"The TBaguette skills directory is: `{skills_dir}`\n\n"
        + (
            tool_mapping
            if tool_mapping
            else "This harness's tool mapping — which Hermes tool each action "
            "a skill names resolves to — did not fit either. Read it at "
            f"`{skills_dir}/using-tbaguette/references/hermes-tools.md`."
        )
        + f"\n</EXTREMELY_IMPORTANT>"
    )


def _stock_skill_names(skills_dir: str) -> list:
    """Every directory under skills/ that actually carries a SKILL.md."""
    return [
        name
        for name in sorted(os.listdir(skills_dir))
        if os.path.isfile(os.path.join(skills_dir, name, "SKILL.md"))
    ]


def register(ctx):
    skills_dir = _skills_dir()
    bootstrap = _build_bootstrap(skills_dir)

    # Register every stock skill with Hermes' native loader so skill_view can
    # load them on demand. Standard markdown; no conversion (plugin guide).
    # register_skill requires a pathlib.Path — a str raises AttributeError and
    # hermes silently disables the whole plugin (verified 2026-07-23).
    #
    # The name registered here is the bare one: Hermes namespaces it itself, as
    # `f"{manifest.skill_namespace or manifest.name}:{name}"`, and rejects a
    # name containing ':'. That is what makes `TBaguette:naming-things` the
    # resolvable form and the bare name a miss.
    for name in _stock_skill_names(skills_dir):
        ctx.register_skill(name, Path(skills_dir, name, "SKILL.md"))

    # pre_llm_call returning {"context": ...} is the documented injection path
    # (on_session_start return values are ignored, and ctx.inject_message
    # refuses from that hook — verified empirically 2026-07-23). The context is
    # appended to the user message.
    def pre_llm_call(
        session_id=None,
        user_message=None,
        conversation_history=None,
        is_first_turn=None,
        model=None,
        platform=None,
        **kwargs,
    ):
        return {"context": bootstrap if is_first_turn else NUDGE}

    ctx.register_hook("pre_llm_call", pre_llm_call)
