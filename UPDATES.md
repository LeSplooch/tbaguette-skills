# Update notes

What changed, newest first, written for someone who has TBaguette installed
rather than for someone reading the diff. The landing page renders the most
recent entries below the “Fresh from the oven” rail; this file is the whole
record.

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

Writing an entry is part of shipping a change here — see CLAUDE.md. The shape
is `## YYYY-MM-DD — Title` followed by `-` bullets, newest date first, and
`scripts/generate.py` refuses to build a site if that shape or that order
breaks. A bullet may wrap across lines; the continuation is joined back on.
Everything above the first `##` is preamble and is never rendered.

## 2026-09-03 — crouton was mostly advice about prose, and prose is not where the tokens go

- **`crouton` has been rewritten around what a session actually spends.** It used
  to give most of its length to registers and word choice. Measured across agent
  sessions, reply prose is low single-digit percent of a run — the cost is what
  gets pulled *in*, and because every turn re-sends the whole conversation, a
  file read costs roughly three times its own size. A 1,300-line file is not a
  13k-token read; it's a 30k-token decision.
- **It now carries read rules you can check yourself**, not adjectives: never
  read what you already have, locate with `grep -n` then read the range, outline
  a long file before opening it, cap what a command can return, and don't re-run
  a command whose output cannot have changed. None of them trade away
  correctness — each returns the same information for less. In a sample of real
  sessions, 95% of file reads pulled the whole file and 44% of sessions re-read
  something already in context, so these are the normal case rather than edge
  cases.
- **It answers the "should I add a tool to save tokens" question, with the
  reason.** A plugin or MCP server's schema joins the per-request floor and is
  charged every turn whether or not it gets used, and the bounded-read tools
  such a server usually offers — ranged read, grep, glob — are already in the
  harness. That's now a row in the false-economies table beside the older traps.
- **Cheaper where it's needed, free where it isn't.** On a read-heavy task, run
  three times per version and per model: driven by Sonnet, the rewrite cost 10%
  less than the old text, and every single run of it was cheaper than every run
  of the old one. Driven by Opus, the two were indistinguishable — it was
  already reading in ranges and outlining before opening, and it finished the
  same task for a third less than Sonnet did under either text. So this is
  guidance that pays on the models that need it and costs nothing on the ones
  that don't. Answers were equally correct throughout. Where it did pay, it got
  there by making *more* tool calls, not fewer — many small ranged reads instead
  of a few whole-file ones.
## 2026-09-03 — The fixture nobody checks is the one describing your own system

- `grounding-test-doubles` was written about things belonging to somebody else —
  a vendor API, another team's service. It now covers the case where the double
  models *your own* configuration, which drifts for the same reason and without
  the changelog: your production code path is edited by other people on other
  days, and a fixture hears about none of it.
- The damage concentrates in fixtures belonging to a benchmark or scoring run.
  An ordinary stale fixture eventually goes red, which is self-correcting; a
  stale one in a benchmark produces a number that gets repeated in decisions,
  and nothing goes red because checking the fixture against reality is the one
  claim no suite makes.
- Includes the reading that is easy to skip: when the diff comes back, the
  fixture is not automatically the wrong half. One describing a capability
  production no longer has is a silent regression announcing itself late.

## 2026-09-03 — A success counter that is really an admission counter

- `instrumenting-for-observability` now covers the metric that is recorded when
  work is *accepted* rather than when it finishes — a status chosen, a header
  written, a job enqueued. For anything that completes later than that instant,
  the counter measures admission and reports completion, so a failure after the
  status is on the wire gets tallied as a success.
- The reason it earns a section rather than a bullet is the direction of the
  error: an uncounted event leaves a hole, and a hole invites someone to look,
  while an event counted as its own opposite produces a clean number over a
  broken period. Missing data is a question; wrong data is an answer.
- The skill's `description:` now routes on the symptom you would actually
  arrive with — a failure counter that has never once incremented — rather than
  on the diagnosis you do not have yet.

## 2026-09-03 — A run now decides what it spends before it starts reading, not after

- **`orchestrating-work-end-to-end` has a fifth envelope dial: register.** The
  other four said who answers a gate, how much run the work is worth, what a
  wrong turn costs, and who else is writing in the tree. None of them said what
  the run *spends* getting there, so the unstated default was the most expensive
  setting available — whole files pulled in, every call narrated, the background
  restated, each phase closed with a summary of what you just watched happen. A
  build charges that eight times, and every other track charges it again.
- **`crouton` owns that dial now, and it is read at the route rather than at the
  report.** By the last phase, every file a run was going to re-read has already
  been read, so a register set there can only tighten the closing prose — the
  smallest of the four spends that skill names. Set at the route it binds the
  larger half instead: ranges rather than whole files, no re-read to confirm an
  edit that already reported success, background said once and referred back to.
- **Three things stay in full prose no matter what the dial says.** The run
  record, anything the run lands, and any warning before something irreversible
  — all three are read by someone who was not there and cannot ask what you
  meant. Compress the run, never the record, and never the warning.
- **Every track reaches it, not just Build.** The Diagnose, Respond, Review and
  Change-in-place spines now name the register the same way, so a run picks up
  the setting whichever shape of work it turned out to be.
- **The run record's first line carries it too.** The envelope line in the
  record template now has five fields rather than four, which is what makes the
  setting survive a compaction — a dial that lives only in the prose is a dial
  the next context does not inherit.
- **And the dial states the rules rather than only pointing at them.** Naming
  `crouton` was leaving the actual saving one skill-invocation away, and a run
  that reads the dial and moves on collects none of it. The register bullet now
  carries the short version inline — what a read costs, read the range, outline
  before opening, cap what a command hands back, never re-read what you already
  have — so a run gets the behaviour without opening anything.

## 2026-09-03 — Before TBaguette opens a pull request for you, it has to say what that means

- **The approval gate now states what the yes does, not just what the change
  is.** `tending-tbaguette` has always required an explicit yes before pushing
  anything or opening a pull request. What it did not require was telling you
  what agreeing would create. So the question described a skill edit, and the
  thing you were actually approving was a public fork under your account, a
  branch pushed to it, a pull request open under your GitHub identity, a
  hundred-file diff that is mostly the regenerated site, and a maintainer who
  now has something to answer. It asks in short bullets now, before the first
  command that touches the network.
- **You can want the change and not want the publication.** Those are two
  different yeses and only one of them is irreversible — closing a pull request
  later does not unmake the fork, the branch, or the record. The question is
  written so you can say yes to the lesson and still say "keep it queued"
  today.
- **The gate binds inside the contribution procedure, not only above it.** The
  full pipeline moved to its own reference file, and the step that opens the
  pull request pointed at "the approval gate above" — which was in a different
  file from the one the skill tells you to work from. The binding rule is
  restated where the command actually lives, so a session following the
  procedure cannot walk past it.

## 2026-09-03 — Telling a real door from one you never tried

- `bounding-autonomous-work` now separates the two. Its door bound lists actions
  a human must own, and says that stopping at one is a correct ending — which
  makes it easy to stop at something that merely resembles one. "This cannot be
  verified without you" is a claim about your own capabilities, not an
  observation, until an attempt has failed; the skill now asks for the cheapest
  experiment that would settle it, run, before the deferral is written down. Three
  limits keep that from becoming the opposite advice: it applies while you are
  deciding whether something is a door and never after a stop condition has fired,
  the experiment is bound by the same list as everything else so the read-only
  half is what is being asked for, and a probe that succeeds never converts a door
  into a self-answer — knowing you *could* rotate the credential is not permission
  to. A probe that answers suspiciously easily gets checked against
  `reproducing-bugs` before a deferral is withdrawn on it.

## 2026-09-03 — A refused tool call is not a capability gap, and green is not the gate

- **`routing-around-capability-gaps` now separates a refusal from a prompt.** It
  already said a permission prompt is not a capability gap. It now says what to
  do when the harness declines a call outright: the tool never ran, there is
  nothing to answer, and retrying it verbatim, rewording it past the check,
  splitting it up, or reaching for a different tool that does the same thing are
  one act — defeating a control the user installed, using exactly the ingenuity
  the rest of that skill supplies. Route around a missing capability, never
  around a withheld permission. What is left is a report: the call, what it
  would have done, and everything the run did that did not depend on it.
- **`tending-tbaguette` says what a green suite actually proves.** The tests
  check the filing — registries agreeing, counts matching, manifests at one
  version, the note well-formed. Nothing in them can see whether a section is
  true, belongs where it was put, or is reachable from the description. Green is
  a precondition for the review gate, never a substitute for it.
