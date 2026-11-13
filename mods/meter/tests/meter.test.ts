import { expect, mock, test } from 'claude-code/testing'
import type { On, PromptEditInput } from 'claude-code'

import { bandText, crossed } from '../hooks/register'
import type { MeterReadings } from '../types'

const MIN = 60_000
const ORIGIN = { kind: 'composer' } as const
const PRESENTATION = { isFullscreen: false, columns: 100 }
const SURFACES = ['terminal', 'desktop', 'vscode', 'mobile'] as const
const HIDE_CELLS = 9

type Surface = (typeof SURFACES)[number]
type Limit = { kind: string; percentUsed: number }
type Figures = { context?: { tokens?: number; window: number; percent?: number }; rateLimits?: Limit[]; cost?: { usd: number } }
type Usage = { input_tokens: number; cache_read_input_tokens: number; cache_creation_input_tokens: number }

const CTX = (percent: number, tokens = percent * 2000) => ({ tokens, window: 200_000, percent })
// 182k tokens in a window large enough to stay under the context line
const BIG = { tokens: 182_000, window: 1_000_000, percent: 18 }
const H5 = (percentUsed: number) => ({ kind: 'five_hour', percentUsed })
const WK = (percentUsed: number) => ({ kind: 'seven_day', percentUsed })
const WARM: Usage = { input_tokens: 2_000, cache_read_input_tokens: 150_000, cache_creation_input_tokens: 3_000 }
const COLD: Usage = { input_tokens: 1_000, cache_read_input_tokens: 0, cache_creation_input_tokens: 160_000 }

const BAND = (bodyColumns: number) => ({
  plugin: 'meter',
  component: 'AbovePrompt',
  requestId: 'band',
  props: { hasSurvey: false, isCollapsed: false, isWorking: false, maxRows: 8, bodyColumns, scroll: { offset: 0, bodyRows: 8 }, view: {} },
}) as const

/** Stands for the engine beneath the mod, and counts what the mod must never do. */
function engine(on: On, start: Figures = {}) {
  const statuses: (string | undefined)[] = []
  const never = { tools: 0, models: 0, submits: 0, logs: 0, toasts: 0 }
  const clock = mock.clock(on, { now: 10 * 60 * MIN })
  const session = mock.session(on)
  let usage: Usage = WARM
  on('command.register', () => ({ value: { command: 'meter' } }))
  on('tool.register', () => {
    never.tools += 1
    return { deny: 'no tool under test' }
  })
  on('model.complete', () => {
    never.models += 1
    return { deny: 'no model under test' }
  })
  on('model.classify', () => {
    never.models += 1
    return { deny: 'no model under test' }
  })
  on('model.fork', () => {
    never.models += 1
    return { deny: 'no model under test' }
  })
  on('session.start', (_, e) => ({ cwd: e.cwd }))
  on('session.end', (_, e) => ({ sessionId: e.sessionId }))
  on('session.measure', (_, e) => ({ changed: e.changed }))
  on('session.usage', () => ({
    value: {
      startedAt: 0,
      context: start.context ?? { window: 200_000 },
      rateLimits: start.rateLimits ?? [],
      ...(start.cost === undefined ? {} : { cost: start.cost }),
    },
  }))
  on('turn.start', (_, e) => ({ turnId: e.turnId }))
  on('turn.complete', () => ({ text: 'done' }))
  on('turn.step', async function* (_, e) {
    return { turnId: e.turnId, index: e.index, answer: '', toolUses: [], stopReason: 'end_turn' as const, usage: { ...usage, output_tokens: 10, model: 'm' } }
  })
  on('prompt.submit', (_, e) => {
    if (e.origin.kind === 'plugin') never.submits += 1
    return { text: e.text, context: ['as the engine let it in'] }
  })
  on('prompt.edit', (_, e) => ({ text: e.text.slice(0, e.start) + e.inputText + e.text.slice(e.end), cursor: e.start + e.inputText.length }))
  // The engine's own band: what the mod must keep beneath its line.
  on('ui.render', { component: 'AbovePrompt' }, ($, e) => {
    const { Text } = $.ui.resolve(e)
    return Text({ children: 'beneath' })
  })
  on('ui.status', (_, e) => {
    statuses.push(e.text)
    return { value: undefined }
  })
  on('ui.log', () => {
    never.logs += 1
    return { value: undefined }
  })
  on('ui.toast', () => {
    never.toasts += 1
    return { value: undefined }
  })
  return { statuses, never, clock, session, setUsage: (u: Usage) => (usage = u) }
}

