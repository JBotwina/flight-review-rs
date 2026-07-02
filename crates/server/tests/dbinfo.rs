use std::sync::Arc;

use axum::extract::State;
use chrono::Utc;
use flight_review_server::db::{create_db, LogRecord};
use flight_review_server::storage::FileStorage;
use uuid::Uuid;

fn record(id: Uuid, is_public: bool, source: &str) -> LogRecord {
    LogRecord { id, filename: format!("{id}.ulg"), created_at: Utc::now(), file_size: 1,
        sys_name: Some("PX4".into()), ver_hw: Some("PX4_FMU_V5".into()), ver_sw_release_str: Some("v1.14.0".into()),
        flight_duration_s: Some(1.0), topic_count: 1, lat: None, lon: None, is_public,
        delete_token: "token".into(), description: Some("dbinfo fixture".into()), wind_speed: None,
        rating: Some(5), feedback: Some("ok".into()), video_url: None, source: Some(source.into()),
        pilot_name: None, vehicle_name: Some("vehicle".into()), tags: None, location_name: None,
        mission_type: Some("test".into()), sys_uuid: Some("uuid".into()), ver_sw: Some("abcdef".into()),
        vehicle_type: Some("Multirotor".into()), localization_sources: None, vibration_status: None,
        battery_min_voltage: None, gps_max_eph: None, max_speed_m_s: None, total_distance_m: None,
        error_count: Some(0), warning_count: Some(0), analysis_version: Some(1), diagnostic_flags: None }
}

#[tokio::test]
async fn dbinfo_exports_public_non_ci_logs() {
    let dir = tempfile::tempdir().unwrap();
    let db = create_db(&format!("sqlite://{}", dir.path().join("test.db").display())).await.unwrap();
    let storage = Arc::new(FileStorage::from_url(&format!("file://{}", dir.path().join("files").display())).unwrap());
    let public_id = Uuid::parse_str("00000000-0000-0000-0000-000000000101").unwrap();
    db.insert(&record(public_id, true, "webui")).await.unwrap();
    db.insert(&record(Uuid::parse_str("00000000-0000-0000-0000-000000000102").unwrap(), false, "webui")).await.unwrap();

    let state = Arc::new(flight_review_server::AppState {
        db, storage, v1_ulg_prefix: None, mapbox_token: None, http_client: reqwest::Client::new(),
    });
    let response = match flight_review_server::api::logs::dbinfo(State(state)).await {
        Ok(response) => response,
        Err(_) => panic!("dbinfo should succeed"),
    };
    let logs = serde_json::to_value(response.0).unwrap();
    assert_eq!(logs.as_array().unwrap().len(), 1);
    assert_eq!(logs[0]["id"], public_id.to_string());
    assert_eq!(logs[0]["rating"], 5);
    assert_eq!(logs[0]["vehicle_name"], "vehicle");
}
