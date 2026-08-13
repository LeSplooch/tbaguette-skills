# calm — reduce the noise

For surfaces that shout: too much motion, too much color, too many alerts, too many competing elements. Calming is subtraction, and subtraction requires knowing what the surface is *for*.

## Method

1. **Establish the one job** the surface serves. Everything is measured against it.
2. **Inventory the attention-grabbers:** animation, saturated color, badges, borders, shadows, all-caps, exclamation, bold, tooltips, banners, sounds, haptics, notifications. Count them. The count is usually the finding.
3. **Rank by consequence.** What actually costs the user something if missed? Almost nothing does. Everything else demotes.
4. **Demote in this order** — this is the cheapest-to-strongest ladder, run it backwards:
   remove → make static → reduce saturation → reduce size → reduce weight → move out of the primary scan path → group with siblings → keep as is.
5. **Restore hierarchy with space and size,** not with the color and motion you just removed. If the surface now reads flat, the fix is a bigger size ratio, not the accent coming back.

## Alarm fatigue

- An alert that is always on is not an alert. Either fix the condition or demote it to status.
- Severity must be ordered and scarce: if everything is red, nothing is.
- Group repeated alerts into one with a count. Ten identical warnings is one warning.
- Anything that interrupts must be actionable now. Otherwise it is a notification, and notifications belong in a list the user visits.
- Sound and haptics are the loudest channels you have. Reserve them for state changes with consequences.

## Motion restraint

- Delete idle and looping animation outside of an explicit loading state.
- One entrance per surface, not one per element. Staggered reveals of every card is an anti-pattern in Operate and Attend modes.
- Halve every duration you find above 300ms for functional transitions.
- Honor reduced-motion by replacing movement with a cross-fade, not by keeping the movement and shortening it.

## Rules

- **Calm is not gray.** Removing all color produces a dead surface, not a calm one. Keep one accent and mean it.
- **Do not calm by hiding.** Moving noise behind a menu the user must now hunt through trades one problem for a worse one. Remove or demote in place.
- **Long-session surfaces get the strictest treatment.** Delight that fires once per session is torture at 200 repetitions.
