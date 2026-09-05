# The seven directions, in full

Each entry: what the direction looks at, the questions that open it, the tell
that selects it, what it characteristically yields, and the specific way it
fails. The failure line is the important one — every direction here has a
degenerate form that feels exactly like the real thing from the inside.

Read the direction the tells selected. Reading all seven is the campaign-sized
sweep and it is rarely the right size.

---

## Behind — the goal the request is already a solution to

**Looks at:** the problem the requester solved in their head before writing to
you. A request is almost never a problem statement. It is a proposed remedy,
with the problem compiled out of it.

**Questions:**
- What would be true if this were done? And what would that make possible?
- What happens if it is not done — what is the actual cost, to whom, when?
- Why now? What changed? A request that could have been made a year ago and was
  not is a request with a recent cause that is not in the sentence.
- Is there a cheaper thing that produces the same *outcome* rather than the same
  *mechanism*?

**Tell:** the request names a mechanism rather than an outcome. "Add caching",
"add a retry", "add a flag", "make it async" — each names a remedy, and each is
one member of a family the requester picked from without saying so.

**Yields:** the highest-value findings in the whole set, and the ones that save
the most work, because a request satisfied at the goal level is often an order
of magnitude smaller than the mechanism it arrived as.

**Fails as:** condescension, every time, and it is the failure that gets this
whole skill switched off. A recovered goal is a *hypothesis about someone
else's intent*. You have noticed their words admit two readings — a fact about
the words, not privileged access to their mind. Ask "is this about X or about
Y?". Never say "what you actually want is". And when they are available to be
asked, asking beats inferring, always; inference here is what you do when
`presence` is not `paired`.

---

## Under — the assumptions the vocabulary smuggles in

**Looks at:** the commitments you accepted by answering in their words. Every
noun asserts a thing exists and is one thing. Every verb asserts a remedy.

**Questions:**
- Which nouns here are assumed to be singular? ("*the* uploader", "*the* user",
  "*the* config")
- Which are assumed to exist at all?
- What is the request treating as given that is actually a choice someone made?
- What would have to be true for this framing to be correct? Is it?

**Tell:** the third fix did not hold. Or the bug moved rather than dying. Or a
parameter appears to have no effect. All three are the signature of a correct
fix applied inside a wrong model — `diagnosing-before-fixing` owns the loop,
and this is what its "escalate to questioning the architecture" step actually
does when you get there.

**Yields:** the finding that explains a whole run of failures at once. Under is
the direction that pays in Diagnose specifically.

**Fails as:** infinite regress. Every assumption rests on another one, so the
questioning never terminates on its own. Bound it: list the assumptions, mark
each *load-bearing* or not by asking whether the work changes if it is false,
and examine only the load-bearing ones. Three is usually all there are.

---

## Above — the class this is one instance of

**Looks at:** the general case. This request has a shape, and the shape has
probably been through here before.

**Questions:**
- What is this an instance of?
- Has something of this shape come up before — in this repo, this month, this
  conversation?
- Is the general case *cheaper* than the specific one? Sometimes it is: one
  mechanism replacing four special cases is less code, not more.
- And the inverse, which matters more often: is the specific case genuinely
  all that is needed, and is the general one a way of avoiding a decision?

**Tell:** this is the second or third time something of this shape has come up.
Repetition is the only honest evidence that a general case exists;
`automating-repetition` owns the judgment about when a repeated shape becomes
machinery, and this is the sight that feeds it.

**Yields:** either a much smaller total, or a clear decision to stay specific
that will not be second-guessed later.

**Fails as:** premature abstraction, which is the most expensive failure in
this file. One instance is not a class. Two is a coincidence. The generalization
built on one instance is always the wrong generalization, because the features
you cannot see are exactly the ones belonging to the single case you have.
`judging-duplication` is the counterweight and it wins by default.

---

## Beside — what stands next to this once it exists

**Looks at:** the neighbourhood. Not consequences over time (that is *ahead*)
but adjacency now — what this touches, enables, blocks, or duplicates.

