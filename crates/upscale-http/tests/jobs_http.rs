use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use tower::ServiceExt;
use upscale_core::JobStore;
use upscale_http::router;

#[tokio::test]
async fn health_returns_ok() {
    // Arrange
    let app = router(JobStore::default());

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .uri("/health")
                .body(Body::empty())
                .expect("health request"),
        )
        .await
        .expect("health response");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("health body")
        .to_bytes();
    assert_eq!(&body[..], b"ok");
}

#[tokio::test]
async fn create_then_get_returns_queued_job() {
    // Arrange
    let store = JobStore::default();
    let payload = r#"{"kind":"image","input_path":"in.png","output_path":"out.png"}"#;

    // Act
    let created = router(store.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/jobs")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .expect("create request"),
        )
        .await
        .expect("create response");

    // Assert
    assert_eq!(created.status(), StatusCode::ACCEPTED);
    let created_body = created
        .into_body()
        .collect()
        .await
        .expect("create body")
        .to_bytes();
    let created_json: serde_json::Value =
        serde_json::from_slice(&created_body).expect("create json");
    assert_eq!(created_json["status"], "queued");
    assert_eq!(created_json["request"]["kind"], "image");
    let id = created_json["id"].as_str().expect("job id");

    let fetched = router(store)
        .oneshot(
            Request::builder()
                .uri(format!("/api/jobs/{id}"))
                .body(Body::empty())
                .expect("get request"),
        )
        .await
        .expect("get response");
    assert_eq!(fetched.status(), StatusCode::OK);
    let fetched_body = fetched
        .into_body()
        .collect()
        .await
        .expect("get body")
        .to_bytes();
    let fetched_json: serde_json::Value = serde_json::from_slice(&fetched_body).expect("get json");
    assert_eq!(fetched_json["id"], id);
}

#[tokio::test]
async fn unknown_job_id_returns_not_found() {
    // Arrange
    let app = router(JobStore::default());

    // Act
    let response = app
        .oneshot(
            Request::builder()
                .uri("/api/jobs/not-a-uuid")
                .body(Body::empty())
                .expect("get request"),
        )
        .await
        .expect("get response");

    // Assert
    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn list_returns_created_jobs() {
    // Arrange
    let store = JobStore::default();
    let payload = r#"{"kind":"video","input_path":"in.mkv","output_path":"out.mkv"}"#;
    router(store.clone())
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/jobs")
                .header("content-type", "application/json")
                .body(Body::from(payload))
                .expect("create request"),
        )
        .await
        .expect("create response");

    // Act
    let response = router(store)
        .oneshot(
            Request::builder()
                .uri("/api/jobs")
                .body(Body::empty())
                .expect("list request"),
        )
        .await
        .expect("list response");

    // Assert
    assert_eq!(response.status(), StatusCode::OK);
    let body = response
        .into_body()
        .collect()
        .await
        .expect("list body")
        .to_bytes();
    let jobs: Vec<serde_json::Value> = serde_json::from_slice(&body).expect("list json");
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0]["request"]["kind"], "video");
}
