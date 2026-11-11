import { expect, mock, test } from 'claude-code/testing'
import type { On, PromptOrigin, RenderSurface } from 'claude-code'
import type { Plugin } from 'claude-code/testing'

const MINUTE = 60_000
const HOUR = 60 * MINUTE
const CWD = '/home/me/work'
const PRESENTATION = { isFullscreen: false, columns: 100 }
const PERSON = { kind: 'composer' } as const
const OUTPUT = { stdout: 'listed', stderr: '', interrupted: false }
// Set by another session: every session with the mod rests from 00:00 until 08:00.
const FLAG = { since: 0, until: 8 * HOUR, by: 'other' }

type World = { store?: Record<string, unknown>; canSleep?: boolean; wakeAfter?: number; surfaces?: RenderSurface[] }

/**
 * Stands for the engine and the machine beneath the mod. `sleep` waits on the mocked clock, and
 * after `wakeAfter` rounds of it another session's `/rest off` clears the flag.
 */
function engine(on: On, world: World = {}) {
  const store: Record<string, unknown> = { ...world.store }
  const seen = {
    store,
    probes: 0,
    rounds: 0,
    ran: [] as string[],
    entered: [] as string[],
    toasts: [] as string[],
    statuses: [] as (string | undefined)[],
    clockSleeps: 0,
    storeWrites: 0,
    tools: 0,
  }
  // An event takes one test hook, so the count wraps the mocked clock's own `clock.sleep` hook.
  const counting = ((event: string, ...rest: unknown[]) => {
    const hook = rest[rest.length - 1] as (...args: unknown[]) => unknown
    if (event === 'clock.sleep') {
      rest[rest.length - 1] = (...args: unknown[]) => {
        seen.clockSleeps += 1
        return hook(...args)
      }
    }
    return (on as (...args: unknown[]) => unknown)(event, ...rest)
  }) as unknown as On
  const clock = mock.clock(counting)
  on('store.get', (_, e) => ({ value: store[e.key] }))
  on('store.set', (_, e) => {
    seen.storeWrites += 1
    store[e.key] = e.value
    return { value: undefined }
  })
  on('store.delete', (_, e) => {
    delete store[e.key]
    return { value: undefined }
  })
  on('store.keys', () => ({ value: Object.keys(store) }))
  on('process.run', async (_, e) => {
    if (world.canSleep === false) return { deny: 'sleep: not found' }
    const seconds = Number(e.argv[1])
    if (e.argv[0] !== 'sleep' || !(seconds >= 0)) return { deny: `not sleep: ${e.argv.join(' ')}` }
    if (seconds === 0) seen.probes += 1
    else {
      seen.rounds += 1
      await clock.sleep(seconds * 1000)
      if (seen.rounds === world.wakeAfter) delete store.rest
    }
    return { value: { exitCode: 0, stdout: '', stderr: '', isStdoutTruncated: false, isStderrTruncated: false } }
  })
  on('session.surfaces', () => ({ value: world.surfaces ?? [] }))
  on('command.register', () => ({ value: { command: 'rest' } }))
  on('tool.register', () => {
    seen.tools += 1
    return { value: { tool: 'none' } }
  })
  on('session.start', (_, e) => ({ cwd: e.cwd }))
  on('tool.call', (_, e) => {
    seen.ran.push(e.agentId === undefined ? e.tool : `${e.tool} in ${e.agentId}`)
    return { result: OUTPUT }
  })
  on('prompt.submit', (_, e) => {
    seen.entered.push(e.text)
    return { text: e.text }
  })
  on('ui.status', (_, e) => {
    seen.statuses.push(e.text)
    return { value: undefined }
  })
  on('ui.toast', (_, e) => {
    seen.toasts.push(e.text)
    return { value: undefined }
  })
  return { seen, clock, status: () => seen.statuses[seen.statuses.length - 1] }
}

/** The clock text the mod writes, read the same way from the same environment. */
function at(ms: number): string {
  const d = new Date(ms)
  const hhmm = `${String(d.getHours()).padStart(2, '0')}:${String(d.getMinutes()).padStart(2, '0')}`
  return d.getTimezoneOffset() === 0 ? `${hhmm} UTC` : hhmm
}

async function begin($: any) {
  await $.session.start({ cwd: CWD, surface: 'terminal', isInteractive: true })
}

async function rest($: any, args: string, origin: PromptOrigin = PERSON) {
  const answer = await $.command.run({ command: 'rest', args, origin, presentation: PRESENTATION })
  // The mod answers `{ text }` alone, a string; `ref` is the engine's, added on the way back.
  expect(typeof answer.text).toBe('string')
  expect(Object.keys(answer).filter(key => key !== 'ref')).toEqual(['text'])
  return answer.text as string
}

