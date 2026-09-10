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

Writing an entry is part of shipping a change here — see
CLAUDE.md. The shape is `## YYYY-MM-DD — Title` followed by `-` bullets, newest date first, and
`scripts/generate.py` refuses to build a site if that shape or that order
breaks. A bullet may wrap across lines; the continuation is joined back on.
Everything above the first `##` is preamble and is never rendered.

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

## 2026-09-09 — Ten checks that were passing because nothing could make them fail

- **A version check on stored data now gets told which way to point.** `schema-evolution`
  covers the guard almost everyone writes — accept anything at or below the current version
  — and why it is exactly backwards when the version moves because a field's *meaning*
  changed rather than because a field was added. Re-reading an old row with new code then
  applies the new meaning to the old number, through the one check written to stop that. It
  also covers the trap underneath: a version field that inherits a whole-record default
  reports the version of whatever is reading it, so every legacy row claims to be current
  and the guard has never once rejected anything.
- **A cache key is a promise to everyone else who computes it.** `caching-strategy` now
  covers keys derived from something local to one process — an identity hash, an address,
  an iteration order. They work everywhere you test them and differ on every other machine,
  which is fine for a private speedup and a correctness bug when two machines are supposed
  to agree. It also covers the hit rate that will not move no matter how the cache is tuned,
  because a router upstream is scattering identical requests across backends and the
  locality was destroyed before the cache ever saw them.
- **A nightly job that fails on a different missing secret every night.** `designing-ci-pipelines`
  covers the cost of discovering prerequisites one failure at a time when the cycle is a
  night or a week, and why the error never tells you there are five more behind it — it comes
  from whichever step touched the first one. The fix is a preflight that checks everything
  and reports every failure at once, including the difference between a credential that was
  never set and one that expired, which otherwise look identical at 3am.
- **Cited another document? You have not checked it.** `red-teaming-your-own-work` gains a
  seventh attack and a section on the one claim that reads as verified because of its shape.
  A reference looks like a link, so nobody opens it — including the person who wrote it. The
  sharper half is that *correcting* a vague citation into a precise one feels more rigorous
  while adding a second false claim about a file still nobody has read.
- **Filters that each work and together let nothing through.** `drawing-boundaries` covers
  chains of accept/reject rules — screens, detectors, vote tallies, stop conditions — where
  every rule is individually right and nothing has ever measured what the assembled chain
  admits. An over-strict chain and a genuinely quiet period produce the same empty output.
- **Matching on a word a human was meant to read.** `modeling-errors` already told whoever
  publishes an interface to emit a stable code rather than prose. It now covers the other
  end, where no such code exists and the only anchor on offer is a printed status word, a
  window title, or a generated class name — each wrong in a way your own machine cannot show
  you, since status words are localized and class names are hashed per build.
- **A tool result may be shorter than what the tool produced.** `calibrating-confidence`
  treats output observed in this session as the top tier of evidence, and that assumed it
  arrived whole. Harnesses cap results at limits they do not announce. This bites hardest on
  a negative claim, because truncation removes the tail: "I searched and found nothing else"
  is exactly the sentence a clipped result reliably produces.
- **A missing file gets quietly replaced by a similar one.** `calibrating-confidence` covers
  what a small, boring obstruction sets off: a 404, a file that is not there, a package that
  will not install. The natural move is to find the nearest thing that does work, answer about
  *that*, and report it under the original's name — with no step along the way that feels like
  fabrication. Name what you actually opened in the sentence that states the finding.
- **An allowlist of commands does not restrict what those commands mean.** `least-privilege-design`
  covers the approved, legitimate operation that runs someone else's payload because an
  unlisted operation changed an environment variable, a search path, or an alias a minute
  earlier. The audit log ends up being a list of things everybody agreed to, and the
  poisoning step looks harmless to any classifier because its whole effect is on what happens
  next.
- **An approval gate has a throughput.** `delegating-tasks-with-review-gates` covers what a
  gate actually spends — the attention of whoever answers it — and the part that inverts
  intuition: the better the delegate performs, the less each approval gets read, so the rare
  wrong one arrives at the moment of least scrutiny. With the checkable rule that if you
  cannot say what a *no* would change, it is a notification wearing a gate's interface.
- **A correct fix that changes nothing looks exactly like a wrong one.** `diagnosing-before-fixing`
  already covered a byte-identical result meaning the input never arrived. It now covers what
  happens when a path holds two blockers: you find and fix the first, the output is still
  identical, and the hypothesis you just confirmed gets retired. Enumerate every stage before
  changing any of them.
- **A refused command should name what it refused.** `routing-around-capability-gaps` and
  `tending-tbaguette` both cover the call that bundles several steps into one line and gets
  declined as a unit, naming none of them — so nobody can report which step is blocked.
  Finding out by re-running the pieces is not diagnosis. One call per step, so a refusal
  identifies its own subject.

## 2026-09-08 — Formidable aims past the brief by default

- **Ask for design work without saying which kind, and you now get the better version of the
  surface rather than the one you described.** `formidable` used to treat a bare request as
  ordinary design work and build to the level of the brief. It now runs a new `elevate` command
  instead: enhancement is the posture you get by default, not one you have to know to ask for.
  Name a job yourself — one fix, one screen, a build you want kept plain — and that job is still
  the whole brief, done to the same standard and no wider. Every existing verb behaves exactly
  as before when you call it.
- **It goes and looks at how the best products in your category solved this, before it designs
  anything.** The bar it works to is no longer the state of your project. It finds three to five
  of the best shipped products matching your domain, mode and stack, reads the real thing rather
  than a gallery thumbnail, and takes one specific move from each — a hierarchy decision, a type
  pairing, one motion moment. Moves your medium cannot express get discarded rather than
  approximated, and the report tells you which product each borrowing came from, so you can veto
  one. With no network it says it is working from memory instead of quietly pretending otherwise.
- **You no longer need a UI for it to work on.** It takes a built interface, a description of one
  you have only imagined, or nothing at all — in which case it works out what your project is
  for and proposes a surface, clearly labelled as a proposal, or tells you plainly that a parser
  or a daemon does not want one. When you have left the standard open, what comes back is
  expected to include something on that surface you would not have thought to specify. Anything
  that would spill onto a different surface gets offered to you instead of built.
- **Upgraded with Fable 5.1.**

## 2026-09-08 — Things that were not protecting what you thought they were

- **A credential should never travel through you.** `secrets-hygiene` gains the surface that
  was missing from its list of places a secret must never be — a conversation, a transcript,
  a model context — and the reason it is the worst entry on that list: it is re-sent verbatim
  every turn, fans out into session stores and request logs, and gets paraphrased into
  summaries that outlive the session, and unlike a log there is no masking step to fail, because there is no masking step. The rule that follows is short. When a secret has to reach a third party, send the
  *person* to the issuer and take back a reference. Careful handling does not help, because
  the recording is not something the handling does — it is what the medium is.
- **Text you pass along arrives in your voice.** `handling-untrusted-input` has always named a
  model's context as the one destination with no way to separate data from instructions. It now
  names the second one: the attention of whoever reads your output. A quoted line from a fetched
  page, a dependency's prose, a tool's error string, another agent's summary — none of it is
  marked, so it acquires your authorship by default. A relayed question reads as your question,
  and its answer authorizes something the reader never meant to authorize.
