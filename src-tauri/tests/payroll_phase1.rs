use chrono::{NaiveDate, TimeZone, Timelike, Utc};
use chrono_tz::{Asia::Shanghai, Tz};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use salary_garden_core::domain::payroll::{
    calculate_payroll_snapshot, effective_work_seconds, work_status, PayrollCycle, PayrollError,
    WorkCalendar, WorkStatus, WORK_SECONDS_PER_DAY,
};

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn january_2024_calendar() -> WorkCalendar {
    WorkCalendar::new("cn-2024-test-v1", [date(2024, 1, 1)])
}

fn january_2024_cycle(salary: Decimal) -> PayrollCycle {
    PayrollCycle::for_month(2024, 1, salary, Shanghai, &january_2024_calendar()).unwrap()
}

fn local_at(
    tz: Tz,
    year: i32,
    month: u32,
    day: u32,
    hour: u32,
    minute: u32,
    second: u32,
) -> chrono::DateTime<Utc> {
    tz.with_ymd_and_hms(year, month, day, hour, minute, second)
        .single()
        .unwrap()
        .with_timezone(&Utc)
}

#[test]
fn work_calendar_excludes_weekends_and_configured_holidays() {
    let calendar = january_2024_calendar();

    assert!(!calendar.is_workday(date(2024, 1, 1))); // Monday holiday
    assert!(calendar.is_workday(date(2024, 1, 2))); // Tuesday
    assert!(!calendar.is_workday(date(2024, 1, 6))); // Saturday
    assert!(!calendar.is_workday(date(2024, 1, 7))); // Sunday
    assert_eq!(
        calendar
            .workdays_inclusive(date(2024, 1, 1), date(2024, 1, 31))
            .unwrap(),
        22
    );
}

#[test]
fn payroll_cycle_uses_first_and_last_actual_workday() {
    let cycle = january_2024_cycle(dec!(22000));

    assert_eq!(cycle.start_date, date(2024, 1, 2));
    assert_eq!(cycle.end_date, date(2024, 1, 31));
    assert_eq!(cycle.workday_count, 22);
    assert_eq!(cycle.calendar_version, "cn-2024-test-v1");
    assert_eq!(cycle.timezone, Shanghai);

    let rates = cycle.pay_rates();
    assert_eq!(rates.daily, dec!(1000));
    assert_eq!(rates.hourly, dec!(1000) / dec!(7));
    assert_eq!(
        rates.per_second,
        dec!(1000) / Decimal::from(WORK_SECONDS_PER_DAY)
    );
}

#[test]
fn work_status_and_effective_seconds_follow_all_frozen_boundaries() {
    let calendar = january_2024_calendar();
    let workday = date(2024, 1, 2);
    let cases = [
        ((8, 39, 59), WorkStatus::BeforeWork, 0),
        ((8, 40, 0), WorkStatus::WorkingAm, 0),
        ((8, 40, 10), WorkStatus::WorkingAm, 10),
        ((9, 40, 0), WorkStatus::WorkingAm, 3_600),
        ((11, 40, 0), WorkStatus::LunchBreak, 10_800),
        ((12, 30, 0), WorkStatus::LunchBreak, 10_800),
        ((13, 30, 0), WorkStatus::WorkingPm, 10_800),
        ((13, 30, 10), WorkStatus::WorkingPm, 10_810),
        ((17, 30, 0), WorkStatus::AfterWork, 25_200),
        ((23, 59, 59), WorkStatus::AfterWork, 25_200),
    ];

    for ((hour, minute, second), expected_status, expected_seconds) in cases {
        let time = chrono::NaiveTime::from_hms_opt(hour, minute, second).unwrap();
        assert_eq!(work_status(workday, time, &calendar), expected_status);
        assert_eq!(
            effective_work_seconds(workday, time, &calendar),
            expected_seconds
        );
        assert_eq!(
            expected_status.as_code(),
            match expected_status {
                WorkStatus::NonWorkday => "NON_WORKDAY",
                WorkStatus::BeforeWork => "BEFORE_WORK",
                WorkStatus::WorkingAm => "WORKING_AM",
                WorkStatus::LunchBreak => "LUNCH_BREAK",
                WorkStatus::WorkingPm => "WORKING_PM",
                WorkStatus::AfterWork => "AFTER_WORK",
            }
        );
    }
}

#[test]
fn non_workdays_never_accrue_effective_seconds() {
    let calendar = january_2024_calendar();
    let noon = chrono::NaiveTime::from_hms_opt(12, 0, 0).unwrap();

    for non_workday in [date(2024, 1, 1), date(2024, 1, 6), date(2024, 1, 7)] {
        assert_eq!(
            work_status(non_workday, noon, &calendar),
            WorkStatus::NonWorkday
        );
        assert_eq!(effective_work_seconds(non_workday, noon, &calendar), 0);
    }
}

#[test]
fn today_earned_is_derived_from_time_and_freezes_during_lunch() {
    let calendar = january_2024_calendar();
    let cycle = january_2024_cycle(dec!(22000));
    let morning_end =
        calculate_payroll_snapshot(&cycle, &calendar, local_at(Shanghai, 2024, 1, 2, 11, 40, 0))
            .unwrap();
    let lunch =
        calculate_payroll_snapshot(&cycle, &calendar, local_at(Shanghai, 2024, 1, 2, 12, 30, 0))
            .unwrap();
    let afternoon_start =
        calculate_payroll_snapshot(&cycle, &calendar, local_at(Shanghai, 2024, 1, 2, 13, 30, 0))
            .unwrap();

    assert_eq!(morning_end.today_earned, dec!(3000) / dec!(7));
    assert_eq!(lunch.today_earned, morning_end.today_earned);
    assert_eq!(afternoon_start.today_earned, morning_end.today_earned);
    assert_eq!(morning_end.cycle_earned, lunch.cycle_earned);
    assert_eq!(lunch.cycle_earned, afternoon_start.cycle_earned);
}

