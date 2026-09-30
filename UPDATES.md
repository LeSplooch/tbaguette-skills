# Update notes

What changed, newest first, written for someone who has TBaguette installed
rather than for someone reading the diff. The landing page renders the most
recent entries below the “Fresh from the oven” rail. This file keeps the newest
thirty; older ones are in [`updates-archive/`](updates-archive/), one file per
month, and together they are the whole record.

**Scope: the plugin, and only what a user of it would notice.** An entry earns
its place if someone who has TBaguette installed would act differently for
having read it — a new skill, a skill that now says something different, a
change to how skills are named, grouped, or installed. Everything else stays
out, and the two categories that keep trying to get in are this repo's own
tooling (test suites, build gates, registries, version bumps) and the showcase
site's furniture (its layout, its search, its animations, its chrome). Both are
real work and neither is news about the plugin. They belong in the commit log.

The test is the reader, not the effort: a change can be the hardest thing
shipped that week and still not belong here, and a one-line fix to a skill's
wording can belong here absolutely.

Writing an entry is part of shipping a change here — see
CLAUDE.md. The shape is `## YYYY-MM-DD — Title` followed by `-` bullets, newest date first, and
`scripts/generate.py` refuses to build a site if that shape or that order
breaks. A bullet may wrap across lines; the continuation is joined back on.
Everything above the first `##` is preamble and is never rendered.

New entries always go here, at the top. Once this file passes 128 KiB the build
stops and asks for `python3 scripts/archive_updates.py`, which moves everything
past the newest thirty into its month's archive, word for word. The ceiling is
there because this file ships in the plugin, and Anthropic's plugin directory
holds any text file over 256 KiB for a reviewer.

## 2026-09-30 — Five quiet failures: a frozen reading, an "off" that isn't, a fence with two jobs, a plan nobody ran, a history cut short

- `diagnosing-before-fixing` now names a culprit behind "nothing changed":
  a tool that writes to a fixed path can fail, exit 0 and leave the last
  run's file behind, so every reading is the first one replayed. Clear the
  output before each run, or check its timestamp.
- `designing-apis` covers the calling side of defaults: leaving a parameter
  out asks for the provider's default, which may be on. To turn something
  off, send the explicit off value and check the provider's usage figures,
  not just the absence of an error.
- `deleting-code` has a section for removing a confirmation, consent or
  approval step because it is tedious: list everything that cites it as its
  justification first, since it often carries a second job.
- `structuring-an-implementation-plan` adds a self-review check: when a plan
  carries both code and the tests it must pass, run them together in a
  scratch directory before handing the plan over.
- `code-archaeology` now checks whether a clone is shallow before trusting
  its history. CI runners and agent sandboxes often fetch only recent
  commits, and in such a clone the oldest commit looks like an import of the
  whole tree while blame pins every older line on it. Check with
  `git rev-parse --is-shallow-repository` (or look for `grafted` in
  `git log --decorate`) and fetch the rest before concluding anything.

## 2026-09-29 — The Atelier in Claude Desktop, claude.ai and Cowork

- Those apps install plugins on your Claude account, not from a folder, so the
  clone command never reached them. They can take the Atelier today as a
  marketplace you add yourself from GitHub: `LeSplooch/tbaguette-skills`,
  from **Customize → Plugins** in the Desktop app or on claude.ai. A plugin
  added there also shows up in Claude Code at its next session.
- The Atelier is being prepared for Anthropic's plugin directory. Once the
  listing is live you will find it under **Customize → Plugins → Discover**,
  and the directory will keep it updated.
- In chat the skills load but plugin hooks do not, so the check for a
  relevant skill before every reply happens in Cowork and Claude Code only.
  In chat a skill fires when your request matches its description.

## 2026-09-29 — Three new skills for software that talks to a model, and what this year's agent incidents changed

- `secrets-hygiene` has an exception to "revoke first". If a credential was
  stolen by something still running on a machine that held it, such as a
  poisoned package or a worm, revoking it can set that thing off. Malware
  that wipes the home directory the moment its stolen token stops working has
  shipped more than once. Take the machine off the network first, so it
  cannot see the token start failing, then revoke from a different machine,
  then clean or reimage the infected one before it reconnects. For any other
  leak, revoke first as before.
- `designing-ci-pipelines`: a CI job that runs an agent over an issue, a
  comment, or a pull request someone else wrote now gets the same rules as a
  pull request from a fork. That means no secrets, no write token, no cache a
  publishing job reuses, and no public output while it holds anything worth
  stealing. Agent and editor settings inside a checkout count as pipeline
  code that the pull request wrote.
- `deciding-reversibility` and `bounding-autonomous-work`: before a write,
  read the target out of the credential or connection string it will
  actually use, not out of the task's description of the environment. A
  credential the run found in some file, rather than one it was given, is a
  stop. So is a credential mismatch, which is a refusal and not a puzzle to
  solve with a different token.
- New skill: `building-llm-features`, for code that calls a language model
  while it runs. It covers pinning the exact model and noting when it will be
  retired, and keeping prompts in the repository as reviewed code. It treats
  a refusal or a cut-off answer as its own outcome, not an answer. The
  model's output is handled as untrusted input wherever it goes, with limits
  on tokens, time and spend, and a fallback model has to pass the same checks
  as the main one.
- New skill: `evaluating-llm-output`, for measuring how often that output is
  right. Read real outputs before choosing what to measure, then write one
  pass/fail check per failure you actually saw. A model used as a judge has to
  be checked against a person's verdicts before anyone trusts it, and every
  pass rate is reported with how uncertain it is.
- New skill: `writing-agent-instructions`, for AGENTS.md, CLAUDE.md, rules
  files and skills. A line earns its place only if the agent could not find it
  in the repository and nothing already enforces it. A rule that must always
  hold goes in a linter, a hook or a test. A generated file is a draft to
  delete from, and the file is checked by running tasks with it and without
  it.
- `confirming-before-claiming-done` and `reviewing-code-deeply`: before
  calling a suite green, look at what happened to the tests as well as the
  code. Watch for deleted or skipped cases, loosened assertions, expected
  values edited to match, and branches keyed on a test's own input. A test
  that looks wrong is reported, not quietly rewritten.
- `auditing-dependencies`: a signed build attestation says where a package
  was built, not that it is safe. Poisoned releases in 2026 carried valid
  ones. A skill or rules file that tells the agent to install something must
  name a package that exists and is the one intended, because attackers
  register the names models make up.
- `designing-apis` has a new section for tools, commands and test runners
  that an agent calls. It covers keeping each response bounded, putting the
  summary first, and error messages that say what to do next.
- `handing-off-for-review`: before sending a change to a project you don't
  work on, read its contributing guide and any policy on AI-assisted
  contributions. Many now require disclosure or a named human who stands
  behind the change.
- Smaller changes: `fanning-out-independent-work` sizes a batch of parallel
  agents to what someone can actually review. `testing-the-untestable` says a
  hosted model cannot be made repeatable with a seed. `upgrading-dependencies`
  treats a model identifier as a dependency with an expiry date.
  `handling-untrusted-input` lists model output among the untrusted sources.
  `recovering-agent-context` no longer tells you to copy recovered history
  into the instruction file.

## 2026-09-29 — Locked doors: a switch nobody can reach, a port anybody can, an update nobody hears

- `configuration-management`: a feature that stays off until someone opts in
  can pass every test and still be off for everyone. Tests turn the switch on
  directly, so they prove the gate opens, not that a user can open it. If the
  only way to set it is an environment variable or a debug menu, no user can,
  and the logs count each of them as having said no. The skill now says to
  find the setting a user would actually use in the build you ship, and its
  `description:` sends that case to it.
