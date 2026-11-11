import { atom, read, update } from 'claude-code'
import type { EngineInterface, PromptOrigin, Register } from 'claude-code'

import type { RestFlag, RestHere } from '../types'

type Dollar = EngineInterface

// Change the tag when RestHere's fields change: it declines what an earlier version kept.
const here = atom({ plugin: 'rest', key: 'here' } as const, null, { shape: 'here-1' })
const awake = atom({ plugin: 'rest', key: 'awake' } as const, null)
const HELD = { plugin: 'rest', key: 'held' } as const

export const MARK = 'rest:'
// The one key in the plugin's own store, a JSON file every session on the machine reads.
const KEY = 'rest'
const MINUTE = 60_000
const HOUR = 60 * MINUTE
const LONGEST = 24 * HOUR
const ROUND = ['sleep', '2']
const ROUND_TIMEOUT = 10_000
const TICK = 5_000
const USAGE = [
  'Usage:',
  '  /rest [45m|2h]         rest every session with this mod: 8 hours, or the length given (24h at most)',
  '  /rest here [45m|2h]    rest this session alone',
  '  /rest off              end the rest for every session, and this session\'s own',
  '  /rest status           who rests, since when, until when, and what this session holds',
  'A resting session holds its next tool call, and any prompt that is not yours, until the rest ends.',
  'A prompt you type in a resting session wakes that session alone.',
].join('\n')

// The gate reads these, not `$.state`: every `$.state.get` of one dispatch reads one moment, and a
// call held for hours is one dispatch. Each write goes to both; `session.start` reloads them.
let local: RestHere | null = null
let wokeAt: number | null = null
let held = 0
let holdable = false
let by = 'a session'
let shown = 'awake'
let ticker: { cancel: () => void } | undefined

/** `45m`, `2h`, `90s`: a length in milliseconds, or why it is not one. */
export function parseLength(text: string): number | string {
  const m = /^(\d+)(s|m|h)$/.exec(text)
  if (m === null) return `${text} is not a length: write one like 45m or 2h.`
  const ms = Number(m[1]) * { s: 1000, m: MINUTE, h: HOUR }[m[2] as 's' | 'm' | 'h']
  if (ms < MINUTE) return 'A rest lasts at least 1 minute.'
  if (ms > LONGEST) return 'A rest lasts at most 24 hours.'
  return ms
}

/** The time of day, in the zone this environment has; labelled UTC where it has none. */
export function clockTime(ms: number): string {
  const d = new Date(ms)
  const hhmm = `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
  return d.getTimezoneOffset() === 0 ? `${hhmm} UTC` : hhmm
}

function spoken(ms: number): string {
  const unit = (n: number, word: string) => `${n} ${word}${n === 1 ? '' : 's'}`
  return ms % HOUR === 0 ? unit(ms / HOUR, 'hour') : unit(Math.round(ms / MINUTE), 'minute')
}

/** The shared flag, when the store holds one this version can trust; anything else is no flag. */
export function asFlag(value: unknown): RestFlag | null {
  if (typeof value !== 'object' || value === null) return null
  const { since, until, by: from } = value as Record<string, unknown>
  if (typeof since !== 'number' || typeof until !== 'number' || typeof from !== 'string') return null
  if (!Number.isFinite(since) || !Number.isFinite(until) || until <= since || until - since > LONGEST) return null
  return { since, until, by: from }
}

/** The shared rest while it is in force, else null. A rest ends by itself at `until`. */
async function sharedRest($: Dollar, now: number): Promise<RestFlag | null> {
  const flag = asFlag(await $.store.get(KEY))
  return flag !== null && now < flag.until ? flag : null
}

/** When this session wakes, or null while it is awake. */
async function restingUntil($: Dollar): Promise<number | null> {
  if (!holdable) return null
  const now = await $.clock.now()
  const flag = await sharedRest($, now)
  const shared = flag !== null && flag.since !== wokeAt ? flag.until : null
  const own = local !== null && now < local.until ? local.until : null
  return shared === null ? own : own === null ? shared : Math.max(shared, own)
}

function paint($: Dollar, until: number | null) {
  $.ui.status(until === null ? undefined : `resting until ${clockTime(until)}${held > 0 ? `, ${held} held` : ''}`)
}

/** Repaints the status. On a change the timer raises one toast; a command or a wake has said it already. */
async function refresh($: Dollar, isTimer: boolean) {
  const until = await restingUntil($)
  paint($, until)
  const state = until === null ? 'awake' : `until ${until}`
  if (state === shown) return
  shown = state
  if (isTimer) $.ui.toast(until === null ? `${MARK} awake` : `${MARK} resting until ${clockTime(until)}`)
}

async function setHere($: Dollar, value: RestHere | null) {
  local = value
  // The new value does not depend on the old one, so overwriting is the whole of the change.
  await update($, here, () => value)
}

async function setAwake($: Dollar, value: number | null) {
  wokeAt = value
  await update($, awake, () => value)
}

/**
 * The module's count is the truth and `$.state` its copy, written with a plain set: `update` from a
 * dispatch that has waited for hours would read that dispatch's first moment, miss, and try again.
 */
async function count($: Dollar, change: number) {
  held += change
  await $.state.set(HELD, held)
  paint($, await restingUntil($))
}

function cannotHold($: Dollar) {
  if (!holdable) return
  holdable = false
  $.ui.toast(`${MARK} \`sleep\` stopped running, so this session no longer holds calls`)
}

