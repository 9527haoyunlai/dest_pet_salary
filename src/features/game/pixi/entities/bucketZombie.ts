import { Container, Graphics, Sprite } from "pixi.js";

import { getPixiAsset } from "../assets/manifest";
import { getCellCenter } from "../layout/lawnGrid";

export const BUCKET_ZOMBIE_SPAWN_X = 1165;
export const BUCKET_ZOMBIE_STOP_X = getCellCenter(1, 4).x + 16;
export const BUCKET_ZOMBIE_SPEED = 8;

export interface BucketZombieVisual {
  container: Container;
  advance(deltaSeconds: number): void;
}

export function createBucketZombie(): BucketZombieVisual {
  const lane = getCellCenter(1, 4);
  const container = new Container({ label: "fixed-bucket-zombie" });
  const shadow = new Graphics().ellipse(0, 43, 35, 11).fill({ color: 0x101707, alpha: 0.45 });
  const body = Sprite.from(getPixiAsset("zombie.body"));
  const head = Sprite.from(getPixiAsset("zombie.head"));
  const bucket = Sprite.from(getPixiAsset("zombie.bucket"));

  body.anchor.set(0.5);
  body.scale.set(1.45);
  body.position.set(0, 2);
  head.anchor.set(0.5);
  head.scale.set(1.35);
  head.position.set(5, -56);
  bucket.anchor.set(0.5);
  bucket.scale.set(1.18);
  bucket.position.set(8, -88);

  container.position.set(BUCKET_ZOMBIE_SPAWN_X, lane.y + 25);
  container.addChild(shadow, body, head, bucket);

  let elapsed = 0;
  return {
    container,
    advance(deltaSeconds) {
      elapsed += deltaSeconds;
      if (container.x > BUCKET_ZOMBIE_STOP_X) {
        container.x = Math.max(
          BUCKET_ZOMBIE_STOP_X,
          container.x - BUCKET_ZOMBIE_SPEED * deltaSeconds,
        );
      }
      container.y = lane.y + 25 + Math.sin(elapsed * 4) * 2;
      container.rotation = Math.sin(elapsed * 3) * 0.012;
    },
  };
}
