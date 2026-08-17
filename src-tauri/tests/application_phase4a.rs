use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Asia::Shanghai;
use rust_decimal_macros::dec;
use salary_garden_core::application::ApplicationEnvironment;
use salary_garden_core::commands::{
    get_app_snapshot_at, get_calendar_month_from_state, get_salary_configuration_at,
    initialize_salary_at, list_offline_reward_bags_from_state, update_next_cycle_salary_at,
    AppState,
};
use salary_garden_core::domain::payroll::{PayrollCycle, WorkCalendar};
use salary_garden_core::persistence::SqliteRepository;
use tempfile::TempDir;

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn local_at(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Shanghai
        .with_ymd_and_hms(year, month, day, hour, minute, 0)
        .single()
        .unwrap()
        .with_timezone(&Utc)
}

fn environment() -> ApplicationEnvironment {
    ApplicationEnvironment::new(
        Shanghai,
        "phase-4a-calendar-v1",
        [(date(2024, 1, 1), "New Year Holiday".to_owned())],
    )
}

fn temporary_database() -> (TempDir, std::path::PathBuf) {
    let directory = tempfile::tempdir().unwrap();
    let path = directory.path().join("salary-garden-phase4a.sqlite3");
    (directory, path)
}

#[test]
fn first_salary_initialization_creates_current_cycle_and_derived_values() {
    let at = local_at(2024, 1, 2, 9, 40);
    let state = AppState::open(
        SqliteRepository::open_in_memory().unwrap(),
        environment(),
        at,
    )
    .unwrap();
    assert!(
        !get_salary_configuration_at(&state, at)
            .unwrap()
            .is_initialized
    );

    let configuration = initialize_salary_at(&state, "22000.00", at).unwrap();
    let cycle = configuration.current_cycle.unwrap();
    assert!(configuration.is_initialized);
    assert_eq!(cycle.cycle_id, "2024-01");
    assert_eq!(cycle.start_date, "2024-01-02");
    assert_eq!(cycle.end_date, "2024-01-31");
    assert_eq!(cycle.workday_count, 22);
    assert_eq!(cycle.monthly_salary_exact, "22000.00");
    assert_eq!(cycle.daily_salary_exact, "1000.00");
    assert_eq!(
        cycle.hourly_salary_exact,
        (dec!(1000) / dec!(7)).to_string()
    );
    assert_eq!(
        cycle.per_second_salary_exact,
        (dec!(1000) / dec!(25200)).to_string()
    );
    assert_eq!(
        dec!(22000) / dec!(22) / dec!(7) / dec!(1014),
        cycle.silver_value_exact.parse().unwrap()
    );
    assert_eq!(
        cycle
            .gold_value_exact
            .parse::<rust_decimal::Decimal>()
            .unwrap(),
        cycle
            .silver_value_exact
            .parse::<rust_decimal::Decimal>()
            .unwrap()
            * dec!(6)
    );
    assert_eq!(
        cycle
            .diamond_value_exact
            .parse::<rust_decimal::Decimal>()
            .unwrap(),
        cycle
            .silver_value_exact
            .parse::<rust_decimal::Decimal>()
            .unwrap()
            * dec!(360)
    );

    let snapshot = get_app_snapshot_at(&state, at).unwrap();
    assert_eq!(snapshot.payroll_cycle.monthly_salary_exact, "22000.00");
    assert_eq!(
        snapshot.real_payroll.today_real_earned_exact,
        cycle.hourly_salary_exact
    );
    assert_eq!(snapshot.reward_entitlement.today.silver, 300);
    assert_eq!(snapshot.reward_entitlement.today.gold, 59);
    assert_eq!(snapshot.reward_entitlement.today.diamond, 1);
    assert_eq!(snapshot.offline.unclaimed_bag_count, 1);
}

#[test]
fn duplicate_initialization_is_idempotent_for_same_salary_and_rejects_change() {
    let at = local_at(2024, 1, 2, 9, 40);
    let state = AppState::open(
        SqliteRepository::open_in_memory().unwrap(),
        environment(),
        at,
    )
    .unwrap();
    let first = initialize_salary_at(&state, "22000", at).unwrap();
    let second = initialize_salary_at(&state, "22000.0", at).unwrap();

    assert_eq!(first.current_cycle, second.current_cycle);
    assert_eq!(
        list_offline_reward_bags_from_state(&state).unwrap().len(),
        1
    );
    assert!(initialize_salary_at(&state, "23000", at)
        .unwrap_err()
        .contains("already initialized"));
}

