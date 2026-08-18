import { Container, Graphics, Sprite } from "pixi.js";

import type { LiveRewardEventDto } from "../../../../shared/types";
import { getPixiAsset } from "../assets/manifest";
import { getLiveRewardAssetKey, getLiveRewardPosition } from "./liveRewardPlacement";

export interface LiveRewardVisual {
  container: Container;
  update(elapsedSeconds: number): void;
}

export function createLiveRewardEntity(
  event: LiveRewardEventDto,
  onCollect: (eventId: string) => Promise<void>,
): LiveRewardVisual {
  const position = getLiveRewardPosition(event);
  const container = new Container({ label: `live-reward:${event.event_id}` });
  const shadow = new Graphics().ellipse(0, 18, 22, 7).fill({ color: 0x15190a, alpha: 0.45 });
  const sprite = Sprite.from(getPixiAsset(getLiveRewardAssetKey(event.reward_type)));
  sprite.anchor.set(0.5);
  const targetSize = event.reward_type === "DIAMOND" ? 45 : 38;
  sprite.scale.set(targetSize / Math.max(sprite.texture.width, sprite.texture.height));
  container.position.set(position.x, position.y);
  container.eventMode = "static";
  container.cursor = "pointer";
  container.hitArea = { contains: (x, y) => x >= -28 && x <= 28 && y >= -28 && y <= 28 };
  container.addChild(shadow, sprite);

  let settling = false;
  container.on("pointertap", () => {
    if (settling) return;
    settling = true;
    container.alpha = 0.65;
    void onCollect(event.event_id).catch(() => {
      settling = false;
      container.alpha = 1;
    });
  });

  return {
    container,
    update(elapsedSeconds) {
      container.y = position.y + Math.sin(elapsedSeconds * 2.2 + event.event_index) * 3;
      sprite.rotation = Math.sin(elapsedSeconds * 1.4 + event.event_index) * 0.035;
    },
  };
}
