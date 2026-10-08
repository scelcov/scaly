use rmcp::handler::server::wrapper::Parameters;
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use upscale_core::{Job, JobStore, MediaKind, UpscaleRequest};

/// Tools Claude Code calls over stdio. The process that serves them owns the job table.
#[derive(Clone)]
pub struct UpscaleServer {
    store: JobStore,
}

impl UpscaleServer {
    pub fn new(store: JobStore) -> Self {
        Self { store }
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpscaleParams {
    /// Absolute path of the source image or video.
    input_path: String,
    /// Absolute path to write.
    output_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct JobIdParams {
    job_id: String,
}

#[tool_router]
impl UpscaleServer {
    #[tool(description = "Queue an image upscale. Returns the job; the GPU step is not wired yet.")]
    fn upscale_image(
        &self,
        Parameters(params): Parameters<UpscaleParams>,
    ) -> Result<String, String> {
        self.enqueue(MediaKind::Image, params)
    }

    #[tool(description = "Queue a video upscale. Returns the job; the GPU step is not wired yet.")]
    fn upscale_video(
        &self,
        Parameters(params): Parameters<UpscaleParams>,
    ) -> Result<String, String> {
        self.enqueue(MediaKind::Video, params)
    }

    #[tool(description = "Read one queued upscale job by id.")]
    fn job_status(
        &self,
        Parameters(JobIdParams { job_id }): Parameters<JobIdParams>,
    ) -> Result<String, String> {
        let id = upscale_core::JobId::parse(&job_id).map_err(|err| err.to_string())?;
        let job = self.store.get(&id).map_err(|err| err.to_string())?;
        job_json(&job)
    }

    #[tool(description = "List upscale jobs held by this process.")]
    fn list_jobs(&self) -> Result<String, String> {
        let jobs = self.store.list().map_err(|err| err.to_string())?;
        serde_json::to_string(&jobs).map_err(|err| err.to_string())
    }

    fn enqueue(&self, kind: MediaKind, params: UpscaleParams) -> Result<String, String> {
        let job = self
            .store
            .create(UpscaleRequest {
                kind,
                input_path: params.input_path.into(),
                output_path: params.output_path.into(),
            })
            .map_err(|err| err.to_string())?;
        job_json(&job)
    }
}

fn job_json(job: &Job) -> Result<String, String> {
    serde_json::to_string(job).map_err(|err| err.to_string())
}

#[tool_handler(
    name = "upscale",
    version = "0.1.0",
    instructions = "Queue local image and video upscale jobs. The GPU pipeline is not connected yet; jobs stay queued."
)]
impl ServerHandler for UpscaleServer {}
