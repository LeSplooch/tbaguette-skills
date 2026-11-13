import { expect, mock, test } from 'claude-code/testing'
import type { On, RenderElement, RenderSurface } from 'claude-code'
import type { Plugin } from 'claude-code/testing'

import {
  ANATOMY,
  CANDIDATES,
  CLOSE,
  COUNT_PREFIX,
  GATE_KEY,
  MARK,
  NOTHING,
  STORE_PREFIX,
  age,
  announced,
  closeness,
  fallen,
  bandPlan,
  hash,
  landing,
  provenance,
  recordText,
  refusal,
  relayText,
  renumbered,
  storeKey,
  takeText,
} from '../hooks/register'
import type { Move } from '../types'

const ROOT = '/repo'
const SESSION = 'sess-1'
const DAY = 86_400_000
const HOUR = 3_600_000
const NOW = 1_000 * DAY
const PRESENTATION = { isFullscreen: false, columns: 100 }
const PERSON = { kind: 'composer' } as const
const SURFACES = ['terminal', 'desktop', 'vscode', 'mobile'] as const
const SET_ASIDE = 'mcp__moves__set_aside'
const HARVEST = 'mcp__moves__harvest'
const BAND = {
  hasSurvey: false,
  isCollapsed: false,
  isWorking: false,
  maxRows: 8,
  bodyColumns: 80,
  scroll: { offset: 0, bodyRows: 8 },
  view: {},
}

type Knobs = {
  session: string
  surfaces: RenderSurface[]
  // null: not a git working tree
  top: string | null
  answers: Record<string, string>
  isDismissed: boolean
  isEditFailing: boolean
  isStoreBroken: boolean
  isDeleteBroken: boolean
  fill: { isFilled: boolean; refusal?: string }
  // what a settings hook beneath the mod answers the end of a turn with
  below: { block?: string }
  // a hook beneath drops every prompt a plugin sends
  isRelayDropped: boolean
}

/** A move as another session, or an earlier version, left it in the store. */
function stored(fields: Partial<Move> & { id: number; move: string }): Record<string, unknown> {
  return {
    shape: 'moves-1',
    uid: `old:${fields.id}`,
    kind: 'found',
    cost: 'ten minutes',
    root: ROOT,
    session: 'sess-0',
    at: NOW - 3 * DAY,
    offered: 0,
    ...fields,
  }
}

function entry(value: Record<string, unknown>): [string, unknown] {
  return [storeKey({ root: String(value.root), uid: String(value.uid) }), value]
}

/** Stands for the engine beneath the mod: what it was asked is kept for the test to read. */
function engine(on: On, entries: [string, unknown][] = [], set: Partial<Knobs> = {}) {
  const knobs: Knobs = {
    session: SESSION,
    surfaces: ['desktop'],
    top: ROOT,
    answers: {},
    isDismissed: false,
    isEditFailing: false,
    isStoreBroken: false,
    isDeleteBroken: false,
    fill: { isFilled: true },
    below: {},
    isRelayDropped: false,
    ...set,
  }
  // The store by hand rather than mock.store, so a test can read what the mod kept.
  const store = new Map<string, unknown>(entries)
  const prompts: { text: string; origin: unknown; context?: readonly string[] }[] = []
  const fills: string[] = []
  const statuses: (string | undefined)[] = []
  const toasts: string[] = []
  const registered: { name?: string; isDeferred?: unknown }[] = []
  const git: { argv: string[]; cwd?: string }[] = []
  const clock = mock.clock(on, { now: NOW })
  on('store.get', (_, e) => ({ value: store.get(e.key) }))
  on('store.set', (_, e) => {
    if (knobs.isStoreBroken) return { deny: 'the store is unreachable' }
    store.set(e.key, JSON.parse(JSON.stringify(e.value)))
    return { value: undefined }
  })
  on('store.delete', (_, e) => {
    if (knobs.isDeleteBroken) return { deny: 'the store is unreachable' }
    store.delete(e.key)
    return { value: undefined }
  })
  on('store.keys', () => ({ value: [...store.keys()] }))
  on('command.register', () => ({ value: { command: 'moves' } }))
  on('tool.register', (_, e) => {
    registered.push(e as { name?: string; isDeferred?: unknown })
    return { value: { tool: 'x' } }
  })
  on('session.start', (_, e) => ({ cwd: e.cwd }))
  on('session.id', () => ({ value: knobs.session }))
  on('session.cwd', () => ({ value: ROOT }))
  on('session.end', (_, e) => ({ sessionId: e.sessionId }))
  on('session.surfaces', () => ({ value: knobs.surfaces }))
  on('process.run', (_, e) => {
    git.push({ argv: [...e.argv], cwd: e.init?.cwd })
    const isRepo = knobs.top !== null
    return { value: { exitCode: isRepo ? 0 : 128, stdout: isRepo ? `${knobs.top}\n` : '', stderr: '', isStdoutTruncated: false, isStderrTruncated: false } }
  })
  on('turn.start', (_, e) => ({ turnId: e.turnId }))
  on('turn.complete', () => ({ text: 'done' }))
  on('tool.call', (_, e) => {
    const input = e as { tool: string; questions?: unknown }
    if (input.tool === 'AskUserQuestion') {
      if (knobs.isDismissed) return { deny: 'The user dismissed the question.' }
      return { result: { questions: input.questions, answers: knobs.answers } }
    }
    if (knobs.isEditFailing) return { isError: true as const, result: 'String to replace not found', text: 'String to replace not found' }
    return { result: { ok: true } }
  })
  on('prompt.submit', (_, e) => {
    if (knobs.isRelayDropped && e.origin.kind === 'plugin') return { drop: 'a hook dropped it' }
    prompts.push({ text: e.text, origin: e.origin, context: e.context })
    return { text: e.text, context: e.context }
  })
  on('prompt.fill', (_, e) => {
    fills.push(e.text)
    return knobs.fill as { isFilled: boolean }
  })
  on('classic.Stop', () => knobs.below)
  on('ui.status', (_, e) => {
    statuses.push(e.text)
    return { value: undefined }
  })
  on('ui.toast', (_, e) => {
    toasts.push(e.text)
    return { value: undefined }
  })
  // What the engine draws in the band when the mod passes.
  on('ui.render', { component: 'AbovePrompt' }, ($, e) => h($.ui.resolve(e).Text, null, 'engine band') as RenderElement)
  /** The moves the store holds, by number. */
  const kept = () =>
    [...store.entries()]
      .filter(([key]) => key.startsWith(STORE_PREFIX))
      .map(([, value]) => value as Move)
      .sort((a, b) => a.id - b.id)
  return { knobs, store, prompts, fills, statuses, toasts, registered, git, clock, kept }
}

let calls = 0

async function begin($: any, cwd = ROOT) {
  await $.session.start({ cwd, surface: 'desktop', isInteractive: true })
}

async function call($: any, input: Record<string, unknown>) {
  return $.tool.call({ tool_use_id: `u${++calls}`, ...input })
}

async function aside($: any, kind: string, move: string, cost = 'ten minutes, and the importer keeps failing on it until then') {
  return call($, { tool: SET_ASIDE, kind, move, cost })
}

async function harvest($: any, done?: number[]) {
  return call($, { tool: HARVEST, ...(done === undefined ? {} : { done }) })
}

async function edit($: any, tool = 'Edit', more: Record<string, unknown> = {}) {
  return call($, { tool, file_path: `${ROOT}/a.ts`, old_string: 'x', new_string: 'y', ...more })
}

async function ask($: any, labels: string[], more: Record<string, unknown> = {}) {
  const options = labels.map(label => ({ label, description: 'what it costs' }))
  return call($, { tool: 'AskUserQuestion', questions: [{ question: 'What next?', header: 'Next move', multiSelect: false, options }], ...more })
}

async function say($: any, text: string, origin: object = PERSON, extra: object = {}) {
  return $.prompt.submit({ text, origin, wait: false, ...extra })
}

async function turn($: any, n: number) {
  await $.turn.start({ text: 'go', turnId: `t${n}` })
}

async function endTurn($: any, n: number) {
  await $.turn.complete({ answer: 'ok', durationMs: 1000, reason: 'answer', isAborted: false, turnId: `t${n}` })
}

async function stop($: any, fields: object = {}) {
  return $.classic.Stop({ stop_hook_active: false, ...fields })
}

async function moves($: any, args: string, origin: object = PERSON) {
  return $.command.run({ command: 'moves', args, origin, presentation: PRESENTATION })
}

async function band($: any, surface: (typeof SURFACES)[number] = 'desktop', bodyColumns = 80) {
  return $.ui.mount({ plugin: 'moves', surface, component: 'AbovePrompt', props: { ...BAND, bodyColumns } })
}

/** A person's turn that changed a file and is about to end. */
async function changed($: any, n: number) {
  await say($, 'fix the importer')
  await turn($, n)
  await edit($)
}

// ---------------------------------------------------------------------------
// The pieces
// ---------------------------------------------------------------------------

