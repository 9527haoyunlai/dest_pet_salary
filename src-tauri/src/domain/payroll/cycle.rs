use chrono::{Datelike, NaiveDate};
use chrono_tz::Tz;
use rust_decimal::Decimal;

use super::{PayrollError, WorkCalendar, WORK_SECONDS_PER_DAY};

const WORK_HOURS_PER_DAY: u32 = 7;

#[derive(Clone, Debug, PartialEq)]
pub struct PayRates {
    pub daily: Decimal,
    pub hourly: Decimal,
    pub per_second: Decimal,
}

/// An immutable Phase 1 payroll-cycle snapshot.
#[derive(Clone, Debug, PartialEq)]
pub struct PayrollCycle {
    pub salary_month: NaiveDate,
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
    pub monthly_salary: Decimal,
    pub workday_count: u32,
    pub timezone: Tz,
    pub calendar_version: String,
}

impl PayrollCycle {
    pub fn for_month(
        year: i32,
        month: u32,
        monthly_salary: Decimal,
        timezone: Tz,
        calendar: &WorkCalendar,
    ) -> Result<Self, PayrollError> {
        if !(1..=12).contains(&month) {
            return Err(PayrollError::InvalidMonth(month));
        }
        if monthly_salary.is_sign_negative() {
            return Err(PayrollError::NegativeSalary);
        }

        let month_start =
            NaiveDate::from_ymd_opt(year, month, 1).ok_or(PayrollError::InvalidMonth(month))?;
        let next_month_start = if month == 12 {
            NaiveDate::from_ymd_opt(year.checked_add(1).ok_or(PayrollError::DateOverflow)?, 1, 1)
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or(PayrollError::DateOverflow)?;

        // Implement the SSOT definition literally: start at the first workday after
        // the previous month's last workday, and end at this month's last workday.
        let previous_month_last_workday = calendar.last_workday_before(month_start)?;
        let start_date = calendar.first_workday_after(previous_month_last_workday)?;
        let end_date = calendar.last_workday_before(next_month_start)?;

        if start_date < month_start
            || start_date >= next_month_start
            || end_date < month_start
            || start_date > end_date
        {
            return Err(PayrollError::NoWorkdays { year, month });
        }

        let workday_count = calendar.workdays_inclusive(start_date, end_date)?;
        if workday_count == 0 {
            return Err(PayrollError::NoWorkdays { year, month });
        }

        Ok(Self {
            salary_month: month_start,
            start_date,
            end_date,
            monthly_salary,
            workday_count,
            timezone,
            calendar_version: calendar.version().to_owned(),
        })
    }

    pub fn pay_rates(&self) -> PayRates {
        let daily = self.monthly_salary / Decimal::from(self.workday_count);
        let hourly = daily / Decimal::from(WORK_HOURS_PER_DAY);
        let per_second = daily / Decimal::from(WORK_SECONDS_PER_DAY);
        PayRates {
            daily,
            hourly,
            per_second,
        }
    }

    pub fn year(&self) -> i32 {
        self.salary_month.year()
    }

    pub fn month(&self) -> u32 {
        self.salary_month.month()
    }
}
