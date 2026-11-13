import { expect, mock, test } from 'claude-code/testing'
import type { On } from 'claude-code'

const CWD = '/work'
const MARK = '[climb]'
const PRESENTATION = { isFullscreen: false, columns: 100 }
const ORIGIN = { kind: 'composer' } as const
const BAND = {
  plugin: 'climb',
  component: 'AbovePrompt',
  props: { hasSurvey: false, isCollapsed: false, view: {}, isWorking: false, maxRows: 3, bodyColumns: 100, scroll: { offset: 0, bodyRows: 3 } },
} as const

// The prompts the mod has queued so far: an iteration's turn starts with the latest of them.
let queuedPrompts: string[] = []
const latest = () => queuedPrompts[queuedPrompts.length - 1] ?? ''

type Run = { exitCode: number; stdout: string } | 'hang' | Promise<{ exitCode: number; stdout: string }>

/** Stands for the engine beneath the mod: each check run answers from `runs` in order. */
function engine(on: On, runs: Run[]) {
  const prompts: string[] = []
  queuedPrompts = prompts
  const checks: string[] = []
  const writes: string[] = []
  const clock = mock.clock(on)
  const knobs = { refuse: false }
  // Paths that exist, and where each one lands: a missing path rejects, as the host's does.
  const real = new Map<string, string>([['/work', '/work'], ['/work/tests', '/work/tests']])
  on('command.register', () => ({ value: { command: 'climb' } }))
  on('session.start', (_, e) => ({ cwd: e.cwd }))
  on('process.run', async (_, e) => {
    checks.push(JSON.stringify(e))
    const run = runs.shift() ?? { exitCode: 1, stdout: '' }
    if (run === 'hang') throw new Error('timed out')
    const r = await run
    return { value: { ...r, stderr: '', isStdoutTruncated: false, isStderrTruncated: false } }
  })
  on('fs.write', (_, e) => {
    writes.push(String((e as { path?: unknown }).path))
    return { value: undefined }
  })
  on('fs.stat', (_, e) => {
    const path = String((e as { path?: unknown }).path)
    const landing = real.get(path)
    if (landing === undefined) throw new Error('no such file')
    return { value: { kind: 'dir', size: 0, mtimeMs: 0, isLink: landing !== path, realPath: landing } }
  })
  on('prompt.submit', (_, e) => {
    if (knobs.refuse) throw new Error('refused')
    prompts.push(e.text)
    return { text: e.text }
  })
  on('turn.start', (_, e) => ({ turnId: e.turnId }))
  on('turn.complete', () => ({ text: 'done' }))
  on('tool.call', () => ({ result: {} }))
  on('ui.status', () => ({ value: undefined }))
  on('ui.toast', () => ({ value: undefined }))
  on('session.measure', (_, e) => ({ changed: e.changed }))
  on('session.usage', () => ({ value: { startedAt: 0, context: { window: 1 }, rateLimits: [{ kind: 'seven_day', percentUsed: 30 }] } }))
  return { prompts, checks, writes, clock, knobs, real }
}

async function climb($: any, args: string) {
  return $.command.run({ command: 'climb', args, origin: ORIGIN, presentation: PRESENTATION })
}

async function start($: any, args: string, clock: { settle: () => Promise<void> }) {
  await $.session.start({ cwd: CWD, surface: 'desktop', isInteractive: true })
  const answer = await climb($, args)
  await clock.settle()
  return answer
}

/** One main-loop turn; `own` says whether it is the mod's queued prompt. */
async function turn($: any, n: number, own = true, reason: 'answer' | 'aborted' = 'answer') {
  await $.turn.start({ text: own ? latest() : 'a question of my own', turnId: `t${n}` })
  await $.turn.complete({ answer: 'ok', durationMs: 60_000, reason, isAborted: reason !== 'answer', turnId: `t${n}` })
}

test('starting queues the first iteration itself', async ($, on) => {
  const seen = engine(on, [])
  const started = await start($, '5 npm test', seen.clock)
  expect(started.text).toContain('Climb started')
  expect(seen.prompts.length).toBe(1)
  expect(seen.prompts[0]?.startsWith(MARK)).toBe(true)
})

test('--in runs the check, writes the log and resolves protected paths in that folder', async ($, on) => {
  const seen = engine(on, [])
  await start($, '5 --in ../demo python3 check.py', seen.clock)
  await climb($, 'protect check.py')
  await turn($, 1)

  const guarded = await $.tool.call({ tool: 'Edit', file_path: '/demo/check.py', old_string: 'a', new_string: 'b' })
  expect(guarded.deny).toContain('protected')
  expect(seen.writes).toContain('/demo/.climb-log.tsv')
  expect(seen.checks[0]).toContain('"/demo"')
  expect((await climb($, '')).text).toContain('`python3 check.py`')
})

