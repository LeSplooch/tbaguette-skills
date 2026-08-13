---
name: threat-modeling
description: Use when a design introduces a new trust boundary, a new class of sensitive data, a new external integration, or a change to authentication or authorization, when a design review needs a security section, when asked what could go wrong with a system or feature, or when reasoning about attackers, attack surface, blast radius, and which risks to fix first. Covers STRIDE, trust boundaries, data flow, attacker capability tiers, and risk ranking.
---

# Threat Modeling

## Overview

A threat model is a 30–60 minute conversation that produces decisions, not a document. Its output is a list of design changes, mitigations with tests, and explicitly accepted risks with owners. If the design did not change and nothing got a test, the session was theater.

## When to use

- A new trust boundary appears: a new listener, IPC surface, plugin or extension point, tenant, file format you now parse, or a queue you now consume.
- A new class of data enters: credentials, personal data, payment, health, keys, or telemetry that can carry user content.
- A new external integration or an authorization change: third-party API, webhook receiver, identity provider, a new role, a new token type, a session mechanism, or any delegation.
- Before writing the security section of a design doc, so the section reports decisions instead of inventing them.
- Not for: root-causing a specific reported bug; evaluating a package's supply chain (`auditing-dependencies`); writing the actual permission scopes (`least-privilege-design`); the input-handling rules themselves (`handling-untrusted-input`).

## The four questions

1. **What are we building.** One diagram, drawn from the running system, not the architecture doc — the doc is aspirational and the real system has an extra debug endpoint, a legacy path, and one listener nobody remembers. Data flows, not boxes: label what crosses each arrow.
2. **What can go wrong.** Enumerated per boundary, never per component. Component lists are unbounded; boundary lists come out at 5–15 and terminate.
3. **What are we going to do about it.** Every threat gets one of four verbs: eliminate, mitigate, transfer, accept — with a named owner.
4. **Did we do a good enough job.** Every "what can go wrong" has a decision; every mitigation names the test or assertion that will fail if someone removes it.

## The boundary list is where the value is

A trust boundary is any place where the trustworthiness of the caller changes. Enumerate all of these before thinking about threats:

| Boundary kind | Examples |
|---|---|
| Privilege | kernel/user, root/service user, sandbox edge, container edge |
| Network | internet-to-service, and service-to-service whenever the two run as different identities |
| Tenant | per-customer rows, per-customer keys, shared caches and shared indexes |
| Runtime and format | FFI, native extensions, unsafe blocks, eval or template compilation, and every parser — uploads, images, archives, config, serialized objects |
| Human | admin console, support tooling, break-glass, the ops runbook |
| Time | a token issued last quarter, an artifact built by a retired pipeline |
| Supply | a dependency's install hook, which runs at your privilege |

For each boundary write three lines: what crosses it, what authority the far side holds, and what happens if the far side lies. **If you cannot state what happens when the far side lies, that boundary is not modeled** — and shared caches, log pipelines, and message queues are the ones consistently missed.

## STRIDE as a prompt list, not a ritual

Apply per boundary, not per box. Budget 60–90 seconds per cell; when a cell needs more than five minutes, that's the finding — record it and move on.

| Prompt | Question at this boundary | Property it defends |
|---|---|---|
| Spoofing | Can the far side claim an identity it does not have? | Authentication |
| Tampering | Can data or code be altered in transit or at rest? | Integrity |
| Repudiation | Can an actor deny an action we cannot prove? | Attribution |
| Information disclosure | Who else can read this, including logs and errors? | Confidentiality |
| Denial of service | What unbounded work can a caller cause? | Availability |
| Elevation of privilege | Can a caller act with authority it was not granted? | Authorization |

Skip categories that genuinely do not apply — a read-only static asset path has no meaningful repudiation story — and say so out loud rather than filling the cell.

## Attackers, in capability tiers

Naming the tier turns "someone could" into "who, and would they bother". A generic "hacker" produces a generic mitigation.

| Tier | Has | Wants | Effort |
|---|---|---|---|
| 0 Curious user | a valid account, dev tools, your API docs | another tenant's data, a role upgrade, free usage | minutes |
| 1 Opportunist | mass scanners, credential lists, public exploits within days | any foothold, compute, a proxy | near zero, untargeted |
| 2 Targeted | reads your source, registers lookalike names, phishes staff, buys your expired domain | your specific data or money | days to weeks |
| 3 Insider / supply chain | a valid credential, a maintainer account, a CI token | anything, with your own authority | mitigated structurally, not detectively |

Label any mitigation that only stops tier 0 — unguessable URLs, undocumented endpoints, client-side checks — as exactly that. Tier 4 (state-level) is out of scope unless you can name why you are a target; write the exclusion down instead of leaving it ambiguous.

## Ranking: impact × reachability

Severity labels are computed for someone else's deployment. Reachability is where your system differs.

- **Reachability:** pre-auth ×4 · any authenticated user ×2 · operator or admin only ×1 · requires an existing compromise ×0.5 (that is a blast-radius item — route it to a privilege review).
- **Impact:** high if irreversible, cross-tenant, or credential-yielding; low if self-only, single-record, or self-recovering degradation.
- Fix order: pre-auth + irreversible, then pre-auth + cross-tenant, then authenticated + cross-tenant, then the rest. A critical-rated flaw in a path no request reaches outranks nothing.

## The output

Three columns: **Threat | Decision | Where it lives now.** Decisions rank in this order — design change so the threat cannot exist, mitigation with a named test, detection with a named alert and a named recipient, accepted with an owner and a review date. "We'll monitor it" without an alert name is acceptance spelled dishonestly.

## Assumptions are the tripwires

List them at the top of the model. Each one, if false, invalidates everything below it, and each has a cheap check that almost nobody runs.

| Assumption | Cheap verification |
|---|---|
| "This port is not reachable from the internet" | scan it from outside |
| "Only our own code calls this" — service, queue, or storage path | list who can actually call, publish, or write, from access logs rather than memory |
| "The input was sanitized upstream" | find the line that does it |
| "Only admins hold this role" | list current role assignments |

Re-run the model when any assumption changes. That change list is the trigger for re-modeling — without it, models rot silently.

## Common mistakes

| Symptom | Real cause |
|---|---|
| 40 threats, zero decisions | Modeled components instead of boundaries; component enumeration never terminates |
| Every threat begins "an attacker gains access and then…" | No capability tiers; "gains access" is a conclusion being used as a premise |
| Ranked by CVSS or a severity label | A context-free score used in place of reachability in your system |
| Long debate on crypto choice, nothing on authorization | Novel-looking risks crowd out the common ones; broken access control outnumbers crypto flaws by a wide margin |
| Mitigation exists, no test | A mitigation without a test is a comment; the next refactor deletes it silently |
| "The gateway will block that" | Control placed at a layer any direct caller bypasses |

## Red flags

- "That's an edge case, no real user would do that" — the attacker is not a user.
- "It's internal-only," asserted without a check that it is.
- "We authenticate the caller, so we're covered" — authentication answers *who*, not *what on whose behalf*.
- "We'll do security after the MVP" — trust boundaries are architecture; they are not retrofitted, and the session that ends with zero design changes has already conceded this.
- Nobody in the room can say which single credential could delete all customer data.
- Exactly one mitigation per threat — defense in depth was never considered.