test('an option says a move when its words are the same, or the same said shorter', () => {
  expect(closeness('Cover the retry path', 'Cover the retry path')).toBe(1)
  expect(closeness('Cover the retry path (Recommended)', 'cover the retry path')).toBe(1)
  expect(closeness('Cover retry path', 'Cover the retry path')).toBe(1)
  expect(closeness('Cover the retry path', 'Cover the retry path with one test')).toBeGreaterThanOrEqual(CLOSE)
  expect(closeness("Split the parser's two jobs", 'Split the parser')).toBeGreaterThanOrEqual(CLOSE)
  expect(closeness('Ship as is', 'Cover the retry path')).toBeLessThan(CLOSE)
  expect(closeness('Handle empty input', 'Handle the timeout')).toBeLessThan(CLOSE)
  expect(closeness('Stop here', 'Stop')).toBeLessThan(CLOSE)
  expect(closeness('', 'Cover the retry path')).toBe(0)
  expect(closeness('...', '???')).toBe(0)
})

test('a move the model sets aside needs a kind, a label, and a cost that is not the label again', () => {
  const ok = (kind: unknown, move: unknown, cost: unknown, list: Move[] = []) => refusal(kind, move, cost, list, SESSION)
  expect(ok('found', 'Cover the retry path', 'one test; the review flagged it and nobody closed it')).toBe(null)
  expect(ok('asked', 'Cover the retry path', 'one test')).toContain('surrendered, ruling, deferred, found')
  expect(ok(undefined, 'Cover the retry path', 'one test')).toContain('The kind is one of')
  expect(ok('found', '   ', 'one test')).toBe('A move needs words: the action, verb first.')
  expect(ok('found', 7, 'one test')).toBe('A move needs words: the action, verb first.')
  expect(ok('found', 'Cover the retry path', ' ')).toContain('what the move costs')
  expect(ok('found', 'x'.repeat(81), 'one test')).toBe('A move is a label of at most 80 characters, and this one has 81. Put the rest in the cost.')
  expect(ok('found', 'x'.repeat(80), 'one test')).toBe(null)
  expect(ok('found', 'Cover the retry path', 'y'.repeat(301))).toBe('The cost is at most 300 characters, and this one has 301.')
  expect(ok('found', 'Cover the retry path', 'cover the path of the retry')).toContain('The cost says the move again.')
  const on = [stored({ id: 4, move: 'Cover the retry path' })] as unknown as Move[]
  expect(ok('ruling', 'cover  retry path', 'one test', on)).toBe('That is already on the table as #4.')
  const full = Array.from({ length: 12 }, (_, i) => stored({ id: i + 1, move: `Move number ${i + 1}`, session: SESSION })) as unknown as Move[]
  expect(ok('found', 'One more move', 'one test', full)).toContain('12 moves are already set aside in this session.')
  // The person's own moves and another session's do not count against this session's twelve.
  const others = full.map((m, i) => ({ ...m, session: i % 2 === 0 ? 'sess-0' : SESSION, kind: i % 2 === 0 ? m.kind : ('asked' as const) }))
  expect(ok('found', 'One more move', 'one test', others)).toBe(null)
})

test('a closing sentence in the first person and the future tense is an announcement', () => {
  expect(announced('The parser is fixed and the suite is green. Tonight I\'ll rebuild the runner.')).toBe("Tonight I'll rebuild the runner.")
  expect(announced('Done.\n\n- I will take the retry path next.')).toBe('I will take the retry path next.')
  expect(announced('That is my first job tomorrow: I’m going to split the parser.')).toContain('split the parser')
  expect(announced('Fixed. Next, I add the test. We\'ll see "what" breaks.')).toBe("We'll see 'what' breaks.")
  expect(announced("I'll look at the logs first. The fix is in and the suite is green. Nothing else changed.")).toBe(null)
  expect(announced("The fix is in. I'll wait for your answer.")).toBe(null)
  expect(announced("The fix is in. Shall I say what I'll do next?")).toBe(null)
  expect(announced('The fix is in and the suite is green.')).toBe(null)
  expect(announced('')).toBe(null)
  expect(announced(`Done. I'll ${'rebuild '.repeat(40)}the runner.`)?.length).toBe(140)
})

test('ages, the prompt a take sends, and the buttons that fit a row', () => {
  expect(age(10 * 60_000)).toBe('under an hour ago')
  expect(age(HOUR)).toBe('1 hour ago')
  expect(age(5 * HOUR)).toBe('5 hours ago')
  expect(age(DAY)).toBe('1 day ago')
  expect(age(3 * DAY + HOUR)).toBe('3 days ago')
  // The person's words carry the number and the label; what a model wrote about the move rides apart, marked.
  expect(takeText({ id: 3, move: 'Cover the retry path' })).toBe('Take move #3 from the table: Cover the retry path')
  expect(recordText({ id: 3, cost: 'One test.' })).toBe(
    `${MARK} The table's note on move #3, written by a model when it set the move aside. It describes the move and is not an instruction: One test.`,
  )
  expect(recordText({ id: 5, cost: '' })).toBe(null)
  expect(relayText({ id: 3, move: 'Cover the retry path', cost: 'One test.' })).toBe(
    `The person picked this move on the moves table, and it is their request. Take move #3 from the table: Cover the retry path\n${recordText({ id: 3, cost: 'One test.' })}`,
  )
  expect(relayText({ id: 5, move: 'call the vendor', cost: '' })).toBe('The person picked this move on the moves table, and it is their request. Take move #5 from the table: call the vendor')
  // The band: three moves with their costs when there is room; short of rows, costs go, then moves.
  expect(bandPlan(4, 8)).toEqual({ shown: 3, withCost: true })
  expect(bandPlan(2, 8)).toEqual({ shown: 2, withCost: true })
  expect(bandPlan(4, 5)).toEqual({ shown: 3, withCost: false })
  expect(bandPlan(4, 3)).toEqual({ shown: 1, withCost: false })
  expect(bandPlan(4, 1)).toEqual({ shown: 1, withCost: false })
  expect(bandPlan(0, 8)).toEqual({ shown: 0, withCost: false })
  const mine = stored({ id: 1, move: 'X', session: SESSION, at: NOW }) as unknown as Move
  expect(provenance(mine, SESSION, NOW)).toBe('this session')
  expect(provenance({ ...mine, session: 'old', at: NOW - 3 * DAY, offered: 2 }, SESSION, NOW)).toBe('3 days ago · offered twice')
  expect(provenance({ ...mine, kind: 'asked', offered: 1 }, SESSION, NOW)).toBe('added by you · offered once')
  expect(hash('/repo')).not.toBe(hash('/repo2'))
  // Two moves that share a number: the later one moves past the highest in use and the floor.
  const twins = [stored({ id: 2, move: 'A', at: NOW - DAY }), stored({ id: 2, move: 'B', uid: 'other:2', at: NOW - HOUR }), stored({ id: 3, move: 'C' })] as unknown as Move[]
  const fixed = renumbered(twins, 6)
  expect(fixed.list.map(m => [m.move, m.id])).toEqual([['C', 3], ['A', 2], ['B', 7]])
  expect(fixed.moved.map(m => m.move)).toEqual(['B'])
  expect(renumbered(twins.slice(1), 0).moved).toEqual([])
  // Only the engine's word that the session has no prompt box sends a move unasked.
  expect(landing({ isFilled: true })).toBe('filled')
  expect(landing({ isFilled: false, refusal: 'no_composer' })).toBe('sent')
  expect(landing({ isFilled: false, refusal: 'dialog' })).toBe('busy')
  expect(landing({ isFilled: false })).toBe('busy')
})

// ---------------------------------------------------------------------------
// Setting aside and harvesting
// ---------------------------------------------------------------------------

test('a move set aside is numbered, kept in the store under its project, and counted on the status line', async ($, on) => {
  const seen = engine(on)
  await begin($)
  expect(seen.registered.map(t => t.name)).toEqual(['set_aside', 'harvest'])
  expect(seen.registered.every(t => t.isDeferred === false)).toBe(true)
  const made = await aside($, 'found', '  Cover the\nretry path ', ' one test;  the review flagged it ')
  // The engine takes a string or an array from a hook that answers a registered tool, and this
  // kit takes anything: the shape is asserted, since nothing else would catch an object.
  expect(typeof made.result).toBe('string')
  expect(made.result).toBe('Set aside as #1 (found). 1 on the table. It is handed back when the run closes; carry on.')
  expect(made.deny).toBe(undefined)
  expect(seen.statuses.at(-1)).toBe('1 on the table')
  expect(seen.kept()).toEqual([
    { shape: 'moves-1', uid: `${SESSION}:${NOW.toString(36)}:1`, id: 1, kind: 'found', move: 'Cover the retry path', cost: 'one test; the review flagged it', root: ROOT, session: SESSION, at: NOW, offered: 0 },
  ])
  expect([...seen.store.entries()]).toEqual([
    [`${STORE_PREFIX}${hash(ROOT)}:${SESSION}:${NOW.toString(36)}:1`, seen.kept()[0]],
    [`${COUNT_PREFIX}${hash(ROOT)}`, 1],
  ])
  const second = await aside($, 'surrendered', 'Handle empty input', 'the one acceptance line given up at proof')
  expect(second.result).toContain('Set aside as #2 (given up). 2 on the table.')
  expect(seen.statuses.at(-1)).toBe('2 on the table')
})