test('a failing check queues the next iteration with its output and score', async ($, on) => {
  const seen = engine(on, [{ exitCode: 1, stdout: '3 failed\nscore: 7' }])
  await start($, '5 npm test', seen.clock)
  await turn($, 1)

  expect(seen.prompts.length).toBe(2)
  expect(seen.prompts[1]).toContain('Iteration 1 of 5')
  expect(seen.prompts[1]).toContain('exited 1')
  expect(seen.prompts[1]).toContain('score 7')
  expect(seen.prompts[1]).toContain('3 failed')
})

test('a passing check ends the climb and queues nothing more', async ($, on) => {
  const seen = engine(on, [{ exitCode: 1, stdout: '' }, { exitCode: 0, stdout: 'all green' }])
  await start($, '5 npm test', seen.clock)
  await turn($, 1)
  await turn($, 2)

  expect(seen.prompts.length).toBe(2)
  expect((await climb($, '')).text).toContain('bar met: the check passed at iteration 2')
})

test('the budget ends the climb', async ($, on) => {
  const seen = engine(on, [{ exitCode: 1, stdout: '' }, { exitCode: 1, stdout: '' }])
  await start($, '2 npm test', seen.clock)
  await turn($, 1)
  await turn($, 2)

  expect(seen.prompts.length).toBe(2)
  expect((await climb($, '')).text).toContain('budget spent')
})

test('no better score for the stall limit ends the climb', async ($, on) => {
  const seen = engine(on, [9, 4, 5, 3].map(score => ({ exitCode: 1, stdout: `score: ${score}` })))
  await start($, '20 npm test', seen.clock)
  await climb($, 'stall 3')
  for (const n of [1, 2, 3, 4]) await turn($, n)

  expect(seen.prompts.length).toBe(4)
  expect((await climb($, '')).text).toContain('stalled: 3 iterations without beating the best score (9)')
})

test('an iteration still counts when the engine wraps the prompt in a header', async ($, on) => {
  const seen = engine(on, [{ exitCode: 1, stdout: 'score: 4' }])
  await start($, '5 npm test', seen.clock)
  await $.turn.start({ text: `The climb plugin sent a message:\n${seen.prompts[0]}`, turnId: 'w1' })
  await $.turn.complete({ answer: 'ok', durationMs: 1000, reason: 'answer', isAborted: false, turnId: 'w1' })

  expect(seen.checks.length).toBe(1)
  expect(seen.prompts.length).toBe(2)
})

test("a turn the person starts mid-climb is not an iteration", async ($, on) => {
  const seen = engine(on, [{ exitCode: 1, stdout: '' }])
  await start($, '5 npm test', seen.clock)
  await turn($, 1, false)

  expect(seen.checks.length).toBe(0)
  expect(seen.prompts.length).toBe(1)
  expect((await climb($, '')).text).toContain('0/5 iterations')
})

test("a subagent's turn is not an iteration", async ($, on) => {
  const seen = engine(on, [])
  await start($, '5 npm test', seen.clock)
  await $.turn.start({ text: latest(), turnId: 'm1' })
  await $.turn.complete({ answer: 'ok', durationMs: 1000, reason: 'answer', isAborted: false, turnId: 's1', agentId: 'a1' })

  expect(seen.checks.length).toBe(0)
})

test('a stop pressed while the check runs stands', async ($, on) => {
  let finish = (_: { exitCode: number; stdout: string }) => {}
  const slow = new Promise<{ exitCode: number; stdout: string }>(resolve => (finish = resolve))
  const seen = engine(on, [slow])
  await start($, '5 npm test', seen.clock)

  await $.turn.start({ text: latest(), turnId: 't1' })
  const ending = $.turn.complete({ answer: 'ok', durationMs: 1000, reason: 'answer', isAborted: false, turnId: 't1' })
  await seen.clock.settle()
  await climb($, 'stop')
  finish({ exitCode: 1, stdout: '' })
  await ending

  expect(seen.prompts.length).toBe(1)
  expect((await climb($, '')).text).toContain('stopped by /climb stop')
})

test('an iteration prompt queued before a stop is dropped', async ($, on) => {
  const seen = engine(on, [])
  await start($, '5 npm test', seen.clock)
  await climb($, 'stop')

  const late = await $.prompt.submit({ text: `${MARK} Iteration 3`, origin: { kind: 'plugin', name: 'climb' } } as any)
  expect((late as { drop?: string }).drop).toContain('climb ended')
})

