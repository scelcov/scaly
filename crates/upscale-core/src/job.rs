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
}

/// What an agent asks the service to upscale. Paths stay local; the GPU step is not wired yet.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpscaleRequest {
    pub kind: MediaKind,
    pub input_path: PathBuf,
    pub output_path: PathBuf,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Job {
    pub id: JobId,
    pub request: UpscaleRequest,
    pub status: JobStatus,
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
        };
        self.jobs.write()?.insert(job.id.clone(), job.clone());
        Ok(job)
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
