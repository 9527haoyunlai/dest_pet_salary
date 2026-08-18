use chrono::{DateTime, Days, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use rust_decimal::Decimal;

use crate::domain::payroll::{
    calculate_payroll_snapshot, effective_work_seconds, PayrollCycle, WorkCalendar,
    WORK_SECONDS_PER_DAY,
};
use crate::domain::rewards::{
    reward_snapshot_from_payroll, RewardCounts, RewardType, RewardValues,
};
use crate::persistence::{
    CollectionLedgerEntry, LiveRewardEvent, LiveRewardStatus, OfflineRewardBag, SqliteRepository,
};
use crate::services::{DailyEntitlement, RewardLedgerService};

use super::{
    AppSettingsDto, AppSnapshotDto, ApplicationError, CollectedWalletDto, CollectionLedgerEntryDto,
    LiveRewardEventDto, LiveRewardStatusDto, LiveRewardTypeDto, OfflineRewardBagDto,
    OfflineSummaryDto, PayrollCycleDto, RealPayrollDto, RewardCountsDto, RewardEntitlementDto,
    RewardValuesDto, WalletDisplayMode,
};

const SETTING_WALLET_DISPLAY_MODE: &str = "wallet_display_mode";
const SETTING_SOUND_ENABLED: &str = "sound_enabled";
const SETTING_AUTO_COLLECT_ENABLED: &str = "auto_collect_enabled";

#[derive(Clone, Debug)]
pub struct ApplicationContext {
    pub cycle_id: String,
    pub cycle: PayrollCycle,
    pub calendar: WorkCalendar,
}

impl ApplicationContext {
    pub fn new(cycle_id: impl Into<String>, cycle: PayrollCycle, calendar: WorkCalendar) -> Self {
        Self {
            cycle_id: cycle_id.into(),
            cycle,
            calendar,
        }
    }
}

pub struct ApplicationService<'repository> {
    repository: &'repository mut SqliteRepository,
    context: &'repository ApplicationContext,
}

impl<'repository> ApplicationService<'repository> {
    pub fn new(
        repository: &'repository mut SqliteRepository,
        context: &'repository ApplicationContext,
    ) -> Self {
        Self {
            repository,
            context,
        }
    }

    pub fn reconcile_offline(
        &mut self,
        at_utc: DateTime<Utc>,
    ) -> Result<Option<OfflineRewardBagDto>, ApplicationError> {
        let entitlements = self.daily_entitlements(at_utc)?;
        let cycle_start =
            local_work_start_utc(self.context.cycle.timezone, self.context.cycle.start_date)?;
        let period_start = cycle_start.min(at_utc);
        let bag = RewardLedgerService::new(self.repository).reconcile_offline(
            &self.context.cycle_id,
            &self.context.cycle,
            period_start,
            at_utc,
            &entitlements,
            at_utc,
        )?;
        Ok(bag.map(OfflineRewardBagDto::from))
    }

    pub fn get_app_snapshot(
        &mut self,
        at_utc: DateTime<Utc>,
    ) -> Result<AppSnapshotDto, ApplicationError> {
        self.repository.ensure_payroll_cycle(
            &self.context.cycle_id,
            &self.context.cycle,
            at_utc,
        )?;
        let payroll =
            calculate_payroll_snapshot(&self.context.cycle, &self.context.calendar, at_utc)?;
        let rewards = reward_snapshot_from_payroll(&self.context.cycle, &payroll);
        let reward_values = RewardValues::for_cycle(&self.context.cycle);
        let local_date = payroll.local_datetime.date_naive();
        let today_collected = self
            .repository
            .daily_reward_state(&self.context.cycle_id, local_date)?
            .map(|state| reward_values.total_value(state.collected))
            .unwrap_or(Decimal::ZERO);
        let cycle_collected = self
            .repository
            .cycle_wallet_totals(&self.context.cycle_id)?;
        let unclaimed: Vec<_> = self
            .repository
            .offline_reward_bags(None)?
            .into_iter()
            .filter(|bag| !bag.claimed)
            .collect();
        let unclaimed_total = unclaimed
            .iter()
            .fold(Decimal::ZERO, |total, bag| total + bag.exact_value);
        let rates = self.context.cycle.pay_rates();

        Ok(AppSnapshotDto {
            current_local_time: payroll.local_datetime.to_rfc3339(),
            work_status: payroll.work_status.as_code().to_owned(),
            effective_work_seconds_today: payroll.effective_work_seconds_today,
            payroll_cycle: PayrollCycleDto {
                cycle_id: self.context.cycle_id.clone(),
                start_date: self.context.cycle.start_date.to_string(),
                end_date: self.context.cycle.end_date.to_string(),
                workday_count: self.context.cycle.workday_count,
                monthly_salary_exact: self.context.cycle.monthly_salary.to_string(),
                daily_salary_exact: rates.daily.to_string(),
                hourly_salary_exact: rates.hourly.to_string(),
            },
            real_payroll: RealPayrollDto {
                today_real_earned_exact: payroll.today_earned.to_string(),
                cycle_real_earned_exact: payroll.cycle_earned.to_string(),
            },
            reward_entitlement: RewardEntitlementDto {
                today: rewards.counts.into(),
                values: RewardValuesDto {
                    silver_exact: rewards.values.silver.to_string(),
                    gold_exact: rewards.values.gold.to_string(),
                    diamond_exact: rewards.values.diamond.to_string(),
                },
            },
            collected_wallet: CollectedWalletDto {
                today_collected_exact: today_collected.to_string(),
                cycle_collected_exact: cycle_collected.exact_value.to_string(),
            },
            offline: OfflineSummaryDto {
                unclaimed_bag_count: u64::try_from(unclaimed.len()).unwrap_or(u64::MAX),
                unclaimed_exact_total: unclaimed_total.to_string(),
            },
        })
    }