- `threat-modeling`: a service that only listens on `localhost` is off the
  network, but not out of reach. Any web page open in your browser can send it
  requests or open a WebSocket to it, and every other account on the machine
  can connect as well. The skill now treats a local control port as two
  boundaries and says what closes each: a secret only your account can read,
  plus `Origin` and `Host` checks, or a socket the browser cannot address at
  all.
- `schema-evolution`: adding a login or token to a connection that
  already-running copies of your app use breaks them like any new required
  field. It is worse when that connection is also how an old copy hears it
  needs to update, because then it never hears it. The skill now says to keep a
  small, read-only answer for callers with no token (enough to see they are out
  of date), to refuse a wrong token outright so the caller re-reads it, and to
  have new clients check the version before logging in.

## 2026-09-29 — Checking where a result was pointed before trusting it

- `diagnosing-before-fixing`: a value you read off a dashboard, badge or
  status line has already been through the code that decided what to print
  when the data was missing or failed to load. That code often shows `0` for
  "not loaded", "failed" and "really zero" alike. Read it before chasing the
  backend, and check any second record of the same number first.
- `red-teaming-your-own-work`: a check that runs cleanly can still have been
  pointed at the wrong thing, like the wrong commit, the wrong spelling or the
  file next door, and its answer looks just as sound. Before a surprising
  result sends someone to look for a problem, restate exactly what it checked.
- `reproducing-bugs`: when you search a built file for a string your change
  added, the known-good string you test the search with has to survive the
  build the same way. Comments don't survive at all, and short strings can be
  compiled into forms a text search can't find. Until you've ruled that out,
  "not found" doesn't tell you the change is missing.
- `finishing-what-you-started`: when a later, legitimate change breaks the
  command a checklist line used as its check, write down the old result, why
  it stopped measuring, and the replacement. Don't rewrite the line, and don't
  report it as failing.

## 2026-09-28 — The plugin is now called tbaguette-atelier

- Every skill has a new prefix: `tbaguette-atelier:formidable` instead of
  `TBaguette:formidable`, and `/tbaguette-atelier:naming-things` where your
  agent calls skills as slash commands. Anything you saved that spells the old
  one, such as a prompt, a note, or `claude plugin disable TBaguette@skills-dir`,
  needs the new one. The skills themselves read exactly as they did.
- If you enabled the plugin for the Copilot coding agent, change the key in that
  repository's `.github/copilot/settings.json` to
  `"tbaguette-atelier@tbaguette-dev": true`. The Copilot CLI install is now
  `copilot plugin install tbaguette-atelier@tbaguette-dev`.
- A Claude Code install stays where it is, in `~/.claude/skills/TBaguette`, and
  switches to the new prefix on its next update. After that it shows up as
  `tbaguette-atelier@skills-dir`.
- The name is lowercase because claude.ai takes nothing else, and that is what
  it takes to list the Atelier in Anthropic's plugin directory, where it can
  reach the Claude apps and Cowork as well as Claude Code. Listings show it as
  TBaguette's Atelier.

## 2026-09-28 — A synthetic click that lands on nothing

- `grounding-test-doubles` now covers automation acting for real, not only
  in tests. A script or agent driving a real interface can have its input
  ignored without any error. An app that keeps its own cursor, like a game
  or a page that has locked the pointer, ignores a pointer warp, and the
  tool still reports the input delivered. The skill now says to read the
  target's own acknowledgement (hover, focus, where its cursor is) before any
  action that cannot be undone. If that never comes, switch to the input form
  the target consumes. The `description:` routes that symptom to the skill.

## 2026-09-26 — Hidden characters a scrubber misses, guards that trust the wrong signal, and the decision a failing fix ignores

- `handling-untrusted-input` now lists the invisible characters a scrubber usually
  misses when it strips only zero-width and bidi marks: the Unicode tag block, the
  invisible operators, the soft hyphen and the Mongolian vowel separator. Each one
  reaches a model intact and shows nothing to the person approving the text. It
  recommends dropping whole character categories over growing a denylist, and warns
  that variation selectors count as marks, so a category rule keeps them.
- `modeling-errors` covers the guard that checks *which producer ran* when it
  should check *whether a value was observed*. The two disagree in exactly the case
  the guard is for, so a placeholder gets through as a real reading. The fix is to
  let the value say it is missing, not to write a better guard.
- `diagnosing-before-fixing`: after three failed fixes, read the design decision the
  code says it implements before trying a fourth. Code that has drifted from that
  decision puts every patch inside the drift.
- `orchestrating-work-end-to-end` now routes to `keeping-copies-in-sync`: when a
  design writes a fact down in a second place, when a change edits one copy of it,
  and when a review has to notice the copies a diff left alone. Its phase index
  promises to cover every skill, and had missed this one since it shipped.

## 2026-09-25 — Plans that say what can run in parallel, a GitHub Copilot map for fanning out (contributed), and fixes for Windows and other harnesses

- `structuring-an-implementation-plan` now gives each task a `Depends on:` and a
  `Parallel-safe:` line and, when a plan may run in parallel, a phase list up front, so a
  controller fanning tasks out, or Copilot's `/fleet` partitioning the plan itself, knows
  which tasks can run together instead of guessing. Its self-review gains a parallel-safety
  check: no two tasks in a phase share a file.
- On GitHub Copilot, `using-tbaguette`'s Copilot mapping now says how dispatching really
  works there: several `task` calls in one response run in parallel, a subagent starts
  without TBaguette's context and shares your checkout, and the split is best decided before
  you have read every unit, because by then doing it all inline nearly always looks cheaper.
- On GitHub Copilot CLI, the plugin now ships three custom agents, `TBaguette:implementer`,
  `TBaguette:reviewer` and `TBaguette:investigator`; the first two carry their standing rules
  word for word from `delegating-tasks-with-review-gates`' own templates. The reviewer and investigator get no
  edit tool, and none pins a model: name one when you dispatch.
- The Copilot mapping now covers the GitHub Copilot app: a project session behaves like the
  CLI, the general chat offers no custom agents, and a lane that needs real isolation can run
  as a session of its own, in its own worktree.
- `keeping-tbaguette-current` now works wherever TBaguette is installed: it uses the install
  path the session-start check reports instead of assuming Claude Code's
  `~/.claude/skills/TBaguette`, so on another harness it no longer decides there is nothing
  to update because it looked in the wrong folder.
- On Windows, TBaguette's hook launcher no longer runs a `bash` it finds in the project you
  opened: it looks only in Git for Windows' install folders and on your `PATH`. The
  repository also checks every text file out with LF endings now, so a fresh Windows install
  made with Git's default settings keeps the hooks runnable. An older Windows install whose
  hooks never ran picks this up only when it is reinstalled.

## 2026-09-25 — Four checks that come back clean for the wrong reason

- `confirming-before-claiming-done`: a screenshot taken right after a resize, a navigation or a click
  can show the frame from before the change, sometimes only in one region. When the claim is about
  layout or state, measure that property directly, and when a picture and a same-second measurement
  disagree, suspect the picture.
- `red-teaming-your-own-work`: when a change is named after a class of defect (silent failures,
  unbounded waits, a false message), search that change for a new instance of the same class. When a
  fix replaces a message, check the new message against every path that reaches it.
- `calibrating-confidence`: finding a feature's classes, dependency or config key in a build does not
  show the feature is on, because frameworks ship that plumbing either way. Look for what registers or
  enables it before saying it ships.
