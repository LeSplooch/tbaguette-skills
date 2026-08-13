# Stack: game HUD and in-engine UI

**Envelope.** Your interface sits over a moving, unpredictable, artist-owned image. Every pixel behind it can be any color next frame. The player's attention belongs to the game, not to you, and you have a per-frame budget measured in tenths of a millisecond. Input may be gamepad, mouse, touch, or motion — often switching mid-session.

**Mode is almost always Attend.** The HUD is glanced at, under stress, in peripheral vision. Everything else follows from that.

## Craft

- **Legibility over arbitrary backgrounds** is the defining problem. Solve it structurally: an outline or drop shadow on text, a subtle scrim behind the element, or a shape that owns its own pixels. Never rely on the current level's art staying dark.
- **Rank by consequence.** Health and imminent danger are largest, brightest, most central-adjacent, and may move. Ammo counts are quieter. Cosmetic counters are quietest. A HUD where everything is equally loud tells the player nothing.
- **Peripheral vision reads motion and shape, not detail.** Critical state changes signal with movement, scale, or silhouette — a number turning red is invisible to someone looking at the crosshair.
- **Redundant channels for critical state.** Low health is color *and* a vignette *and* a sound *and* a shape change. Roughly 1 in 12 men has a color-vision deficiency; competitive players play with saturation altered.
- **Diegetic when it serves the fiction, not when it costs clarity.** An ammo counter on the weapon is beautiful and unreadable at speed; if you commit to it, prove it under real motion.
- **Input-adaptive prompts.** Show the glyph for the device in use, switching live. A keyboard key shown to a gamepad player is a bug.
- **Safe area and overscan.** Console TVs still crop; nothing critical within ~5% of the edge. Design for a 27" monitor at 60cm *and* a TV at 3m — the same layout must work at both angular sizes.
- **Motion has a budget and a hard rule:** the HUD never competes with gameplay motion. Animate on state change, settle fast, and stop. Persistent idle animation in the periphery is a torture device.
- **Menus are a different mode.** Pause menus, inventories, and settings are Operate: dense, keyboard/gamepad navigable, with a predictable focus model and no reliance on pointer hover.
- **Accessibility is table stakes now:** HUD scale slider, colorblind modes that change shape not just hue, subtitle size and background, reduced screen shake, and remappable controls. Ship them as design, not as an options-menu afterthought.

## Failure modes

| Symptom | Real cause |
|---|---|
| HUD disappears in a bright level | Contrast solved against one background only. |
| Players miss low health | Signal is a color change with no motion, size, or audio channel. |
| Menu unusable on gamepad | Built pointer-first; no focus model, no wrap, no default selection. |
| Frame drops in busy scenes | Per-frame layout or allocation in immediate-mode UI; unbatched draws. |
| Unreadable on a TV | Designed at monitor viewing distance; text below ~2.5% of screen height. |

## Audit hooks

Brightest and darkest levels; heaviest particle scene; 5% safe area; gamepad-only traversal; TV viewing distance; each colorblind mode; HUD scale at min and max; localized strings at 40% longer; frame-time cost of the UI pass alone.