async function begin($: any) {
  await $.session.start({ cwd: '/work', surface: 'terminal', isInteractive: true })
}

async function measure($: any, f: Figures) {
  await $.session.measure({
    context: f.context ?? { window: 200_000 },
    rateLimits: f.rateLimits ?? [],
    ...(f.cost === undefined ? {} : { cost: f.cost }),
    changed: ['context', 'rateLimits', 'cost'],
  })
}

async function meter($: any, args = '') {
  return $.command.run({ command: 'meter', args, origin: ORIGIN, presentation: PRESENTATION })
}

/** A whole main-loop turn: its start, its first request, its end. */
async function turn($: any, clock: { settle: () => Promise<void> }, n: number, agentId?: string) {
  const loop = agentId === undefined ? {} : { agentId }
  await $.turn.start({ text: 'go', turnId: `t${n}`, ...loop })
  await clock.settle()
  const stream = $.turn.step({ turnId: `t${n}`, index: 0, model: 'm', messageCount: 2, ...loop })
  for await (const _ of stream) void _
  await stream.result
  await $.turn.complete({ answer: 'ok', durationMs: 1000, reason: 'answer', isAborted: false, turnId: `t${n}`, ...loop })
  await clock.settle()
}

/**
 * One key typed at the end of the draft. The kit raises `prompt.edit` on its `$`, though its
 * `EventCalls` table leaves the event out, so this one call is typed by hand.
 */
async function typing($: any, draft: string, key: string) {
  const prompt = $.prompt as { edit: (e: PromptEditInput) => Promise<unknown> }
  await prompt.edit({ origin: { kind: 'composer' }, text: draft, cursor: draft.length, start: draft.length, end: draft.length, inputText: key })
}

/** The band's line as drawn, or undefined when the mod left the band to the engine. */
async function band($: any, columns = 100, surface: Surface = 'terminal'): Promise<string | undefined> {
  const ui = await $.ui.mount({ ...BAND(columns), surface })
  const hide = await ui.find({ key: 'hide' })
  const text = hide === undefined ? undefined : (await ui.find({ type: 'Text' }))?.text.trim()
  await ui.unmount()
  return text
}

async function hide($: any, surface: Surface = 'terminal') {
  const ui = await $.ui.mount({ ...BAND(100), surface })
  await ui.press({ key: 'hide' })
  const after = await ui.find({ key: 'hide' })
  await ui.unmount()
  return after
}

test('the status line shows every unit, in order, from the first reading', async ($, on) => {
  const seen = engine(on, { context: CTX(41), rateLimits: [WK(18), H5(62)], cost: { usd: 1.84 } })
  await begin($)
  expect(seen.statuses.at(-1)).toBe('ctx 41% · 5h 62% · wk 18% · $1.84')
})

test('a unit with no reading is left out, never zeroed, and no reading at all clears the line', async ($, on) => {
  const seen = engine(on)
  await begin($)
  expect(seen.statuses.at(-1)).toBe(undefined)
  const all = { context: CTX(41), rateLimits: [H5(62), WK(18)], cost: { usd: 1.84 } }
  await measure($, { ...all, context: { window: 200_000 } })
  expect(seen.statuses.at(-1)).toBe('5h 62% · wk 18% · $1.84')
  await measure($, { ...all, rateLimits: [] })
  expect(seen.statuses.at(-1)).toBe('ctx 41% · $1.84')
  await measure($, { context: all.context, rateLimits: all.rateLimits })
  expect(seen.statuses.at(-1)).toBe('ctx 41% · 5h 62% · wk 18%')
  await measure($, {})
  expect(seen.statuses.at(-1)).toBe(undefined)
})

