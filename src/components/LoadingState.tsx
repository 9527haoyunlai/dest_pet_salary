import { pvzUiAssets } from "../assets/pvz-ui";

export function LoadingState() {
  return (
    <section className="state-panel" aria-live="polite">
      <div className="loading-mark" aria-hidden="true">
        <img src={pvzUiAssets.rewards.sun} alt="" />
      </div>
      <p>Loading your salary garden…</p>
    </section>
  );
}
