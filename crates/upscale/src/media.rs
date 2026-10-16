use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::Response;
use tokio_util::io::ReaderStream;
use upscale_core::{JobStatus, JobStore, MediaKind};

pub async fn file(
    State(store): State<JobStore>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let job = completed(&store, &id)?;
    stream_file(&job.request.output_path).await
}

pub async fn poster(
    State(store): State<JobStore>,
    Path(id): Path<String>,
) -> Result<Response, StatusCode> {
    let job = completed(&store, &id)?;
    if job.request.kind != MediaKind::Video {
        return Err(StatusCode::NOT_FOUND);
    }
    let path = job.request.output_path.clone();
    let bytes = tokio::task::spawn_blocking(move || upscale_mcp::poster_jpeg(&path))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Response::builder()
        .header(header::CONTENT_TYPE, "image/jpeg")
        .header(header::CONTENT_LENGTH, bytes.len())
        .body(Body::from(bytes))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn completed(store: &JobStore, id: &str) -> Result<upscale_core::Job, StatusCode> {
    let id = upscale_core::JobId::parse(id).map_err(|_| StatusCode::NOT_FOUND)?;
    let job = store.get(&id).map_err(|_| StatusCode::NOT_FOUND)?;
    if job.status != JobStatus::Completed {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(job)
}

async fn stream_file(path: &std::path::Path) -> Result<Response, StatusCode> {
    let file = tokio::fs::File::open(path)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    let len = file
        .metadata()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .len();
    let mime = mime_of(path);
    let body = Body::from_stream(ReaderStream::new(file));
    Response::builder()
        .header(header::CONTENT_TYPE, mime)
        .header(header::CONTENT_LENGTH, len)
        .header(header::ACCEPT_RANGES, "none")
        .body(body)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn mime_of(path: &std::path::Path) -> &'static str {
    match path
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .as_deref()
    {
        Some("png") => "image/png",
        Some("jpg" | "jpeg") => "image/jpeg",
        Some("webp") => "image/webp",
        Some("gif") => "image/gif",
        Some("mp4") => "video/mp4",
        Some("webm") => "video/webm",
        Some("mov") => "video/quicktime",
        Some("mkv") => "video/x-matroska",
        _ => "application/octet-stream",
    }
}
