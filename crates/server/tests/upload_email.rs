use flight_review_server::api::upload::is_uploader_email_field;
use flight_review_server::db::{create_db, ListFilters, LogRecord};
use uuid::Uuid;

fn test_record(id: Uuid, uploader_email: Option<&str>) -> LogRecord {
    LogRecord {
        id,
        filename: "sample.ulg".to_string(),
        created_at: chrono::Utc::now(),
        file_size: 123,
        sys_name: Some("PX4".to_string()),
        ver_hw: None,
        ver_sw_release_str: None,
        flight_duration_s: None,
        topic_count: 0,
        lat: None,
        lon: None,
        is_public: true,
        delete_token: "delete-token".to_string(),
        description: None,
        wind_speed: None,
        rating: None,
        feedback: None,
        video_url: None,
        source: None,
        uploader_email: uploader_email.map(str::to_string),
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

#[test]
fn upload_email_aliases_match_legacy_and_explicit_fields() {
    assert!(is_uploader_email_field("email"));
    assert!(is_uploader_email_field("uploader_email"));
    assert!(!is_uploader_email_field("pilot_email"));
}

#[tokio::test]
async fn sqlite_persists_and_lists_uploader_email() {
    let tmp_dir = tempfile::tempdir().expect("tempdir");
    let db_url = format!("sqlite://{}", tmp_dir.path().join("upload-email.db").display());
    let db = create_db(&db_url).await.expect("create db");

    let id = Uuid::new_v4();
    let record = test_record(id, Some("pilot@example.com"));
    db.insert(&record).await.expect("insert record");

    let fetched = db.get(id).await.expect("get record").expect("record exists");
    assert_eq!(fetched.uploader_email.as_deref(), Some("pilot@example.com"));

    let listed = db
        .list(&ListFilters {
            include_private: Some(true),
            ..Default::default()
        })
        .await
        .expect("list records");
    assert_eq!(listed.total, 1);
    assert_eq!(listed.logs[0].uploader_email.as_deref(), Some("pilot@example.com"));
}
