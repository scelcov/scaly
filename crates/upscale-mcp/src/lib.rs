mod inbox;
mod pipeline;

pub use pipeline::poster_jpeg;

use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, ContentBlock};
use rmcp::{ServerHandler, tool, tool_handler, tool_router};
use schemars::JsonSchema;
use serde::Deserialize;
use upscale_core::{Job, JobStatus, JobStore, MediaKind, UpscaleRequest};

/// Tools an agent calls. The process that serves them owns the job table.
#[derive(Clone)]
pub struct UpscaleServer {
    store: JobStore,
    /// Origin of this process, such as `http://127.0.0.1:8788`, so the chat can load files.
    media_base: Option<String>,
    /// Folder where LibreChat stores pasted and uploaded images.
    inbox: Option<std::path::PathBuf>,
}

impl UpscaleServer {
    pub fn new(store: JobStore) -> Self {
        Self {
            store,
            media_base: None,
            inbox: None,
        }
    }

    #[must_use]
    pub fn with_media_base(mut self, base: impl Into<String>) -> Self {
        self.media_base = Some(base.into());
        self
    }

    #[must_use]
    pub fn with_inbox(mut self, inbox: impl Into<std::path::PathBuf>) -> Self {
        self.inbox = Some(inbox.into());
        self
    }
}

#[derive(Debug, Deserialize, JsonSchema)]
struct UpscaleParams {
    /// Absolute path on this machine. Leave empty when the user pasted or attached the file in chat.
    #[serde(default)]
    input_path: Option<String>,
    /// Absolute path to write. Leave empty and the server picks a file and returns it in the chat.
    #[serde(default)]
    output_path: Option<String>,
    /// Image bytes as base64 or a data URL. Use this when the client can send the file directly.
    #[serde(default)]
    image_base64: Option<String>,
    /// Width and height multiplier. Allowed values are 2, 3, and 4. Defaults to 2.
    #[serde(default)]
    scale: Option<u8>,
}

#[derive(Debug, Deserialize, JsonSchema)]
struct JobIdParams {
    job_id: String,
}

#[tool_router]
impl UpscaleServer {
    #[tool(
        description = "Upscale an image on the GPU and show it in the chat. When the user pasted or attached an image, call this tool with no path. Do not ask for a filesystem path. scale is 2, 3, or 4 and defaults to 2. Pass image_base64 when you have the image bytes or a data URL."
    )]
    async fn upscale_image(
        &self,
        Parameters(params): Parameters<UpscaleParams>,
    ) -> Result<CallToolResult, String> {
        self.upscale(MediaKind::Image, params).await
    }

    #[tool(
        description = "Upscale a video on the GPU. When the user attached a video, call this tool with no path. Do not ask for a filesystem path. scale is 2, 3, or 4 and defaults to 2. Returns a preview frame and a local URL for the mp4."
    )]
    async fn upscale_video(
        &self,
        Parameters(params): Parameters<UpscaleParams>,
    ) -> Result<CallToolResult, String> {
        self.upscale(MediaKind::Video, params).await
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

    #[tracing::instrument(
        skip(self, params),
        fields(
            kind = ?kind,
            input = params.input_path.as_deref().unwrap_or(""),
            pasted = params.image_base64.is_some(),
            scale = params.scale.unwrap_or(2)
        )
    )]
    async fn upscale(
        &self,
        kind: MediaKind,
        params: UpscaleParams,
    ) -> Result<CallToolResult, String> {
        let request = self.prepare(kind, &params).map_err(|err| {
            tracing::error!(error = %err, "failed to read the chat image");
            err.to_string()
        })?;
        let job = self.store.create(request.clone()).map_err(|err| {
            tracing::error!(error = %err, "failed to queue job");
            err.to_string()
        })?;
        self.store
            .update(&job.id, JobStatus::Running, None)
            .map_err(|err| err.to_string())?;

        let ran = tokio::task::spawn_blocking(move || pipeline::upscale(&request))
            .await
            .map_err(|err| err.to_string())?;
        if let Err(err) = ran {
            tracing::error!(error = %err, "upscale failed");
            self.store
                .update(&job.id, JobStatus::Failed, Some(err.to_string()))
                .ok();
            return Err(err.to_string());
        }

        let job = self
            .store
            .update(&job.id, JobStatus::Completed, None)
            .map_err(|err| err.to_string())?;
        tracing::info!(job_id = %job.id, "upscale finished");
        Ok(self.tool_result(&job))
    }

    fn prepare(
        &self,
        kind: MediaKind,
        params: &UpscaleParams,
    ) -> Result<UpscaleRequest, upscale_core::Error> {
        let input_path = if let Some(raw) = nonempty(&params.image_base64) {
            let (bytes, ext) = inbox::decode_image(raw)?;
            inbox::write_image(&bytes, ext)?
        } else if let Some(path) = nonempty(&params.input_path) {
            std::path::PathBuf::from(path)
        } else if kind == MediaKind::Image {
            let inbox = self
                .inbox
                .as_deref()
                .ok_or(upscale_core::Error::NoChatImage)?;
            inbox::newest_image(inbox)?
        } else {
            let inbox = self
                .inbox
                .as_deref()
                .ok_or(upscale_core::Error::NoChatVideo)?;
            inbox::newest_video(inbox)?
        };
        let scale = upscale_core::parse_scale(params.scale)?;
        let output_path = if let Some(path) = nonempty(&params.output_path) {
            std::path::PathBuf::from(path)
        } else {
            let ext = match kind {
                MediaKind::Image => "png",
                MediaKind::Video => "mp4",
            };
            inbox::default_output(ext)
        };
        Ok(UpscaleRequest {
            kind,
            input_path,
            output_path,
            scale,
        })
    }

    fn tool_result(&self, job: &Job) -> CallToolResult {
        let mut text = format!(
            "upscaled {}x\njob_id: {}\noutput: {}",
            job.request.scale,
            job.id,
            job.request.output_path.display()
        );
        if let Some(base) = &self.media_base {
            text.push_str(&format!("\nview: {base}/media/{}", job.id));
        }
        let mut blocks = vec![ContentBlock::text(text)];
        if let Ok((mime, encoded)) = preview_bytes(job) {
            blocks.push(ContentBlock::image(encoded, mime));
        }
        CallToolResult::success(blocks)
    }
}