- `performance-profiling`: a per-process or per-service rate counts everything that unit did. If the
  ratio moved and the change could not have moved the numerator, check what else was feeding the
  denominator, and compare absolute counts.

## 2026-09-24 — Reading the log is a diagnostic too (contributed), and nine smaller sharpenings

- From an outside contribution: `observing-production-safely` now says that rung 1 is only free when the data is
  read without costing the struggling host anything. Whether a whole-file read or
  text search over a large log does depends on the tool — a streaming matcher costs
  I/O, one that loads the file or its lines spends its size in memory on the box you
  are diagnosing. The log is largest exactly when the incident is worst, so the
  commonest ad-hoc diagnostic of all can be the thing that finishes the host off,
  while looking like the cheapest rung on the ladder because nothing was enabled
  and nothing was written.
- `deciding-reversibility` now says the act of *opening* something can be the write: opening a
  datastore can run its migrations, attaching can take a lease, so a read-only preview may have
  changed what it inspects — take the safety copy before anything opens it.
- `handling-untrusted-input` now covers forwarding someone else's text into an agent's *prompt*:
  the harness may parse it first (a leading slash runs a command, an at-sign pulls in a file), so
  use a literal prompt mode, or pass the text as an attachment instead.
- `portable-shell-scripting` now warns that under zsh a loop variable named `path` replaces `PATH`
  (as do `fpath`, `cdpath`, `manpath`), so every later command goes *not found*.
- `refactoring-safely` now says to check a scripted replace's search text is non-empty and matches
  the expected number of times before writing: an empty pattern matches everywhere and inserts the
  replacement between every character.
- `atomic-commits` no longer has you set your remaining changes aside to test each commit of a split
  when others commit from the same checkout: while the build runs, their next commit can sweep up
  the half-finished state. The working tree now stays at its final state; each commit is tested in
  a throwaway worktree first, then staged file by file straight into the index and committed once
  `git write-tree` shows the staged tree is the one that was tested.
- `diagnosing-before-fixing` now helps tell a real defect from a test-environment artifact when a
  check fails only where something never ran — a hidden browser pane that gets no animation frames
  or scroll events, a suspended app, a sleeping host. Comparing against the real path puts every
  such failure down to the environment; the question that sorts them is whether production can be
  in that state too, and for a view left in the background it can.
- `tending-tbaguette`'s approval gate (nothing is pushed, forked, or opened as a pull request
  without an explicit yes for that one contribution) now has a heading of its own. It used to sit
  under the heading about reopening settled questions, where anyone scanning the headings could
  miss it and a reorganization of that section could carry it off unnoticed. The paragraph on
  re-reading a staged pull request body or review comment just before sending it now lives in the
  contribution procedure, at the step where an unattended run stops to wait for the yes.
- `flaky-test-triage` now names a cause for a test that times out waiting for a message at a steady
  rate while its passing runs are fast: an earlier wait in the same test read that message off the
  socket or queue and threw it away as noise, which happens whenever two messages with no guaranteed
  order arrive the other way round. A longer timeout cannot bring the message back; the fix is one
  wait for the whole set, in any order.
- `flaky-test-triage` now says a clean run count after a race fix is evidence only if the race still
  happened. A fix can shift the timing so the bad interleaving stops occurring, and the count comes
  back clean for the wrong reason. Count the interleaving itself on the fixed build: the fix is
  verified when it still occurs at about the old failure rate and every run that saw it passed.

## 2026-09-23 — A contributor's site build aimed at the wrong address, and a retry that forgot its first failure

- `tending-tbaguette` now tells a contributor who can't run this repo's
  pre-commit hook exactly how to rebuild the site by hand. Skipping the one
  flag that rebuild needs makes every stylesheet, font, and script link point
  at the domain root instead of where the site actually lives — a mistake the
  test suite has no way to catch, since it checks the files are well-formed,
  not where they think they're served from.
- `modeling-errors` now says what a retry should report. A loop that keeps
  only its last attempt's error names the wrong failure whenever an attempt
  leaves something behind, such as a lock or a half-started process. The next
  attempt fails on that leftover, and every report after it describes the
  consequence while the real cause is recorded nowhere. The boundary that owns
  the retry should keep every distinct reason, the first one leading, and the
  skill's `description:` now routes that symptom to the skill.

## 2026-09-22 — One shout instead of two at session start

- The text TBaguette injects into every new session used to wrap itself in
  two nested "EXTREMELY_IMPORTANT" tags — one around the whole notice, a
  second one inside it around the actual rule. Only the outer one is gone
  now: doubled emphasis was reading as more suspicious to a fresh model than
  a single one does, without doing any extra work. Nothing about when or how
  often TBaguette checks its own skills changed — same trigger, same
  persistence across the whole conversation, same behavior across `/clear`
  and `/compact`.

## 2026-09-21 — A report describing itself, and a permission checked on the wrong machine

- `instrumenting-for-observability` now covers a generated report whose own
  description of itself — which data fed it, what setting was in effect, what
  window it covers — was written once and never recomputed, so the report goes
  on describing an earlier configuration long after the real one changed even
  while its actual numbers stay correct.
- `least-privilege-design` now covers a permission check validated against the
  wrong machine's environment: where the component that decides a permission
  and the component that enforces it are different machines, a check that
  passes cleanly against the first can mean nothing on the second.

## 2026-09-20 — Compound-command exemptions, cached trust, and a shell that goes silent

- `modeling-errors` now covers the case where an upstream API gives no stable
  code to distinguish "you've been throttled" from "the service is actually
  broken" — both render the same way, and folding them into one generic
  "degraded" verdict hides which one actually happened from anything reading
  that verdict downstream.
- `rate-limiting-and-backpressure` now covers a fixed shared capacity (a
  prompt budget, a config size limit) split across several unrelated content
  types with no reserved floor per type, where the least-disciplined type can
  grow enough to silently crowd every other type out entirely.
- `least-privilege-design` now covers three narrower ways a permission grant
  quietly reaches further than intended: an exemption or allowlist pattern
  matching just one part of a compound command (`a && b`) and thereby waving
  the whole line through; a cached trust or consent decision whose key never
  encoded a dimension that later changed, so a directory trusted while empty
  ends up pre-authorizing configuration added to it afterward; and a plugin or
  extension mutating the host process's own environment variables, which is
  reviewed as a `Fixed:` configuration tweak rather than as the capability
  grant it actually is.
- `portable-shell-scripting` now catches a process-kill pattern that matches
  its own invoking shell when typed as a single inline command rather than
  saved in a script — previously the skill's self-match guidance only reliably
  fired for scripted kills. The tell it now names: every subsequent command in
  that shell returns a nonstandard exit code with no output at all, because the
  interpreter died before it could write anything.

## 2026-09-19 — Rates, ranked matches, and parallel sameness

- `instrumenting-for-observability` now covers a computed rate (`X per second`)
  whose denominator spans a phase its numerator wasn't actually earned in — a
  transfer rate diluted by connection setup, a decode rate diluted by a
  one-time load — and covers a log tailer that must detect a shrunken file as
  a rotation rather than silently skipping to the end and losing everything
  written in between.
- `calibrating-confidence` now covers a ranked or scored match that returns
  several plausible candidates: picking the top-ranked one is a tie silently
  broken, not a resolution, and the ambiguity itself is worth reporting when
  nothing corroborates the pick.
- `fanning-out-independent-work` now names sameness — same model, tool set,
  working directory, and output shape — as what actually makes parallel work
  cheap; sibling agents that vary for no reason the task required multiply the
  run's real cost.

## 2026-09-18 — Closing a permission is now held to the same standard as opening one

