---
name: configuration-management
description: Use when adding an environment variable, setting, toggle, or config file entry, when secrets and configuration are tangled together, when something works in staging but not in production, when a missing or malformed setting surfaces as a null or a crash hours after startup, when standing up a new environment or deployment target, or when deciding whether a value belongs in code, config, a secret store, or a feature flag system.
---

# Configuration management

## Overview

Configuration is what varies between deployments of the same build, and nothing else. A value that does not vary is code that has been moved somewhere it cannot be typed, tested, or found. A value whose disclosure is harmful is a secret and follows different rules entirely. Getting these three categories confused is the source of most deployment surprises.

## When to use

- Adding a setting, environment variable, toggle, or config file entry
- Standing up a new environment, region, tenant, or deployment target
- A failure that appears only in one environment, or only after some time under load
- Credentials in a repository, a build argument, an image layer, or a log line
- Deciding whether something should be config, a constant, or a runtime feature flag
- Not for: the deployment mechanism itself, or infrastructure provisioning — this is about what the process reads at startup and how it behaves when that is wrong
- Not for: the full discipline once a value is identified as a secret — storage, rotation, and revocation are `secrets-hygiene`; scrubbing one from logs or output is `redacting-sensitive-output`

## Code, config, or secret

The test: **does this value differ between two deployments of the same build?**

| | Test | Where it lives | Rule |
|---|---|---|---|
| Code | Same in every deployment | In source, as a typed constant | A constant in code is reviewable, testable, and greppable. Moving it to config to "make it flexible" buys flexibility nobody uses and loses all three |
| Config | Differs by environment, region, or tenant | Files, environment, or a config service, versioned and reviewed | Non-sensitive. Should be printable in a support ticket |
| Secret | Differs, and disclosure causes harm | A secret manager, injected at runtime | Never in the repo, never in an image layer or build arg, never in a log or crash dump, rotatable without a code change or a rebuild |

Endpoints, pool sizes, timeouts, and log levels are config. Retry counts and business thresholds are usually code — they change with logic, not with environment, and putting them in config means the tested value and the running value are different numbers. Anything read only once at startup and identical everywhere was never config.

## Fail fast at startup

Load, parse, and validate the entire configuration into one typed structure before the process accepts any work. Nothing downstream reads raw environment or files.

- **Validate everything at once and report every failure**, not the first. Each message names the key, what was wrong, what was expected, and which source should supply it. A startup that dies three times in a row over three different keys wastes three deploy cycles.
- **Check more than presence:** type, range and units, format, and cross-field consistency — if TLS is enabled a certificate path is required; if a worker count is set, so is a queue bound. Cross-field validation is where real misconfiguration lives, since each value is individually plausible.
- **Exit non-zero.** Do not start degraded, do not substitute a default for a value the operator explicitly set to garbage, and never let a typo silently fall back to a working default — that produces the deployment that passes health checks while pointing at the wrong database.
- **Never read config at the call site.** A lookup buried on a code path that runs only during checkout is a config error that surfaces three hours later as a null, in a stack trace that names the wrong subsystem. Startup validation converts every one of those into a boot failure.
- **Fail fast means fast:** validation belongs before listeners bind and before the instance reports ready, so a bad rollout stops at the first replica instead of taking the fleet.

## Precedence, declared once

Declare the order in one place and apply it in exactly one loader:

**built-in defaults < config file < environment < command-line flags** (later wins)

Any value resolvable from two sources with different precedence in different modules is a defect waiting for an incident, and it presents as "the setting I changed had no effect." Two rules make it survivable: the effective value must record which source supplied it, and no module may consult a source directly once the loader has run.

## Environment parity

Every environment runs the same code path, the same loader, and the same validation; only values differ. When environments differ in *structure* — extra branches in staging, a mock injected only in test, validation skipped in development — the environments stop being comparable and "works in staging" stops being evidence of anything.

The first debugging move for an environment-specific failure is a diff of the effective configuration between the two, not a code read. Make that diff a single command. Config differences are the leading cause of environment-specific failures, and the diff finds in seconds what code inspection misses for hours because the code is identical.

## Make the effective config observable

The running process must be able to report the configuration it actually resolved — an endpoint, a log line at startup, an admin command — with provenance for each value (default, file, environment, flag) and secrets redacted.

Redact by allowlist, never denylist — see `redacting-sensitive-output` for why a denylist of sensitive-looking names always misses one (`dsn`, `webhook_url`, and `private_key_pem` are the ones that get past `password`/`token`, and the first miss is in a support bundle that's already been emailed). Specific to config: print secrets as a fixed marker plus a fingerprint (length, or the first six characters of a hash) so an operator can tell two wrong values apart without learning either.

## Defaults that are safe, not convenient

A default is a decision made on behalf of every operator who does not read the documentation. Choose the one that fails visibly rather than the one that starts quietly.

| Convenient default | Safe default |
|---|---|
| Bind all interfaces | Bind loopback; exposure is opt-in |
| Auth or TLS verification off | On; disabling requires an explicit, logged setting |
| No timeout, infinite retries | Finite everywhere; unbounded waits are how one slow dependency stalls a fleet |
| Debug endpoints, verbose errors, profiling on | Off outside development |
| Unbounded queues, caches, upload sizes | Bounded, with the bound stated |
| Destructive operations enabled | Disabled, or gated behind a second confirmation setting |

Where a wrong default causes a security or data incident, there is no default: require the value and fail to start without it. "Convenient in development" is served by a development config file, not by a permissive production default.

## When config is really a flag, or really a branch

**A flag, not config,** when the value must change without a restart, targets a subset of users, or gets flipped during an incident. Config is deploy-scoped and fleet-wide by nature; needing it faster or narrower than that means a real flag system with runtime evaluation, targeting, an audit trail, an owner, and an expiry date. A flag with no removal date is config with extra machinery, and a flag older than one release cycle should be deleted in one direction or the other.

**A code branch in disguise** when the value selects between materially different behaviors — a `mode` string that swaps algorithms, a `provider` that changes semantics rather than endpoints. That is polymorphism smuggled through a string: it doubles the test matrix, and at any moment one arm is running untested in production. Either every arm is exercised in CI and in a real environment, or the unused arm is deleted.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Null or type error hours after a clean startup | Config read lazily at the call site instead of validated at boot |
| A changed setting has no effect | Two sources resolve the same key with undeclared precedence, or the process caches a value read once |
| Works in staging, fails in production | A value present in one environment and absent in the other, with a silent default filling the gap |
| Service starts and passes health checks against the wrong dependency | A typo fell back to a default endpoint instead of failing |
| Secret in a log, crash dump, or support bundle | Redaction by denylist of suspicious-looking names |
| Rotating a credential needs a code change and a release | Secret embedded at build time rather than injected at runtime |
| Config file grows to hundreds of keys nobody can explain | Values that never vary by deployment were moved out of code |
| Cannot reproduce a production issue locally | No way to dump the effective config, so the two configurations were never compared |

## Red flags

- "It will just use the default" as an answer to a missing required value
- A new setting added without a validation rule and without a documented default
- Reading an environment variable outside the loader
- Copying a production secret into a local file, a ticket, or a chat message to reproduce something
- A boolean setting that has been flipped more than once during an incident and still lives in a config file
- Environment-specific code branches instead of environment-specific values
- Nobody can state where a running value came from