#[test]
fn salary_configuration_and_current_snapshot_survive_database_restart() {
    let (_directory, path) = temporary_database();
    let at = local_at(2024, 1, 2, 9, 40);
    {
        let state =
            AppState::open(SqliteRepository::open(&path).unwrap(), environment(), at).unwrap();
        initialize_salary_at(&state, "22000", at).unwrap();
    }

    let reopened =
        AppState::open(SqliteRepository::open(&path).unwrap(), environment(), at).unwrap();
    let configuration = get_salary_configuration_at(&reopened, at).unwrap();
    assert!(configuration.is_initialized);
    assert_eq!(
        configuration.current_cycle.unwrap().monthly_salary_exact,
        "22000"
    );
    assert_eq!(
        list_offline_reward_bags_from_state(&reopened)
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn next_cycle_salary_change_never_mutates_current_or_historical_snapshot() {
    let (_directory, path) = temporary_database();
    let january_at = local_at(2024, 1, 2, 9, 40);
    {
        let state = AppState::open(
            SqliteRepository::open(&path).unwrap(),
            environment(),
            january_at,
        )
        .unwrap();
        initialize_salary_at(&state, "22000", january_at).unwrap();
        let configuration = update_next_cycle_salary_at(&state, "29000", january_at).unwrap();
        assert_eq!(
            configuration.current_cycle.unwrap().monthly_salary_exact,
            "22000"
        );
        assert_eq!(
            configuration.next_cycle_salary_exact.as_deref(),
            Some("29000")
        );
    }

    let february_at = local_at(2024, 2, 1, 9, 40);
    {
        let state = AppState::open(
            SqliteRepository::open(&path).unwrap(),
            environment(),
            february_at,
        )
        .unwrap();
        let configuration = get_salary_configuration_at(&state, february_at).unwrap();
        let february = configuration.current_cycle.unwrap();
        assert_eq!(february.cycle_id, "2024-02");
        assert_eq!(february.monthly_salary_exact, "29000");
    }

    let repository = SqliteRepository::open(&path).unwrap();
    assert_eq!(
        repository
            .payroll_cycle("2024-01")
            .unwrap()
            .unwrap()
            .monthly_salary,
        dec!(22000)
    );
    assert_eq!(
        repository
            .payroll_cycle("2024-02")
            .unwrap()
            .unwrap()
            .monthly_salary,
        dec!(29000)
    );
}

#[test]
fn calendar_month_is_authoritative_and_excludes_weekends_and_holidays() {
    let at = local_at(2024, 1, 2, 9, 40);
    let state = AppState::open(
        SqliteRepository::open_in_memory().unwrap(),
        environment(),
        at,
    )
    .unwrap();
    let month = get_calendar_month_from_state(&state, 2024, 1).unwrap();

    assert_eq!(month.cycle_id, "2024-01");
    assert_eq!(month.cycle_start, "2024-01-02");
    assert_eq!(month.cycle_end, "2024-01-31");
    assert_eq!(month.payday, "2024-01-31");
    assert_eq!(month.workday_count, 22);
    assert_eq!(month.days.len(), 31);

    let holiday = &month.days[0];
    assert_eq!(holiday.date, "2024-01-01");
    assert_eq!(holiday.weekday, "MONDAY");
    assert!(holiday.is_holiday);
    assert!(!holiday.is_weekend);
    assert!(!holiday.is_workday);
    assert_eq!(holiday.holiday_name.as_deref(), Some("New Year Holiday"));

    let saturday = &month.days[5];
    assert_eq!(saturday.weekday, "SATURDAY");
    assert!(saturday.is_weekend);
    assert!(!saturday.is_workday);
}

#[test]
fn first_formal_salary_replaces_only_the_phase35_zero_debug_cycle() {
    let at = local_at(2024, 1, 2, 9, 40);
    let calendar = WorkCalendar::new("phase-4a-calendar-v1", [date(2024, 1, 1)]);
    let debug_cycle = PayrollCycle::for_month(2024, 1, dec!(0), Shanghai, &calendar).unwrap();
    let mut repository = SqliteRepository::open_in_memory().unwrap();
    repository
        .ensure_payroll_cycle("2024-01", &debug_cycle, at)
        .unwrap();
    let state = AppState::open(repository, environment(), at).unwrap();

    let configuration = initialize_salary_at(&state, "22000", at).unwrap();
    assert_eq!(
        configuration.current_cycle.unwrap().monthly_salary_exact,
        "22000"
    );
    assert_eq!(
        list_offline_reward_bags_from_state(&state).unwrap().len(),
        1
    );
}