- `least-privilege-design` now covers **revoking a capability that can be
  granted more than one way** — a cache, a second config source, or a second
  component that independently re-grants it — and asks for the same
  enumeration a grant already gets before a disable path is trusted.
- The same skill also covers **a narrow safety-bypass that silently doesn't
  work**: an untested escape hatch is a claim, not a control, and its real
  failure mode is teaching whoever hits it to reach for the big switch instead.
- `bounding-autonomous-work` now names a capability-expanding request —
  installing a plugin, tool, or server — as its own no-go for an unattended
  run, including one routed through a delegated subagent that surfaces the
  approval prompt to a human who can't see where it came from.
- `tending-tbaguette` — the contribution-pipeline skill — now says a declined
  push or pull-request step is a reading to re-try once rather than a fixed
  verdict, and that a clean `--dry-run` never confirms the real write is
  cleared.
- New skill: `keeping-copies-in-sync`, for the moment a version number, a
  constant, a policy document, or any other fact ends up recorded in more
  than one file — why memory ("we'll update both") is the mechanism that
  produces drift rather than a defense against it, and why generating one
  copy from the other beats hand-maintaining both, with a comparison that
  fails loud as the fallback where a second copy has to genuinely exist.
- `diagnosing-before-fixing` now names a trap specific to short-lived
  credentials — a pairing code, a token, a presigned URL — fetched once and
  spent several steps later in an automated flow: the credential can expire
  mid-flow, and the failure surfaces as a transport error (a closed
  connection, a reset socket) rather than a rejected value, which sends
  debugging into the wrong layer first. The fix matches the diagnosis:
  collapse fetch-and-consume into one step with no round trips in between,
  or re-check the remaining lifetime before the step that spends it.
- `formidable`'s color reference now flags a specific way theming a
  hardcoded color goes wrong: fading it toward transparent to fit a runtime
  theme is the idiomatic move and it quietly breaks any contrast ratio that
  was measured and documented against a fixed background, because the
  rendered result now depends on whatever paints behind it. Blend toward the
  theme's own background color instead, and treat a comment explaining why a
  color was hardcoded as the spec for the conversion, not something to
  clear away.

## 2026-09-17 — Clairvoyance now looks at the whole, not only the request

- `clairvoyance` — the skill that questions the request instead of serving
  it — now sweeps anything that arrives with a frame: a plan, a decision, a
  metric, a document, a codebase, the project as a whole, and the run you are
  in, not only the message that opened the session. A table says, for each
  of those, where its frame came from, when a look is cheap, which direction
  to look first, and where a finding goes.
- An eighth direction, `around`: the field of people and forces the work sits
  in — who has a stake, what incentives and deadlines act on it, and what in
  the surrounding world sent it now. The seven existing directions were all
  about the problem; none was about the people, which is most of what a view
  from above means.
- Two new mechanical moves: change altitude (say the problem one level up and
  one level down, and solve it at the cheapest level) and deviation
  guidewords (apply no, more, less, reverse, early, late, other-than to each
  parameter of the thing — the most reliable idea generator in the file).
- Four things to invoke by hand. `orbit` — the whole project from above, in
  one bounded page: altitudes, purpose as revealed by behaviour, the empty
  chairs, the flows across its boundary, what changes at what pace, and the
  negative space; at most three findings, run at named moments, never a mode.
  `imagine` — up to five ideas produced as observations, each carrying the
  check that would tell whether it is worth anything. `lens <name>` — one
  instrument from a catalogue of about thirty borrowed from other disciplines
  (`reference/lenses.md`: ideal final result, pre-mortem, pace layers, key
  assumptions check, POSIWID, Goodhart, Wardley evolution, and the rest),
  each with its tell, its failure mode, and the skill that owns it where one
  does. `seat <chair>` — one look from a named seat: the operator at 3 a.m.,
  the auditor, the integrator, the customer's customer.
- What has not changed: sight is free and action is not. Every finding — an
  idea included — routes to a gate, the ledger, the record, or a discard with
  a reason before anything acts on it; a sweep is never a reason something
  did not ship; and a run with nobody present may notice a reframe and may
  not approve one. The null is still the normal result, for an orbit too.
- One-line handoffs now point back at it from `revalidating-decisions`,
  `threat-modeling`, `orienting-in-unfamiliar-code`,
  `recovering-agent-context`, `bounding-autonomous-work`, and `writing-adrs`,
  and `orchestrating-work-end-to-end`'s phase routing seats an orbit after
  orientation on inherited work and an imagine pass at the design gate.
- The change was swept with the enlarged method before shipping, and the
  record of what that found — including the two readings of "God's view" it
  had to choose between — sits in the skill beside the original creation
  record.
- Measured the same day, blind, against the version it replaced: the enlarged
  skill won six of eight paired runs with one tie, and every point of the
  margin was discipline — findings marked as inference and carrying their
  check, bounded, ranked, delivered as one decision — while both versions
  scored full marks on insight itself. The record says so, so nobody
  overclaims it. One defect the test caught is fixed: `orbit`'s bound read
  "one page" and measured runs came back at three; it is now about five
  hundred words, counted.

## 2026-09-17 — A build's own output folder can hand you the wrong binary

- `confirming-before-claiming-done` now covers finding a test binary by name
  pattern in a build tool's shared output directory: the same directory often
  holds the shipped product under a similarly-shaped name, so a lookup that
  has always landed on the test binary before can silently launch the real
  application instead — and a "hung test" is the tell, not a slow one.

## 2026-09-17 — Contributing to TBaguette is safer when two people do it at once

- `tending-tbaguette` (the skill that turns a lesson into a pull request) now
  flags two ways contributing at the same time as someone else can go wrong:
  picking the same queued idea without knowing someone already has, and a
  shared local checkout redirecting one contributor's commit onto another's
  branch. Both now have a concrete fix in the skill.

## 2026-09-16 — Read a setting before you write it

- **`deciding-reversibility` adds read-before-write to its list of ways to
  turn a one-way door into a two-way one.** A configuration write is only
  irreversible because the value it replaces is gone, so the skill now asks
  for the prior value to be read and kept in the same command as the write —
  and says plainly that believing the new value is safer is no reason to skip
  it, since that belief is exactly what let a "safe" value silently downgrade
  an environment that was already more permissive. The description now
  triggers on overwriting a setting or a configuration file.

## 2026-09-16 — A spec you remember is not a spec you have read

- **`checkpointing-long-runs` names the document the work is measured
  against as a source to re-open, not recall.** Its after-a-boundary rule
  already puts the repository and the record ahead of recollection; it now says that a
  specification, design, contract or reference rendering is the artefact a
  compaction most readily replaces with a summary of itself — and a summary
  cannot be diffed against an implementation. Re-open the source end to end
  before measuring anything against it, and ask whether the run before the
  boundary had ever read it at all: a document that was only grepped for
  identifiers was never read. The description gains the trigger, and
  `reading-specifications` points here for that case.

## 2026-09-16 — A clean zero from a host that never ran is not a measurement

- **`finishing-what-you-started` names a vacuous check that wears a real
  number.** Its watch-it-fail rule already lists the typo'd path and the grep
  against a file that does not exist yet; it now adds the instrument that only
  runs while its host is awake — a layout-shift score from a tab that never
  painted, a queue depth from a scheduler that never ticked — and returns a
  clean zero while the host sleeps. The remedy is stated: arm the measurement
  inside the host on its own wake signal and read the result afterwards, rather
  than polling from outside. The skill's description now triggers on a zero
  that came from an instrument whose host never ran.

## 2026-09-16 — A connected tool provider can wait out a reconnect-only check

