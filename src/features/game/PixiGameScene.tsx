import { useEffect, useRef, useState } from "react";

import { createPixiGameRuntime, type PixiGameRuntime } from "./pixi/createGameApp";
import type { LiveRewardEventDto } from "../../shared/types";

interface PixiGameSceneProps {
  liveRewards: LiveRewardEventDto[];
  autoCollectEnabled: boolean;
  onCollectLiveReward: (eventId: string) => Promise<void>;
  createRuntime?: (
    onCollect: (eventId: string) => Promise<void>,
  ) => Promise<PixiGameRuntime>;
}

export function PixiGameScene({
  liveRewards,
  autoCollectEnabled,
  onCollectLiveReward,
  createRuntime = createPixiGameRuntime,
}: PixiGameSceneProps) {
  const hostRef = useRef<HTMLDivElement>(null);
  const runtimeRef = useRef<PixiGameRuntime | null>(null);
  const rewardsRef = useRef(liveRewards);
  const autoCollectRef = useRef(autoCollectEnabled);
  const collectRef = useRef(onCollectLiveReward);
  const [loadError, setLoadError] = useState<string | null>(null);

  useEffect(() => {
    const host = hostRef.current;
    if (!host) return;

    let cancelled = false;
    let runtime: PixiGameRuntime | null = null;
    let resizeObserver: ResizeObserver | null = null;
    host.replaceChildren();
    setLoadError(null);

    void createRuntime((eventId) => collectRef.current(eventId))
      .then((createdRuntime) => {
        if (cancelled) {
          createdRuntime.destroy();
          return;
        }

        runtime = createdRuntime;
        runtimeRef.current = createdRuntime;
        createdRuntime.setLiveRewards(rewardsRef.current);
        createdRuntime.setAutoCollectEnabled(autoCollectRef.current);
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
      runtimeRef.current = null;
      host.replaceChildren();
    };
  }, [createRuntime]);

  useEffect(() => {
    rewardsRef.current = liveRewards;
    runtimeRef.current?.setLiveRewards(liveRewards);
  }, [liveRewards]);

  useEffect(() => {
    autoCollectRef.current = autoCollectEnabled;
    runtimeRef.current?.setAutoCollectEnabled(autoCollectEnabled);
  }, [autoCollectEnabled]);

  useEffect(() => {
    collectRef.current = onCollectLiveReward;
  }, [onCollectLiveReward]);

  return (
    <section className="game-scene pixi-game-scene" aria-labelledby="scene-title">
      <h2 id="scene-title" className="visually-hidden">Salary Garden lawn</h2>
      <div ref={hostRef} className="pixi-game-host" data-testid="pixi-game-host" />
      <span className="pixi-scene-badge" aria-hidden="true">PHASE 5C · MAGNET COLLECTION</span>
      {loadError ? (
        <p className="pixi-scene-error" role="alert">Scene unavailable: {loadError}</p>
      ) : null}
    </section>
  );
}
