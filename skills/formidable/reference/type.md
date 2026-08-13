# type — the surface's voice

Typography is 90% of most interfaces. Getting the scale, the measure, and the rhythm right fixes more perceived quality than any other single move.

## Scale

- **Build a scale, do not pick sizes.** 5–7 steps from a base, with a consistent ratio (1.2 for dense UI, 1.25–1.333 for general, 1.4+ for editorial and display). Every size in the product comes from the scale.
- **Steps must be obvious.** If two levels are within ~15%, the reader sees no hierarchy — they see a mistake. Merge them or push them apart.
- **Weight is the second axis** and it is underused. A hierarchy built on size alone gets big fast; size + weight stays compact. Two adjacent levels can share a size if the weight differs clearly.
- **Case is the third axis.** All-caps with tracking works for small labels only, never for anything the reader must actually read.

## Measure and rhythm

- **45–75 characters per line** for prose; 60–66 is the sweet spot. Wider is unreadable; much narrower fragments. Constrain the container, not the font size.
- **Line height scales inversely with size.** Body 1.4–1.6; display 1.0–1.2; a 48px heading at 1.5 is a broken heading. Dense tables can go to 1.25.
- **More space above a heading than below it.** The heading belongs to what follows. This single rule fixes most "sections feel muddled" complaints.
- **Tracking:** tighten display sizes (down to about -0.04em, no further), leave body alone, loosen small caps and all-caps labels (+0.05 to +0.1em).
- **One baseline rhythm.** Space between blocks comes from the space scale, not from arbitrary margins.

## Choice and pairing

- **Two families maximum,** and one is usually enough. A third needs a specific job (code, data).
- **Pair by contrast, not by similarity.** A geometric sans with a high-contrast serif reads as a decision; two humanist sans faces read as a mistake.
- **A system display face as the display voice of an own-world design is a failure,** not a fallback. Source and self-host a face whose character matches the direction.
- **Check the whole set you need:** the weights, the italics, the numerals, and the scripts your users actually use. A face with no bold is a face you cannot build hierarchy with.
- **Tabular figures for anything in a column.** Proportional figures in a table make numbers wobble.
- **Verify the fallback.** Metric-matched fallbacks prevent the layout jumping when the real face arrives.

## Per-stack notes

Terminal: one family, one size — hierarchy comes from weight, case, space, and position. Embedded: bitmap or hinted faces below ~14px. Print: serif at 9–11pt, tighter leading, real small caps. XR: angular size ≥1.5°, few words, heavy weight. Email: web-safe stack with a designed fallback.

## Rules

- **Set real content,** including the longest string and the shortest.
- **Never justify text** without hyphenation; ragged-right beats rivers.
- **Balance headings** so a two-line heading does not leave one orphaned word.
