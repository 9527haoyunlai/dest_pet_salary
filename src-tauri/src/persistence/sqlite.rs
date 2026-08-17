use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

use chrono::{DateTime, NaiveDate, SecondsFormat, Utc};
use rusqlite::types::Type;
use rusqlite::{params, Connection, OptionalExtension, Row, TransactionBehavior};
use rust_decimal::Decimal;
use uuid::Uuid;

use crate::domain::payroll::PayrollCycle;
use crate::domain::rewards::{RewardCounts, RewardValues};

use super::{
    CollectionLedgerEntry, DailyRewardState, OfflineRewardBag, PayrollCycleRecord,
    PersistenceError, WalletTotals,
};

const MIGRATIONS: &[(i64, &str)] = &[(
    1,
    include_str!("../../migrations/0001_phase3_persistence.sql"),
)];

pub struct SqliteRepository {
    connection: Connection,
}

impl SqliteRepository {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, PersistenceError> {
        let connection = Connection::open(path)?;
        Self::initialize(connection)
    }

    pub fn open_in_memory() -> Result<Self, PersistenceError> {
        let connection = Connection::open_in_memory()?;
        Self::initialize(connection)
    }

    fn initialize(connection: Connection) -> Result<Self, PersistenceError> {
        connection.execute_batch("PRAGMA foreign_keys = ON;")?;
        connection.busy_timeout(Duration::from_secs(5))?;

        let mut repository = Self { connection };
        repository.apply_migrations()?;
        Ok(repository)
    }

    fn apply_migrations(&mut self) -> Result<(), PersistenceError> {
        self.connection.execute_batch(
            "CREATE TABLE IF NOT EXISTS schema_migrations (
                version INTEGER PRIMARY KEY NOT NULL,
                applied_at TEXT NOT NULL
            );",
        )?;

        for (version, sql) in MIGRATIONS {
            let already_applied = self.connection.query_row(
                "SELECT EXISTS(SELECT 1 FROM schema_migrations WHERE version = ?1)",
                params![version],
                |row| row.get::<_, bool>(0),
            )?;
            if already_applied {
                continue;
            }

            let transaction = self
                .connection
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            transaction.execute_batch(sql)?;
            transaction.execute(
                "INSERT INTO schema_migrations(version, applied_at) VALUES (?1, ?2)",
                params![version, timestamp(Utc::now())],
            )?;
            transaction.commit()?;
        }