function bash($: any, agentId?: string) {
  return $.tool.call({ tool: 'Bash', command: 'ls', tool_use_id: 'u1', ...(agentId === undefined ? {} : { agentId }) })
}

function prompt($: any, text: string, origin: PromptOrigin) {
  return $.prompt.submit({ text, wait: false, origin })
}

test('with no rest a tool call runs at once, and no sleep runs for it', async ($, on) => {
  const w = engine(on)
  await begin($)
  const answer = await bash($)
  expect(answer.result).toEqual(OUTPUT)
  expect(w.seen.ran).toEqual(['Bash'])
  expect(w.seen.probes).toBe(1)
  expect(w.seen.rounds).toBe(0)
  expect(w.status()).toBe(undefined)
})

test('a call waits while every session rests, then runs with its own result when another session ends the rest', async ($, on) => {
  const w = engine(on, { store: { rest: FLAG }, wakeAfter: 3 })
  await begin($)
  const call = bash($)
  await w.clock.settle()
  expect(w.seen.ran).toEqual([])
  expect(w.status()).toBe(`resting until ${at(8 * HOUR)}, 1 held`)
  await w.clock.advance(2000)
  await w.clock.advance(2000)
  expect(w.seen.ran).toEqual([])
  await w.clock.advance(2000)
  const answer = await call
  // The tool's own answer, untouched: the shape the engine takes from a hook that relays `next`.
  expect(answer.result).toEqual(OUTPUT)
  expect(answer.deny).toBe(undefined)
  expect(w.seen.ran).toEqual(['Bash'])
  expect(w.seen.rounds).toBe(3)
  expect(w.status()).toBe(undefined)
  expect(w.seen.clockSleeps).toBe(0)
})

test('a rest ends by itself when the clock passes its end, and the gate never waits on the clock', async ($, on) => {
  const w = engine(on, { store: { rest: { since: 0, until: 10 * MINUTE, by: 'other' } } })
  await begin($)
  expect(w.seen.toasts).toEqual([`rest: resting until ${at(10 * MINUTE)}`])
  const call = bash($)
  await w.clock.settle()
  await w.clock.advance(10 * MINUTE - 2000)
  expect(w.seen.ran).toEqual([])
  await w.clock.advance(2000)
  await call
  expect(w.seen.ran).toEqual(['Bash'])
  expect(w.seen.rounds).toBe(300)
  expect(w.seen.toasts).toContain('rest: awake')
  expect(w.seen.clockSleeps).toBe(0)
})

test('/rest rests every session for 8 hours, takes a length up to 24 hours, and refuses more', async ($, on) => {
  const w = engine(on)
  await begin($)
  expect(await rest($, '')).toContain(`until ${at(8 * HOUR)} (8 hours)`)
  expect(w.seen.store.rest).toEqual({ since: 0, until: 8 * HOUR, by: 'work' })
  await rest($, 'off')
  expect(w.seen.store.rest).toBe(undefined)
  expect(await rest($, '45m')).toContain('(45 minutes)')
  expect(w.seen.store.rest).toEqual({ since: 0, until: 45 * MINUTE, by: 'work' })
  await rest($, 'off')
  await rest($, '24h')
  expect(w.seen.store.rest).toEqual({ since: 0, until: 24 * HOUR, by: 'work' })
  await rest($, 'off')
  expect(await rest($, '25h')).toContain('at most 24 hours')
  expect(await rest($, '30s')).toContain('at least 1 minute')
  expect(await rest($, 'here 2d')).toContain('not a length')
  expect(await rest($, 'soon')).toContain('Usage:')
  expect(w.seen.store.rest).toBe(undefined)
})

test('/rest while a rest is in force says who rests and until when, and changes nothing', async ($, on) => {
  const w = engine(on, { store: { rest: FLAG } })
  await begin($)
  const said = await rest($, '')
  expect(said).toContain(`rests until ${at(8 * HOUR)}, since ${at(0)}, started from a session in other`)
  expect(said).toContain('Held in this session now: 0.')
  expect(w.seen.store.rest).toEqual(FLAG)
  expect(w.seen.storeWrites).toBe(0)
})

test('/rest here holds this session alone and writes nothing to the store', async ($, on) => {
  const w = engine(on)
  await begin($)
  expect(await rest($, 'here 5m')).toContain(`This session rests until ${at(5 * MINUTE)} (5 minutes)`)
  expect(w.seen.store.rest).toBe(undefined)
  expect(w.seen.storeWrites).toBe(0)
  const call = bash($)
  await w.clock.settle()
  expect(w.seen.ran).toEqual([])
  expect(await rest($, 'status')).toContain('Held in this session now: 1.')
  await w.clock.advance(5 * MINUTE - 2000)
  expect(w.seen.ran).toEqual([])
  await w.clock.advance(2000)
  await call
  expect(w.seen.ran).toEqual(['Bash'])
  expect(w.seen.storeWrites).toBe(0)
})

