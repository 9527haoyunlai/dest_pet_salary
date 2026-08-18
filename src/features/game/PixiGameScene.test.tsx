import { render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PixiGameScene } from "./PixiGameScene";
import type { PixiGameRuntime } from "./pixi/createGameApp";
import { liveRewardFixture } from "../../test/fixtures";

describe("PixiGameScene lifecycle", () => {
  it("mounts one canvas and destroys its runtime on unmount", async () => {
    const canvas = document.createElement("canvas");
    const runtime: PixiGameRuntime = {
      canvas,
      resize: vi.fn(),
      setLiveRewards: vi.fn(),
      setAutoCollectEnabled: vi.fn(),
      destroy: vi.fn(),
    };
    let runtimeCollect!: (eventId: string) => Promise<void>;
    const createRuntime = vi.fn().mockImplementation(async (onCollect) => {
      runtimeCollect = onCollect;
      return runtime;
    });
    const onCollectLiveReward = vi.fn().mockResolvedValue(undefined);
    const view = render(
      <PixiGameScene
        liveRewards={[liveRewardFixture]}
        autoCollectEnabled
        onCollectLiveReward={onCollectLiveReward}
        createRuntime={createRuntime}
      />,
    );

    const host = view.getByTestId("pixi-game-host");
    await waitFor(() => expect(host.querySelectorAll("canvas")).toHaveLength(1));
    expect(createRuntime).toHaveBeenCalledTimes(1);
    expect(runtime.resize).toHaveBeenCalledTimes(1);
    expect(runtime.setLiveRewards).toHaveBeenCalledWith([liveRewardFixture]);
    expect(runtime.setAutoCollectEnabled).toHaveBeenCalledWith(true);
    view.rerender(
      <PixiGameScene
        liveRewards={[liveRewardFixture]}
        autoCollectEnabled={false}
        onCollectLiveReward={onCollectLiveReward}
        createRuntime={createRuntime}
      />,
    );
    expect(runtime.setAutoCollectEnabled).toHaveBeenLastCalledWith(false);
    await runtimeCollect(liveRewardFixture.event_id);
    expect(onCollectLiveReward).toHaveBeenCalledWith(liveRewardFixture.event_id);

    view.unmount();
    expect(runtime.destroy).toHaveBeenCalledTimes(1);
    expect(host.querySelectorAll("canvas")).toHaveLength(0);
  });

  it("destroys an async runtime that resolves after unmount", async () => {
    let resolveRuntime!: (runtime: PixiGameRuntime) => void;
    const runtime: PixiGameRuntime = {
      canvas: document.createElement("canvas"),
      resize: vi.fn(),
      setLiveRewards: vi.fn(),
      setAutoCollectEnabled: vi.fn(),
      destroy: vi.fn(),
    };
    const createRuntime = () =>
      new Promise<PixiGameRuntime>((resolve) => {
        resolveRuntime = resolve;
      });
    const view = render(
      <PixiGameScene
        liveRewards={[]}
        autoCollectEnabled={false}
        onCollectLiveReward={vi.fn()}
        createRuntime={createRuntime}
      />,
    );
    view.unmount();
    resolveRuntime(runtime);

    await waitFor(() => expect(runtime.destroy).toHaveBeenCalledTimes(1));
  });
});
