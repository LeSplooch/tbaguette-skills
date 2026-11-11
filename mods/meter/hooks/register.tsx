import { atom, read, update } from 'claude-code'
import type { EngineInterface, ModelUsage, Register, SessionContextUsage, SessionCost, SessionRateLimit, Timer } from 'claude-code'

import type { MeterCold, MeterContext, MeterCrossing, MeterLifetime, MeterReadings, MeterWindow } from '../types'

const MINUTE = 60_000
const HOUR = 60 * MINUTE
const EMPTY: MeterReadings = { context: null, windows: [], cost: null, cold: null }
const ASSUMED: MeterLifetime = {
  ms: 5 * MINUTE,
  label: 'assumed',
  evidence: 'the shortest time the cache keeps an entry; nothing here has shown it longer',
}

// Change a tag when its type's fields change: the atom declines what an earlier version stored.
const readings = atom({ plugin: 'meter', key: 'readings' } as const, EMPTY, { shape: 'readings-1' })
const crossings = atom({ plugin: 'meter', key: 'crossings' } as const, [], { shape: 'crossings-1' })
const lifetime = atom({ plugin: 'meter', key: 'lifetime' } as const, ASSUMED, { shape: 'lifetime-1' })
const isQuiet = atom({ plugin: 'meter', key: 'isQuiet' } as const, false)
const lastActivity = atom({ plugin: 'meter', key: 'lastActivity' } as const, null)

const CONTEXT_LINE = 80
const WINDOW_LINES = [80, 95]
const REARM = 5
const COLD_FLOOR = 50_000
const SHORT_BELOW = 60
// `[ Hide ]` and the space before it
const HIDE_CELLS = 9
const NAMES: Record<string, { short: string; long: string }> = {
  five_hour: { short: '5h', long: '5-hour window' },
  seven_day: { short: 'wk', long: 'weekly window' },
}
const USAGE = [
  'Usage:',
  '  /meter          every figure, where it came from, and the cache lifetime',
  '  /meter quiet    keep the status line and silence the band for this session',
  '  /meter loud     bring the band back',
].join('\n')

type Usage = { context: SessionContextUsage; rateLimits: readonly SessionRateLimit[]; cost?: SessionCost }

function kilo(tokens: number): string {
  return `${Math.round(tokens / 1000)}k`
}

/** 5h first, then the week, then any other window in the order the host reported it. */
function ordered(windows: readonly MeterWindow[]): MeterWindow[] {
  const rank = (kind: string) => (kind === 'five_hour' ? 0 : kind === 'seven_day' ? 1 : 2)
  return [...windows].sort((a, b) => rank(a.kind) - rank(b.kind))
}

/** The fill as the status line reads it; null while the live window has no response yet. */
export function contextOf(c: SessionContextUsage | undefined, at: number): MeterContext | null {
  if (c === undefined) return null
  const percent = c.percent ?? (c.tokens !== undefined && c.window > 0 ? Math.round((c.tokens / c.window) * 100) : undefined)
  if (percent === undefined) return null
  return { percent, tokens: c.tokens ?? null, window: c.window, at }
}

export function measured(prev: MeterReadings, u: Usage, at: number): MeterReadings {
  return {
    ...prev,
    context: contextOf(u.context, at),
    windows: (u.rateLimits ?? []).map(l => ({
      kind: l.kind,
      percentUsed: l.percentUsed,
      ...(l.resetsAt === undefined ? {} : { resetsAt: l.resetsAt }),
      at,
    })),
    cost: u.cost === undefined ? null : { usd: u.cost.usd, at },
  }
}

export function statusLine(r: MeterReadings): string | undefined {
  const units: string[] = []
  if (r.context !== null) units.push(`ctx ${Math.floor(r.context.percent)}%`)
  for (const w of ordered(r.windows)) units.push(`${NAMES[w.kind]?.short ?? w.kind} ${Math.floor(w.percentUsed)}%`)
  if (r.cost !== null) units.push(`$${r.cost.usd.toFixed(2)}`)
  return units.length === 0 ? undefined : units.join(' · ')
}

/** Cold once idle for the lifetime with enough context that a re-read costs something. */
export function coldOf(r: MeterReadings, since: number | null, life: MeterLifetime, now: number): MeterCold | null {
  const tokens = r.context?.tokens ?? null
  if (since === null || tokens === null || tokens < COLD_FLOOR || now - since < life.ms) return null
  return { idleMs: now - since, tokens, at: now }
}

