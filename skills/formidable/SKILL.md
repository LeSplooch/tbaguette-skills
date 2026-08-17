---
name: formidable
description: Use when designing, redesigning, critiquing, auditing, polishing, or hardening any user interface on any stack — web, native mobile, desktop, terminal or TUI, CLI output, game HUD, embedded and e-ink displays, XR and spatial, email, print and PDF, or voice and chat. Covers visual hierarchy, layout, spacing, typography, color, motion, iconography, information density, interaction states, accessibility, latency and perceived performance, theming and tokens, UX copy, error and empty states, internationalization, and cross-stack design systems. Also use when an interface feels generic, bland, cluttered, dated, or inconsistent, or when a design must be carried faithfully from one stack to another.
user-invocable: true
argument-hint: "[shape|audit|critique|polish|harden|densify|calm|animate|typeset|colorize|port|tokens|onboard] [target]"
---

# Formidable

Design that earns to be called out-of-distribution craft — on **any** surface a human looks at, not just a browser window. Whereas your default UI work is safe, web-shaped, and measured, here you work as a design director who has shipped on all of them: someone who knows that a terminal has a type scale, that a HUD has a reading order, that a CLI's error message is interface, and that none of these are excuses for mediocrity.

Core principles:

- **Go all out.** The deliverable is complete — every state, every breakpoint or terminal width, every theme. Not a sketch with TODOs.
- **The constraint envelope is the medium, not the excuse.** 16 colors, 80 columns, 200ms of budget, no shadows, no motion — these are the material. A great design is one that could only exist in that envelope.
- **Verify in bounded passes.** Build fully, inspect once in a batched round covering every size and theme together, fix everything it shows in one batch, confirm with at most one more round, stop. Open-ended self-QA burns budget doing worse what a real review does better — `knowing-when-to-stop` covers this bounded-pass discipline generally; this is its application to a design surface specifically.

## First move: name the envelope

Before any design decision, establish the stack and its constraint envelope. Never assume web. Never assume the stack from the language — a Rust project may be a TUI, a game, or a web server.

1. **Detect.** Look for the rendering authority: view/template/component files, a UI framework in the manifest, terminal escape or widget calls, a shader or canvas, a mail template, a prompt template. When two stacks are present, ask which surface this task is about.
2. **Load the stack file.** Exactly one from the table below. It supplies the envelope — what the medium can express and what it cannot — and the idioms a native user of that stack expects.
3. **Find the incumbent visual truth.** Tokens, theme file, stylesheet, palette constant, widget defaults, existing screens. Inspect at least one before editing. A project with no design file is not automatically greenfield.
4. **Then design.** Load [reference/craft-floor.md](reference/craft-floor.md) immediately before you edit UI — never for planning-only work.

| Stack | Load |
|---|---|
| Browser, web app, site, webview, Electron renderer | [stacks/web.md](reference/stacks/web.md) |
| iOS, Android, React Native, Flutter, mobile | [stacks/native-mobile.md](reference/stacks/native-mobile.md) |
| Desktop native — GTK, Qt, WinUI, AppKit, Tauri, Swing | [stacks/desktop.md](reference/stacks/desktop.md) |
| Terminal UI, full-screen text app, curses, TUI framework | [stacks/terminal-tui.md](reference/stacks/terminal-tui.md) |
| CLI output, logs, help text, progress, diagnostics | [stacks/cli-output.md](reference/stacks/cli-output.md) |
| Game HUD, overlay, in-engine menus, immediate-mode UI | [stacks/game-hud.md](reference/stacks/game-hud.md) |
| Embedded, e-ink, small LCD, watch, appliance, kiosk | [stacks/embedded-display.md](reference/stacks/embedded-display.md) |
| XR, AR, VR, spatial, headset, heads-up | [stacks/xr-spatial.md](reference/stacks/xr-spatial.md) |
| Email — HTML, templated, transactional | [stacks/email.md](reference/stacks/email.md) |
| Print, PDF, report, invoice, generated document | [stacks/print-pdf.md](reference/stacks/print-pdf.md) |
| Voice, chat, conversational, agent replies, notifications | [stacks/voice-chat.md](reference/stacks/voice-chat.md) |
| Dense data — tables, dashboards, monitors, spreadsheets | [stacks/data-dense.md](reference/stacks/data-dense.md) |

Multiple surfaces of one product: design each in its own envelope, share the **decisions** through tokens, and never share the **implementation**. See [reference/porting.md](reference/porting.md).

## Modes

The mode names what success looks like for the person in front of the surface. Choose it from the surface, not the product — a database tool's landing page is still Persuade; a fashion brand's API docs are still Read.

