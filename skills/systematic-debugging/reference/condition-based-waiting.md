# Condition-based waiting — poll for the condition, not a guess about its timing

A fixed delay before checking a result is a bet about how fast a machine will always be. The bet is good on the machine that wrote it and gets worse every time the test runs somewhere slower, busier, or more loaded — which is exactly what CI, a shared runner, or next year's dependency bump all are. Wait for the thing actually being tested, not for a number that once happened to be enough.

## The pattern

```
before — guesses at timing:
  sleep 50ms
  read result
  expect result is set

after — waits for the condition:
  wait until result is set, timeout 5s
  read result
  expect result is set
```

## Writing the wait

- **Poll on an interval, not a tight spin.** Checking every millisecond burns CPU for no benefit; checking every 10ms is usually indistinguishable to a human and costs nothing.
- **Always include a timeout.** A condition that never becomes true must fail loudly with a message naming what it was waiting for, not hang the run.
- **Re-read fresh state inside the loop.** Reading the value once before the loop and checking a cached copy inside it waits forever on data that already arrived.
- **Wait for the real condition, not a proxy for it.** A count, a status field, an event — whatever the test actually depends on, not a related value that usually changes around the same time.

## When a fixed delay is actually correct

Testing timing behavior itself — a debounce interval, a throttle window, a scheduled retry — has no condition to poll for; the delay *is* the thing under test. Even here, don't guess blind: wait for the triggering condition first, then apply the fixed span, and comment the number so the next reader knows it's derived rather than picked. A tool that ticks every 100ms needs two ticks to prove partial output arrived — the fixed wait is 200ms, documented as two tick-intervals, not a round number pulled from nowhere.

## Common mistakes

| Symptom | Real cause |
|---|---|
| Test passes locally, fails in CI | Fixed delay tuned to a faster machine than CI provides |
| Test occasionally hangs the whole suite | Polling loop with no timeout |
| Test flakes back in after being "fixed" | The wait reads a value cached before the loop started, not a fresh read on each check |
| Increasing the delay "fixes" it for a while | The delay was never the actual dependency; the flake rate just dropped below what the sample size would catch |