test('/rest here with no length rests this session for 8 hours', async ($, on) => {
  const w = engine(on)
  await begin($)
  expect(await rest($, 'here')).toContain(`This session rests until ${at(8 * HOUR)} (8 hours)`)
  expect(w.seen.storeWrites).toBe(0)
})

test('/rest off ends the shared rest and this session\'s own', async ($, on) => {
  const w = engine(on, { store: { rest: FLAG } })
  await begin($)
  await rest($, 'here 2h')
  const call = bash($)
  await w.clock.settle()
  expect(await rest($, 'off')).toContain('over for every session')
  expect(w.seen.store.rest).toBe(undefined)
  await w.clock.advance(2000)
  await call
  expect(w.seen.ran).toEqual(['Bash'])
  expect(w.status()).toBe(undefined)
  expect(await rest($, 'status')).toContain('No rest is in force.')
})

test('a prompt you type wakes this session alone, leaves the others resting, and the next rest holds it again', async ($, on) => {
  const w = engine(on, { store: { rest: FLAG } })
  await begin($)
  const call = bash($)
  await w.clock.settle()
  const entered = await prompt($, 'go on', PERSON)
  expect(entered).toEqual({ text: 'go on' })
  expect(w.seen.toasts).toContain('rest: this session woke; the others still rest')
  expect(w.seen.store.rest).toEqual(FLAG)
  await w.clock.advance(2000)
  expect((await call).result).toEqual(OUTPUT)
  expect(await rest($, 'status')).toContain('This session woke when you typed in it')
  expect(await bash($)).toEqual({ result: OUTPUT })
  await rest($, '1h')
  const again = bash($)
  await w.clock.settle()
  expect(w.seen.ran).toEqual(['Bash', 'Bash'])
  await rest($, 'off')
  await w.clock.advance(2000)
  await again
  expect(w.seen.ran).toEqual(['Bash', 'Bash', 'Bash'])
})

test('a prompt from a plugin or a schedule is held, and wakes nothing', async ($, on) => {
  const w = engine(on, { store: { rest: FLAG } })
  await begin($)
  const fromPlugin = prompt($, 'the build is green', { kind: 'plugin', name: 'watch' })
  const fromSchedule = prompt($, 'daily digest', { kind: 'scheduled-trigger' })
  const call = bash($)
  await w.clock.settle()
  await w.clock.advance(4000)
  expect(w.seen.entered).toEqual([])
  expect(w.seen.ran).toEqual([])
  expect(w.status()).toBe(`resting until ${at(8 * HOUR)}, 3 held`)
  expect(w.seen.toasts.some(t => t.includes('woke'))).toBe(false)
  delete w.seen.store.rest
  await w.clock.advance(2000)
  expect(await fromPlugin).toEqual({ text: 'the build is green' })
  expect(await fromSchedule).toEqual({ text: 'daily digest' })
  await call
  expect(w.seen.entered).toEqual(['the build is green', 'daily digest'])
  expect(w.seen.ran).toEqual(['Bash'])
})

test('a subagent\'s tool call is held too', async ($, on) => {
  const w = engine(on, { store: { rest: FLAG }, wakeAfter: 2 })
  await begin($)
  const call = bash($, 'a1')
  await w.clock.settle()
  await w.clock.advance(2000)
  expect(w.seen.ran).toEqual([])
  await w.clock.advance(2000)
  await call
  expect(w.seen.ran).toEqual(['Bash in a1'])
})

// Stands for the person's interrupt: it settles above the mod while the call is held.
const INTERRUPTER: Plugin = {
  name: 'interrupter',
  tier: 'prepend',
  register(on) {
    on('tool.call', async ($, e, next) => {
      void next(e).catch(() => undefined)
      await $.clock.sleep(5_000)
      return { deny: 'interrupted' }
    })
  },
}

test('an interrupted call stops waiting and never runs', { plugins: [INTERRUPTER] }, async ($, on) => {
  const w = engine(on, { store: { rest: FLAG } })
  await begin($)
  const call = bash($)
  await w.clock.settle()
  await w.clock.advance(5000)
  expect((await call).deny).toBe('interrupted')
  await w.clock.settle()
  // Let go at the interrupt itself, not when the round of `sleep` under way ends.
  expect(w.status()).toBe(`resting until ${at(8 * HOUR)}`)
  const rounds = w.seen.rounds
  await w.clock.advance(60_000)
  expect(w.seen.rounds).toBe(rounds)
  expect(w.seen.ran).toEqual([])
  expect(w.status()).toBe(`resting until ${at(8 * HOUR)}`)
  // The interrupter's own wait, and none of the gate's: the counter the other tests read sees them.
  expect(w.seen.clockSleeps).toBe(1)
})