**Questions:**
- What becomes trivial once this exists, that is currently hard?
- What becomes impossible, or much harder?
- What already does most of this? A capability that exists under another name
  is the single most common thing a sweep finds.
- Who else is writing to this, and what do they not know is coming?

**Tell:** the change touches a boundary something else depends on. Or the tree
has another writer in it, in which case *beside* is not optional — see the
envelope note in the main file.

**Yields:** the duplicate-capability find, and the collision nobody had
connected to this work.

**Fails as:** the adjacent-possible trap. Beside reliably produces genuinely
good ideas that are genuinely not this task, and they are seductive precisely
because they are cheap *right now* while you have the context loaded. That
cheapness is real and it is not a route. Third row of the routing table:
into the record, offered at close, built if they say yes.

---

## Ahead — the second-order consequence

**Looks at:** the consequence of the consequence, and the person who inherits
this after everyone who built it has moved on.

**Questions:**
- Who maintains this in six months, and what do they need to know that is
  currently only in this conversation?
- What does this make hard to change later? Every mechanism is also a
  commitment; which one is being made here?
- If this succeeds completely, what breaks? Success is a load profile too.
- What is the reversal cost — and is anyone treating this as reversible when it
  is not? `deciding-reversibility` owns the answer; this is the question.

**Tell:** the work is easy now and permanent afterwards. Schemas, public
interfaces, file formats, anything with a migration on the other side of it,
and anything at all when the blast radius is `live`.

**Yields:** the one-way door nobody had labelled as one.

**Fails as:** speculative futureproofing — building for a load, a scale, or a
requirement nobody has. The discipline is that *ahead* produces **an
observation and a reversal cost**, not a feature. "This is a one-way door and
here is what it costs to walk back" is the finding. "So we should build the
general version now" is `above` failing, wearing this direction's clothes.

---

## Against — the inversion

**Looks at:** the negation of the request. The direction nobody takes, which is
why it is the highest-variance one here.

**Questions:**
- What if the answer is to **delete** something rather than add?
- What if the answer is to do **nothing** — what actually happens then?
- What if this were done **by hand**, once, to find out whether it recurs?
- What if the failure were made **impossible** rather than handled? An
  unrepresentable bad state needs no check.
- What if the constraint everyone is designing around were simply removed?

**Tell:** the design is accumulating special cases. Every special case is the
frame charging interest, and a fourth one is the signal to invert.

**Yields:** the largest simplifications available anywhere. `deleting-code` and
`modeling-state-machines` are where the results usually land.

**Fails as:** contrarianism — inverting to be interesting rather than to be
right. The check is cheap: an inversion has to be *argued for* on the same
terms as the original, and if the argument is "it is unexplored", it has not
been made. Novelty is a reason to look, never a reason to choose.

---

## Outside — what another discipline calls this

**Looks at:** the same problem, in a field that has already solved it and named
it something else.

**Questions:**
- What would someone who has never seen this codebase, this team, or this
  conversation ask first?
- Which discipline has this problem as a *solved standard* — distributed
  systems, control theory, accounting, logistics, typesetting, aviation, law?
- What is this problem called there, and what does that field consider the
  obvious answer?
- What does the stranger's question reveal that the expert's does not? Expertise
  is frame-acceptance made efficient, which is its value and its cost.

**Tell:** the problem feels novel. It almost never is, and that feeling is
mostly a report about your own vocabulary rather than about the problem.

**Yields:** an existing solved answer, an existing name to search under, and
occasionally a whole missing concept. This direction is routinely dismissed as
decorative and is not — on this skill's own creation it was the direction that
paid best, and [self-application.md](self-application.md) records exactly what
it produced and what it cost.

**Fails as:** analogy mistaken for argument. An import from another field is a
*hypothesis with a good pedigree*, and pedigree is not evidence. It gets tested
against this problem's actual constraints like anything else, and the
constraints that made it work there are the first thing to check for here.
