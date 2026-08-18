import { Application } from "pixi.js";

import { calculateContainLayout, LOGICAL_SCENE_HEIGHT, LOGICAL_SCENE_WIDTH } from "./layout/contain";
import { createLawnScene } from "./scene/createLawnScene";

export interface PixiGameRuntime {
  canvas: HTMLCanvasElement;
  resize(width: number, height: number): void;
  destroy(): void;
}

export async function createPixiGameRuntime(): Promise<PixiGameRuntime> {
  const app = new Application();
  await app.init({
    width: LOGICAL_SCENE_WIDTH,
    height: LOGICAL_SCENE_HEIGHT,
    antialias: true,
    autoDensity: true,
    resolution: Math.min(window.devicePixelRatio || 1, 2),
    backgroundAlpha: 0,
  });

  let scene;
  try {
    scene = await createLawnScene();
  } catch (reason) {
    app.destroy({ removeView: true }, { children: true });
    throw reason;
  }
  app.stage.addChild(scene.root);

  // This ticker owns visual motion only. Payroll, rewards, persistence, and
  // reconciliation remain deterministic Rust concerns and never read it.
  const tick = () => scene.update(app.ticker.deltaMS / 1000);
  app.ticker.add(tick);

  app.canvas.className = "pixi-game-canvas";
  app.canvas.setAttribute("aria-hidden", "true");

  let destroyed = false;
  return {
    canvas: app.canvas,
    resize(width, height) {
      const layout = calculateContainLayout(width, height);
      Object.assign(app.canvas.style, {
        width: `${layout.displayWidth}px`,
        height: `${layout.displayHeight}px`,
        left: `${layout.offsetX}px`,
        top: `${layout.offsetY}px`,
      });
    },
    destroy() {
      if (destroyed) return;
      destroyed = true;
      app.ticker.remove(tick);
      app.destroy({ removeView: true }, { children: true });
    },
  };
}
