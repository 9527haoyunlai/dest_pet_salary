use chrono::{DateTime, NaiveDate, TimeDelta, TimeZone, Utc};
use chrono_tz::Asia::Shanghai;
use rust_decimal_macros::dec;
use salary_garden_core::domain::{
    payroll::{PayrollCycle, WorkCalendar},
    rewards::{RewardCounts, RewardValues},
};
use salary_garden_core::persistence::{PersistenceError, SqliteRepository};
use salary_garden_core::services::{DailyEntitlement, RewardLedgerService};
use tempfile::TempDir;

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn at(year: i32, month: u32, day: u32, hour: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, 0, 0).unwrap()
}

fn calendar() -> WorkCalendar {
    WorkCalendar::new("cn-2024-persistence-test-v1", [date(2024, 1, 1)])
}

fn cycle(month: u32) -> PayrollCycle {
    PayrollCycle::for_month(2024, month, dec!(22000), Shanghai, &calendar()).unwrap()
}

fn temporary_database() -> (TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("salary-garden.sqlite3");
    (directory, path)
}

fn reconcile(
    repository: &mut SqliteRepository,
    cycle_id: &str,
    payroll_cycle: &PayrollCycle,
    work_date: NaiveDate,
    work_seconds: u64,
    reconciled_at: DateTime<Utc>,
) -> Option<salary_garden_core::persistence::OfflineRewardBag> {
    let mut service = RewardLedgerService::new(repository);
    service
        .reconcile_offline(
            cycle_id,
            payroll_cycle,
            reconciled_at - TimeDelta::hours(1),
            reconciled_at,
            &[DailyEntitlement {
                work_date,
                effective_work_seconds: work_seconds,
            }],
            reconciled_at,
        )
        .unwrap()
}

#[test]
fn first_startup_migrations_cycle_snapshot_and_settings_are_idempotent() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    let now = at(2024, 1, 2, 9);

    let first = repository
        .ensure_payroll_cycle("cycle-2024-01", &payroll_cycle, now)
        .unwrap();
    let second = repository
        .ensure_payroll_cycle("cycle-2024-01", &payroll_cycle, now + TimeDelta::seconds(1))
        .unwrap();
    repository
        .set_setting("launch_on_startup", "false", now)
        .unwrap();
    repository
        .set_setting("launch_on_startup", "true", now + TimeDelta::seconds(1))
        .unwrap();

    assert_eq!(repository.schema_version().unwrap(), 2);
    assert_eq!(repository.payroll_cycle_count().unwrap(), 1);
    assert_eq!(first, second);
    assert_eq!(
        repository.setting("launch_on_startup").unwrap().as_deref(),
        Some("true")
    );
    assert_eq!(repository.offline_reward_bag_count().unwrap(), 0);
    assert_eq!(repository.collection_ledger_count().unwrap(), 0);
}

#[test]
fn offline_gap_creates_one_bag_and_accounts_it_atomically() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    let work_date = date(2024, 1, 2);
    let bag = reconcile(
        &mut repository,
        "cycle-2024-01",
        &payroll_cycle,
        work_date,
        3_600,
        at(2024, 1, 2, 10),
    )
    .unwrap();
    let expected = RewardCounts {
        silver: 300,
        gold: 59,
        diamond: 1,
    };

    assert_eq!(bag.counts, expected);
    assert_eq!(
        bag.exact_value,
        RewardValues::for_cycle(&payroll_cycle).total_value(expected)
    );
    assert!(!bag.claimed);
    let state = repository
        .daily_reward_state("cycle-2024-01", work_date)
        .unwrap()
        .unwrap();
    assert_eq!(state.entitled, expected);
    assert_eq!(state.accounted, expected);
    assert_eq!(state.collected, RewardCounts::default());
}

#[test]
fn ten_restarts_do_not_create_duplicate_offline_bags() {
    let (_directory, database_path) = temporary_database();
    let payroll_cycle = cycle(1);
    let work_date = date(2024, 1, 2);

    for launch in 0..10 {
        let mut repository = SqliteRepository::open(&database_path).unwrap();
        let bag = reconcile(
            &mut repository,
            "cycle-2024-01",
            &payroll_cycle,
            work_date,
            3_600,
            at(2024, 1, 2, 10) + TimeDelta::seconds(launch),
        );
        assert_eq!(bag.is_some(), launch == 0);
    }

    let repository = SqliteRepository::open(&database_path).unwrap();
    assert_eq!(repository.offline_reward_bag_count().unwrap(), 1);
}

