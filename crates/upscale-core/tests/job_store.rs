use std::path::PathBuf;

use upscale_core::{Error, JobStatus, JobStore, MediaKind, UpscaleRequest};

fn video_request() -> UpscaleRequest {
    UpscaleRequest {
        kind: MediaKind::Video,
        input_path: PathBuf::from("in.mkv"),
        output_path: PathBuf::from("out.mkv"),
    }
}

#[test]
fn list_contains_each_created_job() {
    // Arrange
    let store = JobStore::default();

    // Act
    let image = store
        .create(UpscaleRequest {
            kind: MediaKind::Image,
            input_path: PathBuf::from("still.png"),
            output_path: PathBuf::from("still-out.png"),
        })
        .expect("image job");
    let video = store.create(video_request()).expect("video job");
    let listed = store.list().expect("list jobs");

    // Assert
    assert_eq!(listed.len(), 2);
    assert!(listed.iter().any(|job| job.id == image.id));
    assert!(listed.iter().any(|job| job.id == video.id));
    assert!(listed.iter().all(|job| job.status == JobStatus::Queued));
}

#[test]
fn missing_job_error_names_the_id() {
    // Arrange
    let store = JobStore::default();
    let id = upscale_core::JobId::parse("550e8400-e29b-41d4-a716-446655440000").expect("uuid");

    // Act
    let err = store.get(&id).expect_err("missing");

    // Assert
    match err {
        Error::NotFound(text) => assert_eq!(text, id.to_string()),
        other => panic!("expected NotFound, got {other}"),
    }
}
