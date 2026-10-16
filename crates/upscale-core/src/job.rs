use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, RwLock};

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::error::Error;

/// Identifier parsed at the API boundary. Only valid UUIDs exist.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct JobId(Uuid);

impl JobId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(raw: &str) -> Result<Self, Error> {
        Uuid::parse_str(raw)
            .map(Self)
            .map_err(|_| Error::InvalidJobId)
    }

    pub fn as_str(&self) -> String {
        self.0.to_string()
    }
}

impl Default for JobId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for JobId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for JobId {
    type Err = Error;

    fn from_str(raw: &str) -> Result<Self, Self::Err> {
        Self::parse(raw)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MediaKind {
    Image,
    Video,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Queued,
    Running,
    Completed,
    Failed,
}

/// What an agent asks the service to upscale. Paths stay on this machine.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpscaleRequest {
    pub kind: MediaKind,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
    /// Integer multiplier applied to width and height. Only 2, 3, or 4.
    pub scale: u8,
}

/// Accepts a missing scale as 2. Rejects every other value.
///
/// # Errors
///
/// Returns [`Error::InvalidScale`] when `value` is outside 2..=4.
pub fn parse_scale(value: Option<u8>) -> Result<u8, Error> {
    match value.unwrap_or(2) {
        scale @ 2..=4 => Ok(scale),
        _ => Err(Error::InvalidScale),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub request: UpscaleRequest,
    pub status: JobStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// In-memory jobs for one process. Lost on exit until a store is added.
#[derive(Clone, Debug, Default)]
pub struct JobStore {
    jobs: Arc<RwLock<HashMap<JobId, Job>>>,
}

impl JobStore {
    pub fn create(&self, request: UpscaleRequest) -> Result<Job, Error> {
        let job = Job {
            id: JobId::new(),
            request,
            status: JobStatus::Queued,
            error: None,
        };
        self.jobs.write()?.insert(job.id.clone(), job.clone());
        Ok(job)
    }

    pub fn update(
        &self,
        id: &JobId,
        status: JobStatus,
        error: Option<String>,
    ) -> Result<Job, Error> {
        let mut jobs = self.jobs.write()?;
        let job = jobs
            .get_mut(id)
            .ok_or_else(|| Error::NotFound(id.to_string()))?;
        job.status = status;
        job.error = error;
        Ok(job.clone())
    }

    pub fn get(&self, id: &JobId) -> Result<Job, Error> {
        self.jobs
            .read()?
            .get(id)
            .cloned()
            .ok_or_else(|| Error::NotFound(id.to_string()))
    }

    pub fn list(&self) -> Result<Vec<Job>, Error> {
        let jobs = self.jobs.read()?;
        Ok(jobs.values().cloned().collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn image_request() -> UpscaleRequest {
        UpscaleRequest {
            kind: MediaKind::Image,
            input_path: PathBuf::from("in.png"),
            output_path: PathBuf::from("out.png"),
            scale: 2,
        }
    }

    #[test]
    fn create_then_get_returns_the_same_queued_job() {
        // Arrange
        let store = JobStore::default();

        // Act
        let created = store.create(image_request()).expect("create job");
        let fetched = store.get(&created.id).expect("fetch job");

        // Assert
        assert_eq!(fetched, created);
        assert_eq!(fetched.status, JobStatus::Queued);
    }

    #[test]
    fn two_creates_receive_different_ids() {
        // Arrange
        let store = JobStore::default();

        // Act
        let first = store.create(image_request()).expect("first job");
        let second = store.create(image_request()).expect("second job");

        // Assert
        assert_ne!(first.id, second.id);
    }

    #[test]
    fn get_missing_job_returns_not_found() {
        // Arrange
        let store = JobStore::default();
        let id = JobId::new();

        // Act
        let err = store.get(&id).expect_err("missing job");

        // Assert
        assert!(matches!(err, Error::NotFound(_)));
    }

    #[test]
    fn parse_rejects_a_non_uuid() {
        // Act
        let err = JobId::parse("not-a-uuid").expect_err("invalid id");

        // Assert
        assert!(matches!(err, Error::InvalidJobId));
    }

    #[test]
    fn parsed_id_displays_the_same_text() {
        // Arrange
        let raw = "550e8400-e29b-41d4-a716-446655440000";

        // Act
        let id = JobId::parse(raw).expect("valid uuid");

        // Assert
        assert_eq!(id.to_string(), raw);
    }
}
