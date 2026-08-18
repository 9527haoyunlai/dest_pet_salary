import { describe, expect, it } from "vitest";

import { getPixiAsset, pixiAssetManifest, pixiAssetUrls } from "./manifest";

describe("Pixi asset manifest", () => {
  it("resolves assets through semantic keys", () => {
    expect(getPixiAsset("plant.wallnut")).toBe(pixiAssetManifest["plant.wallnut"]);
    expect(getPixiAsset("zombie.bucket")).toMatch(/Zombie_bucket1\.PNG/);
    expect(pixiAssetUrls).toHaveLength(Object.keys(pixiAssetManifest).length);
  });
});