test('the context percent is worked out from tokens and window when the host gives no percent', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await measure($, { context: { tokens: 50_000, window: 200_000 } })
  expect(seen.statuses.at(-1)).toBe('ctx 25%')
})

test('a window of a kind it does not know is named by its kind', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await measure($, { rateLimits: [{ kind: 'spend_limit', percentUsed: 42.5 }, H5(10)] })
  expect(seen.statuses.at(-1)).toBe('5h 10% · spend_limit 42%')
  await measure($, { rateLimits: [{ kind: 'spend_limit', percentUsed: 81 }, H5(10)] })
  expect(await band($)).toBe('spend_limit at 81% (measured)')
})

test('below every line there is no band, and the engine keeps the band', async ($, on) => {
  const seen = engine(on, { context: { ...BIG, percent: 79 }, rateLimits: [H5(79), WK(79)], cost: { usd: 3 } })
  await begin($)
  await turn($, seen.clock, 1)
  await seen.clock.advance(4 * MIN)
  for (const surface of SURFACES) {
    const ui = await $.ui.mount({ ...BAND(100), surface })
    expect(await ui.find({ key: 'hide' })).toBe(undefined)
    // `keepsBeneath` is declared but this build's kit has none: the engine's own text says it.
    expect((await ui.find({ type: 'Text' }))?.text).toBe('beneath')
    await ui.unmount()
  }
})

for (const [name, at] of [
  ['context', (p: number) => ({ context: CTX(p) })],
  ['a 5-hour window', (p: number) => ({ rateLimits: [H5(p)] })],
  ['a weekly window', (p: number) => ({ rateLimits: [WK(p)] })],
] as const) {
  test(`${name} is announced once at 80%, not again while above, and again only after falling 5 points`, async ($, on) => {
    engine(on)
    await begin($)
    await measure($, at(79))
    expect(await band($)).toBe(undefined)
    await measure($, at(83))
    expect(await band($)).toContain('at 83% (measured)')
    expect(await hide($)).toBe(undefined)
    await measure($, at(85))
    expect(await band($)).toBe(undefined)
    await measure($, at(76))
    await measure($, at(81))
    expect(await band($)).toBe(undefined)
    await measure($, at(75))
    expect(await band($)).toBe(undefined)
    await measure($, at(80))
    expect(await band($)).toContain('at 80% (measured)')
  })
}

test('the context line says what a compaction costs, in the longest form that fits', async ($, on) => {
  engine(on, { context: CTX(83) })
  await begin($)
  expect(await band($, 120)).toBe('context at 83% (measured): a compaction is near; a handoff note written now costs less than one after it')
  expect(await band($, 100)).toBe('context at 83% (measured): a compaction is near; a handoff note costs less before it')
  expect(await band($, 40)).toBe('context 83% (measured)')
})

test('the 95% line comes after the 80% line and takes its place', async ($, on) => {
  engine(on)
  await begin($)
  await measure($, { rateLimits: [H5(82)] })
  expect(await band($)).toBe('5-hour window at 82% (measured)')
  await hide($)
  await measure($, { rateLimits: [H5(90)] })
  expect(await band($)).toBe(undefined)
  await measure($, { rateLimits: [H5(96)] })
  expect(await band($)).toBe('5-hour window at 96% (measured)')
  await measure($, { rateLimits: [H5(91)] })
  expect(await band($)).toBe('5-hour window at 91% (measured)')
  await measure($, { rateLimits: [H5(90)] })
  expect(await band($)).toBe(undefined)
  await measure($, { rateLimits: [H5(95)] })
  expect(await band($)).toBe('5-hour window at 95% (measured)')
})