        Ok(())
    }

    pub fn schema_version(&self) -> Result<i64, PersistenceError> {
        Ok(self.connection.query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
            [],
            |row| row.get(0),
        )?)
    }

    pub fn ensure_payroll_cycle(
        &mut self,
        cycle_id: &str,
        cycle: &PayrollCycle,
        created_at: DateTime<Utc>,
    ) -> Result<PayrollCycleRecord, PersistenceError> {
        if cycle_id.trim().is_empty() {
            return Err(PersistenceError::EmptyCycleId);
        }

        let expected = PayrollCycleRecord::from_cycle(cycle_id, cycle, created_at);
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "INSERT OR IGNORE INTO payroll_cycles (
                cycle_id, salary_month, start_date, end_date, monthly_salary_exact,
                workday_count, daily_pay_exact, hourly_pay_exact, per_second_pay_exact,
                silver_value_exact, gold_value_exact, diamond_value_exact, timezone,
                calendar_version, created_at
            ) VALUES (
                ?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15
            )",
            params![
                expected.cycle_id,
                expected.salary_month.to_string(),
                expected.start_date.to_string(),
                expected.end_date.to_string(),
                expected.monthly_salary.to_string(),
                i64::from(expected.workday_count),
                expected.daily_pay.to_string(),
                expected.hourly_pay.to_string(),
                expected.per_second_pay.to_string(),
                expected.silver_value.to_string(),
                expected.gold_value.to_string(),
                expected.diamond_value.to_string(),
                expected.timezone,
                expected.calendar_version,
                timestamp(expected.created_at),
            ],
        )?;

        let persisted = query_payroll_cycle(&transaction, cycle_id)?.ok_or(
            PersistenceError::InvariantViolation("inserted cycle is missing"),
        )?;
        if !persisted.same_snapshot_as(&expected) {
            return Err(PersistenceError::CycleSnapshotConflict(cycle_id.to_owned()));
        }

        transaction.commit()?;
        Ok(persisted)
    }

    pub fn payroll_cycle(
        &self,
        cycle_id: &str,
    ) -> Result<Option<PayrollCycleRecord>, PersistenceError> {
        query_payroll_cycle(&self.connection, cycle_id)
    }

    /// Removes the zero-value cycle produced by the Phase 3.5 debug bootstrap.
    /// A non-zero payroll snapshot is immutable and is never removed here.
    pub fn remove_unconfigured_zero_cycle(
        &mut self,
        cycle_id: &str,
    ) -> Result<bool, PersistenceError> {
        let Some(cycle) = self.payroll_cycle(cycle_id)? else {
            return Ok(false);
        };
        if !cycle.monthly_salary.is_zero() {
            return Ok(false);
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        transaction.execute(
            "DELETE FROM collection_ledger WHERE cycle_id = ?1",
            params![cycle_id],
        )?;
        transaction.execute(
            "DELETE FROM offline_reward_bag_items
             WHERE bag_id IN (
                SELECT bag_id FROM offline_reward_bags WHERE cycle_id = ?1
             )",
            params![cycle_id],
        )?;
        transaction.execute(
            "DELETE FROM offline_reward_bags WHERE cycle_id = ?1",
            params![cycle_id],
        )?;
        transaction.execute(
            "DELETE FROM daily_reward_state WHERE cycle_id = ?1",
            params![cycle_id],
        )?;
        let deleted = transaction.execute(
            "DELETE FROM payroll_cycles WHERE cycle_id = ?1",
            params![cycle_id],
        )?;
        transaction.commit()?;
        Ok(deleted == 1)
    }

    pub fn set_setting(
        &self,
        key: &str,
        value: &str,
        updated_at: DateTime<Utc>,
    ) -> Result<(), PersistenceError> {
        if key.trim().is_empty() {
            return Err(PersistenceError::EmptySettingKey);
        }
        self.connection.execute(
            "INSERT INTO app_settings(setting_key, setting_value, updated_at)
             VALUES (?1, ?2, ?3)
             ON CONFLICT(setting_key) DO UPDATE SET
                setting_value = excluded.setting_value,
                updated_at = excluded.updated_at",
            params![key, value, timestamp(updated_at)],
        )?;
        Ok(())
    }

    pub fn setting(&self, key: &str) -> Result<Option<String>, PersistenceError> {
        Ok(self
            .connection
            .query_row(
                "SELECT setting_value FROM app_settings WHERE setting_key = ?1",
                params![key],
                |row| row.get(0),
            )
            .optional()?)
    }

    pub fn set_settings(
        &mut self,
        settings: &[(String, String)],
        updated_at: DateTime<Utc>,
    ) -> Result<(), PersistenceError> {
        if settings.iter().any(|(key, _)| key.trim().is_empty()) {
            return Err(PersistenceError::EmptySettingKey);
        }

        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        for (key, value) in settings {
            transaction.execute(
                "INSERT INTO app_settings(setting_key, setting_value, updated_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(setting_key) DO UPDATE SET
                    setting_value = excluded.setting_value,
                    updated_at = excluded.updated_at",
                params![key, value, timestamp(updated_at)],
            )?;
        }
        transaction.commit()?;
        Ok(())
    }

    pub fn daily_reward_state(
        &self,
        cycle_id: &str,
        work_date: NaiveDate,
    ) -> Result<Option<DailyRewardState>, PersistenceError> {
        Ok(self
            .connection
            .query_row(
                "SELECT cycle_id, work_date,
                    entitled_silver, entitled_gold, entitled_diamond,
                    accounted_silver, accounted_gold, accounted_diamond,
                    collected_silver, collected_gold, collected_diamond, updated_at
                 FROM daily_reward_state WHERE cycle_id = ?1 AND work_date = ?2",
                params![cycle_id, work_date.to_string()],
                daily_state_from_row,
            )
            .optional()?)
    }

    pub fn offline_reward_bag(
        &self,
        bag_id: &str,
    ) -> Result<Option<OfflineRewardBag>, PersistenceError> {
        Ok(self
            .connection
            .query_row(
                "SELECT bag_id, cycle_id, period_start, period_end, silver_count,
                    gold_count, diamond_count, exact_value, created_at, claimed, claimed_at
                 FROM offline_reward_bags WHERE bag_id = ?1",
                params![bag_id],
                bag_from_row,
            )
            .optional()?)
    }

    pub fn offline_reward_bags(
        &self,
        cycle_id: Option<&str>,
    ) -> Result<Vec<OfflineRewardBag>, PersistenceError> {
        let mut bags = Vec::new();
        if let Some(cycle_id) = cycle_id {
            let mut statement = self.connection.prepare(
                "SELECT bag_id, cycle_id, period_start, period_end, silver_count,
                    gold_count, diamond_count, exact_value, created_at, claimed, claimed_at
                 FROM offline_reward_bags WHERE cycle_id = ?1 ORDER BY created_at, bag_id",
            )?;
            let rows = statement.query_map(params![cycle_id], bag_from_row)?;
            for row in rows {
                bags.push(row?);
            }
        } else {
            let mut statement = self.connection.prepare(
                "SELECT bag_id, cycle_id, period_start, period_end, silver_count,
                    gold_count, diamond_count, exact_value, created_at, claimed, claimed_at
                 FROM offline_reward_bags ORDER BY created_at, bag_id",
            )?;
            let rows = statement.query_map([], bag_from_row)?;
            for row in rows {
                bags.push(row?);
            }
        }
        Ok(bags)
    }

    pub fn collection_ledger(
        &self,
        cycle_id: Option<&str>,
    ) -> Result<Vec<CollectionLedgerEntry>, PersistenceError> {
        let mut entries = Vec::new();
        if let Some(cycle_id) = cycle_id {
            let mut statement = self.connection.prepare(
                "SELECT transaction_id, cycle_id, source_type, source_id, silver_count,
                    gold_count, diamond_count, exact_value, created_at
                 FROM collection_ledger WHERE cycle_id = ?1 ORDER BY created_at, transaction_id",
            )?;
            let rows = statement.query_map(params![cycle_id], ledger_from_row)?;
            for row in rows {
                entries.push(row?);
            }
        } else {
            let mut statement = self.connection.prepare(
                "SELECT transaction_id, cycle_id, source_type, source_id, silver_count,
                    gold_count, diamond_count, exact_value, created_at
                 FROM collection_ledger ORDER BY created_at, transaction_id",
            )?;
            let rows = statement.query_map([], ledger_from_row)?;
            for row in rows {
                entries.push(row?);
            }
        }
        Ok(entries)
    }

    pub fn wallet_totals(&self) -> Result<WalletTotals, PersistenceError> {
        self.wallet_totals_for_cycle(None)
    }

    pub fn cycle_wallet_totals(&self, cycle_id: &str) -> Result<WalletTotals, PersistenceError> {
        self.wallet_totals_for_cycle(Some(cycle_id))
    }

    fn wallet_totals_for_cycle(
        &self,
        cycle_id: Option<&str>,
    ) -> Result<WalletTotals, PersistenceError> {
        let mut totals = WalletTotals::default();
        for entry in self.collection_ledger(cycle_id)? {
            totals.counts = checked_add_counts(totals.counts, entry.counts)?;
            totals.exact_value += entry.exact_value;
        }
        Ok(totals)
    }

    pub fn payroll_cycle_count(&self) -> Result<u64, PersistenceError> {
        self.table_count("payroll_cycles")
    }

    pub fn offline_reward_bag_count(&self) -> Result<u64, PersistenceError> {
        self.table_count("offline_reward_bags")
    }

    pub fn collection_ledger_count(&self) -> Result<u64, PersistenceError> {
        self.table_count("collection_ledger")
    }

    fn table_count(&self, table: &'static str) -> Result<u64, PersistenceError> {
        let sql = format!("SELECT COUNT(*) FROM {table}");
        let count: i64 = self.connection.query_row(&sql, [], |row| row.get(0))?;
        u64::try_from(count).map_err(|_| PersistenceError::CountOutOfRange)
    }

    pub(crate) fn reconcile_offline_transaction(
        &mut self,
        cycle_id: &str,
        period_start: DateTime<Utc>,
        period_end: DateTime<Utc>,
        entitlements: &[(NaiveDate, RewardCounts)],
        values: RewardValues,
        created_at: DateTime<Utc>,
    ) -> Result<Option<OfflineRewardBag>, PersistenceError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let mut gaps = Vec::new();
        let mut total_gap = RewardCounts::default();

        for (work_date, entitled) in entitlements {
            let entitled_sql = counts_to_sql(*entitled)?;
            transaction.execute(
                "INSERT INTO daily_reward_state (
                    cycle_id, work_date, entitled_silver, entitled_gold, entitled_diamond, updated_at
                 ) VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                 ON CONFLICT(cycle_id, work_date) DO UPDATE SET
                    entitled_silver = MAX(entitled_silver, excluded.entitled_silver),
                    entitled_gold = MAX(entitled_gold, excluded.entitled_gold),
                    entitled_diamond = MAX(entitled_diamond, excluded.entitled_diamond),
                    updated_at = MAX(updated_at, excluded.updated_at)",
                params![
                    cycle_id,
                    work_date.to_string(),
                    entitled_sql.0,
                    entitled_sql.1,
                    entitled_sql.2,
                    timestamp(created_at),
                ],
            )?;

            let accounted = transaction.query_row(
                "SELECT accounted_silver, accounted_gold, accounted_diamond
                 FROM daily_reward_state WHERE cycle_id = ?1 AND work_date = ?2",
                params![cycle_id, work_date.to_string()],
                |row| counts_from_row(row, 0),
            )?;
            let gap = saturating_sub_counts(*entitled, accounted);
            if gap.total_events() > 0 {
                total_gap = checked_add_counts(total_gap, gap)?;
                gaps.push((*work_date, gap));
            }
        }

        if gaps.is_empty() {
            transaction.commit()?;
            return Ok(None);
        }

        let bag = OfflineRewardBag {
            bag_id: Uuid::new_v4().to_string(),
            cycle_id: cycle_id.to_owned(),
            period_start,
            period_end,
            counts: total_gap,
            exact_value: values.total_value(total_gap),
            created_at,
            claimed: false,
            claimed_at: None,
        };
        let bag_counts = counts_to_sql(bag.counts)?;
        transaction.execute(
            "INSERT INTO offline_reward_bags (
                bag_id, cycle_id, period_start, period_end, silver_count, gold_count,
                diamond_count, exact_value, created_at, claimed, claimed_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 0, NULL)",
            params![
                bag.bag_id,
                bag.cycle_id,
                timestamp(bag.period_start),
                timestamp(bag.period_end),
                bag_counts.0,
                bag_counts.1,
                bag_counts.2,
                bag.exact_value.to_string(),
                timestamp(bag.created_at),
            ],
        )?;

        for (work_date, gap) in gaps {
            let gap_sql = counts_to_sql(gap)?;
            transaction.execute(
                "INSERT INTO offline_reward_bag_items (
                    bag_id, work_date, silver_count, gold_count, diamond_count
                 ) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![
                    bag.bag_id,
                    work_date.to_string(),
                    gap_sql.0,
                    gap_sql.1,
                    gap_sql.2,
                ],
            )?;
            let updated = transaction.execute(
                "UPDATE daily_reward_state SET
                    accounted_silver = accounted_silver + ?3,
                    accounted_gold = accounted_gold + ?4,
                    accounted_diamond = accounted_diamond + ?5,
                    updated_at = ?6
                 WHERE cycle_id = ?1 AND work_date = ?2
                    AND accounted_silver + ?3 <= entitled_silver
                    AND accounted_gold + ?4 <= entitled_gold
                    AND accounted_diamond + ?5 <= entitled_diamond",
                params![
                    cycle_id,
                    work_date.to_string(),
                    gap_sql.0,
                    gap_sql.1,
                    gap_sql.2,
                    timestamp(created_at),
                ],
            )?;
            if updated != 1 {
                return Err(PersistenceError::InvariantViolation(
                    "accounted reward count would exceed entitlement",
                ));
            }
        }

        transaction.commit()?;
        Ok(Some(bag))
    }

    pub(crate) fn claim_offline_bag_transaction(
        &mut self,
        bag_id: &str,
        claimed_at: DateTime<Utc>,
    ) -> Result<CollectionLedgerEntry, PersistenceError> {
        let transaction = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let bag = transaction
            .query_row(
                "SELECT bag_id, cycle_id, period_start, period_end, silver_count,
                    gold_count, diamond_count, exact_value, created_at, claimed, claimed_at
                 FROM offline_reward_bags WHERE bag_id = ?1",
                params![bag_id],
                bag_from_row,
            )
            .optional()?
            .ok_or_else(|| PersistenceError::BagNotFound(bag_id.to_owned()))?;
        if bag.claimed {
            return Err(PersistenceError::BagAlreadyClaimed(bag_id.to_owned()));
        }

        let updated = transaction.execute(
            "UPDATE offline_reward_bags SET claimed = 1, claimed_at = ?2
             WHERE bag_id = ?1 AND claimed = 0",
            params![bag_id, timestamp(claimed_at)],
        )?;
        if updated != 1 {
            return Err(PersistenceError::BagAlreadyClaimed(bag_id.to_owned()));
        }

        {
            let mut statement = transaction.prepare(
                "SELECT work_date, silver_count, gold_count, diamond_count
                 FROM offline_reward_bag_items WHERE bag_id = ?1 ORDER BY work_date",
            )?;
            let rows = statement.query_map(params![bag_id], |row| {
                Ok((date_from_column(row, 0)?, counts_from_row(row, 1)?))
            })?;
            for row in rows {
                let (work_date, counts) = row?;
                let counts_sql = counts_to_sql(counts)?;
                let collected = transaction.execute(
                    "UPDATE daily_reward_state SET
                        collected_silver = collected_silver + ?3,
                        collected_gold = collected_gold + ?4,
                        collected_diamond = collected_diamond + ?5,
                        updated_at = ?6
                     WHERE cycle_id = ?1 AND work_date = ?2
                        AND collected_silver + ?3 <= accounted_silver
                        AND collected_gold + ?4 <= accounted_gold
                        AND collected_diamond + ?5 <= accounted_diamond",
                    params![
                        bag.cycle_id,
                        work_date.to_string(),
                        counts_sql.0,
                        counts_sql.1,
                        counts_sql.2,
                        timestamp(claimed_at),
                    ],
                )?;
                if collected != 1 {
                    return Err(PersistenceError::InvariantViolation(
                        "collected reward count would exceed accounted count",
                    ));
                }
            }
        }

        let entry = CollectionLedgerEntry {
            transaction_id: Uuid::new_v4().to_string(),
            cycle_id: bag.cycle_id,
            source_type: "OFFLINE_BAG_CLAIM".to_owned(),
            source_id: bag.bag_id,
            counts: bag.counts,
            exact_value: bag.exact_value,
            created_at: claimed_at,
        };
        let counts_sql = counts_to_sql(entry.counts)?;
        transaction.execute(
            "INSERT INTO collection_ledger (
                transaction_id, cycle_id, source_type, source_id, silver_count, gold_count,
                diamond_count, exact_value, created_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                entry.transaction_id,
                entry.cycle_id,
                entry.source_type,
                entry.source_id,
                counts_sql.0,
                counts_sql.1,
                counts_sql.2,
                entry.exact_value.to_string(),
                timestamp(entry.created_at),
            ],
        )?;

        transaction.commit()?;
        Ok(entry)
    }
}

