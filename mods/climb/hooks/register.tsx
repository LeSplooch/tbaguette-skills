import { atom, read, update } from 'claude-code'
import type { Register } from 'claude-code'

import type { Climb, ClimbStep } from '../types'

// Change the tag when Climb's fields change: it declines what an earlier
// version stored, which outlives a reload.
const climb = atom({ plugin: 'climb', key: 'climb' } as const, null, { shape: 'climb-2' })
const isHidden = atom({ plugin: 'climb', key: 'isHidden' } as const, false)

const LOG = '.climb-log.tsv'
// Every prompt this mod queues starts with it, so a turn can tell its own from the person's.
export const MARK = '[climb]'
const WRITE_TOOLS = new Set(['Edit', 'Write', 'NotebookEdit', 'MultiEdit'])
const USAGE = [
  'Usage:',
  '  /climb <budget> [--in <dir>] <check command>   start: run the check after every iteration until it exits 0',
  '                                    --in runs the check, keeps the log and resolves protected paths in <dir>',
  '  /climb protect <path> [...]       refuse edit-tool changes to these paths (relative to the session folder)',
  '  /climb stall <n>                  stop after n iterations with no better score (default 5)',
  '  /climb stop                       end the climb',
  '  /climb                            show the climb and its last steps',
  'A score is read from the check output as "score: <number>" (last one wins), higher is better.',
].join('\n')

export function parseScore(output: string): number | null {
  const all = [...output.matchAll(/score\s*[:=]\s*(-?\d+(?:\.\d+)?)/gi)]
  const last = all[all.length - 1]
  return last === undefined ? null : Number(last[1])
}

export function absolute(cwd: string, path: string): string {
  const joined = path.startsWith('/') ? path : `${cwd.replace(/\/$/, '')}/${path}`
  const parts: string[] = []
  for (const part of joined.split('/')) {
    if (part === '' || part === '.') continue
    if (part === '..') parts.pop()
    else parts.push(part)
  }
  return `/${parts.join('/')}`
}

export function isUnder(target: string, root: string): boolean {
  return target === root || target.startsWith(root === '/' ? '/' : `${root}/`)
}

/** Why the climb should stop after its latest step, or null to continue. */
export function stopReason(c: Climb): string | null {
  const last = c.steps[c.steps.length - 1]
  if (last === undefined) return null
  if (last.exitCode === 0) return `bar met: the check passed at iteration ${last.iteration}`
  if (c.steps.length >= c.budget) return `budget spent: ${c.budget} iterations without the check passing`
  const scored = c.steps.filter(s => s.score !== null)
  if (scored.length > 0) {
    const best = Math.max(...scored.map(s => s.score as number))
    const bestAt = scored.findIndex(s => s.score === best)
    // iterations since the best one, none of them better
    if (scored.length - 1 - bestAt >= c.stall) {
      return `stalled: ${c.stall} iterations without beating the best score (${best})`
    }
  }
  return null
}

function tsv(c: Climb): string {
  const rows = c.steps.map(s => [s.iteration, s.exitCode, s.score ?? '', s.checkMs, s.turnMs].join('\t'))
  return ['iteration\texit\tscore\tcheck_ms\tturn_ms', ...rows].join('\n') + '\n'
}

function tail(text: string, lines: number): string {
  return text.trimEnd().split('\n').slice(-lines).join('\n')
}

function describe(c: Climb): string {
  const head = c.isActive ? 'Climb running' : `Climb ended (${c.stopReason ?? 'stopped'})`
  const rows = c.steps
    .slice(-10)
    .map(s => `  #${s.iteration}  exit ${s.exitCode}  score ${s.score ?? '-'}  check ${s.checkMs} ms  turn ${s.turnMs} ms`)
  return [
    `${head}: \`${c.check}\`, ${c.steps.length}/${c.budget} iterations, stall ${c.stall}`,
    `Protected: ${c.protect.length === 0 ? '(none)' : c.protect.join(', ')}; the log ${LOG} always.`,
    ...rows,
  ].join('\n')
}

function ended(reason: string) {
  return (x: Climb | null) => (x === null ? x : { ...x, isActive: false, isPending: false, stopReason: reason })
}

