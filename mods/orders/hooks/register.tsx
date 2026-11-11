import { atom, read, update } from 'claude-code'
import type { Register } from 'claude-code'

import type { Order } from '../types'

// Change the tag when Order's fields change: it declines what an earlier version stored, in
// $.state across a reload and in $.store across sessions.
const SHAPE = 'orders-1'
const orders = atom({ plugin: 'orders', key: 'orders' } as const, [], { shape: SHAPE })
const nextId = atom({ plugin: 'orders', key: 'nextId' } as const, 1)
const suggestion = atom({ plugin: 'orders', key: 'suggestion' } as const, null)
const isDelivered = atom({ plugin: 'orders', key: 'isDelivered' } as const, false)
const isOwed = atom({ plugin: 'orders', key: 'isOwed' } as const, false)

export const MARK = '[orders]'
export const HEADER =
  `${MARK} Standing orders from the person in this session. They stay in force until the person lifts them. ` +
  'Follow them over any conflicting default, and do not treat this list as a new request:'
export const REPLACES = 'This list replaces any earlier one.'
export const LIFTED = `${MARK} The person lifted every standing order.`
export const MAX_ORDERS = 12
export const MAX_LENGTH = 500
const DAY = 86_400_000
const STORE_LIFETIME = 30 * DAY
export const STORE_PREFIX = 'session:'
const OFFER = 'Keep as a standing order? "'
const MIN_ROOM = 12
const USAGE = [
  'Usage:',
  '  /order <text>      keep a standing order; the model is told it again after every compaction',
  '  /order             list the orders',
  '  /order drop <n>    drop order #n',
  '  /order clear       drop them all',
  '  /order say         tell the model the list again now',
  'Only what you type here becomes an order: nothing the model reads can add one.',
].join('\n')