test('a check that never finishes ends the climb instead of counting', async ($, on) => {
  const seen = engine(on, ['hang'])
  await start($, '5 npm test', seen.clock)
  await turn($, 1)

  expect(seen.prompts.length).toBe(1)
  expect((await climb($, '')).text).toContain('did not finish')
})

test('an interrupted turn stops the climb', async ($, on) => {
  const seen = engine(on, [{ exitCode: 1, stdout: '' }])
  await start($, '5 npm test', seen.clock)
  await turn($, 1, true, 'aborted')

  expect(seen.checks.length).toBe(0)
  expect((await climb($, '')).text).toContain('paused')
})

test('protected paths and the log are refused for edit tools; others pass', async ($, on) => {
  const seen = engine(on, [])
  await start($, '5 npm test', seen.clock)
  await climb($, 'protect tests')

  const guarded = await $.tool.call({ tool: 'Edit', file_path: '/work/tests/a.test.ts', old_string: 'a', new_string: 'b' })
  expect(guarded.deny).toContain('protected')
  const dotted = await $.tool.call({ tool: 'Edit', file_path: '/work/src/../tests/b.ts', old_string: 'a', new_string: 'b' })
  expect(dotted.deny).toContain('protected')
  const log = await $.tool.call({ tool: 'Write', file_path: '/work/.climb-log.tsv', content: 'x' })
  expect(log.deny).toContain('protected')
  const free = await $.tool.call({ tool: 'Edit', file_path: '/work/src/a.ts', old_string: 'a', new_string: 'b' })
  expect(free.deny).toBeUndefined()
  const read = await $.tool.call({ tool: 'Read', file_path: '/work/tests/a.test.ts' })
  expect(read.deny).toBeUndefined()
})

test('the band shows the trail and a working Stop on every surface that draws it', async ($, on) => {
  const seen = engine(on, [{ exitCode: 1, stdout: 'score: 2' }])
  await start($, '5 npm test', seen.clock)
  await turn($, 1)

  for (const surface of ['terminal', 'desktop'] as const) {
    const ui = await $.ui.mount({ ...BAND, surface })
    const row = await ui.find({ type: 'Text', text: /climb 1\/5/ })
    expect(row?.text).toContain('○')
    expect(row?.text).toContain('score 2')
    expect(row?.text).toContain('turn 60.0s')
    expect((await ui.find({ type: 'Button', key: 'stop' }))?.key).toBe('stop')
    await ui.unmount()
  }

  const ui = await $.ui.mount({ ...BAND, surface: 'desktop' })
  await ui.press({ key: 'stop' })
  await ui.unmount()
  expect((await climb($, '')).text).toContain('stopped from the band')
})

test('a spend ceiling ends the climb and drops the iteration it had queued', async ($, on) => {
  const seen = engine(on, [])
  await start($, '5 npm test', seen.clock)
  const set = await climb($, 'ceiling 42')
  expect(set.text).toContain('at 30% now')
  await $.session.measure({ context: { window: 1 }, rateLimits: [{ kind: 'seven_day', percentUsed: 41 }], changed: ['rateLimits'] })
  expect((await climb($, '')).text).not.toContain('ended')
  await $.session.measure({ context: { window: 1 }, rateLimits: [{ kind: 'seven_day', percentUsed: 42 }], changed: ['rateLimits'] })
  expect((await climb($, '')).text).toContain('spend ceiling')
})

test('a ceiling outside 0 to 100 is refused', async ($, on) => {
  const seen = engine(on, [])
  await start($, '5 npm test', seen.clock)
  expect((await climb($, 'ceiling 150')).text).toContain('at most 100')
})

const NOT_A_PERSON = [{ kind: 'scheduled-trigger' }, { kind: 'plugin', name: 'other' }, { kind: 'peer' }, { kind: 'sdk' }] as any[]

test('only a person starts or changes a climb; anyone else may only read it', async ($, on) => {
  const seen = engine(on, [])
  on('session.surfaces', () => ({ value: [] }))
  await $.session.start({ cwd: CWD, surface: 'desktop', isInteractive: true })
  for (const origin of NOT_A_PERSON) {
    const run = (args: string) => $.command.run({ command: 'climb', args, origin, presentation: PRESENTATION })
    expect((await run('5 rm -rf x')).text).toContain('Only you')
    expect((await run('protect a')).text).toContain('Only you')
    expect((await run('stall 3')).text).toContain('Only you')
    expect((await run('ceiling 40')).text).toContain('Only you')
    expect((await run('stop')).text).toContain('Only you')
    expect((await run('')).text).toContain('Usage')
  }
  await seen.clock.settle()
  expect(seen.prompts.length).toBe(0)
  expect(seen.checks.length).toBe(0)
  await climb($, '5 npm test')
  await seen.clock.settle()
  const stop = await $.command.run({ command: 'climb', args: 'stop', origin: NOT_A_PERSON[1] as never, presentation: PRESENTATION })
  expect(stop.text).toContain('Only you')
  expect((await climb($, '')).text).toContain('Climb running')
})

