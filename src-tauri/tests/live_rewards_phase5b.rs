use std::sync::{Arc, Barrier};
use std::thread;

use chrono::{DateTime, NaiveDate, TimeDelta, TimeZone, Utc};
use chrono_tz::Asia::Shanghai;
use rust_decimal_macros::dec;
use salary_garden_core::application::ApplicationContext;
use salary_garden_core::commands::{
    collect_live_reward_at, list_offline_reward_bags_from_state,
    list_pending_live_rewards_from_state, sync_live_rewards_at, AppState,
};
use salary_garden_core::domain::{
    payroll::{PayrollCycle, WorkCalendar},
    rewards::{RewardCounts, RewardType, RewardValues},
};
use salary_garden_core::persistence::{LiveRewardStatus, PersistenceError, SqliteRepository};
use salary_garden_core::services::{RewardLedgerService, MAX_PENDING_LIVE_REWARDS};
use tempfile::TempDir;

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn at(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
    Shanghai
        .with_ymd_and_hms(2024, 1, 2, hour, minute, second)
        .single()
        .unwrap()
        .with_timezone(&Utc)
}

fn calendar() -> WorkCalendar {
    WorkCalendar::new("phase-5b-calendar", [date(2024, 1, 1)])
}

fn cycle(month: u32) -> PayrollCycle {
    PayrollCycle::for_month(2024, month, dec!(22000), Shanghai, &calendar()).unwrap()
}

fn materialize(
    repository: &mut SqliteRepository,
    cycle_id: &str,
    payroll_cycle: &PayrollCycle,
    work_seconds: u64,
    now: DateTime<Utc>,
) -> Vec<salary_garden_core::persistence::LiveRewardEvent> {
    RewardLedgerService::new(repository)
        .materialize_live_rewards(cycle_id, payroll_cycle, date(2024, 1, 2), work_seconds, now)
        .unwrap()
}

fn temporary_database() -> (TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("salary-garden.sqlite3");
    (directory, path)
}

#[test]
fn ten_second_boundary_materializes_silver() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let events = materialize(&mut repository, "cycle-1", &cycle(1), 10, at(8, 40, 10));
    assert_eq!(events.len(), 1);
    assert_eq!(events[0].event_index, 1);
    assert_eq!(events[0].reward_type, RewardType::Silver);
}

#[test]
fn sixty_second_boundary_materializes_gold_not_silver() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let events = materialize(&mut repository, "cycle-1", &cycle(1), 60, at(8, 41, 0));
    assert_eq!(events.len(), 6);
    assert_eq!(events[5].event_index, 6);
    assert_eq!(events[5].reward_type, RewardType::Gold);
    assert_eq!(
        events
            .iter()
            .filter(|event| event.reward_type == RewardType::Silver)
            .count(),
        5
    );
}

#[test]
fn full_hour_boundary_materializes_diamond() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    let mut diamond = None;
    for batch in 0..30 {
        let events = materialize(
            &mut repository,
            "cycle-1",
            &payroll_cycle,
            3_600,
            at(9, 40, 0) + TimeDelta::seconds(batch),
        );
        for event in events {
            if event.event_index == 360 {
                diamond = Some(event.clone());
            }
            RewardLedgerService::new(&mut repository)
                .collect_live_reward(&event.event_id, at(9, 41, 0) + TimeDelta::seconds(batch))
                .unwrap();
        }
    }
    assert_eq!(diamond.unwrap().reward_type, RewardType::Diamond);
}

#[test]
fn deterministic_key_prevents_duplicate_materialization() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    let first = materialize(
        &mut repository,
        "cycle-1",
        &payroll_cycle,
        10,
        at(8, 40, 10),
    );
    let second = materialize(
        &mut repository,
        "cycle-1",
        &payroll_cycle,
        10,
        at(8, 40, 11),
    );
    assert_eq!(first[0].event_id, second[0].event_id);
    assert_eq!(repository.live_reward_event_count(None).unwrap(), 1);
}

