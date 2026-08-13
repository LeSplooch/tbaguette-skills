# Stack: terminal / TUI

**Envelope.** A grid of character cells. Type is one family at one size; the only weights are normal, bold, dim, italic (unreliable), underline, and reverse. Color is 16 named entries the user has themed, 256 indexed, or 24-bit — and you cannot know which until you ask. No sub-pixel positioning, no shadows, no blur. Redraw is cheap but flicker is real. The window is any size from 40×10 up.

**This is not a degraded medium.** It is a typographic grid with a fixed unit — closer to letterpress than to a browser. Treat the cell as your unit of measure and the constraint produces precision, not poverty.

## Idioms a native user expects

- **Ctrl-C quits or cancels. Always.** Never trap it without a visible reason and an alternative.
- `q`, `Esc`, and `Ctrl-C` all leave something; `?` shows help; arrow keys and `hjkl` both move where a list exists.
- The terminal is **restored on exit** — alternate screen closed, cursor shown, colors reset, mouse mode off — including on crash and on `SIGTERM`.
- Output is still pipeable. If the program can be redirected, detect it and drop to plain text.
- Resize is handled live, not on next keypress.

## Envelope-specific craft

- **Use the theme's named colors, not hardcoded hex.** `red` in the user's palette is their red. Hardcoding `#ff0000` fights a Solarized or high-contrast user and loses. Reserve 24-bit for a deliberate branded surface, and check `COLORTERM`/`NO_COLOR`/`TERM` first.
- **Hierarchy comes from four levers only:** weight (bold/dim), position, space, and rule characters. Learn to build a full hierarchy with them before reaching for color — then color is emphasis rather than the only structure.
- **Space is the strongest tool.** A blank line between groups outperforms every box-drawing character. Boxes around everything is the terminal equivalent of card soup.
- **Box-drawing is a font decision.** Pick one set (light, heavy, double, or rounded) and never mix. Verify the glyphs exist in common terminal fonts, and provide an ASCII fallback for `TERM=dumb` and for locales without them.
- **Every glyph has a width.** CJK and many emoji are double-width; combining marks are zero-width; some emoji vary by terminal. Compute display width, never string length, or every table you draw will drift.
- **Columns are a layout system.** Fixed columns for known widths, weighted flex for the rest, and one designated column that absorbs slack and truncates with an ellipsis. Decide truncation direction per column — paths truncate from the left.
- **Redraw only what changed.** Full-screen repaints flicker on slow links and burn power. Diff the cell buffer.
- **Motion is 1–10 fps, not 60.** Spinners, progress, and transitions are made of characters. Anything faster is noise over SSH.

## Failure modes specific to this stack

| Symptom | Real cause |
|---|---|
| Table borders drift | Counting characters instead of display width; CJK/emoji/ANSI codes in the string. |
| Unreadable on someone's theme | Hardcoded colors, or foreground set without background (or vice versa). |
| Garbled after exit | Alt screen or mouse mode not restored on signal/panic. |
| Flicker | Full repaint per frame; no double buffering. |
| Broken at 80 columns | Designed at the author's 200-column window. 80 is the contract; 40 is the courtesy. |

## Audit hooks

80×24 and 40×12; `NO_COLOR=1`; a light-background theme; piped to `less` and to a file; over SSH with latency; with CJK and emoji content; on resize mid-render; `Ctrl-C` during every long operation.
