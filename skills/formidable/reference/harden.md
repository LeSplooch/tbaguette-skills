# harden — make it survive reality

The design works on your data, in your language, on your device, with your network. Hardening is the work of making it survive everyone else's.

## The hostile inputs

Run the surface against each. Every one that breaks is a finding.

- **Length.** The empty string, one character, and the longest realistic value. Names, titles, emails, URLs, file paths, error messages. Then +40% for translation. Then a single unbroken 200-character token with no spaces.
- **Count.** Zero items, one item, exactly enough to fill the container, and ten thousand.
- **Script and direction.** CJK (double-width, no word breaks), Arabic and Hebrew (RTL mirroring of layout, icons, and progress), Devanagari (tall stacks that clip fixed line heights), Turkish (dotless i in case conversion), combining marks, emoji with modifiers.
- **Numbers, dates, currency.** Locale separators, 24h vs 12h, non-Gregorian calendars, timezone display, negative values, zero, and very large magnitudes. Never hardcode a format.
- **Network.** Offline, slow, flapping, and succeeded-but-slow. Every request has a designed pending, failure, retry, and give-up state.
- **Permission and auth.** Denied, revoked mid-session, expired, and insufficient-role. Each is a designed screen, not a crash or a blank.
- **Environment.** Largest font scale, smallest screen, both themes, high contrast, reduced motion, reduced transparency, screen reader on, and the platform's oldest supported version.

## Error design

Every error answers three questions in this order: what happened, what it means for me, what to do now. Then optionally: how to get help, with an identifier that support can use.

- Name the specific thing that failed, not the layer that reported it.
- Never blame the user. "That email is already registered" not "Invalid input."
- Recovery must be reachable from the error itself — a button, a retry, a link, a suggested value.
- Distinguish transient (retry helps) from permanent (retry is cruel).
- Preserve what the user typed. Losing a filled form to a validation error is the most expensive error-handling failure there is.

## Rules

- **Every state you cannot reach in the UI, force in code** and screenshot it. Untested states are undesigned states.
- **Truncation is a decision.** Which end, with what indicator, and with the full value available how. Middle-truncate paths and IDs; end-truncate prose; never truncate a number.
- **Do not fix by shrinking text.** Auto-shrinking to fit is a failure of layout, and it fails at the next locale.