impl PayrollCycleRecord {
    fn from_cycle(cycle_id: &str, cycle: &PayrollCycle, created_at: DateTime<Utc>) -> Self {
        let pay_rates = cycle.pay_rates();
        let reward_values = RewardValues::for_cycle(cycle);
        Self {
            cycle_id: cycle_id.to_owned(),
            salary_month: cycle.salary_month,
            start_date: cycle.start_date,
            end_date: cycle.end_date,
            monthly_salary: cycle.monthly_salary,
            workday_count: cycle.workday_count,
            daily_pay: pay_rates.daily,
            hourly_pay: pay_rates.hourly,
            per_second_pay: pay_rates.per_second,
            silver_value: reward_values.silver,
            gold_value: reward_values.gold,
            diamond_value: reward_values.diamond,
            timezone: cycle.timezone.to_string(),
            calendar_version: cycle.calendar_version.clone(),
            created_at,
        }
    }

    fn same_snapshot_as(&self, other: &Self) -> bool {
        self.cycle_id == other.cycle_id
            && self.salary_month == other.salary_month
            && self.start_date == other.start_date
            && self.end_date == other.end_date
            && self.monthly_salary == other.monthly_salary
            && self.workday_count == other.workday_count
            && self.daily_pay == other.daily_pay
            && self.hourly_pay == other.hourly_pay
            && self.per_second_pay == other.per_second_pay
            && self.silver_value == other.silver_value
            && self.gold_value == other.gold_value
            && self.diamond_value == other.diamond_value
            && self.timezone == other.timezone
            && self.calendar_version == other.calendar_version
    }
}

