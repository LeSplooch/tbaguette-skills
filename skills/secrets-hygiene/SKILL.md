---
name: secrets-hygiene
description: Use when handling API keys, tokens, passwords, private keys, certificates, or connection strings — adding one to a service, CI pipeline, container image, or client app, or finding one in a commit, log line, screenshot, ticket, or error message. Also use when a credential would have to pass through you on its way to a service, when something you are about to run needs a request authenticated and the choice looks like handing it the credential or not running it, or when a transcript, context, or session summary may have held one. Covers leaked credential response, revocation and rotation, secret scanning, environment variable and dotenv handling, and pre-publication checks on a repository. Also use when a scanner's finding turns out to be your own fixture, or when a detector you wrote keeps flagging something legitimate.
---

# Secrets Hygiene

## Overview

A secret's only property is that its distribution is controlled, so every surface it has touched is a place it now lives. The measure of a healthy system is not whether secrets have leaked — they have — but whether any credential can be replaced on a Tuesday afternoon, without downtime and without a code change.

## When to use

- Adding a credential to a service, pipeline, image, device, or client application.
- A key, token, or private key appears in a diff, log, ticket, chat message, screenshot, or exception.
- Standing up CI or a deploy path; making a repository public; onboarding or offboarding someone.
- Someone offers to hand you a credential so that you can place it somewhere for them.
- Something you are about to run needs a request authenticated, and the choice looks like handing it the credential or not running it.
- Any sentence containing "temporarily hardcode", "just for local dev", or "it's only a test key".
- Not for: deciding what a credential should be *allowed to do* (`least-privilege-design`); whether a dependency can read your environment (`auditing-dependencies`).

## Where secrets must never be, and why each is worse than it looks

| Surface | Why it is worse than it looks |
|---|---|
| Source control | History is permanent and mirrored into every clone, fork, CI cache, and code-search index. Deleting the line changes nothing |
| Logs | Fan out to aggregators and third-party retention with a different access list — often readable by everyone in support |
| Error messages and stack traces | Reach users, crash reporters, and issue trackers; a connection string inside an exception is the classic path |
| URLs and query strings | Recorded by proxies, browser history, server and CDN access logs, and sent onward in referrer headers. Use a header or a body, never a query parameter |
| Client bundles and mobile apps | Anything shipped to a device is public; extracting a key from a binary is routine and tool-assisted. Obfuscation is not storage |
| Build artifacts | Image layers retain files a later layer deleted; source maps, debug symbols, and stray version-control directories ship with the artifact |
| CI output | Masking matches literal strings the system knows — a base64, JSON-wrapped, or URL-encoded variant prints in clear text |
| Screenshots, screen shares, recordings | Terminal scrollback and environment panes; a key in a demo video gets indexed |
| Crash reports and telemetry | Serialize whole config objects and capture the environment by default |
| Shell history, editor swap files, notebook outputs, dotfiles | Backed up, synced between machines, and indexed by desktop search |
| Chat and tickets | Search-indexed indefinitely, broad read access, exported during discovery |
| A conversation, transcript, or model context | Re-sent verbatim on every later turn, fanned out to session stores and request logs, and paraphrased into summaries and notes that outlive the session. Unlike logs and CI output, there is no masking step to fail here, because there is no masking step |
| Environment of a shared process | Readable by every child process and by anything that dumps the environment — better than a committed file, worse than a fetch on demand |

## Injection at runtime, not at build time

A build must be reproducible from a source tree with no secrets in it. If a secret is an input to the build, it is probably an output of the build too.

Precedence, worst to best: literal in source → encrypted file whose key is in the same repo → build-time argument → CI-injected environment at deploy → a mounted file read at start → fetched at runtime against a workload identity → no long-lived secret exists at all (mutual TLS, platform workload identity, signed platform requests). Each step down the list should require a written reason.

- Prefer a mounted file to an environment variable: the file has its own permissions and can be re-read after rotation without a restart, while the environment is visible to every child process and to any process dump.
- Read the secret at use time. A value captured into a global at import or boot is the mechanical reason "rotation requires a restart", and therefore the reason rotation gets postponed.
- Never pass a secret as a command-line argument — process listings are world-readable on most systems.
- Never log the resolved configuration at startup, at any level. This is the single most common way a correctly-stored secret ends up in a log aggregator.

## Never be the conduit for a value

The ladder above ends at the best outcome — no long-lived secret exists at all. There is a collection path that quietly reintroduces one, and it looks like helpfulness rather than like a mistake: a human needs to give a credential to some third party, and you offer to carry it.

Do not. **Anything you hold, you also record.** A value passing through you inherits every retention surface you have — the transcript, the session store, the request log, the summary written when the context is compacted — and it inherits them all at once, before any redaction step exists to catch it. Careful handling does not help, because the recording is not something the handling does; it is what the medium is.

