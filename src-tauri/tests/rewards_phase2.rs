use chrono::{NaiveDate, TimeZone, Utc};
use chrono_tz::Asia::Shanghai;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use salary_garden_core::domain::{
    payroll::{calculate_payroll_snapshot, PayrollCycle, WorkCalendar, WORK_SECONDS_PER_DAY},
    rewards::{
        calculate_reward_snapshot, reward_snapshot_from_payroll, RewardCounts, RewardValues,
    },
};

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn calendar() -> WorkCalendar {
    WorkCalendar::new("cn-2024-reward-test-v1", [date(2024, 1, 1)])
}

fn cycle(salary: Decimal) -> PayrollCycle {
    PayrollCycle::for_month(2024, 1, salary, Shanghai, &calendar()).unwrap()
}

fn local_at(hour: u32, minute: u32, second: u32) -> chrono::DateTime<Utc> {
    Shanghai
        .with_ymd_and_hms(2024, 1, 2, hour, minute, second)
        .single()
        .unwrap()
        .with_timezone(&Utc)
}

#[test]
fn deterministic_counts_match_all_required_boundaries() {
    let cases = [
        (0, (0, 0, 0)),
        (10, (1, 0, 0)),
        (50, (5, 0, 0)),
        (60, (5, 1, 0)),
        (3_590, (300, 59, 0)),
        (3_600, (300, 59, 1)),
        (10_800, (900, 177, 3)),
        (14_400, (1_200, 236, 4)),
        (25_200, (2_100, 413, 7)),
    ];

    for (work_seconds, (silver, gold, diamond)) in cases {
        let counts = RewardCounts::from_work_seconds(work_seconds);
        assert_eq!(
            counts,
            RewardCounts {
                silver,
                gold,
                diamond
            }
        );
        assert_eq!(counts.total_events(), work_seconds / 10);
    }
}

#[test]
fn boundary_priority_emits_only_the_highest_tier_event() {
    let before_minute = RewardCounts::from_work_seconds(50);
    let at_minute = RewardCounts::from_work_seconds(60);
    assert_eq!(at_minute.silver - before_minute.silver, 0);
    assert_eq!(at_minute.gold - before_minute.gold, 1);
    assert_eq!(at_minute.diamond - before_minute.diamond, 0);

    let before_hour = RewardCounts::from_work_seconds(3_590);
    let at_hour = RewardCounts::from_work_seconds(3_600);
    assert_eq!(at_hour.silver - before_hour.silver, 0);
    assert_eq!(at_hour.gold - before_hour.gold, 0);
    assert_eq!(at_hour.diamond - before_hour.diamond, 1);
}

#[test]
fn denomination_values_follow_the_frozen_one_to_six_to_360_ratio() {
    let cycle = cycle(dec!(22000));
    let values = RewardValues::for_cycle(&cycle);

    assert_eq!(values.gold, values.silver * dec!(6));
    assert_eq!(values.diamond, values.silver * dec!(360));
}

#[test]
fn one_full_hour_reward_value_equals_hourly_pay_exactly() {
    let cycle = cycle(dec!(12345.67));
    let rewards = calculate_reward_snapshot(&cycle, 3_600);

    assert_eq!(
        rewards.counts,
        RewardCounts {
            silver: 300,
            gold: 59,
            diamond: 1
        }
    );
    assert_eq!(rewards.total_value, cycle.pay_rates().hourly);
}

#[test]
fn one_full_workday_reward_value_equals_daily_pay_exactly() {
    let cycle = cycle(dec!(12345.67));
    let rewards = calculate_reward_snapshot(&cycle, u64::from(WORK_SECONDS_PER_DAY));

    assert_eq!(
        rewards.counts,
        RewardCounts {
            silver: 2_100,
            gold: 413,
            diamond: 7
        }
    );
    assert_eq!(rewards.total_value, cycle.pay_rates().daily);

    let two_days = calculate_reward_snapshot(&cycle, u64::from(WORK_SECONDS_PER_DAY) * 2);
    assert_eq!(two_days.total_value, cycle.pay_rates().daily * dec!(2));
}

#[test]
fn full_cycle_reward_value_equals_monthly_salary_exactly() {
    let salary = dec!(12345.67);
    let cycle = cycle(salary);
    let full_cycle_seconds = u64::from(cycle.workday_count) * u64::from(WORK_SECONDS_PER_DAY);
    let rewards = calculate_reward_snapshot(&cycle, full_cycle_seconds);

    assert_eq!(
        rewards.counts,
        RewardCounts {
            silver: 2_100 * u64::from(cycle.workday_count),
            gold: 413 * u64::from(cycle.workday_count),
            diamond: 7 * u64::from(cycle.workday_count),
        }
    );
    assert_eq!(rewards.total_value, salary);
}

#[test]
fn reward_counts_freeze_during_lunch_and_continue_after_it() {
    let calendar = calendar();
    let cycle = cycle(dec!(22000));
    let snapshots = [(11, 40, 0), (12, 30, 0), (13, 30, 0)].map(|(hour, minute, second)| {
        let payroll =
            calculate_payroll_snapshot(&cycle, &calendar, local_at(hour, minute, second)).unwrap();
        reward_snapshot_from_payroll(&cycle, &payroll)
    });

    assert_eq!(
        snapshots[0].counts,
        RewardCounts {
            silver: 900,
            gold: 177,
            diamond: 3
        }
    );
    assert_eq!(snapshots[1].counts, snapshots[0].counts);
    assert_eq!(snapshots[2].counts, snapshots[0].counts);

    let payroll_after_lunch =
        calculate_payroll_snapshot(&cycle, &calendar, local_at(13, 30, 10)).unwrap();
    let rewards_after_lunch = reward_snapshot_from_payroll(&cycle, &payroll_after_lunch);
    assert_eq!(
        rewards_after_lunch.counts,
        RewardCounts {
            silver: 901,
            gold: 177,
            diamond: 3
        }
    );
}

#[test]
fn intermediate_reward_value_is_not_forced_to_equal_real_salary() {
    let calendar = calendar();
    let cycle = cycle(dec!(22000));
    let payroll = calculate_payroll_snapshot(&cycle, &calendar, local_at(8, 40, 10)).unwrap();
    let rewards = reward_snapshot_from_payroll(&cycle, &payroll);

    assert_eq!(
        rewards.counts,
        RewardCounts {
            silver: 1,
            gold: 0,
            diamond: 0
        }
    );
    assert_ne!(rewards.total_value, payroll.today_earned);
}

#[test]
fn reward_snapshot_is_deterministic_for_identical_inputs() {
    let cycle = cycle(dec!(22000));
    let first = calculate_reward_snapshot(&cycle, 14_400);
    let second = calculate_reward_snapshot(&cycle, 14_400);

    assert_eq!(first, second);
}