#[test]
fn materialization_increments_accounted_exactly_once() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    materialize(&mut repository, "cycle-1", &payroll_cycle, 60, at(8, 41, 0));
    materialize(&mut repository, "cycle-1", &payroll_cycle, 60, at(8, 41, 1));
    let state = repository
        .daily_reward_state("cycle-1", date(2024, 1, 2))
        .unwrap()
        .unwrap();
    assert_eq!(state.accounted, RewardCounts::from_work_seconds(60));
}

#[test]
fn collecting_pending_reward_credits_wallet_once() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    let event = materialize(
        &mut repository,
        "cycle-1",
        &payroll_cycle,
        10,
        at(8, 40, 10),
    )[0]
    .clone();
    let entry = RewardLedgerService::new(&mut repository)
        .collect_live_reward(&event.event_id, at(8, 40, 11))
        .unwrap();
    assert_eq!(entry.counts.silver, 1);
    assert_eq!(
        entry.exact_value,
        RewardValues::for_cycle(&payroll_cycle).silver
    );
    assert_eq!(
        repository
            .cycle_wallet_totals("cycle-1")
            .unwrap()
            .exact_value,
        entry.exact_value
    );
}

#[test]
fn double_collect_is_rejected_without_second_ledger_entry() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let event = materialize(&mut repository, "cycle-1", &cycle(1), 10, at(8, 40, 10))[0].clone();
    RewardLedgerService::new(&mut repository)
        .collect_live_reward(&event.event_id, at(8, 40, 11))
        .unwrap();
    let error = RewardLedgerService::new(&mut repository)
        .collect_live_reward(&event.event_id, at(8, 40, 12))
        .unwrap_err();
    assert!(matches!(
        error,
        PersistenceError::LiveRewardAlreadySettled { .. }
    ));
    assert_eq!(repository.collection_ledger_count().unwrap(), 1);
}

#[test]
fn missing_or_packaged_event_never_credits_wallet() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let missing = RewardLedgerService::new(&mut repository)
        .collect_live_reward("missing", at(8, 40, 10))
        .unwrap_err();
    assert!(matches!(missing, PersistenceError::LiveRewardNotFound(_)));

    let event = materialize(&mut repository, "cycle-1", &cycle(1), 10, at(8, 40, 10))[0].clone();
    RewardLedgerService::new(&mut repository)
        .package_pending_live_rewards(at(8, 40, 11))
        .unwrap();
    let packaged = RewardLedgerService::new(&mut repository)
        .collect_live_reward(&event.event_id, at(8, 40, 12))
        .unwrap_err();
    assert!(matches!(
        packaged,
        PersistenceError::LiveRewardAlreadySettled { .. }
    ));
    assert_eq!(repository.collection_ledger_count().unwrap(), 0);
    assert_eq!(
        repository.wallet_totals().unwrap().counts,
        RewardCounts::default()
    );
}

#[test]
fn pending_reward_survives_repository_reopen() {
    let (_directory, path) = temporary_database();
    let event_id = {
        let mut repository = SqliteRepository::open(&path).unwrap();
        materialize(&mut repository, "cycle-1", &cycle(1), 10, at(8, 40, 10))[0]
            .event_id
            .clone()
    };
    let repository = SqliteRepository::open(&path).unwrap();
    assert_eq!(
        repository.pending_live_rewards("cycle-1").unwrap()[0].event_id,
        event_id
    );
}

#[test]
fn stale_pending_is_packaged_without_incrementing_accounted_twice() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    materialize(&mut repository, "cycle-1", &payroll_cycle, 60, at(8, 41, 0));
    let before = repository
        .daily_reward_state("cycle-1", date(2024, 1, 2))
        .unwrap()
        .unwrap()
        .accounted;
    let bags = RewardLedgerService::new(&mut repository)
        .package_pending_live_rewards(at(8, 42, 0))
        .unwrap();
    let after = repository
        .daily_reward_state("cycle-1", date(2024, 1, 2))
        .unwrap()
        .unwrap()
        .accounted;
    assert_eq!(bags.len(), 1);
    assert_eq!(bags[0].counts, before);
    assert_eq!(after, before);
    assert_eq!(
        repository
            .live_reward_event_count(Some(LiveRewardStatus::Packaged))
            .unwrap(),
        6
    );
}