So when a secret has to reach a third party, send the *person* to the issuer and take back a reference: the fact that it was set, a key id, a fingerprint, a URL that the party visits directly. An intermediary that never possesses a value cannot leak one, and that is a stronger property than any amount of discipline about a value it does possess.

This forbids being the **carrier**, not being the **configurer**. Writing a reference, a path, or a secret-store lookup into a configuration is the ordinary job, and no value crosses. And where a value reaches you anyway — pasted to be helpful, echoed by a command, printed in a stack trace — it is already on every surface in the table above, and nothing done afterwards takes it back off them. It is leaked. Go to the leak procedure below and start at revoke, rather than carrying on carefully with a value that is now burned.

The same rule runs in the other direction, as two separate failures. A credential belonging to a third party is not relayed back toward whoever asked — that is `least-privilege-design`'s confused deputy, a component applying its own authority to a target somebody else chose. And a token issued to you for one service is not presented to a second on the grounds that it was accepted; that one is an audience check the verifier skipped, which the same skill covers a few lines earlier.

## Between handing it over and refusing, there is a substitution

The section above is about a value travelling outward through you to a third party. The same trap has an inward form that arrives dressed as caution rather than as helpfulness, which is why it catches careful people: something you are about to run — a build step, a script, a subprocess, a tool invoked on your behalf — needs a request authenticated, and the options appear to be putting the credential where that thing can read it, or refusing to run it. The second looks like the responsible choice. It is the cheaper mistake rather than a different kind of one: it costs the task and buys a property a third arrangement would have bought outright.

In that arrangement the consumer is handed a **reference** — a placeholder, a handle, a name — and something between it and the service exchanges that reference for the real value at the moment of the request, for destinations named in advance. The request authenticates, and nothing the consumer can record ever held the credential: not its arguments, not its environment, not its logs, not the exception it throws, not the transcript of whatever ran it. The familiar instances predate the vocabulary — an agent that signs a challenge and never releases the key it signed with, a forwarding proxy that attaches the authorization on the way out.

This is the property the section above named, reached from the other side: an intermediary that never possesses a value cannot leak one, and here the intermediary is you. The shape is easy to reproduce without the property, so check two things before crediting an arrangement with either. **The exchange has to happen somewhere the consumer cannot reach.** A resolver that hands the value back to its caller is the ordinary counterfeit — a credential helper prints the password for the tool to read, which changes where the secret is stored and not whether the tool holds it, and from that moment it is on every surface that process has. One question settles it: if the consumer were told to dump everything it holds, would the credential be in the dump?

**And the list of destinations is the whole of the restriction.** Something that will attach the credential to whatever host it is asked about has widened the set of destinations that can spend it rather than protecting anything, and it has done so while looking exactly like the arrangement that works. That list is an allowlist and fails in all the ways one does; `least-privilege-design` covers what it is worth and what it costs to keep honest.

## Scoping, briefly

Record four narrowings beside every credential — resource, action, network origin, lifetime — plus a named owner and an expiry date at creation. Credentials without an expiry become permanent by default. Test question: could this one key, alone, read every customer's data? Then it is not a credential, it is a master key.

One credential shared by several consumers destroys attribution (logs show the key, not the actor) and makes rotation a cross-team negotiation, which is why it never happens. Non-production must never hold a credential that works in production; test fixtures containing real keys are a leading source of leaks. Deeper treatment: `least-privilege-design`.

## Rotation is a routine operation

**The test:** can you rotate every credential in the system today, in working hours, with no downtime and no code change? If not, you do not have rotation — you have an incident plan.

What makes it routine is dual-credential acceptance: the verifier accepts old and new during an overlap window while the issuer hands out the new one. Order: issue new → distribute → switch traffic → **observe that the old credential serves zero requests** → revoke old. Step four requires per-credential-ID usage telemetry; without it, revocation is a guess and the guess is why rotation feels dangerous. Without an overlap window, rotation is a synchronized outage, so it is always scheduled for a better week.

- Cadence: ≤ 90 days automated for machine credentials; ≤ 24 hours or per-request where a workload identity system can issue them; immediately on staff departure and on any suspected leak.
- Rotate on schedule even with no suspicion. The drill is what finds hardcoded copies, cached globals, and consumers nobody knew about — the same discovery you would otherwise make mid-incident.
- Asymmetric keys and certificates need the same overlap: distribute and trust the new public half before any signer switches to the new private half.

## Detection: three layers, none sufficient alone

1. **Pre-commit hooks** — cheapest, stops the accident before it exists. Cannot be relied on: bypassable, and only present on machines that installed it.
2. **Server-side scanning on every push** — the enforcement point, because it cannot be skipped. Scan diffs on every branch, not just merges (a secret on an abandoned branch is still published), and scan full history on a schedule.
3. **Provider-side monitoring** — leaked-credential feeds and push protection; the provider frequently notices before you do. Enable alerts on anomalous credential use: new region, new network, volume spike.

