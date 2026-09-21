---
name: least-privilege-design
description: Use when creating or reviewing a role, service account, API key, token scope, IAM or RBAC policy, or access control list, when a component needs access to something new, when one credential is shared across services or environments, when a service acts on a caller's behalf, when a local layer — a proxy, hook, sanitizer, formatter, or interceptor — sits between a producer and whoever acts on its output, or when reasoning about blast radius, privilege escalation, wildcards in policies, sandboxing, network egress, and separation of duties. Covers governing anything that can rewrite an observation at the privilege of the decisions it steers, and annotating rather than overwriting so the original survives. Also use when a sandbox, container, or network policy is described as isolated or egress-restricted and nothing has tried to leave it, or when an allowlist of permitted commands, operations, or tools is being written or relied on.
---

# Least-Privilege Design

## Overview

Least privilege is the property that decides what a bug costs. Every other control is probabilistic — this one is deterministic: it bounds the damage of the compromise you did not detect. Design for the assumption that some component will be fully controlled by an attacker, and choose which ones can afford that.

## When to use

- Creating a role, service account, key, token scope, policy, or access control entry, or a component needs access to something it did not previously reach.
- Designing service-to-service authentication, or any path where a service acts on a caller's request.
- Reviewing existing permissions, an over-broad policy, a wildcard, or a credential shared across services or environments; deciding what a build or pipeline may reach; designing irreversible or money-moving operations.
- Not for: where the credential is stored and how it is rotated (`secrets-hygiene`); whether the boundary should exist at all (`threat-modeling`).

## Default deny, and four axes at once

Start from zero and add only what a failed operation proves is required. The only reliable derivation is: deny everything, run the workload, read the denials, grant exactly those. Permissions derived from a design document are a superset every time, because the ones that turned out to be unnecessary are never removed.

| Axis | Question | Weak | Strong |
|---|---|---|---|
| Identity | Who is asking | one account for the fleet | one per workload, per environment |
| Resource | On what | `*`, whole bucket, whole database | one prefix, one table, one row predicate, one queue |
| Action | Doing what | admin, or a "full access" role | the three verbs actually called; read-only wherever reads suffice |
| Time | For how long | a permanent key | a token bounded to the job, minutes to hours |

Narrowing one axis and leaving three open is the common half-measure. Where the platform supports them, add two more that cost nothing and remove whole attack classes: **network origin** (which network or workload may present this identity) and **condition** (source pipeline identity, resource tag match, second factor present).

A policy correct on all four axes can still fail at evaluation time: where the component that computes a permission and the component that enforces it are different machines — a controller and a remote executor — a check validated against the controller's own filesystem or network conventions can pass cleanly and mean nothing on the executor that actually applies it. Validate against the runtime that enforces the policy, not the one that authored it.

## The shared credential is the design flaw

One broadly-scoped credential used by several components turns every bug in *any* of them into total compromise — and the vulnerable component never had to be an important one. The secondary damage is what makes it permanent: attribution disappears (logs show a key, not an actor), rotation becomes cross-team coordination and therefore never happens, and the grant settles at the union of everyone's needs, which only grows. **Test:** for each credential, name the single process that uses it; if the answer is a team, a service group, or "the platform", it is shared. A non-production credential that also works in production is the same flaw with a different label.

## Blast radius is the design metric

For each component, answer in writing: *if an attacker runs arbitrary code as this component, what do they have?* The answer is exactly the union of its identity's permissions, its filesystem view, its network reachability, and every credential sitting in its memory or environment. Rank components by that answer, not by how exposed they are — the dangerous asymmetry is that the most-exposed component usually gets the least scrutiny of its permissions, because "it's only the frontend". Reductions in order of leverage:

