import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import * as api from "../../shared/tauri-api";
import { liveRewardFixture } from "../../test/fixtures";
import { useLiveRewards } from "./useLiveRewards";

vi.mock("../../shared/tauri-api", () => ({
  collectLiveReward: vi.fn(),
  listPendingLiveRewards: vi.fn(),
  syncLiveRewards: vi.fn(),
}));

describe("useLiveRewards", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    vi.mocked(api.listPendingLiveRewards).mockResolvedValue([liveRewardFixture]);
    vi.mocked(api.syncLiveRewards).mockResolvedValue([liveRewardFixture]);
    vi.mocked(api.collectLiveReward).mockResolvedValue({
      transaction_id: "tx-live",
      cycle_id: liveRewardFixture.cycle_id,
      source_type: "LIVE_REWARD_COLLECTION",
      source_id: liveRewardFixture.event_id,
      counts: { silver: 1, gold: 0, diamond: 0 },
      exact_value: liveRewardFixture.exact_value,
      created_at: "2026-08-17T02:00:11Z",
    });
  });

  it("restores the same pending identity after remount", async () => {
    const onSettled = vi.fn().mockResolvedValue(undefined);
    const first = renderHook(() => useLiveRewards(true, onSettled, 60_000));
    await waitFor(() => expect(first.result.current.events).toEqual([liveRewardFixture]));
    first.unmount();

    const second = renderHook(() => useLiveRewards(true, onSettled, 60_000));
    await waitFor(() => expect(second.result.current.events).toEqual([liveRewardFixture]));
    expect(api.listPendingLiveRewards).toHaveBeenCalledTimes(2);
  });

  it("does not restore a successfully collected event", async () => {
    const onSettled = vi.fn().mockResolvedValue(undefined);
    vi.mocked(api.syncLiveRewards)
      .mockResolvedValueOnce([liveRewardFixture])
      .mockResolvedValue([]);
    const hook = renderHook(() => useLiveRewards(true, onSettled, 60_000));
    await waitFor(() => expect(hook.result.current.events).toEqual([liveRewardFixture]));

    await act(async () => {
      await hook.result.current.collect(liveRewardFixture.event_id);
    });
    expect(hook.result.current.events).toEqual([]);
    expect(onSettled).toHaveBeenCalledTimes(1);
  });

  it("deduplicates concurrent manual and magnet settlement calls", async () => {
    let resolveCollection!: (value: Awaited<ReturnType<typeof api.collectLiveReward>>) => void;
    vi.mocked(api.collectLiveReward).mockImplementation(
      () => new Promise((resolve) => { resolveCollection = resolve; }),
    );
    const onSettled = vi.fn().mockResolvedValue(undefined);
    const hook = renderHook(() => useLiveRewards(true, onSettled, 60_000));
    await waitFor(() => expect(hook.result.current.events).toEqual([liveRewardFixture]));
    vi.mocked(api.syncLiveRewards).mockResolvedValue([]);

    let manual!: Promise<void>;
    let magnet!: Promise<void>;
    act(() => {
      manual = hook.result.current.collect(liveRewardFixture.event_id);
      magnet = hook.result.current.collect(liveRewardFixture.event_id);
    });
    expect(api.collectLiveReward).toHaveBeenCalledTimes(1);
    resolveCollection({
      transaction_id: "tx-race",
      cycle_id: liveRewardFixture.cycle_id,
      source_type: "LIVE_REWARD_COLLECTION",
      source_id: liveRewardFixture.event_id,
      counts: { silver: 1, gold: 0, diamond: 0 },
      exact_value: liveRewardFixture.exact_value,
      created_at: "2026-08-17T02:00:11Z",
    });
    await act(async () => Promise.all([manual, magnet]));
    expect(onSettled).toHaveBeenCalledTimes(1);
  });

  it("treats an event absent after a rejected collect as already settled", async () => {
    vi.mocked(api.collectLiveReward).mockRejectedValue(new Error("already collected"));
    vi.mocked(api.listPendingLiveRewards)
      .mockResolvedValueOnce([liveRewardFixture])
      .mockResolvedValueOnce([]);
    const onSettled = vi.fn().mockResolvedValue(undefined);
    const hook = renderHook(() => useLiveRewards(true, onSettled, 60_000));
    await waitFor(() => expect(hook.result.current.events).toEqual([liveRewardFixture]));
    vi.mocked(api.syncLiveRewards).mockResolvedValue([]);

    await act(async () => hook.result.current.collect(liveRewardFixture.event_id));
    expect(hook.result.current.events).toEqual([]);
    expect(hook.result.current.error).toBeNull();
    expect(onSettled).toHaveBeenCalledTimes(1);
  });

  it("retains a pending event after a real collection error", async () => {
    vi.mocked(api.collectLiveReward).mockRejectedValue(new Error("database unavailable"));
    const onSettled = vi.fn().mockResolvedValue(undefined);
    const hook = renderHook(() => useLiveRewards(true, onSettled, 60_000));
    await waitFor(() => expect(hook.result.current.events).toEqual([liveRewardFixture]));

    await expect(
      act(async () => hook.result.current.collect(liveRewardFixture.event_id)),
    ).rejects.toThrow("database unavailable");
    expect(hook.result.current.events).toEqual([liveRewardFixture]);
  });
});