- **A resume restores your record, not the world.** `checkpointing-long-runs` covered memory
  fading across a boundary. The opposite failure is worse for being clean: the record crosses
  *intact*, and a verdict or approval re-attaches to whatever now occupies the slot it named.
  A reviewer who has since left, a token since expired, a workspace predating the patch that was
  tested. Nothing is corrupt; the halves simply never coexisted. The fix is that a recorded
  verdict names what it judged — a path and a hash, a commit — never the role it played.
- **An approval that only records that it happened is a bearer token.** In
  `delegating-tasks-with-review-gates`: when a gate is split across two exchanges, the carrier
  travels through the very party the gate constrains. Reduced to a flag, it clears the
  next one of those of any shape — a plan approved in one turn, executed in another after
  the plan moved. What an approval has to carry is a digest of what was approved: the
  actual paths, parameters, or diff, compared field by field at the point of action
  against what you are about to do, rather than against what the session now believes.
- **A guard applied everywhere but once.** `reviewing-code-deeply` now treats that shape as
  worth a blocking comment. The exemption is granted on a belief that one member is reliable —
  local, ours, the fast path — and it is exactly where the failure will not be caught, because
  nobody is watching a path that was exempted for being unremarkable. The comment to leave is
  not "why is this one different" but what the exemption saves, in units, and what it costs if
  the belief is wrong.
- **A suite green feature by feature, broken the moment two features meet.**
  `writing-the-failing-test-first` names why the loop produces this on its own: the red step is
  per-behaviour, so *n* red steps make *n* isolated paths, and refactor is the only step that
  ever builds what they should have shared. Two consequences that surprise people — more
  attempts make it worse rather than better, and adding more visible checks is not a reliable
  correction, because the new ones get optimized against too.
- **"This is impossible" gets likelier the longer you have been going.** `knowing-when-to-stop`
  adds the reading that has nothing to do with the problem: a long accumulated context makes a
  run give up well before it has exhausted what it could try. The tell is a stop that describes
  exhaustion rather than naming an observable — so *blocked* is the one verdict not to accept
  from inside the context that produced it.
- **A capability you keep available is charged on every decision, not on use.** `crouton` priced
  a tool in tokens. The larger bill is that the set of options is re-read before every choice you
  make, and selection degrades as it grows — which raises the bar from "must save more than it
  costs" to "must be worth making every unrelated decision slightly worse." What that argues for
  is a per-task loadout instead of a permanent one.
- **A declined command is reported, not worked around.** `tending-tbaguette` now says what to do
  when a permission layer refuses a git step rather than git failing: the account is right, the
  network is fine, the command simply never ran. Every declined step leaves an obvious way to get
  the same result by hand, and taking it lands the change while leaving the pull request open and
  its author uncredited.
- **A backup that lives inside the thing it protects is not a backup.**
  `deciding-reversibility` had a list of ways to turn a one-way door into a two-way one,
  and every item on it quietly assumed the mechanism that reopens the door still exists
  after you have gone through. It now says the part that was missing: name the thing that
  would perform the undo, then ask whether the action you are contemplating can reach it.
  Backups in the volume they protect, under the credential that deletes it; a kill switch
  served by the service it kills; a rollback needing the pipeline the bad deploy broke. A
  green restore drill does not test this, and a separate volume behind one credential is
  not separate.
- **An egress allowlist does not cover the resolver.** `least-privilege-design` treated
  restricting egress as one line. It now covers the channels an allowlist of hosts does
  not name — resolution first, which carries data out in subdomain labels and instructions
  back in the answers without connecting to anything on the list, and which stays open
  because a container that cannot resolve names looks broken. Time sync and crash
  reporting are the same shape, as is any allowlisted host a third party can read back.
  The rule underneath: a containment guarantee is tested by attempting to leave, not by
  reading the sentence that claims it.
- **The default that looked neutral to you lands at one end of somebody else's scale.**
  `instrumenting-for-observability` already warned that a prior gets chosen to look
  neutral. It now names the consumer's side of that: a count defaulted to zero renders as
  the most negative reading available, a cost counter reading the wrong stream reports
  *free*, and a capability flag defaulted to `false` to be safe deletes a branch rather
  than narrowing one. An extreme value is not the same as an implausible one, which is what
  makes it hard to see: *0 mentions* sits at the end of the scale and reads as an ordinary
  quiet week. Given two defaults, take the one that produces an answer somebody will query
  over the one that produces an answer somebody will act on.
- **The verification that launches your app runs it against your real profile.**
  `reproducible-environments` now covers the command that isolation gets skipped for,
  because wrapping it feels like weakening the proof. A real binary launched with no
  environment set is a real session — real store, real key, rows in a real audit trail. It also says what to check afterwards, since the obvious check is wrong twice
  over: compare size as well as mtime, and across the write-ahead journal as well as the
  database, and assert the run actually created something in the scratch directory,
  because a binary that failed to start passes every "nothing was touched" test.
- **A pinned test can survive a refactor and quietly stop watching.**
  `characterization-testing` verified a pin has teeth once, against the code as it stood.
  It now says what expires that: any change relocating *where* an effect is produced. The
  pin is untouched, green, and asserting over a stage the effect no longer reaches — and
  re-running the suite is zero evidence, because passing is the symptom. Green is the
  worse failure here; a pin that named an internal at least breaks loudly.
- **The scanner finding everyone has learned to dismiss is usually your own fixture.**
  `secrets-hygiene` gains the credibility-budget rule: one reliably dismissible finding
  spends the whole thing, and the likeliest source is test data written to look real
  precisely because the code under test parses credentials. Assemble the fixture at
  runtime from parts that match nothing, rather than adding an ignore rule — which is a
  control being switched off — or a malformed fixture, which stops testing the shape. The
  same law covers detectors you write: tolerance for false positives is set by how often
  a human has to look at one.
- **The project already turned your change down, and the code does not say so.**
  `orienting-in-unfamiliar-code` now covers what a repository never records — its
  direction, and everything it has refused. A change turned down for one of those reasons
  is *technically correct* — already open, superseded, against a decision the project
  settled a year ago, arrived through the wrong workflow — so nothing in review points at a
  line and says what is wrong with it. The bounded pass that avoids it sits inside the
  orientation time box rather than on top of it: closed pull requests searched by path, the
  tracker searched for your idea rather than your symptom, the governance docs, and whether
  somebody is already doing it.
- **Being told to get it to them is not permission to get it to them another way.**
  `tending-tbaguette` already said a declined command is reported rather than routed
  around. It now covers the version that is hard to obey: by the time a step is refused
  you are usually holding a yes for the *outcome*, and that yes makes the substitute read
  as compliance rather than circumvention. One question separates them, and it is about
  the route rather than the goal — would this have been the obvious way to do it if the
  declined command had never existed? A route that only became attractive at the moment
  of the refusal is the refusal being worked around. The approval is still good, unspent,
  waiting for a session where the command runs.

## 2026-09-07 — When the harness is what failed, not the code

- `grounding-test-doubles` has always been about a double that is too *permissive* —
  a fixture your parser already agrees with, a shape you never captured, green suite
  and ungreen reality. It now also covers the mirror image, which presents as a bug
  rather than as a fixture problem: a stand-in that is more *restrictive* than the
  real thing, because it can reach the interface but cannot reproduce the
  precondition a real caller arrives carrying.
- The tell is narrow and worth knowing: the failure is a permission or authorization
  error under automation, and the same action performed by hand succeeds. That pair
  means the harness got measured, not the feature — and the cost of missing it is an
  afternoon spent fixing code that was never broken.
- Where it concentrates: anything gated on *who is asking* rather than on what is
  asked — a genuine user gesture, a foregrounded window, a real session, an
  interactive terminal, a signed build, a device actually attached.

