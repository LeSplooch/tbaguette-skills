// asked: the person put it there with /moves add. The other four are the rows a closing offer
// is harvested from, strongest first.
export type MoveKind = 'asked' | 'surrendered' | 'ruling' | 'deferred' | 'found'

export type Move = {
  // unique everywhere: the session that set it aside and its count there
  uid: string
  // what /moves lists and takes; unique on one project's table
  id: number
  kind: MoveKind
  // the action, a few words, verb first: what an option's label says
  move: string
  // what it costs, and what changes if it is declined; empty on one the person added
  cost: string
  // the project it belongs to
  root: string
  // the session that set it aside
  session: string
  at: number
  // how many offers it was on and not taken
  offered: number
}

declare module 'claude-code' {
  interface PluginState {
    moves: {
      // the project's table as this session last read it
      table: Shaped<Move[]>
      // an edit or a set-aside since the last question or the person's last prompt
      isOwed: boolean
      // who started the turn that is running: a prompt origin's kind, or `moves` for a pick the mod relayed
      asker: string
      // the main conversation's running turn, and the one the close was already held for
      turn: string | null
      heldTurn: string | null
      isGateOn: boolean
      // the band above the prompt: up after a turn, down at the person's next prompt
      isBandUp: boolean
      // the moves the person answered "Not now" for, by uid
      dismissed: string[]
      // the move the person last dropped, kept so the pane can put it back; null once undone or replaced
      lastDropped: Shaped<Move | null>
    }
  }
}
