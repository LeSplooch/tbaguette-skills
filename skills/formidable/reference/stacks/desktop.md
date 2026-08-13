# Stack: desktop native

**Envelope.** Large, resizable, multi-window, multi-monitor with mixed DPI. Keyboard and pointer are both first-class. Sessions are long and work is real. The OS supplies conventions the user has trusted for decades — violating them reads as amateurism, not innovation.

## Idioms a native user expects

- **The menu bar is real** and complete: every command reachable, every shortcut discoverable there, standard items in standard places.
- **Keyboard everything.** Tab order, mnemonics/access keys, standard shortcuts unchanged (copy, paste, find, save, close, quit), and a visible focus ring the whole way.
- **Window behavior:** resizable to a sensible minimum, remembers size and position per monitor, restores sanely when a monitor disappears, and supports the platform's full-screen and tiling.
- **Right-click has a context menu** anywhere a contextual action exists.
- **Undo is expected**, not exceptional, and it is multi-level for anything document-shaped.
- **Native title bar, native scrollbars, native controls** unless the design commits fully and consistently to replacing all of them.

## Envelope-specific craft

- **Density is a preference.** Desktop users trade whitespace for information; a mobile-density layout stretched to 2560px is the most common desktop design failure. Design comfortable and compact variants.
- **Resizing is a design state.** Decide what grows, what stays fixed, what wraps, what hides into an overflow. Test at the minimum window size and at ultrawide.
- **DPI-independent everything.** Vector icons, scalable spacing units. Test with mixed-DPI monitors — drag the window between them.
- **Long sessions demand restraint.** Lower saturation, gentler motion, no attention-grabbing that repeats. Something that delights once per session is a torment at 200 repetitions.
- **Multi-selection, drag-and-drop, and bulk actions** are expected wherever a list of objects exists.
- **Respect the system theme,** including accent color and high-contrast modes, and offer an override.

## Failure modes specific to this stack

| Symptom | Real cause |
|---|---|
| "Feels like a website" | Web spacing, no menu bar, no shortcuts, no context menus, non-native scroll. |
| Blurry on one monitor | Raster assets or DPI captured once at launch. |
| Unusable at small window | Fixed minimums, no overflow strategy, no wrap. |
| Power users complain it is slow | It is not slow — it is mouse-only. Add keyboard paths. |
| Lost work | No undo, no autosave, destructive action with no confirmation or no reversal. |

## Audit hooks

Full keyboard traversal with no pointer; minimum and maximum window size; both light and dark system themes; high-contrast mode; mixed-DPI monitor drag; screen reader on the main window; every menu command has a reachable UI path and vice versa.