Scanners find high entropy and known prefixes and miss custom formats. **Give your own issued tokens a distinctive, greppable prefix.** It is a one-time design decision and the highest-leverage item on this page — it makes your secrets detectable by every scanner, yours and everyone else's. For logs, redact by field name in a structured logger with an explicit allowlist; regex redaction of free text catches only the shape you anticipated.

## Your own test data is the likeliest false positive

A scanner has a credibility budget and it is small. One finding a reader can reliably dismiss teaches them to dismiss the next one, and a secret scanner that gets skimmed has failed completely — there is no partial credit here, because the entire value sits in the one morning it fires for real.

The likeliest source of that dismissible finding is not a careless commit. It is **your own test data**, written by someone who needed a realistic-looking value precisely because the code under test parses, redacts, or refuses credentials. A string shaped convincingly enough to exercise the parser is shaped convincingly enough to trip the scanner, and now the repository's own suite is manufacturing the finding everyone learns to wave through.

Keep the matching literal out of the tree rather than keeping the scanner quiet about it. Assemble the fixture at runtime from parts that individually match nothing — a prefix constant joined to a body generated in the test — so no file contains a string to find, and the test still exercises the real shape. That beats both alternatives it replaces: an ignore rule turns detection off for a path permanently and is a control in its own right, and a deliberately malformed fixture stops testing the thing it was written for.

`designing-ci-pipelines` owns the general form — every advisory check eventually becomes ignored, and one overridden more than about one time in ten is either wrong or should be required. A scanner is the case where that law costs the most, because the finding it is ignored on is the only one that ever mattered.

## When a secret leaks, the order is the whole procedure

1. **Revoke.** Invalidate it at the provider. This is the only step that stops the bleeding. If revoking breaks production, break production — a live leaked credential is the worse outage.
2. **Rotate.** Issue and distribute the replacement. Deliberately after revocation: make rotation a prerequisite and you will delay revocation by hours.
3. **Audit for use.** Pull provider access logs for the credential's *entire lifetime*, not from the moment of discovery — exposure began when it was written, not when it was noticed. Look for unfamiliar sources and for actions the legitimate consumer never performs.
4. **Purge from history.** Rewrite history and delete artifacts last: it is slow, it forces every clone to be re-cloned, and it does nothing about copies that already exist.

Starting at step 4 is the common instinct and it is wrong. It feels like undoing the mistake, it removes the visible evidence, and it burns the hours during which the credential still works. It also manufactures a false resolution: the secret is gone from the tip of the branch and still authenticates.

**Any secret that touched a shared surface is burned.** "It was only in a private repo / an internal channel / live for five minutes" is not a control — private repos have forks, integrations, and read tokens, and five minutes is longer than an automated scraper needs. Rotate. The cost of an unnecessary rotation is one drill you owed yourself anyway.

Keeping a secret out of output that has already received it is a separate discipline with its own failure modes — see `redacting-sensitive-output`.

## Common mistakes

| Symptom | Real cause |
|---|---|
| The secret was removed in a follow-up commit | The commit that added it is still reachable; removal from the tip is cosmetic |
| Rotation requires a deploy | The value was captured into a constant or global at process start |
| Rotation was scheduled every quarter and never done | No overlap window, so rotation is an outage and every week is a bad week |
| Scanners are clean and a key still leaked | A custom token format with no recognizable prefix or entropy signature |
| Nobody can say which service uses this key | One credential, many consumers; it cannot be rotated without breaking something unknown |
| CI masking failed | The value was transformed before printing; masks match literals |
| An "encrypted" private key sits in the repo | The decryption key is in the same repo, the same image, or an environment everyone can read |
| The leak response began with a history rewrite | Revocation postponed behind the slowest and least effective step |
| Ignore rules treated as the control | Ignore files do not cover already-tracked files, other surfaces, or anyone's local tooling |
| Same key in staging and production | One credential spanning environments; the weakest environment sets the security of the strongest |
| A credential was pasted into the conversation so it could be placed | The conduit records whatever it carries; that value is now burned and needs revoking, not placing |
| The team skims the scanner's output | It has a standing false positive, and the usual source is a fixture shaped like a real credential because the code under test parses credentials |

## Red flags

- "That hit is just our test fixture." — said aloud once, then silently forever, including on the day it is not.
- "It's just a dev key" / "it's a test account".
- "It's in a private repository."
- "We'll rotate if we see abuse" — without per-credential telemetry, you will not see it.
- "Hardcode it for now and clean it up before merge."
- "The CI masking will catch it."
- Any response that begins by estimating who might have seen it.
- Rewriting history before revoking.
- Nobody can name the owner or the expiry of a production credential.
- A secret pasted into chat "so it can be copied over" — that message is now permanent.
