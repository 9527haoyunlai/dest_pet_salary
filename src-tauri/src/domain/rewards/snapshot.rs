use rust_decimal::Decimal;

use crate::domain::payroll::{PayrollCycle, PayrollSnapshot};

use super::{RewardCounts, RewardValues};

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RewardSnapshot {
    pub effective_work_seconds: u64,
    pub counts: RewardCounts,
    pub values: RewardValues,
    pub total_value: Decimal,
}

pub fn calculate_reward_snapshot(
    cycle: &PayrollCycle,
    effective_work_seconds: u64,
) -> RewardSnapshot {
    let counts = RewardCounts::from_work_seconds(effective_work_seconds);
    let values = RewardValues::for_cycle(cycle);
    let total_value = values.total_value(counts);

    RewardSnapshot {
        effective_work_seconds,
        counts,
        values,
        total_value,
    }
}

pub fn reward_snapshot_from_payroll(
    cycle: &PayrollCycle,
    payroll: &PayrollSnapshot,
) -> RewardSnapshot {
    calculate_reward_snapshot(cycle, u64::from(payroll.effective_work_seconds_today))
}
