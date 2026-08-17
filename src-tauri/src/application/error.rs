use thiserror::Error;

use crate::domain::payroll::PayrollError;
use crate::persistence::PersistenceError;

#[derive(Debug, Error)]
pub enum ApplicationError {
    #[error(transparent)]
    Payroll(#[from] PayrollError),
    #[error(transparent)]
    Persistence(#[from] PersistenceError),
    #[error("unable to resolve the configured local date/time")]
    InvalidLocalDateTime,
    #[error("monthly salary must be a positive exact decimal string")]
    InvalidSalary,
    #[error("salary configuration has not been initialized")]
    SalaryNotInitialized,
    #[error("salary is already initialized for the current payroll cycle")]
    SalaryAlreadyInitialized,
    #[error("persisted salary configuration is incomplete: {0}")]
    IncompleteSalaryConfiguration(&'static str),
    #[error("invalid calendar month: {year}-{month}")]
    InvalidCalendarMonth { year: i32, month: u32 },
    #[error("configured calendar version does not match payroll cycle `{0}`")]
    CalendarVersionMismatch(String),
}
