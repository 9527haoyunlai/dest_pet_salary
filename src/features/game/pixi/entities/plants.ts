import { Container, Graphics, Sprite } from "pixi.js";

import { getPixiAsset, type PixiAssetKey } from "../assets/manifest";
import { getCellCenter, type LawnCell } from "../layout/lawnGrid";

export interface FixedPlantDefinition extends LawnCell {
  kind: "marigold" | "magnetshroom" | "wallnut";
}

// Fixed Phase 5A occupants. They are scene decoration, not plantable game state.
export const FIXED_PLANTS: readonly FixedPlantDefinition[] = [
  { kind: "marigold", row: 1, column: 0 },
  { kind: "magnetshroom", row: 1, column: 1 },
  { kind: "wallnut", row: 1, column: 3 },
] as const;

const PLANT_ASSETS: Record<FixedPlantDefinition["kind"], PixiAssetKey> = {
  marigold: "plant.marigold",
  magnetshroom: "plant.magnetshroom",
  wallnut: "plant.wallnut",
};

export function createFixedPlants(): Container[] {
  return FIXED_PLANTS.map((definition) => {
    const position = getCellCenter(definition.row, definition.column);
    const plant = new Container({ label: `fixed-${definition.kind}` });

    const shadow = new Graphics().ellipse(0, 34, 38, 12).fill({ color: 0x14290c, alpha: 0.42 });
    const sprite = Sprite.from(getPixiAsset(PLANT_ASSETS[definition.kind]));
    sprite.anchor.set(0.5, 0.8);

    const targetHeight = definition.kind === "wallnut" ? 105 : 98;
    sprite.scale.set(targetHeight / sprite.texture.height);
    plant.position.set(position.x, position.y + 28);
    plant.addChild(shadow, sprite);

    return plant;
  });
}