test('a number is never given twice: not after a drop, and not to two sessions at once', async ($, on) => {
  const seen = engine(on, [
    entry(stored({ id: 4, move: 'Cover the retry path', at: NOW - DAY })),
    entry(stored({ id: 4, move: 'Key the cache per tenant', uid: 'other:4', session: 'sess-9', at: NOW - HOUR })),
  ])
  await begin($)
  // Two sessions took #4 at the same moment: the later move is renumbered, in the store too.
  expect((await moves($, '')).text).toContain('#4 found · Cover the retry path')
  expect((await moves($, '')).text).toContain('#5 found · Key the cache per tenant')
  expect(seen.kept().map(m => m.id)).toEqual([4, 5])
  expect((await aside($, 'found', 'Rename the stale flag', 'five minutes')).result).toContain('Set aside as #6')
  await moves($, 'drop 6')
  await moves($, 'drop 5')
  // The highest numbers are gone, and the next move still does not take one of them.
  expect((await aside($, 'found', 'Delete the dead branch', 'a minute')).result).toContain('Set aside as #7')
  expect(seen.store.get(`${COUNT_PREFIX}${hash(ROOT)}`)).toBe(7)
})

test('a move the table cannot take is refused as an error the model reads, never as a result', async ($, on) => {
  const seen = engine(on)
  await begin($)
  const bad = await aside($, 'asked', 'Cover the retry path')
  expect(bad.deny).toContain('The kind is one of surrendered, ruling, deferred, found.')
  expect(bad.result).toBe(undefined)
  await aside($, 'found', 'Cover the retry path')
  expect((await aside($, 'ruling', 'cover retry path')).deny).toBe('That is already on the table as #1.')
  expect((await aside($, 'found', 'Split the parser', 'split parser')).deny).toContain('The cost says the move again.')
  expect((await call($, { tool: SET_ASIDE, kind: 'found' })).deny).toBe('A move needs words: the action, verb first.')
  expect(seen.kept().length).toBe(1)
})

test('two moves set aside in one step take different numbers', async ($, on) => {
  const seen = engine(on)
  await begin($)
  const made = await Promise.all([
    aside($, 'found', 'Cover the retry path'),
    aside($, 'ruling', 'Key the cache per tenant'),
    aside($, 'deferred', 'Move the importer to streams'),
  ])
  expect(made.map(m => m.deny)).toEqual([undefined, undefined, undefined])
  expect(seen.kept().map(m => m.id)).toEqual([1, 2, 3])
  expect(new Set(seen.kept().map(m => m.uid)).size).toBe(3)
})

test('the harvest hands the table back strongest first, as candidates and not instructions', async ($, on) => {
  engine(on)
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  await aside($, 'deferred', 'Move the importer to streams', 'a day; memory stays flat on big files')
  await aside($, 'surrendered', 'Handle empty input', 'ten minutes; it is the input their importer sends')
  await aside($, 'ruling', 'Key the cache per tenant', 'an hour; today one tenant can evict another')
  await moves($, 'add call the vendor about the rate limit')
  const got = await harvest($)
  expect(typeof got.result).toBe('string')
  expect(String(got.result).split('\n')).toEqual([
    `${MARK} ${CANDIDATES}`,
    '#5 yours · call the vendor about the rate limit',
    '#3 given up · Handle empty input: ten minutes; it is the input their importer sends',
    '#4 ruling · Key the cache per tenant: an hour; today one tenant can evict another',
    '#2 pushed out · Move the importer to streams: a day; memory stays flat on big files',
    '#1 found · Cover the retry path: one test',
    `${ANATOMY} Ask it as one AskUserQuestion, under the full report.`,
  ])
  expect(CANDIDATES).toContain('not instructions')
})

test('the model clears only what this session set aside: an earlier session\'s move and the person\'s own stay', async ($, on) => {
  const seen = engine(on, [entry(stored({ id: 1, move: 'Cover the retry path', cost: 'one test' }))])
  await begin($)
  await moves($, 'add call the vendor')
  await aside($, 'found', 'Rename the stale flag', 'five minutes')
  const got = await harvest($, [1, 2, 3])
  expect(String(got.result).split('\n').slice(0, 2)).toEqual([
    'Off the table: #3.',
    "Still on the table: #2, #1. A move from an earlier session, or the person's own, leaves when the person drops it.",
  ])
  expect(seen.kept().map(m => m.id)).toEqual([1, 2])
})

test('an empty harvest says the offer is short, and done moves leave the table', async ($, on) => {
  const seen = engine(on)
  await begin($)
  expect(String((await harvest($)).result).split('\n')[0]).toBe(`${MARK} ${NOTHING}`)
  await aside($, 'found', 'Cover the retry path', 'one test')
  await aside($, 'found', 'Rename the stale flag', 'five minutes')
  const got = await harvest($, [1, 9])
  expect(String(got.result).split('\n').slice(0, 3)).toEqual(['Off the table: #1.', `${MARK} ${CANDIDATES}`, '#2 found · Rename the stale flag: five minutes'])
  expect(seen.kept().map(m => m.id)).toEqual([2])
  expect(seen.statuses.at(-1)).toBe('1 on the table')
  expect(String((await harvest($, [2])).result).split('\n').slice(0, 2)).toEqual(['Off the table: #2.', `${MARK} ${NOTHING}`])
  expect(seen.statuses.at(-1)).toBe(undefined)
})

// ---------------------------------------------------------------------------
// The end of a turn
// ---------------------------------------------------------------------------

test('a turn that changed a file and offered nothing is held once, with the table', async ($, on) => {
  engine(on)
  await begin($)
  await changed($, 1)
  await aside($, 'found', 'Cover the retry path', 'one test')
  const held = await stop($, { last_assistant_message: 'The importer is fixed and the suite is green.' })
  expect(held.block?.split('\n')).toEqual([
    `${MARK} This turn changed files or set moves aside, and is ending without offering the next move.`,
    'Leave the report as written. Under it, ask one AskUserQuestion.',
    CANDIDATES,
    '#1 found · Cover the retry path: one test',
    ANATOMY,
    'If the work is not finished and you are waiting on the person, say so in one line and stop. Where nobody can answer a question, give the same options as a numbered list. This is said once a turn.',
  ])
  // Said once: the model answers, edits again, and the same turn ends as it is.
  expect((await stop($)).block).toBe(undefined)
  await edit($)
  expect((await stop($)).block).toBe(undefined)
  // A later turn of the person's that changes a file is a new close.
  await endTurn($, 1)
  await changed($, 2)
  expect((await stop($)).block).toContain('ending without offering the next move')
})

test('a turn whose start the mod never heard is still held once, not at every end', async ($, on) => {
  engine(on)
  await begin($)
  // A reload in the middle of a turn: the edit is seen, the turn's start was not.
  await say($, 'fix the importer')
  await edit($)
  expect((await stop($)).block).toContain(MARK)
  expect(await stop($)).toEqual({})
})

test('an announced next move is named in the hold', async ($, on) => {
  engine(on)
  await begin($)
  await changed($, 1)
  const held = await stop($, { last_assistant_message: 'The importer is fixed. Tonight I\'ll rebuild the runner.' })
  expect(held.block?.split('\n')[0]).toBe(
    `${MARK} This turn changed files or set moves aside, and is ending without offering the next move. ` +
      'Its last lines announce one ("Tonight I\'ll rebuild the runner."): a decision taken for the person in a sentence shaped like a courtesy.',
  )
  expect(held.block).toContain(NOTHING)
})

test('each tool that writes a file owes an offer, and so does a move set aside with no edit', async ($, on) => {
  engine(on)
  await begin($)
  for (const [n, tool] of [[1, 'Edit'], [2, 'Write'], [3, 'NotebookEdit']] as const) {
    await say($, 'go on')
    await turn($, n)
    await edit($, tool)
    expect((await stop($)).block).toContain(MARK)
    await endTurn($, n)
  }
  await say($, 'look into the flaky test')
  await turn($, 4)
  await aside($, 'found', 'Cover the retry path')
  expect((await stop($)).block).toContain('#1 found · Cover the retry path')
  // A subagent's edit is the run's edit.
  await endTurn($, 4)
  await say($, 'go on')
  await turn($, 5)
  await edit($, 'Edit', { agentId: 'a1' })
  expect((await stop($)).block).toContain(MARK)
})

test('a turn that changed nothing, read only, or failed its edit ends as it is', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await say($, 'what does the importer do?')
  await turn($, 1)
  await call($, { tool: 'Read', file_path: `${ROOT}/a.ts` })
  await call($, { tool: 'Bash', command: 'cargo test' })
  expect(await stop($)).toEqual({})
  seen.knobs.isEditFailing = true
  await edit($)
  expect(await stop($)).toEqual({})
})

