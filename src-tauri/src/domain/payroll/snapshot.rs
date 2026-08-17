use chrono::{DateTime, Days, Utc};
use rust_decimal::Decimal;

use super::{
    effective_work_seconds, work_status, PayrollCycle, PayrollError, WorkCalendar, WorkStatus,
    WORK_SECONDS_PER_DAY,
};

#[derive(Clone, Debug, PartialEq)]
pub struct PayrollSnapshot {
    pub local_datetime: DateTime<chrono_tz::Tz>,
    pub work_status: WorkStatus,
    pub effective_work_seconds_today: u32,
    pub completed_workdays_before_today: u32,
    pub today_earned: Decimal,
    pub cycle_earned: Decimal,
}

pub fn calculate_payroll_snapshot(
    cycle: &PayrollCycle,
    calendar: &WorkCalendar,
    at_utc: DateTime<Utc>,
) -> Result<PayrollSnapshot, PayrollError> {
    if cycle.calendar_version != calendar.version() {
        return Err(PayrollError::CalendarVersionMismatch {
            cycle_version: cycle.calendar_version.clone(),
            calendar_version: calendar.version().to_owned(),
        });
    }

    let local_datetime = at_utc.with_timezone(&cycle.timezone);
    let local_date = local_datetime.date_naive();
    let local_time = local_datetime.time();
    let status = work_status(local_date, local_time, calendar);

    let within_cycle = (cycle.start_date..=cycle.end_date).contains(&local_date);
    let effective_seconds_today = if within_cycle {
        effective_work_seconds(local_date, local_time, calendar)
    } else {
        0
    };

    let (completed_workdays_before_today, elapsed_effective_seconds) =
        if local_date < cycle.start_date {
            (0, 0_u64)
        } else if local_date > cycle.end_date {
            (
                cycle.workday_count,
                u64::from(cycle.workday_count) * u64::from(WORK_SECONDS_PER_DAY),
            )
        } else {
            let completed = match local_date.checked_sub_days(Days::new(1)) {
                Some(yesterday) => calendar.workdays_inclusive(cycle.start_date, yesterday)?,
                None => 0,
            };
            (
                completed,
                u64::from(completed) * u64::from(WORK_SECONDS_PER_DAY)
                    + u64::from(effective_seconds_today),
            )
        };

    let total_cycle_seconds = u64::from(cycle.workday_count) * u64::from(WORK_SECONDS_PER_DAY);
    let today_earned = proportional_salary(
        cycle.monthly_salary,
        u64::from(effective_seconds_today),
        total_cycle_seconds,
    );
    let cycle_earned = proportional_salary(
        cycle.monthly_salary,
        elapsed_effective_seconds,
        total_cycle_seconds,
    );

    Ok(PayrollSnapshot {
        local_datetime,
        work_status: status,
        effective_work_seconds_today: effective_seconds_today,
        completed_workdays_before_today,
        today_earned,
        cycle_earned,
    })
}

fn proportional_salary(monthly_salary: Decimal, elapsed: u64, total: u64) -> Decimal {
    if elapsed == 0 || monthly_salary.is_zero() {
        Decimal::ZERO
    } else if elapsed >= total {
        // This explicit terminal branch preserves the primary Phase 1 invariant
        // even when an intermediate daily rate has a repeating decimal expansion.
        monthly_salary
    } else {
        monthly_salary * Decimal::from(elapsed) / Decimal::from(total)
    }
}
