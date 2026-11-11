// What the host pushed, with when it was read (`$.clock.now()` milliseconds).
export type MeterContext = { percent: number; tokens: number | null; window: number; at: number }
export type MeterWindow = { kind: string; percentUsed: number; resetsAt?: string; at: number }
export type MeterCost = { usd: number; at: number }
// The one estimate: how long the session has been idle and what the next send re-reads.
export type MeterCold = { idleMs: number; tokens: number; at: number }

export type MeterReadings = {
  // null, or empty, where the engine has no figure: never zeroed
  context: MeterContext | null
  windows: MeterWindow[]
  cost: MeterCost | null
  cold: MeterCold | null
}

// A line crossed and not yet re-armed (`context@80`, `five_hour@95`, `cache@0`).
// A line absent from the list is armed.
export type MeterCrossing = { id: string; source: string; isHidden: boolean }

export type MeterLifetime = {
  ms: number
  label: 'assumed' | 'observed'
  // what it was derived from, in words, for /meter
  evidence: string
}

declare module 'claude-code' {
  interface PluginState {
    meter: {
      readings: Shaped<MeterReadings>
      crossings: Shaped<MeterCrossing[]>
      lifetime: Shaped<MeterLifetime>
      isQuiet: boolean
      // when the last main-loop turn ended; null while one runs or before the first
      lastActivity: number | null
    }
  }
}