type Line = { id: string; source: string; isAbove: boolean; isClear: boolean }

function line(source: string, level: number, value: number | undefined): Line {
  return {
    id: `${source}@${level}`,
    source,
    isAbove: value !== undefined && value >= level,
    // a figure that is gone counts as fallen back
    isClear: value === undefined || value <= level - REARM,
  }
}

function linesOf(r: MeterReadings): Line[] {
  return [
    line('context', CONTEXT_LINE, r.context?.percent),
    ...r.windows.flatMap(w => WINDOW_LINES.map(level => line(w.kind, level, w.percentUsed))),
    { id: 'cache@0', source: 'cache', isAbove: r.cold !== null, isClear: r.cold === null },
  ]
}

/**
 * Lines whose figure fell 5 points under re-arm; armed lines whose figure reached them are
 * announced. A new crossing hides older ones of the same figure, so 95% replaces 80%.
 */
export function crossed(prev: readonly MeterCrossing[], r: MeterReadings): MeterCrossing[] {
  const lines = linesOf(r)
  let next = prev.filter(c => lines.some(l => l.id === c.id && !l.isClear))
  for (const l of lines) {
    if (!l.isAbove || next.some(c => c.id === l.id)) continue
    next = [...next.map(c => (c.source === l.source ? { ...c, isHidden: true } : c)), { id: l.id, source: l.source, isHidden: false }]
  }
  return next
}

/** Longest first. */
function forms(c: MeterCrossing, r: MeterReadings): string[] | null {
  if (c.source === 'context') {
    if (r.context === null) return null
    const at = `context at ${Math.floor(r.context.percent)}% (measured)`
    return [
      `${at}: a compaction is near; a handoff note written now costs less than one after it`,
      `${at}: a compaction is near; a handoff note costs less before it`,
      `${at}: a compaction is near`,
      `context ${Math.floor(r.context.percent)}% (measured)`,
    ]
  }
  if (c.source === 'cache') {
    if (r.cold === null) return null
    const lead = `cache likely cold (estimate: idle ${Math.floor(r.cold.idleMs / MINUTE)} min)`
    return [
      `${lead}: the next send re-reads about ${kilo(r.cold.tokens)} tokens at full price`,
      `${lead}: the next send re-reads about ${kilo(r.cold.tokens)} tokens`,
      lead,
      'cache likely cold (estimate)',
    ]
  }
  const w = r.windows.find(x => x.kind === c.source)
  if (w === undefined) return null
  const percent = Math.floor(w.percentUsed)
  return [`${NAMES[w.kind]?.long ?? w.kind} at ${percent}% (measured)`, `${NAMES[w.kind]?.short ?? w.kind} ${percent}% (measured)`]
}

/** The newest crossing still shown, in the longest form that fits beside `Hide`. */
export function bandText(list: readonly MeterCrossing[], r: MeterReadings, columns: number): string | null {
  const shown = list.filter(c => !c.isHidden && forms(c, r) !== null)
  const top = shown[shown.length - 1]
  if (top === undefined) return null
  const all = forms(top, r) ?? []
  const others = new Set(shown.filter(c => c.source !== top.source).map(c => c.source)).size
  const isShort = columns < SHORT_BELOW
  const room = Math.max(1, columns - HIDE_CELLS)
  const suffix = others === 0 ? '' : isShort ? ` +${others}` : ` (+${others} more in /meter)`
  for (const text of isShort ? all.slice(-1) : all) {
    if (text.length + suffix.length <= room) return text + suffix
  }
  const last = `${all[all.length - 1] ?? ''}${suffix}`
  return last.length <= room ? last : `${last.slice(0, room - 1)}…`
}

/**
 * The rule, from the first request of a turn of at least 50,000 input tokens: after more than
 * 5 minutes idle, reading most of it from the cache means the lifetime is longer than 5 minutes
 * (60 observed); after less than the current lifetime, writing nearly all of it fresh means it
 * is not (back to 5 assumed). Two numbers and an inference, so it can be wrong.
 */
