import { Assets, Container, Graphics, Sprite, Text } from "pixi.js";

import { pixiAssetUrls, getPixiAsset } from "../assets/manifest";
import { createBucketZombie } from "../entities/bucketZombie";
import { createFixedPlants } from "../entities/plants";
import { LAWN_GRID } from "../layout/lawnGrid";
import { LOGICAL_SCENE_HEIGHT, LOGICAL_SCENE_WIDTH } from "../layout/contain";
import { createLawnSceneLayers } from "./layers";

export const SHOW_DEBUG_GRID = false;

export interface LawnScene {
  root: Container;
  update(deltaSeconds: number): void;
}

function createGrid(showDebug: boolean): Container {
  const grid = new Container({ label: "3x7-lawn-grid" });
  for (let row = 0; row < LAWN_GRID.rows; row += 1) {
    for (let column = 0; column < LAWN_GRID.columns; column += 1) {
      const x = LAWN_GRID.originX + column * LAWN_GRID.cellWidth;
      const y = LAWN_GRID.originY + row * LAWN_GRID.cellHeight;
      const tint = (row + column) % 2 === 0 ? 0xb9e66c : 0x86c653;
      grid.addChild(
        new Graphics()
          .rect(x, y, LAWN_GRID.cellWidth, LAWN_GRID.cellHeight)
          .fill({ color: tint, alpha: 0.055 }),
      );

      if (showDebug) {
        grid.addChild(
          new Graphics()
            .rect(x, y, LAWN_GRID.cellWidth, LAWN_GRID.cellHeight)
            .stroke({ color: 0xffffff, alpha: 0.7, width: 2 }),
        );
        const label = new Text({
          text: `${row},${column}`,
          style: { fill: 0xffffff, fontFamily: "monospace", fontSize: 15 },
        });
        label.position.set(x + 6, y + 5);
        grid.addChild(label);
      }
    }
  }
  return grid;
}

export async function createLawnScene(): Promise<LawnScene> {
  await Assets.load(pixiAssetUrls);

  const root = new Container({ label: "salary-garden-lawn" });
  const layers = createLawnSceneLayers();
  root.addChild(
    layers.backgroundLayer,
    layers.gridLayer,
    layers.plantLayer,
    layers.zombieLayer,
    layers.projectileLayer,
    layers.rewardLayer,
    layers.effectLayer,
    layers.debugLayer,
  );

  const background = Sprite.from(getPixiAsset("scene.lawn"));
  background.width = LOGICAL_SCENE_WIDTH;
  background.height = LOGICAL_SCENE_HEIGHT;
  layers.backgroundLayer.addChild(background);

  layers.gridLayer.addChild(createGrid(false));
  if (SHOW_DEBUG_GRID) {
    layers.debugLayer.addChild(createGrid(true));
  }

  const plants = createFixedPlants();
  layers.plantLayer.addChild(...plants);
  const basePlantY = plants.map((plant) => plant.y);
  const bucketZombie = createBucketZombie();
  layers.zombieLayer.addChild(bucketZombie.container);

  let elapsed = 0;
  return {
    root,
    update(deltaSeconds) {
      elapsed += deltaSeconds;
      plants.forEach((plant, index) => {
        plant.y = basePlantY[index] + Math.sin(elapsed * 1.7 + index) * 1.6;
        plant.scale.y = 1 + Math.sin(elapsed * 1.5 + index) * 0.012;
      });
      bucketZombie.advance(deltaSeconds);
    },
  };
}