#[test]
fn claiming_bag_increases_only_the_collected_wallet() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    let work_date = date(2024, 1, 2);
    let bag = reconcile(
        &mut repository,
        "cycle-2024-01",
        &payroll_cycle,
        work_date,
        3_600,
        at(2024, 1, 2, 10),
    )
    .unwrap();

    let entry = RewardLedgerService::new(&mut repository)
        .claim_offline_bag(&bag.bag_id, at(2024, 1, 2, 11))
        .unwrap();
    let wallet = repository.wallet_totals().unwrap();
    let state = repository
        .daily_reward_state("cycle-2024-01", work_date)
        .unwrap()
        .unwrap();

    assert_eq!(entry.counts, bag.counts);
    assert_eq!(entry.exact_value, bag.exact_value);
    assert_eq!(wallet.counts, bag.counts);
    assert_eq!(wallet.exact_value, bag.exact_value);
    assert_eq!(state.entitled, bag.counts);
    assert_eq!(state.accounted, bag.counts);
    assert_eq!(state.collected, bag.counts);
}

#[test]
fn same_bag_cannot_be_claimed_twice() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    let bag = reconcile(
        &mut repository,
        "cycle-2024-01",
        &payroll_cycle,
        date(2024, 1, 2),
        60,
        at(2024, 1, 2, 9),
    )
    .unwrap();
    RewardLedgerService::new(&mut repository)
        .claim_offline_bag(&bag.bag_id, at(2024, 1, 2, 10))
        .unwrap();

    let error = RewardLedgerService::new(&mut repository)
        .claim_offline_bag(&bag.bag_id, at(2024, 1, 2, 11))
        .unwrap_err();
    assert!(matches!(error, PersistenceError::BagAlreadyClaimed(id) if id == bag.bag_id));
}

#[test]
fn double_claim_creates_only_one_collection_ledger_entry() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    let bag = reconcile(
        &mut repository,
        "cycle-2024-01",
        &payroll_cycle,
        date(2024, 1, 2),
        3_600,
        at(2024, 1, 2, 10),
    )
    .unwrap();
    let first = RewardLedgerService::new(&mut repository)
        .claim_offline_bag(&bag.bag_id, at(2024, 1, 2, 11))
        .unwrap();
    assert!(RewardLedgerService::new(&mut repository)
        .claim_offline_bag(&bag.bag_id, at(2024, 1, 2, 11))
        .is_err());

    assert_eq!(repository.collection_ledger_count().unwrap(), 1);
    let ledger = repository.collection_ledger(None).unwrap();
    assert_eq!(ledger, vec![first]);
    assert_eq!(repository.wallet_totals().unwrap().counts, bag.counts);
}

#[test]
fn accounted_counts_never_exceed_the_high_water_entitlement() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    let work_date = date(2024, 1, 2);

    let first = reconcile(
        &mut repository,
        "cycle-2024-01",
        &payroll_cycle,
        work_date,
        60,
        at(2024, 1, 2, 9),
    )
    .unwrap();
    let second = reconcile(
        &mut repository,
        "cycle-2024-01",
        &payroll_cycle,
        work_date,
        3_600,
        at(2024, 1, 2, 10),
    )
    .unwrap();
    assert_eq!(
        RewardCounts {
            silver: first.counts.silver + second.counts.silver,
            gold: first.counts.gold + second.counts.gold,
            diamond: first.counts.diamond + second.counts.diamond,
        },
        RewardCounts::from_work_seconds(3_600)
    );

    let state = repository
        .daily_reward_state("cycle-2024-01", work_date)
        .unwrap()
        .unwrap();
    assert_eq!(state.accounted, state.entitled);
    assert!(state.accounted.silver <= state.entitled.silver);
    assert!(state.accounted.gold <= state.entitled.gold);
    assert!(state.accounted.diamond <= state.entitled.diamond);

    let error = RewardLedgerService::new(&mut repository)
        .reconcile_offline(
            "cycle-2024-01",
            &payroll_cycle,
            at(2024, 1, 2, 10),
            at(2024, 1, 2, 11),
            &[DailyEntitlement {
                work_date,
                effective_work_seconds: 25_201,
            }],
            at(2024, 1, 2, 11),
        )
        .unwrap_err();
    assert!(matches!(
        error,
        PersistenceError::EffectiveWorkSecondsOutOfRange {
            seconds: 25_201,
            ..
        }
    ));
    assert_eq!(repository.offline_reward_bag_count().unwrap(), 2);
}

#[test]
fn payroll_cycles_keep_reward_state_and_wallet_entries_isolated() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let january = cycle(1);
    let february = cycle(2);
    let january_bag = reconcile(
        &mut repository,
        "cycle-2024-01",
        &january,
        date(2024, 1, 2),
        60,
        at(2024, 1, 2, 9),
    )
    .unwrap();
    let february_bag = reconcile(
        &mut repository,
        "cycle-2024-02",
        &february,
        date(2024, 2, 1),
        3_600,
        at(2024, 2, 1, 10),
    )
    .unwrap();
    RewardLedgerService::new(&mut repository)
        .claim_offline_bag(&january_bag.bag_id, at(2024, 2, 1, 11))
        .unwrap();

    assert_eq!(repository.payroll_cycle_count().unwrap(), 2);
    assert_eq!(
        repository
            .offline_reward_bags(Some("cycle-2024-01"))
            .unwrap(),
        vec![repository
            .offline_reward_bag(&january_bag.bag_id)
            .unwrap()
            .unwrap()]
    );
    assert_eq!(
        repository
            .offline_reward_bags(Some("cycle-2024-02"))
            .unwrap(),
        vec![february_bag]
    );
    assert_eq!(
        repository
            .collection_ledger(Some("cycle-2024-01"))
            .unwrap()
            .len(),
        1
    );
    assert!(repository
        .collection_ledger(Some("cycle-2024-02"))
        .unwrap()
        .is_empty());
}