## 2026-09-07 — Controls you are not allowed to break to get unblocked

- An autonomous run that gets refused by the thing it is testing — a lockout, a rate limit,
  a guard that has latched — now has a rule for it. `bounding-autonomous-work` treats
  clearing that state, loosening the assertion, or passing a bypass flag as a door: the
  refusal is the behaviour under test producing output, and removing it does not restore
  access to the evidence, it destroys the evidence. The three arguments that show up here
  are named and answered — it is only a test environment, I will put it back afterwards, it
  is obviously a misconfiguration — and none of them is a licence. The section sits beside
  the existing one about not stopping in front of doors you never tried, and now says
  explicitly how the two differ: untried is not a door, broken open is not a pass.
- `delegating-tasks-with-review-gates` had four ways a subagent can finish and no way for it
  to be stopped. A delegate that hits a turn ceiling, an output-size limit, a timeout, or a
  crash comes back with no status and a final paragraph that reads like a summary, because
  it expected to finish. Routed as done, the task-scoped reviewer then reads a truncated
  diff, finds it internally consistent, and has nothing to notice the missing half with. The
  status table gains that row, and the rule is that your own dispatch record — not the
  delegate's closing prose — says whether all of it came back.
- A dry run, a simulation, or a paper mode that has been green for months may never have
  reached the thing you think it rehearses. `confirming-before-claiming-done` names the two
  places it diverges: a branch that fires so early nothing downstream has ever run, and
  consequences that differ enough that the two modes are not comparable — a gate that is
  tolerated in one and terminal in the other. The prescription is to put the branch at the
  last boundary that actually effects something, record every gate's outcome identically on
  both sides, and state what the safe mode never reaches.
- Benchmarks run through a router, a connection pool, a gateway, or a provider layer are
  measuring the routing as much as the code. `performance-profiling` adds this as its own
  trap because it does not behave like noise: it moves the value and holds it steady enough
  to look like a real effect, and interleaving A and B does not help, since the variable
  resamples on every call rather than drifting with time. Record what served each
  observation, pin it, then confirm afterwards that both arms actually sat on the same one —
  a pin expressed as a preference gets ignored under load, which is when you were measuring.
- `choosing-test-scope` now covers the second way a guard can be silently absent. The first
  was already there: nothing calls it. The second is that it is called on every request and
  cannot reach any verdict but the one it always gives — a rule carried into an environment
  where the shape it rejects is unreachable, credited ever since with protecting against
  something it structurally cannot see. A check whose output has never varied across a real
  population is unproven, not passing; sample the distribution of its verdicts rather than
  their value.
- Reporting a process as deliberately left running now carries a second obligation in
  `finishing-what-you-started`: it has to actually survive. A job started from a run is
  usually a child of that run's shell, writing into scratch space that exists because the
  session exists, so the disposition can be stated honestly and be false within the hour.
  Reparent it, write its output somewhere durable, and hand the reader that path.
- A backlog entry you keep re-reading has already been declined. `revalidating-decisions`
  extends its premise-decay treatment to the decision nobody made: deferring is free and
  deciding costs a justification, so a marginal item gets re-read, re-agreed to be marginal
  and re-queued by a careful reader every time. Write what would change the answer next to
  the item; a second deferral cites what changed; the third reading settles it. An item
  whose stated trigger has not fired is waiting rather than deferring and does not count.
- Adding a confirmation step to a dangerous operation can make it less safe, and
  `designing-for-idempotency` now says why. Where the thing carrying the operation cannot
  pause, the standard way to ask a question partway through is to abandon the call, ask, and
  re-send the whole original request with the answer attached — so every effect the operation
  produced *before* the question happens again, once per asking. One logical operation becomes
  several complete ones. The skill gives the three placements that survive it and says that
  picking one is part of adding the gate, not a follow-up to it.
- Anything installed between a tool and whoever acts on its output holds more power than the
  tool does. `least-privilege-design` adds the case: a proxy, hook, sanitizer, or formatter
  decides what was observed, while the producer's authentication, authorization and audit log
  all describe only what the producer *sent*. These layers get installed for cosmetic reasons
  and reviewed as formatting conveniences, and one that strips anything error-shaped can make
  a real failure invisible to the only party able to react. Govern them at the privilege of
  the decisions they steer, and prefer a transform that adds a field to one that overwrites
  it, so the original survives to be compared against.
- A version, source, or origin field that defaults to whatever the writing code is at that
  moment can never record *unknown*. `tracking-data-provenance` names the trap and its delayed
  fuse: while the current value is the first one, a record written without the field reads
  correctly and behaves correctly, so the bug ships dormant — and the day the value increments,
  every such record starts asserting the new value to the guard written specifically to catch
  records from before the change. The same section covers the compatibility check that only
  looks one way, which is half a guard when the version marks a change in meaning.
- Before reading a result for what it says about the system, read it against itself.
  `diagnosing-before-fixing` gains a third discriminator beside its existing two: a reading
  whose own parts contradict each other — a count that disagrees with its listing, a pass that
  disagrees with its log — is a proof the instrument is broken, not a surprising fact about the
  system. It matters because the neighbouring rule points the other way: self-contradiction
  arrives as a single deviation, which that rule says usually means a wrong hypothesis.
- A search that returned hits verifies one thing — that those bytes matched that pattern.
  `calibrating-confidence` now covers what happens next, when the count gets read as the
  answer to a question asked in words. Fourteen hits becomes "widely used"; a list of paths
  becomes a map of where the thing lives; and the hits may be comments, dead branches, a
  vendored copy, a fixture, or a different symbol sharing a substring. Open them, sample them
  and say so, or make the smaller claim that is actually true — a count reported as a finding
  is a proxy wearing the name of the answer.
- Rules that defer work to a job's "next run" are rules about the clock, not about the work.
  `automating-repetition` adds the failure: nothing guarantees the gap between runs, so when a
  schedule tightens or two runs land close together, run *n+1* arrives holding exactly the
  inputs run *n* declined to act on — and a rule written to force a decision after a real wait
  instead manufactures one on unchanged evidence. Write the deferral against the condition it
  is waiting on rather than against the count.
- `tending-tbaguette` picks up two rules from the same window. When a skill's description has
  no room left for a new trigger, that is evidence about *fit* and not only about space: a
  saturated description usually describes a saturated scope, so ask whether another skill is
  the lesson's general home before deciding which existing trigger has to lose. And its own
  deferral rule now carries the interval caveat above — two visits to the queue in one
  afternoon are one pass, not two, and an entry they both saw is still on its first deferral.
  The mirror-image failure gets named too — a recurring run re-deriving a question it
  already answered "no", because a negative finding was never written where the next run
  would look. `tending-tbaguette` picks up the same rule for its own candidate queue.
- Finding a string inside a compiled, minified, or packed artifact is not proof it shipped.
  `confirming-before-claiming-done` already said a miss there is no evidence of absence; it
  now says the same about a hit, which is the direction that gets believed, because it
  arrives as good news. Name what produced the match or find a second pattern that must
  co-occur with it.
- Three sections that existed but could not be routed to now have triggers.
  `writing-durable-docs` reaches the same document living in two places with nothing
  generating one from the other; `automating-repetition` reaches the one-shot bulk edit
  whose diff is too large for anyone to actually review; `choosing-test-scope` reaches a
  redaction, permission, validation or rate-limit step with a green unit suite and nothing
  proving it is wired into the path it guards.

## 2026-09-06 — Sections nobody could have been routed to