- **And it no longer stops at "pull request opened".** A contribution gets
  reviewed, and the skill now carries the return leg: a maintainer arguing with
  a section is neither a request to delete it nor to defend it, `verifying-review-feedback`
  decides which a given comment is, and a change that goes quiet under review
  costs the maintainer more than one that was never opened.

## 2026-09-03 — Three skills learn what to do when the thing you need to watch will not talk to you

- **`routing-around-capability-gaps`: a gap in seeing routes differently from a
  gap in doing.** The ladder assumes the missing capability is an action, and
  every rung is a way of doing more work — which performs no observation you have
  no channel for. When the subject has stopped reporting, the move is an inventory
  of what in the environment already sees it: a second device pointed at the
  first, a neighbour on the same bus that logs what the subject will not, a
  downstream consumer that recorded what it received, a side effect left somewhere
  the subject does not control. It also flags this as the gap most often misfiled
  as needing a human — "I cannot see the state" is a claim about your channels.
- **`automating-repetition`: a person asked to watch is an instrument with a
  latency floor.** "Tell me when X happens" imports the round trip as a sampling
  interval, so a transition that passes in less time than a reply is invisible by
  construction — and what comes back is not late information but information about
  a state that no longer exists, which reads as current. The watcher section now
  says to put a watcher on the transition and leave the person the physical
  action.
- **`checkpointing-long-runs`: a compaction strips the tier off everything that
  crosses it.** What you verified by running a command, what you inferred from one
  file, and what you assumed because it was plausible all arrive on the far side
  in the same voice. The skill now says what to do with each: re-check anything
  still checkable, downgrade anything no longer checkable to assumed on arrival,
  and write the tier into the record for anything expensive to re-derive, while
  you still have it.

## 2026-09-03 — Two commands that report success without having checked anything

- **`automating-repetition`: a replacement that matched nothing exits zero.**
  `sed -i`, a regex codemod, a scripted find-and-replace, an `UPDATE ... WHERE`
  — every pattern-driven edit *succeeds* when its pattern is absent. Nothing
  matches, nothing is written, the file comes back byte-identical, and the
  status code says the run was fine, because it was. The skill now says that an
  edit meant to change something has to assert that it did: count the matches
  and fail on zero, diff before against after, or read the affected-row count
  back — separating "ran without error" from "did the thing", two claims a
  mechanical editor reports with the same number.
- **`diagnosing-before-fixing`: an environmental cause is the one hypothesis
  that ends the search.** "It's the network", "the runner is slow today", "that
  dependency is flaky" — accepting one of those stops an investigation instead
  of directing it, which is what makes it the cheapest thing to believe. It is
  also the cheapest thing in the whole investigation to refute, because whatever
  it blames has a liveness check measured in seconds. The skill now asks for
  that probe to be named and run before the attribution gets written down, and
  again before it gets repeated. It says explicitly that this does not
  contradict the legitimate outcome two paragraphs later: documenting an
  environmental limit is not the same act as establishing one.

## 2026-09-03 — Two ways a check can come back clean and mean nothing

- **A verification command read through a filter can lose its verdict.**
  `confirming-before-claiming-done` now names the pipeline trap in the one place
  it does the most damage: piping a noisy build or suite into `tail` or `grep`
  to read it means the shell reports the *filter's* exit status, so the check
  fails, `set -e` notices nothing, and a chained success line prints anyway. It
  points at `portable-shell-scripting` for the mechanics and says what to read
  instead.
- **A negative result from a compiled artifact is not an absence.** The same
  skill now covers the case where the search is sound and the surface is not:
  grepping a release build for a short string and finding nothing looks exactly
  like the code having been dead-stripped, and is no evidence of anything.
  Optimizers store short literals as immediate operands rather than contiguous
  bytes, minifiers rename what you are looking for, and stripped builds keep no
  symbol to match. It says what to probe with instead.

## 2026-09-03 — Seven skills, three of them from the first outside contributions

- **`orienting-in-unfamiliar-code`: a text search and an index answer different
  questions.** Searching text tells you where a string appears. A resolver — a
  language server, an IDE index, a tags file — tells you where something is
  *defined* and who *uses* it, and the two diverge exactly where orienting is
  hardest: a short name, a method shared across unrelated types, a symbol
  re-exported under another name. Text search degrades there in a way that feels
  like progress, because the reflex is to narrow the pattern and narrowing throws
  away the call that mattered. The skill now says to stand a resolver up once you
  know the language, and says the other half too: it only knows what it can
  resolve, so a **zero-references answer is a hypothesis, not a finding** —
  confirm it against configuration, templates and data files before deleting
  anything.
- **`using-tbaguette`: the check is owed twice on a long response.** Its rule was
  always per response, and quietly assumed a response is short. Measured in one
  harness the median ran four tool calls — but the 99th percentile ran 49 and one
  reached 194, and fifteen substantive responses in a row invoked nothing while
  the notice fired correctly at the top of every one. The guidance was never
  missing; it had scrolled away. So it is owed again once a response has grown
  past the point where its opening still counts as recent.
- **`automating-repetition` learns two things about triggers.** First, where a
  proposed rule's inputs are already recorded — logs, transcripts, ticket history
  — replay them and measure how often it *would* have fired instead of watching
  for a month. A guard that goes off on a fifth of all occasions is furniture
  before it is armed, and replay also answers the question watching answers badly:
  whether the condition separates anything at all. Second, some conditions arise
  on the world's schedule rather than inside your procedure, and those need a
  watcher rather than a step — with the two costs named, since a dead watcher's
  silence is identical to a quiet day.
- **`choosing-test-scope`: automating an exclusion list audits half of it.** A
  guard that perturbs each excluded case and asserts nothing changes will find the
  stale entries in batches. What it verifies is the *claim*, not the
  *explanation* — an entry whose reason names the wrong component passes forever
  if the case is unreachable for some other reason. That is worse than it sounds:
  the reason is now the only part that can be wrong, nothing is looking at it, and
  the reason is the half the next reader acts on.
- **`designing-test-data`: near-misses, not just examples.** Fixtures for a rule
  that selects — a matcher, filter, alert condition, suppression — get written by
  whoever wants it to work, so every one comes from the side that should match.
  Such a corpus cannot express over-firing at all. Keep a boundary pair instead:
  one input just outside that must not match, one just inside that must, so
  loosening and tightening both go red.
- **`orchestrating-work-end-to-end`: what Isolate and Land mean with no
  repository.** Two phases named git, and plenty of real work has none —
  operations on a live system, data analysis, research, an incident on
  infrastructure nobody version-controls. Isolate is *the change cannot reach what
  you would be sorry to break*, and its other half travels unchanged: measure the
  baseline first. Land is *the artifact reached a durable home the next person can
  find*. The point is the failure it prevents — reading two phases you cannot
  enter and concluding the whole track is for somebody else's kind of work.
- **`tending-tbaguette` is shorter, and now covers landing.** Its contribution
  pipeline moved into a reference file loaded when you need it, roughly halving
  what the skill costs in every conversation it watches. What it gained: check
  the library does not already say your lesson *before* drafting it, since a
  written section is hard to abandon; report a check that fails for reasons that
  are not yours with a baseline from a pristine upstream checkout, or it reads as
  your change breaking the build; and say which files in a pull request are the
  change and which are the regenerated site.

## 2026-09-02 — Four more skills, and a null result that means the opposite of what it looks like

- **`diagnosing-before-fixing`: an exact null result indicts the plumbing, not
  the parameter.** Change a knob, watch the output not move, and you have two
  very different findings depending on how exactly it did not move. A little, or
  within noise, means the input arrived and its effect is weak. *Byte-identical*
  means the input almost certainly never arrived — a real value with a real
  effect essentially never lands on the same bytes, but one that was overwritten,
  defaulted, or dropped on the way in does exactly that. Perfect identity is the
  stronger signal and reads as the weaker one.
- **`choosing-test-scope`: an exclusion names a mechanism, not a
  justification.** Every skipped test, ignored rule and allowlist entry carries a
  reason, and there are two kinds that look identical the day you write them. "Not
  applicable here" stays true after it stops being true, so the entry outlives the
  change that should have deleted it and starts hiding a live defect. "The harness
  constructs the other implementation, which never reads this" goes visibly false
  the moment that changes. Write the reason so it *could* be falsified — and when
  one entry turns out wrong this way, re-read the whole list, because they fail in
  batches.
- **`orienting-in-unfamiliar-code` now says to check for the file the project
  wrote for you.** Most repositories you land in have a root instruction file
  written to answer exactly the question you are opening the repo with. It is
  worth reading before anything you would otherwise have to infer — as intent
  rather than as fact, like any prose in a repo.
