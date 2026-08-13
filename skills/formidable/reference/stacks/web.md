# Stack: web

**Envelope.** Effectively unlimited color, type, and motion; unlimited viewport variance; hostile network; the user controls font size, zoom, theme, reduced motion, and extensions. The DOM is accessible by default and you can only make it worse.

## Idioms a native user expects

- Links look like links and navigate; buttons look like buttons and act. Never a `div` with a click handler.
- Back button works. URLs are addressable and shareable. State that matters lives in the URL.
- Ctrl/Cmd-click, middle-click, and right-click on anything navigational.
- Native form controls unless you have a specific reason, and if you replace one, you owe its entire keyboard and screen-reader behavior.
- Scroll belongs to the user. No hijacking, no scroll-jacked sections, no infinite scroll on anything with a footer.

## Envelope-specific craft

- **Fluid over breakpoint-hopping.** `clamp()` for type and space, container queries for components, breakpoints only where the layout genuinely reorganizes. A component should not care about viewport width.
- **Self-host the display face.** Subset it, preload it, and set `font-display: swap` with a metric-matched fallback so the reflow does not jump. A system face as the display voice of an own-world page is a failure, not a fallback.
- **Reach past transform and opacity** once those are smooth: `filter`, `backdrop-filter`, `clip-path`, `mask`, `background-blend-mode`, and shadow are part of the palette. Animate only compositor-friendly properties in loops.
- **Dark mode is a designed theme,** not an inversion. Pure black and pure white are both wrong for large areas; elevation in dark mode comes from lighter surfaces, not darker shadows.
- **The layout must survive** 200% zoom, 400% text-only zoom, a 320px viewport, and a translated string. Test with the longest real string, not the average one.

## Failure modes specific to this stack

| Symptom | Real cause |
|---|---|
| Layout shift on load | Unsized media, late fonts, injected banners. Reserve the box. |
| "Feels slow" but metrics are fine | Missing immediate feedback on interaction; work on the main thread blocking paint. |
| Works for you, broken for users | Zoom, extensions, autofill styling, OS font scaling, `prefers-reduced-motion`. |
| Keyboard trap | Custom overlay without focus management. Trap intentionally, restore on close. |
| Mobile tap does nothing | Hover-only affordance, or a 300ms tap delay, or a target under 44px. |

## Audit hooks

Contrast at computed values; tab order matches visual order; every image has intent-appropriate alt (empty for decorative); one `h1` and no skipped heading levels; focus visible on every interactive element; forms have real labels; reduced-motion honored; no fixed `px` on anything text-sized.
