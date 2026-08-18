use std::collections::HashSet;

use chrono::{DateTime, NaiveDate, Utc};

use crate::domain::payroll::{PayrollCycle, WORK_SECONDS_PER_DAY};
use crate::domain::rewards::{RewardCounts, RewardValues};
use crate::persistence::{
    CollectionLedgerEntry, LiveRewardEvent, OfflineRewardBag, PersistenceError, SqliteRepository,
};

pub const MAX_PENDING_LIVE_REWARDS: usize = 12;

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

    pub fn materialize_live_rewards(
        &mut self,
        cycle_id: &str,
        cycle: &PayrollCycle,
        work_date: NaiveDate,
        effective_work_seconds: u64,
        created_at: DateTime<Utc>,
    ) -> Result<Vec<LiveRewardEvent>, PersistenceError> {
        if effective_work_seconds > u64::from(WORK_SECONDS_PER_DAY) {
            return Err(PersistenceError::EffectiveWorkSecondsOutOfRange {
                work_date: work_date.to_string(),
                seconds: effective_work_seconds,
            });
        }
        self.repository
            .ensure_payroll_cycle(cycle_id, cycle, created_at)?;
        self.repository.materialize_live_rewards_transaction(
            cycle_id,
            work_date,
            RewardCounts::from_work_seconds(effective_work_seconds),
            RewardValues::for_cycle(cycle),
            MAX_PENDING_LIVE_REWARDS,
            created_at,
        )
    }

    pub fn collect_live_reward(
        &mut self,
        event_id: &str,
        collected_at: DateTime<Utc>,
    ) -> Result<CollectionLedgerEntry, PersistenceError> {
        self.repository
            .collect_live_reward_transaction(event_id, collected_at)
    }

    pub fn package_pending_live_rewards(
        &mut self,
        packaged_at: DateTime<Utc>,
    ) -> Result<Vec<OfflineRewardBag>, PersistenceError> {
        self.repository
            .package_pending_live_rewards_transaction(packaged_at)
    }
}