- **`designing-apis`: stateless core, identity per request.** An interface that
  remembers you between calls has bound every later call to one instance, and
  everything downstream inherits it — affinity, dropped sessions on deploy,
  instances that cannot be replaced under load. The test is whether two
  consecutive calls can land on different instances with no coordination; if they
  cannot, that is a scaling limit written into the interface rather than the
  deployment, which is the more expensive place to keep it.
- **`caching-strategy`: let the response say how fresh it is.** A TTL chosen by
  the caller is a guess about data the caller does not own, and one TTL per
  endpoint has to be short enough for its most volatile result. Let each response
  carry its own freshness, with two guardrails: a per-response directive may only
  *shorten* what the consumer would hold, and a response that says nothing gets
  the consumer's default — never "forever", never "not at all".

## 2026-09-02 — Five skills learn the failure that looks exactly like success

- **`formidable`: an entrance must degrade to appearing, not to absence.** Put
  the visible state in the base rule and let the animation supply only the state
  it starts from. Written the other way round it looks identical while it works,
  and any browser that skips the transition leaves a focus-trapping,
  scroll-locking modal with nothing drawn on it. The question it hands you: if
  the motion never runs, what is on screen?
- **`finding-resource-leaks`: a release that waits to be told.** The classic leak
  is a release that got skipped; this is one that is never called, because it
  hangs off an event you do not control. What leaks is usually global, so the
  damage is not an unseen handle but everything else's behaviour — a page-wide
  lock never lifted long after the thing that set it is gone. Drive the release
  from the state that actually changed rather than from being told it changed,
  and ask what stays changed if the event never arrives.
- **`configuration-management`: discovery comes before precedence.** A config
  file the tool never looks for has no precedence at all. Tools search a fixed,
  documented list, and a file placed to match your project's naming convention
  is invisible unless that path happens to be on it. Where several tools — or
  several surfaces of one tool — must read the same config, the correct location
  is the *intersection* of their lists, not the union.
- **`choosing-test-scope`: a setup too small to show the difference has not
  tested for it.** A concurrency test below the parallelism the race needs, a
  truncation test whose fixture fits, a layout measured in a frame barely bigger
  than the element. Right layer, real assertion, and a value that would have been
  the same either way. Ask what the broken version would have measured; if the
  answer is "about the same", the number describes your setup, not your code.
- **`writing-the-failing-test-first`: red both times is not evidence.** Watching
  a test fail only proves something if it passes in the other condition. A check
  that was already broken — a stale selector, a fixture that no longer loads —
  goes red under every mutation and is indistinguishable from one that works. The
  pair is the evidence, never the single reading.

## 2026-09-02 — Every harness works now, and a skill learned why they hadn't

- **`confirming-before-claiming-done` gained the section this week was a
  worked example of.** A test suite that only compares a project against itself
  proves the parts agree with each other, which is a different claim from the
  one you are making when you say an integration works — and it is most
  convincing exactly when it is most wrong. The new section gives you a
  mechanical tell for it (read the assertions and ask which would fail if the
  other side changed its mind), what to do when you cannot exercise the real
  other side, and why a process exiting 0 says nothing about whether anything
  received what it wrote. If you write code that has to satisfy someone else's
  format, this one is worth reading on its own.
- **Three integrations were delivering nothing, and none of them looked
  broken.** The files were there, the hooks ran, they exited cleanly — and the
  harness quietly ignored an output shape it did not recognise. All three are
  fixed below, and every other harness was re-read against its own
  documentation to find out whether the same was true of it.
- **Cursor was doing nothing at all.** The standing check-the-skills-first
  rule never reached the model, because Cursor wants a different shape than
  the one it was being handed. It works now, and Cursor also gains the
  periodic re-assertion it never had — so a long session stops drifting away
  from the skills the way it used to. If TBaguette has felt inert in Cursor,
  that is why, and it is worth trying again.
- **Codex had its bootstrap switched off.** Codex could find the skills but
  was never told to check them. It now gets the same treatment Claude Code
  does: the rule at session start, and again on every turn. Install it with
  `codex plugin marketplace add LeSplooch/tbaguette-skills`.
- **GitHub Copilot works in all three places it runs** — the CLI, VS Code, and
  the coding agent. **CLI:** `copilot plugin marketplace add
  LeSplooch/tbaguette-skills`, then `copilot plugin install
  TBaguette@tbaguette-dev`. **VS Code:** run **Chat: Install Plugin From
  Source** from the Command Palette and give it this repo's git URL. **Coding
  agent:** add TBaguette to `enabledPlugins` in your repository's
  `.github/copilot/settings.json` — `PORTING.md` has the block to paste.
- **Copilot has no Skill tool**, so being told to use one was sending you
  looking for a button that isn't there. There a skill is a slash command —
  `/TBaguette:orienting-in-unfamiliar-code` — and skills also load on their
  own when what you're asking matches one. TBaguette now says whichever of
  those is true where you are.
- **Devin is the one that cannot be fixed.** It finds the skills, but its own
  documentation is clear that skills are chosen from task context rather than
  run automatically — there is no session-start mechanism to hang the standing
  rule on. On Devin, name the skill you want. `PORTING.md` says so plainly now
  rather than implying otherwise.
- **Gemini CLI, Kimi Code, OpenCode, Pi and Hermes were checked and left
  alone.**
  Gemini turns out to be the strongest of the lot: it re-sends its context
  with every prompt, so it has never needed the per-turn reminder the others
  do. `PORTING.md` records what was verified for each, and which two are
  built on APIs their own vendors call experimental.
- The install prompt on the site knows every one of these commands, so
  pasting it into whichever agent you are in is still the whole procedure.

## 2026-09-02 — Contributing: pick the right file, make it reachable, and stop at the gate

- `tending-tbaguette` now covers the step that used to be left to instinct:
  which existing skill a lesson belongs in. File by the family of judgments the
  lesson joins, not by its subject — a lesson about an installer refusing to run
  is a reading-the-instrument lesson, not a dependency one. The test is
  neighbourliness: read the candidate file's section headings and ask whether
  yours reads as a sibling or a visitor.
- It also adds the check with no build gate behind it, and the one that quietly
  wastes the work. A skill's `description:` is the only part of it that is always
  loaded, so
  it is the whole of the routing — a new section in a file whose description
  never mentions the question it answers is unreachable, and every suite stays
  green. If the sentence that made you write it would not land on that file, the
  description is part of the change.
- And the approval gate now says what happens when nobody is there to answer it:
  nothing. It is not substituted the way other gates are. An unattended run does
  the edit, the suite, the adversarial pass and the local commit, then stops with
  the work staged and says what is waiting on a yes — which is the run ending
  correctly rather than failing.

## 2026-09-02 — Choosing between an agent that starts cold and one that inherits everything

- `fanning-out-independent-work` assumed a dispatched agent gets a context built
  only for its slice, because for a long time there was no other kind. Harnesses
  now also offer a forked agent that inherits the whole session and its warm
  cache — genuinely cheaper, and the wrong instrument for a fan-out.
- The skill now says why: every discipline it teaches is enforced by the agent
  knowing nothing. The prompt has to be complete because nothing else is there;
  the write scope has to be stated because it cannot be inferred; the report has
  to stand alone because the reader shares no memory. A fork removes the cold
  start and all three forcing functions with it, and several forks of one session
  are several copies of one set of assumptions.
- A short table splits it by what the task needs to know, and the section puts
  the cost multiplier of running work across several agents into the judgment
  rather than leaving it as a background fact.

## 2026-09-02 — Let the harness fire the checkpoint you were going to forget

- `checkpointing-long-runs` says to checkpoint when the context is visibly
  filling, and that has always been the weakest line on its list: it asks the
  part of a run least able to judge its own state to judge its own state.
- The skill now points at the alternative. Where a harness exposes the boundary
  as an event — a callback before compaction, at session end, when a subagent
  returns, when a worktree is removed — wiring the checkpoint to it turns a
  discipline into a mechanism. Pre-compaction in particular is the exact instant
  the section was asking you to notice, handed over as something that cannot
  forget.

## 2026-09-02 — Auditing a dependency whose payload is prose, not code

- `auditing-dependencies` now fires on the question "should I install this
  skill, plugin, extension, or tool server?" and answers it. Every other check
  in that skill assumes a dependency is code with a call site; an artifact made
  of instructions takes effect the moment something reads it, with the reader's
  full privileges and none of its own. The footprint row and the install-script
  row both see nothing, and the thing still changes what your tooling does.