test('once 95% is announced, falling back under it does not announce 80% a second time', async ($, on) => {
  engine(on)
  await begin($)
  await measure($, { rateLimits: [H5(82)] })
  expect(await band($)).toBe('5-hour window at 82% (measured)')
  await measure($, { rateLimits: [H5(96)] })
  expect(await band($)).toBe('5-hour window at 96% (measured)')
  await measure($, { rateLimits: [H5(90)] })
  expect(await band($)).toBe(undefined)
})

test('a window already announced at 80% reaches 95%: the 95% line is shown, the 80% one hidden, and nothing is left to count', () => {
  const at = (percentUsed: number): MeterReadings => ({ context: null, windows: [{ kind: 'five_hour', percentUsed, at: 0 }], cost: null, cold: null })
  const first = crossed([], at(82))
  expect(first).toEqual([{ id: 'five_hour@80', source: 'five_hour', isHidden: false }])
  const next = crossed(first, at(96))
  expect(next).toEqual([
    { id: 'five_hour@80', source: 'five_hour', isHidden: true },
    { id: 'five_hour@95', source: 'five_hour', isHidden: false },
  ])
  const text = bandText(next, at(96), 100)
  expect(text).toBe('5-hour window at 96% (measured)')
  expect(text).not.toContain('more')
})

test('a first reading past both lines is one line, not two', async ($, on) => {
  engine(on, { rateLimits: [WK(97)] })
  await begin($)
  expect(await band($)).toBe('weekly window at 97% (measured)')
})

test('Hide hides until the next crossing, and that crossing shows alone', async ($, on) => {
  engine(on, { context: CTX(83) })
  await begin($)
  expect(await band($)).toContain('context at 83%')
  for (const surface of SURFACES) expect(await band($, 100, surface)).toContain('context at 83%')
  expect(await hide($, 'mobile')).toBe(undefined)
  for (const surface of SURFACES) expect(await band($, 100, surface)).toBe(undefined)
  await measure($, { context: CTX(84), rateLimits: [H5(81)] })
  expect(await band($)).toBe('5-hour window at 81% (measured)')
})

test('two crossings at once show the newest, and say there is more', async ($, on) => {
  engine(on, { context: CTX(83), rateLimits: [H5(85)] })
  await begin($)
  expect(await band($, 100)).toBe('5-hour window at 85% (measured) (+1 more in /meter)')
  expect(await band($, 40)).toBe('5h 85% (measured) +1')
})

test('/meter quiet keeps the status line and silences the band; /meter loud brings it back', async ($, on) => {
  const seen = engine(on, { context: CTX(83) })
  await begin($)
  expect(await band($)).toContain('context at 83%')
  const quiet = await meter($, 'quiet')
  expect(quiet.text).toContain('quiet for this session')
  expect(await band($)).toBe(undefined)
  await measure($, { context: CTX(84), rateLimits: [H5(90)] })
  expect(seen.statuses.at(-1)).toBe('ctx 84% · 5h 90%')
  expect(await band($)).toBe(undefined)
  expect((await meter($)).text).toContain('band: quiet')
  await meter($, 'loud')
  expect(await band($)).toContain('5-hour window at 90% (measured)')
})

