import { formatExactCurrency } from "../../app/format";
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
      <div className="bag-mark" aria-hidden="true">
        +
      </div>
      <div className="bag-details">
        <p className="eyebrow">Offline reward bag</p>
        <strong title={bag.exact_value}>{formatExactCurrency(bag.exact_value)}</strong>
        <span>
          Silver {bag.counts.silver} · Gold {bag.counts.gold} · Diamond {bag.counts.diamond}
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