- Three questions the code-shaped checks miss: does every instruction serve the
  stated purpose, is there anything here you cannot see (instructions have been
  hidden in the `U+E0000` tag block, where they survive visual review and reach
  the model intact), and does it write into anything that outlives it — because
  an instruction that edits your repository's own instruction file turns a
  removable dependency into a resident one.
- The upside the section leans on: this is the only dependency class you can
  realistically read in full. A library is fifty thousand lines you will never
  open. A skill is a page of English, which makes the review tedious rather than
  impossible and removes the usual excuse.

## 2026-09-02 — The person you are working with is the bottom rung of the routing ladder

- `routing-around-capability-gaps` extends its capability-not-preference
  guardrail to cover the user. Handing someone a block of commands to paste is a
  routing decision — it asserts *this cannot be done from here* — and it is wrong
  more often than a model swap is, because the alternative usually went
  unchecked. A permission prompt is not a capability gap; neither is a slow
  command, nor a step that merely feels like it ought to be theirs.
- Three things do belong to the user, recognizable by what they need rather than
  how they feel: a secret only they hold, a decision that is theirs, and an
  irreversible action a run may prepare but not take. Everything else handed over
  is an unchecked capability claim — the same substitution this skill exists to
  prevent, aimed at the person instead of at the work.

## 2026-09-02 — A run is not finished while the things it started are still running

- `finishing-what-you-started` now covers the part of a run that never gets a
  ledger line: the backgrounded build, the watcher, the long benchmark, the dev
  server, the dispatched agent. None of them is a deliverable, so none of them
  is in the acceptance criteria, and every one of them outlives the report that
  says the work is done.
- The close-out gains one more question, answered by looking rather than by
  remembering: what did this run start that is still running? Every answer gets
  a disposition — stopped, or deliberately left running and said so, with what it
  is and how to stop it. Left running on purpose is a fine outcome; left running
  silently is a cost the reader discovers hours later, and it reads to them like
  the run losing track of itself.

## 2026-09-02 — The check that refuses to run has already told you something

- `diagnosing-before-fixing` adds the case where the instrument declines to take
  a reading at all: a preflight check, a version gate, a compatibility guard, a
  health probe that will not proceed. It arrives looking like an obstacle, and
  the reflex is to hunt for the flag that turns it off.
- The skill now says to read the guard before overriding it — what condition does
  it test, and is that condition true here? A guard fires because somebody knew
  something about the target that whoever is bypassing it does not, and forcing
  past it converts a clean, free failure into a confident silent no-op. A guard
  that really is wrong is worth suppressing *and* worth fixing; a `--force` with
  no answer behind it is neither.

## 2026-09-02 — Feeding both variants the same inputs, and knowing when that stops helping

- `performance-profiling` picks up the rung above interleaved A/B runs: give
  both variants the same inputs in the same order and difference them per input
  instead of comparing averages. Whatever made input 47 slow made it slow for
  both arms, so the subtraction cancels it. Same generator, same seed, fed
  twice — and on a noisy workload it is often the difference between a result
  and a shrug.
- It also says what to report alongside: the correlation between the two arms'
  per-input results. How much the pairing buys is a property of the pair, not of
  the technique, and quoting the reduction on its own reads as though it were the
  technique's. A low correlation has a usual cause worth naming — the change
  altered *which work happens* rather than how fast the work went, so most of
  each input's difficulty was never common to both and had nothing to cancel
  against.

## 2026-09-02 — A threshold that measures consistency will adopt changes nobody can feel

- `performance-profiling` now says what a significance test actually answers.
  It asks whether a difference is *reliable*, never whether it is *large* — so a
  variant better by one percent on every trial has a tiny difference, a tinier
  spread, and a ratio between them that clears any threshold effortlessly. The
  rule is working as designed and promoting changes that will never be observed.
- Every significance threshold now gets a minimum effect size beside it, written
  in the units of the decision: milliseconds at p95, bytes, queries, dollars.
  Both clear or the change does not land.
- It also carries a diagnostic that needs no statistics: run the whole procedure
  twice and see which decisions move. The settings that wander between two runs
  of identical code are exactly the ones the criterion was never really
  deciding.

## 2026-09-02 — "Did this help?" is a question about a run that did not happen

- `confirming-before-claiming-done` gains a section on the baseline that has to
  be built rather than read. Any claim that a cache, an index, a retry policy, a
  compression step or a tuned parameter *helped* is a comparison against a run
  where it was switched off — and the number that is actually to hand is another
  reading off the treated run, close enough to the result to look like the other
  half of the ratio.
- The discriminator is one question asked before the division: if the
  intervention were switched off, would this denominator change? A `no` means it
  is a second measurement of the treated run, not a baseline. The failure it
  catches runs in the costly direction — it reports no effect for something that
  is working, and the sensible response to no effect is removal.

## 2026-09-02 — The determinism test that could not see the clock it depended on

- `testing-the-untestable`'s "same seed, twice" check now carries the flaw that
  makes it pass over a system that is not deterministic at all. Two runs held
  back to back in one process read the same clock, the same environment, the
  same working directory and the same machine identity, so a system genuinely
  depending on any of them still emits identical output and the check goes
  green — for as long as the two runs stay adjacent.
- The skill now says to vary the ambient condition *between* the pair rather
  than around it, and what to do when varying it is genuinely impossible: the
  test still earns its place, and its name is where the narrower guarantee gets
  written down, so a later reader is not told `is_deterministic` about something
  that was only ever checked inside one process.

## 2026-09-01 — Capability gaps: the environment you stood up answers, and is still missing services

- `routing-around-capability-gaps` checks a tool on three layers — installed,
  credentialed, reachable — and the third asks whether a real call returns a real
  answer. That is the right question for one tool and not enough for a whole
  environment brought up to close a gap: a virtual display, an emulator, a container,
  a sandboxed browser. None of those is a smaller version of the real thing. Each is
  the real thing minus a particular set of services, the environment will not tell you
  which ones, and everything that does not need what is absent behaves perfectly.
- The example the skill now carries: on a bare virtual display, clicks land, navigation
  works and screenshots come back correct, because none of that needs a window manager
  — and keyboard input silently does nothing, because setting input focus does. Nothing
  errors, nothing is logged, and the natural reading of a keystroke with no effect is
  that the application ignored it.
- So: enumerate the interactions the work needs — click, type, focus, drag, copy, drop
  a file — and prove each one against a known-correct response before anything is built
  on top. That order matters more than it looks, because completing such an environment
  is not additive. Starting a window manager to fix focus also gives every window a
  titlebar, which moves every screen coordinate down by its height and invalidates a
  click map that was working a minute earlier.

## 2026-09-01 — Confirming done: read the payload, not the code that writes it

- `confirming-before-claiming-done` now covers the questions that are about something
  the system produces rather than about whether it works — what key a field ends up
  under, what an enum looks like once it is serialized, what is actually in the column.
  Reading the code that produces it is the reflex and it is the wrong place: an
  annotation re-cases the name, a custom encoder overrides the declaration, an
  inherited default comes from nowhere near the type, and a library changes its own
  default between releases. The declaration is a request; the bytes are the answer.
- Produce one payload and read it. Serialize a single real value and print it, read
  one stored row, dump the header off one real request — usually one command, usually
  faster than the reading it replaces, and it settles the question instead of raising
  confidence in a guess. Reading the code is a hypothesis; producing the payload is
  evidence. The claims table gains a row for it, whose only real evidence is one real
  value put through the real encoder with the output printed.
- The half that costs the most when it is missed: a grep or a throwaway script written
  to answer the question is untested code, and it can be wrong in the direction that
  costs the most — reporting the absence of what is there. `diagnosing-before-fixing`
  already covers telling a broken instrument from a refuted hypothesis, and its
  discriminator is that a broken apparatus usually fails in several ways at once. A
  pattern that is quietly too narrow fails in exactly one way: it returns nothing,
  cleanly, and nothing about that looks like a malfunction. Which is the argument for
  not building the checker in the first place.

## 2026-09-01 — Test scope: a hundred passing tests can agree with each other and be wrong

- `choosing-test-scope` answers "a bug escaped every layer" by asking which is the
  lowest layer that could have caught it. It now covers the case where the honest
  answer is "no layer", because the escape was never about layers. A systematic
  offset — a per-unit figure that omits a fee the total includes, a rate applied at
  the wrong precision throughout — makes every number wrong in the same way and every
  number plausible, and a test checking one of them passes whenever the expectation
  holds the same mistake. It usually does: it came from the same reading of the same
  specification by the same person.
- The skill already named that failure for fixtures — one party, no second party to
  disagree, the same misreading encoded twice and passing forever. This is the
  arithmetic form of it, and adding more point-checks does not escape it at any layer.
