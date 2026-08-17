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
}
