import { pvzUiAssets } from "../assets/pvz-ui";

const plants = [
  {
    name: "Marigold",
    detail: "Salary rewards",
    deployed: true,
    image: pvzUiAssets.plants.marigold,
  },
  {
    name: "Peashooter",
    detail: "Available in a later phase",
    deployed: false,
    image: pvzUiAssets.plants.peashooter,
  },
  {
    name: "Sunflower",
    detail: "Available in a later phase",
    deployed: false,
    image: pvzUiAssets.plants.sunflowerFace,
    backdrop: pvzUiAssets.plants.sunflowerPetals,
  },
  {
    name: "Magnet-shroom",
    detail: "Auto collect",
    deployed: true,
    image: pvzUiAssets.plants.magnetshroom,
  },
] as const;

export function PlantStatusBar() {
  return (
    <section className="plant-status-bar" aria-label="Plant status">
      {plants.map((plant) => (
        <article
          className={`plant-slot${plant.deployed ? " is-deployed" : " is-locked"}`}
          key={plant.name}
        >
          <span className="plant-art" aria-hidden="true">
            {"backdrop" in plant ? (
              <img className="plant-art-backdrop" src={plant.backdrop} alt="" />
            ) : null}
            <img className="plant-art-main" src={plant.image} alt="" />
          </span>
          <span className="plant-card-copy">
            <strong>{plant.name}</strong>
            <small>{plant.deployed ? "已部署" : "暂不可种植"}</small>
          </span>
          <span className="plant-card-status" aria-hidden="true">
            {plant.deployed ? "DEPLOYED" : "LOCKED"}
          </span>
          <span className="visually-hidden">{plant.detail}</span>
        </article>
      ))}
    </section>
  );
}
