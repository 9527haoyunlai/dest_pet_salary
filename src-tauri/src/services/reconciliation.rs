use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, Utc};

use crate::domain::payroll::{PayrollCycle, WORK_SECONDS_PER_DAY};
use crate::domain::rewards::{RewardCounts, RewardValues};
use crate::persistence::{
    CollectionLedgerEntry, OfflineRewardBag, PersistenceError, SqliteRepository,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct DailyEntitlement {
    pub work_date: NaiveDate,
    pub effective_work_seconds: u64,
}

pub struct RewardLedgerService<'repository> {
    repository: &'repository mut SqliteRepository,
}

impl<'repository> RewardLedgerService<'repository> {
    pub fn new(repository: &'repository mut SqliteRepository) -> Self {
        Self { repository }
    }

    pub fn reconcile_offline(
        &mut self,
        cycle_id: &str,
        cycle: &PayrollCycle,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        entitlements: &[DailyEntitlement],
        reconciled_at: DateTime<Utc>,
    ) -> Result<Option<OfflineRewardBag>, PersistenceError> {
        if period_start > period_end {
            return Err(PersistenceError::InvalidPeriod);
        }

        let mut dates = HashSet::with_capacity(entitlements.len());
        let mut rows = Vec::with_capacity(entitlements.len());
        for entitlement in entitlements {
            if !dates.insert(entitlement.work_date) {
                return Err(PersistenceError::DuplicateEntitlementDate(
                    entitlement.work_date.to_string(),
                ));
            }
            if entitlement.effective_work_seconds > u64::from(WORK_SECONDS_PER_DAY) {
                return Err(PersistenceError::EffectiveWorkSecondsOutOfRange {
                    work_date: entitlement.work_date.to_string(),
                    seconds: entitlement.effective_work_seconds,
                });
            }
            rows.push((
                entitlement.work_date,
                RewardCounts::from_work_seconds(entitlement.effective_work_seconds),
            ));
        }

        self.repository
            .ensure_payroll_cycle(cycle_id, cycle, reconciled_at)?;
        self.repository.reconcile_offline_transaction(
            cycle_id,
            period_start,
            period_end,
            &rows,
            RewardValues::for_cycle(cycle),
            reconciled_at,
        )
    }

    pub fn claim_offline_bag(
        &mut self,
        bag_id: &str,
        claimed_at: DateTime<Utc>,
    ) -> Result<CollectionLedgerEntry, PersistenceError> {
        self.repository
            .claim_offline_bag_transaction(bag_id, claimed_at)
    }
}