- **`auditing-dependencies` now covers a connected server that changes its
  instructions mid-session rather than on reconnect.** The existing advice —
  record what a provider's instructions said and diff them on reconnect — has
  a blind spot for a connection that never drops: a payload that stays clean
  for the first several calls and only then rewrites itself slips past both a
  one-time human read and an early smoke test, because nothing about that
  session ever reconnects for the diff to run against. The skill now asks for
  a periodic re-diff within a long-lived session, not only at reconnect.

## 2026-09-16 — Two gaps in `scoping-before-building`'s design-approval step

- **A written design now has to state its own status** — which sections were
  approved in conversation, whether the document as a whole has been
  reviewed, and what gate still stands before an implementation plan exists.
  A spec that only lists what already happened reads, to a later reader with
  nobody to ask, as fully cleared — "approved section by section in
  conversation" describes the sections, not the document. `bounding-
  autonomous-work` now names the same hazard from the unattended reader's
  side: treat a handoff doc silent on its own status as unreviewed, never as
  approved.
- **A design that relies on independent corroboration — quorum, multi-sig,
  k-of-n distinct reporters, "review by two people" — now has to name the
  population it draws from, not just the threshold.** "Most secure" and
  "does anything at all" trade off through that number, and a trust circle
  described as "the general public" can, today, have zero or one actual
  members — which makes the mechanism inert by construction until the
  population catches up, silently, with no error anywhere to notice it.

## 2026-09-15 — Slow startup now routes to the concurrency-model skill

- **`choosing-concurrency-model` now covers slow startup with several
  independent dependencies** — config, cache, connection pools, plugins. Bring
  them up one after another and the total wait is their sum, and one that
  hangs blocks everything queued behind it, which reads as the whole app being
  slow rather than as one stuck dependency. The skill now names this pattern
  and its fix: start independent dependencies concurrently, each with its own
  timeout.

## 2026-09-14 — A dropped clause, a liveness check that lies, and a screenshot of the wrong app

- **`finishing-what-you-started` now covers the recurring run.** Its surrender
  rule was written for one run: mark a criterion surrendered, never delete it.
  A job that fires on a schedule against a fixed request obeys that every time
  and still loses a clause, because each run derives its ledger fresh,
  surrenders — or never writes — the same line, and nothing compares runs. The
  skill now says to write one ledger line per clause of the standing request
  even for a line about to be surrendered, to read the previous run's
  surrendered lines first, and to settle rather than re-surrender a line that
  would be surrendered twice on unchanged grounds — addressed, or declined with
  a reason and a reopen condition in a record that outlives the run. New
  trigger in the `description:` for a recurring run dropping the same clause
  each pass.
- **`portable-shell-scripting` now covers the one-shot liveness check.** Its
  kill-and-wait section already said that `pkill -f` can kill the calling
  shell and that a `pgrep -f` wait loop can wait for itself. The quietest
  form was missing: a single `if pgrep -f name` asked before a restart, which
  matches the asking shell's own command line, answers yes, skips the
  restart, and leaves every later step running against a process that is not
  there — with no symptom at all. Ask liveness by handle (`kill -0` on the
  saved PID, or `pgrep -x` on the short name), and treat an inline one-liner
  in a tool call as the same shell. New trigger for a script that asks once
  whether a process is still running.
- **`formidable`'s audit reference and `confirming-before-claiming-done` now
  treat a capture as a measurement of whatever owned the surface.** Two
  unrelated projects hit it the same week: a desktop screenshot of "the
  active window" caught the wrong one, and a phone screenshot came back
  showing a different app because another session had deployed to the
  shared device between two steps. Address every capture to the thing under
  test (window id or class, package, tab), read and record the foreground
  owner at capture time, abort the step when it is not the subject, and
  make every state reachable from a script so all of them land in one
  contact sheet. `confirming-before-claiming-done`'s artifact section gains
  the same case as a positive-probe rule.
- **`tending-tbaguette` asks a contributor to state the mechanism as its own
  claim.** A pull request's "X rejects Y because of Z" is two claims — the fix
  worked, and Z is why — and the second gets checked against the source of
  whatever did the refusing, because a fix that works by a fallback the
  diagnosis never named lands a side effect after the merge. A deleted test is
  quoted, with why it is wrong now. Also a one-line pointer from the
  re-reading-a-candidate rule to its general form in
  `finishing-what-you-started`.
- **`formidable`'s craft floor now says how to verify a state that lives
  between two events.** A pressed or active state is set by the down event
  and cleared by the up event, and an automation tool's default tap or click
  delivers both inside one frame — the flag is set and cleared before any
  render sees it, so a capture shows the effect of the press and none of its
  feedback, and the feedback reads as missing when it is not. Hold the input
  for the length of a real press (roughly 100 ms), and treat an instantaneous
  synthetic event as unable to observe a transient state rather than as
  evidence it is absent. The audit reference's "drive it with synthetic
  input" step now points at the rule in `craft-floor.md`. New trigger for a
  state the code implements that does not show in a capture.
- **`formidable`'s craft floor now verifies motion from frames, not from a
  still.** A screenshot has a capture latency of its own, and any change that
  completes inside that window is invisible to it by construction — a
  quarter-second capture path returns the resting state of a quarter-second
  animation every time, and every micro state on `motion.md`'s duration table
  is shorter than that, so the motion reads as absent when it is not. Record
  the moment instead, split the recording into frames at a rate that puts
  several inside the shortest animation under test, and take one number per
  frame from the region that should change; the result is a curve (starts
  within a frame, holds as long as its cause lasts, resolves over the stated
  duration) that can be checked against the spec. Write the capture latency
  beside the animation's duration before choosing the instrument. The
  `description:` trigger added above now reads "a state or animation the
  code defines".
- **`orchestrating-work-end-to-end`'s Crew dial now covers a shared instrument,
  not only a shared tree.** A test device, a browser profile, a serial port, a
  staging box, a terminal session — anything with one foreground — carries the
  state of whoever is using it now, and "connected and unlocked" says the run
  is permitted, not that the instrument is free. One read-only look at what is
  in front and a second to see whether it moved without you settles it: a
  foreground you did not start is another run's state, and launching on top
  breaks that run and captures a surface nobody was testing. Do not take it;
  use the next-best evidence, say so on the deliverable, and ask for the
  instrument when you actually need it. New row in the envelope-change table
  for the moment it happens mid-run. `formidable`'s rung rule in `elevate.md`
  gains the case where the top rung is available and occupied at once, and its
  audit reference's "abort the step" now says abort rather than relaunch on
  top.

## 2026-09-13 — A default that looks correct alone can still be impossible in combination

- **`configuration-management` now covers a default configuration that is
  individually well-formed but jointly impossible for the system it actually
  ships on.** Field-level validation confirms each setting is the right type
  and in range; it says nothing about whether the *combination* of shipped
  defaults can ever be satisfied by a real input once that combination meets
  the one platform, network, or tier the running instance is fixed to. That
  failure doesn't look like a misconfiguration — nothing crashes, every
  request is answered, and the system just quietly rejects everything, which
  reads as calm rather than as a defect. The skill now says to test the
  shipped defaults directly for that case, rather than trusting field
  validation to catch it by accident.

## 2026-09-13 — A candidate waiting on time isn't the same as one waiting on evidence

- **`tending-tbaguette` now tells apart a candidate that's out of room from one
  that's out of evidence.** Its guidance for re-reading a queued contribution
  used to treat every re-visit the same way — as a `deferral`, settled after two
  unchanged readings. That was wrong for a candidate that simply arrived after
  a pass had already used up its attention for the sitting: it isn't waiting on
  a second sighting or a fuller argument, only on the next pass having room for
  it. The skill now marks that case `earmarked` rather than `deferred`, so it
  gets picked back up on the next pass with capacity instead of being held for
  a second reading it was never actually waiting on.

## 2026-09-12 — The rule at the bottom of the file, the prompt nobody answered, and who actually said it

- **`writing-durable-docs` now treats where a rule sits in a document as a
  reliability property, not a style one.** A document a harness carries across a
  compaction or a handoff is shortened on the way, and every mechanism you did
  not choose shortens it from the end — a compaction keeps the opening of each
  file it re-attaches, a summarised handoff keeps the opening and paraphrases the
  rest, a capped listing keeps whatever came first. So a hard constraint written
  after the explanation that motivates it is in force for the first hour of a
  long run and silently absent after the first compaction, while the overview
  survives. The new section says to put first what must hold last — constraints,
  stop conditions, the things a later reader may not decide for itself — and to
  put the reasoning after the rule rather than before it, since the reasoning is
  what a person needs and the rule is what a machine needs, and only one of the
  two has to survive the cut. `checkpointing-long-runs` now points there from its
  constraint paragraph: a bound that has to live in prose lives at the top.

- **The same skill now names the negative claim that closes a door.** Its
  section on claims no build can check covered promises about your own software
  — *sends nothing, stores nothing* — which go false when someone adds the thing.
  A new passage covers the other kind: *the platform has no such field, the API
  cannot page*, written to explain a design and read by every later reader as a
  reason not to try. No inverted test can catch it, because the absence is in a
  system you do not own, and it is often wrong on the day it is written. The rule:
  a negative claim about a dependency carries the spec section, the version, or
  the command that was run — or says `not checked`, which invites the next reader
  to check where *not possible* invites nobody. `revalidating-decisions` still owns
  what a reader does with such a claim once it exists.

- **`red-teaming-your-own-work` now audits the artifact you ship, not just the
  one you built.** A build is written for the audience it has in development — the
  author, a few testers, machines the author controls — and plenty of behaviour
  that is right for that audience is a defect for the next one: a trial timer that
  bricks the binary a stranger paid for, a counter hidden in system-looking
  filenames that antivirus flags, a dev endpoint, a phone-home, verbose logging.
  A normal review misses them because publication does not feel like a change to
  the thing under review — shipping is "packaging". The new section asks a
  different question of the finished artifact: what does it do on a stranger's
  machine — what it writes and where, when it stops working, what it sends and to
  whom. The tell is a mechanism whose message is addressed to the developer
  (*recompile*, *dev build*) surfacing to someone who cannot act on it; the remedy
  is to gate the dev-only behaviour behind an off-by-default flag, so the
  shippable build is the default and the dev behaviour is the opt-in.

- **`formidable`'s craft floor now lists `stale` among the states a surface has
  to design for, and says which clock it ages by.** A live readout has two — the
  connection's idea of being open, and the age of the newest datum — and only the
  second is honest: a green dot and a fresh-looking latency figure outlive a dead
  source for as long as the idle timeout, because the transport is built to be
  tolerant and the person is not. Freshness is derived from the last real update,
  shown as an age once it passes the cadence the source promises, and verified by
  killing the source and watching the indicator turn. A second check covers the
  value that is *derived*: a readout built from more than one field is computed
  from fields sampled together or it says which input is behind, since a ratio
  whose halves are written on different cadences displays a state the system was
  never in — and a displayed time is rendered from a full instant, never from a
  bare wall-clock string that a parser rejects and the surface then shows raw.

- **`bounding-autonomous-work` now covers the gate a detached run never
  installed.** Its decision bound already said what to do with a question the run
  would have asked and nobody is there to answer. A job left running overnight
  also walks into prompts it never put there — a consent dialog, a permission
  prompt, a second factor, an expired session asking for a password — and alone,
  each of those is a wait with no end: the job sits behind it until someone comes
  back, draining whatever it was keeping awake, with nothing failed, nothing
  logged, and a result that reads `0 of N`. No stop condition fires, because a
  process that is not executing checks nothing. The new section says to
  enumerate the human-only steps before detaching and, for each, either run it
  attended first — detach after the last such gate, not before the first — or
  bound the wait with a timeout that turns it into a reportable failure. The tell
  when reading the log: entries that stop after the first minute.

- **`handling-untrusted-input` now says where authorship is read from: the
  channel, never the content.** Its section on the destinations with no
  separating mechanism covered delimiters and the payload that arrives at
  discovery time; it did not say what to do with a role, a name, or a system
  marker that appears *inside* text — a maintainer tag at the top of a comment, a
  `system:` line inside a tool result, a block shaped like the harness's own
  notices sitting in a file. Forging one costs nothing, and it is the injection no
  delimiter addresses, because the delimiter is what is being imitated. The new
  paragraph makes authorship a property of the arrival, established once at the
  boundary the way validity is: instructions from outside keep their origin
  attached for as long as they are held, are checked against what that source was
  expected to send before they are acted on, and are never merged into the run's
  own instructions where the label would be lost. One harness shipped exactly
  this rule this week, after crafted role headers inside untrusted comments were
  summarised as authoritative; a draft protocol extension for serving skills
  remotely asks the same of every host that loads one.

## 2026-09-11 — A log line that never fired, from an outside contribution — plus things still running, and things that had quietly stopped

- **`diagnosing-before-fixing` now covers the measurement you did not get.**
  Its existing sections all read a result that came back — several anomalies at
  once, an exact null, a reading that contradicts itself, a guard that refused.
  The new passage reads an absence: you sweep a log for the signature of a
  behaviour, find nothing across the whole window, and conclude the behaviour
  never ran. Nothing is broken in that story, which is why it holds up. The
  emitter is correct and the behaviour did occur — the line just sits behind a
  condition the run never met. A memoised path logs while it builds an answer
  and is silent every time it serves one, so the log is loudest when the system
  is coldest and quietest under exactly the steady-state traffic you are usually
  investigating; a level-gated line needs a verbosity nobody turned on, a
  sampled one fires on a hundredth of the traffic, a once-per-process one fired
  before you started reading. The confidence is proportional to how carefully
  you searched, and a thorough sweep of a window the emitter was never armed in
  is worth nothing. The tell is the un-varied factor's, one section up: name the
  run in which that line would have been written. Then read the emitter's
  guard before reading its silence, and arm it — clear the cache, use a cold
  key, raise the level — so you have watched the line appear at least once.
- **A pipe can truncate the command, not just its output.** `portable-shell-scripting`
  now covers what happens when the reader on the right of a pipe exits first: the writer
  on the left is killed. For a command that only prints, that is the point of it. For one
  that changes things — a package operation, an extract, an in-place rewrite, a
  version-control step that stages many paths and then writes one reference — it stops the
  work wherever it had got to. How loudly depends on settings made elsewhere in the
  script, which is the part worth knowing: a bare pipeline reports only its last stage, so
  the status is the reader's zero and nothing says the writer died, while `pipefail`
  reports 141 and a per-stage status array names which stage it was. So it is detectable
  exactly where the file's own `set -eu` advice has been followed, and silent in a one-off
  command, inside a substitution, or in any `sh` script without it. Worse, a tool killed
  before its own bookkeeping does not recognise its own wreckage, so the recovery command
  reports nothing to recover.
- **A delegate that reports success is the wrong thing to make more rigorous.**
  `delegating-tasks-with-review-gates` already covered a run cut off mid-sentence. It now
  explains the one that finished, checked itself, and reported done about work it did not
  do — and why the two-stage review it prescribes is written as a fresh reviewer against
  the diff rather than as a stronger instruction to self-review. What moves the false-done
  rate is whether anything other than the delegate can observe the end state, by much more
  than the gap between a careful delegate and a sloppy one. So when the gate is
  inconvenient, the substitution to refuse is "have the implementer check more carefully",
  and the one to reach for is any other observer of the same result. It also adds a
  forensic tell for the review itself: judging a claim by how it is written is close to
  guessing, because confidence is a property of prose — a false completion is
  characteristically a record of looking, with nothing in it that could have caused the
  effect being claimed.
- **Telling the author what to fix is not the same act as deciding the change is
  right.** `reviewing-code-deeply` gains the failure that no measure of review
  participation can see. When the author will revise on request — and overwhelmingly when
  the author is something that rewrites on command — the cheap move is to say what the next
  version should contain, and it crowds out the expensive one of forming a verdict on this
  one. Comments keep arriving at a normal rate from an engaged reviewer, and the change is
  never actually judged. A steering comment is an instruction about the next revision; a
  review comment is a verdict on this one, and only the second can end in no.
- **Sandboxing a thing bounds what it can run, not what it can write.**
  `threat-modeling` now asks the boundary question twice. The usual audit enumerates what a
  confined component can invoke and confirms that attempts to step outside fail — and it
  comes back clean on a system with an exit in it, because the exit is not an invocation.
  Anything the confined thing may write that something outside will later read, run, or
  treat as configuration is a way out at the outer thing's privilege, deferred until that
  outer thing next reads. Nothing is happening at the moment of the write, and the
  escalation is eventually performed by a component behaving exactly as designed.
- **Only the premises that hurt ever get a review date.** `revalidating-decisions` gains
  the half of its own subject that goes unrecorded. A constraint that blocks you is written
  down when it bites, often with the one check that would overturn it. A premise that
  *permits* you costs nothing while it holds, so it is written down nowhere, acquires no
  date, and is inherited by every later reader as a fact about the world — then found to
  have lapsed at the moment it is finally needed, which for anything at the end of a
  sequence is after everything before it has been spent. The skill also separates two
  things that look identical afterwards: a result you can observe is not a capability you
  have, because the step may have been refused and something else may have produced the
  outcome. Credit a capability only if you watched it work.
- **A rule you are obeying can have stopped doing anything.** Same skill, and it is a
  premise decaying like the others, except that this decay makes the rule look better.
  Guidance telling an actor to do what it would have done anyway is obeyed perfectly and
  changes nothing, and from outside, full compliance and complete irrelevance are the same
  observation. The drift has a direction — the world moves toward the rule — so rules
  convert from load-bearing to decorative one at a time while the overall compliance figure
  rises. `choosing-test-scope` already owned this for a check running against real traffic,
  with the instrument: sample the distribution of its verdicts. The new section carries the
  two things that do not fit there — the direction of the drift, and what to do when the
  thing in force is a sentence with no verdicts to sample, which is to withhold it and see
  whether the outcome changes.
- **Where reuse is decided by a prefix, a volatile element costs everything behind it.**
  `caching-strategy` treated a cached value as atomic — it matches or it does not. It now
  covers the family where the match is a leading run rather than a whole key, and the
  arrangement of material inside one entry becomes a cost decision nobody made. A timestamp
  or a request id at the front of a large stable body is charged not for its own few bytes
  but for everything stable behind it. The diagnostic is not how much changes between
  calls, which is usually almost nothing; it is how early the first difference occurs.
  Which is why extending retention does not help: it treats a placement problem as a
  lifetime problem.
- **"There is a manual override" is not an exit until you say who can reach it.**
  `modeling-state-machines` already required every non-terminal state to have an exit, and
  a manual override satisfied that line while being, for the person actually stuck, no exit
  at all — an operator with database access is not a route a user has. The skill now asks
  for the actor and the control by name, and it asks hardest about protective states, where
  entering is the designed behaviour and leaving is the afterthought: a breaker that trips
  with no reset control, a pairing nothing in the product can clear, a lockout whose only
  cure is a support ticket.
- **`tending-tbaguette` separates the step from the outcome.** A contributor's pipeline
  ends in a push and a pull request, so a permission that has lapsed since last time is met
  at the last step, after the reading, the drafting, the approval gate and the commit have
  all been spent — and one dry run at the start moves that discovery to the front. The
  trap specific to contributing is that the work can land without you: a maintainer applies
  the lesson after reading it in an issue, or another contributor sends the same
  observation, and afterwards the repository looks exactly as it would have if your own
  push had worked. Record what actually happened, because the repository's state never
  will.
- **Republishing an artifact under a version it already carries splits your users in two.**
  `reproducible-environments` covered pinning by digest as something you do because someone
  else's registry is mutable. It now also covers the day you are the one making it mutable:
  a toolchain patch lands, the source has not changed, the rebuilt bytes differ, and
  re-uploading them under the existing version looks like no change at all. Where an update
  channel compares versions rather than digests, everyone already holding the old bytes
  never fetches and everyone arriving later does, so two populations run different code
  under one name and nothing anywhere records it. With the corollary to publish a versioned
  name alongside a moving one like `latest`, which is what makes the population countable
  afterwards.
- **An installer that is safe to re-run can be the thing undoing your settings.**
  `designing-for-idempotency` has always said to prefer an absolute write over a relative
  one. That is right for retries seconds apart and wrong for a setup step that runs again on
  next month's upgrade, after somebody deliberately changed the value — which is how a
  setting comes to turn itself back on after every update while the code review keeps
  finding a correctly written idempotent operation. New guidance on telling a default from
  an assertion when both look identical at the call site, and on the one question that says
  which you are holding. Its description now routes install and provisioning work here.
- **`tending-tbaguette`'s cross-project sweep no longer trusts its own coverage count.**
  Counting the projects in scope and the projects actually read only catches a gap if the
  two numbers come from differently shaped queries; derive both from the same traversal and
  they agree by construction and certify the gap instead of finding it. Also: the projects a
  limited traversal silently drops are systematically the busiest ones, and the difference
  is worth reporting as a list of names rather than as a number.
- **`tending-tbaguette` tells contributors what happens to the date on their note.** Date
  the `UPDATES.md` entry the day you write it, and expect it to move to the day it lands —
  that is the day a reader's clock measures. A note left under an earlier date sits below
  the entry already at the top, which on the site means inside the archive fold, published
  and seen by nobody. The first bullet of this entry spent its first day exactly there.
- **`reproducible-environments` covers the script that was executable and still got lost.**
  Its rule that setup must be code rather than prose has a converse: a script that
  reproduces an expensive step is not source while it lives in a scratch or temp directory,
  because that directory is designed to be emptied and will be. The prose write-up survives
  the reboot; the recipe it describes does not, and the next rebuild fails on a file you
  wrote yourself. The test for where a script belongs is whether you would mind deriving it
  again, and a scratch script that has worked once moves into the repository on the spot.
- **`secrets-hygiene` names one more place a secret travels: the platform's own backup.** A
  key or identity a program generates on the device and never ships anywhere is still copied
  out by the platform's backup and sync, which defaults to the whole data directory and
  restores it onto a reinstall or a new device — so a signing key round-trips through
  someone else's servers and a fresh install comes up already paired to a peer that has
  forgotten it. Exclude by name anything that identifies *this install* rather than *this
  user*; the tell of a restore is state present before the code that writes it has run.
- **`tending-tbaguette`'s sweep now separates repositories that are not yours from ones that
  were quiet.** Searching deep enough to stop dropping nested projects also reaches vendored
  upstream trees and reference mirrors, which inflate every count and — the day one syncs —
  flood the cluster step with one project's subjects written by many hands. Classify by
  authorship rather than by location, matching on the email rather than the name.

## 2026-09-10 — The output you kept, and the file that came back larger

- **`confirming-before-claiming-done` now covers what a `tail` takes away.** It
  already warned that piping a check through a filter can swallow the verdict,
  since a pipeline reports the status of its last stage. The new passage is the
  other half of that loss: `head` and `tail` pick what to discard by position,
  and position is the one axis that has nothing to do with which lines a report
  will want. A success is argued from lines that print early — the gate that
  passed, the version the run resolved, the branch it took — so a `tail` keeps a
  long build's closing spam and drops every one of them, and the shortfall shows
  up only much later, when the claim is being written and there is nothing left
  to cite but the exit code. Capture with `> run.log 2>&1`, which puts none of the
  run in context, and filter the file by pattern instead.
- **`crouton` no longer recommends the move that causes it.** Its read rules
  still say to cap a command when you already know the shape of the answer, and
  now carry the exception — a run you will quote from goes to a file, which is
  the cheaper option as well as the safer one.

- **`automating-repetition` now carries the mirror of its matched-nothing rule.** It
  already said that a substitution whose pattern is absent exits zero and leaves the
  file byte-identical. The new section is the costlier direction — a pattern that
  matches the *wrong* place — and what reliably produces one is a document that
  describes its own format, markdown quoting one of its own headings inside a code span
  being the everyday case. A first-match search finds the mention, so anchor a
  structural delimiter to the structure (`^## Changelog$`) rather than to the bare
  string. The sharper half is what a wrong coordinate then feeds: `text[:start] + text[end:]`
  is a deletion while the bounds are ordered, and once they are reversed it emits the
  region between them twice without raising — in Python, JavaScript and Go alike, since
  each half is a legal slice on its own. So the failure ships a longer, well-formed,
  entirely plausible file with nothing removed at all. Assert the bounds are ordered
  before slicing, and check that the size moved in the direction the edit intended.

