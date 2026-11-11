// The rest every session shares, as the plugin's store keeps it under `rest`. Another session wrote
// it, perhaps with an older version of this mod, so it is read as unknown and checked first.
export type RestFlag = {
  since: number
  // the rest ends here on its own: at most 24 hours after `since`
  until: number
  // the folder of the session that started it
  by: string
}

// A rest of this session alone (`/rest here`); nothing of it reaches the store.
export type RestHere = {
  since: number
  until: number
}

declare module 'claude-code' {
  interface PluginState {
    // `awake` is the `since` of the shared rest the person woke this session from; `held`, how many
    // of this session's tool calls and prompts wait now.
    rest: { here: Shaped<RestHere | null>; awake: number | null; held: number }
  }
}