fn query_payroll_cycle(
    connection: &Connection,
    cycle_id: &str,
) -> Result<Option<PayrollCycleRecord>, PersistenceError> {
    Ok(connection
        .query_row(
            "SELECT cycle_id, salary_month, start_date, end_date, monthly_salary_exact,
                workday_count, daily_pay_exact, hourly_pay_exact, per_second_pay_exact,
                silver_value_exact, gold_value_exact, diamond_value_exact, timezone,
                calendar_version, created_at
             FROM payroll_cycles WHERE cycle_id = ?1",
            params![cycle_id],
            payroll_cycle_from_row,
        )
        .optional()?)
}

fn payroll_cycle_from_row(row: &Row<'_>) -> rusqlite::Result<PayrollCycleRecord> {
    let workday_count = u32::try_from(row.get::<_, i64>(5)?)
        .map_err(|_| rusqlite::Error::IntegralValueOutOfRange(5, row.get(5).unwrap_or(-1)))?;
    Ok(PayrollCycleRecord {
        cycle_id: row.get(0)?,
        salary_month: date_from_column(row, 1)?,
        start_date: date_from_column(row, 2)?,
        end_date: date_from_column(row, 3)?,
        monthly_salary: decimal_from_column(row, 4)?,
        workday_count,
        daily_pay: decimal_from_column(row, 6)?,
        hourly_pay: decimal_from_column(row, 7)?,
        per_second_pay: decimal_from_column(row, 8)?,
        silver_value: decimal_from_column(row, 9)?,
        gold_value: decimal_from_column(row, 10)?,
        diamond_value: decimal_from_column(row, 11)?,
        timezone: row.get(12)?,
        calendar_version: row.get(13)?,
        created_at: datetime_from_column(row, 14)?,
    })
}