- **`secrets-hygiene` now covers the case where you are the one who needs the
  credential.** It already refused to let you carry somebody else's secret onward
  to a third party, on the grounds that anything you hold you also record. The new
  section is the inward mirror, and it arrives dressed as caution rather than as
  helpfulness: something you are about to run needs a request authenticated, and
  the options look like putting the credential where that thing can read it or
  refusing to run it at all. Refusing reads as the responsible choice, and it is
  the cheaper mistake rather than a different kind of one. A third arrangement
  hands the consumer a reference and exchanges it for the real value at the
  boundary, for destinations named in advance — the request authenticates, and
  nothing the consumer can record ever held the credential. Most of the new text
  is about telling that apart from its counterfeit, because the shape is easy to
  reproduce without the property: a resolver that hands the value back to its
  caller changes where the secret is stored and not whether the caller holds it,
  and the list of destinations is the entire restriction.

- **`confirming-before-claiming-done` now treats an unpredicted number as no
  check at all.** A gate that reports a figure — tests collected, files changed,
  rows migrated, warnings emitted — reads as a check and on its own is not one:
  whatever it printed you were going to accept, because no value had been
  committed to in advance as the one that would look wrong. Saying the number
  first costs one sentence and turns the reading into a comparison, which is what
  makes a mismatch visible at all — and what a mismatch usually means is that the
  inputs are not the ones you think you have, a concurrent session writing into
  the same checkout being the everyday case. None of that turns a run red. Two
  limits come with it. A match is worth what its arithmetic is worth, since two
  changes that cancel produce one just as readily; and a prediction taken from a
  record rather than a measurement is only as fresh as that record, so everything
  that legitimately landed since shows up as an unexplained delta and sends the
  pass investigating its own stale bookkeeping. Where a figure genuinely cannot
  be predicted, the rule inverts rather than softening — that number is an
  observation, not something a claim can stand on.