- What does: a quantity computed by a *different route* and asserted against the
  first. A running total accumulated from individual events against the same figure
  derived from opening and closing balances; inventory on hand against receipts minus
  shipments. Neither route is the authority — the assertion is that they agree. It is
  normally one test per accumulator, and the one condition on the layer is that the two
  routes must not both run through the function that holds the error.

## 2026-09-01 — Determinism: seeding everything and it still differs every run

- `testing-the-untestable` now covers the source of randomness that survives seeding.
  Its seam table gives identifiers their own row and their own injection point, and
  skipping that row is what this failure is made of: v4 UUIDs, session tokens and
  generated slugs are usually minted by a library drawing on the operating system's
  entropy, not on the generator you seeded. Seed every source you own and the ids keep
  changing anyway — so the same-seed check still comes back with a difference, and the
  search sets off after a second bug that does not exist.
- Where that difference does the most damage is a sort. A comparator's final term
  decides the order of every pair the earlier terms tied on, so `(created_at, id)` with
  a randomly generated `id` breaks each of those ties at random. The code reads as
  careful, because it is a deliberate total ordering — just a different one each time.
- The rule the skill lands on: tiebreak on something intrinsic to the record — a
  natural key, its insertion index, a hash of its content — never on an id minted
  alongside it. And ties are the normal case rather than the edge case: any batch that
  stamps its rows from a clock read once at the top gives every one of them the same
  value, so across that batch the tiebreak is not breaking ties, it is the sort.

## 2026-09-01 — Confirming done: a green suite is not a program that starts

- `confirming-before-claiming-done` already named several ways a check can be fresh,
  first-hand and still not prove the claim. It now names one more: the check that ran
  in the wrong *context*. Anything placed inside a hook a framework calls — a setup or
  init function, a registration callback, a plugin entry point, an installer's
  post-install step — is compiled as ordinary code and does not execute under the
  conditions the rest of the program executes under. It runs where the framework
  decided: possibly before the async runtime is up, before there is a window to draw
  into, on a thread that does not own what the line touches, or before configuration
  has been read.
- Nothing at the call site says so, which is why the usual evidence misses it. A test
  that calls the hook directly supplies the test's surroundings rather than the
  framework's, so build, typecheck and every last test can be green on something that
  fails on every single start.
- The tell the skill hands you costs nothing to look for: if the framework ships its
  own version of the thing you were about to call — its own spawn, its own timer, its
  own way onto the main thread — that wrapper exists because the general-purpose one
  does not work there. The claims table gains a starting-up row, whose only real
  evidence is starting it the way it will actually be started and watching it get past
  the hook.

## 2026-09-01 — Shell: a loop that waits for a process can be waiting on itself

- `portable-shell-scripting` covered killing a process by pattern. It now covers
  waiting on one, which is the same bug wearing the opposite symptom:
  `until ! pgrep -f 'thing'; do sleep 20; done` never exits, because the waiting
  shell's own command line contains the pattern. Nothing dies and nothing errors, so
  there is no failure to notice — only a loop that runs for hours, and a second copy
  of it the next time the shape looks obviously right.
- The half worth having is what to wait on instead: the artifact the job produces, a
  file it touches when it finishes, or the job's own exit status via `wait "$pid"` —
  and always with a timeout, so a wait that is wrong anyway gives up in minutes. The
  section is now headed "Killing and waiting on processes by pattern" and the skill's
  description names wait loops, since nobody writing one was ever going to look under
  a heading about killing.
- Also new: a precise pattern is still not a private one. Defeating self-match with
  the bracket trick or a tighter anchor closes one direction and leaves the other
  open — the process table is shared with the user's editor, window manager and
  browser, so a pattern naming a common binary matches a stranger's copy of it and
  the kill takes that one too. The PID saved at spawn is the only handle that cannot.

## 2026-09-01 — Contributing: the description cap, and checking your words against the glossary

- `tending-tbaguette` now warns about the one build gate that fires on the easy path
  — editing an existing skill rather than adding a new one. `description` is capped
  at 1024 characters by the Agent Skills format, and the suite enforces it, so past
  the cap a newly-noticed trigger has to displace an older one instead of joining
  the list. That can make a description tweak an edit to a sentence you did not come
  to touch.
- It also adds a check nothing automated can do: grep the library for your key noun
  before adopting it. A captured observation arrives in whatever words were to hand,
  and some of them already mean something narrower here — `seam` is spoken for. That
  is how a second, vaguer sense of a defined term gets into the corpus, and it is one
  `grep` to avoid.

## 2026-09-01 — Dependencies: what to do when there is no hash to check

- `auditing-dependencies` assumed throughout that the canonical integrity signal
  exists. It now covers the artifact least protected by the rest of the skill — the
  one-off download outside any package manager, whose publisher lists no digest for
  that file at all.
- A missing hash is not permission to skip verification. It removes the one check
  that would have settled the question alone and leaves several that settle it
  together: exact byte size against the published size, container magic and
  structure, an internal build timestamp consistent with the release date, the
  archive's own listing carrying the expected entry point, the same bytes fetched
  over a different network path.
- The skill is explicit that this is not a signature and does not replace asking for
  one. It trades a single thing that would have to be broken for several independent
  things that would all have to be forged, and says to record which signals were
  used so the next reader inherits a verification with a known shape.

## 2026-09-01 — Test scope: a guard nothing calls passes every test written for it

- `choosing-test-scope` now separates guard code from ordinary code when picking a
  layer. An unwired feature does nothing and somebody notices, because what they
  asked for is visibly missing. An unwired redaction step, permission check, or
  validator leaves nothing missing at all — the output still appears and the check
  simply never ran, while its unit suite goes on passing.
- The skill now says why that suite could never have helped: whether anything calls
  a function is a question about its callers, and coverage of the callee does not
  answer it. The test that carries the weight runs a real input through the real
  call path and asserts on what the pipeline emitted.
- It also names the reason these survive so long. The failure usually errs safe, and
  a safety property that is accidentally too strong produces no symptom — so an
  audit that only looks for the too-weak direction finds nothing and concludes all
  is well.

## 2026-09-01 — Shell: don't put the cleanup and the relaunch in one command

- `portable-shell-scripting` already warned that a pattern kill can match the shell
  that runs it. It now covers the neighbouring case: `pkill -f foo; start foo &`
  reads as stop-then-start and is not. The replacement can already be in the process
  table when the pattern is evaluated, so the kill takes the very process it was run
  to make room for.
- The symptom is what makes this expensive. The job reports starting and then
  produces nothing, which reads as the job failing rather than as the cleanup having
  killed it — so the debugging goes to the wrong process. Kill, confirm the target
  is gone, then start.

## 2026-09-01 — Release notes: a fix line claims the reader had the bug

- `writing-release-notes` now names the check that separates a fix from internal
  churn. A `Fixed:` line quietly asserts that this reader could have run into the
  bug, and the commit log cannot settle that — it records when a bug was fixed,
  never whether anyone outside the team was ever exposed to it.
- The span that can settle it is the one the notes are for: from the build this
  audience is on to the build they are getting. Two entries fail it while looking
  like real fixes — a bug introduced and fixed between two of their builds, and a
  fix for something that never produced a symptom at all.
- Keeping such a line is still a legitimate call. The skill's point is that running
  the check makes it a decision rather than an accident.

## 2026-09-01 — Naming: when the same string is both a display name and an identifier

- `naming-things` now covers the rename its cost table could not express. That table
  sorts occurrences by where the name lives, which works while every occurrence is
  the same kind of thing. Renaming a product whose old spelling stays valid for users
  is not that: the same string is a word a human reads in some places and a token
  something resolves in others — a path segment, an invocation prefix, a manifest
  field, a config key — and the two are not separable by file.
- The skill now says to partition by role before touching anything, asking it per
  occurrence rather than per file, and to leave every resolved occurrence alone. A
  search-and-replace is wrong on all of those, and wrong silently.
- It also names what to do afterwards: pin the identifiers with a test, because both
  spellings now sit in the same files and the next reader will tidy the
  inconsistency. That is a regression test guarding against the change after this
  one rather than against this one.

## 2026-09-01 — Automating repetition: the one-shot bulk edit, and the check its validator cannot be

- `automating-repetition` now covers the case its ladder never fitted. The
  report-then-plan-then-act progression assumes a tool that will run many times and
  can earn trust across them; a one-shot mechanical edit across thousands of
  generated values gets a single run, and a diff that size defeats the review that
  would normally catch the mistake.