fn nonempty(value: &Option<String>) -> Option<&str> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|text| !text.is_empty())
}

fn image_mime(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        _ => "image/png",
    }
}

fn preview_bytes(job: &Job) -> Result<(&'static str, String), upscale_core::Error> {
    use base64::Engine;

    let bytes = match job.request.kind {
        MediaKind::Image => std::fs::read(&job.request.output_path)
            .map_err(|err| upscale_core::Error::Upscale(err.to_string()))?,
        MediaKind::Video => pipeline::poster_jpeg(&job.request.output_path)?,
    };
    let mime = match job.request.kind {
        MediaKind::Image => image_mime(&job.request.output_path),
        MediaKind::Video => "image/jpeg",
    };
    Ok((
        mime,
        base64::engine::general_purpose::STANDARD.encode(bytes),
    ))
}

fn job_json(job: &Job) -> Result<String, String> {
    serde_json::to_string(job).map_err(|err| err.to_string())
}

#[tool_handler(
    name = "upscale",
    version = "0.1.0",
    instructions = "Upscale images and videos on the GPU. Scale is 2, 3, or 4 and defaults to 2. When the user pastes or attaches a file, call upscale_image or upscale_video with no path. Do not ask for a file path. Image results include the picture. Video results include a preview frame and a view URL."
)]
impl ServerHandler for UpscaleServer {}

#[cfg(test)]
mod tests {
    use std::process::Command;

    use base64::Engine;

    use super::*;

    fn server() -> UpscaleServer {
        UpscaleServer::new(JobStore::default())
    }

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("upscale-{name}-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("scratch dir");
        dir
    }

