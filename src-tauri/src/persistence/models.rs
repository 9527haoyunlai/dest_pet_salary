use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;

use crate::domain::rewards::RewardCounts;

#[derive(Clone, Debug, PartialEq)]
pub struct PayrollCycleRecord {
    pub cycle_id: String,
    pub salary_month: NaiveDate,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub monthly_salary: Decimal,
    pub workday_count: u32,
    pub daily_pay: Decimal,
    pub hourly_pay: Decimal,
    pub per_second_pay: Decimal,
    pub silver_value: Decimal,
    pub gold_value: Decimal,
    pub diamond_value: Decimal,
    pub timezone: String,
    pub calendar_version: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DailyRewardState {
    pub cycle_id: String,
    pub work_date: NaiveDate,
    pub entitled: RewardCounts,
    pub accounted: RewardCounts,
    pub collected: RewardCounts,
    pub updated_at: DateTime<Utc>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OfflineRewardBag {
    pub bag_id: String,
    pub cycle_id: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub counts: RewardCounts,
    pub exact_value: Decimal,
    pub created_at: DateTime<Utc>,
    pub claimed: bool,
    pub claimed_at: Option<DateTime<Utc>>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CollectionLedgerEntry {
    pub transaction_id: String,
    pub cycle_id: String,
    pub source_type: String,
    pub source_id: String,
    pub counts: RewardCounts,
    pub exact_value: Decimal,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WalletTotals {
    pub counts: RewardCounts,
    pub exact_value: Decimal,
}
