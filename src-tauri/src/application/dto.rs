use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum WalletDisplayMode {
    RealSalary,
    CollectedWallet,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct AppSettingsDto {
    pub wallet_display_mode: WalletDisplayMode,
    pub sound_enabled: bool,
    pub auto_collect_enabled: bool,
}

impl Default for AppSettingsDto {
    fn default() -> Self {
        Self {
            wallet_display_mode: WalletDisplayMode::RealSalary,
            sound_enabled: true,
            auto_collect_enabled: true,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RewardCountsDto {
    pub silver: u64,
    pub gold: u64,
    pub diamond: u64,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RewardValuesDto {
    pub silver_exact: String,
    pub gold_exact: String,
    pub diamond_exact: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct PayrollCycleDto {
    pub cycle_id: String,
    pub start_date: String,
    pub end_date: String,
    pub workday_count: u32,
    pub monthly_salary_exact: String,
    pub daily_salary_exact: String,
    pub hourly_salary_exact: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RealPayrollDto {
    pub today_real_earned_exact: String,
    pub cycle_real_earned_exact: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct RewardEntitlementDto {
    pub today: RewardCountsDto,
    pub values: RewardValuesDto,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CollectedWalletDto {
    pub today_collected_exact: String,
    pub cycle_collected_exact: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OfflineSummaryDto {
    pub unclaimed_bag_count: u64,
    pub unclaimed_exact_total: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct AppSnapshotDto {
    pub current_local_time: String,
    pub work_status: String,
    pub effective_work_seconds_today: u32,
    pub payroll_cycle: PayrollCycleDto,
    pub real_payroll: RealPayrollDto,
    pub reward_entitlement: RewardEntitlementDto,
    pub collected_wallet: CollectedWalletDto,
    pub offline: OfflineSummaryDto,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct OfflineRewardBagDto {
    pub bag_id: String,
    pub cycle_id: String,
    pub period_start: String,
    pub period_end: String,
    pub counts: RewardCountsDto,
    pub exact_value: String,
    pub created_at: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct CollectionLedgerEntryDto {
    pub transaction_id: String,
    pub cycle_id: String,
    pub source_type: String,
    pub source_id: String,
    pub counts: RewardCountsDto,
    pub exact_value: String,
    pub created_at: String,
}
