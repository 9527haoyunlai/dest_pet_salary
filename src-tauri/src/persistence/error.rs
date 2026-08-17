use thiserror::Error;

#[derive(Debug, Error)]
pub enum PersistenceError {
    #[error("SQLite operation failed: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("invalid exact decimal stored in SQLite: {0}")]
    Decimal(#[from] rust_decimal::Error),
    #[error("invalid date/time stored in SQLite: {0}")]
    DateTime(#[from] chrono::ParseError),
    #[error("payroll cycle id must not be empty")]
    EmptyCycleId,
    #[error("setting key must not be empty")]
    EmptySettingKey,
    #[error("payroll cycle snapshot conflicts with persisted cycle `{0}`")]
    CycleSnapshotConflict(String),
    #[error("offline reward bag `{0}` was not found")]
    BagNotFound(String),
    #[error("offline reward bag `{0}` has already been claimed")]
    BagAlreadyClaimed(String),
    #[error("invalid reconciliation period: period_start is after period_end")]
    InvalidPeriod,
    #[error("duplicate daily entitlement for {0}")]
    DuplicateEntitlementDate(String),
    #[error("effective work seconds for {work_date} exceed the daily maximum: {seconds}")]
    EffectiveWorkSecondsOutOfRange { work_date: String, seconds: u64 },
    #[error("reward count exceeds SQLite's signed integer range")]
    CountOutOfRange,
    #[error("persistence invariant violated: {0}")]
    InvariantViolation(&'static str),
}
