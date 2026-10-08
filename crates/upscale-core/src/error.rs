use std::sync::PoisonError;

/// Failures a caller can match on.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("job id is not a uuid")]
    InvalidJobId,
    #[error("job {0} was not found")]
    NotFound(String),
    #[error("job store lock was poisoned")]
    LockPoisoned,
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_err: PoisonError<T>) -> Self {
        Self::LockPoisoned
    }
}