export const register: Register = on => {
  let cwd = '/'
  // Whether the main-loop turn now running was started by this mod's own prompt.
  let isOwnTurn = false

  on('session.start', async ($, e, next) => {
    cwd = e.cwd
    await $.command.register({
      name: 'climb',
      description: 'Iterate on a goal until a check command passes: the mod runs the check after every iteration',
      argumentHint: '<budget> [--in <dir>] <check command> | protect <path> | stall <n> | stop',
      immediate: true,
    })
    return next(e)
  })

  on('command.run', { command: 'climb' }, async ($, e) => {
    const args = e.args.trim()
    const current = await read($, climb)

    if (args === '') {
      return { text: current === null ? USAGE : describe(current) }
    }
    if (args === 'stop') {
      if (current === null || !current.isActive) return { text: 'No climb is running.' }
      await update($, climb, ended('stopped by /climb stop'))
      $.ui.status(undefined)
      return { text: 'Climb stopped. An iteration prompt already queued is dropped.' }
    }
    const protect = args.match(/^protect\s+(.+)$/)
    if (protect) {
      const paths = (protect[1] ?? '').split(/\s+/)
      if (current === null) return { text: 'Start a climb first, then protect paths.' }
      await update($, climb, c => (c === null ? c : { ...c, protect: [...new Set([...c.protect, ...paths])] }))
      return { text: `Protected: ${paths.join(', ')}. Edit-tool changes to them are refused while the climb runs.` }
    }
    const stall = args.match(/^stall\s+(\d+)$/)
    if (stall) {
      if (current === null) return { text: 'Start a climb first.' }
      await update($, climb, c => (c === null ? c : { ...c, stall: Number(stall[1] ?? '5') }))
      return { text: `Stall limit set to ${stall[1]} iterations.` }
    }
    const start = args.match(/^(\d+)\s+(?:--in\s+(\S+)\s+)?(.+)$/)
    if (!start) return { text: USAGE, exitCode: 1 }

    const fresh: Climb = {
      isActive: true,
      isPending: true,
      check: start[3] ?? '',
      budget: Math.max(1, Number(start[1] ?? '1')),
      stall: current?.stall ?? 5,
      protect: current?.protect ?? [],
      cwd: start[2] === undefined ? cwd : absolute(cwd, start[2]),
      steps: [],
      stopReason: null,
    }
    await update($, climb, () => fresh)
    await update($, isHidden, () => false)
    $.ui.status(`climb 0/${fresh.budget}`)
    // A command hook may not submit (it would wait on the turn it holds): queue it once this hook has returned.
    $.clock.after(0, () => void $.prompt.submit({
      text:
        `${MARK} A climb has started. The climb mod runs \`${fresh.check}\` itself after each of your turns, ` +
        `up to ${fresh.budget} iterations, and only its result counts. Make one change toward making it pass, ` +
        'then end your turn. Do not edit the check or anything it reads to pass it.',
    }))
    return { text: `Climb started on \`${fresh.check}\`, budget ${fresh.budget}. A fresh start discards any earlier climb's history.` }
  })

  // A climb prompt queued before a stop never reaches the model.
  on('prompt.submit', async ($, e, next) => {
    if (e.text.includes(MARK) && e.origin.kind === 'plugin') {
      const c = await read($, climb)
      if (c === null || !c.isActive) return { drop: 'climb: the climb ended before this iteration started' }
    }
    return next(e)
  })

  on('turn.start', async ($, e, next) => {
    if (e.agentId === undefined) {
      // The engine may wrap a plugin's prompt in a header, so the mark is looked for anywhere.
      isOwnTurn = e.text.includes(MARK)
      if (isOwnTurn) await update($, climb, x => (x === null ? x : { ...x, isPending: false }))
    }
    return next(e)
  })

  on('tool.call', async ($, e, next) => {
    if (!WRITE_TOOLS.has(String(e.tool))) return next(e)
    const c = await read($, climb)
    if (c === null || !c.isActive) return next(e)
    const input = e as { file_path?: unknown; notebook_path?: unknown }
    const path = typeof input.file_path === 'string' ? input.file_path
      : typeof input.notebook_path === 'string' ? input.notebook_path
      : null
    if (path === null) return next(e)
    const target = absolute(c.cwd, path)
    const roots = [...c.protect, LOG].map(p => absolute(c.cwd, p))
    // Spellings first; then, where the path exists, its real path, so a link into a protected root is caught too.
    let real = target
    try {
      real = (await $.fs.stat(target, { resolve: true })).realPath ?? target
    } catch {
      // a file that does not exist yet has no real path; its spelling stands
    }
    if (roots.some(r => isUnder(target, r) || isUnder(real, r))) {
      return { deny: `climb: ${path} is protected while the climb runs. Make the check pass without changing it.` }
    }
    return next(e)
  }).catch(($, e, next) => (next.called ? next(e) : { deny: 'climb: its guard failed, so the edit is refused.' }))

  on('turn.complete', async ($, e, next) => {
    const done = await next(e)
    if (e.agentId !== undefined) return done
    const c = await read($, climb)
    if (c === null || !c.isActive) return done
    // A turn the person started while the mod's own prompt waits is theirs, not an iteration.
    if (!isOwnTurn) return done

    if (e.reason !== 'answer') {
      await update($, climb, ended(`paused: the turn ended by ${e.reason}`))
      $.ui.status(undefined)
      $.ui.toast(`climb stopped: the turn ended by ${e.reason}. /climb ${c.budget} ${c.check} starts a fresh climb.`)
      return done
    }

    const turnMs = Math.round(e.durationMs)
    const t0 = await $.clock.now()
    let exitCode = 0
    let output = ''
    let failure: string | null = null
    try {
      const ran = await $.process.run(['sh', '-c', c.check], { cwd: c.cwd, timeoutMs: 600_000 })
      exitCode = ran.exitCode
      output = `${ran.stdout}\n${ran.stderr}`
    } catch (err) {
      failure = `the check did not finish within ten minutes or could not start (${String(err).slice(0, 200)})`
    }
    const checkMs = Math.round((await $.clock.now()) - t0)

    // Built from the state as it is now, not as it was before the check: a stop,
    // a protect or a stall change made while the check ran stands.
    let step: ClimbStep | null = null
    let reason: string | null = null
    const after = await update($, climb, x => {
      if (x === null || !x.isActive) return x
      if (failure !== null) {
        reason = failure
        return ended(failure)(x)
      }
      step = { iteration: x.steps.length + 1, exitCode, score: parseScore(output), checkMs, turnMs }
      const grown: Climb = { ...x, steps: [...x.steps, step] }
      reason = stopReason(grown)
      return reason === null ? { ...grown, isPending: true } : ended(reason)(grown)
    })
    if (after === null) return done
    if (step === null && reason === null) return done // stopped while the check ran

    try {
      await $.fs.write(`${after.cwd.replace(/\/$/, '')}/${LOG}`, tsv(after))
    } catch {
      // The log file is a convenience; the state holds the same rows.
    }

    if (reason !== null) {
      $.ui.status(undefined)
      $.ui.toast(`climb: ${reason}`)
      return done
    }
    const s = step as ClimbStep | null
    if (s === null) return done
    $.ui.status(`climb ${s.iteration}/${after.budget}`)
    const best = after.steps.reduce<number | null>((b, x) => (x.score === null ? b : b === null ? x.score : Math.max(b, x.score)), null)
    void $.prompt.submit({
      text: [
        `${MARK} Iteration ${s.iteration} of ${after.budget}: the check \`${after.check}\` exited ${s.exitCode}` +
          (s.score === null ? '.' : `, score ${s.score} (best ${best}).`),
        'Its last lines:',
        '```',
        tail(output, 30),
        '```',
        'Make one change toward passing it, then end your turn; the check runs again by itself.' +
          (after.protect.length > 0 ? ` Protected, and refused: ${after.protect.join(', ')}.` : ''),
      ].join('\n'),
    })
    return done
  })

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    const c = await read($, climb)
    if (c === null || e.props.hasSurvey || (await read($, isHidden))) return next(e)
    const { Box, Button, Text } = $.ui.resolve(e)
    const last = c.steps[c.steps.length - 1]
    const trail = c.steps
      .slice(-12)
      .map(s => (s.exitCode === 0 ? '●' : '○'))
      .join('')
    const head = c.isActive ? `climb ${c.steps.length}/${c.budget}` : `climb ended: ${c.stopReason ?? 'stopped'}`
    const detail = last === undefined
      ? ' waiting for the first iteration'
      : ` ${trail}  exit ${last.exitCode}${last.score === null ? '' : `  score ${last.score}`}  check ${(last.checkMs / 1000).toFixed(1)}s  turn ${(last.turnMs / 1000).toFixed(1)}s`

    return (
      <Box>
        <Text dimColor>
          {head}
          {detail}{' '}
        </Text>
        {c.isActive ? (
          <Button
            key="stop"
            label="Stop"
            onPress={async () => {
              await update($, climb, ended('stopped from the band'))
              $.ui.status(undefined)
            }}
          />
        ) : (
          <Button key="hide" label="Hide" onPress={() => update($, isHidden, () => true)} />
        )}
      </Box>
    )
  })
}