async function canHold($: Dollar): Promise<boolean> {
  try {
    return (await $.process.run(['sleep', '0'], { timeoutMs: 5_000 })).exitCode === 0
  } catch {
    return false
  }
}

/**
 * Waits while this session rests, then says whether the dispatch is still wanted. Each round is a
 * `sleep 2` on the host: a hook's budget stops while a `$` call is in flight, but runs on through a
 * `$.clock` wait, so a rest of hours costs the hook a few milliseconds of its ten seconds.
 */
async function hold($: Dollar, signal: AbortSignal): Promise<'free' | 'abandoned'> {
  let quit = () => {}
  const abandoned = new Promise<'abandoned'>(resolve => {
    quit = () => resolve('abandoned')
    signal.addEventListener('abort', quit, { once: true })
  })
  await count($, 1)
  try {
    for (;;) {
      if (signal.aborted) return 'abandoned'
      const slept = $.process
        .run(ROUND, { timeoutMs: ROUND_TIMEOUT })
        .then(ran => ran.exitCode === 0, () => false)
      const how = await Promise.race([slept, abandoned])
      if (how === 'abandoned') return how
      if (!how) {
        cannotHold($)
        return 'free'
      }
      if ((await restingUntil($)) === null) return 'free'
    }
  } finally {
    signal.removeEventListener('abort', quit)
    await count($, -1)
  }
}

/**
 * The person at this session: the composer, Remote Control, or an app that hosts the session. The
 * desktop app runs it as an SDK host with its surface attached; a bare `claude -p` run draws on none.
 */
async function isPerson($: Dollar, origin: Pick<PromptOrigin, 'kind'>): Promise<boolean> {
  if (origin.kind === 'composer' || origin.kind === 'bridge') return true
  return origin.kind === 'sdk' && (await $.session.surfaces()).length > 0
}

/** The person typed in a resting session: this one wakes, until the next shared rest begins. */
async function wake($: Dollar) {
  const flag = await sharedRest($, await $.clock.now())
  if (flag !== null) await setAwake($, flag.since)
  if (local !== null) await setHere($, null)
  $.ui.toast(flag === null ? `${MARK} this session woke` : `${MARK} this session woke; the others still rest`)
  await refresh($, false)
}

async function report($: Dollar): Promise<string> {
  const now = await $.clock.now()
  const flag = await sharedRest($, now)
  const lines: string[] = []
  if (flag !== null) {
    lines.push(`Every session with this mod rests until ${clockTime(flag.until)}, since ${clockTime(flag.since)}, started from a session in ${flag.by}.`)
    if (flag.since === wokeAt) lines.push('This session woke when you typed in it; the others still rest.')
  }
  if (local !== null && now < local.until) lines.push(`This session alone rests until ${clockTime(local.until)}, since ${clockTime(local.since)}.`)
  if (lines.length === 0) lines.push('No rest is in force.')
  lines.push(`Held in this session now: ${held}.`)
  if (!holdable) lines.push('This session cannot hold a call, because `sleep` could not run here, so it does not rest.')
  return lines.join('\n')
}

