# motion — purpose inside the budget

Motion exists to explain change: where something came from, what belongs to what, what is happening now. Motion that explains nothing is a cost paid in attention, battery, and frames.

## The four legitimate jobs

1. **Continuity** — the element that was there is the element that is here. Transform it; do not cross-fade unrelated things.
2. **Causality** — the thing you touched produced the thing that appeared. Origin matters: a menu grows from its trigger.
3. **Status** — something is happening that you did not initiate. Loading, receiving, syncing.
4. **Attention** — exactly one thing, rarely, when the consequence justifies it.

Anything else — entrance animations on static content, hover flourishes on every card, staggered reveals of a list the user is trying to read — is decoration and belongs to Persuade and Experience modes only.

## Durations and easing

| Kind | Duration | Easing |
|---|---|---|
| Micro state (hover, toggle, press) | 80–150ms | ease-out |
| Element enter/exit | 150–250ms | enter ease-out, exit ease-in |
| Layout or view transition | 250–400ms | ease-in-out, or a spring |
| Attention pulse | 400–600ms, once | ease-in-out |
| Anything above 500ms | needs a reason | — |

- **Ease-out for arrivals** (fast start, soft landing — feels responsive), **ease-in for departures**, **ease-in-out for movement between two known points**. Linear is for continuous indeterminate motion only (spinners, marquees).
- **Distance scales duration,** but sub-linearly. A modal crossing the screen is not 10× a chip's toggle.
- **Springs read as physical** and handle interruption gracefully; use them where the user can grab the thing mid-flight. Tune by mass/stiffness, not by copying values.

## Interruption

Every animation must be interruptible and must resolve from wherever it is. A user who taps twice must not queue two animations or snap back to the start. This is the difference between motion that feels alive and motion that feels like a cutscene.

## Reduced motion

The preference is not "make it faster." Replace movement with an instant change or a short opacity fade, keep everything that conveys state, and remove parallax, auto-play, looping, and large-area movement entirely. Anything that only exists as motion needs a static equivalent.

## Per-stack budget

- **Web:** compositor-only properties in loops; measure, do not assume 60fps.
- **Mobile:** motion drains battery and thermals; transitions must mirror navigation direction.
- **Desktop:** long sessions — halve everything decorative.
- **Terminal:** 1–10fps, made of characters, redraw only what changed.
- **Embedded/e-ink:** effectively none; state changes are instant.
- **Game/XR:** the UI never competes with world motion, and uninitiated camera movement is forbidden in XR.

## Rules

- **Animate the property that changed,** not the whole element.
- **Never gate information behind an animation.** If the number is there, show it.
- **One authored moment per surface** beats six competing effects.
- **An entrance must degrade to appearing, not to absence.** Put the visible
  state in the base rule and let the animation supply only the *from*-state.
  Written the other way round — the hidden state as the resting style, the
  visible one applied by the animation — it looks identical while the motion
  runs and inverts the failure. Anything that skips the transition (an
  unsupported entry-animation feature, a stalled compositor, a paint the engine
  never scheduled) leaves the element at its resting style, which is now
  invisible. On a modal that means a focus-trapped, scroll-locked surface with
  nothing drawn on it and no visible way out. Which way round costs nothing
  while it works and decides everything when it does not, so the question to
  ask of any entrance is: **if the motion never runs, what is on screen?**
