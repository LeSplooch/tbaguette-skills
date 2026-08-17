# Persuasion principles for skill design

## Contents

- Why this applies to a model at all
- The seven principles
- Which principles for which skill type
- Ethical boundary
- Sources

## Why this applies to a model at all

Large language models respond to the same persuasion principles that move people, because they were trained on a world of text saturated with them. That isn't a license to manipulate — it's a reason to notice that soft language ("consider", "try to") reads as genuinely optional to a model for the same reason it does to a person, and to write discipline-enforcing skills accordingly.

Meincke et al. (2025) tested seven classical persuasion principles against a large language model across roughly 28,000 conversations and found compliance with a request roughly doubled when it was framed using them instead of a neutral control. The specific study measured compliance with requests most models are trained to resist, which isn't the interesting part here. The mechanism it demonstrates is: persuasion-shaped language moves compliance measurably beyond what clarity-shaped language gets on its own. That's the part worth carrying into skill-writing, where the "request" is a rule genuinely worth following.

## The seven principles

### Authority

Deference to expertise or a rule stated as settled rather than as an opinion.

In a skill: imperative, non-negotiable language — "no exceptions," a flat "delete it, start over" rather than "you might want to reconsider." It removes the moment of deciding whether this particular case is the exception.

Use for: discipline-enforcing skills, safety-critical rules, established practice this repo has already paid to learn the hard way. Compare weak and strong versions of the same rule:

- Weak: "Consider staging only the files you personally wrote."
- Strong (`atomic-commits`' actual stance): "Stage by naming the files you wrote... Re-read the status output immediately before *each* stage, not once when the task begins."

### Commitment

Consistency with something already stated or already done.

In a skill: forcing an explicit choice ("choose A, B, or C") instead of leaving a decision implicit, or requiring a stated plan before work starts so later steps are measured against a commitment rather than renegotiated from scratch under pressure.

Use for: multi-step processes, and anywhere a skill wants a decision on record before the pressure that would relitigate it arrives.

### Scarcity

Urgency created by a closing window.

In a skill: "immediately after X," "before proceeding" — sequencing language that forecloses "I'll get to it later" as an available option.

Use for: verification steps that are cheap to do now and expensive to reconstruct once the moment has passed.

### Social proof

Conformity to what's normal, or framed as universal.

In a skill: stating a pattern as universal ("every commit," "always") and naming the failure mode plainly ("X without Y — every time") instead of leaving it to read as one opinion among several equally valid ones.

Use for: reinforcing a standard this repo already treats as settled. Weak fit for introducing a genuinely contested judgment call, which social-proof framing would misrepresent as already-decided.

### Unity

Shared identity — "we," in-group framing.

In a skill: collaborative framing where the work genuinely is collaborative — a review, a handoff, a pass meant to happen alongside someone else. A weak fit for a solo discipline-enforcing rule, where it adds nothing that authority and commitment aren't already carrying.

### Reciprocity

Obligation created by a benefit already received.

Rarely useful in a skill, and it reads as manipulative faster than the other six when forced. Default to leaving it out.

### Liking

Preference for cooperating with whoever seems likeable.

Actively avoid it for compliance. It's the one principle that trades directly against honest pushback — a skill that leans on being liked is a skill quietly discouraging disagreement, which is the opposite of what `karen-and-the-manager` or a real code review is for.

## Which principles for which skill type

| Skill type | Reach for | Avoid |
|---|---|---|
| Discipline-enforcing (a rule someone is tempted to skip) | Authority and commitment; social proof for "this is already the norm" framing | Liking, reciprocity |
| Technique or pattern (a method to apply) | Light authority at most; unity where the work is genuinely collaborative | Heavy authority — it overclaims for a skill that's really just good, optional-in-tone advice |
| Reference (API shapes, syntax, a lookup table) | Clarity only | All seven — persuasion language on reference material reads as oversell, and undermines trust in the parts that are just facts |

## Ethical boundary

The same techniques that make a discipline-enforcing skill actually get followed can also manipulate. The line: does this serve the reader's own genuine interest, if they fully understood why it's phrased this way? "No exceptions, delete it, start over" for a testing discipline passes that test — the reader benefits from not having to relitigate the same rationalization every time it resurfaces under pressure. Using the identical toolkit to make a bad default look normal, or to manufacture urgency for something that isn't actually urgent, does not — regardless of how the wording is justified afterward.

## Sources

Cialdini, R. B. (2021). *Influence: The Psychology of Persuasion* (New and Expanded). Harper Business — the original seven-principle framework this reference adapts.

Meincke, L., Shapiro, D., Duckworth, A. L., Mollick, E., Mollick, L., & Cialdini, R. (2025). "Call Me A Jerk: Persuading AI to Comply with Objectionable Requests." University of Pennsylvania — the compliance study behind the LLM-specific claim above.
