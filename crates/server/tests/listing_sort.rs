use chrono::Utc;
use flight_review_server::db::{create_db, ListFilters, LogRecord};
use uuid::Uuid;

fn record(id: Uuid, vehicle_type: &str, ver_hw: &str, ver_sw: &str, errors: i32, warnings: i32) -> LogRecord {
    LogRecord { id, filename: format!("{id}.ulg"), created_at: Utc::now(), file_size: 1,
        sys_name: Some("PX4".into()), ver_hw: Some(ver_hw.into()), ver_sw_release_str: Some("v1.14.0".into()),
        flight_duration_s: Some(1.0), topic_count: 1, lat: None, lon: None, is_public: true,
        delete_token: "token".into(), description: None, wind_speed: None, rating: None, feedback: None,
        video_url: None, source: Some("webui".into()), pilot_name: None, vehicle_name: None, tags: None,
        location_name: None, mission_type: None, sys_uuid: None, ver_sw: Some(ver_sw.into()),
        vehicle_type: Some(vehicle_type.into()), localization_sources: None, vibration_status: None,
        battery_min_voltage: None, gps_max_eph: None, max_speed_m_s: None, total_distance_m: None,
        error_count: Some(errors), warning_count: Some(warnings), analysis_version: Some(1), diagnostic_flags: None }
}

#[tokio::test]
async fn listing_sort_supports_legacy_browse_metadata_fields() {
    let dir = tempfile::tempdir().unwrap();
    let db = create_db(&format!("sqlite://{}", dir.path().join("test.db").display())).await.unwrap();
    let a = Uuid::parse_str("00000000-0000-0000-0000-0000000000a1").unwrap();
    let b = Uuid::parse_str("00000000-0000-0000-0000-0000000000b1").unwrap();
    db.insert(&record(a, "Rover", "PX4_FMU_V2", "bbbb", 2, 1)).await.unwrap();
    db.insert(&record(b, "Multirotor", "PX4_FMU_V5", "aaaa", 0, 4)).await.unwrap();

    let by_type = db.list(&ListFilters { sort: Some("vehicle_type:asc".into()), ..Default::default() }).await.unwrap();
    assert_eq!(by_type.logs[0].id, b);
    let by_hw = db.list(&ListFilters { sort: Some("ver_hw:desc".into()), ..Default::default() }).await.unwrap();
    assert_eq!(by_hw.logs[0].id, b);
    let by_sw = db.list(&ListFilters { sort: Some("ver_sw:asc".into()), ..Default::default() }).await.unwrap();
    assert_eq!(by_sw.logs[0].id, b);
    let by_errors = db.list(&ListFilters { sort: Some("error_count:desc".into()), ..Default::default() }).await.unwrap();
    assert_eq!(by_errors.logs[0].id, a);
    let by_warnings = db.list(&ListFilters { sort: Some("warning_count:desc".into()), ..Default::default() }).await.unwrap();
    assert_eq!(by_warnings.logs[0].id, b);
}
