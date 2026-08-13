# critique — design review with a verdict

A critique that ends in a list of observations is not a critique. It ends in a ranked judgment and a decision.

## Method

Look at the built result, not the source. If the surface can be rendered, render it. Review in this order — early failures make later ones irrelevant.

1. **Purpose.** Can a first-time viewer state what this is and what to do, within five seconds? If not, nothing below matters.
2. **Hierarchy.** Squint or blur the surface. What is still legible? That is your actual hierarchy. Does it match the intended rank?
3. **Structure.** Is related content grouped and unrelated content separated? Count the distinct alignment edges — more than three or four is usually accidental.
4. **Density.** Is the information-to-decoration ratio right for the mode? Too sparse is as much a failure as too dense.
5. **Consistency.** Same idea, same appearance. Count the distinct button styles, the distinct corner radii, the distinct grays. Unintentional variety is the most common defect in real products.
6. **States.** Ask for the empty, error, loading, and overflow cases. Their absence is the finding.
7. **Copy.** Does every control name its action? Does every error name its recovery? Is the voice consistent?
8. **Craft details.** Optical alignment, spacing rhythm, type steps, color intent, motion purpose.
9. **The scene test.** Would this work for the actual person in the actual place? Sunlight, one hand, gloves, hurry, low vision, tenth hour of a shift.

## Output shape

- **Verdict first.** One line: ship it, fix these N things first, or reconsider the direction.
- **Findings ranked by consequence,** not by where they appear on screen. Each finding: what is wrong, why it costs the user something, and the specific fix. Not "spacing is inconsistent" — "the 12/16/20px gaps between form rows read as accidental grouping; use 16px throughout and 32px between sections."
- **What is working,** briefly and specifically, so it survives the next revision.
- **Open questions** where you would need the brief or the data to judge.

## Rules

- **Never rewrite the brief.** If the design is doing what was asked and you would have asked for something else, say so once and then critique against the actual brief.
- **Separate taste from defect.** Label each finding as one or the other. A defect is measurable or costs a user something; taste is a recommendation and is marked as such.
- **No praise sandwich.** It buries the finding. Verdict, then findings, then what works.
- **Rank ruthlessly.** Twenty equally-weighted findings is the same as none.