- What replaces the ladder is two checks that fail in unrelated ways: an applier
  written to refuse a file rather than apply-and-report, and a second check aimed at
  content rather than shape. The first is blind by construction to anything shaped
  correctly and meaning the wrong thing.
- The section also names the question that generates the second check — what class
  of error would pass every assertion I just wrote? — and the reason a validator
  that catches nothing was still worth writing.

## 2026-09-01 — Diagnosing: a result that is wrong in several ways at once is a broken experiment

- `diagnosing-before-fixing` now separates a refuted hypothesis from a
  malfunctioning experiment. When the test you built to settle a hypothesis comes
  back far worse than the baseline, that reads as a refutation — and it is one only
  if the experiment actually ran.
- The discriminator is a count, available before any analysis: one wrong idea about
  the cause produces one deviation, so a run that is anomalous in several unrelated
  ways at once is describing the instrument, not the world. The verdict on it is
  "no result", not "refuted" — which matters because a hypothesis dropped this way
  leaves nothing behind pointing back at itself.

## 2026-09-01 — Two skills now fit inside the Agent Skills description limit

- `confirming-before-claiming-done` and `offering-the-next-move` had descriptions
  of 1107 and 1037 characters. The Agent Skills format caps a description at 1024,
  so on any harness that validates the frontmatter rather than merely reading it,
  those two skills were the ones that could fail to load — quietly, and only on the
  harness furthest from where they were written.
- Both are now under the cap with every routing trigger intact. Nothing about when
  either skill fires has changed; they say the same things in fewer words.

## 2026-08-30 — Diagnosing: a factor that never varied is not a factor you ruled out

- `diagnosing-before-fixing` now names the dismissal that survives a whole
  investigation: deciding a cause is eliminated because every run so far shows
  it was the same. A factor that never varies cannot explain variance, which is
  not the same as it not being the cause — and the check is one question, "name
  the run where it was different."
- It carries the reason the mistake is expensive rather than merely wrong: the
  dismissal is made once, early, in half a sentence, and every later hypothesis
  is built on top of it. When several individually-sound fixes have all failed,
  the thing to look at is the factor all of them held fixed.

## 2026-08-29 — The orchestrator now asks who is answering, not just what the work is

- `orchestrating-work-end-to-end` reads a second thing before the first action:
  not just what shape the work is, but what the run may assume about the world
  it runs in. Who answers a gate — someone replying within a turn, someone
  replying tomorrow, or nobody at all. How much run the work is worth. What a
  wrong turn costs. Who else is writing to the same tree. Its gates were always
  written for one setting of those four, and nothing ever said so.
- Work that nobody will see until it is finished now has an answer that is not
  "use your judgment". New skill: `bounding-autonomous-work`. A gate whose
  answerer is absent gets **substituted, not skipped** — an approval becomes a
  written design carrying the approach that lost and a condition that fires if
  it turns out wrong; a clarifying question becomes a recorded ruling naming the
  reading that lost, never a fact. It adds the four stop conditions a long
  unattended run needs written down before it starts, and the one gate that has
  no substitute in any circumstances: an irreversible action still gets a human,
  however obviously correct it looks at hour six.
- Small changes get a real lane rather than a warning. Express runs four beats
  instead of eight, behind entry conditions you check rather than hope — the
  target already known, one reviewable diff, proof a line away, nothing live,
  nothing new introduced. The floor under it is lower and is not zero, and it
  only ever promotes: the judgment that says "this got smaller" is made by the
  part of a run that most wants to be finished.
- New skill: `checkpointing-long-runs`, for work that outlives the context
  holding it. Its point is that a long run does not fail by forgetting — it
  fails by remembering fluently and partially, so the account you hold of your
  own run is confident and missing exactly what was never restated. It ranks
  what to write by what it costs to lose, and the top of that list is the dead
  ends, which almost nobody writes down.
- New skill: `responding-to-incidents`, and a Respond track to run it from.
  Ordinary debugging reproduces first and fixes second; an outage inverts that,
  because every minute spent reproducing is spent on users. It covers declaring
  it out loud early, preserving the evidence your own mitigation is about to
  destroy, ranking mitigations by how easily they come back out rather than by
  how right they are, holding all three incident roles when you are the only
  responder, and handing back to an ordinary diagnosis once people are safe.
- Two more tracks that were missing: **Review**, for judging work you did not
  write, where the gate is coverage of the diff rather than confidence in a
  conclusion; and **Author**, for a deliverable that is a document, whose verify
  phase is the one that always gets skipped. The skill had been describing
  reviews as something it routed since it shipped, with no row for one.
- The orchestrator can now be called directly with a verb — `route` to name the
  track and the dials and stop there, `resume` to enter at the first gate not
  yet met, `gate` to say what evidence is missing, `record`, `handoff`, `abort`.
- It also gained a Refuse list, in the same register as `formidable`'s: a plan
  document for a two-task change, a clarifying question you could answer by
  reading one file, a track named after the first three edits. None are banned.
  Reaching for one when the choice was free means you were performing process
  rather than routing.
- Later the same day, two of the three new skills were reshaped to lead with
  what they make you **write down** rather than with the judgment they argue
  for. `responding-to-incidents` now opens on the six artifacts an incident
  produces — declaration, impact statement, timeline, evidence manifest, comms
  cadence, handback note — each with required fields and each with a note on
  why it does not get written by itself. The cadence in particular is now an
  artifact with a form: a stated interval, honoured even when nothing has
  changed, rather than an intention to keep people posted.
- `checkpointing-long-runs` gained the thing it was missing: what a single
  checkpoint has to contain, and a required four-part form for a dead end —
  what was tried, the narrow thing it actually rules out, what it *looks* like
  it ruled out and did not, and how many trials the null rests on. "Pinned the
  clock, no change" reads as "not a timing issue" and means nothing of the
  sort; every cache, database, and broker keeps its own clock and none of them
  was pinned. Deprioritized is not eliminated, and the difference now belongs
  in the file rather than in the head of whoever ran it.

## 2026-08-28 — Running TBaguette next to the other libraries you already have

- Getting started now answers the question nobody asks until something looks
  broken: what happens when another skill library is installed beside this one.
  Nothing has to be uninstalled to make room. Two skills that share a name stay
  separately reachable, because TBaguette's are always called with its prefix —
  `TBaguette:naming-things` is this library's, the same name without the prefix
  is whoever else's.
- If you also run something that injects its own check-the-skills-first notice,
  Superpowers being the likely one, both fire and both apply. They are not
  competing: `using-tbaguette` speaks only for TBaguette's own skills. When the
  other library has the better skill for what you are doing, that is the one to
  use — the rule was always to check what is available, never to prefer this
  library's answer for being this library's.
- Removing it is deleting one directory, and the page now says so outright:
  `~/.claude/skills/TBaguette` and nothing else. No uninstaller, no settings
  entry to unpick, nothing left behind to unwind later.

## 2026-08-28 — TBaguette is free software, and you can improve it from your side

- TBaguette is licensed under the GNU General Public License, version 2. Pass a
  changed copy of these skills on to anyone and it goes on under the same terms,
  with source. Nothing about how you *use* them changes.
- New skill: `tending-tbaguette`. It catches the lessons worth keeping while you
  work — a correction that generalizes past the codebase you are in, a gap you
  hit in a skill while that skill was running — and turns them into a pull
  request here. It asks before anything is pushed, every time, and a yes for one
  contribution is never a yes for the next.
- Editing a skill inside your own install is a dead end:
  `keeping-tbaguette-current` will not update a plugin directory that has local
  changes, so a hand-edit silently pins your install at that commit for as long
  as it sits there. `tending-tbaguette` is the way out, and a merged change
  comes back to you through the ordinary update.
- `orchestrating-work-end-to-end` now runs it immediately after the currency
  check, and `keeping-tbaguette-current` hands the local-changes case over
  instead of leaving you at a status command with no next step.
- `tending-tbaguette` names where a project-specific rule actually belongs: the
  project's own `CLAUDE.md`, which sits *above* a library skill instead of
  modifying it. Most edits someone wants to make to an installed skill are not
  corrections to it — they are one project's fact colliding with a deliberately
  general rule, and that was never an upstream change to begin with.
- It also corrects the belief that makes hand-editing feel harmless: that the
  next update quietly overwrites your edit. It does not. The update reads the
  tree, backs off rather than discarding your work, and stops there — so the
  edit persists and blocks every update behind it.
- The library now goes by its full name, **TBaguette's Atelier**, on the site, in
  the READMEs, and wherever your agent lists its installed plugins. Nothing you
  type changes: the plugin is still `TBaguette`, skills are still
  `TBaguette:<skill-name>`, and the install still lands in
  `~/.claude/skills/TBaguette`.

