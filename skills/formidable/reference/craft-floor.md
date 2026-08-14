# Craft floor

Load immediately before editing UI, after direction is settled. Build without announcing the checklist. A pinned brief or the committed visual world overrides anything here; your own habit does not.

## Verify on the built result

Checks on what rendered, not on what you intended. Run them together in one batched inspection round — they share a render.

- **Contrast** measured, not judged. Body ≥4.5:1, large text and meaningful glyphs ≥3:1. Secondary text on a colored surface is tinted from that hue, never neutral gray dropped on top.
- **Spacing** obeys one scale. Tight inside a group, generous between groups, more space above a heading than below it. Read the computed values rather than trusting the source.
- **Type** has obvious steps in size *and* weight. Body measure 45–75 characters. Real copy at every width — the longest label, the longest name, the longest translation.
- **Alignment** — everything sits on a shared grid or an intentional break from one. Optical alignment beats mathematical alignment for icons, quotes, and round shapes.
- **Depth**, where the stack has it, comes from consistent light: one direction, offset plus soft blur. A zero-offset colored halo is decoration.
- **Motion** is one authored moment, not scattered effects and not the same entrance on every section. Ease out from an already-visible default.
- **States** all present: hover or focus, active, disabled, loading, empty, error, selected, and whatever the stack adds.
- **Exactly one focus indicator per focus stop.** Where a wrapper owns a control's visible chrome, the wrapper's focus treatment and a global per-element focus rule both fire and paint two rings — the inner one clipped on the side that sits against a leading icon and bleeding past the wrapper's edge everywhere else. It reads as a bug, not as a state. The inner control opts out explicitly, which then obliges the fallback in [harden.md](harden.md).
- **Copy** in the product's own language. Controls name their action; errors name problem and recovery.
- **Coverage** — every brief requirement present and findable within seconds.
- **Magnify before dismissing a reported artifact.** Banding, seams, halos, and half-pixel misalignment are routinely invisible at normal size in a compressed screenshot and obvious at 3–4×. A report you could not reproduce at the reporter's scale has not been reproduced.

## The reflexes no detector catches

- **Hierarchy is a decision, not an accident.** Exactly one thing is most important on a surface. If two things compete, you did not decide. Rank the elements before styling them.
- **Contrast of size beats contrast of color.** When a design reads flat, the fix is usually a bigger size ratio or a heavier weight, not a brighter accent.
- **Space is the cheapest tool and the first one you forget.** Most "cluttered" is a spacing problem, not a content problem.
- **Alignment edges should be few.** Every additional left edge is a new axis the eye has to track. Three columns of differently-indented text is three designs.
- **Repetition creates system.** The same idea should look the same everywhere; the second variant needs a reason.
- **Borders are the weakest separator.** Prefer space, then background shift, then a border. A grid full of boxes is usually a grid that needed gaps.
- **Optical size beats nominal size.** A 16px glyph and 16px text are not the same weight on screen. Trust the render.
- **Density is a target, not a leftover.** Decide how much should be visible at once before you decide how it looks.
- **The default state is the design.** Most users see the empty, the one-item, and the truncated cases far more than your populated mock.

## Finishing

Every deliverable ships with: real content (not lorem, not placeholder names), working interactive states, the full theme set the stack supports, and behavior at the smallest and largest size the surface can take. Anything missing is called out explicitly, never quietly left.