- A skill's `description:` line is the only part of it that is always loaded, which makes it
  the whole routing surface: a section the description never hints at is unreachable, and the
  answer sits in the file while nobody who has the question ever arrives. A pass over the
  library asked one thing of every section — if you had exactly this problem and said it out
  loud, would this skill have surfaced? — and this entry collects the ones that now do.
- `designing-ci-pipelines` had a whole section on the scheduled job that has never once
  run, and no way to reach it. That job is the one thing in a pipeline with nobody waiting
  on it, so success and total non-existence produce exactly the same nothing: merged before
  its credentials were armed, a cron expression that parses but never matches, disabled at
  the platform level, suspended because the repository went quiet — every one of them
  renders as blank rather than red, and the breakage surfaces as the backup that is not
  there, months later. The skill now loads when a scheduled, nightly, or cron job is added
  and nothing yet proves it has ever run, and it says to force one run when the job lands
  and to report *age of last success* instead of *status of last run*.
- `threat-modeling` says a limit is a property of *state*, not of one transition, and had no
  way to be reached by anyone holding that problem. A quota, an entitlement, a per-seat cap, a
  uniqueness rule — each gets implemented as a check on the single interactive path whoever
  wrote it had in mind, and every other route to the same state walks past it: import, sync,
  restore from backup, bulk first-run seeding, admin tooling, an undo that re-adds what was
  removed. Those paths typically run with more authority and less scrutiny than the one that
  got the check. The skill now loads for exactly that shape, and for its second half — a gate
  that runs only at creation cannot repair what arrived around it, so without a read-time
  check the rule degrades into an honour system the moment a second writer appears.
- `drawing-boundaries` explains why a knob stated per-module says nothing about the pipeline,
  and nobody holding that problem could get to it. Two layers that each "retry three times"
  make nine attempts and a thundering herd nobody designed; each stage having a sensible
  timeout is how an inner one ends up longer than the outer one and never fires; two correct
  caches in series make staleness their sum rather than the smaller. Every layer passes its
  own review, forever, while the composition stays broken — because a property no single
  component can see is exactly the kind a boundary destroys, and it needs an owner above the
  boundary or it has no owner at all. The sharp version now reachable: **a gate that runs
  after the spend is not a gate, it is a receipt.**
- `finishing-what-you-started` tracks deliverables, which quietly scopes *finished* to
  artifacts — files written, tests green, a branch landed. A run also starts **things**: a
  backgrounded build, a watcher, a dev server, a long benchmark, a subagent, a polling loop.
  None of those has a ledger line, every one of them survives the report saying the work is
  done, and they are cheap to forget for a structural reason — a background job's whole
  purpose is to stop demanding attention. The person who finds it is you, hours later,
  noticing something has been running since morning, which is not a report but a discovery.
  The skill now loads for a run closing over one, and the close-out has one more question,
  answered by looking rather than remembering: what did this run start that is still running?
  Each one gets a disposition — stopped, or deliberately left and *said so*, with how to stop
  it. Same rule as a surrendered criterion, applied to something with a PID.
- `naming-things` covers the rename where one string is doing two jobs — a display word a
  human reads and a token something resolves — and had no trigger for it. Renaming a product
  while its old spelling stays valid for users is the ordinary case, and the two roles are not
  separable by file: one page holds both, occasionally one sentence does. So partition by role
  and ask it of each *occurrence*, not each file: does a human read this, or does something
  resolve it? A search-and-replace is wrong on every occurrence of the second kind and wrong
  **silently**, because a config key that no longer matches anything is still a valid string.
  Then pin what you left — both spellings now sit in the same files, and whoever reads them
  next sees an inconsistency and tidies it. This is the unusual rename whose regression test
  guards against the change *after* it rather than against itself.
- `rate-limiting-and-backpressure` says a per-item deadline is only meaningful if the item's
  clock starts when the item does, and nothing in its description mentioned a deadline at all.
  Most timeout primitives fix theirs at construction, so building every wrapper up front — the
  natural shape of a fan-out — starts every clock at once, including for items queued behind
  their own siblings. Items several waves deep then spend their whole budget waiting for a slot
  and time out having done no work. The diagnostic now routes: every item times out while the
  same item on its own finishes comfortably, which is queue time inside the deadline rather
  than slow work. It is quieter than it looks, too — a timeout that fired before any work began
  is indistinguishable downstream from work that ran and returned something neutral, so
  "no agreement" among workers that never started is not disagreement.
- `checkpointing-long-runs` had a section saying a negative result has two halves and only one
  ever gets written, with no trigger for anyone about to write one. "I tried X and it didn't
  work" is half a finding, and on its own it is worse than nothing, because it reads as closing
  a door it did not close. Pinning the clock and seeing no change reads as *time is not
  involved*; what it establishes is that the clock **in this process** is not involved, while a
  cache, a database `now()` and a broker each keep their own. The skill now loads for that
  moment and gives the dead end a required four-part form — tried, ruled out, still open, and
  what the null rests on in trials against what base rate. Deprioritized is not eliminated, and
  the difference belongs in the file rather than in the head of whoever ran it.
- `delegating-tasks-with-review-gates` has a table assigning a model tier to each role in the
  loop — cheapest for an implementer transcribing a complete spec, most capable for the final
  whole-branch review, at least one tier above whatever got stuck when a fix loop stalls — and
  its description never mentioned models at all. Anyone asking whether a cheap model would do
  for this subagent went unrouted. It now loads for that question, and carries the two things
  the table exists to say: name the model explicitly on every dispatch, because an unspecified
  one quietly inherits the session's own and that is usually the most expensive available; and
  count turns, not just per-token price, since cheap models routinely take two or three times
  as many on multi-step work and can cost more overall than a mid-tier one would have.
- `performance-profiling` warns that reliable is not large, and had no trigger for anyone whose
  criterion was the defect. A significance test answers whether a difference is **consistent**,
  and consistency is not size: a variant better by one percent on every single trial has a tiny
  difference and a tinier spread, so the ratio is enormous and clears any threshold trivially.
  It compounds wherever the decision is automated — a regression gate, an alert threshold, a
  rollout promoting on a p-value — because every pass accretes another change that is real and
  pointless, and complexity is permanent while a one-percent win is not. The skill now loads
  before that adoption, and asks for a minimum effect size in the units of the thing being
  decided — milliseconds at p95, bytes, queries, dollars — with both required to clear.

## 2026-09-06 — When the check and the mistake share an assumption

- `portable-shell-scripting` already warned that a *successful* `cd` outlives the command
  that ran it, so every later relative path resolves somewhere nobody chose and a read
  comes back as a confident false negative. It now covers the same trap on the way in and
  on the way out. A `cd` that **fails** moves nothing and stops nothing, so
  `cd "$dir" || cd "$fallback"` has one intended outcome and two unintended ones that look
  identical afterwards — the fallback put you somewhere unrelated, or nothing moved at all.
  The chain guarantees you end up somewhere; nothing in it guarantees it is the right
  somewhere. Errexit will not save you, since both sit in an `||` chain, and a persistent
  agent or terminal session usually has no `set -e` running in the first place. Where the
  move is genuinely wanted, the floor is `cd "$dir" || exit 1`.
