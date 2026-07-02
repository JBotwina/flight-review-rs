use axum::body::Body;
use axum::http::{Request, StatusCode};
use bytes::Bytes;
use flight_review::analysis::{FlightAnalysis, ParamDiff};
use flight_review::metadata::{FlightMetadata, ParamValue};
use flight_review_server::storage::FileStorage;
use std::sync::Arc;
use tower::ServiceExt;
use uuid::Uuid;

async fn test_app() -> (axum::Router, Uuid) {
    let tmp_dir = tempfile::tempdir().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    let storage_path = tmp_dir.path().join("files");
    std::fs::create_dir_all(&storage_path).unwrap();

    let db_url = format!("sqlite://{}", db_path.display());
    let storage_url = format!("file://{}", storage_path.display());

    let db = flight_review_server::db::create_db(&db_url)
        .await
        .expect("failed to create test db");
    let storage =
        Arc::new(FileStorage::from_url(&storage_url).expect("failed to create test storage"));

    let log_id = Uuid::new_v4();
    let mut metadata = FlightMetadata::default();
    metadata
        .parameters
        .insert("MAV_TYPE".to_string(), ParamValue::Int32(2));
    metadata
        .parameters
        .insert("SYS_AUTOSTART".to_string(), ParamValue::Int32(4001));
    metadata
        .parameters
        .insert("FLOAT_PARAM".to_string(), ParamValue::Float(1.5));
    metadata
        .parameters
        .insert("RC1_TRIM".to_string(), ParamValue::Int32(1500));
    metadata
        .default_parameters
        .insert("SYS_AUTOSTART".to_string(), ParamValue::Int32(4000));
    metadata
        .default_parameters
        .insert("FLOAT_PARAM".to_string(), ParamValue::Float(1.5));
    metadata
        .default_parameters
        .insert("RC1_TRIM".to_string(), ParamValue::Int32(0));
    metadata.analysis = Some(FlightAnalysis {
        non_default_params: vec![ParamDiff {
            name: "SYS_AUTOSTART".to_string(),
            value: 4001.0,
            default: 4000.0,
        }],
        ..Default::default()
    });

    let metadata_json = serde_json::to_vec(&metadata).unwrap();
    storage
        .put_file(log_id, "metadata.json", Bytes::from(metadata_json))
        .await
        .unwrap();

    let state = Arc::new(flight_review_server::AppState {
        db,
        storage,
        v1_ulg_prefix: None,
        mapbox_token: None,
        http_client: reqwest::Client::new(),
    });

    // Keep tmp_dir alive for the duration of the test process.
    std::mem::forget(tmp_dir);

    (flight_review_server::build_router(state), log_id)
}

#[tokio::test]
async fn parameter_downloads_use_legacy_px4_param_format() {
    let (app, log_id) = test_app().await;

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .uri(format!("/api/logs/{log_id}/download/parameters"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-disposition").unwrap(),
        "attachment; filename=vehicle.params"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(
        body,
        "1\t1\tFLOAT_PARAM\t1.5\t9\n1\t1\tMAV_TYPE\t2\t6\n1\t1\tRC1_TRIM\t1500\t6\n1\t1\tSYS_AUTOSTART\t4001\t6\n"
    );

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!(
                    "/api/logs/{log_id}/download/non-default-parameters"
                ))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(
        response.headers().get("content-disposition").unwrap(),
        "attachment; filename=non-default.params"
    );
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body = String::from_utf8(body.to_vec()).unwrap();
    assert_eq!(body, "1\t1\tSYS_AUTOSTART\t4001\t6\n");
}

#[tokio::test]
async fn kml_download_reports_explicitly_unimplemented_for_known_logs() {
    let (app, log_id) = test_app().await;

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/logs/{log_id}/download/kml"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_IMPLEMENTED);
    let body = axum::body::to_bytes(response.into_body(), usize::MAX)
        .await
        .unwrap();
    let body: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(body["error"].as_str().unwrap().contains("KML"));
}

#[tokio::test]
async fn download_routes_return_not_found_when_metadata_is_missing() {
    let (app, _) = test_app().await;
    let missing_id = Uuid::new_v4();

    let response = app
        .oneshot(
            Request::builder()
                .uri(format!("/api/logs/{missing_id}/download/parameters"))
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}