#[test]
fn repeated_stale_packaging_does_not_duplicate_bag() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    materialize(&mut repository, "cycle-1", &cycle(1), 10, at(8, 40, 10));
    assert_eq!(
        RewardLedgerService::new(&mut repository)
            .package_pending_live_rewards(at(8, 41, 0))
            .unwrap()
            .len(),
        1
    );
    assert!(RewardLedgerService::new(&mut repository)
        .package_pending_live_rewards(at(8, 42, 0))
        .unwrap()
        .is_empty());
    assert_eq!(repository.offline_reward_bag_count().unwrap(), 1);
}

#[test]
fn payroll_cycles_keep_live_events_isolated() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    materialize(&mut repository, "cycle-1", &cycle(1), 10, at(8, 40, 10));
    materialize(&mut repository, "cycle-2", &cycle(2), 10, at(8, 40, 10));
    assert_eq!(repository.pending_live_rewards("cycle-1").unwrap().len(), 1);
    assert_eq!(repository.pending_live_rewards("cycle-2").unwrap().len(), 1);
    assert_ne!(
        repository.pending_live_rewards("cycle-1").unwrap()[0].event_id,
        repository.pending_live_rewards("cycle-2").unwrap()[0].event_id
    );
}

#[test]
fn pending_screen_cap_preserves_unmaterialized_gap() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    let events = materialize(
        &mut repository,
        "cycle-1",
        &payroll_cycle,
        500,
        at(8, 48, 20),
    );
    assert_eq!(events.len(), MAX_PENDING_LIVE_REWARDS);
    let state = repository
        .daily_reward_state("cycle-1", date(2024, 1, 2))
        .unwrap()
        .unwrap();
    assert_eq!(state.entitled, RewardCounts::from_work_seconds(500));
    assert_eq!(
        state.accounted.total_events(),
        MAX_PENDING_LIVE_REWARDS as u64
    );
    assert!(state.entitled.total_events() > state.accounted.total_events());
}

#[test]
fn collecting_one_capped_event_allows_next_gap_event_to_materialize() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    let first = materialize(
        &mut repository,
        "cycle-1",
        &payroll_cycle,
        500,
        at(8, 48, 20),
    );
    RewardLedgerService::new(&mut repository)
        .collect_live_reward(&first[0].event_id, at(8, 48, 21))
        .unwrap();
    let second = materialize(
        &mut repository,
        "cycle-1",
        &payroll_cycle,
        500,
        at(8, 48, 22),
    );
    assert_eq!(second.len(), MAX_PENDING_LIVE_REWARDS);
    assert!(second.iter().any(|event| event.event_index == 13));
}

#[test]
fn unmaterialized_gap_is_recoverable_by_offline_reconciliation() {
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    let payroll_cycle = cycle(1);
    materialize(
        &mut repository,
        "cycle-1",
        &payroll_cycle,
        500,
        at(8, 48, 20),
    );
    RewardLedgerService::new(&mut repository)
        .package_pending_live_rewards(at(8, 49, 0))
        .unwrap();
    let bag = RewardLedgerService::new(&mut repository)
        .reconcile_offline(
            "cycle-1",
            &payroll_cycle,
            at(8, 40, 0),
            at(8, 49, 0),
            &[salary_garden_core::services::DailyEntitlement {
                work_date: date(2024, 1, 2),
                effective_work_seconds: 500,
            }],
            at(8, 49, 0),
        )
        .unwrap()
        .unwrap();
    assert_eq!(
        bag.counts.total_events(),
        50 - MAX_PENDING_LIVE_REWARDS as u64
    );
}

