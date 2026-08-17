use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Asia::Shanghai;
use rust_decimal_macros::dec;
use salary_garden_core::application::{AppSettingsDto, ApplicationContext, WalletDisplayMode};
use salary_garden_core::commands::{
    claim_offline_reward_bag_at, get_app_settings_from_state, get_app_snapshot_at,
    list_offline_reward_bags_from_state, update_app_settings_at, AppState,
};
use salary_garden_core::domain::payroll::{PayrollCycle, WorkCalendar};
use salary_garden_core::persistence::SqliteRepository;

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn local_at(hour: u32, minute: u32, second: u32) -> DateTime<Utc> {
    Shanghai
        .with_ymd_and_hms(2024, 1, 2, hour, minute, second)
        .single()
        .unwrap()
        .with_timezone(&Utc)
}

fn context() -> ApplicationContext {
    let calendar = WorkCalendar::new("phase-3.5-test-calendar", [date(2024, 1, 1)]);
    let cycle = PayrollCycle::for_month(2024, 1, dec!(22000), Shanghai, &calendar).unwrap();
    ApplicationContext::new("cycle-2024-01", cycle, calendar)
}

fn state(at: DateTime<Utc>) -> AppState {
    AppState::initialize(SqliteRepository::open_in_memory().unwrap(), context(), at).unwrap()
}

#[test]
fn app_snapshot_is_composed_from_payroll_reward_and_persistence_layers() {
    let at = local_at(9, 40, 0);
    let state = state(at);
    let snapshot = get_app_snapshot_at(&state, at).unwrap();

    assert_eq!(snapshot.current_local_time, "2024-01-02T09:40:00+08:00");
    assert_eq!(snapshot.work_status, "WORKING_AM");
    assert_eq!(snapshot.effective_work_seconds_today, 3_600);
    assert_eq!(snapshot.payroll_cycle.cycle_id, "cycle-2024-01");
    assert_eq!(snapshot.payroll_cycle.start_date, "2024-01-02");
    assert_eq!(snapshot.payroll_cycle.end_date, "2024-01-31");
    assert_eq!(snapshot.payroll_cycle.workday_count, 22);
    assert_eq!(snapshot.payroll_cycle.monthly_salary_exact, "22000");
    assert_eq!(
        snapshot.real_payroll.today_real_earned_exact,
        (dec!(1000) / dec!(7)).to_string()
    );
    assert_eq!(
        snapshot.real_payroll.cycle_real_earned_exact,
        snapshot.real_payroll.today_real_earned_exact
    );
    assert_eq!(snapshot.reward_entitlement.today.silver, 300);
    assert_eq!(snapshot.reward_entitlement.today.gold, 59);
    assert_eq!(snapshot.reward_entitlement.today.diamond, 1);
    assert_eq!(snapshot.collected_wallet.today_collected_exact, "0");
    assert_eq!(snapshot.collected_wallet.cycle_collected_exact, "0");
    assert_eq!(snapshot.offline.unclaimed_bag_count, 1);
    assert_eq!(
        snapshot.offline.unclaimed_exact_total,
        snapshot.real_payroll.today_real_earned_exact
    );
}

#[test]
fn command_bridge_lists_and_claims_a_bag_via_phase3_transaction() {
    let state = state(local_at(9, 40, 0));
    let bags = list_offline_reward_bags_from_state(&state).unwrap();
    assert_eq!(bags.len(), 1);
    assert_eq!(bags[0].counts.silver, 300);

    let entry = claim_offline_reward_bag_at(&state, &bags[0].bag_id, local_at(9, 41, 0)).unwrap();
    assert_eq!(entry.source_type, "OFFLINE_BAG_CLAIM");
    assert_eq!(entry.source_id, bags[0].bag_id);
    assert!(list_offline_reward_bags_from_state(&state)
        .unwrap()
        .is_empty());

    let snapshot = get_app_snapshot_at(&state, local_at(9, 41, 0)).unwrap();
    assert_eq!(
        snapshot.collected_wallet.today_collected_exact,
        entry.exact_value
    );
    assert_eq!(
        snapshot.collected_wallet.cycle_collected_exact,
        entry.exact_value
    );
    assert_eq!(snapshot.offline.unclaimed_bag_count, 0);
    assert_eq!(snapshot.offline.unclaimed_exact_total, "0");
}

#[test]
fn command_bridge_rejects_a_second_claim_without_changing_snapshot() {
    let state = state(local_at(9, 40, 0));
    let bag = list_offline_reward_bags_from_state(&state)
        .unwrap()
        .remove(0);
    let first = claim_offline_reward_bag_at(&state, &bag.bag_id, local_at(9, 41, 0)).unwrap();
    let error = claim_offline_reward_bag_at(&state, &bag.bag_id, local_at(9, 41, 1)).unwrap_err();

    assert!(error.contains("already been claimed"));
    let snapshot = get_app_snapshot_at(&state, local_at(9, 41, 1)).unwrap();
    assert_eq!(
        snapshot.collected_wallet.cycle_collected_exact,
        first.exact_value
    );
}

#[test]
fn settings_commands_round_trip_typed_values() {
    let state = state(local_at(8, 0, 0));
    assert_eq!(
        get_app_settings_from_state(&state).unwrap(),
        AppSettingsDto::default()
    );

    let updated = AppSettingsDto {
        wallet_display_mode: WalletDisplayMode::CollectedWallet,
        sound_enabled: false,
        auto_collect_enabled: false,
    };
    assert_eq!(
        update_app_settings_at(&state, updated.clone(), local_at(8, 1, 0)).unwrap(),
        updated
    );
    assert_eq!(get_app_settings_from_state(&state).unwrap(), updated);
}

#[test]
fn snapshot_refresh_is_read_only_and_does_not_create_micro_bags() {
    let state = state(local_at(9, 40, 0));
    for second in 1..=10 {
        let snapshot = get_app_snapshot_at(&state, local_at(9, 40, second)).unwrap();
        assert_eq!(snapshot.offline.unclaimed_bag_count, 1);
    }
    assert_eq!(
        list_offline_reward_bags_from_state(&state).unwrap().len(),
        1
    );
}
