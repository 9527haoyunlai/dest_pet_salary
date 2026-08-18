export type LiveRewardVisualState =
  | "SPAWNING"
  | "IDLE"
  | "MAGNETIZING"
  | "SETTLING"
  | "REMOVED";

export const LIVE_REWARD_SPAWN_MS = 700;
export const LIVE_REWARD_MANUAL_LINGER_MS = 2_500;
export const LIVE_REWARD_MAGNET_MS = 700;
export const MAX_VISIBLE_LIVE_REWARDS = 12;

export class LiveRewardPresentation {
  state: LiveRewardVisualState = "SPAWNING";
  stateElapsedMs = 0;
  idleElapsedMs = 0;
  magnetProgress = 0;

  advance(deltaMs: number): boolean {
    const safeDelta = Math.max(0, deltaMs);
    this.stateElapsedMs += safeDelta;

    if (this.state === "SPAWNING" && this.stateElapsedMs >= LIVE_REWARD_SPAWN_MS) {
      this.state = "IDLE";
      this.stateElapsedMs -= LIVE_REWARD_SPAWN_MS;
      this.idleElapsedMs = this.stateElapsedMs;
    } else if (this.state === "IDLE") {
      this.idleElapsedMs += safeDelta;
    } else if (this.state === "MAGNETIZING") {
      this.magnetProgress = Math.min(1, this.stateElapsedMs / LIVE_REWARD_MAGNET_MS);
      return this.magnetProgress >= 1;
    }

    return false;
  }

  canBeginMagnet(autoCollectEnabled: boolean): boolean {
    return autoCollectEnabled && this.state === "IDLE" && this.idleElapsedMs >= LIVE_REWARD_MANUAL_LINGER_MS;
  }

  beginMagnet(): boolean {
    if (this.state !== "IDLE") return false;
    this.state = "MAGNETIZING";
    this.stateElapsedMs = 0;
    this.magnetProgress = 0;
    return true;
  }

  beginSettlement(): boolean {
    if (this.state === "REMOVED" || this.state === "SETTLING") return false;
    this.state = "SETTLING";
    this.stateElapsedMs = 0;
    return true;
  }

  settlementSucceeded(): void {
    this.state = "REMOVED";
    this.stateElapsedMs = 0;
  }

  settlementFailed(): void {
    this.state = "IDLE";
    this.stateElapsedMs = 0;
    this.idleElapsedMs = 0;
    this.magnetProgress = 0;
  }
}

export class SettlementGate {
  private inFlight: Promise<boolean> | null = null;

  run(settle: () => Promise<void>): Promise<boolean> {
    if (this.inFlight) return this.inFlight;
    const request = settle().then(
      () => true,
      () => false,
    );
    this.inFlight = request.finally(() => {
      this.inFlight = null;
    });
    return this.inFlight;
  }
}

export function easeOutBack(value: number): number {
  const c1 = 1.70158;
  const c3 = c1 + 1;
  const shifted = value - 1;
  return 1 + c3 * shifted ** 3 + c1 * shifted ** 2;
}

export function easeInOutCubic(value: number): number {
  return value < 0.5 ? 4 * value ** 3 : 1 - (-2 * value + 2) ** 3 / 2;
}

interface MagnetCandidate {
  readyForMagnet: boolean;
  event: { created_at: string; event_index: number };
}

export function selectOldestMagnetCandidate<T extends MagnetCandidate>(
  candidates: Iterable<T>,
): T | undefined {
  return [...candidates]
    .filter((candidate) => candidate.readyForMagnet)
    .sort((left, right) =>
      left.event.created_at.localeCompare(right.event.created_at) ||
      left.event.event_index - right.event.event_index,
    )[0];
}
