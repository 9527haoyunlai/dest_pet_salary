import { Container } from "pixi.js";

export interface LawnSceneLayers {
  backgroundLayer: Container;
  gridLayer: Container;
  plantLayer: Container;
  zombieLayer: Container;
  projectileLayer: Container;
  rewardLayer: Container;
  effectLayer: Container;
  debugLayer: Container;
}

export function createLawnSceneLayers(): LawnSceneLayers {
  return {
    backgroundLayer: new Container({ label: "backgroundLayer" }),
    gridLayer: new Container({ label: "gridLayer" }),
    plantLayer: new Container({ label: "plantLayer" }),
    zombieLayer: new Container({ label: "zombieLayer" }),
    projectileLayer: new Container({ label: "projectileLayer" }),
    rewardLayer: new Container({ label: "rewardLayer" }),
    effectLayer: new Container({ label: "effectLayer" }),
    debugLayer: new Container({ label: "debugLayer" }),
  };
}