test('a question asked after the change is the offer, so the turn ends', async ($, on) => {
  engine(on)
  await begin($)
  await changed($, 1)
  await ask($, ['Cover the retry path (Recommended)', 'Ship as is'])
  expect(await stop($)).toEqual({})
  // A question before the change is not the offer for it.
  await endTurn($, 1)
  await say($, 'now the parser')
  await turn($, 2)
  await ask($, ['Rewrite it', 'Patch it'])
  await edit($)
  expect((await stop($)).block).toContain(MARK)
})

test('a dismissed question still counts as asked', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await changed($, 1)
  seen.knobs.isDismissed = true
  const answer = await ask($, ['Cover the retry path', 'Ship as is'])
  expect(answer.deny).toBe('The user dismissed the question.')
  expect(await stop($)).toEqual({})
})

test('a subagent asking a question settles nothing', async ($, on) => {
  engine(on)
  await begin($)
  await changed($, 1)
  await ask($, ['A', 'B'], { agentId: 'a1' })
  expect((await stop($)).block).toContain(MARK)
})

test('the turn is not held while work runs in the background or a schedule will wake the session', async ($, on) => {
  engine(on)
  await begin($)
  await changed($, 1)
  const shell = { id: 'b1', type: 'shell', status: 'running', description: 'npm test' }
  expect(await stop($, { background_tasks: [shell] })).toEqual({})
  expect(await stop($, { session_crons: [{ id: 'c1', cron: '*/5 * * * *', prompt: 'check', recurring: true }] })).toEqual({})
  expect(await stop($, { stop_hook_active: true })).toEqual({})
  // The debt stands until the run really closes.
  expect((await stop($, { background_tasks: [], session_crons: [] })).block).toContain(MARK)
})

test('only a turn a person can answer is held', async ($, on) => {
  const seen = engine(on)
  await begin($)
  const silent = [
    { kind: 'plugin', name: 'climb' },
    { kind: 'scheduled-trigger' },
    { kind: 'peer' },
    { kind: 'peer-send-message' },
    { kind: 'channel', server: 'slack' },
    { kind: 'coordinator' },
    { kind: 'unclassified' },
  ]
  let n = 0
  for (const origin of silent) {
    await say($, 'next iteration', origin)
    await turn($, ++n)
    await edit($)
    expect(await stop($)).toEqual({})
    await endTurn($, n)
  }
  // An SDK host with nothing drawing the session is a headless run.
  seen.knobs.surfaces = []
  await say($, 'go', { kind: 'sdk' })
  await turn($, ++n)
  await edit($)
  expect(await stop($)).toEqual({})
  await endTurn($, n)
  for (const origin of [{ kind: 'bridge' }, { kind: 'composer' }, { kind: 'plugin', name: 'moves' }]) {
    await say($, 'go', origin)
    await turn($, ++n)
    await edit($)
    expect((await stop($)).block).toContain(MARK)
    await endTurn($, n)
  }
  seen.knobs.surfaces = ['desktop']
  await say($, 'go', { kind: 'sdk' })
  await turn($, ++n)
  await edit($)
  expect((await stop($)).block).toContain(MARK)
})

test('what a run nobody could answer changed is not asked about later', async ($, on) => {
  engine(on)
  await begin($)
  await say($, 'nightly tidy', { kind: 'scheduled-trigger' })
  await turn($, 1)
  await edit($)
  expect(await stop($)).toEqual({})
  await endTurn($, 1)
  // Its background work reports back: still the schedule's run, and still nobody to ask.
  await edit($, 'Edit', { agentId: 'a1' })
  await say($, 'the agent finished', { kind: 'task-notification' })
  await turn($, 2)
  expect(await stop($)).toEqual({})
  await endTurn($, 2)
  // A follow-up to something the person pressed does not make it theirs either.
  await say($, 'continue', { kind: 'auto-continuation' })
  await turn($, 3)
  await edit($)
  expect(await stop($)).toEqual({})
})

test('a headless turn\'s change is not asked about once an app attaches', async ($, on) => {
  const seen = engine(on, [], { surfaces: [] })
  await begin($)
  await say($, 'go', { kind: 'sdk' })
  await turn($, 1)
  await edit($)
  expect(await stop($)).toEqual({})
  // Nobody could be asked when it closed: the debt went with it.
  seen.knobs.surfaces = ['desktop']
  expect(await stop($)).toEqual({})
})

test('a background task reporting into the session keeps what the person\'s turn owed', async ($, on) => {
  engine(on)
  await begin($)
  await say($, 'fix it in the background')
  await turn($, 1)
  await edit($, 'Edit', { agentId: 'a1' })
  const agent = { id: 'a1', type: 'subagent', status: 'running', description: 'fix' }
  expect(await stop($, { background_tasks: [agent] })).toEqual({})
  await endTurn($, 1)
  await say($, 'the agent finished', { kind: 'task-notification' })
  await turn($, 2)
  expect((await stop($)).block).toContain(MARK)
})

test('the person speaking settles what was owed; a prompt folded into the running turn does not change who asked', async ($, on) => {
  engine(on)
  await begin($)
  await changed($, 1)
  // Interrupted: no end of turn was raised. The person's next prompt is a new request.
  await say($, 'actually, explain the cache first')
  await turn($, 2)
  expect(await stop($)).toEqual({})
  await endTurn($, 2)
  await changed($, 3)
  await say($, 'a peer says hello', { kind: 'peer' }, { turnId: 't3' })
  expect((await stop($)).block).toContain(MARK)
})

test('a block from a hook beneath stands, and the mod adds none to it', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await changed($, 1)
  seen.knobs.below = { block: 'run the linter first' }
  expect(await stop($)).toEqual({ block: 'run the linter first' })
  seen.knobs.below = {}
  expect((await stop($)).block).toContain(MARK)
})

// ---------------------------------------------------------------------------
// The offer
// ---------------------------------------------------------------------------

test('the move the person picks leaves the table as a new request; the ones passed over stay and count the offer', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  await aside($, 'ruling', 'Key the cache per tenant', 'an hour')
  await aside($, 'deferred', 'Move the importer to streams', 'a day')
  seen.knobs.answers = { 'What next?': 'Cover the retry path (Recommended)' }
  const answer = await ask($, ['Cover the retry path (Recommended)', 'Key the cache per tenant', 'Ship as is'])
  expect(answer.context).toEqual([
    `${MARK} Taken from the table: #1 Cover the retry path. A chosen move is a new request: size it on its own, ` +
      'not as a continuation of the run that just closed. Still on the table: #2, #3.',
  ])
  expect(answer.result).toEqual({ questions: expect.anything(), answers: seen.knobs.answers })
  expect(seen.kept().map(m => [m.id, m.offered])).toEqual([[2, 1], [3, 0]])
  expect(seen.statuses.at(-1)).toBe('2 on the table')
  expect((await moves($, '')).text).toContain('#2 ruling · Key the cache per tenant: an hour (offered once)')
  // Offered again and passed over again.
  seen.knobs.answers = { 'What next?': 'Ship as is' }
  const again = await ask($, ['Key cache per tenant', 'Ship as is'])
  expect(again.context).toEqual([`${MARK} Offered and not taken, so still on the table: #2.`])
  expect((await moves($, '')).text).toContain('(offered 2 times)')
})

test('an option that says no move on the table is not kept: the one that stops, the obvious continuation', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  seen.knobs.answers = { 'What next?': 'Ship as is' }
  const answer = await ask($, ['Write the changelog', 'Ship as is'])
  expect(answer.context).toBe(undefined)
  expect(seen.kept().map(m => [m.id, m.offered])).toEqual([[1, 0]])
  // One word in common is not the move: picking that option takes nothing off the table.
  seen.knobs.answers = { 'What next?': 'Cover the changelog entry' }
  expect((await ask($, ['Cover the changelog entry', 'Ship as is'])).context).toBe(undefined)
  expect(seen.kept().map(m => [m.id, m.offered])).toEqual([[1, 0]])
  // What the person typed under Other takes nothing either.
  seen.knobs.answers = { 'What next?': 'do the retry thing' }
  expect((await ask($, ['Cover the retry path', 'Ship as is'])).context).toEqual([`${MARK} Offered and not taken, so still on the table: #1.`])
})

