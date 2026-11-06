export type Watch = {
  id: number
  kind: 'usage' | 'command'
  // what the session is told when it fires
  note: string
  expiresAt: number
  // usage: which rate-limit window, and the ceiling in percent
  window?: string
  ceiling?: number
  // command: a shell command that exits 0 when the wait is over
  check?: string
  cwd?: string
  everyMs?: number
  checks?: number
  lastExit?: number | null
}

declare module 'claude-code' {
  interface PluginState {
    watch: { watches: Shaped<Watch[]>; nextId: number }
  }
}
