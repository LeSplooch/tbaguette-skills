import { expect, mock, test } from 'claude-code/testing'
import type { On } from 'claude-code'

const CWD = '/work'
const MARK = '[climb]'
const PRESENTATION = { isFullscreen: false, columns: 100 }
const ORIGIN = { kind: 'composer' } as const
const BAND = {
  plugin: 'climb',
  component: 'AbovePrompt',
  props: { hasSurvey: false, isWorking: false, maxRows: 3, bodyColumns: 100, scroll: { offset: 0, bodyRows: 3 } },
} as const

type Run = { exitCode: number; stdout: string } | 'hang' | Promise<{ exitCode: number; stdout: string }>

/** Stands for the engine beneath the mod: each check run answers from `runs` in order. */
function engine(on: On, runs: Run[]) {
  const prompts: string[] = []
  const checks: string[] = []
  const clock = mock.clock(on)
  on('command.register', () => ({ value: { command: 'climb' } }))
  on('session.start', (_, e) => ({ cwd: e.cwd }))
  on('process.run', async (_, e) => {
    checks.push(String((e as { argv?: unknown }).argv ?? ''))
    const run = runs.shift() ?? { exitCode: 1, stdout: '' }
    if (run === 'hang') throw new Error('timed out')
    const r = await run
    return { value: { ...r, stderr: '', isStdoutTruncated: false, isStderrTruncated: false } }
  })
  on('fs.write', () => ({ value: undefined }))
  on('fs.stat', () => {
    throw new Error('no such file')
  })
  on('prompt.submit', (_, e) => {
    prompts.push(e.text)
    return { text: e.text }
  })
  on('turn.start', (_, e) => ({ turnId: e.turnId }))
  on('turn.complete', () => ({ text: 'done' }))
  on('tool.call', () => ({ result: {} }))
  on('ui.status', () => ({ value: undefined }))
  on('ui.toast', () => ({ value: undefined }))
  return { prompts, checks, clock }
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
  await $.turn.start({ text: own ? `${MARK} go` : 'a question of my own', turnId: `t${n}` })
  await $.turn.complete({ answer: 'ok', durationMs: 60_000, reason, isAborted: reason !== 'answer', turnId: `t${n}` })
}

test('starting queues the first iteration itself', async ($, on) => {
  const seen = engine(on, [])
  const started = await start($, '5 npm test', seen.clock)
  expect(started.text).toContain('Climb started')
  expect(seen.prompts.length).toBe(1)
  expect(seen.prompts[0]?.startsWith(MARK)).toBe(true)
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
  await $.turn.start({ text: `${MARK} go`, turnId: 'm1' })
  await $.turn.complete({ answer: 'ok', durationMs: 1000, reason: 'answer', isAborted: false, turnId: 's1', agentId: 'a1' })

  expect(seen.checks.length).toBe(0)
})

test('a stop pressed while the check runs stands', async ($, on) => {
  let finish = (_: { exitCode: number; stdout: string }) => {}
  const slow = new Promise<{ exitCode: number; stdout: string }>(resolve => (finish = resolve))
  const seen = engine(on, [slow])
  await start($, '5 npm test', seen.clock)

  await $.turn.start({ text: `${MARK} go`, turnId: 't1' })
  const ending = $.turn.complete({ answer: 'ok', durationMs: 1000, reason: 'answer', isAborted: false, turnId: 't1' })
  await new Promise(r => setTimeout(r, 20))
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
