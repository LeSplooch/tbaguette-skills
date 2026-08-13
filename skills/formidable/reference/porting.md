# port — carry a design across stacks

Porting fails in two opposite ways: cloning the pixels into a stack that cannot hold them, or abandoning the design and shipping the target stack's defaults. Both lose the product's identity.

**Port decisions, not pixels.**

## Method

1. **Extract the decisions** from the source design, stated stack-independently. What is the hierarchy? What is the density? What carries emphasis — size, weight, color, or space? What is the voice of the copy? What is the one accent and what does it mean? What does motion communicate? Write these down before opening the target.
2. **Map each decision into the target envelope.** Read the target's stack file. For every decision, either it survives directly, it has a substitute, or it is dropped with a reason.
3. **Honor the target's idioms above the source's.** A design ported to iOS that keeps Android's back button is not faithful — it is broken. Navigation, controls, gestures, and platform conventions belong to the target. The identity lives in type, color, space, density, iconography, motion character, and copy.
4. **Rebuild, do not translate.** Reimplement in the target's native structure. Recreating a web layout with absolute positioning in a native toolkit produces something that breaks on the first font-scale change.
5. **Verify against the source side by side,** then verify against the target's own conventions separately. Both must pass.

## Substitution table

Common mappings when a decision cannot survive literally.

| Source decision | When the target lacks it | Substitute |
|---|---|---|
| Elevation via shadow | No shadow (terminal, e-ink, print) | Border weight, background shift, or space |
| Corner radius | No radius | Consistent inset, or a chosen corner glyph |
| Brand color | Themed or limited palette | Position, weight, and the one nearest available hue |
| Hover state | No pointer | Focus/selection state; make it always-visible |
| Motion transition | No motion budget | Instant change plus a persistent state marker |
| Custom display face | No font control | Weight, case, and spacing carry the voice |
| Dense multi-column | Narrow surface | Prioritized single column; secondary data behind disclosure |
| Tooltip | No hover, no overlay | Inline caption, or a dedicated detail area |
| Infinite scroll | Paged or fixed viewport | Explicit paging with position indicator |

## Rules

- **Never ship a webview and call it a port** unless the brief says so, and then design it as web with native chrome.
- **Density is re-decided per stack,** not carried. Mobile density on desktop wastes the screen; desktop density on mobile is untappable.
- **Copy is re-fitted, not re-flowed.** Labels that fit a wide button do not fit a narrow one; write shorter labels rather than shrinking type.
- **Record the mapping** alongside the tokens. The next surface will need the same substitutions, and undocumented ones get re-improvised differently.
