export function GameScenePlaceholder() {
  return (
    <section className="game-scene" aria-labelledby="scene-title">
      <div className="scene-copy">
        <p className="eyebrow">Garden view</p>
        <h2 id="scene-title">Game Scene Placeholder</h2>
        <p>Reserved for the Phase 5 PixiJS scene.</p>
      </div>
      <div className="lawn-placeholder" aria-hidden="true">
        {Array.from({ length: 21 }, (_, index) => (
          <span key={index} />
        ))}
      </div>
    </section>
  );
}
