---
name: judging-duplication
description: Use when two pieces of code look nearly identical and merging them is the obvious cleanup, when a reviewer asks why a near-copy exists, when extracting a shared helper or a base class, or when a DRY pass is about to unify things that differ only in a small arbitrary detail. Covers telling duplication that is debt from duplication that is a contract, the rule of three, what a merge breaks that no local test observes, and the residue to leave when a split is deliberate.
---

# Judging duplication

## Overview

Two functions that differ only in a detail are the most obvious cleanup in the file, and merging them can be invisible to every test you currently have.

The question before unifying is never "are these the same?" It is **"what would notice if they stopped differing?"**

## When to use

- A near-duplicate pair, and one of them looks like a sloppy copy of the other.
- A review comment that just says "DRY" or "extract this".
- Building a shared helper, base class, or generic to serve two existing callers.
- Deleting one of two implementations because the other appears equivalent.
- Not for: the mechanics of performing the merge once decided (`refactoring-safely`), or where to place the resulting module (`drawing-boundaries`).

## Who observes the difference

Find the observer. The answer decides everything else.

| The difference is observed by | Then the duplication is | Do |
|---|---|---|
| Nothing — only these two call sites | Debt | Merge |
| Another part of this process | Debt, with a test to update | Merge, fix the test |
| A peer verifying something you produced — a signature, a digest, a checksum | **A contract** | Keep the split |
| A format already written to disk, a wire, or a queue | **A contract** | Keep the split |
| A counterpart that rebuilds the same bytes and compares them | **A contract** | Keep the split |
| Two independent policies that happen to agree today | Coincidence, not duplication | Keep the split |
| You cannot determine who | **Unknown — treat as a contract** | Keep the split |

The last row is the one people get wrong under pressure. The cost of maintaining two functions is bounded and known. The cost of merging them when the observer is external is neither, so an undetermined observer resolves toward keeping the split, not toward the tidier diff.

Finding the observer is a search, not a judgement call: look for who consumes the output — a verification step, a stored artifact, a comparison against a rebuilt value — and check whether either function's history mentions a specification, an interop fix, or a third party. `code-archaeology` is the tool when the answer is not in the file.

## Coincidence versus a shared reason

Identical code is not the same as one idea written twice. Ask what would have to be true for a future change to apply to both sites:

- **A shared reason** — both encode the same rule, and a change to that rule must hit both. That is duplication, and it is debt.
- **A coincidence** — two rules that presently produce the same code and can diverge without anything being wrong. Merging couples them permanently: the next change to one now needs a flag, and the flag is how a clean helper becomes a parameterized mess that serves nobody.

The reliable test is prospective, not retrospective. Do not ask "are these the same today". Ask "when one changes, must the other?" If the honest answer is "probably not", the resemblance is a coincidence and the merge is a mistake dressed as hygiene.

## Arbitrary is what a specification looks like from inside

The tell for a load-bearing split is that the difference looks small and arbitrary:

- Two encoders disagreeing only about a space and a few delimiter characters.
- Two serializers differing only in field order.
- Two hash inputs differing only in a trailing separator.
- Two escapers disagreeing about one character class.

Arbitrary is exactly what somebody else's specification looks like from inside your codebase. You are not seeing sloppiness; you are seeing the shape of a rule written down somewhere you have not read.

**A reference implementation carrying the same split is evidence *for* the split**, not evidence that everyone copies badly. When two independent codebases both contain the same "redundant" pair, that is a specification leaving fingerprints in both.

## The rule of three, and why it is about information

Duplicate twice, extract on the third. The usual justification is that two points do not establish a pattern, which is true and undersells it.

The real reason: **the third instance is the first one that tells you which parts vary.** With two, every difference is ambiguous — you cannot tell the essential axis from the accident. An abstraction extracted from two call sites encodes a guess about which is which, and a wrong boundary costs more than the duplication ever would.

This cuts against the instinct to extract early, and the asymmetry is real: inlining a premature abstraction back out is far harder than extracting a real one late, because by then every caller has bent itself to fit the wrong shape.

## Where the failure lands

What makes a wrong merge expensive is not the probability but the blast radius.

Unifying a signing path does not fail where the refactor is. It fails at a third party, as a generic rejection indistinguishable from a bad credential, pointing nowhere near the diff — and often only for the subset of inputs containing the character that used to be treated differently. That last clause is why the change ships green: the fixtures do not contain the character.

The general shape: merges that break a contract fail **remotely, later, partially, and with a message naming something other than the change.** Weigh that against the tidiness the merge buys.

## Residue when you keep the split

A deliberate split that looks accidental will be merged by the next person, correctly following the same instinct you just overrode. Two pieces of residue prevent that:

- **Put the reason at both sites.** A comment on one of a pair explains nothing to whoever arrives at the other, and they are the one who will merge them. Name the external observer explicitly — the spec, the peer, the format.
- **Pin the divergence with a test asserting the two produce *different* output for the same input**, naming the character or field that must differ. It reads like a strange test. It is the only thing that turns an attempted unification into a local, immediate failure instead of a remote one, and it is worth more than the comment because it cannot be skimmed past.

## Common mistakes

| Symptom | Real cause |
|---|---|
| A third party started rejecting requests after a cleanup | Merged an encoder pair whose split was a contract |
| A shared helper grew a boolean, then three | Merged a coincidence; each divergence became a flag |
| Green suite, broken integration | The only observer was outside the process, so no local test could see it |
| An abstraction that fits neither caller well | Extracted at two instances, before the varying axis was knowable |
| The same "redundant" pair keeps getting re-merged | Split was deliberate but left no residue at either site |
| A reference implementation was copied "more cleanly" | Its redundancy was the specification, and it got tidied away |
| Reviewer says DRY, nobody asks who observes the difference | "Are these the same?" asked instead of "what would notice?" |

## Red flags

- "These are basically identical."
- "It's just a space" / "it's only the field order."
- "The other implementation does it redundantly" — about someone else's reference code.
- "I'll add a flag for the difference."
- "Nothing broke when I merged them" — said before anything external has run.
- "We can always split it again later" — after the merged version reached a wire format.
- Reaching for DRY without having found the observer.
