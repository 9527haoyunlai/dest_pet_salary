import { describe, expect, it } from "vitest";

import type { LiveRewardEventDto } from "../../../../shared/types";
import { getLiveRewardAssetKey, getLiveRewardPosition } from "./liveRewardPlacement";

const event: LiveRewardEventDto = {
  event_id: "cycle:2024-01-02:1",
  cycle_id: "cycle",
  work_date: "2024-01-02",
  effective_second_boundary: 10,
  event_index: 1,
  reward_type: "SILVER",
  status: "PENDING",
  exact_value: "0.1",
  created_at: "2024-01-02T00:40:10Z",
};

describe("live reward visual DTO mapping", () => {
  it("maps all Rust reward types to semantic manifest keys", () => {
    expect(getLiveRewardAssetKey("SILVER")).toBe("reward.silver");
    expect(getLiveRewardAssetKey("GOLD")).toBe("reward.gold");
    expect(getLiveRewardAssetKey("DIAMOND")).toBe("reward.diamond");
  });

  it("derives stable placement from event index without persisted x/y", () => {
    expect(getLiveRewardPosition(event)).toEqual(getLiveRewardPosition({ ...event }));
    expect(getLiveRewardPosition({ ...event, event_index: 2 })).not.toEqual(
      getLiveRewardPosition(event),
    );
  });

  it("keeps all twelve visible rewards in distinct deterministic slots", () => {
    const positions = Array.from({ length: 12 }, (_, index) =>
      getLiveRewardPosition({ ...event, event_index: index + 1 }),
    );
    expect(new Set(positions.map(({ x, y }) => `${x}:${y}`))).toHaveLength(12);
  });
});
