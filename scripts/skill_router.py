"""Build hooks/skill-index.tsv, the table the per-turn hook routes a prompt with.

    python3 scripts/skill_router.py          # write hooks/skill-index.tsv
    python3 scripts/skill_router.py --check  # exit 1 if the committed table is stale

A generic "does a skill cover this?" fired on every turn stopped changing behaviour,
and the skill listing it pointed at keeps trigger text for only the most-used skills.
So hooks/route-skills.awk checks each prompt against hooks/skill-routes.tsv, a short
hand-written list of intents, then against this table, built from every skill's full
`description:`, and the hook names the skills that fit.

Each line: slug TAB one-line trigger TAB term=weight,term=weight,...
A term is a stem_stem pair of neighbouring words. The stemmer and the
stopwords live in both this file and the awk script; test_hooks.py compares them.
"""
from __future__ import annotations

import math
import re
import sys
from collections import Counter
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SKILLS = ROOT / "skills"
OUT = ROOT / "hooks" / "skill-index.tsv"

# Listed nowhere else on purpose: awk has no import. test_hooks.py holds the two copies equal.
STOPWORDS = frozenset("""
a about above after again all also am an and any are as at be been before being but by can could did
do does doing done down each even every few for from further get gets getting had has have having he
her here hers him his how i if in into is it its itself just let lets like make makes making may me
might more most much must my need needs no nor not now of off on once one only or other our out over
own please same she should so some still such than that the their them then there these they this
those through to too under until up us use used uses using very was way we were what when where which
while who why will with without would yes yet you your
""".split())

# Words nearly every description uses to describe its own shape, or that say nothing about the work.
BOILERPLATE = frozenset("""
covers skill already someone something anything nothing thing things whatever whoever instead rather
across another whether because around first second third next last once new old real long short small
big good bad right wrong want wanted going back keep look looks show shows give gives take takes
""".split())


def stem(word: str) -> str:
    if len(word) > 5 and word.endswith("ies"):
        return word[:-3] + "y"
    if len(word) > 5 and word.endswith("ing"):
        return word[:-3]
    if len(word) > 4 and word.endswith("ed"):
        return word[:-2]
    if len(word) > 4 and word.endswith("es"):
        return word[:-2]
    if len(word) > 3 and word.endswith("s") and not word.endswith("ss"):
        return word[:-1]
    return word


def terms(text: str) -> list[str]:
    """Stems and neighbour pairs. A stopword breaks a pair, as it does in the awk script."""
    out: list[str] = []
    previous = ""
    for word in re.findall(r"[a-z0-9]+", text.lower().replace("'", "")):
        if len(word) < 3 or word in STOPWORDS or word in BOILERPLATE:
            previous = ""
            continue
        current = stem(word)
        out.append(current)
        if previous:
            out.append(f"{previous}_{current}")
        previous = current
    return out


def description_of(skill_md: Path) -> str:
    match = re.search(r"^description:\s*(.+)$", skill_md.read_text(encoding="utf-8"), re.M)
    if not match:
        raise ValueError(f"{skill_md}: no one-line description")
    return match.group(1).strip().strip("\"'")


def blurb(description: str, limit: int = 110) -> str:
    text = re.sub(r"^Use (this )?(at|when|whenever|before|after|for)\b\s*", "", description).strip()
    text = re.split(r"(?<=[.;])\s", text)[0].rstrip(".;")
    text = text.replace("\t", " ").replace('"', "'").replace("`", "'").replace("\\", "/")
    if len(text) > limit:
        text = text[:limit].rsplit(" ", 1)[0].rstrip(",;: ") + "..."
    return text


def build() -> str:
    skills = sorted(p.parent.name for p in SKILLS.glob("*/SKILL.md"))
    per_skill: dict[str, Counter] = {}
    for slug in skills:
        description = description_of(SKILLS / slug / "SKILL.md")
        trigger, _, covers = description.partition(" Covers ")
        weights: Counter = Counter()
        for term in set(terms(trigger)):
            weights[term] = 2
        for term in set(terms(covers)):
            weights[term] = max(weights[term], 1)
        for term in set(terms(slug.replace("-", " "))):
            weights[term] = 3
        per_skill[slug] = weights

    n = len(skills)
    df = Counter(term for weights in per_skill.values() for term in weights)
    lines = []
    for slug in skills:
        scored = {}
        for term, base in per_skill[slug].items():
            # A pair only. Measured on 565 real prompts, single words matched 84% of them,
            # "Continue" included, and most of those matches were noise.
            if "_" not in term or df[term] > n / 20:
                continue
            scored[term] = round(base * math.log(n / df[term]) * 10)
        table = ",".join(f"{t}={w}" for t, w in sorted(scored.items()) if w > 0)
        lines.append(f"{slug}\t{blurb(description_of(SKILLS / slug / 'SKILL.md'))}\t{table}")
    return "\n".join(lines) + "\n"


def main() -> int:
    table = build()
    if "--check" in sys.argv[1:]:
        if not OUT.exists() or OUT.read_text(encoding="utf-8") != table:
            print("hooks/skill-index.tsv is stale: run python3 scripts/skill_router.py")
            return 1
        return 0
    OUT.write_text(table, encoding="utf-8")
    return 0


if __name__ == "__main__":
    sys.exit(main())