- **`least-privilege-design` now covers the rule that never loaded.** A policy
  has to be read by something, and what that reader does with a line it cannot
  parse is usually nobody's decision — the common behaviour is to skip it and
  carry on. The new section is why that skip is not neutral: its cost splits on
  the polarity of the rule dropped. A malformed allow that gets skipped breaks
  something, and somebody reports it within the hour. A malformed deny that gets
  skipped does nothing at all, which is exactly what a working deny rule looks
  like — so the restriction is gone, the system is more permissive than the file
  on disk describes, and the file goes on describing it. That turns out to be a
  fresh argument for default-deny, since a broad allow with denies carving
  exceptions out of it is precisely the shape where every carve-out can vanish
  unseen. A loader should refuse to start on a rule it cannot parse, and where it
  must tolerate unknown syntax that tolerance belongs on the permissive rules
  only. Which behaviour you have takes a minute to establish: feed a deliberately
  malformed rule to a copy of the policy and see whether the loader objects or
  comes up clean. An empty denial log cannot tell you, because a rule that never
  loaded and one that was never violated leave the same record.

## 2026-09-09 — What reaches the reader, and what quietly does not

- **What someone watches while the work runs is technical writing too.**
  `formidable` has always owned those surfaces as *interface* — which display a wait
  earns, what goes to stderr, how honest a progress bar has to be. Nothing owned what the
  line actually **says**. `explaining-technical-work` now does, because every test it
  already applies to a report applies to a status line, and three things change when the
  reader is watching rather than reading. What is nearest to hand at the moment of
  emission is the system's own vocabulary, and there is no later place for the
  translation to happen. The same true sentence is a different and worse message after
  the reader has committed than before it — a warning that fires on save, when it could
  have fired on the keystroke, is not late, it is misplaced. And going quiet asserts that
  nothing is happening, to a reader who has no way to check.
