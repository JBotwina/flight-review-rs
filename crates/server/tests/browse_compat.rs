use serde_json::Value;
use std::{net::TcpListener, sync::Arc};
use uuid::Uuid;

fn free_port() -> u16 {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    listener.local_addr().unwrap().port()
}

fn fixture_log(
    filename: &str,
    created_at: &str,
    vehicle_type: &str,
    ver_hw: &str,
) -> flight_review_server::db::LogRecord {
    flight_review_server::db::LogRecord {
        id: Uuid::new_v4(),
        filename: filename.to_string(),
        created_at: chrono::DateTime::parse_from_rfc3339(created_at)
            .unwrap()
            .with_timezone(&chrono::Utc),
        file_size: 1024,
        sys_name: Some("PX4".to_string()),
        ver_hw: Some(ver_hw.to_string()),
        ver_sw_release_str: Some("v1.15.0".to_string()),
        flight_duration_s: Some(42.0),
        topic_count: 12,
        lat: None,
        lon: None,
        is_public: true,
        delete_token: "delete-token".to_string(),
        description: Some(format!("{vehicle_type} compatibility log")),
        wind_speed: None,
        rating: None,
        feedback: None,
        video_url: None,
        source: Some("web".to_string()),
        pilot_name: None,
        vehicle_name: None,
        tags: None,
        location_name: None,
        mission_type: None,
        sys_uuid: None,
        ver_sw: None,
        vehicle_type: Some(vehicle_type.to_string()),
        localization_sources: None,
        vibration_status: None,
        battery_min_voltage: None,
        gps_max_eph: None,
        max_speed_m_s: None,
        total_distance_m: None,
        error_count: Some(if vehicle_type == "multirotor" { 1 } else { 0 }),
        warning_count: Some(if vehicle_type == "multirotor" { 3 } else { 1 }),
        analysis_version: Some(1),
        diagnostic_flags: None,
    }
}

async fn start_server_with_logs() -> (String, tokio::task::JoinHandle<()>) {
    let port = free_port();
    let base_url = format!("http://127.0.0.1:{port}");

    let tmp_dir = tempfile::tempdir().unwrap();
    let db_path = tmp_dir.path().join("test.db");
    let storage_path = tmp_dir.path().join("files");
    std::fs::create_dir_all(&storage_path).unwrap();

    let db_url = format!("sqlite://{}", db_path.display());
    let storage_url = format!("file://{}", storage_path.display());

    let db = flight_review_server::db::create_db(&db_url)
        .await
        .expect("failed to create test db");
    db.insert(&fixture_log(
        "alpha-multirotor.ulg",
        "2024-01-01T00:00:00Z",
        "multirotor",
        "Pixhawk 6C",
    ))
    .await
    .unwrap();
    db.insert(&fixture_log(
        "beta-fixedwing.ulg",
        "2024-01-02T00:00:00Z",
        "fixedwing",
        "Cube Orange",
    ))
    .await
    .unwrap();

    let storage = Arc::new(
        flight_review_server::storage::FileStorage::from_url(&storage_url)
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
async fn browse_data_retrieval_supports_datatables_pagination_search_and_legacy_fields() {
    let (base_url, _handle) = start_server_with_logs().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!(
            "{base_url}/api/browse_data_retrieval?draw=7&start=0&length=1&search[value]=multi&order[0][column]=0&order[0][dir]=desc"
        ))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    let body: Value = resp.json().await.unwrap();

    assert_eq!(body["draw"], 7);
    assert_eq!(body["recordsTotal"], 2);
    assert_eq!(body["recordsFiltered"], 1);
    assert_eq!(body["total"], 1);
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
    assert_eq!(body["logs"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"][0]["vehicle_type"], "multirotor");
    assert_eq!(body["data"][0]["ver_hw"], "Pixhawk 6C");
    assert_eq!(body["data"][0]["ver_sw_release_str"], "v1.15.0");
    assert_eq!(body["data"][0]["error_count"], 1);
    assert_eq!(body["data"][0]["warning_count"], 3);
    assert_eq!(body["logs"][0]["id"], body["data"][0]["id"]);
}
