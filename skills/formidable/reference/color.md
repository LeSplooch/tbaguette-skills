# color — meaning, not decoration

Color is the least reliable channel you have: it is themed by users, inverted by clients, crushed by printers, misread by 8% of men, and destroyed by sunlight. Design so the surface works without it, then use it to make the working surface fast.

## Structure

- **Build in a perceptual space** (OKLCH/LCH or HSLuv), not raw HSL or hex arithmetic. Equal lightness steps in HSL are not equal to the eye; ramps built there develop muddy midtones and unpredictable contrast.
- **Neutrals carry a hue.** Pure gray next to a colored brand reads dead. Tint the neutral ramp slightly toward the accent or its complement, and keep that tint consistent.
- **One accent, used rarely.** The accent means "this is the action" or "this is the exception." A surface with six accent-colored things has no accent.
- **Severity is ordered.** info < success < warning < danger. The palette must express that order in saturation and weight, not just hue. Each role needs a foreground *and* a background variant with verified contrast in both themes.
- **Categorical palettes cap out around 7** distinguishable series, fewer for colorblind readers. Past that, use direct labels, position, or shape — not more hues.

## Dark mode is designed, not inverted

- Never pure black behind large areas or pure white text on it — halation makes it vibrate. Aim for a very dark neutral surface and a slightly-off-white text.
- **Elevation reverses:** in light mode, higher surfaces cast shadows; in dark mode, higher surfaces get lighter. Shadows barely exist in the dark.
- **Saturated colors need desaturating** in dark mode; the same accent that reads confident on white glares on black.
- Contrast ratios must be verified independently in each theme. Passing in light says nothing about dark.

## The second channel

Every meaning encoded in color carries something else: a glyph, a label, a shape, a position, a weight, or a pattern. Test by desaturating the whole surface and reading it. This is the single most-skipped and most-consequential color rule.

## Per-stack notes

- **Terminal:** use the theme's named colors; the user's red is their red. 24-bit only for a deliberate branded surface, gated on `COLORTERM` and `NO_COLOR`.
- **Print:** convert and check in CMYK and in grayscale; light grays below ~15% vanish.
- **Email:** forced dark mode will alter your colors; check the inverted render.
- **Embedded/e-ink:** 1–4 levels; dithering and hatching are your ramp.
- **Game/XR:** color sits over unknown backgrounds; the element must own its own pixels.

## Rules

- **Measure every pair you ship.** Body ≥4.5:1, large text and meaningful non-text ≥3:1, focus rings ≥3:1 against both neighbors.
- **Do not encode with hue alone, ever.** Red/green is the specific pair to avoid.
- **Name tokens by role,** never by hue — see [tokens.md](tokens.md).