- The half of that worth the reading is what happens next, and it is about **writes**
  rather than reads. A wrong-root read gives a wrong answer that stays inside your own
  conclusions; a wrong-root write *succeeds* — file created, status 0, output identical to
  the run that did what you meant — and quietly changes a tree nobody is looking at. Then
  the check agrees with it, because the obvious check is to read the file back by the same
  relative path, and that resolves against the same wrong root. It confirms the write and
  can never say where it went, and it will go on confirming however many times it is run.
  Settling a claim about *location* takes something that does not share the assumption:
  `pwd`, the absolute path, a listing of the parent you actually named, or the destination
  tree's own status — checked in the tree you meant and in the neighbouring one you may
  have hit. The skill's description also mentions the working directory for the first time,
  so it now loads for the script that is about to write to a relative path rather than only
  for the one with quoting trouble.
- `diagnosing-before-fixing` warns you off the environmental cause because accepting one
  *ends* the investigation instead of directing it. That warning has a shadow, and the same
  section now carries it. When the thing has only ever been **seen** through something local
  — a preview server, a viewer, a staging copy, an emulator, a scratch export — the
  environment is not the lazy hypothesis to resist. It is the first one, because it sits
  between you and the object and it is the half that is not the deliverable. A local server
  that sends no charset renders every em dash of a perfectly valid document as mojibake; a
  missing header, a base path, a compression setting, a stale cache each do the same for
  their own class of defect, and all of them look like faults in the artifact because
  presentation is precisely what they change. Reported as seen, that is a defect filed
  against something that does not have it — and then fixed there, in an artifact that was
  already correct. The remedy is the section's own, pointed the other way: obtain the same
  artifact through the path its real consumers use and compare, because a difference between
  the two is a fact about the stand-in. The green-result half of this argument was already
  in `confirming-before-claiming-done`; a red result inherits that limit unchanged.
- `verifying-review-feedback` already told you a bot comments from what a diff shows it,
  which is less than what the repository knows. A **mechanical** check is a review comment
  too — produced by the narrowest reviewer you will ever get — and its output hides how
  narrow, so the skill now says so. A tool reports on the one mechanism it reads, an effect
  can usually be produced more than one way, and outside its model a tool that still emits a
  number reports a confident failure rather than an abstention. An accessibility check
  computing contrast from an element's text-colour property returned 1:1 for an element
  whose own colour was transparent and whose visible rendering came entirely from a stroke
  it never read; measured through the mechanism actually in use, near 10:1. A size budget
  measured before compression, or a permission check reading the declared manifest instead
  of the runtime grant, fails identically. **Precision is not scope**, and a number carries
  no trace of which properties were consulted. So a mechanical finding is a *candidate*, not
  a result — confirm the specific instance through the mechanism the artifact actually uses
  before it reaches a report or a fix, and when a check flags something that looks visibly
  fine, suspect the check's coverage before the artifact.
- `confirming-before-claiming-done` covers publishing in *The push is not the reach*:
  check from the audience's seat, not the seat that published, because auth and caching
  differ there. Both of its failures assume a push happened. The section now carries the
  case where none did. Where the deliverable has a canonical source and copies kept in step
  by hand — a document mirrored onto a site, a notice repeated in a store listing, a policy
  filed with a registrar — **"fixed" is a claim about the source and nothing else.** The
  edit is real, the review is real, the ticket closes, and every published copy goes on
  serving the old text, because no act of publishing was ever part of the fix. Nothing
  detects it: no check fails, the source reads correctly, and the record says resolved. **A
  record saying resolved is evidence about an intent, not about a deployed state.**
- And *which* copies is not a question memory can answer. The number you can name is the
  number you knew about last time you looked, which makes it a floor rather than a count,
  and the copy nobody remembers is necessarily the copy nobody updates. So enumerate from
  the side that does the serving — what the host publishes, what the registry or store
  lists, what a search for the document's own title returns — then fetch each one's bytes.
  The skill's own *A look is not a search* was already this move, written for claims of
  absence; it holds unchanged for a claim that something has been fixed, and now says so.
  `writing-durable-docs` owns the repair: stop hand-mirroring, generate or diff in CI.

## 2026-09-06 — Two places the separation you rely on quietly stops holding

- `handling-untrusted-input` teaches one move: find the destination, then use the
  mechanism that keeps data and code apart there — a prepared statement, an argument
  vector, a contextual auto-escaper. Its table now has a row for the destination where
  that move has nowhere to land: **a model's context**. A fetched page, an issue body, a
  file's contents, an agent's report, and most easily missed a tool's own *description*
  all arrive on the same channel the instructions came in on, and nothing downstream can
  separate them. That is prompt injection, and until now the library described it three
  times without ever using the words, so nobody looking for it could find it. The new
  section says the two things that follow. Delimiters are escaping, and escaping is the
  fallback this skill already says fails — here with no correct nesting to fall back on,
  because the reader is probabilistic and has no parser to be right about; use them, do
  not count them. And the payload can land before you invoke anything, because a tool
  description is read at discovery time — which moves the check to connection time and
  makes the size of your connected set part of the exposure. The control that is left is
  structural: exfiltration needs untrusted content, something worth taking, and a route
  outward, and you remove one of the three. With the caveat that an egress cut is not a
  general answer, since injected text that makes a run destroy or spend something locally
  never needed a route out.
- `checkpointing-long-runs` says a compaction strips the *tier* off everything crossing
  it — verified, inferred and assumed all arrive in the same fluent voice — and tells you
  to re-check what is checkable and downgrade what is not. That rule is right for a claim
  and wrong for an obligation. A prohibition, a stop condition, a "check with me before
  X" loses its provenance the same way, but downgrading one to *assumed* is not caution,
  it is the failure: a constraint held at low confidence has already stopped
  constraining. There is no honest tier for an obligation — it is in force and gets
  re-read from wherever it was written, or it is gone and the run is doing something
  nobody authorized. Pre-committed stop conditions are the case that matters most, and
  the likeliest to come back as a general sentence about being careful, which reads like
  survival and is not.

## 2026-09-06 — A green check can be green in the wrong place

- `confirming-before-claiming-done` covered evidence going stale when code moves.
  It now also covers evidence that never applied where the claim does. The run was
  fresh, complete and read, and it happened on an emulator rather than the device,
  a container rather than the host, a staging tenant rather than production — and
  that failure is quieter than staleness, because nothing about the run looks
  weakened. A stand-in is not a smaller version of the real thing; it is the real
  thing minus a set of services it never lists, so everything that does not need
  what is absent passes perfectly. The library already made that argument for
  whoever *chose* the stand-in to get unblocked. It now makes it for the far more
  common reader who simply inherited one as the ordinary place work runs, and who
  therefore never made a decision they could think back to. An acceptance line
  closed on a stand-in gets marked with where it was proven rather than with done.
- `using-tbaguette` said a compaction rewrites your context and you cannot tell
  from the inside what survived. Half of that is now known, and the known half is
  worse than the general warning. What comes back is the body of each skill you
  actually invoked; what does not come back is the listing of everything you did
  not. So the loss is not even — it is shaped exactly like whatever you were
  already doing. A compacted run wakes up feeling oriented, holding a menu that has
  quietly narrowed, and every relevance check it runs against that menu comes back
  honestly empty. Re-checking your judgment does not fix a shrunken list; read
  `CATALOG.md`, which ships with the plugin and which a compaction cannot shorten.
- `redacting-sensitive-output` listed the places redaction gets missed and did not
  list the most automatic one: your language's own whole-value formatter. A derived
  `Debug`, a default `toString`, a `__repr__`, a struct-to-JSON call with no field
  list — each renders every field, including the one added last week, so the leak
  is created by an edit nowhere near any logging code. Nobody touched a call site;
  the type grew a field and every existing log line that formats it started
  emitting it. It is also the gap an allowlisting logger does not close on its own,
  because the rendered value arrives as one already-formatted string inside an
  allowlisted field.