1. **Split the credential per workload** — the largest single reduction, usually a configuration change.
2. **Make the reachable component read-only**, and move writes behind a narrow validated interface owned by a different identity.
3. **Remove the ability to grant permissions.** A role that can edit policy, attach roles, or create identities holds the transitive closure of everything it can grant itself. Permission-granting permissions do not look like privileges, which is why they survive policy reviews.
4. **Restrict egress.** A compromised component that cannot open arbitrary outbound connections cannot exfiltrate directly or pull a second stage. Almost never configured; among the highest-value controls available.
5. **Enforce the tenant predicate at the storage layer**, so a missing filter is a failed query rather than a cross-tenant read.

### Egress is a list of permitted relays, not permitted destinations

Control 4 above gets written as an allowlist of hosts, and the channels that survive it are the ones nobody thinks of as connections. Name resolution is the standard one: arbitrary data leaves in subdomain labels and arbitrary instructions come back in the answers, without a single connection to anything the allowlist has heard of — and the resolver is left open because a container that cannot resolve names looks broken. Time sync, crash and telemetry reporting are the same shape. So is every allowlisted host that will store content a third party can read back: a package registry, a paste service, an issue tracker, a CDN that accepts uploads. An allowlist restricts *who you talk to*. It does not restrict *what can be relayed by the parties you are allowed to talk to*.

The second half matters more than the enumeration, because the list will always be incomplete: **a containment guarantee is tested by attempting to leave, never by reading the sentence that claims it.** Try a resolution, a time sync, and an upload to each allowlisted host, from inside the thing that is supposed to be contained. Documentation describing isolation is a statement of intent written by someone who was not measuring, and the gap between that sentence and the implementation is ordinary rather than exceptional — a sandbox advertised as having no external access can permit arbitrary DNS for its whole life, with the eventual fix including a correction to the documentation.

### An allowlist of operations restricts which names run, never what those names mean

The section above is about an allowlist of hosts. The same sentence holds one axis over, where it is worse: an allowlist of permitted commands, operations, or tools restricts *which names execute*, and it says nothing about what those names will do when they get there. Between the approval and the execution sits a layer of ambient interpretation that no allowlist models as a resource — an environment variable, a search path, a preloaded library, an alias, a shell profile, a pager or editor invoked on the caller's behalf, a file in the working directory that shadows the binary you meant. Change any of those and a listed, approved, entirely legitimate operation becomes a delivery mechanism. The check read the name, waved it through, and the setting that decides what the name actually does was changed a minute earlier by something the check was never shown.

Two properties let this survive the review that would catch a normal escalation. The first is that the audit trail is exculpatory by construction: it records the approvals, so the account of the incident is a list of things everybody agreed to and nothing in it is anomalous — while the operation that did the work either never reached the check or was waved through as harmless. Which is the second property, and it is what defeats per-operation classification. An operation that merely *sets* something touches no resource, returns no output, and is rated harmless by any classifier asking "what does this do", because its entire effect is on the interpretation of a later operation.

So the search heuristic that works everywhere else is inverted here. Blast-radius review points you at powerful operations; this lives among the null-looking ones. The instruction that catches it: **enumerate every operation that can run under this policy — the entries on the list, and anything the policy structurally never sees, such as a language's built-ins or an interpreter's own directives — and ask which of them change what a later entry means. Start with the ones that appear to do nothing.** If any exist, the allowlist bounds nothing, and its real scope is the interpretation state rather than the list.

None of this is specific to agents or to shells. It is the same shape wherever a policy matches on a string that something else later resolves: a command allowlist that permits environment to be carried across it, aliases and hooks that rebind a name, package lifecycle scripts, build-tool targets, and any policy engine deciding on the requested spelling of a thing rather than on the thing it will resolve to.

### An exemption matching part of a compound command must not exempt the whole of it

The allowlist failure above assumes one command per decision. A policy that instead matches a *string* — an exemption glob, a pre-approved pattern, a "this is safe, skip the sandbox for it" rule — inherits a second failure the moment the string it matches against can itself contain more than one command. `a && b`, `a; b`, `a | b`, and `$(a)` are several independent operations wearing one line, and a pattern that matches anywhere in that line authorizes the *line*, not the part it matched — so `b` runs with whatever `a`'s match bought it, unreviewed. The fix is the same discipline the section above asks of enumeration: split the string into its independent units first — respecting `&&`, `||`, `;`, `|`, and subshells — evaluate each unit against the policy separately, and let a match on one unit exempt only that unit.

