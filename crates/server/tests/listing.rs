use chrono::Utc;
use flight_review_server::db::{create_db, ListFilters, LogRecord};
use uuid::Uuid;

fn record(source: Option<&str>) -> LogRecord {
    LogRecord {
        id: Uuid::new_v4(),
        filename: format!("{}.ulg", source.unwrap_or("web")),
        created_at: Utc::now(),
        file_size: 42,
        sys_name: Some("PX4".to_string()),
        ver_hw: Some("SITL".to_string()),
        ver_sw_release_str: None,
        flight_duration_s: Some(12.0),
        topic_count: 1,
        lat: None,
        lon: None,
        is_public: true,
        delete_token: Uuid::new_v4().simple().to_string(),
        description: source.map(|s| format!("{s} upload")),
        wind_speed: None,
        rating: None,
        feedback: None,
        video_url: None,
        source: source.map(str::to_string),
        pilot_name: None,
        vehicle_name: None,
        tags: None,
        location_name: None,
        mission_type: None,
        sys_uuid: None,
        ver_sw: None,
        vehicle_type: None,
        localization_sources: None,
        vibration_status: None,
        battery_min_voltage: None,
        gps_max_eph: None,
        max_speed_m_s: None,
        total_distance_m: None,
        error_count: None,
        warning_count: None,
        analysis_version: None,
        diagnostic_flags: None,
    }
}

#[tokio::test]
async fn public_listing_excludes_ci_uploads_by_default() {
    let tmp = tempfile::tempdir().unwrap();
    let db_url = format!("sqlite://{}", tmp.path().join("listing.db").display());
    let db = create_db(&db_url).await.unwrap();

    let web = record(Some("webui"));
    let ci = record(Some("CI"));
    db.insert(&web).await.unwrap();
    db.insert(&ci).await.unwrap();

    let default = db.list(&ListFilters::default()).await.unwrap();
    assert_eq!(default.total, 1);
    assert_eq!(default.logs[0].id, web.id);

    let with_ci = db
        .list(&ListFilters {
            include_ci: Some(true),
            ..ListFilters::default()
        })
        .await
        .unwrap();
    assert_eq!(with_ci.total, 2);
    assert!(with_ci.logs.iter().any(|log| log.id == web.id));
    assert!(with_ci.logs.iter().any(|log| log.id == ci.id));
}
