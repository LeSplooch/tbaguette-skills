export type ClimbStep = {
  iteration: number
  exitCode: number
  score: number | null
  checkMs: number
  turnMs: number
}

export type Climb = {
  isActive: boolean
  // the mod has queued its own prompt and not yet seen that turn start
  isPending: boolean
  check: string
  budget: number
  stall: number
  // end the climb when this rate-limit window reaches this percent
  ceiling: number | null
  ceilingWindow: string
  protect: string[]
  cwd: string
  steps: ClimbStep[]
  stopReason: string | null
}

declare module 'claude-code' {
  interface PluginState {
    climb: {
      climb: Shaped<Climb | null>
      isHidden: boolean
      // the prompts this mod queued that have not started a turn yet
      ownPrompts: string[]
    }
  }
}