## Capabilities over ambient authority

Ambient authority means the callee's power comes from *who it is*, so any code path that reaches it inherits everything. That single mechanism produces confused-deputy bugs, request forgery, and metadata-service compromise alike: the request arrived, therefore the authority applied.

A capability travels with the reference: a pre-signed URL for one object and one method expiring in ten minutes; a file descriptor handed to a sandboxed child; a token naming one record. Possession *is* the authorization, so there is nothing to confuse. Practical rule — when a component needs to act on one resource, hand it a reference to that resource rather than access to the resource class. Every capability token needs an expiry, an audience, and an intended action, and the verifier must check all three: **a token whose signature is verified but whose audience is not is a capability for any service that accepts it**, which is how a token minted for one service gets replayed against another.

### Mutating the caller's own environment is a grant, not a side effect

A plugin, extension, or subprocess that can set or rewrite the parent session's environment variables holds a capability equivalent to redirecting which binary a later command runs, disabling a safety check that reads a flag, or leaking a secret into every subprocess spawned afterward — but it gets reviewed as configuration rather than as a grant, because nothing about "setting a variable" resembles a file write or a network call. It is the allowlist blind spot from the section above, one layer up: an operation that looks like it changes nothing about the environment *is* the environment. Gate a request to mutate the host process's own environment the same as a request for a new permission class — explicit consent, and outright refusal (not merely a warning) for the small set of names the current process already trusts implicitly: `PATH`, `LD_PRELOAD`/`DYLD_INSERT_LIBRARIES`, interpreter-options variables like `NODE_OPTIONS`, and anything else that chooses which binary or library resolves.

### A cached grant's key must include every dimension that can change its meaning

A trust or consent decision cached against an identity, a path, or a session is only as safe as the key it is cached under. The common failure is a key that names *where* or *who* the decision was about but omits a dimension of *what* that can later change under the same identity and path — a directory trusted while empty can end up silently pre-authorizing project configuration added to that same directory afterward, because the cache key was the directory, not "this directory, holding nothing yet." The fix is not caching less; it is choosing a key that changes whenever the thing the original decision actually depended on changes, and invalidating or re-deciding whenever that dimension shifts — not only when the identity itself does.

## The confused deputy

A service with more authority than its caller, acting on the caller's request, applies its own authority to the caller's chosen target. The deputy is not compromised and is not buggy in the ordinary sense; it is obeying. Everyday instances: a URL-fetching feature reaching internal addresses or an instance metadata endpoint; an export job reading rows the requester cannot see; an admin tool that accepts a user identifier without checking whether the operator may act on *that* user; a webhook forwarder using its own credentials against a caller-supplied destination; a build system running caller-supplied configuration under the pipeline's identity.

The fix is structural — the deputy must carry the caller's authority, not its own. Concretely: forward the caller's token, or exchange it for one scoped to that caller with an audit trail, or check the caller's authority against the specific resolved target before using the deputy's own credential. The check runs on the resolved target, after canonicalization and name resolution, and is re-checked at use time; checking the requested value and acting on the resolved one is the bypass. "The caller is authenticated" is not the check — authentication answers *who*, and this problem is entirely *what, on whose behalf*.

## Time, machine identity, and human access

Preference order for a service credential, each step down requiring a written reason: platform workload identity (no secret exists) > short-lived token from an identity broker > per-workload long-lived key with narrow scope > shared key. Bind tokens to the shortest lifetime the operation tolerates — a 20-minute job does not need a 12-hour token.

