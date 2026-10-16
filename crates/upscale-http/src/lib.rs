use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;
use upscale_core::{Error, Job, JobStore, MediaKind, UpscaleRequest};

pub fn router(store: JobStore) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/jobs", get(list_jobs).post(create_job))
        .route("/api/jobs/{id}", get(get_job))
        .with_state(store)
}

async fn health() -> &'static str {
    "ok"
}

#[derive(Deserialize)]
struct CreateJobBody {
    kind: MediaKind,
    input_path: String,
    output_path: String,
    #[serde(default)]
    scale: Option<u8>,
}

async fn create_job(
    State(store): State<JobStore>,
    Json(body): Json<CreateJobBody>,
) -> Result<(StatusCode, Json<Job>), ApiError> {
    let job = store.create(UpscaleRequest {
        kind: body.kind,
        input_path: body.input_path.into(),
        output_path: body.output_path.into(),
        scale: upscale_core::parse_scale(body.scale)?,
    })?;
    Ok((StatusCode::ACCEPTED, Json(job)))
}

async fn list_jobs(State(store): State<JobStore>) -> Result<Json<Vec<Job>>, ApiError> {
    Ok(Json(store.list()?))
}

async fn get_job(
    State(store): State<JobStore>,
    Path(id): Path<String>,
) -> Result<Json<Job>, ApiError> {
    let id = upscale_core::JobId::parse(&id)?;
    Ok(Json(store.get(&id)?))
}

struct ApiError(Error);

impl From<Error> for ApiError {
    fn from(err: Error) -> Self {
        Self(err)
    }
}

impl axum::response::IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let status = match self.0 {
            Error::InvalidJobId | Error::NotFound(_) => StatusCode::NOT_FOUND,
            Error::InputNotFound
            | Error::SamePath
            | Error::NoChatImage
            | Error::NoChatVideo
            | Error::InvalidScale => StatusCode::BAD_REQUEST,
            Error::LockPoisoned | Error::Upscale(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, self.0.to_string()).into_response()
    }
}