- `tending-tbaguette` now has a rule for the case where nothing is watching. Its
  advice to stay silent about an empty queue was written for a reader mid-
  conversation, where a report about nothing is noise. A pass started by a timer or
  a delegation has nothing to interrupt and its report is its only output, so there
  the empty result *is* the report. And a run whose stated purpose is improvement,
  finding nothing wrong, will go looking until it finds something — which in this
  skill's case becomes a pull request a maintainer has to read and turn down.
- If you install TBaguette on Gemini CLI, `PORTING.md` was recommending it as the
  strongest integration of the lot without saying that Gemini CLI stopped serving
  Google AI Pro, Ultra and free-tier requests on 2026-06-18, or that Google has
  announced its transition to Antigravity CLI. The mechanism the recommendation
  rests on is genuinely still current and the repository is still maintained for
  Code Assist Standard and Enterprise — but most readers on a consumer plan cannot
  run it. The row and the audit note now say so, name what the successor's manifest
  actually is, and state plainly that no Antigravity manifest is shipped yet.

## 2026-09-06 — What "compatible" asks of a component that also writes

- `schema-evolution` told you to design readers that ignore unknown fields. That
  is right up until the reader also saves. A settings screen, an editor, a config
  rewriter or an admin tool loads a record, changes one part of it and stores the
  whole thing again — and "ignore" there does not mean tolerate, it means delete
  on the next save. The skill now separates tolerating an unknown field from
  preserving one, says why the loss is invisible (the read, the edit, the write
  and the validation all succeed, so nothing logs anything), and gives you the
  question to put to any writer: does it build its payload from the record it
  loaded, or from the fields it happens to render? The second one is a deleter.
- `auditing-dependencies` already covered the dependency whose payload is prose —
  an agent skill, a plugin, a tool server. Its advice ended at pin to a commit and
  review the diff when it moves, which cannot reach the half of that class you
  never fetch. A connected provider hands its instructions over at connect time
  and is free to hand over different ones tomorrow, with no commit to pin and
  nothing local that changed. The skill now says so, and adds the part that
  catches people out: that text is not scoped to the provider that sent it, so
  something you barely use can change how its reader handles the one you depend
  on, leaving no trace in the log of which tools ran.
- `tending-tbaguette` picks up the check that found both of the above. When your
  search for prior coverage lands on a passage that seems to cover the candidate,
  ask whether it covers the candidate's *actor* — a rule written for something
  that reads can be exactly wrong for something that reads and then writes back,
  and from a search those two results look identical. It also stops reading an
  empty commit body as a sweep that failed to reach its evidence.

## 2026-09-06 — Hermes' security scan stops blocking the install

- Yesterday's note said TBaguette installs on Hermes now. That was half true, and
  the missing half was another wall. Hermes security-scans a plugin before
  installing it, and on this library it returned a `dangerous` verdict — which is a
  hard block, not a warning: `--force` does not override it, and the only way past
  was to turn your own scanner off. The install still failed, just later and for a
  different reason than the day before.
- It now comes back `caution`, which is a door rather than a wall: at a terminal you
  see what was flagged and decide, and from a tool with no terminal to answer in you
  get a block you can act on instead of a dead end. The prompt on the site says so,
  and tells your agent to hand that decision to you rather than forcing past it.
- Most of what the scan flags is fair and stays flagged. A hundred skills about
  secrets, untrusted input and shell scripting read, to a pattern matcher, like the
  things they teach you to watch for, and no version of this library will ever come
  back clean. What changed is the six findings that were scored as *critical* — all
  of them the same false positive, where the scanner sees a filename rendered in
  code formatting on this site and reads the closing bracket of that HTML tag as a
  shell redirect into the file.
- Which is why AGENTS.md and CLAUDE.md are now written as plain prose in the handful
  of skills that mention them, rather than in code formatting like every other
  filename. It reads as an oversight and it is not one — it is the whole difference
  between installable and not on that harness, and a test holds the line now so a
  later edit cannot quietly put it back.

## 2026-09-05 — Hermes Agent installs, for the first time

- If you use Hermes Agent, TBaguette has never actually worked there. The install
  command the site publishes failed outright — `plugin.json name does not satisfy
  v1 constraints`, nothing installed — and it had been failing since the day
  Hermes was listed as supported. That error is gone — though a second wall
  behind it, Hermes' own security scan, only came down the next day; see the
  entry above. If you tried it and gave up, nothing you did was wrong.
- The Hermes install command has changed shape: it is now
  `hermes plugins install LeSplooch/tbaguette-skills --enable`, and the flag is
  not decoration. Hermes only offers to enable a plugin when it has a terminal to
  ask in, so an agent running the install through its shell tool would otherwise
  leave you with a plugin that is installed, disabled, and completely silent. The
  plugin then takes effect on `hermes gateway restart`, not on your next session.
  The prompt on the site says all of this now, so pasting it is enough.
- On Hermes, TBaguette now reminds itself to check for a relevant skill on every
  turn, rather than only at the start of a session. That is the same re-assertion
  Claude Code, Codex, Cursor and Copilot have had for a while, and it is the
  difference between a long session that reaches for a skill and one that
  remembers it existed an hour ago.
- Also on Hermes: what you see at session start is a shorter, straighter opening
  than it was designed to be. Hermes puts a size limit on what a plugin may
  inject, and the old bootstrap sailed past it and got silently cut to its first
  and last few lines — while still insisting the skill was fully loaded and
  telling the agent not to load it again. It now fits by design, says plainly
  which parts it left out and why, and points at `skill_view` for the rest.

## 2026-09-05 — A skill that questions the request instead of serving it

- New skill: `clairvoyance`. Every other skill in this library points inward at the
  request — reproduce it, scope it, plan it, prove it, land it — so a run that
  follows all of them produces an excellent implementation of whatever arrived in
  the first message, and has no way to notice that the first message was the wrong
  thing to build. This is the one that looks outward. It names the frame a request
  arrives inside, the five places that frame comes from — the requester's own
  vocabulary, the first file you opened, the codebase's existing pattern, your own
  previous turn, and any report you did not produce yourself — and seven directions
  to look, each with a tell that says which one this situation calls for, so it is
  a diagnostic rather than a brainstorm.

- What it will not do is expand your work. Everything it finds routes somewhere
  before anything acts on it: back to the design gate, into the acceptance ledger,
  into the run record as an option offered at close, or to a discard with the reason
  written down. Seeing more is free; doing more is not, and `managing-scope-drift`
  still wins. Two harder limits come with it — a sweep is never a reason something
  did not ship, and a run with nobody present may notice a reframe but may not
  approve one for itself.

- `orchestrating-work-end-to-end` now gives it a seat in all seven tracks, and the
  seat differs in each. On a build it runs between framing and design, which is the
  last moment a reframe costs a paragraph instead of the implementation. On a
  diagnosis it waits until the third fix has not held, then goes after the
  reproduction rather than the patch. On a review it is the pass that finds what the
  diff does not contain. During an incident it is explicitly barred until the harm
  has stopped.

- Six existing skills now hand off to it at the point where each one runs out of
  road. `steelmanning-alternatives` when three generated options come back sharing
  one shape, which means the constraint is upstream of the generation.
  `diagnosing-before-fixing` when three correct fixes did not hold, which is one
  wrong model applied three times rather than three bad patches. `managing-scope-drift`
  gained the sentence its readers needed most: noticing is not drifting, and here is
  where the noticing goes.