// Interrupts at once: the call is abandoned before the mod's wait has begun.
const IMPATIENT: Plugin = {
  name: 'impatient',
  tier: 'prepend',
  register(on) {
    on('tool.call', ($, e, next) => {
      void next(e).catch(() => undefined)
      return { deny: 'interrupted at once' }
    })
  },
}

test('a call abandoned before its wait begins never waits', { plugins: [IMPATIENT] }, async ($, on) => {
  const w = engine(on, { store: { rest: FLAG } })
  await begin($)
  expect((await bash($)).deny).toBe('interrupted at once')
  await w.clock.advance(60_000)
  expect(w.seen.rounds).toBe(0)
  expect(w.seen.ran).toEqual([])
})

test('where sleep cannot run, /rest starts nothing and says why, and nothing waits', async ($, on) => {
  const w = engine(on, { canSleep: false })
  await begin($)
  for (const args of ['', '2h', 'here']) {
    const said = await rest($, args)
    expect(said).toContain('Nothing was started')
    expect(said).toContain('`sleep` could not run here')
  }
  expect(w.seen.store.rest).toBe(undefined)
  // Another session, one that can hold, started a rest: this one still cannot wait.
  w.seen.store.rest = FLAG
  expect(await bash($)).toEqual({ result: OUTPUT })
  expect(await rest($, 'status')).toContain('cannot hold a call')
  expect(w.status()).toBe(undefined)
})

test('a store value this version cannot read is no rest', async ($, on) => {
  const w = engine(on)
  await begin($)
  const malformed = ['yes', null, { since: '0', until: HOUR, by: 'x' }, { since: 0, until: 48 * HOUR, by: 'x' }, { since: HOUR, until: 0, by: 'x' }, { since: 0, until: HOUR }]
  for (const value of malformed) {
    w.seen.store.rest = value
    expect(await bash($)).toEqual({ result: OUTPUT })
    expect(await rest($, 'status')).toContain('No rest is in force.')
  }
  expect(w.seen.rounds).toBe(0)
})

test('the status line and toasts say when it rests, how much it holds, and when it wakes', async ($, on) => {
  const w = engine(on)
  await begin($)
  expect(w.seen.toasts).toEqual([])
  w.seen.store.rest = { since: 0, until: 2 * HOUR, by: 'other' }
  await w.clock.advance(5000)
  expect(w.seen.toasts).toEqual([`rest: resting until ${at(2 * HOUR)}`])
  expect(w.status()).toBe(`resting until ${at(2 * HOUR)}`)
  const call = bash($)
  await w.clock.settle()
  expect(w.status()).toBe(`resting until ${at(2 * HOUR)}, 1 held`)
  await w.clock.advance(5000)
  expect(w.seen.toasts.length).toBe(1)
  delete w.seen.store.rest
  await w.clock.advance(5000)
  await call
  expect(w.seen.toasts).toEqual([`rest: resting until ${at(2 * HOUR)}`, 'rest: awake'])
  expect(w.status()).toBe(undefined)
})

test('no tool is registered, and only a person can start or end a rest', async ($, on) => {
  const w = engine(on, { store: { rest: FLAG } })
  await begin($)
  for (const origin of [{ kind: 'plugin', name: 'other' }, { kind: 'peer' }, { kind: 'unclassified' }] as const) {
    expect(await rest($, 'off', origin)).toContain('Only you can')
    expect(await rest($, '2h', origin)).toContain('Only you can')
  }
  expect(w.seen.store.rest).toEqual(FLAG)
  expect(w.seen.storeWrites).toBe(0)
  await rest($, 'off')
  await rest($, '2h')
  expect(w.seen.tools).toBe(0)
})

test('in an app that hosts the session, a prompt you type there wakes it', async ($, on) => {
  const w = engine(on, { store: { rest: FLAG }, surfaces: ['desktop'] })
  await begin($)
  expect(await prompt($, 'from the app', { kind: 'sdk' })).toEqual({ text: 'from the app' })
  expect(w.seen.toasts).toContain('rest: this session woke; the others still rest')
})

test('a bare -p run\'s prompt waits for the rest to end', async ($, on) => {
  const w = engine(on, { store: { rest: FLAG }, wakeAfter: 1 })
  await begin($)
  const held = prompt($, 'from a script', { kind: 'sdk' })
  await w.clock.settle()
  expect(w.seen.entered).toEqual([])
  await w.clock.advance(2000)
  expect(await held).toEqual({ text: 'from a script' })
})
