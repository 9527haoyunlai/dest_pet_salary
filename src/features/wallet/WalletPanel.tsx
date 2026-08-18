import { formatExactCurrency } from "../../app/format";
import { pvzUiAssets } from "../../assets/pvz-ui";
import type { AppSnapshotDto, WalletDisplayMode } from "../../shared/types";

interface WalletPanelProps {
  snapshot: AppSnapshotDto;
  mode: WalletDisplayMode;
  onModeChange: (mode: WalletDisplayMode) => void;
}

export function WalletPanel({ snapshot, mode, onModeChange }: WalletPanelProps) {
  const isRealSalary = mode === "REAL_SALARY";
  const todayExact = isRealSalary
    ? snapshot.real_payroll.today_real_earned_exact
    : snapshot.collected_wallet.today_collected_exact;
  const cycleExact = isRealSalary
    ? snapshot.real_payroll.cycle_real_earned_exact
    : snapshot.collected_wallet.cycle_collected_exact;

  return (
    <section className="wallet-panel" aria-labelledby="wallet-title">
      <div className="section-heading">
        <div>
          <p className="eyebrow">Salary Wallet</p>
          <h2 id="wallet-title">
            {isRealSalary ? "Real-time salary" : "Collected wallet"}
          </h2>
        </div>
        <div className="segmented-control" aria-label="Wallet display mode">
          <button
            type="button"
            aria-pressed={isRealSalary}
            onClick={() => onModeChange("REAL_SALARY")}
          >
            Real salary
          </button>
          <button
            type="button"
            aria-pressed={!isRealSalary}
            onClick={() => onModeChange("COLLECTED_WALLET")}
          >
            Game wallet
          </button>
        </div>
      </div>

      <div className="wallet-values">
        <article>
          <img src={pvzUiAssets.rewards.coinSilver} alt="" aria-hidden="true" />
          <span>Today</span>
          <strong title={todayExact}>{formatExactCurrency(todayExact)}</strong>
        </article>
        <article>
          <img src={pvzUiAssets.rewards.coinGold} alt="" aria-hidden="true" />
          <span>Current payroll cycle</span>
          <strong title={cycleExact}>{formatExactCurrency(cycleExact)}</strong>
        </article>
      </div>

      {snapshot.offline.unclaimed_bag_count > 0 ? (
        <p className="pending-value">
          <img src={pvzUiAssets.rewards.moneyBag} alt="" aria-hidden="true" />
          Pending rewards · {formatExactCurrency(snapshot.offline.unclaimed_exact_total)}
        </p>
      ) : null}
    </section>
  );
}
