use std::collections::BTreeMap;
use std::str::FromStr;

use chrono::{DateTime, Datelike, Days, NaiveDate, Utc, Weekday};
use chrono_tz::{Asia::Shanghai, Tz};
use rust_decimal::Decimal;

use crate::domain::payroll::{PayrollCycle, WorkCalendar};
use crate::domain::rewards::RewardValues;
use crate::persistence::{PayrollCycleRecord, SqliteRepository};

use super::{
    ApplicationContext, ApplicationError, CalendarDayDto, CalendarMonthDto, SalaryConfigurationDto,
    SalaryCycleDto,
};

const DEFAULT_CALENDAR_VERSION: &str = "weekdays-only-v1";
const SETTING_SALARY_INITIALIZED: &str = "salary_initialized";
const SETTING_SALARY_TIMEZONE: &str = "salary_timezone";
const SETTING_CURRENT_CYCLE_ID: &str = "salary_current_cycle_id";
const SETTING_NEXT_CYCLE_SALARY: &str = "next_cycle_salary_exact";
const SETTING_SALARY_CALENDAR_VERSION: &str = "salary_calendar_version";

#[derive(Clone, Debug)]
pub struct ApplicationEnvironment {
    pub timezone: Tz,
    pub calendar: WorkCalendar,
    holiday_names: BTreeMap<NaiveDate, String>,
}

impl Default for ApplicationEnvironment {
    fn default() -> Self {
        Self {
            timezone: Shanghai,
            calendar: WorkCalendar::new(DEFAULT_CALENDAR_VERSION, []),
            holiday_names: BTreeMap::new(),
        }
    }
}

impl ApplicationEnvironment {
    pub fn from_calendar(timezone: Tz, calendar: WorkCalendar) -> Self {
        Self {
            timezone,
            calendar,
            holiday_names: BTreeMap::new(),
        }
    }

    pub fn new(
        timezone: Tz,
        calendar_version: impl Into<String>,
        holidays: impl IntoIterator<Item = (NaiveDate, String)>,
    ) -> Self {
        let holiday_names: BTreeMap<_, _> = holidays.into_iter().collect();
        let calendar = WorkCalendar::new(
            calendar_version,
            holiday_names.keys().copied().collect::<Vec<_>>(),
        );
        Self {
            timezone,
            calendar,
            holiday_names,
        }
    }
}

pub struct SalaryConfigurationService<'repository> {
    repository: &'repository mut SqliteRepository,
    environment: &'repository ApplicationEnvironment,
}

impl<'repository> SalaryConfigurationService<'repository> {
    pub fn new(
        repository: &'repository mut SqliteRepository,
        environment: &'repository ApplicationEnvironment,
    ) -> Self {
        Self {
            repository,
            environment,
        }
    }

    pub fn get_salary_configuration(
        &mut self,
        at_utc: DateTime<Utc>,
    ) -> Result<(SalaryConfigurationDto, Option<ApplicationContext>), ApplicationError> {
        let local_date = at_utc
            .with_timezone(&self.environment.timezone)
            .date_naive();
        if !self.is_initialized()? {
            return Ok((
                SalaryConfigurationDto {
                    is_initialized: false,
                    timezone: self.environment.timezone.to_string(),
                    current_year: local_date.year(),
                    current_month: local_date.month(),
                    current_cycle: None,
                    next_cycle_salary_exact: None,
                },
                None,
            ));
        }

        let context = self.ensure_current_cycle(at_utc)?;
        let next_salary = self.repository.setting(SETTING_NEXT_CYCLE_SALARY)?.ok_or(
            ApplicationError::IncompleteSalaryConfiguration(SETTING_NEXT_CYCLE_SALARY),
        )?;
        let dto = SalaryConfigurationDto {
            is_initialized: true,
            timezone: self.environment.timezone.to_string(),
            current_year: local_date.year(),
            current_month: local_date.month(),
            current_cycle: Some(SalaryCycleDto::from_cycle(
                &context.cycle_id,
                &context.cycle,
            )),
            next_cycle_salary_exact: Some(next_salary),
        };
        Ok((dto, Some(context)))
    }

