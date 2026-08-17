import { useCallback, useEffect, useState } from "react";

import {
  claimOfflineRewardBag,
  getAppSnapshot,
  listOfflineRewardBags,
} from "./shared/tauri-api";
import type { AppSnapshotDto, OfflineRewardBagDto } from "./shared/types";

export default function App() {
  const [snapshot, setSnapshot] = useState<AppSnapshotDto | null>(null);
  const [bags, setBags] = useState<OfflineRewardBagDto[]>([]);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  const refresh = useCallback(async () => {
    setBusy(true);
    setError(null);
    try {
      const [nextSnapshot, nextBags] = await Promise.all([
        getAppSnapshot(),
        listOfflineRewardBags(),
      ]);
      setSnapshot(nextSnapshot);
      setBags(nextBags);
    } catch (reason) {
      setError(String(reason));
    } finally {
      setBusy(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
  }, [refresh]);

  const claimFirstBag = async () => {
    const bag = bags[0];
    if (!bag) return;

    setBusy(true);
    setError(null);
    try {
      await claimOfflineRewardBag(bag.bag_id);
      await refresh();
    } catch (reason) {
      setError(String(reason));
      setBusy(false);
    }
  };

  return (
    <main className="app-shell">
      <section className="status-card" aria-labelledby="app-title">
        <p className="phase-label">Phase 3.5 · Tauri API / React Bridge</p>
        <h1 id="app-title">Salary Garden Debug</h1>

        {snapshot ? (
          <dl className="debug-grid">
            <dt>Local Time</dt>
            <dd>{snapshot.current_local_time}</dd>
            <dt>Work Status</dt>
            <dd>{snapshot.work_status}</dd>
            <dt>Payroll Cycle</dt>
            <dd>{snapshot.payroll_cycle.cycle_id}</dd>
            <dt>Today Real Earned</dt>
            <dd>¥{snapshot.real_payroll.today_real_earned_exact}</dd>
            <dt>Cycle Real Earned</dt>
            <dd>¥{snapshot.real_payroll.cycle_real_earned_exact}</dd>
            <dt>Silver</dt>
            <dd>
              {snapshot.reward_entitlement.today.silver} × ¥
              {snapshot.reward_entitlement.values.silver_exact}
            </dd>
            <dt>Gold</dt>
            <dd>
              {snapshot.reward_entitlement.today.gold} × ¥
              {snapshot.reward_entitlement.values.gold_exact}
            </dd>
            <dt>Diamond</dt>
            <dd>
              {snapshot.reward_entitlement.today.diamond} × ¥
              {snapshot.reward_entitlement.values.diamond_exact}
            </dd>
            <dt>Collected Today</dt>
            <dd>¥{snapshot.collected_wallet.today_collected_exact}</dd>
            <dt>Collected Cycle</dt>
            <dd>¥{snapshot.collected_wallet.cycle_collected_exact}</dd>
            <dt>Offline Bags</dt>
            <dd>
              {snapshot.offline.unclaimed_bag_count} / ¥
              {snapshot.offline.unclaimed_exact_total}
            </dd>
          </dl>
        ) : (
          <p className="loading">Waiting for Rust snapshot…</p>
        )}

        {error ? <p className="error-message">{error}</p> : null}

        <div className="debug-actions">
          <button type="button" onClick={() => void refresh()} disabled={busy}>
            Refresh
          </button>
          {bags.length > 0 ? (
            <button type="button" onClick={() => void claimFirstBag()} disabled={busy}>
              Claim Bag
            </button>
          ) : null}
        </div>
      </section>
    </main>
  );
}