    pub fn list_offline_reward_bags(&self) -> Result<Vec<OfflineRewardBagDto>, ApplicationError> {
        Ok(self
            .repository
            .offline_reward_bags(None)?
            .into_iter()
            .filter(|bag| !bag.claimed)
            .map(OfflineRewardBagDto::from)
            .collect())
    }

    pub fn claim_offline_reward_bag(
        &mut self,
        bag_id: &str,
        at_utc: DateTime<Utc>,
    ) -> Result<CollectionLedgerEntryDto, ApplicationError> {
        let entry = RewardLedgerService::new(self.repository).claim_offline_bag(bag_id, at_utc)?;
        Ok(entry.into())
    }

    pub fn sync_live_rewards(
        &mut self,
        at_utc: DateTime<Utc>,
    ) -> Result<Vec<LiveRewardEventDto>, ApplicationError> {
        let payroll =
            calculate_payroll_snapshot(&self.context.cycle, &self.context.calendar, at_utc)?;
        let events = RewardLedgerService::new(self.repository).materialize_live_rewards(
            &self.context.cycle_id,
            &self.context.cycle,
            payroll.local_datetime.date_naive(),
            u64::from(payroll.effective_work_seconds_today),
            at_utc,
        )?;
        Ok(events.into_iter().map(Into::into).collect())
    }

