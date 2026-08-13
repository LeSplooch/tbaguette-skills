# Stack: dense data — tables, dashboards, monitors

**Envelope.** The content is the design. The reader is comparing, scanning, and deciding, usually under time pressure and often repeatedly all day. Space is scarce because information is the point. Any pixel spent on decoration is stolen from data.

**Density is a feature.** The instinct to add whitespace, cards, and rounded containers destroys these interfaces. Tufte's ratio holds: maximize the share of ink that carries information.

## Craft

- **Rank before you render.** Decide the one question the surface answers, the three supporting facts, and the rest. A dashboard where every tile is the same size answers nothing.
- **Tables:** tabular figures, right-aligned numbers, left-aligned text, consistent decimal places, units in the header. Zebra striping only past ~7 columns — otherwise a hairline or nothing. Sticky header and sticky first column. Sortable columns show the current sort. Row height comfortable *and* compact.
- **Never make the reader do arithmetic.** Show the delta, the percentage, and the total if that is what they are computing. Add a comparison baseline to every number that has one — target, prior period, peer.
- **A number without context is decoration.** Sparkline, trend arrow with magnitude, or a range. "Revenue: 1.2M" is not information; "1.2M, +8% vs last month, above target" is.
- **One accent color, plus an ordered severity ramp.** Categorical colors max out around 7 distinguishable series, and fewer for colorblind readers — use direct labels instead of a legend, and reserve saturated color for what is wrong.
- **Y axes start at zero for bars.** Truncated axes on bars lie. Lines may truncate if the baseline is labeled.
- **Chart type follows the question:** trend over time → line; comparison across categories → bar, sorted by value not alphabetically; part-to-whole → stacked bar or a plain table (rarely a pie, never more than 3 slices); correlation → scatter; distribution → histogram or box plot. A donut with a number in the middle is a number.
- **Alerts must be rare to mean anything.** Every always-red element is one the reader has stopped seeing. Design thresholds, grouping, and suppression as part of the interface.
- **Show state honesty:** stale data, partial results, sampling, timezone, and the exact query window. A dashboard that silently shows old numbers is worse than one that is down.
- **Empty, one row, and one million rows** are three distinct designs. Virtualize, paginate, or aggregate — never render 50,000 DOM rows and call it done.

## Failure modes

| Symptom | Real cause |
|---|---|
| Users export to a spreadsheet immediately | The table cannot sort, filter, compare, or show enough rows. |
| Nobody notices real incidents | Alert fatigue: too many always-on warnings. |
| Numbers misread | Mixed decimal places, proportional figures, unlabeled units, truncated bar axes. |
| Beautiful and useless | Tiles ranked by layout convenience rather than by decision value. |
| Slow and janky | No virtualization; re-rendering the whole grid per update; per-cell formatting cost. |

## Audit hooks

Zero rows, one row, and the realistic maximum; longest possible cell content; a colorblind simulation; grayscale; the smallest supported width; stale/failed data source; a full workday of continuous viewing for alarm fatigue; keyboard-only navigation of the grid.
