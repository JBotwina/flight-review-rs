pub mod ai;
pub mod health;
pub mod logs;
pub mod stats;
pub mod upload;
pub mod version;

use axum::{http::StatusCode, response::IntoResponse, Json};

use crate::db::DbError;
use crate::storage::StorageError;

#[derive(Debug)]
pub enum ApiError {
    NotFound,
    Forbidden,
    BadRequest(String),
    BadGateway(String),
    ServiceUnavailable(String),
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> axum::response::Response {
        let (status, message) = match self {
            ApiError::NotFound => (StatusCode::NOT_FOUND, "not found".to_string()),
            ApiError::Forbidden => (StatusCode::FORBIDDEN, "invalid token".to_string()),
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::BadGateway(msg) => (StatusCode::BAD_GATEWAY, msg),
            ApiError::ServiceUnavailable(msg) => (StatusCode::SERVICE_UNAVAILABLE, msg),
            ApiError::Internal(msg) => {
                tracing::error!("internal error: {msg}");
                (StatusCode::INTERNAL_SERVER_ERROR, msg)
            }
        };
        (status, Json(serde_json::json!({"error": message}))).into_response()
    }
}

impl From<DbError> for ApiError {
    fn from(err: DbError) -> Self {
        ApiError::Internal(err.to_string())
    }
}

impl From<StorageError> for ApiError {
    fn from(err: StorageError) -> Self {
        ApiError::Internal(err.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(err: std::io::Error) -> Self {
        ApiError::Internal(err.to_string())
    }
}
