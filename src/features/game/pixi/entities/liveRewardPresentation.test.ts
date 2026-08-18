import { describe, expect, it, vi } from "vitest";

import {
  LIVE_REWARD_MANUAL_LINGER_MS,
  LIVE_REWARD_SPAWN_MS,
  MAX_VISIBLE_LIVE_REWARDS,
  LiveRewardPresentation,
  SettlementGate,
  selectOldestMagnetCandidate,
} from "./liveRewardPresentation";

describe("Phase 5C live reward presentation", () => {
  function idlePresentation() {
    const presentation = new LiveRewardPresentation();
    presentation.advance(LIVE_REWARD_SPAWN_MS);
    return presentation;
  }

  it("does not make an idle reward magnet-ready while auto collect is off", () => {
    const presentation = idlePresentation();
    presentation.advance(LIVE_REWARD_MANUAL_LINGER_MS);
    expect(presentation.canBeginMagnet(false)).toBe(false);
    expect(presentation.state).toBe("IDLE");
  });

  it("makes an idle reward magnet-ready only after the manual linger", () => {
    const presentation = idlePresentation();
    presentation.advance(LIVE_REWARD_MANUAL_LINGER_MS - 1);
    expect(presentation.canBeginMagnet(true)).toBe(false);
    presentation.advance(1);
    expect(presentation.canBeginMagnet(true)).toBe(true);
    expect(presentation.beginMagnet()).toBe(true);
  });

  it("allows manual settlement during the linger window", () => {
    const presentation = idlePresentation();
    expect(presentation.beginSettlement()).toBe(true);
    expect(presentation.state).toBe("SETTLING");
  });

  it("shares one frontend settlement across a manual and magnet race", async () => {
    let resolve!: () => void;
    const settle = vi.fn(() => new Promise<void>((done) => { resolve = done; }));
    const gate = new SettlementGate();
    const manual = gate.run(settle);
    const magnet = gate.run(settle);
    expect(settle).toHaveBeenCalledTimes(1);
    resolve();
    await expect(Promise.all([manual, magnet])).resolves.toEqual([true, true]);
  });

  it("removes a reward only after settlement success", () => {
    const presentation = idlePresentation();
    presentation.beginSettlement();
    presentation.settlementSucceeded();
    expect(presentation.state).toBe("REMOVED");
  });

  it("restores a failed settlement to idle for a later retry", () => {
    const presentation = idlePresentation();
    presentation.beginMagnet();
    presentation.beginSettlement();
    presentation.settlementFailed();
    expect(presentation.state).toBe("IDLE");
    expect(presentation.idleElapsedMs).toBe(0);
  });

  it("creates clean presentation state after remount without stale progress", () => {
    const first = idlePresentation();
    first.advance(LIVE_REWARD_MANUAL_LINGER_MS);
    const remounted = new LiveRewardPresentation();
    expect(remounted.state).toBe("SPAWNING");
    expect(remounted.idleElapsedMs).toBe(0);
  });

  it("retains the Phase 5B twelve-reward screen cap", () => {
    expect(MAX_VISIBLE_LIVE_REWARDS).toBe(12);
  });

  it("selects the oldest eligible reward, with event index as tie-breaker", () => {
    const candidates = [
      { readyForMagnet: true, event: { created_at: "2026-08-17T02:01:00Z", event_index: 4 } },
      { readyForMagnet: false, event: { created_at: "2026-08-17T01:00:00Z", event_index: 1 } },
      { readyForMagnet: true, event: { created_at: "2026-08-17T02:01:00Z", event_index: 2 } },
    ];
    expect(selectOldestMagnetCandidate(candidates)).toBe(candidates[2]);
  });
});
