import type {
  AppSnapshotDto,
  OfflineRewardBagDto,
  WalletDisplayMode,
} from "../../shared/types";
import { GameScenePlaceholder } from "../../components/GameScenePlaceholder";
import { PlantStatusBar } from "../../components/PlantStatusBar";
import { OfflineRewardBag } from "../offline-bag/OfflineRewardBag";
import { WalletPanel } from "../wallet/WalletPanel";
import { WorkStatus } from "./WorkStatus";

interface DashboardProps {
  snapshot: AppSnapshotDto;
  bags: OfflineRewardBagDto[];
  walletMode: WalletDisplayMode;
  claimingBagId: string | null;
  onWalletModeChange: (mode: WalletDisplayMode) => void;
  onClaimBag: (bagId: string) => void;
}

export function Dashboard({
  snapshot,
  bags,
  walletMode,
  claimingBagId,
  onWalletModeChange,
  onClaimBag,
}: DashboardProps) {
  return (
    <div className="dashboard">
      <PlantStatusBar />
      <WorkStatus snapshot={snapshot} />
      <div className="dashboard-grid">
        <GameScenePlaceholder />
        <WalletPanel
          snapshot={snapshot}
          mode={walletMode}
          onModeChange={onWalletModeChange}
        />
      </div>
      {bags.length > 0 ? (
        <section className="offline-bag-list" aria-label="Offline reward bags">
          {bags.map((bag) => (
            <OfflineRewardBag
              key={bag.bag_id}
              bag={bag}
              claiming={claimingBagId === bag.bag_id}
              onClaim={onClaimBag}
            />
          ))}
        </section>
      ) : null}
    </div>
  );
}