    pub fn list_pending_live_rewards(&self) -> Result<Vec<LiveRewardEventDto>, ApplicationError> {
        Ok(self
            .repository
            .pending_live_rewards(&self.context.cycle_id)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn collect_live_reward(
        &mut self,
        event_id: &str,
        at_utc: DateTime<Utc>,
    ) -> Result<CollectionLedgerEntryDto, ApplicationError> {
        Ok(RewardLedgerService::new(self.repository)
            .collect_live_reward(event_id, at_utc)?
            .into())
    }

    pub fn package_pending_live_rewards(
        &mut self,
        at_utc: DateTime<Utc>,
    ) -> Result<Vec<OfflineRewardBagDto>, ApplicationError> {
        Ok(RewardLedgerService::new(self.repository)
            .package_pending_live_rewards(at_utc)?
            .into_iter()
            .map(Into::into)
            .collect())
    }

    pub fn get_app_settings(&self) -> Result<AppSettingsDto, ApplicationError> {
        let defaults = AppSettingsDto::default();
        Ok(AppSettingsDto {
            wallet_display_mode: match self
                .repository
                .setting(SETTING_WALLET_DISPLAY_MODE)?
                .as_deref()
            {
                Some("COLLECTED_WALLET") => WalletDisplayMode::CollectedWallet,
                _ => defaults.wallet_display_mode,
            },
            sound_enabled: read_bool_setting(
                self.repository,
                SETTING_SOUND_ENABLED,
                defaults.sound_enabled,
            )?,
            auto_collect_enabled: read_bool_setting(
                self.repository,
                SETTING_AUTO_COLLECT_ENABLED,
                defaults.auto_collect_enabled,
            )?,
        })
    }

    pub fn update_app_settings(
        &mut self,
        settings: AppSettingsDto,
        at_utc: DateTime<Utc>,
    ) -> Result<AppSettingsDto, ApplicationError> {
        let display_mode = match settings.wallet_display_mode {
            WalletDisplayMode::RealSalary => "REAL_SALARY",
            WalletDisplayMode::CollectedWallet => "COLLECTED_WALLET",
        };
        self.repository.set_settings(
            &[
                (
                    SETTING_WALLET_DISPLAY_MODE.to_owned(),
                    display_mode.to_owned(),
                ),
                (
                    SETTING_SOUND_ENABLED.to_owned(),
                    settings.sound_enabled.to_string(),
                ),
                (
                    SETTING_AUTO_COLLECT_ENABLED.to_owned(),
                    settings.auto_collect_enabled.to_string(),
                ),
            ],
            at_utc,
        )?;
        self.get_app_settings()
    }

    fn daily_entitlements(
        &self,
        at_utc: DateTime<Utc>,
    ) -> Result<Vec<DailyEntitlement>, ApplicationError> {
        let local_datetime = at_utc.with_timezone(&self.context.cycle.timezone);
        let local_date = local_datetime.date_naive();
        if local_date < self.context.cycle.start_date {
            return Ok(Vec::new());
        }
        let last_date = local_date.min(self.context.cycle.end_date);
        let mut date = self.context.cycle.start_date;
        let mut entitlements = Vec::new();

        loop {
            if self.context.calendar.is_workday(date) {
                let effective_work_seconds = if date < local_date {
                    u64::from(WORK_SECONDS_PER_DAY)
                } else {
                    u64::from(effective_work_seconds(
                        date,
                        local_datetime.time(),
                        &self.context.calendar,
                    ))
                };
                entitlements.push(DailyEntitlement {
                    work_date: date,
                    effective_work_seconds,
                });
            }
            if date == last_date {
                break;
            }
            date = date
                .checked_add_days(Days::new(1))
                .ok_or(ApplicationError::InvalidLocalDateTime)?;
        }
        Ok(entitlements)
    }
}

fn local_work_start_utc(
    timezone: Tz,
    date: chrono::NaiveDate,
) -> Result<DateTime<Utc>, ApplicationError> {
    let local = date
        .and_time(NaiveTime::from_hms_opt(8, 40, 0).ok_or(ApplicationError::InvalidLocalDateTime)?);
    timezone
        .from_local_datetime(&local)
        .single()
        .map(|value| value.with_timezone(&Utc))
        .ok_or(ApplicationError::InvalidLocalDateTime)
}

fn read_bool_setting(
    repository: &SqliteRepository,
    key: &str,
    default: bool,
) -> Result<bool, ApplicationError> {
    Ok(match repository.setting(key)?.as_deref() {
        Some("true") => true,
        Some("false") => false,
        _ => default,
    })
}

impl From<RewardCounts> for RewardCountsDto {
    fn from(value: RewardCounts) -> Self {
        Self {
            silver: value.silver,
            gold: value.gold,
            diamond: value.diamond,
        }
    }
}

impl From<OfflineRewardBag> for OfflineRewardBagDto {
    fn from(value: OfflineRewardBag) -> Self {
        Self {
            bag_id: value.bag_id,
            cycle_id: value.cycle_id,
            period_start: value.period_start.to_rfc3339(),
            period_end: value.period_end.to_rfc3339(),
            counts: value.counts.into(),
            exact_value: value.exact_value.to_string(),
            created_at: value.created_at.to_rfc3339(),
        }
    }
}

impl From<CollectionLedgerEntry> for CollectionLedgerEntryDto {
    fn from(value: CollectionLedgerEntry) -> Self {
        Self {
            transaction_id: value.transaction_id,
            cycle_id: value.cycle_id,
            source_type: value.source_type,
            source_id: value.source_id,
            counts: value.counts.into(),
            exact_value: value.exact_value.to_string(),
            created_at: value.created_at.to_rfc3339(),
        }
    }
}

impl From<LiveRewardEvent> for LiveRewardEventDto {
    fn from(value: LiveRewardEvent) -> Self {
        Self {
            event_id: value.event_id,
            cycle_id: value.cycle_id,
            work_date: value.work_date.to_string(),
            effective_second_boundary: value.effective_second_boundary,
            event_index: value.event_index,
            reward_type: match value.reward_type {
                RewardType::Silver => LiveRewardTypeDto::Silver,
                RewardType::Gold => LiveRewardTypeDto::Gold,
                RewardType::Diamond => LiveRewardTypeDto::Diamond,
            },
            status: match value.status {
                LiveRewardStatus::Pending => LiveRewardStatusDto::Pending,
                LiveRewardStatus::Collected => LiveRewardStatusDto::Collected,
                LiveRewardStatus::Packaged => LiveRewardStatusDto::Packaged,
            },
            exact_value: value.exact_value.to_string(),
            created_at: value.created_at.to_rfc3339(),
        }
    }
}
