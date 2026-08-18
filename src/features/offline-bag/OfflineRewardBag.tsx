import { formatExactCurrency } from "../../app/format";
import { pvzUiAssets } from "../../assets/pvz-ui";
import type { OfflineRewardBagDto } from "../../shared/types";

interface OfflineRewardBagProps {
  bag: OfflineRewardBagDto;
  claiming: boolean;
  onClaim: (bagId: string) => void;
}

export function OfflineRewardBag({
  bag,
  claiming,
  onClaim,
}: OfflineRewardBagProps) {
  return (
    <article className="offline-bag">
      <img className="bag-mark" src={pvzUiAssets.rewards.moneyBag} alt="" aria-hidden="true" />
      <div className="bag-details">
        <p className="eyebrow">Offline reward bag</p>
        <strong title={bag.exact_value}>{formatExactCurrency(bag.exact_value)}</strong>
        <span className="bag-counts">
          <span><img src={pvzUiAssets.rewards.coinSilver} alt="" />Silver {bag.counts.silver}</span>
          <span><img src={pvzUiAssets.rewards.coinGold} alt="" />Gold {bag.counts.gold}</span>
          <span><img src={pvzUiAssets.rewards.diamond} alt="" />Diamond {bag.counts.diamond}</span>
        </span>
      </div>
      <button
        type="button"
        className="primary-button"
        disabled={claiming}
        onClick={() => onClaim(bag.bag_id)}
      >
        {claiming ? "Claiming…" : "Claim bag"}
      </button>
    </article>
  );
}