- **Persuade** — they decide and act. Design is the product. Earn attention, then earn the action.
- **Operate** — they complete a task. Scanability, consistency, native expectation, and the real usage scene outrank expression. Brand lives in precise details.
- **Read** — they understand something. Structure for comprehension first, then make staying worth it.
- **Experience** — they are inside the work. The interface recedes and the artifact leads.
- **Attend** — they glance, in motion, under load, possibly in danger. Legibility at arm's length in one second, ranked by consequence. Dashboards in cars, HUDs, watches, alerting, wearables, control rooms. Decoration here is a defect.

Attend is the mode most often missed. If the person will not be sitting still and looking directly at the surface, it is Attend, whatever the stack.

## Commands

| Command | Category | Does |
|---|---|---|
| `shape [feature]` | Build | Decide UX and structure before code. [reference/shape.md](reference/shape.md) |
| `tokens` | Build | Establish or extract a cross-stack token system. [reference/tokens.md](reference/tokens.md) |
| `onboard [target]` | Build | First-run, empty, zero-data, and permission states. [reference/onboard.md](reference/onboard.md) |
| `critique [target]` | Evaluate | Design review against heuristics, with a verdict. [reference/critique.md](reference/critique.md) |
| `audit [target]` | Evaluate | Mechanical checks: contrast, targets, focus, reflow, latency. [reference/audit.md](reference/audit.md) |
| `polish [target]` | Refine | Final craft pass before shipping. [reference/craft-floor.md](reference/craft-floor.md) |
| `harden [target]` | Refine | Errors, edge cases, i18n, long strings, failure modes. [reference/harden.md](reference/harden.md) |
| `densify [target]` | Refine | Raise information density without raising effort. [reference/stacks/data-dense.md](reference/stacks/data-dense.md) |
| `calm [target]` | Refine | Reduce noise, motion, color, and alarm fatigue. [reference/calm.md](reference/calm.md) |
| `animate [target]` | Enhance | Purposeful motion inside the stack's real budget. [reference/motion.md](reference/motion.md) |
| `typeset [target]` | Enhance | Type scale, measure, rhythm, and voice. [reference/type.md](reference/type.md) |
| `colorize [target]` | Enhance | Palette with meaning, in the stack's color space. [reference/color.md](reference/color.md) |
| `port [target]` | Adapt | Carry a design to another stack faithfully. [reference/porting.md](reference/porting.md) |

Routing:

- **Explicit or clearly implied command:** load its reference plus the stack file, then follow it.
- **No command, general design request:** treat as ordinary design work — envelope, incumbent truth, craft floor, build.
- **Two commands fit:** ask once, then commit.
- **Ambiguous stack:** ask which surface. Never guess between web and native.

## Non-negotiables across every stack

These survive the envelope. A stack that cannot satisfy one owes an explicit substitute, not a shrug.

- **Contrast is measured, not eyeballed.** Body text ≥4.5:1, large text and meaningful non-text ≥3:1. In a 16-color terminal, a 1-bit display, or a HUD over unknown pixels, you still owe the equivalent guarantee — see the stack file.
- **Nothing is conveyed by color alone.** Every color-coded state carries a second channel: glyph, weight, position, label, shape, or sound.
- **Every interactive element has a visible focus state** reachable without a pointer, and a target big enough for the input device actually in use.
- **Every state exists:** loading, empty, partial, error, offline, permission-denied, too-long, too-many, zero, one, and enormous. Designing only the happy state is an unfinished deliverable.
- **Copy is interface.** Controls name their action. Errors name the problem *and* the recovery. No "An error occurred."
- **Perceived latency is design.** Under ~100ms feels instant; acknowledge by ~1s; explain and offer escape past ~10s. Meeting the budget beats animating the wait.
- **Motion respects the user's reduced-motion preference** wherever the platform exposes one, and never carries information that exists nowhere else.
- **Text can grow.** Translations run 30–40% longer than English, users raise font sizes, and data is longer than your placeholder. Layouts that only fit the sample string are broken.

## Refuse

Category defaults, not bans — a committed brief can earn any of them, but reaching for one when the axis is free means you were not deciding.

- Cards of icon + heading + text as the page structure. Nested cards are always wrong.
- The hero-metric template: big number, small label, supporting stats, accent.
- An eyebrow or kicker above a heading. This one is a ban. The heading carries itself.
- Emoji or arbitrary Unicode standing in for an icon system. Icons are drawn, consistent in stroke and weight — including in terminals, where the icon system is a chosen, width-checked glyph set.
- Gradient text; glass and blur as decoration rather than a specific effect; hard zero-blur offset shadows outside a world that is genuinely neobrutalist.
- Monospace as a costume for "technical" rather than for code, data, or alignment.
- Light or dark chosen by category instead of by the actual use scene — who, where, under what ambient light.
- Progress bars for unknowable durations, spinners for sub-300ms waits, and toasts for anything the user must act on.
- Rainbow status palettes where severity has no order. Severity is ordered; the palette must be too.
- Modals for tasks needing neither interruption nor protected focus.

With every check green, spend the surface on the committed direction. When torn between refined and committed, commit.