- The skill was drafted, then swept with its own method before shipping, and the
  record of that is in the skill rather than in a commit message: which of the seven
  directions found something, what each finding changed, and what was discarded and
  why. Six of seven yielded — including the one that caught the skill's own text
  saying "the frame" in the singular after listing four separate sources for it.
## 2026-09-05 — Three ways a green check is measuring the wrong thing

- `writing-the-failing-test-first` now covers the change that has no behavior to
  assert. An optimization, a cache, a batch, a swap to a cheaper backend — the
  point is that the answer stays the same, so every test you write passes before
  the change and after it, and the loop looks inapplicable. It is not: the change
  did move something, and it was a cost. Assert that instead — elapsed time,
  queries, calls, allocations — and it goes red first like anything else. The skill
  carries the case that shows why it matters, where a fast path returned the right
  value and passed while still paying the exact cost it was written to avoid,
  because the slow dependency sat one call further down in code the change never
  touched. Two conditions make such an assertion usable rather than flaky, and the
  skill names both; its description now fires on a change made for cost rather
  than behavior.

- `resolving-merge-conflicts` names a resolution that is textually clean, builds
  green, passes its tests, and silently un-publishes work that had already shipped.
  Any repository that commits its own build output — a rendered site, a generated
  client, a compiled schema — conflicts in those files on every integration, and
  both sides look equally authoritative because each agrees with its own branch's
  source. Take your side and you have just reverted everything that landed while
  you waited. The rule is that a generated file has no side to take: settle the
  source, then regenerate from the merged tree. The skill also says what a
  `.gitattributes` entry can and cannot do about it, and why a long-lived branch
  costs more in such a repository than the usual advice implies.

- `instrumenting-for-observability` extends its case on components that run
  without input to the harder one: a rule that *decides*. A gate treating missing
  data as disqualifying is sound in isolation, but if one of its inputs has no
  source where it was deployed it rejects everything, forever, for absence rather
  than for any measurement — and from outside that is indistinguishable from a
  strict rule during a quiet spell, with nothing logged and nothing to
  investigate. The fix is one field: attribute each rejection to the clause that
  made it, and never sum "failed the check" with "had nothing to check." The skill
  also says why the alarm on that has to be written narrowly, or the gate's
  legitimate strict spells will train everyone to dismiss it.

- `using-tbaguette` adds the third moment its own discipline goes stale, after
  "before the response" and "once the response has run long." A compaction rewrites
  the conversation underneath you, and what survived it is a decision the harness
  made rather than one the run made — so the first response afterward owes a fresh
  check, because "I already checked" now refers to a context that no longer exists.
  It also points at `/skill-doctor` where it previously named only the general
  diagnostics, since that one reports both halves of what the listing-budget
  decision actually needs: what each skill costs and how often it gets reached for.

- `tending-tbaguette` turns its own strongest rule into a procedure. It already
  said that a lesson appearing in two unrelated projects is the best evidence the
  bar can get; it now says how to find one, which is to sweep every project's
  commit subject lines into a single list first and group them by shape before
  opening anything. Reading one commit at a time can never see a repeat — each
  lesson arrives alone and gets judged alone. It also answers what to do when a
  contribution sits: re-merge, keep both update notes, and settle the generated
  half by regenerating rather than by choosing a side, since a waiting branch here
  does not merely fall behind.

## 2026-09-05 — What a result that found nothing is allowed to claim

- `red-teaming-your-own-work` now covers the outcome that gets no scrutiny at all:
  the search that came back empty. A null does not say no improvement exists — it
  says the search could not separate one from noise at the sample size it ran, and
  until the smallest detectable effect is measured, "no effect" and "no effect
  larger than X" are the same sentence. The skill says how to get that number and
  what it does when you have it, which is not overturn one result but re-scope
  every null the search ever produced, downward and at once. Its description now
  fires on a sweep, benchmark, or experiment about to be reported as nothing being
  there.

- `confirming-before-claiming-done` names a way a fix gets believed on evidence
  that was never about the product. The thing that showed the problem cleared was
  written for the investigation — a script, a probe, a one-off runner — and it
  imports the product's real code, which is what makes it convincing. What it
  proves is that the data exists and that some arrangement of the parts gets the
  right answer, not that the arrangement a user's action goes through is that one.
  Two tells that the check is owed, and what to do about a helper only the harness
  owns.

- `calibrating-confidence` covers the field with no data source of its own that
  gets mapped onto a named concept anyway. The trouble is not the correlation, it
  is that the name travels and the caveat stays behind: every message and screen
  naming the concept then makes the original claim with the stand-in's evidence
  under it. The check is to read what the system will say when the proxy fires,
  and the counterintuitive answer is that an admitted gap beats a borrowed name —
  a rule that abstains says something true, and one wearing the wrong name says
  something false.

- `choosing-test-scope` follows its own advice about exclusion lists one step
  further, because that advice turns out not to be enough. Requiring an entry's
  reason to name a real mechanism makes it well-formed; it does not make it true,
  and the two come apart the moment the named component changes. The fix is to
  stop storing the reason as prose: an exclusion is a claim that perturbing
  something cannot change the outcome, so store the perturbation and let the suite
  apply it. A stale reason becomes a red test instead of a paragraph nobody
  re-reads.

- `bounding-autonomous-work` tightens the probe it recently started encouraging.
  It already said the cheap experiment has to be the read-only half; it did not
  say you can be wrong about which half a command is on. Running an unfamiliar
  binary with a version flag reads as a question and, on a program that does not
  parse that flag, is simply a start — the real application, against whatever real
  state it finds. An invocation is an execution until the thing invoked is known
  to treat it otherwise, and this is not a new reason to defer to a human: it asks
  for a different experiment, not a line in the report.

- `tending-tbaguette` says what the strongest evidence for a lesson actually is,
  and it is not how good the lesson sounds. Agnosticism cannot be inspected for —
  from inside one project every lesson looks general, because the features you
  cannot see are the project's own. The same shape arriving in two unrelated
  codebases demonstrates the property the bar is asking about. Since a contributor
  usually has exactly one instance, the skill asks you to say which you have:
  "twice, in unrelated stacks" and "once, and here is why I think it generalizes"
  are both fine, and they ask a reviewer for different things.

## 2026-09-04 — When a check comes back clean without having looked

- `crouton` sharpens what a bounded read costs you. Its read rules tell you to cap
  what a command returns; what they did not say is that a cap does not merely omit,
  it answers — a count taken off `head -80` is the count of eighty lines, and it
  arrives looking exactly like the file's. Every count, every absence and every
  "last one" read off a capped command is a claim about the cap until something
  unbounded confirms it. Cap what you are reading; never cap what you are counting.
- `red-teaming-your-own-work` adds the attack to run on a measurement that has
  just confirmed something you are about to act on. A held-out check answers
  whether one *point* agrees, and cannot separate a real effect from a noisy peak
  a lucky point sat near — pairing, a stated noise floor and held-out samples each
  address a different threat, and none of them tests whether the shape reproduces.
  Re-run the whole sweep on one further untouched sample: a peak whose neighbours
  do not also beat the incumbent is a spike, and an incumbent that wins on the
  fresh curve means the original selection was noise. Worth running because it
  gives opposite answers on different results, not because it is cautious.
- `using-tbaguette` told you that every skill's trigger description is already in
  front of you, and at this library's size that can be false without anything
  saying so. The listing has a character budget; when it overflows, names all
  survive and **descriptions** get dropped, starting with the skills you invoke
  least — so the triggers disappear from exactly the skills you were relying on
  the listing to surface, and a check for a covering skill comes back empty with
  full confidence. The skill now names the tell, points at `CATALOG.md`, and says
  what to do where the harness exposes the budget as a setting.
