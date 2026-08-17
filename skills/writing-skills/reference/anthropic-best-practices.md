# Anthropic's skill-authoring guidance, adapted

## Contents

- Concise is the default assumption
- Matching freedom to fragility
- Naming conventions, and where this repo diverges
- Writing descriptions: the official version
- Progressive disclosure patterns
- Workflows and feedback loops
- Content hygiene
- What doesn't apply here: bundled code

This adapts Anthropic's published Agent Skills authoring guidance for this repo. The main `SKILL.md` already covers this repo's own frontmatter register and testing discipline in full; treat this file as the more general backdrop those specifics were drawn from, useful when a question comes up that the main file doesn't address directly.

## Concise is the default assumption

Anthropic's own guidance starts from one assumption: the model reading the skill is already very capable. Every paragraph should survive the question "does the agent already know this?" A skill that explains what a PDF is, or what a git commit is, before getting to the actual technique is spending tokens on something the reader didn't need.

Concise (roughly 30 tokens):

```markdown
## Extract PDF text

Use pdfplumber:

    import pdfplumber
    with pdfplumber.open("file.pdf") as pdf:
        text = pdf.pages[0].extract_text()
```

Bloated (roughly 100 tokens) — the same information, wrapped in a paragraph explaining what PDFs are and why libraries exist, none of which the reader needed.

This isn't just politeness. Every token in a loaded skill competes with conversation history and every other loaded skill's content for the same context window. Cutting a paragraph that doesn't teach anything new is a small thing that compounds across every skill an agent might load in one session.

## Matching freedom to fragility

Not every instruction should be equally prescriptive. Match specificity to how much the task can vary and how bad it is to get it wrong:

| Freedom | When | Looks like |
|---|---|---|
| High | Multiple valid approaches; judgment matters more than a specific sequence | Prose heuristics, a numbered list of considerations rather than steps |
| Medium | A preferred pattern exists, but some variation is fine | A template or pseudocode with parameters to adapt |
| Low | The operation is fragile, order-dependent, or expensive to get wrong | An exact sequence, explicitly marked not to be modified |

Think of it as a path with variable hazard: an open field tolerates general direction, because most routes across it are fine. A narrow bridge over a real drop gets exact instructions, because there's exactly one safe way across and every other way costs something to recover from. Most of this repo's content is high-freedom judgment calls; the low-freedom end is rare here specifically, because TBaguette skills don't ship the kind of exact, order-dependent scripts (a migration runner, a release script) that would call for it.

## Naming conventions, and where this repo diverges

Anthropic's official recommendation is gerund form — "Processing PDFs," "Testing Code" — because it reads as an activity rather than a category. This repo mostly follows that (`naming-things`, `orienting-in-unfamiliar-code`, `choosing-test-scope`), but not as a hard rule: a meaningful minority of skills here are noun phrases (`atomic-commits`, `caching-strategy`, `schema-evolution`) or even a persona name (`karen-and-the-manager`), where that reads more naturally than any gerund would have. The actual constraint in this repo is narrower than Anthropic's own: letters, numbers, and hyphens only, matching the directory name exactly, and avoiding the genuinely bad names Anthropic also warns against — `helper`, `utils`, `tools`, anything so generic it could describe half the library.

## Writing descriptions: the official version

The main `SKILL.md` covers this repo's own register in detail. Anthropic's underlying guidance is the same shape, stated more generally: third person always, specific enough to include real trigger terms, written for an agent choosing among potentially a hundred other skills' descriptions and needing to decide in one pass whether this is the right one.

Two habits worth keeping from the source guidance specifically:

**Keyword coverage.** Include the words someone would actually search for: error messages verbatim ("ENOTEMPTY", "Hook timed out"), symptom words ("flaky," "hanging," "zombie"), synonym pairs a search might use either half of ("timeout/hang/freeze," "cleanup/teardown"). A description that only uses the polished term for a problem misses the agent that's still describing the symptom in raw terms.

**One skill, one description field.** Resist the urge to pack in edge cases the trigger doesn't need — a description's job is getting the right reader to open the file, not replacing the file.

## Progressive disclosure patterns

The mechanism this repo's tooling relies on: an agent's system prompt carries every skill's `name` and `description` from the start, at near-zero cost, but the body of `SKILL.md` and anything under `reference/` loads only when the skill is actually opened, and only for the specific file that gets read. This is why a heavy reference file costs nothing until the moment it's needed, and also why nesting matters:

- **Keep references one level deep.** Every reference file should be linked directly from `SKILL.md`. An agent skimming a long file under time pressure will sometimes preview with a partial read instead of reading it whole; a file only reachable through another reference file is one that a partial read never surfaces.
- **Table of contents past roughly 100 lines.** A long reference file with a contents block at the top stays useful even to a partial read, because the scope is visible before the detail is.
- **Name files for their content, not their order.** `stacks/web.md` tells an agent what's inside before opening it; `reference2.md` doesn't.

## Workflows and feedback loops

For a task with several sequential steps, a numbered list an agent can visibly track tends to survive interruption better than prose describing the same steps — the state of "which step am I on" stays externalized instead of needing to be re-derived from context. This repo's own convention leans toward inline numbered steps for short sequences (three to five steps) and a table for anything checklist-like with more rows than that.

The other pattern worth keeping generally: **validate before proceeding**, stated as an explicit loop rather than a hope. "Draft, then check against the requirements, then only proceed once it passes" catches more than "draft carefully" does, because it names the check as a discrete, repeatable step instead of trusting care alone.

## Content hygiene

- **No time-sensitive claims.** "Before August 2025, use the old API" is wrong the day after whatever date it names. If an old pattern needs documenting at all, mark it as legacy explicitly rather than dating the cutover.
- **One term per concept, used consistently.** Pick "API endpoint" or "route," not both; pick "extract" or "pull," not both. A reader tracking whether two terms mean the same thing is spending attention the skill should have saved them.
- **Concrete examples over abstract ones.** An example from a real, specific scenario teaches faster than a templated one with placeholder fields, and is easier to verify as actually correct.

## What doesn't apply here: bundled code

A large fraction of Anthropic's own guidance concerns skills that ship executable scripts — pre-written utilities the agent runs rather than generates, package dependencies, a code-execution sandbox with its own filesystem rules. None of that applies to this repo: TBaguette skills are prose only, by constraint, not by omission. Where the source guidance says to write a `validate_form.py` script and have the agent run it, the equivalent move here is a clearly stated procedure the agent carries out itself, or — if the operation is genuinely mechanical and language-specific — a note that it belongs in the target project's own tooling rather than in the skill.

The one piece of that guidance worth carrying over anyway: **solve, don't punt.** Whether the deliverable is a script or a paragraph of instructions, handle the actual edge case rather than leaving a gap for the agent to figure out under pressure. "If the file doesn't exist, ask the user what to do" is a punt; "if the file doesn't exist, treat it as empty and proceed" is a decision. A skill should make the decisions that don't actually need the agent's judgment, and reserve the agent's judgment for the ones that do.