    #[tokio::test]
    async fn upscale_image_writes_a_2x_png_and_can_be_read_back() {
        // Arrange
        let dir = scratch("image");
        let input = dir.join("in.png");
        let output = dir.join("out.png");
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=red:s=16x16:d=1",
                "-frames:v",
                "1",
                "-update",
                "1",
            ])
            .arg(&input)
            .status()
            .expect("write source png");
        assert!(status.success());
        let server = server();

        // Act
        let created = server
            .upscale_image(Parameters(UpscaleParams {
                input_path: Some(input.display().to_string()),
                output_path: Some(output.display().to_string()),
                image_base64: None,
                scale: None,
            }))
            .await
            .expect("upscale image");
        let listed = server.list_jobs().expect("list");

        // Assert
        assert!(
            created
                .content
                .iter()
                .any(|block| block.as_image().is_some())
        );
        let jobs: Vec<serde_json::Value> = serde_json::from_str(&listed).expect("list json");
        assert_eq!(jobs[0]["status"], "completed");
        let id = jobs[0]["id"].as_str().expect("id");
        let fetched = server
            .job_status(Parameters(JobIdParams {
                job_id: id.to_owned(),
            }))
            .expect("status");
        let fetched_json: serde_json::Value = serde_json::from_str(&fetched).expect("status json");
        assert_eq!(fetched_json["request"]["kind"], "image");
        let probe = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=width,height",
                "-of",
                "csv=p=0",
            ])
            .arg(&output)
            .output()
            .expect("ffprobe");
        assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "32,32");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn upscale_image_rejects_a_missing_file() {
        // Arrange
        let server = server();

        // Act
        let err = server
            .upscale_image(Parameters(UpscaleParams {
                input_path: Some("/tmp/upscale-missing-source.png".to_owned()),
                output_path: Some("/tmp/upscale-missing-out.png".to_owned()),
                image_base64: None,
                scale: None,
            }))
            .await
            .expect_err("missing file");

        // Assert
        assert!(err.contains("not found"));
    }

    #[test]
    fn job_status_rejects_a_non_uuid() {
        // Arrange
        let server = server();

        // Act
        let err = server
            .job_status(Parameters(JobIdParams {
                job_id: "nope".to_owned(),
            }))
            .expect_err("bad id");

        // Assert
        assert!(err.contains("uuid"));
    }

    #[tokio::test]
    async fn list_jobs_includes_a_completed_video() {
        // Arrange
        let dir = scratch("video");
        let input = dir.join("in.mp4");
        let output = dir.join("out.mp4");
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "testsrc=size=32x32:rate=8:duration=0.5",
                "-f",
                "lavfi",
                "-i",
                "sine=frequency=440:duration=0.5",
                "-shortest",
                "-c:v",
                "libx264",
                "-pix_fmt",
                "yuv420p",
                "-c:a",
                "aac",
            ])
            .arg(&input)
            .status()
            .expect("write source video");
        assert!(status.success());
        let server = server();
        server
            .upscale_video(Parameters(UpscaleParams {
                input_path: Some(input.display().to_string()),
                output_path: Some(output.display().to_string()),
                image_base64: None,
                scale: None,
            }))
            .await
            .expect("upscale video");

        // Act
        let listed = server.list_jobs().expect("list");

        // Assert
        let jobs: Vec<serde_json::Value> = serde_json::from_str(&listed).expect("list json");
        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0]["request"]["kind"], "video");
        assert_eq!(jobs[0]["status"], "completed");
        assert!(output.is_file());
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn newest_image_picks_the_later_paste() {
        // Arrange
        let dir = scratch("inbox");
        let older = dir.join("older.png");
        let newer = dir.join("newer.png");
        std::fs::write(&older, b"not-a-real-png").expect("older");
        std::thread::sleep(std::time::Duration::from_millis(20));
        std::fs::write(&newer, b"not-a-real-png").expect("newer");

        // Act
        let picked = inbox::newest_image(&dir).expect("newest");

        // Assert
        assert_eq!(picked, newer);
        let _ = std::fs::remove_dir_all(dir);
    }

    #[test]
    fn newest_video_reads_the_uploads_folder() {
        // Arrange
        let root = scratch("media");
        let images = root.join("images");
        let uploads = root.join("uploads");
        std::fs::create_dir_all(&images).expect("images");
        std::fs::create_dir_all(&uploads).expect("uploads");
        std::fs::write(uploads.join("clip.mp4"), b"video").expect("clip");

        // Act
        let picked = inbox::newest_video(&images).expect("video");

        // Assert
        assert_eq!(picked, uploads.join("clip.mp4"));
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn upscale_image_honors_scale_three() {
        // Arrange
        let dir = scratch("scale3");
        let input = dir.join("in.png");
        let output = dir.join("out.png");
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=blue:s=16x16:d=1",
                "-frames:v",
                "1",
                "-update",
                "1",
            ])
            .arg(&input)
            .status()
            .expect("source");
        assert!(status.success());
        let server = server();

        // Act
        server
            .upscale_image(Parameters(UpscaleParams {
                input_path: Some(input.display().to_string()),
                output_path: Some(output.display().to_string()),
                image_base64: None,
                scale: Some(3),
            }))
            .await
            .expect("scale 3");

        // Assert
        let probe = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-select_streams",
                "v:0",
                "-show_entries",
                "stream=width,height",
                "-of",
                "csv=p=0",
            ])
            .arg(&output)
            .output()
            .expect("ffprobe");
        assert_eq!(String::from_utf8_lossy(&probe.stdout).trim(), "48,48");
        let _ = std::fs::remove_dir_all(dir);
    }

    #[tokio::test]
    async fn upscale_image_rejects_a_scale_outside_2_to_4() {
        // Arrange
        let server = server();

        // Act
        let err = server
            .upscale_image(Parameters(UpscaleParams {
                input_path: Some("/tmp/upscale-scale.png".to_owned()),
                output_path: None,
                image_base64: None,
                scale: Some(8),
            }))
            .await
            .expect_err("bad scale");

        // Assert
        assert!(err.contains("scale"));
    }

    #[tokio::test]
    async fn upscale_image_reads_a_pasted_base64_png() {
        // Arrange
        let dir = scratch("b64");
        let input = dir.join("in.png");
        let status = std::process::Command::new("ffmpeg")
            .args([
                "-y",
                "-hide_banner",
                "-loglevel",
                "error",
                "-f",
                "lavfi",
                "-i",
                "color=c=green:s=8x8:d=1",
                "-frames:v",
                "1",
                "-update",
                "1",
            ])
            .arg(&input)
            .status()
            .expect("source");
        assert!(status.success());
        let bytes = std::fs::read(&input).expect("read");
        let encoded = base64::Engine::encode(&base64::engine::general_purpose::STANDARD, &bytes);
        let server = server();

        // Act
        let created = server
            .upscale_image(Parameters(UpscaleParams {
                input_path: None,
                output_path: None,
                image_base64: Some(encoded),
                scale: None,
            }))
            .await
            .expect("upscale pasted image");

        // Assert
        let text = created.content[0].as_text().expect("text").text.clone();
        assert!(text.contains("upscaled 2x"));
        let image = created.content[1].as_image().expect("image");
        assert!(
            base64::engine::general_purpose::STANDARD
                .decode(image.data.as_bytes())
                .is_ok()
        );
        let _ = std::fs::remove_dir_all(dir);
    }
}