For humans: zero standing access to production by default; elevation is requested, justified, time-boxed to ≤ 4–8 hours, and logged. The record of who elevated and why is a stronger control than the approval step itself. Break-glass is exactly one path, alerts a human on use, and is reviewed within a day — break-glass used routinely is just the normal path with worse logging.

## Separation of duties

Apply to operations that are irreversible, move money or entitlements, change who has access, or alter the audit trail: deletion of data or environments, granting a role, refunds and payouts above a threshold, full dataset export, disabling logging or retention.

- The requester and the approver must be different identities, and **the approval must be verified by the system performing the action.** A two-person rule that any one person can bypass with their own credential is a documentation artifact, not a control.
- Prefer removing irreversibility to gating it: soft delete with a retention window, queued transfers with a cancellation period, grants that expire. A control converting permanent loss into recoverable loss beats a control that gates permanent loss.
- Automation is a party. A pipeline that can both approve and deploy has merged the duties, and whoever can edit the pipeline definition holds every permission the pipeline holds.

## Boundaries the permission system does not cover

| Layer | Narrowest practical form | Usually skipped |
|---|---|---|
| Process and filesystem | dedicated non-root user, no privilege elevation, dropped capabilities, syscall filter, read-only root with one writable path, only the credentials this process uses mounted | running as root because the base image did; mounting the whole config directory or a host control socket |
| Network | deny-by-default egress, ingress only from the one caller, metadata endpoint blocked from application containers | egress — nearly every deployment allows all outbound |
| Network — relays | resolver restricted to an internal server with logging, no direct outbound DNS; time sync and crash reporting pinned to internal endpoints; allowlisted hosts assessed for whether a third party can read back what is written to them | the resolver, always — an egress allowlist is written in hostnames and enforced on connections, and resolution is neither |
| Data | tenant predicate enforced in storage, column-level restriction on sensitive fields, separate identities for read, write, and migration | one connection identity for reads, writes, and schema changes |
| Build / CI | job-scoped token, no secrets in jobs that execute untrusted contributions, separate identities for build and publish | one pipeline credential that both tests and deploys |

## The layer that rewrites an observation outranks the one that produced it

Permission systems are built around who may *call* a thing. They rarely model
who may change what the call appeared to return. A result that passes through
any local transform on its way to whoever acts on it — a proxy, an interceptor,
a formatter, a sanitizer, a log shipper, a post-processing hook — is a result
that transform decided. The producer's authentication, authorization and audit
log all describe what the producer sent. None of them describes what the
consumer saw, so a correctly authorized call and a fabricated answer are
indistinguishable on the far side.

That makes the transform the higher-privilege component of the pair, which is
the reverse of how it gets reviewed. These layers are installed for cosmetic
reasons — truncate this, redact that, normalize the shape — and are read as
formatting conveniences, so they attract far less scrutiny than the service
whose output they rewrite while holding strictly more power over the decision.
A sanitizer that strips anything error-shaped does not merely tidy the output;
it can make a failure invisible to the only party in a position to react to it.

The second half is worse and is easy to miss while arguing about the first. A
transform that **replaces** rather than annotates destroys the evidence of what
was actually returned. Nothing downstream can reconstruct it, the audit trail on
the producer's side records a call that looks entirely normal, and the
discrepancy exists only in the gap between two logs nobody joins. So: govern
anything that can rewrite an observation at the privilege of the decisions it
influences, not at the privilege of the cosmetic job it was installed to do —
and prefer transforms that add a field over transforms that overwrite one, so
the original survives to be compared against.

## A rule the parser could not read is a rule that is not there

A policy has to be read from somewhere, and something does the reading. What that reader does with a line it cannot understand is usually not a decision anybody made: the common behaviour is to skip the line and continue, because refusing to start over a single bad entry is an unattractive property in a component everything else waits on, and tolerating syntax from a later version wants the same leniency.

