use std::collections::BTreeSet;

use chrono::{Datelike, Days, NaiveDate, Weekday};

use super::PayrollError;

/// The frozen MVP work calendar: Monday-Friday, excluding configured holidays.
/// Weekend make-up workdays are deliberately unsupported by the product rules.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorkCalendar {
    version: String,
    holidays: BTreeSet<NaiveDate>,
}

impl WorkCalendar {
    pub fn new(version: impl Into<String>, holidays: impl IntoIterator<Item = NaiveDate>) -> Self {
        Self {
            version: version.into(),
            holidays: holidays.into_iter().collect(),
        }
    }

    pub fn version(&self) -> &str {
        &self.version
    }

    pub fn is_holiday(&self, date: NaiveDate) -> bool {
        self.holidays.contains(&date)
    }

    pub fn is_workday(&self, date: NaiveDate) -> bool {
        !matches!(date.weekday(), Weekday::Sat | Weekday::Sun) && !self.is_holiday(date)
    }

    pub fn workdays_inclusive(
        &self,
        start: NaiveDate,
        end: NaiveDate,
    ) -> Result<u32, PayrollError> {
        if start > end {
            return Ok(0);
        }

        let mut date = start;
        let mut count = 0;
        loop {
            if self.is_workday(date) {
                count += 1;
            }
            if date == end {
                return Ok(count);
            }
            date = date
                .checked_add_days(Days::new(1))
                .ok_or(PayrollError::DateOverflow)?;
        }
    }

    pub(crate) fn first_workday_after(&self, date: NaiveDate) -> Result<NaiveDate, PayrollError> {
        let mut candidate = date
            .checked_add_days(Days::new(1))
            .ok_or(PayrollError::DateOverflow)?;
        while !self.is_workday(candidate) {
            candidate = candidate
                .checked_add_days(Days::new(1))
                .ok_or(PayrollError::DateOverflow)?;
        }
        Ok(candidate)
    }

    pub(crate) fn last_workday_before(&self, date: NaiveDate) -> Result<NaiveDate, PayrollError> {
        let mut candidate = date
            .checked_sub_days(Days::new(1))
            .ok_or(PayrollError::DateOverflow)?;
        while !self.is_workday(candidate) {
            candidate = candidate
                .checked_sub_days(Days::new(1))
                .ok_or(PayrollError::DateOverflow)?;
        }
        Ok(candidate)
    }
}
