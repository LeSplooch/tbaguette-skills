export type Order = {
  // stable for the session: what /order lists and /order drop takes
  id: number
  // the person's words, trimmed
  text: string
}

declare module 'claude-code' {
  interface PluginState {
    orders: {
      orders: Shaped<Order[]>
      nextId: number
      // the sentence the band offers to keep, or null when it offers nothing
      suggestion: string | null
      // the model's context holds a list still in force, so emptying it is news
      isDelivered: boolean
      // a compaction or a /clear dropped the list, and it rides on the next tool result or prompt
      isOwed: boolean
    }
  }
}
