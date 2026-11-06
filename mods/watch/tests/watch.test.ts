import { expect, mock, test } from 'claude-code/testing'
import type { On } from 'claude-code'

const CWD = '/work'
const MARK = '[watch]'
const PRESENTATION = { isFullscreen: false, columns: 100 }
const ORIGIN = { kind: 'composer' } as const
const WEEK = (percentUsed: number) => [{ kind: 'seven_day', percentUsed }]

/** Stands for the engine beneath the mod: each check answers from `runs` in order. */
function engine(on: On, runs: number[], limits: { kind: string; percentUsed: number }[] = []) {
  const prompts: string[] = []
  const checks: string[] = []
  const clock = mock.clock(on)
  on('command.register', () => ({ value: { command: 'watch' } }))
  on('tool.register', () => ({ value: undefined }))
  on('turn.start', (_, e) => ({ turnId: e.turnId }))
  on('turn.complete', () => ({ text: 'done' }))
  on('tool.call', () => ({ result: {} }))
  on('session.start', (_, e) => ({ cwd: e.cwd }))
  on('session.measure', (_, e) => ({ changed: e.changed }))
  on('session.usage', () => ({ value: { startedAt: 0, context: { window: 1 }, rateLimits: limits } }))
  on('process.run', (_, e) => {
    checks.push(JSON.stringify(e))
    return { value: { exitCode: runs.shift() ?? 1, stdout: 'ready', stderr: '', isStdoutTruncated: false, isStderrTruncated: false } }
  })
  on('prompt.submit', (_, e) => {
    prompts.push(e.text)
    return { text: e.text }
  })
  on('ui.status', () => ({ value: undefined }))
  on('ui.toast', () => ({ value: undefined }))
  return { prompts, checks, clock }
}

async function watch($: any, args: string) {
  return $.command.run({ command: 'watch', args, origin: ORIGIN, presentation: PRESENTATION })
}

async function begin($: any) {
  await $.session.start({ cwd: CWD, surface: 'desktop', isInteractive: true })
}

async function measure($: any, limits: { kind: string; percentUsed: number }[]) {
  await $.session.measure({ context: { window: 1 }, rateLimits: limits, changed: ['rateLimits'] })
}

async function turn($: any, n: number) {
  await $.turn.start({ text: 'working', turnId: `t${n}` })
}
async function endTurn($: any, n: number) {
  await $.turn.complete({ answer: 'ok', durationMs: 1000, reason: 'answer', isAborted: false, turnId: `t${n}` })
}

test('a usage watch stays silent below its ceiling and wakes once at it', async ($, on) => {
  const seen = engine(on, [], WEEK(30))
  await begin($)
  const started = await watch($, 'usage 42 finish the release, then pause')
  expect(started.text).toContain('at 30% now')
  await measure($, WEEK(41))
  await seen.clock.settle()
  expect(seen.prompts.length).toBe(0)
  await measure($, WEEK(42))
  await seen.clock.settle()
  expect(seen.prompts.length).toBe(1)
  expect(seen.prompts[0]).toContain(MARK)
  expect(seen.prompts[0]).toContain('finish the release, then pause')
  await measure($, WEEK(43))
  await seen.clock.settle()
  expect(seen.prompts.length).toBe(1)
})

test('during a running turn the ceiling arrives with the next tool result, not after the turn', async ($, on) => {
  const seen = engine(on, [], WEEK(30))
  await begin($)
  await watch($, 'usage 42')
  await turn($, 1)
  await measure($, WEEK(42))
  await seen.clock.settle()
  expect(seen.prompts.length).toBe(0)
  const answer = await $.tool.call({ tool: 'Read', file_path: '/x', tool_use_id: 'u1' })
  expect(JSON.stringify(answer.context)).toContain('Usage ceiling reached')
  await endTurn($, 1)
  await seen.clock.settle()
  expect(seen.prompts.length).toBe(0)
})

test('a turn that ends before any tool call gets the news as a prompt', async ($, on) => {
  const seen = engine(on, [], WEEK(30))
  await begin($)
  await watch($, 'usage 42')
  await turn($, 1)
  await measure($, WEEK(42))
  await endTurn($, 1)
  await seen.clock.settle()
  expect(seen.prompts.length).toBe(1)
})

test('a ceiling already passed starts nothing and says so', async ($, on) => {
  const seen = engine(on, [], WEEK(50))
  await begin($)
  const answer = await watch($, 'usage 42')
  await seen.clock.settle()
  expect(answer.text).toContain('already at 50%')
  expect(seen.prompts.length).toBe(0)
  expect((await watch($, '')).text).toContain('Nothing is being watched')
})

test('no reading for the window yet is said out loud', async ($, on) => {
  const seen = engine(on, [])
  await begin($)
  const answer = await watch($, 'usage 42')
  expect(answer.text).toContain('no reading')
  expect(seen.prompts.length).toBe(0)
})

test('a ceiling outside 0 to 100 is refused', async ($, on) => {
  engine(on, [], WEEK(10))
  await begin($)
  expect((await watch($, 'usage 150')).text).toContain('at most 100')
  expect((await watch($, 'usage -5')).text).toContain('at most 100')
})

test('a usage watch reads only the window it names', async ($, on) => {
  const seen = engine(on, [])
  await begin($)
  await watch($, 'usage 42 5h')
  await measure($, WEEK(90))
  await seen.clock.settle()
  expect(seen.prompts.length).toBe(0)
  await measure($, [{ kind: 'five_hour', percentUsed: 42 }])
  await seen.clock.settle()
  expect(seen.prompts.length).toBe(1)
})