// High-precision phrases only: a band that offers every "always" would be dismissed unread.
const PHRASES =
  /\b(?:from now on|from here on|going forward|for the rest of (?:this|the) (?:session|conversation|chat)|every time you|each time you|whenever you|don['’]t ever)\b/i
const OPENERS = /^(?:[-*•]\s+)?(?:always\s|never\s(?!mind\b))/i

/** A prompt cut into sentences, one line at a time, a full stop inside a word (`v1.2`) kept. */
export function sentences(text: string): string[] {
  const out: string[] = []
  for (const line of text.split(/\n+/)) {
    const end = /[.!?]+(?=\s|$)/g
    let start = 0
    for (let m = end.exec(line); m !== null; m = end.exec(line)) {
      out.push(line.slice(start, m.index + m[0].length))
      start = m.index + m[0].length
    }
    out.push(line.slice(start))
  }
  return out.map(s => s.trim()).filter(s => s !== '')
}

/** The first sentence that reads as a standing order, or null. */
export function standingSentence(text: string): string | null {
  return sentences(text).find(s => PHRASES.test(s) || OPENERS.test(s)) ?? null
}

const normal = (text: string) => text.trim().replace(/\s+/g, ' ').toLowerCase()

/** The person's own Enter, at the terminal or through Remote Control: never a plugin, a peer or a schedule. */
const isPerson = (origin: { kind: string }) => origin.kind === 'composer' || origin.kind === 'bridge'

/** Why `text` cannot be kept beside `list`, or null when it can. */
export function refusal(text: string, list: readonly Order[]): string | null {
  if (text === '') return 'An order needs words.'
  if (text.length > MAX_LENGTH) return `An order is at most ${MAX_LENGTH} characters, and this one has ${text.length}.`
  const twin = list.find(o => normal(o.text) === normal(text))
  if (twin !== undefined) return `That is already order #${twin.id}.`
  if (list.length >= MAX_ORDERS) return `${MAX_ORDERS} orders are already kept. Drop one first.`
  return null
}

/** The one message that carries the whole list, each order on a line of its own. */
export function ordersMessage(list: readonly Order[]): string {
  return [HEADER, ...list.map((o, i) => `${i + 1}. ${o.text.replace(/\s+/g, ' ')}`), REPLACES].join('\n')
}

export function cut(text: string, room: number): string {
  const width = Math.max(MIN_ROOM, room)
  return text.length <= width ? text : `${text.slice(0, width - 1).trimEnd()}…`
}

function listing(list: readonly Order[]): string {
  return list.length === 0 ? `No standing orders.\n\n${USAGE}` : list.map(o => `#${o.id} ${o.text}`).join('\n')
}

async function status($: any) {
  const count = (await read($, orders)).length
  $.ui.status(count === 0 ? undefined : count === 1 ? '1 order' : `${count} orders`)
}

type Stored = { shape: string; savedAt: number; orders: Order[]; nextId: number }

function asStored(value: unknown): Stored | null {
  const v = value as Partial<Stored> | null | undefined
  if (typeof v !== 'object' || v === null || v.shape !== SHAPE) return null
  if (typeof v.savedAt !== 'number' || typeof v.nextId !== 'number' || !Array.isArray(v.orders)) return null
  return v.orders.every(o => typeof o?.id === 'number' && typeof o?.text === 'string') ? (v as Stored) : null
}

/** Keeps the list past the session's end, under its id, so a resumed session gets it back. */
async function mirror($: any) {
  try {
    const key = STORE_PREFIX + (await $.session.id())
    const list = await read($, orders)
    if (list.length === 0) await $.store.delete(key)
    else await $.store.set(key, { shape: SHAPE, savedAt: await $.clock.now(), orders: list, nextId: await read($, nextId) })
  } catch {
    // The store is a convenience: the session's own state still holds the list.
  }
}

/** Forgets what sessions left in the store more than 30 days ago. */
async function prune($: any) {
  const now = await $.clock.now()
  for (const key of await $.store.keys()) {
    if (!key.startsWith(STORE_PREFIX)) continue
    const savedAt = ((await $.store.get(key)) as { savedAt?: unknown } | null | undefined)?.savedAt
    if (typeof savedAt !== 'number' || now - savedAt > STORE_LIFETIME) await $.store.delete(key)
  }
}

/** A resumed session starts with empty state: its list comes back from the store. */
async function restore($: any) {
  if ((await read($, orders)).length > 0) return
  const kept = asStored(await $.store.get(STORE_PREFIX + (await $.session.id())))
  if (kept === null || kept.orders.length === 0) return
  await update($, orders, (list: Order[]) => (list.length > 0 ? list : kept.orders.slice(0, MAX_ORDERS)))
  await update($, nextId, (n: number) => Math.max(n, kept.nextId))
  // The resumed transcript may have lost the list to a compaction: restate it.
  await update($, isOwed, () => true)
}

/** What the model should be told now: the list, the one-time lift, or nothing. */
async function pendingText($: any): Promise<string | null> {
  const list = await read($, orders)
  if (list.length > 0) return ordersMessage(list)
  return (await read($, isDelivered)) ? LIFTED : null
}

async function markDelivered($: any, text: string) {
  await update($, isDelivered, () => text !== LIFTED)
  await update($, isOwed, () => false)
}

/**
 * How the list reaches the model. `$.session.append` with a `type: "user"` row stores a
 * hidden user-role row the model reads: in an idle session it waits in the conversation for
 * the next request, and in a running turn it joins the requests from the loop's next top. So a
 * change reaches the model without starting a turn, which `$.prompt.submit` would do for a
 * list the person did not ask a reply to, and without waiting for one, as `context` on the next
 * tool result would. Queued on the clock, so the row lands after the command's own rows. An
 * append refused or failed is owed instead, like a compaction's.
 */
function deliverSoon($: any) {
  $.clock.after(0, () => void deliverNow($))
}

async function deliverNow($: any) {
  const text = await pendingText($)
  if (text === null) return
  let isStored = false
  try {
    isStored = (await $.session.append({ message: { type: 'user', content: [{ type: 'text', text }] } })).deny === undefined
  } catch {
    isStored = false
  }
  if (isStored) await markDelivered($, text)
  else await update($, isOwed, () => true)
}

/** Takes an owed restatement. Only the caller that took it gets the text, so it rides once. */
async function takeOwed($: any): Promise<string | null> {
  if (!(await read($, isOwed))) return null
  let wasOwed = false as boolean
  await update($, isOwed, (owed: boolean) => {
    wasOwed = owed
    return false
  })
  return wasOwed ? pendingText($) : null
}

/** Adds an order, or answers why not. The checks run on the list as it is written. */
async function addOrder($: any, raw: string): Promise<Order | string> {
  const text = raw.trim()
  const floor = await read($, nextId)
  let made = '' as Order | string
  await update($, orders, (list: Order[]) => {
    const why = refusal(text, list)
    if (why !== null) {
      made = why
      return list
    }
    const order = { id: Math.max(floor, ...list.map(o => o.id + 1)), text }
    made = order
    return [...list, order]
  })
  const result = made
  if (typeof result !== 'string') await update($, nextId, (n: number) => Math.max(n, result.id + 1))
  return result
}

async function dropOrder($: any, id: number): Promise<Order | null> {
  let dropped = null as Order | null
  await update($, orders, (list: Order[]) => {
    dropped = list.find(o => o.id === id) ?? null
    return list.filter(o => o.id !== id)
  })
  return dropped
}

/** After the list changes: keep it past the session, show the count, tell the model. */
async function changed($: any) {
  await mirror($)
  await status($)
  deliverSoon($)
}

/** The newest prompt from the person replaces the band's offer, with a sentence or with none. */
async function offer($: any, text: string) {
  const sentence = standingSentence(text)
  const shown = sentence !== null && refusal(sentence, await read($, orders)) === null ? sentence : null
  if (shown === null && (await read($, suggestion)) === null) return
  await update($, suggestion, () => shown)
}

/** Keeps the sentence the band showed, once however many presses arrive. */
async function keep($: any, shown: string) {
  let isTaken = false as boolean
  await update($, suggestion, (current: string | null) => {
    isTaken = current === shown
    return isTaken ? null : current
  })
  if (!isTaken) return
  const made = await addOrder($, shown)
  if (typeof made === 'string') {
    $.ui.toast(made)
    return
  }
  await changed($)
}

async function notThis($: any, shown: string) {
  await update($, suggestion, (current: string | null) => (current === shown ? null : current))
}

// The module's own variable starts over on a reload; it only carries the list across a /clear,
// in case the host starts the session's state over with the conversation.
let carried: Order[] | null = null

async function bringBack($: any) {
  if (carried === null) return
  const list = carried
  carried = null
  if (list.length > 0 && (await read($, orders)).length === 0) {
    await update($, orders, (now: Order[]) => (now.length > 0 ? now : list))
    await update($, nextId, (n: number) => Math.max(n, ...list.map(o => o.id + 1)))
    await update($, isOwed, () => true)
    await status($)
  }
  // A /clear goes on under a new session id: the store follows it.
  await mirror($)
}

export const register: Register = on => {
  on('session.start', async ($, e, next) => {
    await $.command.register({
      name: 'order',
      description: 'Keep a standing order for this session, told to the model again after every compaction',
      argumentHint: '<text> | drop <n> | clear | say',
      immediate: true,
    })
    try {
      await prune($)
      await restore($)
    } catch {
      // A store that cannot be read leaves this session's orders to the person.
    }
    await status($)
    return next(e)
  })

  // Only a person changes the orders: no tool is registered for the model, and a run of this
  // command from a plugin or a peer may list or restate them, nothing more. An instruction that
  // persists must not come from a page, a file or a tool result that says "from now on".
  on('command.run', { command: 'order' }, async ($, e) => {
    await bringBack($)
    const args = e.args.trim()
    const verb = args.toLowerCase()
    if (args === '') return { text: listing(await read($, orders)) }
    if (verb === 'say') {
      const count = (await read($, orders)).length
      if (count === 0) return { text: `There are no standing orders to tell the model.\n\n${USAGE}` }
      deliverSoon($)
      return { text: count === 1 ? 'Telling the model the 1 order again.' : `Telling the model the ${count} orders again.` }
    }
    if (!isPerson(e.origin)) return { text: 'Only a person typing /order changes the standing orders.' }
    if (verb === 'clear') {
      let count = 0
      await update($, orders, (list: Order[]) => {
        count = list.length
        return []
      })
      if (count === 0) return { text: 'There were no standing orders to clear.' }
      await changed($)
      return { text: count === 1 ? 'Cleared 1 order.' : `Cleared ${count} orders.` }
    }
    const drop = /^drop\s+#?(\d+)$/i.exec(args)
    if (drop !== null) {
      const dropped = await dropOrder($, Number(drop[1]))
      if (dropped === null) return { text: `There is no order #${drop[1]}.\n\n${listing(await read($, orders))}` }
      await changed($)
      return { text: `Dropped order #${dropped.id}: ${dropped.text}` }
    }
    if (verb === 'drop') return { text: `Name the order to drop by its number, like /order drop 2.\n\n${listing(await read($, orders))}` }
    const made = await addOrder($, args)
    if (typeof made === 'string') return { text: made }
    await changed($)
    return { text: `Kept as order #${made.id}: ${made.text}` }
  })

  // The summary drops what the person said once. The compacted transcript is installed once the
  // whole chain has answered, so a row appended here, even after `next(e)`, could land ahead of
  // it: the list is owed instead, and rides on the first event that is certainly later.
  on('session.compact', async ($, e, next) => {
    const done = await next(e)
    const isMain = e.agentId === undefined && e.trigger !== 'precompute'
    if (isMain && done.skip === undefined && (await read($, orders)).length > 0) await update($, isOwed, () => true)
    return done
  })

  // A /clear starts the conversation over in the same process, and no session.start follows.
  on('session.end', async ($, e, next) => {
    if (e.reason === 'clear') {
      const list = await read($, orders)
      carried = list
      await update($, isDelivered, () => false)
      if (list.length > 0) await update($, isOwed, () => true)
    }
    return next(e)
  })

  // Inside a turn, an owed list goes with the next tool result of the main conversation; a
  // subagent's result would hand it to the subagent.
  on('tool.call', async ($, e, next) => {
    const answer = await next(e)
    if (e.agentId !== undefined || answer.deny !== undefined) return answer
    const text = await takeOwed($)
    if (text === null) return answer
    await markDelivered($, text)
    return { ...answer, context: [...(answer.context ?? []), text] }
  })

  // The prompt always goes through as typed. The person's own prompt may raise the band, and
  // any prompt that is not a slash command carries an owed list beside it.
  on('prompt.submit', async ($, e, next) => {
    if (e.command !== undefined) return next(e)
    if (isPerson(e.origin)) await offer($, e.text)
    await bringBack($)
    const text = await takeOwed($)
    if (text === null) return next(e)
    const entered = await next({ ...e, context: [...(e.context ?? []), text] })
    if (entered.drop !== undefined) await update($, isOwed, () => true)
    else await markDelivered($, text)
    return entered
  })

  on('ui.render', { component: 'AbovePrompt' }, async ($, e, next) => {
    const shown = await read($, suggestion)
    if (shown === null || e.props.hasSurvey) return next(e)
    const { Box, Button, Text } = $.ui.resolve(e)
    return (
      <Box flexDirection="column">
        <Text>{`${OFFER}${cut(shown, e.props.bodyColumns - OFFER.length - 1)}"`}</Text>
        <Box flexDirection="row" gap={1}>
          <Button key="keep" label="Keep" variant="primary" onPress={() => keep($, shown)} />
          <Button key="not-this" label="Not this" onPress={() => notThis($, shown)} />
        </Box>
      </Box>
    )
  })
}
