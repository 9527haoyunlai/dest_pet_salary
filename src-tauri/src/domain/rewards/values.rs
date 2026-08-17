use rust_decimal::Decimal;

use crate::domain::payroll::PayrollCycle;

use super::counts::{DIAMOND_WEIGHT, GOLD_WEIGHT};
use super::RewardCounts;

const WEIGHTED_UNITS_PER_HOUR: u64 = 1_014;
const WORK_HOURS_PER_DAY: u64 = 7;

/// Exact per-cycle denomination values plus the payroll anchors needed for reconciliation.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RewardValues {
    pub silver: Decimal,
    pub gold: Decimal,
    pub diamond: Decimal,
    hourly_pay: Decimal,
    daily_pay: Decimal,
    monthly_salary: Decimal,
    workday_count: u32,
}

impl RewardValues {
    pub fn for_cycle(cycle: &PayrollCycle) -> Self {
        let rates = cycle.pay_rates();
        let base = rates.hourly / Decimal::from(WEIGHTED_UNITS_PER_HOUR);

        Self {
            silver: base,
            gold: base * Decimal::from(GOLD_WEIGHT),
            diamond: base * Decimal::from(DIAMOND_WEIGHT),
            hourly_pay: rates.hourly,
            daily_pay: rates.daily,
            monthly_salary: cycle.monthly_salary,
            workday_count: cycle.workday_count,
        }
    }

    pub fn total_value(self, counts: RewardCounts) -> Decimal {
        if counts == RewardCounts::default() {
            return Decimal::ZERO;
        }

        if let Some(hours) = counts.complete_hours() {
            let cycle_hours = u64::from(self.workday_count) * WORK_HOURS_PER_DAY;
            if hours == cycle_hours {
                return self.monthly_salary;
            }
            if hours % WORK_HOURS_PER_DAY == 0 {
                return self.daily_pay * Decimal::from(hours / WORK_HOURS_PER_DAY);
            }
            return self.hourly_pay * Decimal::from(hours);
        }

        // Decimal cannot represent division by 1014 exactly for every salary. Compute
        // totals from the unrounded hourly anchor and weighted units, rather than by
        // summing UI-formatted denomination values. Whole-hour/day/cycle totals above
        // use their payroll anchors so the frozen SSOT invariants remain exact.
        self.hourly_pay * counts.weighted_units() / Decimal::from(WEIGHTED_UNITS_PER_HOUR)
    }
}
