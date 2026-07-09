use axum::{
    extract::{Path, State},
    Json,
};
use bytes::Bytes;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

use crate::ai::{AiAnalysis, AiBalanceResponse, AiError, AiModelsResponse};

use super::ApiError;

/// GET /api/ai/models — models available to the configured OpenRouter key.
pub async fn list_models(
    State(state): State<Arc<crate::AppState>>,
) -> Result<Json<AiModelsResponse>, ApiError> {
    let Some(client) = &state.openrouter else {
        return Ok(Json(AiModelsResponse {
            enabled: false,
            default_model: None,
            models: vec![],
        }));
    };

    let models = client.list_models().await.map_err(map_ai_error)?;
    Ok(Json(AiModelsResponse {
        enabled: true,
        default_model: Some(client.default_model().to_string()),
        models,
    }))
}

/// GET /api/ai/balance — safe usage and spending-limit details for the configured key.
pub async fn get_balance(
    State(state): State<Arc<crate::AppState>>,
) -> Result<Json<AiBalanceResponse>, ApiError> {
    let Some(client) = &state.openrouter else {
        return Ok(Json(AiBalanceResponse::disabled()));
    };

    client.key_balance().await.map(Json).map_err(map_ai_error)
}

/// GET /api/logs/:id/ai-analysis — return the most recently saved analysis.
pub async fn get_analysis(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<AiAnalysis>, ApiError> {
    if state.db.get(id).await?.is_none() {
        return Err(ApiError::NotFound);
    }
    let bytes = state
        .storage
        .get_file(id, "ai-analysis.json")
        .await
        .map_err(|_| ApiError::NotFound)?;
    let analysis = serde_json::from_slice(&bytes)
        .map_err(|e| ApiError::Internal(format!("invalid stored AI analysis: {e}")))?;
    Ok(Json(analysis))
}

#[derive(Debug, Deserialize)]
pub struct GenerateAnalysisRequest {
    pub model: String,
}

/// POST /api/logs/:id/ai-analysis — generate or replace an analysis.
pub async fn generate_analysis(
    State(state): State<Arc<crate::AppState>>,
    Path(id): Path<Uuid>,
    Json(request): Json<GenerateAnalysisRequest>,
) -> Result<Json<AiAnalysis>, ApiError> {
    if state.db.get(id).await?.is_none() {
        return Err(ApiError::NotFound);
    }
    let client = state
        .openrouter
        .as_ref()
        .ok_or_else(|| ApiError::ServiceUnavailable("AI analysis is not configured".into()))?;
    let metadata = state
        .storage
        .get_file(id, "metadata.json")
        .await
        .map_err(|_| ApiError::NotFound)?;
    let metadata: serde_json::Value = serde_json::from_slice(&metadata)
        .map_err(|e| ApiError::Internal(format!("invalid metadata: {e}")))?;

    let analysis = client
        .analyze(request.model.trim(), &metadata)
        .await
        .map_err(map_ai_error)?;
    store_analysis(&state, id, &analysis).await?;
    Ok(Json(analysis))
}

pub async fn store_analysis(
    state: &crate::AppState,
    id: Uuid,
    analysis: &AiAnalysis,
) -> Result<(), ApiError> {
    let bytes = serde_json::to_vec_pretty(analysis)
        .map_err(|e| ApiError::Internal(format!("AI analysis serialization: {e}")))?;
    state
        .storage
        .put_file(id, "ai-analysis.json", Bytes::from(bytes))
        .await?;
    Ok(())
}

pub fn map_ai_error(error: AiError) -> ApiError {
    match error {
        AiError::NotConfigured => ApiError::ServiceUnavailable(error.to_string()),
        AiError::InvalidModel => ApiError::BadRequest(error.to_string()),
        AiError::Request(_)
        | AiError::Provider { .. }
        | AiError::MissingContent
        | AiError::InvalidResponse(_) => ApiError::BadGateway(error.to_string()),
    }
}