#[test]
fn unclaimed_bag_from_previous_cycle_survives_cycle_change() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let january_bag = reconcile(
        &mut repository,
        "cycle-2024-01",
        &cycle(1),
        date(2024, 1, 31),
        3_600,
        at(2024, 1, 31, 10),
    )
    .unwrap();
    let _february_bag = reconcile(
        &mut repository,
        "cycle-2024-02",
        &cycle(2),
        date(2024, 2, 1),
        60,
        at(2024, 2, 1, 9),
    )
    .unwrap();

    let persisted = repository
        .offline_reward_bag(&january_bag.bag_id)
        .unwrap()
        .unwrap();
    assert!(!persisted.claimed);
    assert_eq!(repository.offline_reward_bags(None).unwrap().len(), 2);

    RewardLedgerService::new(&mut repository)
        .claim_offline_bag(&january_bag.bag_id, at(2024, 2, 2, 9))
        .unwrap();
    assert_eq!(
        repository.wallet_totals().unwrap().counts,
        january_bag.counts
    );
}

#[test]
fn clock_rollback_produces_no_negative_gap_and_preserves_history() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    let work_date = date(2024, 1, 2);
    let original = reconcile(
        &mut repository,
        "cycle-2024-01",
        &payroll_cycle,
        work_date,
        3_600,
        at(2024, 1, 2, 10),
    )
    .unwrap();
    RewardLedgerService::new(&mut repository)
        .claim_offline_bag(&original.bag_id, at(2024, 1, 2, 11))
        .unwrap();

    let rollback = reconcile(
        &mut repository,
        "cycle-2024-01",
        &payroll_cycle,
        work_date,
        60,
        at(2024, 1, 2, 9),
    );
    assert!(rollback.is_none());
    assert_eq!(repository.offline_reward_bag_count().unwrap(), 1);
    assert_eq!(repository.collection_ledger_count().unwrap(), 1);
    let state = repository
        .daily_reward_state("cycle-2024-01", work_date)
        .unwrap()
        .unwrap();
    assert_eq!(state.entitled, RewardCounts::from_work_seconds(3_600));
    assert_eq!(state.accounted, state.entitled);
    assert_eq!(state.collected, state.entitled);
}

#[test]
fn closing_and_reopening_database_restores_bags_wallet_state_and_settings() {
    let (_directory, database_path) = temporary_database();
    let payroll_cycle = cycle(1);
    let work_date = date(2024, 1, 2);
    let bag;
    {
        let mut repository = SqliteRepository::open(&database_path).unwrap();
        bag = reconcile(
            &mut repository,
            "cycle-2024-01",
            &payroll_cycle,
            work_date,
            3_600,
            at(2024, 1, 2, 10),
        )
        .unwrap();
        repository
            .set_setting("locale", "zh-CN", at(2024, 1, 2, 10))
            .unwrap();
        RewardLedgerService::new(&mut repository)
            .claim_offline_bag(&bag.bag_id, at(2024, 1, 2, 11))
            .unwrap();
    }

    let repository = SqliteRepository::open(&database_path).unwrap();
    let restored_bag = repository.offline_reward_bag(&bag.bag_id).unwrap().unwrap();
    let restored_state = repository
        .daily_reward_state("cycle-2024-01", work_date)
        .unwrap()
        .unwrap();
    assert!(restored_bag.claimed);
    assert_eq!(restored_bag.claimed_at, Some(at(2024, 1, 2, 11)));
    assert_eq!(restored_state.collected, bag.counts);
    assert_eq!(repository.wallet_totals().unwrap().counts, bag.counts);
    assert_eq!(
        repository.setting("locale").unwrap().as_deref(),
        Some("zh-CN")
    );
    assert_eq!(repository.collection_ledger_count().unwrap(), 1);
}

#[test]
fn immutable_payroll_cycle_snapshot_rejects_conflicting_reuse_of_cycle_id() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let now = at(2024, 1, 2, 9);
    repository
        .ensure_payroll_cycle("cycle-2024", &cycle(1), now)
        .unwrap();

    let error = repository
        .ensure_payroll_cycle("cycle-2024", &cycle(2), now)
        .unwrap_err();
    assert!(matches!(
        error,
        PersistenceError::CycleSnapshotConflict(id) if id == "cycle-2024"
    ));
    assert_eq!(repository.payroll_cycle_count().unwrap(), 1);
}
