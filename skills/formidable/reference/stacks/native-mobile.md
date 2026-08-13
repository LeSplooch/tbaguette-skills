# Stack: native mobile

**Envelope.** One hand, in motion, interrupted, on a screen between 4.7" and 7", with a notch, a home indicator, a keyboard that eats half the screen, and an OS with strong opinions. Battery and thermals are real. The user can revoke permissions at any moment.

## Idioms a native user expects

- **Platform navigation, not your own.** Back gesture on Android and its predictive preview; swipe-back on iOS; tab bars stay put; a modal is dismissible the way that platform dismisses modals.
- **System components where they exist** — pickers, share sheets, permission dialogs, keyboards. A custom date picker is almost always worse than the OS one.
- **Respect the safe areas** and the dynamic insets. Nothing important under the notch, the status bar, the home indicator, or the IME.
- **Dynamic Type / font scale is honored.** Layout reflows; it does not clip. This is the single most-failed mobile requirement.
- **Haptics mean something.** Confirmation, selection, and failure are distinct. Haptics for decoration is noise.

## Envelope-specific craft

- **Reachability.** Primary actions belong in the lower half. The top-right corner is the hardest pixel on the device.
- **Touch targets ≥44pt/48dp**, with spacing between them — adjacency errors are worse than small targets.
- **Design the interrupted state.** Incoming call, backgrounding, rotation, low battery, lost connection mid-flow. State survives or the user is told plainly what was lost.
- **Offline is a state, not an error.** Show what is cached, mark it stale, queue the write, and reconcile.
- **Permission priming.** Never fire a system permission prompt cold. Explain the value in your own UI first, request in context, and design the denied path as a first-class flow — it is permanent on most platforms.
- **Motion is navigational.** Transitions communicate hierarchy: push means deeper, modal means aside, and the reverse must mirror. Decorative motion drains battery and patience.
- **Lists are the app.** Design the row, the separator, the swipe actions, the selected state, the empty list, the loading skeleton, and the 10,000-item scroll performance before designing the screen.

## Failure modes specific to this stack

| Symptom | Real cause |
|---|---|
| Text clipped for some users | Font scale ignored; fixed-height containers around text. |
| Jank on scroll | Work in the row builder; unbounded image decode; shadow/blur per cell. |
| "Where did my data go" | State not restored after process death, not just backgrounding. |
| Users abandon at a permission | Cold prompt with no context, or no designed denial path. |
| Keyboard covers the field | No inset handling; scroll container not resized on IME show. |

## Audit hooks

Largest font scale; smallest supported device; both orientations if supported; dark mode; screen reader traversal order and labels on icon-only buttons; keyboard-open layout; airplane mode; permission denied; cold launch after process death.
