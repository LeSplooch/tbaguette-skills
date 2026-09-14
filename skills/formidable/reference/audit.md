# audit — mechanical checks

Critique judges. Audit measures. Every finding here is verifiable, and you verify it rather than estimating it.

## Universal checks

Run against the built result. Where a stack cannot support a check, the stack file names the substitute.

| Check | Threshold | How |
|---|---|---|
| Text contrast | ≥4.5:1 body, ≥3:1 large (≥18.7px regular / ≥14px bold) | Compute from rendered colors, including over gradients and images |
| Non-text contrast | ≥3:1 for icons, borders, focus rings, chart marks | Same, against adjacent color |
| Color independence | Every color-coded meaning has a second channel | Desaturate the render and re-read it |
| Focus visibility | Every interactive element, ≥3:1 against both adjacent surfaces | Traverse with keyboard/D-pad only |
| Traversal order | Matches visual reading order; no traps | Tab/next through the whole surface and back |
| Target size | ≥44×44pt touch, ≥24×24px pointer, plus spacing | Measure the hit area, not the visual |
| Text scaling | Legible and uncropped at 200% | Raise the platform font scale and re-render |
| Reflow | No loss of content or function at the minimum supported size | Resize to the floor |
| Names and roles | Every control has an accessible name; icon-only buttons especially | Screen reader or accessibility inspector |
| Motion | Honors reduced-motion; nothing >3 flashes/sec | Toggle the OS setting |
| Latency | Feedback <100ms, progress by 1s, escape by 10s | Measure with the slowest realistic input |
| Longest string | No clipping or overlap at +40% text length | Substitute the longest real or translated string |
| Empty / error / offline | Each renders something designed | Force each state |

## Output shape

A table of findings: check, measured value, threshold, location, severity. Severity is **blocker** (excludes users or loses data), **defect** (measurably wrong), or **note**. Then a one-line summary count. No prose narrative.

## Rules

- **Measure, do not estimate.** "Looks like enough contrast" is not an audit result. If you cannot measure it in this environment, say the check was not run rather than passing it.
- **Report the location precisely** — file and line, or element and state.
- **Do not fix while auditing.** Produce the list first; fixing mid-scan produces an incomplete scan.
- **A passed check is worth reporting** so the next audit knows what was covered.
- **A capture is a measurement of whatever owned the surface at that instant.** Address every screenshot, recording, or accessibility dump to the thing under test — a window id or class, a package and activity, a tab, a display — never to "the active window" or "the current screen". Focus-based capture is a race with everything else on the machine, and a shared instrument (a phone on a desk, a browser profile, a display, a staging box) can change hands between two steps of a scripted pass: another session deploys its own build, a dialog lands on top, the user alt-tabs. The capture still succeeds, and it is a correct picture of something else. So read the foreground owner at capture time, record it beside the capture, and abort the step when it is not the subject. Abort rather than relaunch on top: that foreground is another run's state, and `orchestrating-work-end-to-end`'s Crew dial says what to do instead. A screenshot without that line is a screenshot of whatever was there. The obvious wrong-app frame is the lucky case; a same-looking app or a dialog on top produces a plausible capture of the wrong thing, and every finding read off it is a finding about a surface nobody was auditing.
- **Make every state reachable from a script, then capture them all in one round.** Give the build a flag, an environment variable, or a URL parameter that selects theme and state, wait for the window or activity to exist rather than sleeping, drive it with synthetic input, capture by id, and batch the states into one contact sheet for the single inspection round the bounded-pass rule prescribes. A state that can only be reached by hand is a state the audit will skip; a state that lives between two events is reached only by an input held for the length of a real press — [craft-floor.md](craft-floor.md) says why.