async function start($: Dollar, scope: 'all' | 'here', length: number): Promise<string> {
  if (!holdable) return 'Nothing was started: this session cannot hold a call, because `sleep` could not run here.'
  const since = await $.clock.now()
  const until = since + length
  if (scope === 'here') {
    await setHere($, { since, until })
    await refresh($, false)
    return `This session rests until ${clockTime(until)} (${spoken(length)}). Its next tool call waits until then; a prompt you type here wakes it, and /rest off ends it.`
  }
  const flag: RestFlag = { since, until, by }
  await $.store.set(KEY, flag)
  await refresh($, false)
  return `Every session with this mod rests until ${clockTime(until)} (${spoken(length)}). Each holds its next tool call until then; a prompt you type in one wakes that one alone, and /rest off ends it for all.`
}

async function off($: Dollar): Promise<string> {
  const now = await $.clock.now()
  const wasShared = (await sharedRest($, now)) !== null
  const wasHere = local !== null && now < local.until
  await $.store.delete(KEY)
  if (local !== null) await setHere($, null)
  await refresh($, false)
  if (wasShared) return 'The rest is over for every session. What each held goes on within a few seconds.'
  return wasHere ? 'This session is awake again.' : 'No rest was in force.'
}

export const register: Register = on => {
  on('session.start', async ($, e, next) => {
    by = e.cwd.split(/[\\/]/).filter(Boolean).pop() ?? e.cwd
    await $.command.register({
      name: 'rest',
      description: 'Pause every session with this mod, with no model running, and resume them where they stopped',
      argumentHint: '[45m|2h] | here [45m|2h] | off | status',
      // A resting session is mid-turn, held at a tool call, and `/rest off` must reach it there.
      immediate: true,
    })
    holdable = await canHold($)
    local = await read($, here)
    wokeAt = await read($, awake)
    held = 0
    await $.state.set(HELD, 0)
    shown = 'awake'
    ticker?.cancel()
    // Only repaints and toasts: it reads the store and the clock, and makes no model call.
    ticker = $.clock.every(TICK, () => void refresh($, true))
    await refresh($, true)
    return next(e)
  })

  // No matcher: every tool, in every loop, subagents' included. No `.catch` either, on purpose: a
  // gate that fails lets the call run, so a broken rest never stops work.
  on('tool.call', async ($, e, next) => {
    if ((await restingUntil($)) === null) return next(e)
    if ((await hold($, next.signal)) === 'abandoned') return { deny: `${MARK} interrupted while this session rested` }
    return next(e)
  })

  on('prompt.submit', async ($, e, next) => {
    if (e.command?.name === 'rest' || (await restingUntil($)) === null) return next(e)
    if (await isPerson($, e.origin)) {
      await wake($)
      return next(e)
    }
    // A plugin, a schedule, a task's notification, a peer, a channel: nothing starts a turn in a rest.
    if ((await hold($, next.signal)) === 'abandoned') return { drop: `${MARK} abandoned while this session rested` }
    return next(e)
  })

  on('command.run', { command: 'rest' }, async ($, e) => {
    const words = e.args.trim().split(/\s+/).filter(Boolean)
    const verb = words[0] ?? ''
    const length = verb === 'here' ? parseLength(words[1] ?? '8h') : parseLength(verb === '' ? '8h' : verb)
    const isLength = /^\d+[smh]$/.test(verb)
    if (verb === 'status') return { text: await report($) }
    if (!(verb === '' || verb === 'off' || verb === 'here' || isLength) || words.length > (verb === 'here' ? 2 : 1)) {
      return { text: USAGE }
    }
    // No tool is registered either: a session never pauses its owner's others, or wakes itself.
    if (!(await isPerson($, e.origin))) return { text: `Only you can start or end a rest, by typing /rest. Nothing was changed.\n\n${await report($)}` }
    if (verb === 'off') return { text: await off($) }
    if (verb === '' && ((await restingUntil($)) !== null || (await sharedRest($, await $.clock.now())) !== null)) {
      return { text: `${await report($)}\n/rest <length> starts a new rest; /rest off ends this one.` }
    }
    if (typeof length === 'string') return { text: `${length}\n\n${USAGE}` }
    return { text: await start($, verb === 'here' ? 'here' : 'all', length) }
  })
}