test('the cold-cache line waits for the lifetime, reads the mocked clock, updates when typing starts, and leaves the send alone', async ($, on) => {
  const seen = engine(on, { context: BIG })
  await begin($)
  await turn($, seen.clock, 1)
  await seen.clock.advance(4 * MIN)
  expect(await band($)).toBe(undefined)
  await seen.clock.advance(1 * MIN)
  expect(await band($, 120)).toBe('cache likely cold (estimate: idle 5 min): the next send re-reads about 182k tokens at full price')
  await seen.clock.advance(2 * MIN)
  expect(await band($, 120)).toContain('idle 5 min')
  await typing($, '', 'h')
  await seen.clock.settle()
  expect(await band($, 120)).toContain('idle 7 min')
  // Only the first key of a draft refreshes it: the rest of the typing costs nothing.
  await seen.clock.advance(1 * MIN)
  await typing($, 'h', 'i')
  await seen.clock.settle()
  expect(await band($, 120)).toContain('idle 7 min')
  const sent = await $.prompt.submit({ text: 'carry on', wait: false, origin: ORIGIN })
  expect(sent).toEqual({ text: 'carry on', context: ['as the engine let it in'] })
  await $.turn.start({ text: 'carry on', turnId: 't2' })
  await seen.clock.settle()
  expect(await band($)).toBe(undefined)
  expect(seen.statuses.at(-1)).toBe('ctx 18%')
})

for (const [tokens, isShown] of [[49_999, false], [50_000, true]] as const) {
  test(`the cold-cache line needs 50,000 tokens in context: ${tokens} ${isShown ? 'shows it' : 'does not'}`, async ($, on) => {
    const seen = engine(on, { context: { tokens, window: 1_000_000, percent: 5 } })
    await begin($)
    await turn($, seen.clock, 1)
    await seen.clock.advance(6 * MIN)
    expect((await band($)) !== undefined).toBe(isShown)
  })
}

test('the cold-cache line needs the idle time: a turn running or a fresh session shows none', async ($, on) => {
  const seen = engine(on, { context: BIG })
  await begin($)
  await seen.clock.advance(30 * MIN)
  await typing($, '', 'h')
  await seen.clock.settle()
  expect(await band($)).toBe(undefined)
  await $.turn.start({ text: 'long', turnId: 't1' })
  await seen.clock.advance(30 * MIN)
  expect(await band($)).toBe(undefined)
})

test('starting a draft before the lifetime has passed shows no cold-cache line', async ($, on) => {
  const seen = engine(on, { context: BIG })
  await begin($)
  await turn($, seen.clock, 1)
  await seen.clock.advance(2 * MIN)
  await typing($, '', 'h')
  await seen.clock.settle()
  expect(await band($)).toBe(undefined)
})

test('the lifetime becomes 60 minutes, observed, when the cache outlives a longer gap, and falls back to 5', async ($, on) => {
  const seen = engine(on, { context: BIG })
  await begin($)
  await turn($, seen.clock, 1)
  expect((await meter($)).text).toContain('cache lifetime: 5 min (assumed:')
  await seen.clock.advance(12 * MIN)
  seen.setUsage(WARM)
  await turn($, seen.clock, 2)
  const learned = (await meter($)).text
  expect(learned).toContain('cache lifetime: 60 min (observed: a turn after 12 min idle still read 97% of its input from the cache)')
  await seen.clock.advance(30 * MIN)
  expect(await band($)).toBe(undefined)
  await seen.clock.advance(30 * MIN)
  expect(await band($)).toContain('cache likely cold (estimate: idle 60 min)')
  await $.turn.start({ text: 'again', turnId: 't3' })
  await seen.clock.settle()
  await seen.clock.advance(1 * MIN)
  await $.turn.complete({ answer: 'ok', durationMs: 1000, reason: 'answer', isAborted: false, turnId: 't3' })
  await seen.clock.advance(20 * MIN)
  seen.setUsage(COLD)
  await turn($, seen.clock, 4)
  expect((await meter($)).text).toContain('cache lifetime: 5 min (assumed: a turn after 20 min idle wrote 99% of its input fresh')
})

