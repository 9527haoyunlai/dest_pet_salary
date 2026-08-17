use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum PayrollError {
    #[error("month must be in 1..=12, got {0}")]
    InvalidMonth(u32),

    #[error("monthly salary cannot be negative")]
    NegativeSalary,

    #[error("the payroll month {year:04}-{month:02} has no workdays")]
    NoWorkdays { year: i32, month: u32 },

    #[error("date arithmetic exceeded chrono's supported range")]
    DateOverflow,

    #[error(
        "calendar version mismatch: cycle uses '{cycle_version}', calculation uses '{calendar_version}'"
    )]
    CalendarVersionMismatch {
        cycle_version: String,
        calendar_version: String,
    },
}
