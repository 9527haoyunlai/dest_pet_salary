import type { LiveRewardEventDto, LiveRewardType } from "../../../../shared/types";
import type { PixiAssetKey } from "../assets/manifest";
import { getCellCenter } from "../layout/lawnGrid";

export function getLiveRewardAssetKey(rewardType: LiveRewardType): PixiAssetKey {
  switch (rewardType) {
    case "SILVER":
      return "reward.silver";
    case "GOLD":
      return "reward.gold";
    case "DIAMOND":
      return "reward.diamond";
  }
}

export function getLiveRewardPosition(event: LiveRewardEventDto) {
  const marigold = getCellCenter(1, 0);
  const slot = (event.event_index - 1) % 12;
  return {
    x: marigold.x - 74 + (slot % 4) * 48,
    y: marigold.y - 92 + Math.floor(slot / 4) * 47,
  };
}

export function getLiveRewardSpawnPosition() {
  const marigold = getCellCenter(1, 0);
  return { x: marigold.x, y: marigold.y + 4 };
}

export function getMagnetCollectionPosition() {
  const magnet = getCellCenter(1, 1);
  return { x: magnet.x, y: magnet.y + 2 };
}
