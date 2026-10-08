//! Shared job types for the upscale service.
//!
//! The GPU pipeline is intentionally absent. This crate records what an agent asked for.

mod error;
mod job;

pub use error::Error;
pub use job::{Job, JobId, JobStatus, JobStore, MediaKind, UpscaleRequest};

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{JobStatus, JobStore, MediaKind, UpscaleRequest};

    #[test]
    fn create_then_get_returns_the_same_job() {
        let store = JobStore::default();
        let created = store
            .create(UpscaleRequest {
                kind: MediaKind::Image,
                input_path: PathBuf::from("in.png"),
                output_path: PathBuf::from("out.png"),
            })
            .expect("create");
        let fetched = store.get(&created.id).expect("get");
        assert_eq!(fetched, created);
        assert_eq!(fetched.status, JobStatus::Queued);
    }

    #[test]
    fn rejects_a_non_uuid_job_id() {
        let err = super::JobId::parse("not-a-uuid").expect_err("invalid");
        assert!(matches!(err, super::Error::InvalidJobId));
    }
}