fn daily_state_from_row(row: &Row<'_>) -> rusqlite::Result<DailyRewardState> {
    Ok(DailyRewardState {
        cycle_id: row.get(0)?,
        work_date: date_from_column(row, 1)?,
        entitled: counts_from_row(row, 2)?,
        accounted: counts_from_row(row, 5)?,
        collected: counts_from_row(row, 8)?,
        updated_at: datetime_from_column(row, 11)?,
    })
}

fn bag_from_row(row: &Row<'_>) -> rusqlite::Result<OfflineRewardBag> {
    Ok(OfflineRewardBag {
        bag_id: row.get(0)?,
        cycle_id: row.get(1)?,
        period_start: datetime_from_column(row, 2)?,
        period_end: datetime_from_column(row, 3)?,
        counts: counts_from_row(row, 4)?,
        exact_value: decimal_from_column(row, 7)?,
        created_at: datetime_from_column(row, 8)?,
        claimed: row.get(9)?,
        claimed_at: optional_datetime_from_column(row, 10)?,
    })
}

fn ledger_from_row(row: &Row<'_>) -> rusqlite::Result<CollectionLedgerEntry> {
    Ok(CollectionLedgerEntry {
        transaction_id: row.get(0)?,
        cycle_id: row.get(1)?,
        source_type: row.get(2)?,
        source_id: row.get(3)?,
        counts: counts_from_row(row, 4)?,
        exact_value: decimal_from_column(row, 7)?,
        created_at: datetime_from_column(row, 8)?,
    })
}

