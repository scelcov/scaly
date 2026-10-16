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
    #[error("input file was not found")]
    InputNotFound,
    #[error("no pasted image was found")]
    NoChatImage,
    #[error("no pasted video was found")]
    NoChatVideo,
    #[error("scale must be 2, 3, or 4")]
    InvalidScale,
    #[error("output path is the input path")]
    SamePath,
    #[error("upscale failed: {0}")]
    Upscale(String),
}

impl<T> From<PoisonError<T>> for Error {
    fn from(_err: PoisonError<T>) -> Self {
        Self::LockPoisoned
    }
}
