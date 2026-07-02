use chrono::Utc;
use flight_review_server::db::{create_db, ListFilters, LogRecord, };
use uuid::Uuid;

fn record(id: Uuid, ver_sw: Option<&str>, sys_uuid: Option<&str>) -> LogRecord {
    LogRecord {
        id,
        filename: format!("{id}.ulg"),
        created_at: Utc::now(),
        file_size: 1,
        sys_name: Some("PX4".to_string()),
        ver_hw: Some("PX4_FMU_V5".to_string()),
        ver_sw_release_str: Some("v1.14.0".to_string()),
        flight_duration_s: Some(1.0),
        topic_count: 1,
        lat: None,
        lon: None,
        is_public: true,
        delete_token: "token".to_string(),
        description: Some("search parity fixture".to_string()),
        wind_speed: None,
        rating: None,
        feedback: None,
        video_url: None,
        source: Some("webui".to_string()),
        pilot_name: None,
        vehicle_name: None,
        tags: None,
        location_name: None,
        mission_type: None,
        sys_uuid: sys_uuid.map(str::to_string),
        ver_sw: ver_sw.map(str::to_string),
        vehicle_type: Some("Multirotor".to_string()),
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

#[tokio::test]
async fn listing_search_matches_software_hash_and_system_uuid() {
    let dir = tempfile::tempdir().unwrap();
    let db = create_db(&format!("sqlite://{}", dir.path().join("test.db").display()))
        .await
        .unwrap();

    let by_sw = Uuid::parse_str("00000000-0000-0000-0000-000000000011").unwrap();
    let by_uuid = Uuid::parse_str("00000000-0000-0000-0000-000000000012").unwrap();
    db.insert(&record(by_sw, Some("abcdef1234567890"), None)).await.unwrap();
    db.insert(&record(by_uuid, None, Some("vehicle-uuid-xyz"))).await.unwrap();

    let sw_result = db
        .list(&ListFilters { search: Some("abcdef".to_string()), ..Default::default() })
        .await
        .unwrap();
    assert_eq!(sw_result.total, 1);
    assert_eq!(sw_result.logs[0].id, by_sw);

    let uuid_result = db
        .list(&ListFilters { search: Some("vehicle-uuid-xyz".to_string()), ..Default::default() })
        .await
        .unwrap();
    assert_eq!(uuid_result.total, 1);
    assert_eq!(uuid_result.logs[0].id, by_uuid);
}
