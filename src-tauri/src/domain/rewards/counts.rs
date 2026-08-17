use rust_decimal::Decimal;

const SILVER_INTERVAL_SECONDS: u64 = 10;
const GOLD_INTERVAL_SECONDS: u64 = 60;
const DIAMOND_INTERVAL_SECONDS: u64 = 3_600;

pub(crate) const SILVER_WEIGHT: u64 = 1;
pub(crate) const GOLD_WEIGHT: u64 = 6;
pub(crate) const DIAMOND_WEIGHT: u64 = 360;

pub(crate) const SILVER_PER_HOUR: u64 = 300;
pub(crate) const GOLD_PER_HOUR: u64 = 59;

/// Deterministic reward entitlement derived only from accumulated effective work seconds.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct RewardCounts {
    pub silver: u64,
    pub gold: u64,
    pub diamond: u64,
}

impl RewardCounts {
    pub fn from_work_seconds(work_seconds: u64) -> Self {
        let diamond = work_seconds / DIAMOND_INTERVAL_SECONDS;
        let gold = work_seconds / GOLD_INTERVAL_SECONDS - diamond;
        let silver = work_seconds / SILVER_INTERVAL_SECONDS - work_seconds / GOLD_INTERVAL_SECONDS;

        Self {
            silver,
            gold,
            diamond,
        }
    }

    pub fn total_events(self) -> u64 {
        self.silver + self.gold + self.diamond
    }

    pub fn weighted_units(self) -> Decimal {
        Decimal::from(self.silver) * Decimal::from(SILVER_WEIGHT)
            + Decimal::from(self.gold) * Decimal::from(GOLD_WEIGHT)
            + Decimal::from(self.diamond) * Decimal::from(DIAMOND_WEIGHT)
    }

    pub(crate) fn complete_hours(self) -> Option<u64> {
        let hours = self.diamond;
        let expected_silver = SILVER_PER_HOUR.checked_mul(hours)?;
        let expected_gold = GOLD_PER_HOUR.checked_mul(hours)?;
        (hours > 0 && self.silver == expected_silver && self.gold == expected_gold).then_some(hours)
    }
}
