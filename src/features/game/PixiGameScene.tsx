import { useEffect, useRef, useState } from "react";

import { createPixiGameRuntime, type PixiGameRuntime } from "./pixi/createGameApp";

interface PixiGameSceneProps {
  createRuntime?: () => Promise<PixiGameRuntime>;
}

export function PixiGameScene({
  createRuntime = createPixiGameRuntime,
}: PixiGameSceneProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    let cancelled = false;
    let runtime: PixiGameRuntime | null = null;
    let resizeObserver: ResizeObserver | null = null;
    host.replaceChildren();
    setLoadError(null);

    void createRuntime()
      .then((createdRuntime) => {
        if (cancelled) {
          createdRuntime.destroy();
          return;
        }

        runtime = createdRuntime;
        host.replaceChildren(createdRuntime.canvas);
        const resize = () => {
          const bounds = host.getBoundingClientRect();
          createdRuntime.resize(bounds.width, bounds.height);
        };
        resize();

        if (typeof ResizeObserver !== "undefined") {
          resizeObserver = new ResizeObserver(resize);
          resizeObserver.observe(host);
        }
      })
      .catch((reason: unknown) => {
        if (!cancelled) setLoadError(String(reason));
      });

    return () => {
      cancelled = true;
      resizeObserver?.disconnect();
      runtime?.destroy();
      host.replaceChildren();
    };
  }, [createRuntime]);

  return (
    <section className="game-scene pixi-game-scene" aria-labelledby="scene-title">
      <h2 id="scene-title" className="visually-hidden">Salary Garden lawn</h2>
      <div ref={hostRef} className="pixi-game-host" data-testid="pixi-game-host" />
      <span className="pixi-scene-badge" aria-hidden="true">PHASE 5A · 3×7 LAWN</span>
      {loadError ? (
        <p className="pixi-scene-error" role="alert">Scene unavailable: {loadError}</p>
      ) : null}
    </section>
  );
}
