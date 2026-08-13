# Stack: email

**Envelope.** The most hostile rendering environment still in production. Dozens of clients with incompatible engines, some stripping your CSS, some rewriting your colors, most blocking your images by default, many showing your message in a 300px preview pane inside a dark theme you did not design. You cannot run scripts, cannot query a container, and cannot fix it after sending.

**Design for the worst client, then enhance.** Progressive enhancement is not a nicety here; it is the only strategy that survives.

## Craft

- **The message must work with images off.** Alt text is designed content: styled, sized, and meaningful. A single hero image carrying the entire message is the most common total failure.
- **Preheader text is the second headline.** The subject and the first ~90 characters are the real interface; most of your audience reads only those. Write them deliberately and hide the filler.
- **Single column, ~600px, real text.** Tables for structure where support demands it, inline styles, web-safe fonts with a designed fallback stack. Anything more ambitious needs testing evidence, not optimism.
- **Dark mode will be forced on you.** Some clients invert, some partially invert, some do nothing. Avoid pure white backgrounds behind dark logos, avoid images with baked-in white backgrounds, use transparent PNGs with a stroke, and check the inverted render.
- **One primary action, bulletproof.** A background-color cell with padded text beats an image button. Make the whole block tappable and at least 44px tall. Repeat the action once near the end if the email is long.
- **Link and footer honesty.** Visible unsubscribe, physical address where required, plain-language sender identity. This is compliance *and* design — a hidden unsubscribe converts to a spam report.
- **Length has a hard ceiling.** Some clients clip past ~100KB and hide the footer with it.
- **Accessibility survives:** semantic heading order, real text, contrast ≥4.5:1, `lang` set, tables marked presentational, and a plain-text alternative that is actually written rather than auto-stripped.

## Failure modes

| Symptom | Real cause |
|---|---|
| Blank message for many readers | Image-only design, images blocked by default. |
| Broken layout in one client | Unsupported CSS (flex, grid, position) without a table fallback. |
| Logo disappears | Forced dark mode inverting a transparent or white-background asset. |
| Footer missing | Message clipped for exceeding the size limit. |
| Low engagement despite a good design | Subject and preheader treated as metadata rather than as the primary interface. |

## Audit hooks

Render with images blocked; in a narrow preview pane; in forced dark mode; in the two most common desktop, webmail, and mobile clients your audience uses; as plain text; at 200% zoom; with every link resolved.