    pub fn initialize_salary(
        &mut self,
        monthly_salary_exact: &str,
        at_utc: DateTime<Utc>,
    ) -> Result<(SalaryConfigurationDto, ApplicationContext), ApplicationError> {
        let salary = parse_positive_salary(monthly_salary_exact)?;
        if self.is_initialized()? {
            let (configuration, context) = self.get_salary_configuration(at_utc)?;
            let context = context.ok_or(ApplicationError::SalaryNotInitialized)?;
            if context.cycle.monthly_salary == salary {
                return Ok((configuration, context));
            }
            return Err(ApplicationError::SalaryAlreadyInitialized);
        }

        let local_date = at_utc
            .with_timezone(&self.environment.timezone)
            .date_naive();
        let cycle_id = cycle_id(local_date.year(), local_date.month());
        if let Some(existing) = self.repository.payroll_cycle(&cycle_id)? {
            if existing.monthly_salary.is_zero() {
                self.repository.remove_unconfigured_zero_cycle(&cycle_id)?;
            } else if existing.monthly_salary != salary {
                return Err(ApplicationError::SalaryAlreadyInitialized);
            }
        }

        let cycle = PayrollCycle::for_month(
            local_date.year(),
            local_date.month(),
            salary,
            self.environment.timezone,
            &self.environment.calendar,
        )?;
        self.repository
            .ensure_payroll_cycle(&cycle_id, &cycle, at_utc)?;
        self.persist_configuration(&cycle_id, salary, at_utc)?;
        let context = ApplicationContext::new(cycle_id, cycle, self.environment.calendar.clone());
        let dto = SalaryConfigurationDto {
            is_initialized: true,
            timezone: self.environment.timezone.to_string(),
            current_year: local_date.year(),
            current_month: local_date.month(),
            current_cycle: Some(SalaryCycleDto::from_cycle(
                &context.cycle_id,
                &context.cycle,
            )),
            next_cycle_salary_exact: Some(salary.to_string()),
        };
        Ok((dto, context))
    }

    pub fn update_next_cycle_salary(
        &mut self,
        monthly_salary_exact: &str,
        at_utc: DateTime<Utc>,
    ) -> Result<SalaryConfigurationDto, ApplicationError> {
        if !self.is_initialized()? {
            return Err(ApplicationError::SalaryNotInitialized);
        }
        let salary = parse_positive_salary(monthly_salary_exact)?;
        self.repository.set_settings(
            &[(SETTING_NEXT_CYCLE_SALARY.to_owned(), salary.to_string())],
            at_utc,
        )?;
        Ok(self.get_salary_configuration(at_utc)?.0)
    }

    pub fn get_calendar_month(
        &self,
        year: i32,
        month: u32,
    ) -> Result<CalendarMonthDto, ApplicationError> {
        let month_start = NaiveDate::from_ymd_opt(year, month, 1)
            .ok_or(ApplicationError::InvalidCalendarMonth { year, month })?;
        let next_month = if month == 12 {
            year.checked_add(1)
                .and_then(|next_year| NaiveDate::from_ymd_opt(next_year, 1, 1))
        } else {
            NaiveDate::from_ymd_opt(year, month + 1, 1)
        }
        .ok_or(ApplicationError::InvalidCalendarMonth { year, month })?;
        let id = cycle_id(year, month);
        let cycle = if let Some(record) = self.repository.payroll_cycle(&id)? {
            self.context_from_record(record)?.cycle
        } else {
            PayrollCycle::for_month(
                year,
                month,
                Decimal::ZERO,
                self.environment.timezone,
                &self.environment.calendar,
            )?
        };

        let mut date = month_start;
        let mut days = Vec::new();
        while date < next_month {
            let weekday = date.weekday();
            let is_weekend = matches!(weekday, Weekday::Sat | Weekday::Sun);
            let is_holiday = self.environment.calendar.is_holiday(date);
            days.push(CalendarDayDto {
                date: date.to_string(),
                weekday: weekday_code(weekday).to_owned(),
                is_workday: self.environment.calendar.is_workday(date),
                is_weekend,
                is_holiday,
                holiday_name: is_holiday
                    .then(|| self.environment.holiday_names.get(&date).cloned())
                    .flatten(),
            });
            date = date
                .checked_add_days(Days::new(1))
                .ok_or(ApplicationError::InvalidCalendarMonth { year, month })?;
        }

        Ok(CalendarMonthDto {
            year,
            month,
            timezone: self.environment.timezone.to_string(),
            cycle_id: id,
            cycle_start: cycle.start_date.to_string(),
            cycle_end: cycle.end_date.to_string(),
            workday_count: cycle.workday_count,
            payday: cycle.end_date.to_string(),
            days,
        })
    }