export function judged(life: MeterLifetime, gapMs: number, u: ModelUsage): MeterLifetime | null {
  const total = u.input_tokens + u.cache_read_input_tokens + u.cache_creation_input_tokens
  if (total < COLD_FLOOR) return null
  const minutes = Math.round(gapMs / MINUTE)
  const read = u.cache_read_input_tokens / total
  const fresh = u.cache_creation_input_tokens / total
  if (life.ms !== HOUR && gapMs > ASSUMED.ms && read > 0.5) {
    return {
      ms: HOUR,
      label: 'observed',
      evidence: `a turn after ${minutes} min idle still read ${Math.round(read * 100)}% of its input from the cache`,
    }
  }
  if (life.ms !== ASSUMED.ms && gapMs < life.ms && fresh >= 0.9) {
    return {
      ...ASSUMED,
      evidence: `a turn after ${minutes} min idle wrote ${Math.round(fresh * 100)}% of its input fresh, so 60 minutes did not hold`,
    }
  }
  return null
}

export function reportLines(r: MeterReadings, life: MeterLifetime, since: number | null, isQuietNow: boolean, now: number): string[] {
  const ago = (at: number) => {
    const seconds = Math.max(0, Math.round((now - at) / 1000))
    return seconds < 60 ? `${seconds}s ago` : `${Math.floor(seconds / 60)} min ago`
  }
  const lines: string[] = []
  lines.push(r.context === null
    ? 'context: no reading yet; one arrives with the next reply'
    : `context: ${Math.floor(r.context.percent)}% of a ${kilo(r.context.window)} window` +
      `${r.context.tokens === null ? '' : `, ${kilo(r.context.tokens)} tokens`} (measured ${ago(r.context.at)})`)
  if (r.windows.length === 0) {
    lines.push("rate limits: none reported; off a subscription there are none, on one they arrive with this session's next reply")
  }
  for (const w of ordered(r.windows)) {
    lines.push(`${NAMES[w.kind]?.long ?? w.kind}: ${w.percentUsed}%${w.resetsAt === undefined ? '' : `, resets ${w.resetsAt}`} (measured ${ago(w.at)})`)
  }
  lines.push(r.cost === null ? 'cost: no figure; this host keeps no ledger' : `cost: $${r.cost.usd.toFixed(2)} (measured ${ago(r.cost.at)})`)
  const minutes = life.ms / MINUTE
  if (since === null) {
    lines.push('cache: no estimate while a turn runs or before the first one ends')
  } else {
    const idle = Math.floor((now - since) / MINUTE)
    const tokens = r.context?.tokens ?? null
    lines.push(now - since >= life.ms
      ? `cache: likely cold (estimate from idle ${idle} min against the ${minutes} min lifetime` +
        `${tokens === null ? '' : `; the next send re-reads about ${kilo(tokens)} tokens`})`
      : `cache: likely warm (estimate from idle ${idle} min against the ${minutes} min lifetime)`)
  }
  lines.push(`cache lifetime: ${minutes} min (${life.label}: ${life.evidence})`)
  lines.push(isQuietNow ? 'band: quiet for this session; /meter loud brings it back' : 'band: shows when a figure crosses a line; /meter quiet silences it')
  return lines
}

// The module's own variables start over on a reload; `session.start` arms the timer again.
let coldTimer: Timer | null = null
// The idle gap the running main turn began after, for its first request to answer.
let pendingGap: number | null = null

function disarm() {
  coldTimer?.cancel()
  coldTimer = null
}

/** One timer per idle stretch, set at a turn's end: it marks the cache cold once, and stops. */
function arm($: EngineInterface, ms: number) {
  disarm()
  coldTimer = $.clock.after(ms, () => {
    coldTimer = null
    void refreshCold($)
  })
}

async function recross($: EngineInterface) {
  const r = await read($, readings)
  const before = await read($, crossings)
  if (JSON.stringify(crossed(before, r)) === JSON.stringify(before)) return
  await update($, crossings, list => crossed(list, r))
}

async function take($: EngineInterface, u: Usage, at: number) {
  const r = await update($, readings, prev => measured(prev, u, at))
  $.ui.status(statusLine(r))
  await recross($)
}

async function refreshCold($: EngineInterface) {
  const now = await $.clock.now()
  const since = await read($, lastActivity)
  const life = await read($, lifetime)
  const before = await read($, readings)
  if (before.cold === null && coldOf(before, since, life, now) === null) return
  await update($, readings, r => ({ ...r, cold: coldOf(r, since, life, now) }))
  await recross($)
}

async function begin($: EngineInterface) {
  const now = await $.clock.now()
  const since = await read($, lastActivity)
  pendingGap = since === null ? null : now - since
  await update($, lastActivity, () => null)
  await refreshCold($)
}