The cost of that skip is not uniform. It splits on the polarity of the rule dropped. Skip a malformed **allow** and something stops working — a request that should have succeeded is refused, somebody notices within the hour, and the report arrives naming the thing. Skip a malformed **deny** and nothing happens at all, which is exactly what a deny rule looks like when it is working. The restriction is gone, the system is more permissive than the file on disk describes, and the file goes on describing it. One typo, loud in one half of a policy and silent in the other.

That is one more argument for the default-deny section above, arriving from an unexpected direction. Wherever the grant is a broad allow with explicit denies carving exceptions out of it — a permissive firewall with specific blocks, a wide role with a restriction attached, an ignore file with negations — every carve-out is a rule whose disappearance is invisible. Start from zero instead and the fragile direction becomes the one that fails loudly.

Two consequences. **A policy loader should refuse to start on a rule it cannot parse**, and where it genuinely must tolerate unknown syntax, that tolerance belongs on the permissive rules and not on the restrictive ones. And which behaviour you actually have takes about a minute to establish: put a deliberately malformed rule into a copy of the policy and see whether the loader objects or comes up clean. An empty denial log will not tell you, because a rule that never loaded and a rule that was never violated leave the identical record — which is why step 4 below has to end in an attempt rather than in a reading.

### A narrow bypass that does not work erodes the whole control

A scoped, single-purpose escape hatch — an override for one specific blocked case, a break-glass command, a permissive retry after one named denial — exists so an operator does not have to disable the real control to get past a false positive. That only holds if the narrow path actually works. One found in the wild silently did not: the permissive retry after a sandbox denial stopped instead of running the command it was supposed to allow, so invoking the bypass simply did nothing. The only route that reliably worked was turning the sandbox off for the whole session.

An escape hatch nobody has exercised end to end is a claim, not a control — the same gap the paragraph above closes for a deny rule — and its failure mode is worse, because a broken deny rule fails loud and gets fixed, while a broken bypass fails by teaching whoever hit it that the reliable way past *this* control is the big switch, a lesson that generalizes to the next control they meet. **Test the narrow path itself, on the case it claims to handle, on the same cadence the main control gets tested** — not only that the control blocks, but that the sanctioned way around it actually gets someone through.

## Reviewing what a role can actually do

Intent is not a control. Review the effective permissions, not the name or the description.

1. **Enumerate the effective set:** direct grants, everything inherited through groups and hierarchy, everything reachable via any role this identity may assume, and whatever the wildcards currently resolve to. Use the platform's effective-permission query — reading policy documents by hand misses inheritance chains.
2. **Find escalation paths:** can it grant permissions, assume another identity, modify a policy or a pipeline, write to a location that is later executed, read a secret belonging to a broader identity, or create a new identity? Any yes means its real privilege is the transitive closure.
3. **Compare against actual use:** list the actions taken in the last 90 days. The delta is the removal list, and it is the only evidence-based way to shrink a grant. Run it even on policies that look tidy — grants outlive their reasons.
4. **Verify the negative:** attempt one action the role must not perform and confirm the denial. An untested denial is an assumption — this is the step everyone skips and the only one that produces evidence. Repeat on every change to the role, and every 90 days for anything reaching production data.

When the demand for a justification arrives from outside — a questionnaire, an audit finding, a distribution review asking why a declared capability is needed — it is a prompt to run the compare-against-use step above, before writing a word. What that comparison returns decides the answer: used as described, justify it; used more narrowly, narrow the grant and justify what is left; not used at all, remove it, and the question retires along with it. Compare against observed calls rather than against whether the symbol appears — a capability referenced only from a path that never executes reads as used to a grep and as unused to the audit log. That last state accumulates because declaring is cheap once and removing later feels risky, while the justification cost recurs every review cycle until either the feature exists or the declaration is removed.

## Revoking a capability must close every path that reaches it

Granting narrowly and revoking completely are the same discipline read in opposite directions, and only one of them usually gets the review above. A grant is checked against every route it opens; a disable or revoke path is written against the one route its own subsystem tracks, and stays reachable through whichever other route the toggle did not clear.

