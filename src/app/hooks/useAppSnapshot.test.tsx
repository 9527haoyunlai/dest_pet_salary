import { act, renderHook, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { getAppSnapshot } from "../../shared/tauri-api";
import { snapshotFixture } from "../../test/fixtures";
import { useAppSnapshot } from "./useAppSnapshot";

vi.mock("../../shared/tauri-api", () => ({
  getAppSnapshot: vi.fn(),
}));

describe("useAppSnapshot", () => {
  beforeEach(() => vi.mocked(getAppSnapshot).mockReset());

  it("loads an authoritative snapshot and refreshes it from Rust", async () => {
    vi.mocked(getAppSnapshot).mockResolvedValue(snapshotFixture);
    const { result } = renderHook(() => useAppSnapshot(true, 60_000));

    expect(result.current.loading).toBe(true);
    await waitFor(() => expect(result.current.snapshot).toEqual(snapshotFixture));
    expect(result.current.loading).toBe(false);

    const refreshed = {
      ...snapshotFixture,
      current_local_time: "2026-08-17T10:00:01+08:00",
    };
    vi.mocked(getAppSnapshot).mockResolvedValue(refreshed);
    await act(async () => result.current.refresh());

    expect(result.current.snapshot).toEqual(refreshed);
    expect(getAppSnapshot).toHaveBeenCalledTimes(2);
  });
});
