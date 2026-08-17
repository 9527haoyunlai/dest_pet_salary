const plants = [
  { name: "Marigold", detail: "Salary rewards", deployed: true },
  { name: "Peashooter", detail: "Available in a later phase", deployed: false },
  { name: "Sunflower", detail: "Available in a later phase", deployed: false },
  { name: "Magnet-shroom", detail: "Auto collect", deployed: true },
] as const;

export function PlantStatusBar() {
  return (
    <section className="plant-status-bar" aria-label="Plant status">
      {plants.map((plant) => (
        <article
          className={`plant-slot${plant.deployed ? " is-deployed" : " is-locked"}`}
          key={plant.name}
        >
          <span className="plant-placeholder" aria-hidden="true">
            {plant.name.slice(0, 1)}
          </span>
          <span>
            <strong>{plant.name}</strong>
            <small>{plant.deployed ? "Deployed" : "Not planted"}</small>
          </span>
          <span className="visually-hidden">{plant.detail}</span>
        </article>
      ))}
    </section>
  );
}