fn counts_to_sql(counts: RewardCounts) -> Result<(i64, i64, i64), PersistenceError> {
    Ok((
        i64::try_from(counts.silver).map_err(|_| PersistenceError::CountOutOfRange)?,
        i64::try_from(counts.gold).map_err(|_| PersistenceError::CountOutOfRange)?,
        i64::try_from(counts.diamond).map_err(|_| PersistenceError::CountOutOfRange)?,
    ))
}

fn counts_from_row(row: &Row<'_>, offset: usize) -> rusqlite::Result<RewardCounts> {
    Ok(RewardCounts {
        silver: nonnegative_u64(row, offset)?,
        gold: nonnegative_u64(row, offset + 1)?,
        diamond: nonnegative_u64(row, offset + 2)?,
    })
}

fn nonnegative_u64(row: &Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value = row.get::<_, i64>(index)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}

fn checked_add_counts(
    left: RewardCounts,
    right: RewardCounts,
) -> Result<RewardCounts, PersistenceError> {
    Ok(RewardCounts {
        silver: left
            .silver
            .checked_add(right.silver)
            .ok_or(PersistenceError::CountOutOfRange)?,
        gold: left
            .gold
            .checked_add(right.gold)
            .ok_or(PersistenceError::CountOutOfRange)?,
        diamond: left
            .diamond
            .checked_add(right.diamond)
            .ok_or(PersistenceError::CountOutOfRange)?,
    })
}

