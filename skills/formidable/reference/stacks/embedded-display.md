# Stack: embedded, e-ink, and small displays

**Envelope.** Few pixels, often few colors, sometimes no backlight, frequently no GPU and little RAM. Refresh may cost hundreds of milliseconds and visible flashing (e-ink), or be limited by power budget. Input may be two buttons, a dial, a resistive touch panel, or nothing. The device may live in sunlight, in a dark room, on a wrist, or on a wall for a decade.

**Mode is Attend or Operate, never Persuade.** Nobody browses a thermostat.

## Craft

- **One screen, one job.** With this little space, hierarchy is not "make it bigger" — it is "cut the other things." Decide the single value the glance is for, give it the majority of the display, and demote everything else to a caption.
- **Design at 1:1 in real pixels.** A mock scaled 8× lies about legibility. Bitmap fonts and hinted small sizes beat scaled vector type below ~14px.
- **Refresh is a design material.** On e-ink, budget partial refreshes, accept ghosting, and schedule the full flash where it will not startle. Never animate. On a slow LCD, avoid full-screen redraws for a single changed digit.
- **Contrast under real light.** Sunlight readability is a reflectance problem: maximize black-on-white, avoid mid-grays, and never use color as the only differentiator on a 1-bit or 3-color panel. In a dark room, the same design must not be a flashlight — offer a dark or dimmed variant.
- **Dithering is your gradient.** With 2–4 levels, ordered dithering and hatching provide texture and separation. Use them deliberately, at a consistent scale.
- **Input poverty shapes the IA.** Two buttons means a shallow list and a long-press. A dial means ordered options and a confirm. Never design a menu deeper than the input can traverse in a few seconds.
- **The device shows something when it fails.** No network, no sensor, stale data, low battery, and mid-update all need a designed screen. A frozen last-good reading that looks live is dangerous.
- **Stale data must look stale.** A timestamp, a dimmed value, or an explicit marker. This is the most consequential rule on any always-on display.
- **Burn-in and power.** Static bright regions damage OLED and cost battery; shift pixels periodically, prefer dark backgrounds on emissive panels, prefer light on reflective ones.

## Failure modes

| Symptom | Real cause |
|---|---|
| Unreadable outdoors | Mid-gray text; low reflectance contrast; backlight assumption. |
| Looks fine in the mock, mush on device | Designed scaled up; anti-aliased vector type at 10px. |
| Users trust a wrong number | No staleness indicator, no error screen, last value frozen. |
| Ghosting and flashing | Partial-refresh budget unmanaged on e-ink. |
| Dead battery | Full refreshes, bright static areas, or a polling cadence chosen without a power budget. |

## Audit hooks

The physical device, at 1:1, in direct sun and in darkness; every failure screen; longest realistic string; lowest battery state; after 24h of continuous display; traversal with the actual input hardware only.
