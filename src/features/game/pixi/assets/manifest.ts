import lawnBackground from "../../../../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/backgrounds/background1.PNG?url";
import sodThreeRows from "../../../../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/Sod/sod3row.PNG?url";
import magnetshroom from "../../../../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Plants/Magnetshroom/Magnetshroom_head1.PNG?url";
import marigold from "../../../../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Plants/Marigold/Marigold_head.PNG?url";
import wallnut from "../../../../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Plants/Wallnut/Wallnut_body.PNG?url";
import zombieBody from "../../../../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Zombies/ZombieBody/Zombie_body.PNG?url";
import zombieHead from "../../../../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Zombies/ZombieBody/Zombie_head/Zombie_head.PNG?url";
import zombieBucket from "../../../../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Zombies/NormalZombie/Zombie_bucket/Zombie_bucket1.PNG?url";

export const pixiAssetManifest = {
  "scene.lawn": lawnBackground,
  "scene.sodThreeRows": sodThreeRows,
  "plant.marigold": marigold,
  "plant.magnetshroom": magnetshroom,
  "plant.wallnut": wallnut,
  "zombie.body": zombieBody,
  "zombie.head": zombieHead,
  "zombie.bucket": zombieBucket,
} as const;

export type PixiAssetKey = keyof typeof pixiAssetManifest;

export function getPixiAsset(key: PixiAssetKey): string {
  return pixiAssetManifest[key];
}

export const pixiAssetUrls = Object.values(pixiAssetManifest);