async function learn($: EngineInterface, gapMs: number, usage: ModelUsage) {
  if (judged(await read($, lifetime), gapMs, usage) === null) return
  await update($, lifetime, life => judged(life, gapMs, usage) ?? life)
}

async function report($: EngineInterface): Promise<string> {
  const now = await $.clock.now()
  const lines = reportLines(await read($, readings), await read($, lifetime), await read($, lastActivity), await read($, isQuiet), now)
  return lines.join('\n')
}

export const register: Register = on => {
  on('session.start', async ($, e, next) => {
    await $.command.register({
      name: 'meter',
      description: 'Context, rate-limit windows, cost and the prompt cache, each marked measured or estimated',
      argumentHint: '[quiet | loud]',
      immediate: true,
    })
    // Read once here; from now on the host pushes every change through `session.measure`.
    await take($, await $.session.usage(), await $.clock.now())
    const since = await read($, lastActivity)
    if (since !== null) {
      const left = (await read($, lifetime)).ms - ((await $.clock.now()) - since)
      if (left > 0) arm($, left)
      else await refreshCold($)
    }
    return next(e)
  })

  // Pushed by the host when a figure moves: no poll and no model call.
  on('session.measure', async ($, e, next) => {
    await take($, e, await $.clock.now())
    return next(e)
  })

  // A /clear starts a new conversation: its context and its cache are not the old ones.
  on('session.end', { reason: 'clear' }, async ($, e, next) => {
    disarm()
    pendingGap = null
    await update($, lastActivity, () => null)
    const r = await update($, readings, prev => ({ ...prev, context: null, cold: null }))
    $.ui.status(statusLine(r))
    await recross($)
    return next(e)
  })

  // The bookkeeping waits a tick so the turn's first request does not wait for it.
  on('turn.start', ($, e, next) => {
    if (e.agentId === undefined) {
      disarm()
      $.clock.after(0, () => void begin($))
    }
    return next(e)
  })

  // The first request of a turn is the one that says whether the cache outlived the idle gap:
  // the turn's own usage is summed over all its requests, which hides a cold start.
  on('turn.step', { index: 0 }, async function* ($, e, next) {
    const result = yield* next(e)
    const gap = pendingGap
    if (e.agentId === undefined && gap !== null) {
      pendingGap = null
      if (result.usage !== null) await learn($, gap, result.usage)
    }
    return result
  })

  on('turn.complete', async ($, e, next) => {
    const done = await next(e)
    if (e.agentId === undefined) {
      const now = await $.clock.now()
      await update($, lastActivity, () => now)
      arm($, (await read($, lifetime)).ms)
    }
    return done
  })

  // Starting a draft is when the estimate matters, so the idle time is brought up to date then,
  // after the key has landed.
  on('prompt.edit', { text: '' }, ($, e, next) => {
    $.clock.after(0, () => void refreshCold($))
    return next(e)
  })

  // The send goes at once and its answer passes on untouched; the estimate is refreshed after.
  on('prompt.submit', ($, e, next) => {
    $.clock.after(0, () => void refreshCold($))
    return next(e)
  }).catch(($, e, next) => next(e))

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    if (e.props.hasSurvey || (await read($, isQuiet))) return next(e)
    const list = await read($, crossings)
    if (!list.some(c => !c.isHidden)) return next(e)
    const text = bandText(list, await read($, readings), e.props.bodyColumns)
    if (text === null) return next(e)
    const beneath = await next(e)
    const { Box, Button, Text } = $.ui.resolve(e)
    return (
      <Box flexDirection="column">
        <Box>
          <Text wrap="truncate-end">{text} </Text>
          <Button key="hide" label="Hide" onPress={() => update($, crossings, all => all.map(c => ({ ...c, isHidden: true })))} />
        </Box>
        {beneath}
      </Box>
    )
  })

  on('command.run', { command: 'meter' }, async ($, e) => {
    const verb = e.args.trim()
    if (verb === 'quiet') {
      await update($, isQuiet, () => true)
      return { text: 'meter: the band is quiet for this session; the status line stays. /meter loud brings it back.' }
    }
    if (verb === 'loud') {
      await update($, isQuiet, () => false)
      return { text: 'meter: the band speaks again when a figure crosses a line.' }
    }
    if (verb !== '') return { text: USAGE }
    return { text: await report($) }
  })
}