## 2026-08-27 — Saying something is not installed now has a rule of its own

- `confirming-before-claiming-done` names another way a check goes wrong, and it is
  the first one in that skill about claiming something is *not* there. Open the one
  place a plugin, a patch, a hook, or a migration would live, find it untouched,
  report "not installed" — fresh evidence, gathered first-hand, and wrong. That
  location was the right one for an earlier version and stays untouched forever now
  that nothing writes to it, so the check returns the same clean answer whether the
  thing is installed or not.
- What it tells you to do instead: sweep the whole target tree for the artifact's own
  name, ask the running process what it actually has open and loaded, and read which
  candidate the runtime's resolution order really selects. A location you know about
  can confirm presence the moment the thing turns up in it, and can never establish
  absence — so an absence is only ever earned by a search, and the claim should name
  what was searched.
- The skill now fires on negative claims too. Asking it to check whether something is
  installed, applied, or registered used to reach nothing: its triggers were all
  claims that something was done, fixed, or passing.

## 2026-08-27 — Work now ends with a choice instead of a paragraph

- New skill, `offering-the-next-move`: a run closes by offering what to do next
  as a selectable set of options through your harness's question tool, rather
  than describing the possibilities in a summary paragraph and leaving you to
  turn them back into a decision.
- The options are harvested, not invented. It reads the acceptance line that was
  surrendered, the ruling that could have gone the other way, the scope pushed
  out on purpose, and the thing found in passing and never chased — then ranks
  them by one test: would you have thought of this without the run? The obvious
  next step ranks last, because you would have asked for it anyway. Expect the
  menu to name things you did not know were on the table.
- It refuses a few things on your behalf. No option for work that was already
  inside the agreed scope — that gets finished first rather than handed back as
  a question. No option it would argue against if you picked it. No two options
  that mean the same thing padded out to fill the widget. And one option always
  ends the work, with a real read on whether stopping there is defensible, so a
  menu can never be four flavors of more.
- A run that failed or stalled gets an offer too, and that is where it pays
  most: what blocked it, what would unblock it, and what is worth doing instead.
- `orchestrating-work-end-to-end` now runs it as the last beat of landing and of
  any investigation's report, and says to assemble the offer before the run
  record is torn down — the record is where most of the options come from.
- Nothing is listening on a headless or scheduled run, so there the same ranked
  list goes into the report instead of into a prompt nobody would see.
- Sharpened after watching fresh agents close the same finished task with and
  without it. The failure worth naming turned out not to be the obviously generic
  menu — it is the polite one: a wrap-up that reports every finding accurately and
  then tells you what it intends to do about them. *Tonight I'll rebuild the
  runner. That's my first job tomorrow.* All the information arrives and none of
  the decision does. The skill now says that outright — a closing sentence in the
  first person and the future tense is a decision that was never offered.
- The wrap-up you were already getting does not shrink to make room for the
  options. The report comes first and in full — what was done, the evidence, what
  is not done and why — and the offer sits underneath it. The test the skill sets
  itself: if you ignore the options entirely you should be no worse off than if
  none of this existed, so anything you would lose by skipping them was in the
  wrong place.
- The offer arrives through your harness's question tool, so a direction is one
  click rather than a sentence you have to type back. Options written into the
  message body are a fallback for harnesses that have no such tool, not a
  shortcut when one is sitting there.
- `orchestrating-work-end-to-end` now opens by checking that TBaguette itself is
  current, ahead of naming the track and ahead of any work. Nothing to do on your
  side: where the start-of-session check has already answered, it reads that
  answer instead of hitting the network again, it stays silent when you are up to
  date, and it never blocks a run — a library that cannot update is reported and
  the work carries on.
- Four more skills now hand off to that close-out, so it reaches you from where
  runs actually end rather than only from the orchestrator: landing a branch,
  writing up technical work, marking what you are confident about, and
  surrendering an acceptance line. The last of those is the one worth knowing —
  a criterion you asked for and did not get is the first thing the close reaches
  for, so it comes back to you as a decision rather than as a footnote in the
  report. Landing also now says to assemble the offer before cleanup deletes the
  plan and run record it draws on.

## 2026-08-25 — The orchestrator now owns the documents a run leaves behind

- `orchestrating-work-end-to-end` treated writing things down as a landing chore:
  `writing-durable-docs` and `writing-adrs` were reachable only from the final
  phase, while the design phase's own gate already required "a written spec" and
  said nothing about how to write one, where to put it, or who keeps it true. Both
  now sit in the design phase too, alongside `reading-specifications` for when a
  spec is the input rather than the output.
- It also distinguishes the four things a run produces, which get confused in one
  direction: the spec and the ADR outlive the branch and are maintained as though
  they were scratch, while the plan and the run ledger are scratch and get left in
  the repo for a year. Each now has a stated lifespan, and phase 8 has to decide
  what happens to the scaffolding rather than leaving it because nobody looked.
- The rule that costs most to miss: routing backward to the design phase leaves
  the spec describing an approach that lost. Leaving it is worse than never having
  written one, because the next reader gets a confident description of the wrong
  design with nothing marking it as dead. The skill now says the document changes
  in the same move that reopens the phase — and that the rejected approach and the
  reason for rejecting it are what a later reader most needs, not what to delete.

## 2026-08-25 — Two rules in one skill that cancelled each other out

- `configuration-management` carried a contradiction that produced a broken
  build if you followed it. One section tells you a switch needing to flip
  without a restart is a flag, not config. An earlier rule says **never read
  config at the call site** — and read together, those build a flag loaded once
  at boot, which cannot flip during the incident it exists for. Each rule is
  correct alone, which is why the pair survived review. The call-site rule now
  carries an explicit carve-out, keyed to something checkable: if the value must
  change without a restart, evaluating it at the call site is the mechanism, not
  a violation, and what gets validated at startup is the flag client's wiring
  rather than its value.
- The same skill's flag-expiry rule — *delete a flag older than one release
  cycle in one direction or the other* — is written for release flags and
  misfires badly on an operational kill switch, where deleting toward on
  reinstates the incident and deleting toward off removes the feature. It now
  says which flag type it is about, and that a kill switch takes a removal
  *condition* and a scheduled exercise instead of a date, because the arm you
  need at 3am is the arm that never runs.
- `authoring-a-new-skill` now warns about the gate that actually ambushes people
  when adding a skill: the library refuses a skill that no other skill points
  at, so a new one turns the suite red until some neighbour's "Not for:" line
  redirects to it — a skill worth adding is one some existing skill should be
  handing off to.

## 2026-08-25 — A ship step that told you to do work that no longer exists

- `authoring-a-new-skill` was still instructing anyone shipping a new skill to
  translate its description into every locale the site builds. The site has been
  English-only since 23 August, when `i18n/` was deleted and the locale list was
  cut to English alone — so that step described work that cannot be done and does
  not need doing. Removed. The same checklist named only `CATALOG.md` as the place
  a new skill gets filed, omitting the registry in `content_pipeline.py` that
  actually refuses to build without it; it now names both, and points at how to
  create a category when none of the existing ones fit.
- Ten more skills gained cross-references to the skills that own the other half
  of their problem, found by reading all 92 end to end rather than by sampling.
  The ones worth naming: `configuration-management`'s "this is really a flag"
  section now hands off to `feature-flagging` instead of restating its rules;
  `code-archaeology` and `atomic-commits` both discussed bisection at length
  without ever naming `bisecting-failures`; `explaining-technical-work` asked you
  to mark claims verified, inferred, or assumed without mentioning that
  `calibrating-confidence` defines those three tiers; and `deleting-code`'s
  proof-of-deadness step now says why it inverts the usual observability rule —
  you are trying to emit that nothing happened, and only a counter can show that.
- Across both of today's updates the library went from 308 cross-references to
  355, and from three skills that referenced nothing at all to none.

## 2026-08-25 — Two new categories, and a pointer that led nowhere

- The library is organised into **twelve categories instead of ten**, and eleven
  skills moved. **Planning and delegation** collects the five skills about turning
  an approved design into tasks and getting those tasks done —
  `structuring-an-implementation-plan`, `working-a-plan-task-by-task`,
  `delegating-tasks-with-review-gates`, `fanning-out-independent-work` and
  `routing-around-capability-gaps` — which were previously split between
  "Communicating" and "Environment and tooling" despite each one's own text
  defining its edges against the others. **Finishing and proving** collects the
  five that guard the finish line: `finishing-what-you-started`,
  `confirming-before-claiming-done`, `red-teaming-your-own-work`,
  `karen-and-the-manager` and `knowing-when-to-stop`. "Judgment and meta" had
  grown to fourteen skills spanning four unrelated activities; it is now ten.
  `reading-specifications` also moved out of "Reading code", where it was the one
  entry that is not about reading code, to sit beside `scoping-before-building`.
  Nothing about invoking a skill changes — the names, and every skill's own page
  URL, are exactly as they were.