test('a first turn, a subagent, or a small request teaches the lifetime nothing', async ($, on) => {
  const seen = engine(on, { context: BIG })
  await begin($)
  seen.setUsage(WARM)
  await turn($, seen.clock, 1)
  await seen.clock.advance(12 * MIN)
  expect(await band($)).toContain('cache likely cold')
  // A subagent's turn neither ends the main idle stretch nor speaks for its cache.
  await turn($, seen.clock, 2, 'agent-1')
  expect(await band($)).toContain('cache likely cold')
  expect((await meter($)).text).toContain('cache lifetime: 5 min (assumed:')
  seen.setUsage({ input_tokens: 1_000, cache_read_input_tokens: 40_000, cache_creation_input_tokens: 1_000 })
  await turn($, seen.clock, 3)
  expect((await meter($)).text).toContain('cache lifetime: 5 min (assumed:')
})

test('/meter labels each figure measured, with when, or estimate, with what from', async ($, on) => {
  const seen = engine(on, { context: CTX(41, 82_000), rateLimits: [H5(62.5), WK(18)], cost: { usd: 1.84 } })
  await begin($)
  await turn($, seen.clock, 1)
  await seen.clock.advance(2 * MIN)
  const answer = await meter($)
  // The shape the engine takes for a command's own answer: a string `text`, no notes for the
  // model, no exit code. (`ref` is the engine's stamp on the run, not the mod's.)
  expect(typeof answer.text).toBe('string')
  expect(answer.context).toBe(undefined)
  expect(answer.exitCode).toBe(undefined)
  expect(answer.isTextDisplayOnly).toBe(undefined)
  const lines = answer.text.split('\n')
  expect(lines).toContain('context: 41% of a 200k window, 82k tokens (measured 2 min ago)')
  expect(lines).toContain('5-hour window: 62% (measured 2 min ago)')
  expect(lines).toContain('weekly window: 18% (measured 2 min ago)')
  expect(lines).toContain('cost: $1.84 (measured 2 min ago)')
  expect(lines).toContain('cache: likely warm (estimate from idle 2 min against the 5 min lifetime)')
  expect(lines.some((l: string) => l.startsWith('cache lifetime: 5 min (assumed:'))).toBe(true)
  const figures = lines.filter((x: string) => /^(context|5-hour window|weekly window|cost|cache):/.test(x))
  expect(figures).toHaveLength(5)
  for (const l of figures) {
    expect(/\(measured \d+( min|s) ago\)|\(estimate from /.test(l), l).toBe(true)
  }
  await seen.clock.advance(4 * MIN)
  expect((await meter($)).text).toContain('cache: likely cold (estimate from idle 6 min against the 5 min lifetime; the next send re-reads about 82k tokens)')
})

test('/meter says plainly which figures it does not have', async ($, on) => {
  engine(on)
  await begin($)
  const text = (await meter($)).text
  expect(text).toContain('context: no reading yet')
  expect(text).toContain('rate limits: none reported')
  expect(text).toContain('cost: no figure')
  expect(text).toContain('cache: no estimate')
  expect(text).not.toContain('0%')
  expect((await meter($, 'bogus')).text).toContain('Usage:')
})

test('the band draws on every surface and fits 40 and 100 columns', async ($, on) => {
  const seen = engine(on, { context: { ...BIG, tokens: 182_000 } })
  await begin($)
  await turn($, seen.clock, 1)
  await seen.clock.advance(9 * MIN)
  await measure($, { context: { tokens: 182_000, window: 200_000, percent: 91 }, rateLimits: [H5(96), WK(85)] })
  for (const surface of SURFACES) {
    for (const columns of [40, 100]) {
      const ui = await $.ui.mount({ ...BAND(columns), surface })
      expect(await ui.drawn()).toMatchObject({ type: 'Box' })
      const button = await ui.find({ type: 'Button' })
      expect(button?.key).toBe('hide')
      expect(button?.props.label).toBe('Hide')
      const text = (await ui.find({ type: 'Text' }))?.text.trim() ?? ''
      expect(text.length).toBeGreaterThan(0)
      expect(text.length + HIDE_CELLS).toBeLessThanOrEqual(columns)
      expect(await ui.find({ type: 'Text', text: 'beneath' })).toBeDefined()
      await ui.unmount()
    }
  }
})

test('/clear forgets the old context and its cache estimate', async ($, on) => {
  const seen = engine(on, { context: BIG, rateLimits: [H5(10)] })
  await begin($)
  await turn($, seen.clock, 1)
  await seen.clock.advance(6 * MIN)
  expect(await band($)).toContain('cache likely cold')
  await $.session.end({ reason: 'clear', sessionId: 's1', resume: { id: 's1' } })
  expect(await band($)).toBe(undefined)
  expect(seen.statuses.at(-1)).toBe('5h 10%')
  await seen.clock.advance(60 * MIN)
  expect(await band($)).toBe(undefined)
})

test('a value stored under an older shape tag is declined', async ($, on) => {
  on('state.get', { plugin: 'meter', key: 'readings' }, () => ({
    value: { value: { shape: 'readings-0', value: { context: { percent: 90, tokens: 180_000, window: 200_000, at: 0 }, windows: [], cost: null, cold: null } }, version: 1 },
  }))
  engine(on)
  expect((await meter($)).text).toContain('context: no reading yet')
})

test('a value stored under the current shape tag is read, so the test above can tell', async ($, on) => {
  on('state.get', { plugin: 'meter', key: 'readings' }, () => ({
    value: { value: { shape: 'readings-1', value: { context: { percent: 90, tokens: 180_000, window: 200_000, at: 0 }, windows: [], cost: null, cold: null } }, version: 1 },
  }))
  engine(on)
  expect((await meter($)).text).toContain('context: 90%')
})

test('the mod registers no tool, calls no model, and writes nothing to the conversation', async ($, on) => {
  const seen = engine(on, { context: BIG, rateLimits: [H5(50)], cost: { usd: 0.5 } })
  await begin($)
  await turn($, seen.clock, 1)
  await measure($, { context: { tokens: 182_000, window: 200_000, percent: 91 }, rateLimits: [H5(96)], cost: { usd: 2 } })
  await seen.clock.advance(12 * MIN)
  await typing($, '', 'x')
  await seen.clock.settle()
  expect(await band($)).toBeDefined()
  await meter($)
  await meter($, 'quiet')
  await meter($, 'loud')
  await hide($)
  await $.prompt.submit({ text: 'go', wait: false, origin: ORIGIN })
  await turn($, seen.clock, 2)
  await seen.clock.advance(90 * MIN)
  expect(seen.never).toEqual({ tools: 0, models: 0, submits: 0, logs: 0, toasts: 0 })
  expect(seen.session.appended()).toHaveLength(0)
})

test('the /meter verbs ignore case: Quiet and LOUD act, and do not print usage', async ($, on) => {
  engine(on, { context: CTX(83) })
  await begin($)
  expect((await meter($, 'Quiet')).text).toContain('quiet for this session')
  expect(await band($)).toBe(undefined)
  expect((await meter($, ' LOUD ')).text).toContain('speaks again')
  expect(await band($)).toContain('context at 83%')
  expect((await meter($, 'QUIET')).text).toContain('quiet for this session')
  expect((await meter($, 'nonsense')).text).toContain('Usage:')
})

test('/meter floors a window percentage as the status line does', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await measure($, { rateLimits: [{ kind: 'five_hour', percentUsed: 42.7 }] })
  expect(seen.statuses.at(-1)).toBe('5h 42%')
  const text = (await meter($)).text
  expect(text).toContain('5-hour window: 42%')
  expect(text).not.toContain('42.7')
})

test('a crossing is not marked seen by drawing the band, collapsed or not: it stays until Hide or a newer crossing', async ($, on) => {
  engine(on, { context: CTX(83) })
  await begin($)
  const collapsed = { ...BAND(100), surface: 'terminal' as const, props: { ...BAND(100).props, isCollapsed: true } }
  for (let i = 0; i < 2; i++) {
    const ui = await $.ui.mount(collapsed)
    await ui.unmount()
  }
  expect(await band($)).toContain('context at 83%')
})