fn saturating_sub_counts(left: RewardCounts, right: RewardCounts) -> RewardCounts {
    RewardCounts {
        silver: left.silver.saturating_sub(right.silver),
        gold: left.gold.saturating_sub(right.gold),
        diamond: left.diamond.saturating_sub(right.diamond),
    }
}

fn timestamp(value: DateTime<Utc>) -> String {
    value.to_rfc3339_opts(SecondsFormat::Nanos, true)
}

fn date_from_column(row: &Row<'_>, index: usize) -> rusqlite::Result<NaiveDate> {
    let value: String = row.get(index)?;
    NaiveDate::parse_from_str(&value, "%Y-%m-%d").map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(error))
    })
}

fn datetime_from_column(row: &Row<'_>, index: usize) -> rusqlite::Result<DateTime<Utc>> {
    let value: String = row.get(index)?;
    DateTime::parse_from_rfc3339(&value)
        .map(|value| value.with_timezone(&Utc))
        .map_err(|error| {
            rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(error))
        })
}

fn optional_datetime_from_column(
    row: &Row<'_>,
    index: usize,
) -> rusqlite::Result<Option<DateTime<Utc>>> {
    let value: Option<String> = row.get(index)?;
    value
        .map(|value| {
            DateTime::parse_from_rfc3339(&value)
                .map(|value| value.with_timezone(&Utc))
                .map_err(|error| {
                    rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(error))
                })
        })
        .transpose()
}

fn decimal_from_column(row: &Row<'_>, index: usize) -> rusqlite::Result<Decimal> {
    let value: String = row.get(index)?;
    Decimal::from_str(&value).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(error))
    })
}
