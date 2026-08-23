---
name: least-privilege-design
description: Use when creating or reviewing a role, service account, API key, token scope, IAM or RBAC policy, or access control list, when a component needs access to something new, when one credential is shared across services or environments, when a service acts on a caller's behalf, or when reasoning about blast radius, privilege escalation, wildcards in policies, sandboxing, network egress, and separation of duties.
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

## The shared credential is the design flaw

One broadly-scoped credential used by several components turns every bug in *any* of them into total compromise — and the vulnerable component never had to be an important one. The secondary damage is what makes it permanent: attribution disappears (logs show a key, not an actor), rotation becomes cross-team coordination and therefore never happens, and the grant settles at the union of everyone's needs, which only grows. **Test:** for each credential, name the single process that uses it; if the answer is a team, a service group, or "the platform", it is shared. A non-production credential that also works in production is the same flaw with a different label.

## Blast radius is the design metric

For each component, answer in writing: *if an attacker runs arbitrary code as this component, what do they have?* The answer is exactly the union of its identity's permissions, its filesystem view, its network reachability, and every credential sitting in its memory or environment. Rank components by that answer, not by how exposed they are — the dangerous asymmetry is that the most-exposed component usually gets the least scrutiny of its permissions, because "it's only the frontend". Reductions in order of leverage:

1. **Split the credential per workload** — the largest single reduction, usually a configuration change.
2. **Make the reachable component read-only**, and move writes behind a narrow validated interface owned by a different identity.
3. **Remove the ability to grant permissions.** A role that can edit policy, attach roles, or create identities holds the transitive closure of everything it can grant itself. Permission-granting permissions do not look like privileges, which is why they survive policy reviews.
4. **Restrict egress.** A compromised component that cannot open arbitrary outbound connections cannot exfiltrate directly or pull a second stage. Almost never configured; among the highest-value controls available.
5. **Enforce the tenant predicate at the storage layer**, so a missing filter is a failed query rather than a cross-tenant read.

## Capabilities over ambient authority

Ambient authority means the callee's power comes from *who it is*, so any code path that reaches it inherits everything. That single mechanism produces confused-deputy bugs, request forgery, and metadata-service compromise alike: the request arrived, therefore the authority applied.

A capability travels with the reference: a pre-signed URL for one object and one method expiring in ten minutes; a file descriptor handed to a sandboxed child; a token naming one record. Possession *is* the authorization, so there is nothing to confuse. Practical rule — when a component needs to act on one resource, hand it a reference to that resource rather than access to the resource class. Every capability token needs an expiry, an audience, and an intended action, and the verifier must check all three: **a token whose signature is verified but whose audience is not is a capability for any service that accepts it**, which is how a token minted for one service gets replayed against another.

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
| Data | tenant predicate enforced in storage, column-level restriction on sensitive fields, separate identities for read, write, and migration | one connection identity for reads, writes, and schema changes |
| Build / CI | job-scoped token, no secrets in jobs that execute untrusted contributions, separate identities for build and publish | one pipeline credential that both tests and deploys |

## Reviewing what a role can actually do

Intent is not a control. Review the effective permissions, not the name or the description.

1. **Enumerate the effective set:** direct grants, everything inherited through groups and hierarchy, everything reachable via any role this identity may assume, and whatever the wildcards currently resolve to. Use the platform's effective-permission query — reading policy documents by hand misses inheritance chains.
2. **Find escalation paths:** can it grant permissions, assume another identity, modify a policy or a pipeline, write to a location that is later executed, read a secret belonging to a broader identity, or create a new identity? Any yes means its real privilege is the transitive closure.
3. **Compare against actual use:** list the actions taken in the last 90 days. The delta is the removal list, and it is the only evidence-based way to shrink a grant. Run it even on policies that look tidy — grants outlive their reasons.
4. **Verify the negative:** attempt one action the role must not perform and confirm the denial. An untested denial is an assumption — this is the step everyone skips and the only one that produces evidence. Repeat on every change to the role, and every 90 days for anything reaching production data.

When the demand for a justification arrives from outside — a questionnaire, an audit finding, a distribution review asking why a declared capability is needed — it is a prompt to run the compare-against-use step above, before writing a word. What that comparison returns decides the answer: used as described, justify it; used more narrowly, narrow the grant and justify what is left; not used at all, remove it, and the question retires along with it. Compare against observed calls rather than against whether the symbol appears — a capability referenced only from a path that never executes reads as used to a grep and as unused to the audit log. That last state accumulates because declaring is cheap once and removing later feels risky, while the justification cost recurs every review cycle until either the feature exists or the declaration is removed.

## Common mistakes

| Symptom | Real cause |
|---|---|
| One bug became total compromise | A shared broadly-scoped credential; the vulnerable component was never the sensitive one |
| Permissions were reviewed and are still too broad | The intended grant was reviewed, not the effective set with inheritance and assumable roles |
| Wildcards everywhere, and a dev credential that also works in production | Permissions derived up front from a design instead of from observed denials; one identity spanning environments |
| A "read-only" role can escalate | It can modify a policy, a pipeline, or a config that something later executes |
| An internal service was reached through a user-supplied URL | The deputy used its own network position; no check on the resolved target |
| A token was accepted by the wrong service | Signature verified, audience not |
| The two-person rule was bypassed | Approval enforced by process, not verified by the executing system |
| Temporary elevation became permanent | Grant with no expiry, and no review comparing grants against use |
| A compromised container exfiltrated data over an ordinary connection | Egress unrestricted, which is the default everywhere |

## Red flags

- "Give it admin for now, we'll narrow it later" — *later* has no trigger.
- "Declare it now so it's ready when we build the feature" — the feature is hypothetical; the review cost is not.
- "It's behind the firewall" / "only our own code calls it."
- A permission set that has only ever grown, or a wildcard with no written reason next to it.
- Being unable to name the single process that uses a credential.
- A role reviewed by reading its name or its description.
- "The user is authenticated" offered as the answer to an authorization question.
- Any credential with no expiry and no owner.
- Break-glass used more than a few times a year, or a pipeline that can modify its own permissions.
