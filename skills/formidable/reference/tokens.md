# tokens — one decision, every stack

A token system exists so a decision is made once. It is not a color list. The failure mode of every token system is naming tokens after what they look like, which makes them impossible to change and impossible to theme.

## Three tiers

1. **Primitive** — the raw value, named literally. `blue-600`, `space-4`, `size-14`, `radius-md`. Never referenced by a component.
2. **Semantic** — the role, named by meaning. `surface-raised`, `text-secondary`, `border-focus`, `feedback-danger`, `space-section`. This is the only tier components use.
3. **Component** — only where a component genuinely deviates and the deviation is intentional. `button-primary-bg`. Keep this tier small; a large third tier means the second tier is wrong.

Themes swap tier 1→2 bindings. If a component references a primitive, that component cannot be themed, and you will find this out during the dark-mode work.

## Naming rules

- Name by **role**, never by appearance. `text-danger`, not `text-red`. The day danger becomes orange, `text-red` is a lie in every file.
- Name by **relationship**, not by number, where a scale is semantic. `surface`, `surface-raised`, `surface-sunken` beats `gray-100/200/300` for anything a theme will invert.
- **Ordinal scales stay ordinal.** `space-1..8`, not `space-sm/md/lg/xl/xxl` — you will need a value between `md` and `lg` within a month.
- One name, one meaning, everywhere. The same token in web CSS, mobile resources, terminal palette, and design tool must mean the same thing.

## Minimum viable set

- **Color:** 3–4 surfaces, 3 text levels, 1 border, 1 focus, 1 accent (+hover/active), 4 feedback roles (danger/warning/success/info), each with a foreground *and* a background variant.
- **Space:** one geometric-ish scale of 6–8 steps from a base unit (commonly 4). Everything uses it. A one-off `13px` is a bug.
- **Type:** 5–7 sizes, 2–3 weights, 2–3 line heights, 1–2 families. Fewer than you think.
- **Radius:** 3 steps plus `full`. **Border:** 1–2 widths. **Elevation:** 3 steps, each a real light model.
- **Motion:** 3 durations, 3 easings, and a `duration-instant: 0` that reduced-motion maps everything to.

## Cross-stack

Keep the source of truth in a neutral, machine-readable format (JSON or similar) and generate per-stack outputs — CSS custom properties, mobile resource files, a terminal palette map, a shader constant block, an email inline-style map. Hand-maintaining the same palette in four languages guarantees drift.

**Not every token survives every stack.** A terminal has no radius or elevation; e-ink has no motion. Map the missing ones deliberately: elevation becomes a border weight, motion becomes an instant state change, radius becomes a corner glyph. Record the mapping — undocumented, it gets re-improvised per screen.

## Rules

- **Extract from the built product, not from imagination.** Inventory the real values in use first; the count of unique grays and radii tells you the actual state.
- **Every token needs a usage rule,** one line, or it will be used wrong. A token nobody knows when to use becomes a synonym.
- **Deleting a token is a migration.** Alias, migrate, then remove.
