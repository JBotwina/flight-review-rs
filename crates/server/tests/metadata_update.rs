use std::net::TcpListener;
use std::sync::Arc;

use chrono::Utc;
use flight_review_server::db::{create_db, LogRecord};
use flight_review_server::storage::FileStorage;
use serde_json::json;
use uuid::Uuid;

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn record(id: Uuid) -> LogRecord {
    LogRecord {
        id,
        filename: format!("{id}.ulg"),
        created_at: Utc::now(),
        file_size: 1,
        sys_name: Some("PX4".into()),
        ver_hw: Some("PX4_FMU_V5".into()),
        ver_sw_release_str: Some("v1.14.0".into()),
        flight_duration_s: Some(1.0),
        topic_count: 1,
        lat: None,
        lon: None,
        is_public: true,
        delete_token: "correct-token".into(),
        description: Some("old description".into()),
        wind_speed: Some("calm".into()),
        rating: Some(3),
        feedback: Some("old feedback".into()),
        video_url: None,
        source: Some("web".into()),
        pilot_name: Some("unchanged pilot".into()),
        vehicle_name: Some("old vehicle".into()),
        tags: Some("old".into()),
        location_name: Some("old location".into()),
        mission_type: Some("test".into()),
        sys_uuid: Some("uuid".into()),
        ver_sw: Some("abcdef".into()),
        vehicle_type: Some("Multirotor".into()),
        localization_sources: None,
        vibration_status: None,
        battery_min_voltage: None,
        gps_max_eph: None,
        max_speed_m_s: None,
        total_distance_m: None,
        error_count: Some(0),
        warning_count: Some(0),
        analysis_version: Some(1),
        diagnostic_flags: None,
    }
}

async fn start_server_with_record(id: Uuid) -> (String, tokio::task::JoinHandle<()>) {
    let port = free_port();
    let base_url = format!("http://127.0.0.1:{port}");

    let tmp_dir = tempfile::tempdir().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    let storage_path = tmp_dir.path().join("files");
    std::fs::create_dir_all(&storage_path).unwrap();

    let db = create_db(&format!("sqlite://{}", db_path.display()))
        .await
        .expect("failed to create test db");
    db.insert(&record(id)).await.unwrap();

    let storage = Arc::new(
        FileStorage::from_url(&format!("file://{}", storage_path.display()))
            .expect("failed to create test storage"),
    );

    let state = Arc::new(flight_review_server::AppState {
        db,
        storage,
        v1_ulg_prefix: None,
        mapbox_token: None,
        http_client: reqwest::Client::new(),
    });

    let app = flight_review_server::build_router(state);
    let listener = tokio::net::TcpListener::bind(format!("127.0.0.1:{port}"))
        .await
        .unwrap();
    let handle = tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    for _ in 0..50 {
        if reqwest::get(format!("{base_url}/health")).await.is_ok() {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
    }

    std::mem::forget(tmp_dir);
    (base_url, handle)
}

#[tokio::test]
async fn patch_log_metadata_updates_mutable_fields_with_delete_token() {
    let id = Uuid::parse_str("00000000-0000-0000-0000-000000000801").unwrap();
    let (base_url, _handle) = start_server_with_record(id).await;
    let client = reqwest::Client::new();

    let resp = client
        .patch(format!("{base_url}/api/logs/{id}?token=wrong-token"))
        .json(&json!({"description": "should not apply"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::FORBIDDEN);

    let resp = client
        .patch(format!("{base_url}/api/logs/{id}?token=correct-token"))
        .json(&json!({
            "description": "new description",
            "wind_speed": "breeze",
            "rating": 5,
            "feedback": "new feedback",
            "video_url": "https://example.com/flight.mp4",
            "is_public": false,
            "vehicle_name": "new vehicle",
            "source": "API",
            "tags": "alpha,beta",
            "location_name": "new location",
            "mission_type": "survey"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);

    let updated: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(updated["description"], "new description");
    assert_eq!(updated["wind_speed"], "breeze");
    assert_eq!(updated["rating"], 5);
    assert_eq!(updated["feedback"], "new feedback");
    assert_eq!(updated["video_url"], "https://example.com/flight.mp4");
    assert_eq!(updated["is_public"], false);
    assert_eq!(updated["vehicle_name"], "new vehicle");
    assert_eq!(updated["source"], "API");
    assert_eq!(updated["tags"], "alpha,beta");
    assert_eq!(updated["location_name"], "new location");
    assert_eq!(updated["mission_type"], "survey");

    // Non-mutable/generated fields are preserved.
    assert_eq!(updated["filename"], format!("{id}.ulg"));
    assert_eq!(updated["pilot_name"], "unchanged pilot");
    assert!(updated.get("delete_token").is_none());
}
