# Stack: XR and spatial

**Envelope.** The interface exists in physical space around a person whose comfort you can break. Resolution is low per degree, text is expensive, the passthrough or scene behind your UI is arbitrary, and the user's head is the camera. Sessions are short because they are tiring. Mistakes here cause nausea, not just annoyance.

**Comfort outranks every aesthetic goal.** A beautiful interface that induces sickness is a failed interface.

## Craft

- **Angular size, not pixels.** Design in degrees of visual field. Body text needs roughly ≥1.5° of height; interactive targets need ≥2–3° with generous spacing. A "16px" measurement is meaningless here.
- **The comfort zone is a band,** not the whole sphere. Content sits within roughly ±30° horizontally and slightly below eye level, at 1–3m of apparent depth. Anything that requires sustained neck rotation or upward gaze is a design error.
- **Never lock UI to the head.** Head-locked elements swim and nauseate. Use body-locked or lazy-follow panels that settle after the head stops, or world-locked panels the user places.
- **Depth is a hierarchy channel — use it sparingly.** Conflicting depth cues (a near panel occluded by a far one, text at a different depth than its background) break fusion and cause eye strain. Keep a panel's contents at one depth.
- **Contrast against unknown reality.** In passthrough, your background can be a white wall or a window. Give panels their own opaque or strongly tinted substrate; pure additive rendering vanishes over bright surfaces.
- **Motion rules are safety rules.** No camera movement the user did not initiate. No acceleration. Vignette during locomotion. Offer teleport as well as smooth movement, snap turning as well as smooth turning, and a seated mode. Honor every comfort setting.
- **Input is imprecise and fatiguing.** Gaze-and-pinch, ray-cast, and hand tracking all have jitter and no tactile stop. Enlarge targets, add dwell forgiveness, snap to targets, and never require sustained arm elevation ("gorilla arm"). Confirm destructive actions.
- **Text is the weakest element.** Prefer few large words, high weight, and generous tracking. Long-form reading is the wrong use of the medium — offload it or paginate it.
- **Ground the user.** Persistent spatial anchors, a visible floor or horizon, and stable reference geometry reduce disorientation.
- **Design the boundary.** What happens when the user walks away, turns around, loses tracking, or the panel is behind them — a recall gesture is mandatory.

## Failure modes

| Symptom | Real cause |
|---|---|
| Nausea | Uninitiated camera motion, acceleration, head-locked UI, low frame rate. |
| Eye strain | Mixed depths within one element; text too small; convergence conflict. |
| Users can't hit things | Real target smaller than input jitter; no snapping; no dwell tolerance. |
| Panel invisible | Additive blending over a bright real-world background. |
| Arm fatigue in minutes | Targets placed high; sustained precision required. |

## Audit hooks

Frame rate never below the headset's floor; every comfort setting on and off; standing and seated; a bright room and a dark one; a user who turns 180°; tracking loss; the longest localized string at the same angular size.