#[test]
fn simulated_manual_and_magnet_race_settles_once() {
    let (_directory, path) = temporary_database();
    let event_id = {
        let mut repository = SqliteRepository::open(&path).unwrap();
        materialize(&mut repository, "cycle-1", &cycle(1), 10, at(8, 40, 10))[0]
            .event_id
            .clone()
    };
    let barrier = Arc::new(Barrier::new(2));
    let handles: Vec<_> = (0..2)
        .map(|offset| {
            let path = path.clone();
            let event_id = event_id.clone();
            let barrier = Arc::clone(&barrier);
            thread::spawn(move || {
                let mut repository = SqliteRepository::open(path).unwrap();
                barrier.wait();
                RewardLedgerService::new(&mut repository)
                    .collect_live_reward(&event_id, at(8, 40, 11) + TimeDelta::seconds(offset))
            })
        })
        .collect();
    let results: Vec<_> = handles
        .into_iter()
        .map(|handle| handle.join().unwrap())
        .collect();
    assert_eq!(results.iter().filter(|result| result.is_ok()).count(), 1);
    let repository = SqliteRepository::open(&path).unwrap();
    assert_eq!(repository.collection_ledger_count().unwrap(), 1);
    assert_eq!(
        repository
            .cycle_wallet_totals("cycle-1")
            .unwrap()
            .counts
            .silver,
        1
    );
}

#[test]
fn dashboard_remount_lists_same_pending_identity() {
    let context = ApplicationContext::new("cycle-1", cycle(1), calendar());
    let state = AppState::initialize(
        SqliteRepository::open_in_memory().unwrap(),
        context,
        at(8, 40, 0),
    )
    .unwrap();
    let first = sync_live_rewards_at(&state, at(8, 40, 10)).unwrap();
    let second = list_pending_live_rewards_from_state(&state).unwrap();
    assert_eq!(first, second);
}

#[test]
fn collected_event_does_not_return_after_dashboard_remount() {
    let context = ApplicationContext::new("cycle-1", cycle(1), calendar());
    let state = AppState::initialize(
        SqliteRepository::open_in_memory().unwrap(),
        context,
        at(8, 40, 0),
    )
    .unwrap();
    let event = sync_live_rewards_at(&state, at(8, 40, 10))
        .unwrap()
        .remove(0);
    collect_live_reward_at(&state, &event.event_id, at(8, 40, 11)).unwrap();
    assert!(list_pending_live_rewards_from_state(&state)
        .unwrap()
        .is_empty());
}

#[test]
fn inactivity_packages_pending_then_reconciles_new_gap() {
    let context = ApplicationContext::new("cycle-1", cycle(1), calendar());
    let state = AppState::initialize(
        SqliteRepository::open_in_memory().unwrap(),
        context,
        at(8, 40, 0),
    )
    .unwrap();
    sync_live_rewards_at(&state, at(8, 40, 10)).unwrap();
    let pending = sync_live_rewards_at(&state, at(8, 41, 0)).unwrap();
    assert!(pending.is_empty());
    assert!(list_pending_live_rewards_from_state(&state)
        .unwrap()
        .is_empty());
}

#[test]
fn restart_packages_pending_once_without_duplicate_bag() {
    let (_directory, path) = temporary_database();
    {
        let context = ApplicationContext::new("cycle-1", cycle(1), calendar());
        let state = AppState::initialize(
            SqliteRepository::open(&path).unwrap(),
            context,
            at(8, 40, 0),
        )
        .unwrap();
        assert_eq!(
            sync_live_rewards_at(&state, at(8, 40, 10)).unwrap().len(),
            1
        );
    }
    {
        let context = ApplicationContext::new("cycle-1", cycle(1), calendar());
        let reopened = AppState::initialize(
            SqliteRepository::open(&path).unwrap(),
            context,
            at(8, 40, 11),
        )
        .unwrap();
        assert_eq!(
            list_offline_reward_bags_from_state(&reopened)
                .unwrap()
                .len(),
            1
        );
        assert!(list_pending_live_rewards_from_state(&reopened)
            .unwrap()
            .is_empty());
    }
    let context = ApplicationContext::new("cycle-1", cycle(1), calendar());
    let reopened_again = AppState::initialize(
        SqliteRepository::open(&path).unwrap(),
        context,
        at(8, 40, 12),
    )
    .unwrap();
    assert_eq!(
        list_offline_reward_bags_from_state(&reopened_again)
            .unwrap()
            .len(),
        1
    );
}
