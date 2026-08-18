import { Container, Graphics, Sprite } from "pixi.js";

import type { LiveRewardEventDto, LiveRewardType } from "../../../../shared/types";
import { getPixiAsset } from "../assets/manifest";
import type { Point } from "../layout/lawnGrid";
import {
  getLiveRewardAssetKey,
  getLiveRewardPosition,
  getLiveRewardSpawnPosition,
} from "./liveRewardPlacement";
import {
  easeInOutCubic,
  easeOutBack,
  LIVE_REWARD_SPAWN_MS,
  LiveRewardPresentation,
  SettlementGate,
  type LiveRewardVisualState,
} from "./liveRewardPresentation";

class LiveRewardSpritePool {
  private readonly available = new Map<LiveRewardType, Sprite[]>();

  acquire(rewardType: LiveRewardType): Sprite {
    const sprites = this.available.get(rewardType);
    const sprite = sprites?.pop() ?? Sprite.from(getPixiAsset(getLiveRewardAssetKey(rewardType)));
    sprite.visible = true;
    sprite.alpha = 1;
    sprite.rotation = 0;
    sprite.anchor.set(0.5);
    return sprite;
  }

  release(rewardType: LiveRewardType, sprite: Sprite): void {
    sprite.removeFromParent();
    sprite.visible = false;
    const sprites = this.available.get(rewardType) ?? [];
    if (sprites.length < 12) {
      sprites.push(sprite);
      this.available.set(rewardType, sprites);
    } else {
      sprite.destroy();
    }
  }

  destroy(): void {
    for (const sprites of this.available.values()) {
      for (const sprite of sprites) sprite.destroy();
    }
    this.available.clear();
  }
}

export const liveRewardSpritePool = new LiveRewardSpritePool();

export interface LiveRewardVisual {
  readonly event: LiveRewardEventDto;
  readonly container: Container;
  readonly state: LiveRewardVisualState;
  readonly readyForMagnet: boolean;
  beginMagnet(target: Point): boolean;
  update(deltaSeconds: number, elapsedSeconds: number): void;
  destroy(): void;
}

function createTypeEffect(rewardType: LiveRewardType): Graphics {
  const effect = new Graphics();
  if (rewardType === "GOLD") {
    effect.circle(0, 0, 24).fill({ color: 0xffdf4d, alpha: 0.2 });
    effect.circle(0, 0, 21).stroke({ color: 0xfff7a0, alpha: 0.72, width: 2 });
  } else if (rewardType === "DIAMOND") {
    effect
      .moveTo(0, -30)
      .lineTo(6, -8)
      .lineTo(29, 0)
      .lineTo(6, 8)
      .lineTo(0, 30)
      .lineTo(-6, 8)
      .lineTo(-29, 0)
      .lineTo(-6, -8)
      .closePath()
      .fill({ color: 0xb9f8ff, alpha: 0.25 });
  }
  return effect;
}

export function createLiveRewardEntity(
  event: LiveRewardEventDto,
  onCollect: (eventId: string) => Promise<void>,
): LiveRewardVisual {
  const landing = getLiveRewardPosition(event);
  const spawn = getLiveRewardSpawnPosition();
  const presentation = new LiveRewardPresentation();
  const settlementGate = new SettlementGate();
  const container = new Container({ label: `live-reward:${event.event_id}` });
  const shadow = new Graphics().ellipse(0, 18, 22, 7).fill({ color: 0x15190a, alpha: 0.45 });
  const effect = createTypeEffect(event.reward_type);
  const sprite = liveRewardSpritePool.acquire(event.reward_type);
  const targetSize = event.reward_type === "DIAMOND" ? 48 : event.reward_type === "GOLD" ? 42 : 37;
  sprite.scale.set(targetSize / Math.max(sprite.texture.width, sprite.texture.height));
  container.position.set(spawn.x, spawn.y);
  container.scale.set(0.35);
  container.eventMode = "static";
  container.cursor = "pointer";
  container.hitArea = { contains: (x, y) => x >= -30 && x <= 30 && y >= -30 && y <= 30 };
  container.addChild(shadow, effect, sprite);

  let magnetStart: Point = landing;
  let magnetTarget: Point = landing;
  let destroyed = false;

  const requestSettlement = async () => {
    if (destroyed || presentation.state === "REMOVED") return;
    presentation.beginSettlement();
    container.alpha = 0.72;
    const settled = await settlementGate.run(() => onCollect(event.event_id));
    if (destroyed) return;
    if (settled) {
      presentation.settlementSucceeded();
      container.visible = false;
    } else {
      presentation.settlementFailed();
      container.position.set(landing.x, landing.y);
      container.scale.set(1);
      container.alpha = 1;
    }
  };

  container.on("pointertap", () => void requestSettlement());

  return {
    event,
    container,
    get state() {
      return presentation.state;
    },
    get readyForMagnet() {
      return presentation.canBeginMagnet(true);
    },
    beginMagnet(target) {
      if (!presentation.beginMagnet()) return false;
      magnetStart = { x: container.x, y: container.y };
      magnetTarget = target;
      return true;
    },
    update(deltaSeconds, elapsedSeconds) {
      const arrived = presentation.advance(deltaSeconds * 1_000);
      if (presentation.state === "SPAWNING") {
        const progress = Math.min(1, presentation.stateElapsedMs / LIVE_REWARD_SPAWN_MS);
        const eased = easeOutBack(progress);
        container.x = spawn.x + (landing.x - spawn.x) * progress;
        container.y = spawn.y + (landing.y - spawn.y) * progress - Math.sin(progress * Math.PI) * 66;
        container.scale.set(0.35 + 0.65 * eased);
        sprite.rotation = (1 - progress) * (event.event_index % 2 === 0 ? 0.45 : -0.45);
      } else if (presentation.state === "IDLE") {
        container.x = landing.x;
        container.y = landing.y + Math.sin(elapsedSeconds * 2.2 + event.event_index) * 3;
        container.scale.set(1);
        container.alpha = 1;
        sprite.rotation = Math.sin(elapsedSeconds * 1.4 + event.event_index) * 0.035;
        effect.alpha = 0.78 + Math.sin(elapsedSeconds * 3.2 + event.event_index) * 0.18;
      } else if (presentation.state === "MAGNETIZING") {
        const progress = easeInOutCubic(presentation.magnetProgress);
        const controlX = (magnetStart.x + magnetTarget.x) / 2;
        const controlY = Math.min(magnetStart.y, magnetTarget.y) - 95;
        const inverse = 1 - progress;
        container.x = inverse ** 2 * magnetStart.x + 2 * inverse * progress * controlX + progress ** 2 * magnetTarget.x;
        container.y = inverse ** 2 * magnetStart.y + 2 * inverse * progress * controlY + progress ** 2 * magnetTarget.y;
        container.scale.set(1 + Math.sin(progress * Math.PI) * 0.22 - progress * 0.18);
        sprite.rotation += deltaSeconds * 8;
      }

      if (arrived) void requestSettlement();
    },
    destroy() {
      if (destroyed) return;
      destroyed = true;
      container.removeAllListeners();
      liveRewardSpritePool.release(event.reward_type, sprite);
      container.destroy({ children: true });
    },
  };
}