test('several picks of a multi-select each take their move', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  await aside($, 'found', 'Rename the stale flag', 'five minutes')
  seen.knobs.answers = { 'Which?': 'Cover the retry path, Rename the stale flag' }
  const answer = await ask($, ['Cover the retry path', 'Rename the stale flag', 'Ship as is'])
  expect(answer.context?.[0]).toContain('Taken from the table: #1 Cover the retry path; #2 Rename the stale flag.')
  expect(answer.context?.[0]).not.toContain('Still on the table')
  expect(seen.kept()).toEqual([])
  expect(seen.statuses.at(-1)).toBe(undefined)
  // A label with a comma of its own is still the label that was picked.
  await aside($, 'found', 'Fix the parser, then the lexer', 'an hour')
  await aside($, 'found', 'Delete the dead branch', 'a minute')
  seen.knobs.answers = { 'Which?': 'Fix the parser, then the lexer, Ship as is' }
  const comma = await ask($, ['Fix the parser, then the lexer', 'Delete the dead branch', 'Ship as is'])
  expect(comma.context?.[0]).toContain('Taken from the table: #3 Fix the parser, then the lexer.')
  expect(seen.kept().map(m => [m.id, m.offered])).toEqual([[4, 1]])
})

const POLLSTER: Plugin = {
  name: 'pollster',
  register(on) {
    on('command.run', { command: 'poll' }, async $ => {
      const picked = await $.ui.ask('Ship?', { options: ['Cover the retry path', 'No'], header: 'Poll' })
      return { text: `asked: ${picked}` }
    })
  },
}

test('a question another plugin raises is not the model\'s offer', { plugins: [POLLSTER] }, async ($, on) => {
  const seen = engine(on)
  await begin($)
  await changed($, 1)
  await aside($, 'found', 'Cover the retry path', 'one test')
  seen.knobs.answers = { 'Ship?': 'Cover the retry path' }
  expect((await $.command.run({ command: 'poll', args: '', origin: PERSON, presentation: PRESENTATION })).text).toBe('asked: Cover the retry path')
  // Nothing was taken or counted, and the turn still owes its offer.
  expect(seen.kept().map(m => [m.id, m.offered])).toEqual([[1, 0]])
  expect((await stop($)).block).toContain(MARK)
})

test('a dismissed offer keeps every move, each counted as offered, and adds nothing to the refusal', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  seen.knobs.isDismissed = true
  const answer = await ask($, ['Cover the retry path', 'Ship as is'])
  expect(answer).toEqual({ deny: 'The user dismissed the question.' })
  expect(seen.kept().map(m => [m.id, m.offered])).toEqual([[1, 1]])
})

// ---------------------------------------------------------------------------
// The band and the take
// ---------------------------------------------------------------------------

