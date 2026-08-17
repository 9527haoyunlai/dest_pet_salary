use chrono::{NaiveDate, NaiveTime};

use super::WorkCalendar;

pub const WORK_SECONDS_PER_DAY: u32 = 25_200;
const MORNING_SECONDS: u32 = 10_800;

fn work_start() -> NaiveTime {
    NaiveTime::from_hms_opt(8, 40, 0).expect("the frozen work-start time is valid")
}

fn lunch_start() -> NaiveTime {
    NaiveTime::from_hms_opt(11, 40, 0).expect("the frozen lunch-start time is valid")
}

fn afternoon_start() -> NaiveTime {
    NaiveTime::from_hms_opt(13, 30, 0).expect("the frozen afternoon-start time is valid")
}

fn work_end() -> NaiveTime {
    NaiveTime::from_hms_opt(17, 30, 0).expect("the frozen work-end time is valid")
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WorkStatus {
    NonWorkday,
    BeforeWork,
    WorkingAm,
    LunchBreak,
    WorkingPm,
    AfterWork,
}

impl WorkStatus {
    pub const fn as_code(self) -> &'static str {
        match self {
            Self::NonWorkday => "NON_WORKDAY",
            Self::BeforeWork => "BEFORE_WORK",
            Self::WorkingAm => "WORKING_AM",
            Self::LunchBreak => "LUNCH_BREAK",
            Self::WorkingPm => "WORKING_PM",
            Self::AfterWork => "AFTER_WORK",
        }
    }
}

pub fn work_status(date: NaiveDate, time: NaiveTime, calendar: &WorkCalendar) -> WorkStatus {
    if !calendar.is_workday(date) {
        WorkStatus::NonWorkday
    } else if time < work_start() {
        WorkStatus::BeforeWork
    } else if time < lunch_start() {
        WorkStatus::WorkingAm
    } else if time < afternoon_start() {
        WorkStatus::LunchBreak
    } else if time < work_end() {
        WorkStatus::WorkingPm
    } else {
        WorkStatus::AfterWork
    }
}

/// Returns completed whole effective work seconds for the local work date.
pub fn effective_work_seconds(date: NaiveDate, time: NaiveTime, calendar: &WorkCalendar) -> u32 {
    match work_status(date, time, calendar) {
        WorkStatus::NonWorkday | WorkStatus::BeforeWork => 0,
        WorkStatus::WorkingAm => time
            .signed_duration_since(work_start())
            .num_seconds()
            .try_into()
            .expect("working-AM duration is non-negative and fits u32"),
        WorkStatus::LunchBreak => MORNING_SECONDS,
        WorkStatus::WorkingPm => {
            let afternoon_seconds: u32 = time
                .signed_duration_since(afternoon_start())
                .num_seconds()
                .try_into()
                .expect("working-PM duration is non-negative and fits u32");
            MORNING_SECONDS + afternoon_seconds
        }
        WorkStatus::AfterWork => WORK_SECONDS_PER_DAY,
    }
}