- **A rule you stated out loud is not stored anywhere.** `checkpointing-long-runs`
  ranks what a checkpoint should hold by what it costs to lose, and there is now a row
  above the old first one: the constraints somebody stated in conversation. "Don't push."
  "Ask me before deploying." Those are not kept as rules — whatever enforces them re-reads
  them out of the run's own context on every check, so a compaction, a summarised handoff,
  a fresh session resuming from a transcript, or a subagent that never received the message
  can delete the bound while everything goes on reporting normally. And because a bound is
  exactly the thing your own judgment is not allowed to lift, one that has silently vanished
  and one you have decided was satisfied look identical from the inside. Copy them into the
  record verbatim, and into a durable form where the system offers one.
- **A check for existing coverage can match the topic and the reader and still miss.**
  `tending-tbaguette` gains the second axis: a passage can be about your subject, written
  for your reader, and still not answer you, because it was written for a different
  *moment* — composing a report after the work rather than watching a surface during it.
  The tell is that from a search the two are the same result.
- **A sweep across several projects now says how many it could actually read.**
  Same skill. Whatever decides what to read inside each project will silently drop any
  project it cannot open at all, and that result is indistinguishable from a quiet week —
  so the number in scope and the number actually read are two figures, and both belong in
  what gets reported.
- **Being told to improve something is not evidence that it needs improving.**
  `tending-tbaguette` already warned that a pass arriving with nothing queued will go
  looking for something to change and will find it, because a corpus this size always
  holds a sentence that could be put more sharply. It now covers the stronger version —
  a run holding an instruction that names the target. That reads as settling in advance
  the question every filter in the skill exists to ask, and declining then feels like
  declining the task rather than like exercising the filter. It is not: an instruction to
  improve something is an instruction to look, and looking can honestly come back empty.