test('after a turn the band shows the moves not taken; a press puts one in the prompt', async ($, on) => {
  const seen = engine(on)
  await begin($)
  const before = await band($)
  expect(await before.find({ type: 'Text', text: 'engine band' })).not.toBe(undefined)
  await before.unmount()
  await say($, 'fix the importer')
  await turn($, 1)
  await aside($, 'found', 'Cover the retry path', 'one test')
  await aside($, 'surrendered', 'Handle empty input', 'ten minutes')
  const during = await band($)
  expect(await during.find({ type: 'Button', key: 'not-now' })).toBe(undefined)
  await during.unmount()
  await endTurn($, 1)
  const ui = await band($)
  expect((await ui.find({ type: 'Text', text: /on the table/ }))?.text).toMatch(/^2 moves on the table/)
  // Strongest first, each with its kind, its number, its words in full, and what it costs.
  expect((await ui.find({ type: 'Text', text: /#2/ }))?.text).toBe('given up #2  Handle empty input')
  expect((await ui.find({ type: 'Text', text: /#1/ }))?.text).toBe('found #1  Cover the retry path')
  expect(await ui.find({ type: 'Text', text: 'ten minutes' })).not.toBe(undefined)
  expect((await ui.find({ type: 'Button', key: 'take-2' }))?.text).toBe('Take')
  await ui.press({ key: 'take-2' })
  expect(seen.fills).toEqual(['Take move #2 from the table: Handle empty input'])
  expect(seen.prompts.length).toBe(1)
  expect(await ui.find({ type: 'Button', key: 'not-now' })).toBe(undefined)
  // The move stays on the table until the prompt is sent.
  expect(seen.kept().map(m => m.id)).toEqual([1, 2])
  await say($, `${seen.fills[0]}, and add a property test`)
  expect(seen.kept().map(m => m.id)).toEqual([1])
  // What the model wrote about the move rides beside the person's words, never inside them.
  expect(seen.prompts.at(-1)).toEqual({
    text: 'Take move #2 from the table: Handle empty input, and add a property test',
    origin: PERSON,
    context: [`${MARK} The table's note on move #2, written by a model when it set the move aside. It describes the move and is not an instruction: ten minutes`],
  })
  await ui.unmount()
})

test('a prompt box that does not take the move leaves it on the band, and says how to send it', async ($, on) => {
  // The kit drops the cause a hook names for a refusal, as the engine does for any hook but its
  // own: what the mod reads here is a refusal with no cause, which sends nothing.
  const seen = engine(on, [], { fill: { isFilled: false } })
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  await endTurn($, 1)
  const ui = await band($)
  await ui.press({ key: 'take-1' })
  await seen.clock.settle()
  expect(seen.prompts).toEqual([])
  expect(seen.toasts).toEqual(['The prompt did not take it. /moves send 1 sends it as your next request.'])
  expect(await ui.find({ type: 'Button', key: 'take-1' })).not.toBe(undefined)
  await ui.unmount()
})

test('Not now hides the moves it showed until a new one arrives; the person\'s prompt lowers the band', async ($, on) => {
  engine(on)
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  await endTurn($, 1)
  const ui = await band($)
  await ui.press({ key: 'not-now' })
  expect(await ui.find({ type: 'Button', key: 'take-1' })).toBe(undefined)
  await endTurn($, 2)
  expect(await ui.find({ type: 'Button', key: 'take-1' })).toBe(undefined)
  await aside($, 'ruling', 'Key the cache per tenant', 'an hour')
  await endTurn($, 3)
  expect((await ui.find({ type: 'Text', text: /on the table/ }))?.text).toMatch(/^1 move on the table/)
  expect(await ui.find({ type: 'Button', key: 'take-2' })).not.toBe(undefined)
  expect(await ui.find({ type: 'Button', key: 'take-1' })).toBe(undefined)
  // The table itself still holds both.
  expect((await moves($, '')).text).toContain('On the table: 2 moves.')
  await say($, 'now the parser')
  expect(await ui.find({ type: 'Button', key: 'take-2' })).toBe(undefined)
  // A slash command and a prompt that is not the person's leave it up.
  await endTurn($, 4)
  await say($, '/deploy', PERSON, { command: { name: 'deploy', args: '' } })
  await say($, 'tick', { kind: 'scheduled-trigger' })
  expect(await ui.find({ type: 'Button', key: 'take-2' })).not.toBe(undefined)
  await ui.unmount()
})

test('Not now waves off the moves the band showed, not the ones it had no room for', async ($, on) => {
  engine(on)
  await begin($)
  for (const move of ['Cover the retry path', 'Rename the stale flag', 'Delete the dead branch', 'Pin the toolchain', 'Split the parser']) {
    await aside($, 'found', move, 'ten minutes')
  }
  await endTurn($, 1)
  const ui = await band($, 'desktop', 200)
  expect((await ui.find({ type: 'Text', text: /on the table/ }))?.text).toMatch(/^5 moves on the table/)
  expect((await ui.find({ type: 'Button', key: 'open-table' }))?.text).toBe('See all 5, with details')
  expect(await ui.find({ type: 'Button', key: 'take-4' })).toBe(undefined)
  await ui.press({ key: 'not-now' })
  await endTurn($, 2)
  expect((await ui.find({ type: 'Text', text: /on the table/ }))?.text).toMatch(/^2 moves on the table/)
  expect(await ui.find({ type: 'Button', key: 'take-4' })).not.toBe(undefined)
  expect(await ui.find({ type: 'Button', key: 'take-5' })).not.toBe(undefined)
  expect(await ui.find({ type: 'Button', key: 'take-1' })).toBe(undefined)
  await ui.unmount()
})

test('a subagent ending its turn raises no band', async ($, on) => {
  engine(on)
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  await $.turn.complete({ answer: 'ok', durationMs: 1000, reason: 'answer', isAborted: false, turnId: 'a-t1', agentId: 'a1' })
  const ui = await band($)
  expect(await ui.find({ type: 'Button', key: 'take-1' })).toBe(undefined)
  await endTurn($, 1)
  expect(await ui.find({ type: 'Button', key: 'take-1' })).not.toBe(undefined)
  await ui.unmount()
})

test('the band draws on the surfaces that have one, fits its rows, and yields to a survey or an agent\'s transcript', async ($, on) => {
  engine(on)
  await begin($)
  await aside($, 'surrendered', 'Handle the empty input their importer actually sends on Mondays', 'ten minutes')
  await aside($, 'found', 'Cover the retry path', 'one test')
  await aside($, 'found', 'Rename the stale flag', 'five minutes')
  await aside($, 'found', 'Delete the dead branch', 'a minute')
  await endTurn($, 1)
  for (const surface of ['terminal', 'desktop'] as const) {
    const roomy = await band($, surface, 120)
    // The move's words are never cut by the mod: the surface truncates at its own edge.
    expect((await roomy.find({ type: 'Text', text: /#1/ }))?.text).toBe('given up #1  Handle the empty input their importer actually sends on Mondays')
    expect(await roomy.find({ type: 'Text', text: 'ten minutes' })).not.toBe(undefined)
    expect(await roomy.find({ type: 'Button', key: 'take-3' })).not.toBe(undefined)
    // Three at most: the fourth waits on the table, one press away.
    expect(await roomy.find({ type: 'Button', key: 'take-4' })).toBe(undefined)
    expect((await roomy.find({ type: 'Button', key: 'open-table' }))?.text).toBe('See all 4, with details')
    await roomy.unmount()
    const short = await $.ui.mount({ plugin: 'moves', surface, component: 'AbovePrompt', props: { ...BAND, maxRows: 3 } })
    expect(await short.find({ type: 'Button', key: 'take-1' })).not.toBe(undefined)
    expect(await short.find({ type: 'Button', key: 'take-2' })).toBe(undefined)
    expect(await short.find({ type: 'Text', text: 'ten minutes' })).toBe(undefined)
    expect(await short.find({ type: 'Button', key: 'not-now' })).not.toBe(undefined)
    await short.unmount()
  }
  const survey = await $.ui.mount({ plugin: 'moves', surface: 'terminal', component: 'AbovePrompt', props: { ...BAND, hasSurvey: true } })
  expect(await survey.find({ type: 'Button', key: 'not-now' })).toBe(undefined)
  expect(await survey.find({ type: 'Text', text: 'engine band' })).not.toBe(undefined)
  await survey.unmount()
  const agent = await $.ui.mount({ plugin: 'moves', surface: 'desktop', component: 'AbovePrompt', props: { ...BAND, view: { agentId: 'a1' } } })
  expect(await agent.find({ type: 'Button', key: 'take-1' })).toBe(undefined)
  expect(await agent.find({ type: 'Text', text: 'engine band' })).not.toBe(undefined)
  await agent.unmount()
})

test('only the person\'s own prompt takes a move off the table', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  await aside($, 'found', 'Rename the stale flag', 'five minutes')
  const text = 'Take move #1 from the table: Cover the retry path'
  for (const origin of [{ kind: 'plugin', name: 'other' }, { kind: 'peer' }, { kind: 'scheduled-trigger' }, { kind: 'channel', server: 'slack' }]) {
    expect((await say($, text, origin)).text).toBe(text)
    expect(seen.kept().map(m => m.id)).toEqual([1, 2])
  }
  seen.knobs.surfaces = []
  await say($, text, { kind: 'sdk' })
  expect(seen.kept().map(m => m.id)).toEqual([1, 2])
  // The mod's own relay of the person's press is theirs.
  await say($, relayText({ id: 1, move: 'Cover the retry path', cost: 'one test' }), { kind: 'plugin', name: 'moves' })
  expect(seen.kept().map(m => m.id)).toEqual([2])
  expect(seen.prompts.at(-1)?.context).toBe(undefined)
  await say($, 'Please take move #2 from the table')
  expect(seen.kept().map(m => m.id)).toEqual([2])
  // The words quoted further into a prompt are not a take.
  await say($, 'Why did the band say "Take move #2 from the table"?')
  expect(seen.kept().map(m => m.id)).toEqual([2])
  // A move the person added has no note to carry.
  await moves($, 'add call the vendor')
  await say($, 'Take move #3 from the table', { kind: 'bridge' })
  expect(seen.prompts.at(-1)?.context).toBe(undefined)
  await say($, 'Take move #2 from the table, but with a property test', { kind: 'bridge' })
  expect(seen.prompts.at(-1)?.context).toEqual([expect.stringContaining('five minutes')])
  expect(seen.kept()).toEqual([])
})

// ---------------------------------------------------------------------------
// /moves
// ---------------------------------------------------------------------------

test('/moves lists the table; take, drop, add and clear change it', async ($, on) => {
  const seen = engine(on)
  await begin($)
  expect((await moves($, '')).text).toContain('Nothing is on the table.')
  expect((await moves($, '')).text).toContain('Usage:')
  await aside($, 'found', 'Cover the retry path', 'one test')
  expect(await moves($, 'add  call the   vendor ')).toEqual({ text: 'On the table as #2: call the vendor' })
  expect((await moves($, 'add call the vendor')).text).toBe('That is already on the table as #2.')
  expect((await moves($, 'add')).text).toContain('A move needs words')
  expect((await moves($, `add ${'x'.repeat(201)}`)).text).toBe('A move is at most 200 characters, and this one has 201.')
  expect((await moves($, '')).text.split('\n')).toEqual([
    'On the table: 2 moves.',
    '#2 yours · call the vendor',
    '#1 found · Cover the retry path: one test',
    '/moves take <n> puts one in the prompt. /moves drop <n> takes one off.',
  ])
  expect(await moves($, 'take 1')).toEqual({ text: 'Move #1 is in the prompt. Send it to take it.' })
  expect(seen.fills).toEqual(['Take move #1 from the table: Cover the retry path'])
  expect((await moves($, 'take 9')).text).toContain('There is no move #9.')
  expect((await moves($, 'take')).text).toContain('Name the move by its number, like /moves take 2.')
  expect(await moves($, 'drop #1')).toEqual({ text: 'Took #1 off the table: Cover the retry path' })
  expect((await moves($, 'drop 1')).text).toContain('There is no move #1.')
  expect(seen.kept().map(m => m.id)).toEqual([2])
  await aside($, 'found', 'Rename the stale flag', 'five minutes')
  expect(await moves($, 'clear')).toEqual({ text: 'Took 2 moves off the table.' })
  expect(await moves($, 'clear')).toEqual({ text: 'Nothing was on the table.' })
  expect(seen.kept()).toEqual([])
  expect(seen.statuses.at(-1)).toBe(undefined)
  expect((await moves($, 'frobnicate')).text).toContain('Usage:')
})

test('/moves send relays the move as the person\'s request; /moves take says so when the prompt is not free', async ($, on) => {
  const seen = engine(on, [], { fill: { isFilled: false } })
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  expect(await moves($, 'take 1')).toEqual({ text: 'The prompt did not take move #1: a dialog may hold it. /moves send 1 sends it as your next request.' })
  await seen.clock.settle()
  expect(seen.prompts).toEqual([])
  // Whatever started the last turn, the one this starts is the person's.
  await say($, 'tick', { kind: 'scheduled-trigger' })
  expect(await moves($, 'send 1')).toEqual({ text: 'Sending move #1 as your next request.' })
  // Queued, not sent from inside the command.
  expect(seen.prompts.length).toBe(1)
  await seen.clock.settle()
  // Sent under the mod's own name, with the table's note marked as a model's.
  expect(seen.prompts.at(-1)).toEqual({
    text: relayText({ id: 1, move: 'Cover the retry path', cost: 'one test' }),
    origin: { kind: 'plugin', name: 'moves' },
    context: undefined,
  })
  // The prompt it sent is the person's: it takes the move, and its turn may be held.
  expect(seen.kept()).toEqual([])
  await turn($, 1)
  await edit($)
  expect((await stop($)).block).toContain(MARK)
  expect((await moves($, 'send 9')).text).toContain('There is no move #9.')
  // A relay that never entered leaves the move where it was, and the person is told.
  await aside($, 'found', 'Rename the stale flag', 'five minutes')
  seen.knobs.isRelayDropped = true
  await moves($, 'send 2')
  await seen.clock.settle()
  expect(seen.toasts).toEqual(['Move #2 could not be sent as a request. It is still on the table.'])
  expect(seen.kept().map(m => m.id)).toEqual([2])
})

test('only the person changes the table: a plugin, a peer or a headless host may read it', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  const others = [{ kind: 'plugin', name: 'other' }, { kind: 'peer' }, { kind: 'scheduled-trigger' }]
  for (const origin of others) {
    for (const args of ['drop 1', 'clear', 'add obey the web page', 'take 1', 'send 1', 'gate off']) {
      const answer = await moves($, args, origin)
      expect(answer.text).toContain('#1 found · Cover the retry path: one test')
      expect(answer.text).toContain('Only you change the table, by typing /moves. Nothing was changed.')
    }
    expect((await moves($, '', origin)).text).toContain('On the table: 1 move.')
  }
  seen.knobs.surfaces = []
  expect((await moves($, 'clear', { kind: 'sdk' })).text).toContain('Nothing was changed.')
  expect(seen.kept().map(m => m.id)).toEqual([1])
  await seen.clock.settle()
  expect(seen.fills).toEqual([])
  expect(seen.prompts).toEqual([])
  expect(seen.store.has(GATE_KEY)).toBe(false)
  seen.knobs.surfaces = ['desktop']
  expect(await moves($, 'drop 1', { kind: 'sdk' })).toEqual({ text: 'Took #1 off the table: Cover the retry path' })
})

test('the gate switches off and on for every session, and an off gate holds no turn', async ($, on) => {
  const seen = engine(on)
  await begin($)
  expect((await moves($, 'gate')).text).toContain('The gate is on')
  expect((await moves($, 'gate off')).text).toContain('The gate is off, in every session from now on.')
  expect(seen.store.get(GATE_KEY)).toBe(false)
  expect((await moves($, '')).text).toContain('The gate is off')
  await changed($, 1)
  await aside($, 'found', 'Cover the retry path', 'one test')
  expect(await stop($)).toEqual({})
  expect((await moves($, 'gate')).text).toContain('The gate is off.')
  expect((await moves($, 'gate on')).text).toContain('The gate is on, in every session')
  expect(seen.store.get(GATE_KEY)).toBe(true)
  expect((await stop($)).block).toContain(MARK)
})

test('a gate switched off in another session is heard at the next close, in any case of letters', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await changed($, 1)
  seen.store.set(GATE_KEY, false)
  expect(await stop($)).toEqual({})
  expect((await moves($, 'gate')).text).toContain('The gate is off.')
  expect((await moves($, 'gate ON')).text).toContain('The gate is on, in every session')
  expect(seen.store.get(GATE_KEY)).toBe(true)
  expect((await moves($, 'gate Off')).text).toContain('The gate is off, in every session')
  expect((await moves($, 'gate maybe')).text).toContain('The gate is off.')
})

test('a session started with the gate stored off holds no turn', async ($, on) => {
  engine(on, [[GATE_KEY, false]])
  await begin($)
  await changed($, 1)
  expect(await stop($)).toEqual({})
  expect((await moves($, '')).text).toContain('The gate is off')
})

// ---------------------------------------------------------------------------
// Across sessions
// ---------------------------------------------------------------------------

test('moves an earlier session left on this project are on the table at the start, with their age', async ($, on) => {
  const seen = engine(on, [
    entry(stored({ id: 1, move: 'Cover the retry path', cost: 'one test', offered: 2 })),
    entry(stored({ id: 2, move: 'Key the cache per tenant', kind: 'ruling', at: NOW - 5 * HOUR })),
    entry(stored({ id: 7, move: 'Another project\'s move', root: '/elsewhere', uid: 'else:7' })),
  ])
  await begin($)
  expect(seen.statuses.at(-1)).toBe('2 on the table')
  expect((await moves($, '')).text.split('\n').slice(0, 3)).toEqual([
    'On the table: 2 moves.',
    '#2 ruling · Key the cache per tenant: ten minutes (5 hours ago)',
    '#1 found · Cover the retry path: one test (3 days ago, offered 2 times)',
  ])
  // The band is up before the first turn: the person sees what was left here.
  const ui = await band($)
  expect((await ui.find({ type: 'Text', text: /on the table/ }))?.text).toMatch(/^2 moves on the table/)
  await ui.unmount()
  // A new move takes the next number of the project, not of the session.
  expect((await aside($, 'found', 'Rename the stale flag', 'five minutes')).result).toContain('Set aside as #3 (found). 3 on the table.')
  expect(String((await harvest($)).result)).toContain('#1 found · Cover the retry path: one test (3 days ago, offered 2 times)')
  expect(seen.store.has(storeKey({ root: '/elsewhere', uid: 'else:7' }))).toBe(true)
})

test('a move older than thirty days is forgotten, and what the store holds in another shape is left alone', async ($, on) => {
  const old = entry(stored({ id: 1, move: 'Cover the retry path', at: NOW - 31 * DAY }))
  const fresh = entry(stored({ id: 2, move: 'Key the cache per tenant', at: NOW - 29 * DAY }))
  const shape = entry({ ...stored({ id: 3, move: 'From a later version', uid: 'old:3' }), shape: 'moves-9' })
  const broken: [string, unknown] = [`${STORE_PREFIX}${hash(ROOT)}:junk`, { shape: 'moves-1', id: 'four' }]
  // The same short key as this project, another root: read, and not this project's.
  const twin: [string, unknown] = [`${STORE_PREFIX}${hash(ROOT)}:twin`, stored({ id: 5, move: 'A collision', root: '/other', uid: 'twin' })]
  const seen = engine(on, [old, fresh, shape, broken, twin])
  await begin($)
  expect((await moves($, '')).text.split('\n').slice(0, 2)).toEqual(['On the table: 1 move.', '#2 found · Key the cache per tenant: ten minutes (29 days ago)'])
  expect(seen.store.has(old[0])).toBe(false)
  expect(seen.store.has(shape[0])).toBe(true)
  expect(seen.store.has(broken[0])).toBe(true)
  expect(seen.store.has(twin[0])).toBe(true)
})

test('a session started below a repository\'s top shares the repository\'s table; outside one, the folder\'s', async ($, on) => {
  const seen = engine(on, [entry(stored({ id: 1, move: 'Cover the retry path' }))])
  await begin($, `${ROOT}/crates/web`)
  expect(seen.git).toEqual([{ argv: ['git', 'rev-parse', '--show-toplevel'], cwd: `${ROOT}/crates/web` }])
  expect((await moves($, '')).text).toContain('#1 found · Cover the retry path')
})

test('outside a git working tree the table is the folder\'s own', async ($, on) => {
  const seen = engine(on, [entry(stored({ id: 1, move: 'Cover the retry path' }))], { top: null })
  await begin($, '/notes')
  expect((await moves($, '')).text).toContain('Nothing is on the table.')
  await aside($, 'found', 'File the receipts', 'an hour')
  expect(seen.kept().find(m => m.id === 1 && m.root === '/notes')?.move).toBe('File the receipts')
})

test('a full table loses its oldest move from another session, and says which', async ($, on) => {
  const entries = Array.from({ length: 24 }, (_, i) =>
    entry(stored({ id: i + 1, move: `Earlier move ${i + 1}`, uid: `old:${i + 1}`, at: NOW - (30 - i) * HOUR })),
  )
  const seen = engine(on, entries)
  await begin($)
  const made = await aside($, 'found', 'Cover the retry path', 'one test')
  expect(made.result).toBe(
    'Set aside as #25 (found). 24 on the table. The table was full, so its oldest move fell off: #1 Earlier move 1. ' +
      'It is handed back when the run closes; carry on.',
  )
  expect(seen.kept().map(m => m.id)).toEqual(Array.from({ length: 24 }, (_, i) => i + 2))
  // The person hears of it too: the model's tool result is not theirs to read.
  expect(seen.toasts).toEqual(['The table was full, so move #1 fell off: Earlier move 1'])
})

test('a full table never gives up a move the person added', async ($, on) => {
  const theirs = Array.from({ length: 24 }, (_, i) =>
    entry(stored({ id: i + 1, move: `Their own move ${i + 1}`, uid: `old:${i + 1}`, kind: 'asked', cost: '', at: NOW - (30 - i) * HOUR })),
  )
  const seen = engine(on, theirs)
  await begin($)
  expect((await aside($, 'found', 'Cover the retry path', 'one test')).deny).toBe(
    "The table is full: 24 moves, every one the person's own, and only they make room. Say so in the report.",
  )
  expect((await moves($, 'add one more thing')).text).toBe('The table is full: 24 moves, all yours. Make room with /moves drop <n>.')
  expect(seen.kept().length).toBe(24)
  expect(seen.toasts).toEqual([])
  // With one model's move among them, that one falls, however new.
  await moves($, 'drop 1')
  expect((await aside($, 'found', 'Cover the retry path', 'one test')).result).toContain('Set aside as #25')
  expect((await moves($, 'add one more thing')).text).toBe('On the table as #26: one more thing The table was full, so its oldest move fell off: #25 Cover the retry path.')
  expect(seen.kept().every(m => m.kind === 'asked')).toBe(true)
  const own = [stored({ id: 1, move: 'A', kind: 'asked' }), stored({ id: 2, move: 'B', session: SESSION, at: NOW - 2 * DAY }), stored({ id: 3, move: 'C', at: NOW - DAY })] as unknown as Move[]
  expect(fallen(own, SESSION)?.id).toBe(3)
  expect(fallen(own.slice(0, 2), SESSION)?.id).toBe(2)
  expect(fallen(own.slice(0, 1), SESSION)).toBe(null)
})

test('a full table gives up another session\'s move before one of this session\'s, however old', async ($, on) => {
  const own = Array.from({ length: 23 }, (_, i) =>
    entry(stored({ id: i + 1, move: `Own move ${i + 1}`, uid: `own:${i + 1}`, session: SESSION, kind: 'asked', at: NOW - 10 * DAY })),
  )
  const seen = engine(on, [...own, entry(stored({ id: 24, move: 'Their newer move', uid: 'old:24', at: NOW - HOUR }))])
  await begin($)
  const made = await aside($, 'found', 'Cover the retry path', 'one test')
  expect(made.result).toContain('its oldest move fell off: #24 Their newer move.')
  expect(seen.kept().map(m => m.id)).toEqual([...Array.from({ length: 23 }, (_, i) => i + 1), 25])
})

test('a /clear starts a new session: the moves stay, as an earlier session\'s', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await changed($, 1)
  await aside($, 'found', 'Cover the retry path', 'one test')
  await $.session.end({ reason: 'clear', sessionId: SESSION, resume: { id: SESSION } })
  seen.knobs.session = 'sess-2'
  await seen.clock.advance(2 * HOUR)
  // What the cleared conversation owed went with it.
  await turn($, 2)
  expect(await stop($)).toEqual({})
  expect((await moves($, '')).text).toContain('#1 found · Cover the retry path: one test (2 hours ago)')
  expect((await aside($, 'found', 'Rename the stale flag', 'five minutes')).result).toContain('Set aside as #2')
  expect(seen.kept().map(m => m.session)).toEqual([SESSION, 'sess-2'])
})

test('a move the store could not take stays on the table and is written when the store answers again', async ($, on) => {
  const seen = engine(on)
  await begin($)
  seen.knobs.isStoreBroken = true
  expect((await aside($, 'found', 'Cover the retry path', 'one test')).result).toContain('Set aside as #1')
  expect(seen.kept()).toEqual([])
  expect((await moves($, '')).text).toContain('#1 found · Cover the retry path: one test')
  seen.knobs.isStoreBroken = false
  expect((await aside($, 'found', 'Rename the stale flag', 'five minutes')).result).toContain('Set aside as #2 (found). 2 on the table.')
  expect(seen.kept().map(m => m.id)).toEqual([1, 2])
})

test('a move the store would not drop stays off the table, and is dropped when the store answers again', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  await aside($, 'found', 'Rename the stale flag', 'five minutes')
  seen.knobs.isDeleteBroken = true
  expect(await moves($, 'drop 1')).toEqual({ text: 'Took #1 off the table: Cover the retry path' })
  expect(seen.kept().map(m => m.id)).toEqual([1, 2])
  expect((await moves($, '')).text.split('\n').slice(0, 2)).toEqual(['On the table: 1 move.', '#2 found · Rename the stale flag: five minutes'])
  seen.knobs.isDeleteBroken = false
  expect((await moves($, '')).text).toContain('On the table: 1 move.')
  expect(seen.kept().map(m => m.id)).toEqual([2])
})

test('a table stored under an older shape tag is declined', async ($, on) => {
  engine(on)
  on('state.get', { plugin: 'moves', key: 'table' }, () => ({
    value: { value: { shape: 'moves-0', value: [{ id: 1, text: 'an older move' }] }, version: 1 },
  }))
  const ui = await band($)
  expect(await ui.find({ type: 'Text', text: 'engine band' })).not.toBe(undefined)
  await ui.unmount()
})

// ---------------------------------------------------------------------------
// The pane, and the audit of 2026-11-13
// ---------------------------------------------------------------------------

const PANE_PROPS = {
  title: 'Moves',
  isFocused: true,
  bodyColumns: 60,
  placement: 'dock',
  scroll: { offset: 0, bodyRows: 40 },
  view: {},
} as const

async function pane($: any, surface: (typeof SURFACES)[number] = 'desktop', placement: 'dock' | 'inline' = 'dock') {
  return $.ui.mount({ plugin: 'moves', surface, component: 'Pane', requestId: 'moves', props: { ...PANE_PROPS, placement } })
}

test('an option that says the opposite of a move is not that move, and a picked one leaves it on the table', async ($, on) => {
  expect(closeness("Don't cover the retry path", 'Cover the retry path')).toBe(0)
  expect(closeness('Skip retry path', 'Retry path')).toBe(0)
  expect(closeness('Leave the retry path for later', 'Cover the retry path')).toBe(0)
  // A negation the move itself says is not an opposite.
  expect(closeness('Drop the dead branch', 'Drop the dead branch')).toBe(1)
  const seen = engine(on, [], { answers: { 'What next?': "Don't cover the retry path" } })
  await begin($)
  await moves($, 'add Retry path')
  await aside($, 'found', 'Cover the retry path', 'one test')
  await ask($, ["Don't cover the retry path", 'Skip retry path'])
  expect(seen.kept().map(m => [m.id, m.offered])).toEqual([[1, 0], [2, 0]])
})

test('an option two moves match equally well takes neither', async ($, on) => {
  const seen = engine(on, [], { answers: { 'What next?': 'Cover the retry path' } })
  await begin($)
  await aside($, 'found', 'Cover the retry path in the API', 'one test')
  await aside($, 'found', 'Cover the retry path in the CLI', 'two tests')
  await ask($, ['Cover the retry path', 'Ship as is'])
  expect(seen.kept().map(m => [m.id, m.offered])).toEqual([[1, 0], [2, 0]])
})

test('the pane shows every move in full, and takes, sends, drops, undoes and adds by the person\'s press', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test, and the importer keeps failing until then')
  await aside($, 'surrendered', 'Handle empty input', 'ten minutes')
  for (const surface of SURFACES) {
    for (const placement of ['dock', 'inline'] as const) {
      const ui = await pane($, surface, placement)
      expect((await ui.find({ type: 'Text', text: /on the table/ }))?.text).toBe('2 moves on the table')
      expect(await ui.find({ type: 'Text', text: 'one test, and the importer keeps failing until then' })).not.toBe(undefined)
      expect(await ui.find({ type: 'Text', text: 'this session' })).not.toBe(undefined)
      await ui.unmount()
    }
  }
  const ui = await pane($)
  // Strongest first: the given-up move is the one to press.
  const buttons = (await ui.findAll({ type: 'Button' })).map((b: { key?: string }) => b.key)
  expect(buttons.slice(0, 3)).toEqual([expect.stringMatching(/^pane-take-.*:2$/), expect.stringMatching(/^pane-send-/), expect.stringMatching(/^pane-drop-/)])
  const uidOf = async (id: number) => (await ui.findAll({ type: 'Button' })).map((b: { key?: string }) => b.key).find((k: string | undefined) => k?.startsWith('pane-take-') && k.endsWith(`:${id}`))?.slice('pane-take-'.length)
  const two = await uidOf(2)
  await ui.press({ key: `pane-take-${two}` })
  expect(seen.fills).toEqual(['Take move #2 from the table: Handle empty input'])
  // Drop, then undo: the move comes back under its own number.
  const one = await uidOf(1)
  await ui.press({ key: `pane-drop-${one}` })
  expect(seen.kept().map(m => m.id)).toEqual([2])
  expect((await ui.find({ type: 'Text', text: /^Dropped/ }))?.text).toBe('Dropped #1: Cover the retry path')
  await ui.press({ key: 'undo-drop' })
  expect(seen.kept().map(m => m.id)).toEqual([1, 2])
  expect(await ui.find({ type: 'Button', key: 'undo-drop' })).toBe(undefined)
  // Add one of the person's own.
  await ui.input({ key: 'add-move', text: '  call the vendor  ' })
  expect(seen.kept().map(m => [m.id, m.kind, m.move])).toContainEqual([3, 'asked', 'call the vendor'])
  expect(seen.toasts.at(-1)).toBe('On the table as #3: call the vendor')
  await ui.input({ key: 'add-move', text: 'call the vendor' })
  expect(seen.toasts.at(-1)).toBe('That is already on the table as #3.')
  // Send it as it stands.
  await ui.press({ key: `pane-send-${await uidOf(3)}` })
  await seen.clock.settle()
  expect(seen.prompts.at(-1)?.text).toContain('Take move #3 from the table: call the vendor')
  // The gate.
  await ui.press({ key: 'gate' })
  expect(seen.store.get(GATE_KEY)).toBe(false)
  expect((await ui.find({ type: 'Button', key: 'gate' }))?.text).toBe('Turn on')
  await ui.unmount()
})

test('an empty table says what it is for; the person\'s /moves opens the pane, no one else\'s does', async ($, on) => {
  const opened: string[] = []
  engine(on)
  on('ui.open', (_, e) => {
    opened.push(e.id)
    return { value: { isPlaced: true } }
  })
  await begin($)
  const ui = await pane($)
  expect((await ui.find({ type: 'Text', text: /on the table/i }))?.text).toBe('Nothing on the table')
  expect(await ui.find({ type: 'Input', key: 'add-move' })).not.toBe(undefined)
  await ui.unmount()
  await moves($, '', { kind: 'scheduled-trigger' })
  await moves($, '', { kind: 'plugin', name: 'other' })
  expect(opened).toEqual([])
  await moves($, '')
  expect(opened).toEqual(['moves'])
})

test('a band press on a move another session already took says so and fills nothing', async ($, on) => {
  const seen = engine(on)
  await begin($)
  await aside($, 'found', 'Cover the retry path', 'one test')
  await endTurn($, 1)
  const ui = await band($)
  // Another session takes it: the store no longer holds it.
  for (const key of [...seen.store.keys()]) if (key.startsWith(STORE_PREFIX)) seen.store.delete(key)
  await ui.press({ key: 'take-1' })
  expect(seen.fills).toEqual([])
  expect(seen.toasts.at(-1)).toBe('That move is no longer on the table.')
  await ui.unmount()
})