test('an sdk origin with a surface is a person', async ($, on) => {
  const seen = engine(on, [])
  on('session.surfaces', () => ({ value: ['desktop'] }))
  await $.session.start({ cwd: CWD, surface: 'desktop', isInteractive: true })
  const answer = await $.command.run({ command: 'climb', args: '5 npm test', origin: { kind: 'sdk' }, presentation: PRESENTATION })
  await seen.clock.settle()
  expect(answer.text).toContain('Climb started')
})

test('a new file under a symlinked directory is protected by where it would land', async ($, on) => {
  const seen = engine(on, [])
  seen.real.set('/work/link', '/work/tests')
  seen.real.set('/work/out', '/elsewhere')
  seen.real.set('/elsewhere', '/elsewhere')
  await start($, '5 npm test', seen.clock)
  await climb($, 'protect tests')
  const viaLink = await $.tool.call({ tool: 'Write', file_path: '/work/link/new/deep.ts', content: 'x' })
  expect(viaLink.deny).toContain('protected')
  const elsewhere = await $.tool.call({ tool: 'Write', file_path: '/work/out/new.ts', content: 'x' })
  expect(elsewhere.deny).toBeUndefined()
})

test('a protected root that is itself a symlink is matched by where it lands', async ($, on) => {
  const seen = engine(on, [])
  seen.real.set('/work/spec', '/vault/spec')
  seen.real.set('/work/alias', '/vault/spec')
  await start($, '5 npm test', seen.clock)
  await climb($, 'protect spec')
  const viaAlias = await $.tool.call({ tool: 'Write', file_path: '/work/alias/new.ts', content: 'x' })
  expect(viaAlias.deny).toContain('protected')
})

test('a person typing the mark is not an iteration; only the mod\'s own prompt is', async ($, on) => {
  const seen = engine(on, [{ exitCode: 1, stdout: '' }])
  await start($, '5 npm test', seen.clock)
  await $.turn.start({ text: `${MARK} I typed this myself`, turnId: 'p1' })
  await $.turn.complete({ answer: 'ok', durationMs: 1000, reason: 'answer', isAborted: false, turnId: 'p1' })
  expect(seen.checks.length).toBe(0)
  await turn($, 2)
  expect(seen.checks.length).toBe(1)
})

test('a prompt the host refuses ends the climb with a reason, not a silent stall', async ($, on) => {
  const seen = engine(on, [{ exitCode: 1, stdout: '' }])
  await start($, '5 npm test', seen.clock)
  seen.knobs.refuse = true
  await turn($, 1)
  await seen.clock.settle()
  expect((await climb($, '')).text).toContain('could not be queued')
  expect((await climb($, '')).text).toContain('Climb ended')
})

test('the first prompt the host refuses also ends the climb', async ($, on) => {
  const seen = engine(on, [])
  seen.knobs.refuse = true
  await start($, '5 npm test', seen.clock)
  expect((await climb($, '')).text).toContain('could not be queued')
})

test('stall 0 is refused', async ($, on) => {
  const seen = engine(on, [])
  await start($, '5 npm test', seen.clock)
  const refused = await climb($, 'stall 0')
  expect(refused.text).toContain('at least 1')
  expect((await climb($, '')).text).toContain('stall 5')
})

test('a long stop reason is cut to one line so the Stop or Hide button stays on screen', async ($, on) => {
  const seen = engine(on, ['hang'])
  await start($, '5 npm test', seen.clock)
  await turn($, 1)
  const ui = await $.ui.mount({ ...BAND, surface: 'desktop' })
  const row = await ui.find({ type: 'Text', text: /climb ended/ })
  expect(row?.props?.wrap).toBe('truncate-end')
  expect(row?.text).not.toContain('\n')
  expect((await ui.find({ type: 'Button', key: 'hide' }))?.key).toBe('hide')
  await ui.unmount()
})

test('a climb started after the session moved folders runs its check where the session is now', async ($, on) => {
  const seen = engine(on, [])
  // The desktop app starts a session in a spare folder under ~/.claude, then moves it to the project.
  on('session.cwd', () => ({ value: '/work' }))
  await $.session.start({ cwd: '/home/me/.claude/spares/spare-abc', surface: 'desktop', isInteractive: true })
  await climb($, '5 npm test')
  await seen.clock.settle()
  await turn($, 1)
  expect(seen.checks[0]).toContain('"cwd":"/work"')
  expect(seen.writes).toContain('/work/.climb-log.tsv')
})
