use axum::Json;
use utoipa::OpenApi;

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::api::health::health,
        crate::api::version::version,
        crate::api::upload::upload,
        crate::api::logs::list_logs,
        crate::api::logs::list_facets,
        crate::api::logs::get_log,
        crate::api::logs::delete_log,
        crate::api::logs::get_track,
        crate::api::logs::get_log_file,
        crate::api::stats::get_stats,
    ),
    components(
        schemas(
            crate::api::ErrorResponse,
            crate::api::health::HealthResponse,
            crate::api::version::VersionInfo,
            crate::api::upload::UploadRequest,
            crate::api::upload::UploadResponse,
            crate::api::logs::DeleteParams,
            crate::api::logs::TrackPointCompact,
            crate::api::stats::StatsResponse,
            crate::db::LogRecord,
            crate::db::ListFilters,
            crate::db::ListResponse,
            crate::db::FacetsResponse,
            crate::db::StatsParams,
            crate::db::StatRow,
        )
    ),
    tags(
        (name = "Health", description = "Health check"),
        (name = "Version", description = "Build and dependency versions"),
        (name = "Upload", description = "ULog uploads"),
        (name = "Logs", description = "Flight log listing, detail, files, and deletion"),
        (name = "Stats", description = "Aggregated flight log statistics")
    )
)]
pub struct ApiDoc;

pub async fn openapi_json() -> Json<utoipa::openapi::OpenApi> {
    Json(ApiDoc::openapi())
}

#[cfg(test)]
mod tests {
    use super::ApiDoc;
    use std::{fs, path::PathBuf};
    use utoipa::OpenApi;

    #[test]
    fn export_openapi() {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let path = manifest_dir.join("../../openapi/openapi.json");
        let spec = serde_json::to_string_pretty(&ApiDoc::openapi()).expect("serialize OpenAPI");

        fs::create_dir_all(path.parent().expect("openapi dir")).expect("create openapi dir");
        fs::write(path, format!("{spec}\n")).expect("write openapi spec");
    }

    #[test]
    fn openapi_lists_expected_paths() {
        let spec = ApiDoc::openapi();
        for path in [
            "/health",
            "/api/version",
            "/api/upload",
            "/api/logs",
            "/api/logs/facets",
            "/api/stats",
            "/api/logs/{id}",
            "/api/logs/{id}/track",
            "/api/logs/{id}/data/{filename}",
        ] {
            assert!(spec.paths.paths.contains_key(path), "missing {path}");
        }
    }
}