The shape recurs wherever a capability can be granted through more than one mechanism: a sync cache that outlives the setting that filled it, a second configuration source layered alongside the one just edited, a second component that independently re-contributes the same connector or permission. Three unrelated systems hit this shape independently and at once: a client-side cache that kept a capability available after an org-level toggle turned it off, a managed policy field silently ignored whenever a second managed-settings source also existed, and a connector that stayed reachable through a second still-enabled plugin after its canonical owner was disabled.

**Before shipping a disable or revoke path, enumerate every route by which the capability could still be reachable — the same enumeration step 1 above runs for a grant — and confirm the toggle clears all of them, not just the one it was written against.** Then verify it the way step 4 above verifies a denial: put the capability in place through every route you can find, flip the toggle meant to remove it, and check each route in turn rather than trusting the one the code intercepts.

## Common mistakes

| Symptom | Real cause |
|---|---|
| One bug became total compromise | A shared broadly-scoped credential; the vulnerable component was never the sensitive one |
| A disabled capability stayed reachable | The revoke path cleared the one route its own subsystem tracked; a cache, a second config source, or a second grantor left another route open |
| An operator disabled the whole control to get past one blocked case | The narrow bypass built for that case silently didn't work, so the switch was the only route that reliably did |
| Permissions were reviewed and are still too broad | The intended grant was reviewed, not the effective set with inheritance and assumable roles |
| Wildcards everywhere, and a dev credential that also works in production | Permissions derived up front from a design instead of from observed denials; one identity spanning environments |
| A "read-only" role can escalate | It can modify a policy, a pipeline, or a config that something later executes |
| Every entry in the audit log was approved and the outcome was still hostile | The allowlist matched names; an unlisted, effect-free operation had already changed what one of those names meant |
| An exempted command's second half did something nobody approved | The exemption matched the whole compound line, not the one unit it was written for |
| A directory or session became pre-authorized for something added to it later | A cached trust decision's key named the identity but not the dimension that changed |
| A plugin quietly redirected which binary or library a later command used | An extension's environment-mutation request was treated as configuration, not as a capability grant |
| An internal service was reached through a user-supplied URL | The deputy used its own network position; no check on the resolved target |
| A token was accepted by the wrong service | Signature verified, audience not |
| The two-person rule was bypassed | Approval enforced by process, not verified by the executing system |
| A restriction everyone can point to in the policy file was never in force | The loader skipped a rule it could not parse; a deny that never loaded and one that was never triggered log the same nothing |
| Temporary elevation became permanent | Grant with no expiry, and no review comparing grants against use |
| A compromised container exfiltrated data over an ordinary connection | Egress unrestricted, which is the default everywhere |
| A component with no outbound access still exfiltrated | Egress was enforced on connections and written in hostnames; resolution is neither, and it was left open because a container that cannot resolve looks broken |

## Red flags

- "The docs say the sandbox has no external access." — a sentence written by somebody who was not measuring; try a resolution.
- "Give it admin for now, we'll narrow it later" — *later* has no trigger.
- "Declare it now so it's ready when we build the feature" — the feature is hypothetical; the review cost is not.
- "It's behind the firewall" / "only our own code calls it."
- A permission set that has only ever grown, or a wildcard with no written reason next to it.
- Being unable to name the single process that uses a credential.
- A role reviewed by reading its name or its description.
- "The user is authenticated" offered as the answer to an authorization question.
- Any credential with no expiry and no owner.
- Break-glass used more than a few times a year, or a pipeline that can modify its own permissions.
- "We turned it off" said about a capability with more than one source that can grant it.
- An escape hatch nobody who built it has ever actually walked through end to end.
- An exemption or allowlist pattern tested only against a whole command line, never against each unit a `&&`, `;`, `|`, or subshell splits it into.
- "It's just setting an environment variable" said about anything a plugin or extension does to the host process.
- A trust or consent decision cached under an identity or a path, with nothing that re-checks it when what that identity or path actually holds changes.
