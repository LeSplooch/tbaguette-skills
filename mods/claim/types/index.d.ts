// What `git status` showed at a moment no write-capable call of this session was in flight.
export type ClaimFingerprint = {
  // path from the root -> `<status code>|<mtimeMs>|<size>`, `<status code>|gone`, or the code alone when capped
  entries: Record<string, string>
  // more than 300 entries: compared by status code alone
  capped: boolean
}

declare module 'claude-code' {
  interface PluginState {
    claim: {
      isOn: boolean
      // path from the root -> its status code, for every dirty file another writer changed
      foreign: Shaped<Record<string, string>>
      // paths the person took as this session's
      taken: Shaped<string[]>
      // paths this session's Edit, Write and NotebookEdit calls wrote
      mine: Shaped<string[]>
      baseline: Shaped<ClaimFingerprint | null>
      // background commands this session started that have not been reported ended
      background: Shaped<string[]>
    }
  }
}