    fn ensure_current_cycle(
        &mut self,
        at_utc: DateTime<Utc>,
    ) -> Result<ApplicationContext, ApplicationError> {
        let local_date = at_utc
            .with_timezone(&self.environment.timezone)
            .date_naive();
        let id = cycle_id(local_date.year(), local_date.month());
        if let Some(record) = self.repository.payroll_cycle(&id)? {
            let context = self.context_from_record(record)?;
            self.repository
                .set_settings(&[(SETTING_CURRENT_CYCLE_ID.to_owned(), id)], at_utc)?;
            return Ok(context);
        }

        let next_salary = self.repository.setting(SETTING_NEXT_CYCLE_SALARY)?.ok_or(
            ApplicationError::IncompleteSalaryConfiguration(SETTING_NEXT_CYCLE_SALARY),
        )?;
        let salary = parse_positive_salary(&next_salary)?;
        let cycle = PayrollCycle::for_month(
            local_date.year(),
            local_date.month(),
            salary,
            self.environment.timezone,
            &self.environment.calendar,
        )?;
        self.repository.ensure_payroll_cycle(&id, &cycle, at_utc)?;
        self.repository
            .set_settings(&[(SETTING_CURRENT_CYCLE_ID.to_owned(), id.clone())], at_utc)?;
        Ok(ApplicationContext::new(
            id,
            cycle,
            self.environment.calendar.clone(),
        ))
    }

    fn context_from_record(
        &self,
        record: PayrollCycleRecord,
    ) -> Result<ApplicationContext, ApplicationError> {
        if record.calendar_version != self.environment.calendar.version() {
            return Err(ApplicationError::CalendarVersionMismatch(record.cycle_id));
        }
        let cycle = PayrollCycle {
            salary_month: record.salary_month,
            start_date: record.start_date,
            end_date: record.end_date,
            monthly_salary: record.monthly_salary,
            workday_count: record.workday_count,
            timezone: self.environment.timezone,
            calendar_version: record.calendar_version,
        };
        Ok(ApplicationContext::new(
            record.cycle_id,
            cycle,
            self.environment.calendar.clone(),
        ))
    }

    fn persist_configuration(
        &mut self,
        cycle_id: &str,
        salary: Decimal,
        at_utc: DateTime<Utc>,
    ) -> Result<(), ApplicationError> {
        self.repository.set_settings(
            &[
                (SETTING_SALARY_INITIALIZED.to_owned(), "true".to_owned()),
                (
                    SETTING_SALARY_TIMEZONE.to_owned(),
                    self.environment.timezone.to_string(),
                ),
                (SETTING_CURRENT_CYCLE_ID.to_owned(), cycle_id.to_owned()),
                (SETTING_NEXT_CYCLE_SALARY.to_owned(), salary.to_string()),
                (
                    SETTING_SALARY_CALENDAR_VERSION.to_owned(),
                    self.environment.calendar.version().to_owned(),
                ),
            ],
            at_utc,
        )?;
        Ok(())
    }

    fn is_initialized(&self) -> Result<bool, ApplicationError> {
        Ok(matches!(
            self.repository
                .setting(SETTING_SALARY_INITIALIZED)?
                .as_deref(),
            Some("true")
        ))
    }
}

impl SalaryCycleDto {
    fn from_cycle(cycle_id: &str, cycle: &PayrollCycle) -> Self {
        let rates = cycle.pay_rates();
        let values = RewardValues::for_cycle(cycle);
        Self {
            cycle_id: cycle_id.to_owned(),
            start_date: cycle.start_date.to_string(),
            end_date: cycle.end_date.to_string(),
            workday_count: cycle.workday_count,
            monthly_salary_exact: cycle.monthly_salary.to_string(),
            daily_salary_exact: rates.daily.to_string(),
            hourly_salary_exact: rates.hourly.to_string(),
            per_second_salary_exact: rates.per_second.to_string(),
            silver_value_exact: values.silver.to_string(),
            gold_value_exact: values.gold.to_string(),
            diamond_value_exact: values.diamond.to_string(),
        }
    }
}

fn parse_positive_salary(value: &str) -> Result<Decimal, ApplicationError> {
    let salary = Decimal::from_str(value.trim()).map_err(|_| ApplicationError::InvalidSalary)?;
    if salary <= Decimal::ZERO {
        return Err(ApplicationError::InvalidSalary);
    }
    Ok(salary)
}

fn cycle_id(year: i32, month: u32) -> String {
    format!("{year:04}-{month:02}")
}

fn weekday_code(weekday: Weekday) -> &'static str {
    match weekday {
        Weekday::Mon => "MONDAY",
        Weekday::Tue => "TUESDAY",
        Weekday::Wed => "WEDNESDAY",
        Weekday::Thu => "THURSDAY",
        Weekday::Fri => "FRIDAY",
        Weekday::Sat => "SATURDAY",
        Weekday::Sun => "SUNDAY",
    }
}