- `routing-around-capability-gaps` extends "discovery is a sweep, never a
  recollection" down one level, to the tool inventory in your own context. A
  harness can defer its tools — callable, but with no schema loaded until
  something fetches one — so a tool can be fully available and absent from the
  list you just read. Query the harness's index by name and by keyword before
  writing "nothing here can do X", and note that being right by accident leaves
  no mark to correct the method next time.
- `bounding-autonomous-work` now says how to draw an attempt budget so it fires
  on the failure it was meant to catch. Counting attempts assumes each one is an
  independent try at the same problem; compilers, type checkers, linters and
  schema validators instead enumerate the remaining work in waves, so each
  "failure" names a different site and the count measures the size of the change
  rather than the futility of the approach. Run the command that enumerates all
  the work before counting, and treat a budget that can only be obeyed by halting
  on a half-applied change as mis-drawn.
- `checkpointing-long-runs` adds the background process started before a session
  boundary and never seen to end. The transcript records that it began and has no
  way to record an end that happened outside it, so "finished", "was killed" and
  "still running" all arrive as the same silence. Resolve it from what the
  process was writing to — the file, its size, the mtime, the lock, the pid —
  because the durable state is the artifact and the transcript only recorded the
  intention to produce one.
- `tending-tbaguette` marks where its catch-up sweep stops working. Reading back
  through the conversation you are in is cheap and effective; recovering capture
  across conversations that already ended is not, because a lesson has no
  consistent surface form and a keyword sweep over finished sessions comes back
  nearly empty. The queue file is the only durable record that a moment happened.
  It also says to check for the harness's question tool rather than reading its
  absence off a list, since that gate is the one an unattended run may not
  substitute.
- `modeling-errors` says which default to reach for when a layer genuinely cannot
  propagate a failure and has to put something in the slot. Substituting a default
  is already named as a defect; what the skill did not say is why some of those
  survive for months and others get caught the same afternoon. The ones that
  survive substitute the value a healthy system produces — the empty collection
  where the load failed, the zero where the read never happened, the `false` where
  the check could not run — so the wrong answer reads as good news and nobody
  investigates. Put in something no one could mistake for health, and carry the
  failure beside it.
- `tending-tbaguette` says what to do with the answer its own coverage check keeps
  returning. Before drafting a candidate you are told to search whether some skill
  already says it; what was missing is that on a library this size, "yes, and
  better" is the *ordinary* result. A session that hits coverage three or four
  times in a row starts reading its own filter as failure and goes hunting for
  something it can land instead — which is how the bar gets lowered by someone who
  never decided to lower it. Four candidates dropped for coverage is a pass that
  worked.

## 2026-09-03 — Tidying a shared tree is worse than sweeping it

- `atomic-commits` already said not to broad-stage a checkout you do not solely
  own: a wide `git add` commits someone else's in-progress work under your
  message. It now covers the move people actually reach for instead — clearing
  the tree first with `git stash -u`, `git clean -fd`, or a `restore`.
- That is worse, because it removes the work rather than mis-attributing it and
  leaves no commit for its author to recover it from. And it is tempting for the
  opposite of the usual reason: stashing is the documented tidy step, so a rule
  that only warns against the careless option never reaches the person reaching
  for the careful one.
- It is also the quietest failure in the section. A stash shows up in no view
  either party normally checks, so nobody notices until the author goes looking
  for work they assumed was still there. Ask the owner instead; where nobody
  answers, leave the edit alone and enumerate your own paths around it.

## 2026-09-03 — Checking the artifact is not enough if the artifact was already there

- `confirming-before-claiming-done` already said that when a pipeline can swallow
  a command's exit status, you should verify the artifact rather than the
  pipeline. That remedy was incomplete, and the new section says how: an artifact
  is usually already there from last time, so presence proves the step ran at
  some point in its history — a claim nobody was making.
- Two failures, two different checks. A **stale** artifact needs freshness — a
  modification time later than the change, or an identifier the run stamped in.
  Its tell fires before you think to look: a verification that finished faster
  than the work it claims to have done.
- An artifact that is fresh but **built from the wrong source** is the one a
  digest cannot see. A hash answers whether the bytes arrived intact and is
  silent on whether they are the bytes you meant. That needs a positive probe —
  search for content the change itself introduced — built from long, distinctive
  content, because short strings get folded into surrounding code and a probe
  made of them fails open.
- The skill's `description:` gained a trigger for it and lost a `Covers` clause
  that restated one the `Use when` half already made.

## 2026-09-03 — Ask the other author before you land their work

- `landing-a-finished-branch` now covers the case where what you are landing
  carries somebody else's commits. Its existing checks all ask whether *your*
  change is finished; none of them reach whether theirs is, and version control
  cannot answer it — a commit means committed, which is much weaker than ready.
- The gap is widest where it matters most: code that compiles, passes its suite,
  and has never once been executed on the platform it targets is indistinguishable
  in the log from code that has run in production for a week.
- So ask, even when you are confident of the answer. An author re-reading their
  own work under *is this safe to release* finds things that reading it under
  *is this correct* did not.
- And their answer authorizes their code being included, never your release —
  two agents agreeing is not approval when each answers to a different person.

## 2026-09-03 — Six corrections to crouton, found by reading it instead of the diff

- **It claimed the read rules cost nothing, and that was false.** "Every one of
  them returns the same information for less" is true of two of the six — the
  ones about not re-reading. A range is not the file, an outline is not the
  body, a capped command is not its whole output. The section now says which
  two are free and what the other four actually trade, which also settles a
  contradiction with the limit added to it earlier the same day.
- **"Cap what a command can return" would have hidden your test failures.**
  Test runners print failures last, so `| head -50` and `-q` truncate exactly
  what you ran the command for, and the re-run costs more than the cap saved.
  The rule now applies only where you already know the shape of the answer, and
  says never to cap the first look at a failure.
- **"Locate, then read the range" now has a size threshold.** Below a couple of
  hundred lines, two tool calls cost more than the file they avoid — the rule
  was losing money by the skill's own arithmetic.
- **Not re-reading a file you just edited assumed an edit tool.** A shell
  `sed -i` whose pattern matches nothing exits 0 and changes nothing. Confirm
  that one with a `grep` for the new text, which the rule now says.
- **The register direction sentence said the opposite of what it meant.**
  "Move further toward it" read as *tighten* on hard content — the exact red
  flag listed further down the same file. It says loosen now.
- **The numbers name their sample.** 95% of reads pulling the whole file is 307
  reads across 25 sessions on one machine, and it says so rather than travelling
  as a general fact about agents. A worked example that multiplied 13k by three
  and got 30k has been corrected.
- **Read discipline is no longer described as a mode the user can end.** The
  register is session state someone asked for; the read rules are not, are on
  in every session, and are nobody's to switch off. One paragraph was governing
  both.

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
- **It also says where reading less costs you.** The read rules are free in the
  sense that they return the same information for less — but not opening files
  is also how you stop noticing what you weren't looking for, and that lands on
  discovery work: orientation, review, an audit. Compress hardest when the
  question is known; spend the reads when the job is finding the question.
- **And it now names the first thing to check: whether your harness already
  does it.** Claude Code declines a repeated identical read of an unchanged file
  on its own. Anything you build to catch that case is overhead on every read it
  doesn't fire on — which is worth knowing before you write it, not after.
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
