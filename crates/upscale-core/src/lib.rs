//! Shared job types for the upscale service.
//!
//! The GPU pipeline is intentionally absent. This crate records what an agent asked for.

mod error;
mod job;

pub use error::Error;
pub use job::{Job, JobId, JobStatus, JobStore, MediaKind, UpscaleRequest};
