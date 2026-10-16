//! Shared job types for the upscale service.

mod error;
mod job;

pub use error::Error;
pub use job::{Job, JobId, JobStatus, JobStore, MediaKind, UpscaleRequest, parse_scale};
