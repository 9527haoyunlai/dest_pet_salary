import { render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { PixiGameScene } from "./PixiGameScene";
import type { PixiGameRuntime } from "./pixi/createGameApp";

describe("PixiGameScene lifecycle", () => {
  it("mounts one canvas and destroys its runtime on unmount", async () => {
    const canvas = document.createElement("canvas");
    const runtime: PixiGameRuntime = {
      canvas,
      resize: vi.fn(),
      destroy: vi.fn(),
    };
    const createRuntime = vi.fn().mockResolvedValue(runtime);
    const view = render(<PixiGameScene createRuntime={createRuntime} />);

    const host = view.getByTestId("pixi-game-host");
    await waitFor(() => expect(host.querySelectorAll("canvas")).toHaveLength(1));
    expect(createRuntime).toHaveBeenCalledTimes(1);
    expect(runtime.resize).toHaveBeenCalledTimes(1);

    view.unmount();
    expect(runtime.destroy).toHaveBeenCalledTimes(1);
    expect(host.querySelectorAll("canvas")).toHaveLength(0);
  });

  it("destroys an async runtime that resolves after unmount", async () => {
    let resolveRuntime!: (runtime: PixiGameRuntime) => void;
    const runtime: PixiGameRuntime = {
      canvas: document.createElement("canvas"),
      resize: vi.fn(),
      destroy: vi.fn(),
    };
    const createRuntime = () =>
      new Promise<PixiGameRuntime>((resolve) => {
        resolveRuntime = resolve;
      });
    const view = render(<PixiGameScene createRuntime={createRuntime} />);
    view.unmount();
    resolveRuntime(runtime);

    await waitFor(() => expect(runtime.destroy).toHaveBeenCalledTimes(1));
  });
});