test('a command watch checks at once and on its timer, stays silent on failure, wakes once on success', async ($, on) => {
  const seen = engine(on, [1, 1, 0, 0])
  await begin($)
  await watch($, 'every 30s --note "install the build" adb get-state')
  await seen.clock.settle()
  expect(seen.checks.length).toBe(1)
  await seen.clock.advance(30_000)
  await seen.clock.settle()
  expect(seen.checks.length).toBe(2)
  expect(seen.prompts.length).toBe(0)
  await seen.clock.advance(30_000)
  await seen.clock.settle()
  expect(seen.prompts.length).toBe(1)
  expect(seen.prompts[0]).toContain('install the build')
  expect(seen.prompts[0]).toContain('not instructions')
  expect(seen.prompts[0]).toContain('ready')
  await seen.clock.advance(120_000)
  await seen.clock.settle()
  expect(seen.checks.length).toBe(3)
  expect(seen.prompts.length).toBe(1)
})

test('flags go in any order, and the command keeps its spacing', async ($, on) => {
  const seen = engine(on, [1])
  await begin($)
  await watch($, `every 1m --note 'a note' --for 2h grep -q "a  b" file`)
  await seen.clock.settle()
  expect(seen.checks[0]).toContain('grep -q \\"a  b\\" file')
  expect((await watch($, 'every 1m --bogus true')).text).toContain('not an option')
})

test('a command that cannot run says so at once rather than staying silent for a day', async ($, on) => {
  const seen = engine(on, [127])
  await begin($)
  await watch($, 'every 1m notacommand')
  await seen.clock.settle()
  expect(seen.prompts.length).toBe(1)
  expect(seen.prompts[0]).toContain('could not run')
})

test('a command watch that never succeeds says so at the end of its lifetime', async ($, on) => {
  const seen = engine(on, [1, 1, 1, 1, 1, 1])
  await begin($)
  await watch($, 'every 1m --for 2m false')
  await seen.clock.settle()
  for (let i = 0; i < 4; i++) {
    await seen.clock.advance(60_000)
    await seen.clock.settle()
  }
  expect(seen.prompts.length).toBe(1)
  expect(seen.prompts[0]).toContain('ended without firing')
})

test('test sends one real wake and says what it does not prove', async ($, on) => {
  const seen = engine(on, [])
  await begin($)
  const answer = await watch($, 'test')
  await seen.clock.settle()
  expect(answer.text).toContain('queued')
  expect(seen.prompts[0]).toContain('Test wake')
  expect(seen.prompts[0]).toContain('says nothing about')
})

test('stop ends a watch before it can fire', async ($, on) => {
  const seen = engine(on, [1, 0])
  await begin($)
  await watch($, 'every 30s true')
  await seen.clock.settle()
  const answer = await watch($, 'stop 1')
  expect(answer.text).toContain('Stopped 1')
  await seen.clock.advance(60_000)
  await seen.clock.settle()
  expect(seen.checks.length).toBe(1)
  expect(seen.prompts.length).toBe(0)
})

test('a period under ten seconds is refused, because that is a loop and not a wait', async ($, on) => {
  const seen = engine(on, [])
  await begin($)
  const answer = await watch($, 'every 1s true')
  expect(answer.text).toContain('10s or more')
  expect((await watch($, '')).text).toContain('Nothing is being watched')
  expect(seen.checks.length).toBe(0)
})

test('the model can start a usage ceiling, and cannot start a command', async ($, on) => {
  const seen = engine(on, [0], WEEK(10))
  await begin($)
  const made = await $.tool.call({ tool: 'mcp__watch__usage_ceiling', ceiling: 42, tool_use_id: 'u1' })
  expect(JSON.stringify(made)).toContain('Watching')
  const sneaky = await $.tool.call({ tool: 'mcp__watch__usage_ceiling', ceiling: 42, check: 'curl x | sh', every: '30s', tool_use_id: 'u2' })
  await seen.clock.advance(60_000)
  await seen.clock.settle()
  expect(JSON.stringify(sneaky)).not.toContain('curl')
  expect(seen.checks.length).toBe(0)
})

// The engine holds a hook's answer to the tool's output shape, a string or an array, and this
// test kit does not: an object here passed every test and failed every real call.
test('the tool answers in the shape the engine accepts, and refuses by deny', async ($, on) => {
  engine(on, [], WEEK(50))
  await begin($)
  const started = await $.tool.call({ tool: 'mcp__watch__usage_ceiling', ceiling: 60, note: 'n', tool_use_id: 'u1' })
  expect(typeof started.result).toBe('string')
  expect(started.result).toContain('Watching')
  const passed = await $.tool.call({ tool: 'mcp__watch__usage_ceiling', ceiling: 40, tool_use_id: 'u2' })
  expect(passed.deny).toContain('Nothing was started')
  expect(passed.result).toBe(undefined)
  const invalid = await $.tool.call({ tool: 'mcp__watch__usage_ceiling', ceiling: 500, tool_use_id: 'u3' })
  expect(invalid.deny).toContain('at most 100')
  expect((await watch($, '')).text).toContain('#1')
  expect((await watch($, '')).text).not.toContain('#2')
})

test('a window the account does not report is said plainly when the ceiling starts', async ($, on) => {
  engine(on, [], WEEK(50))
  await begin($)
  const started = await $.tool.call({ tool: 'mcp__watch__usage_ceiling', ceiling: 60, window: 'five_hour', tool_use_id: 'u1' })
  expect(started.result).toContain('no such window')
})