- `tracing-data-flow` had been sending readers to a skill that does not exist.
  Its "Not for" line pointed at `systematic-debugging`, which is Superpowers'
  name for the thing this library calls `diagnosing-before-fixing`; it survived
  because it was written without backticks, so no search for skill names ever
  saw it. Fixed.
- Twelve skills gained cross-references to the skills that own the other half of
  their problem. `caching-strategy` and `rate-limiting-and-backpressure`
  previously mentioned **no other skill at all** — the two halves of the same
  overload incident, with nothing pointing between them. Also newly wired:
  `managing-scope-drift` to the ledger that makes silent narrowing visible,
  `steelmanning-alternatives` to the premises that expire underneath an old
  decision, `testing-the-untestable` to the question of whether a double is
  faithful once the seam is in the right place, and `data-migrations` to the
  fact that a migration is almost always a one-way door.

## 2026-08-25 — Eight lessons from real work, folded back in

- `instrumenting-for-observability` separates *did it run* from *did it have
  anything to work with*. Anything that learns, tunes, calibrates or ranks reads
  a ground-truth source, and when that source is empty the component does not
  fail — aggregates over an empty set are well defined, so sums are zero, fitness
  functions return values, counters advance, and it looks productive on every
  dashboard. Worse, it degrades to its prior, and priors are chosen to look
  neutral rather than absent, so the failure arrives as a confident middling
  number. Emit input volume and age as their own signals, carry the sample size
  to the point of use so a consumer can tell a measurement from a default, and
  let the component refuse below a floor instead of fitting noise.
- `drawing-boundaries` picks up the properties that boundaries quietly destroy.
  A rule like *run the cheapest checks first* gets written for one stage, and
  that stage obeys it — while the layer above does the reverse, because a module
  cannot observe an ordering only visible from outside it, and cannot enforce
  what it cannot observe. Every layer passes its own review and the composition
  stays broken. For gates the audit is mechanical: list each one, annotate cost
  and what it needs to know, and check the ordering is monotonic *across*
  boundaries. A gate that runs after the spend is not a gate, it is a receipt.
  The same inversion hides in retries that multiply, inner timeouts longer than
  outer ones, per-client rate limits that blow the aggregate, and TTLs in series.
- `diagnosing-before-fixing` adds a signal that arrives before the expensive one.
  Re-running a failing operation is only an experiment if something differed
  between the runs; when nothing did, the second attempt is a re-observation of
  the first. Byte-identical output across attempts is itself a finding — the
  failure is deterministic, so the cause is structural, and that is the moment to
  read the error rather than run it again. It sits in deliberate tension with the
  reproduction step, which wants identical results: same observation, opposite
  meaning, separated by asking what differed. The three-failed-fixes rule already
  in the skill charges three fixes before it fires; this costs a re-read.
- `writing-durable-docs` names the claims its own drift ladder cannot reach.
  Every rung of that ladder needs something to execute or generate, so a claim
  that something *is not* the case — no telemetry, no third-party calls, no
  runtime dependencies, stores no personal data — is stuck at silent rot
  permanently. Those are also the sentences readers lean on hardest, because they
  are promises rather than descriptions. And they break backwards: a negative
  claim goes false when something is added somewhere else entirely, in a diff
  with no reason to touch the doc. Invert the check — a test that fails when the
  forbidden thing appears — and treat a doc asserting behaviour as a call site of
  it. Plus: two copies of one document with no generation step is a fork.
- `designing-ci-pipelines` names why scheduled jobs rot unnoticed. Every other
  check in a pipeline has somebody waiting on it, so silence means no trouble. A
  scheduled job has no audience, and its success and its total non-existence
  produce the same silence — which is why one that has never run successfully
  looks identical to a healthy one on any dashboard showing last-run status. It
  surfaces as the absence of the thing the job maintained, after the deadline it
  protected. Two habits fix it: force one run by hand when the job lands, and
  report *age of last success* rather than status of the last run, since a job
  suspended in March is green on the second and four months stale on the first.
- `portable-shell-scripting` covers the mirror of a trap it already had. A `cd`
  inside `$(…)` cannot escape; a `cd` at the front of one command in a session
  that persists between commands re-roots every relative path used afterwards, by
  every later command, including ones written by someone who never saw it. The
  expensive part is not the breakage but that "no such file" and a grep matching
  nothing are exactly what a real absence looks like — so the result is a
  confident false negative that reads as a finding about the codebase. Prefer
  `git -C`-style path options, then absolute paths, then `(cd x && …)`; and print
  `pwd` before believing a surprising negative.
- `writing-the-failing-test-first` gains the collection case, which the scalar
  advice about hand-deriving expected values does not reach: an assertion of the
  form *for each item in the result, assert it is well-formed* quantifies over a
  set the code under test chose. Drop half the input and the survivors still
  pass; drop all of it and the loop never runs, so the test is at its most
  confident exactly when the code has failed hardest. Get one number from outside
  the unit — count the source, not the result — and when a loss is deliberate,
  state the expected number rather than deriving it.
- `threat-modeling` now asks a question that catches a whole class of bypass: a
  limit, quota or entitlement is a property of *state*, but it almost always gets
  implemented as a check on one *transition* — the interactive one someone had in
  mind. Seeding, import, migration, restore and sync reach the same state without
  passing it, usually with more authority and less scrutiny. Enumerate the
  producers, not the check. And ask whether anything re-examines the state after
  it exists, because a gate that runs only at creation cannot repair what arrived
  around it.

## 2026-08-25 — A check that would pass even if nothing worked

- Some things you are asked to confirm are not about the present at all: that a
  service comes back after a restart, that a cache rebuilds cold, that a project
  builds from a fresh clone, that a backup restores. The check to hand almost
  always measures the present instead — it is running, it answers, it is set to
  start — and those pass just as convincingly in the world where the requirement
  is completely unmet.
- One question tells the two apart: would this check still pass if the condition
  the requirement names had never once occurred? If yes, it is a proxy, and
  re-running it proves nothing further. Worth asking before calling anything done
  on a service, a cache, a scheduled job, or a backup.
- What settles it is inducing the condition once — actually restarting the host,
  clearing the cache, cloning into an empty directory. That is disruptive and
  often not yours to authorise, so it now says to treat it as a reversibility
  decision and get the owner's go-ahead first. When you cannot, the honest report
  is "configured but unverified", plus the test that would settle it — not the
  proxy dressed up as proof.
- Two things read a result better than any status string: a start timestamp
  beside a boot timestamp tells you *who* started something, and a restart
  counter separates coming up cleanly from crashing and being retried, which look
  identical from outside.
- Asking for something to be made to start on boot, survive a restart, or work
  from scratch now reaches `confirming-before-claiming-done` on its own.

## 2026-08-23 — Three skills sharpened

- `least-privilege-design` now covers being asked to justify a permission
  something already holds: check what actually calls it before defending it
  in the abstract.
- `confirming-before-claiming-done` closes the gap between “pushed” and
  “reached the user” — a deploy nothing has fetched yet is not evidence the
  change landed.
- `feature-flagging` says to pin the value of a flag that suspends a rule, so
  a test proves the suspension instead of inheriting whatever the environment
  happened to have set.

## 2026-08-22 — The spine the rest of the library hangs off

- New skill: `orchestrating-work-end-to-end`. Invoke it when a request will
  take more than one edit — it routes the work to a track, names the phase
  you are in, and says what evidence opens the next one. It is the skill that
  sequences the other ninety-one.
- `rate-limiting-and-backpressure`, `instrumenting-for-observability`,
  `deleting-code`, and `authoring-a-new-skill` each gained a case they
  previously got wrong.

## 2026-08-21 — When the model itself is the blocker

- New skill: `routing-around-capability-gaps` — what to do when the thing
  being asked for is outside what this model or harness can actually do,
  rather than quietly delivering something adjacent and calling it done.
- A conversation that was already open when you install will not pick up the
  new skills. Start a fresh one.

## 2026-08-20 — Finishing what you started

- New skill: `finishing-what-you-started`, for long runs and multi-part
  requests where stopping short would go unnoticed. Write the acceptance
  ledger to a file before starting, and re-measure every number at report
  time rather than quoting it from memory.
- The skill check is re-asserted every turn rather than only at the start of
  a session, which is where it used to quietly lapse in long conversations.