#[test]
fn first_hour_equals_hourly_pay_without_accumulation() {
    let calendar = january_2024_calendar();
    let cycle = january_2024_cycle(dec!(22000));
    let snapshot =
        calculate_payroll_snapshot(&cycle, &calendar, local_at(Shanghai, 2024, 1, 2, 9, 40, 0))
            .unwrap();

    assert_eq!(snapshot.effective_work_seconds_today, 3_600);
    assert_eq!(snapshot.today_earned, cycle.pay_rates().hourly);
    assert_eq!(snapshot.cycle_earned, cycle.pay_rates().hourly);
}

#[test]
fn cycle_earned_counts_completed_workdays_plus_current_partial_day() {
    let calendar = january_2024_calendar();
    let cycle = january_2024_cycle(dec!(22000));
    let snapshot =
        calculate_payroll_snapshot(&cycle, &calendar, local_at(Shanghai, 2024, 1, 3, 9, 40, 0))
            .unwrap();

    assert_eq!(snapshot.completed_workdays_before_today, 1);
    assert_eq!(snapshot.today_earned, dec!(1000) / dec!(7));
    assert_eq!(snapshot.cycle_earned, dec!(1000) + dec!(1000) / dec!(7));
}

#[test]
fn weekend_freezes_today_and_preserves_completed_cycle_earnings() {
    let calendar = january_2024_calendar();
    let cycle = january_2024_cycle(dec!(22000));
    let snapshot =
        calculate_payroll_snapshot(&cycle, &calendar, local_at(Shanghai, 2024, 1, 6, 15, 0, 0))
            .unwrap();

    assert_eq!(snapshot.work_status, WorkStatus::NonWorkday);
    assert_eq!(snapshot.today_earned, Decimal::ZERO);
    assert_eq!(snapshot.completed_workdays_before_today, 4);
    assert_eq!(snapshot.cycle_earned, dec!(4000));
}

#[test]
fn full_day_and_full_cycle_end_at_exact_salary_invariants() {
    let calendar = january_2024_calendar();
    let salary = dec!(12345.67);
    let cycle = january_2024_cycle(salary);

    let first_day_end =
        calculate_payroll_snapshot(&cycle, &calendar, local_at(Shanghai, 2024, 1, 2, 17, 30, 0))
            .unwrap();
    assert_eq!(first_day_end.today_earned, cycle.pay_rates().daily);

    let cycle_end = calculate_payroll_snapshot(
        &cycle,
        &calendar,
        local_at(Shanghai, 2024, 1, 31, 17, 30, 0),
    )
    .unwrap();
    assert_eq!(cycle_end.effective_work_seconds_today, WORK_SECONDS_PER_DAY);
    assert_eq!(cycle_end.cycle_earned, salary);

    let after_cycle =
        calculate_payroll_snapshot(&cycle, &calendar, local_at(Shanghai, 2024, 2, 10, 12, 0, 0))
            .unwrap();
    assert_eq!(after_cycle.today_earned, Decimal::ZERO);
    assert_eq!(after_cycle.cycle_earned, salary);
}

#[test]
fn local_timezone_not_utc_controls_the_payroll_clock() {
    let calendar = january_2024_calendar();
    let cycle = january_2024_cycle(dec!(22000));
    let at_utc = Utc.with_ymd_and_hms(2024, 1, 2, 0, 40, 10).unwrap();
    let snapshot = calculate_payroll_snapshot(&cycle, &calendar, at_utc).unwrap();

    assert_eq!(snapshot.local_datetime.hour(), 8);
    assert_eq!(snapshot.local_datetime.minute(), 40);
    assert_eq!(snapshot.local_datetime.second(), 10);
    assert_eq!(snapshot.effective_work_seconds_today, 10);
}

#[test]
fn repeated_snapshot_calculation_is_deterministic() {
    let calendar = january_2024_calendar();
    let cycle = january_2024_cycle(dec!(22000));
    let at = local_at(Shanghai, 2024, 1, 16, 14, 20, 37);

    let first = calculate_payroll_snapshot(&cycle, &calendar, at).unwrap();
    let second = calculate_payroll_snapshot(&cycle, &calendar, at).unwrap();
    assert_eq!(first, second);
}

#[test]
fn invalid_cycle_input_and_calendar_mutation_are_rejected() {
    let calendar = january_2024_calendar();
    assert_eq!(
        PayrollCycle::for_month(2024, 13, dec!(10000), Shanghai, &calendar).unwrap_err(),
        PayrollError::InvalidMonth(13)
    );
    assert_eq!(
        PayrollCycle::for_month(2024, 1, dec!(-0.01), Shanghai, &calendar).unwrap_err(),
        PayrollError::NegativeSalary
    );

    let cycle = january_2024_cycle(dec!(22000));
    let changed_calendar = WorkCalendar::new("cn-2024-test-v2", [date(2024, 1, 1)]);
    let error = calculate_payroll_snapshot(
        &cycle,
        &changed_calendar,
        local_at(Shanghai, 2024, 1, 2, 9, 0, 0),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        PayrollError::CalendarVersionMismatch { .. }
    ));
}
