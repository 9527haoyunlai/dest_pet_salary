export function GameScenePlaceholder() {
  return (
    <section className="game-scene" aria-labelledby="scene-title">
      <div className="scene-vignette" aria-hidden="true" />
      <div className="scene-copy">
        <span className="scene-phase-tag">PHASE 5 · PIXIJS MOUNT</span>
        <p className="eyebrow">Salary Garden Lawn</p>
        <h2 id="scene-title">Game Scene Placeholder</h2>
        <p>草坪已经就位，后续 PixiJS 场景将直接挂载到这里。</p>
      </div>
      <div className="scene-mount-outline" aria-hidden="true">PIXIGAME SCENE</div>
    </section>
  );
}
