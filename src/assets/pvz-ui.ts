import almanacPlantCard from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/Almanac/Almanac_PlantCard.PNG?url";
import lawnBackground from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/backgrounds/background1.PNG?url";
import moneyBag from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/Map_Elements/moneybag_hi_res.PNG?url";
import sodThreeRows from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/Sod/sod3row.PNG?url";
import buttonMiddle from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/UI_Texture/Button/button_middle.PNG?url";
import dialogCenter from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/UI_Texture/Dialog/dialog_centermiddle.PNG?url";
import dialogHeader from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/UI_Texture/Dialog/dialog_header.PNG?url";
import checkboxOff from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/UI_Texture/Options/options_checkbox0.PNG?url";
import checkboxOn from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/UI_Texture/Options/options_checkbox1.PNG?url";
import optionsPanel from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/UI_Texture/Options/options_menuback.PNG?url";
import seedBank from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/UI_Texture/SeedBank.PNG?url";
import woodSign from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/images/UI_Texture/Selector_Screen/SelectorScreen_WoodSign1.PNG?url";
import diamond from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Diamond/Diamond.PNG?url";
import coinGold from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Anim_Item/Coin/coin_gold_dollar.PNG?url";
import coinSilver from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Anim_Item/Coin/coin_silver_dollar.PNG?url";
import magnetshroom from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Plants/Magnetshroom/Magnetshroom_head1.PNG?url";
import marigold from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Plants/Marigold/Marigold_head.PNG?url";
import peashooter from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Plants/PeaShooter/PeaShooter_Head.PNG?url";
import sunflowerFace from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Plants/SunFlower/SunFlower_head.PNG?url";
import sunflowerPetals from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Plants/SunFlower/SunFlower_double_petals.PNG?url";
import sun from "../../resources/PVZ-Resources-Sorted-master/PVZ-Resources-Sorted-master/Sprites/reanim/Sun/Sun1.PNG?url";

export const pvzUiAssets = {
  chrome: {
    almanacPlantCard,
    buttonMiddle,
    checkboxOff,
    checkboxOn,
    dialogCenter,
    dialogHeader,
    optionsPanel,
    seedBank,
    woodSign,
  },
  plants: {
    marigold,
    peashooter,
    sunflowerFace,
    sunflowerPetals,
    magnetshroom,
  },
  rewards: {
    moneyBag,
    coinGold,
    coinSilver,
    diamond,
    sun,
  },
  scene: {
    lawnBackground,
    sodThreeRows,
  },
} as const;

export const pvzUiCssVariables = {
  "--pvz-seed-bank": `url("${seedBank}")`,
  "--pvz-plant-card": `url("${almanacPlantCard}")`,
  "--pvz-button": `url("${buttonMiddle}")`,
  "--pvz-dialog-center": `url("${dialogCenter}")`,
  "--pvz-dialog-header": `url("${dialogHeader}")`,
  "--pvz-options-panel": `url("${optionsPanel}")`,
  "--pvz-wood-sign": `url("${woodSign}")`,
  "--pvz-lawn-background": `url("${lawnBackground}")`,
  "--pvz-sod-three-rows": `url("${sodThreeRows}")`,
  "--pvz-checkbox-off": `url("${checkboxOff}")`,
  "--pvz-checkbox-on": `url("${checkboxOn}")`,
} as const;
