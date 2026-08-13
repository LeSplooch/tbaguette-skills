# shape — decide before you build

Produce a decision document, not code. Shape ends when someone else could build the thing and make the same choices you would.

## Sequence

1. **Who, where, how often.** The use scene, not the persona. Standing in a warehouse with gloves on; three seconds between calls; every morning for two years. The scene determines density, target size, contrast, and motion budget more than any brand input.
2. **The one job.** One sentence: what the person accomplishes here. If it takes two sentences, it is two surfaces.
3. **Mode.** Persuade, Operate, Read, Experience, or Attend. Attend if they will not be sitting still and looking straight at it.
4. **Envelope.** Stack, minimum and maximum size, color depth, input devices, latency budget, offline behavior, and theme obligations.
5. **Content inventory.** Every element that must appear, ranked. Then cut. The rank *is* the visual hierarchy — do not defer it to styling.
6. **The states.** List them explicitly: empty, loading, partial, error, offline, permission-denied, one item, maximum items, longest string, no-permission, first-run. Each gets a designed answer, even if the answer is "same as X."
7. **Structure.** How the ranked content is grouped and what the eye does first, second, third. Sketch in words or as a rough tree; do not style yet.
8. **Interaction contract.** Every action, its trigger, its feedback, its failure, its undo.
9. **Risks.** What is unknown, what might not fit, what needs real data before it can be trusted.

## Output shape

A short document with those nine headings filled. Then one paragraph: the direction — what this should feel like and why, in terms a builder can act on. Then a list of open questions, each with your recommended default so work can proceed without an answer.

## Rules

- **Do not write UI code during shape.** The moment you style, you stop deciding.
- **No persona fiction.** If you do not know the use scene, say so and state your assumption.
- **Ranked, not listed.** An unranked content inventory is the most common way shape fails — it defers every hard call to the person implementing.
- **Name what you are cutting.** A shape document with no cuts did not do the work.
