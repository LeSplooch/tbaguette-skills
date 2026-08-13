# Stack: print and generated documents

**Envelope.** Fixed page geometry, no interaction, no reflow, and no chance to fix it after it is printed or archived. Color may be CMYK or grayscale or a fax. The reader may scan, skim, file, or be legally bound by it. Page breaks are the whole layout problem.

## Craft

- **Design the page, then the flow.** Set the grid, the margins, and the baseline first. Generated documents that ignore the page produce orphans, split tables, and a signature block alone on page 4.
- **Break control is the craft.** Keep headings with their content, keep table headers repeating across pages, forbid single-line orphans and widows, keep a total with its rows, and never split a signature block or a labeled figure from its caption.
- **Type for paper, not for screen.** Serif or a high-contrast sans at 9–11pt for body, 1.3–1.45 leading, 60–80 characters per line, and true small caps/oldstyle figures if the face has them. Hairline strokes and pale grays vanish in print and in fax.
- **Grayscale is the fallback.** Anything encoded in color must survive being printed in black and white — use pattern, weight, or label as the second channel. Test by desaturating.
- **Numbers align.** Right-align numeric columns, use tabular figures, fix decimal places, and put the unit in the header rather than in every cell.
- **Every page carries orientation:** document title or section, page N of M, a date or version, and a document identifier. A page found alone on a printer must identify itself.
- **The first page answers the question.** Invoices lead with the amount and due date; reports lead with the finding. Do not make the reader assemble the summary from the body.
- **Bleed, safe margins, and binding.** Full-bleed art extends past the trim; nothing critical within the safe margin; add gutter margin if it will be bound or hole-punched.
- **A PDF is also a digital artifact:** selectable real text (not an image of text), tagged structure and reading order, bookmarks for anything over a few pages, embedded and subsetted fonts, sensible metadata, and no invisible draft layers.

## Failure modes

| Symptom | Real cause |
|---|---|
| Table header on page 1 only | Repeat-header not set on the generated table. |
| Signature block stranded | No keep-together rule on the closing group. |
| Chart unreadable when printed | Color-only encoding; light grays below print threshold. |
| Not searchable, not accessible | Text rendered as vectors or images; untagged output. |
| Wrong on someone else's printer | Fonts not embedded; margins inside the hardware's unprintable area. |

## Audit hooks

Print it, physically, on the cheapest printer available; grayscale; both A4 and Letter; the longest realistic data set (a 40-page table); a one-row table; a document with no data; text selection and search; a screen reader on the PDF.
