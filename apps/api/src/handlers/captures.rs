//! Capture management endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use ubertooth_platform::{comparison, CaptureData, CaptureMetadata, ComparisonEngine};
use utoipa::ToSchema;

use crate::state::AppState;

/// Capture response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CaptureResponse {
    pub capture_id: String,
    pub timestamp: String,
    pub capture_type: String,
    pub packet_count: usize,
    pub duration_sec: Option<u64>,
    pub file_size_bytes: u64,
    pub tags: Vec<String>,
    pub description: String,
    pub category: Option<String>,
}

impl From<CaptureMetadata> for CaptureResponse {
    fn from(meta: CaptureMetadata) -> Self {
        Self {
            capture_id: meta.capture_id,
            timestamp: meta.timestamp.to_rfc3339(),
            capture_type: meta.capture_type,
            packet_count: meta.packet_count,
            duration_sec: meta.duration_sec,
            file_size_bytes: meta.file_size_bytes,
            tags: meta.tags,
            description: meta.description,
            category: meta.category,
        }
    }
}

/// Capture list response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct CaptureListResponse {
    pub captures: Vec<CaptureResponse>,
    pub total: usize,
}

/// Compare request
#[derive(Debug, Deserialize, ToSchema)]
pub struct CompareRequest {
    pub capture_ids: Vec<String>,
}

/// List all captures
#[utoipa::path(
    get,
    path = "/api/v1/captures",
    tag = "captures",
    responses(
        (status = 200, description = "List of captures", body = CaptureListResponse),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_captures(
    State(state): State<AppState>,
) -> Result<Json<CaptureListResponse>, StatusCode> {
    let captures = state
        .capture_store
        .list_captures()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let total = captures.len();
    let captures: Vec<CaptureResponse> = captures.into_iter().map(Into::into).collect();

    Ok(Json(CaptureListResponse { captures, total }))
}

/// Get capture by ID
#[utoipa::path(
    get,
    path = "/api/v1/captures/{id}",
    tag = "captures",
    params(
        ("id" = String, Path, description = "Capture ID")
    ),
    responses(
        (status = 200, description = "Capture details", body = CaptureResponse),
        (status = 404, description = "Capture not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_capture(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<CaptureResponse>, StatusCode> {
    let metadata = state
        .capture_store
        .load_metadata(&id)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(metadata.into()))
}

/// Delete capture by ID
#[utoipa::path(
    delete,
    path = "/api/v1/captures/{id}",
    tag = "captures",
    params(
        ("id" = String, Path, description = "Capture ID")
    ),
    responses(
        (status = 204, description = "Capture deleted"),
        (status = 404, description = "Capture not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn delete_capture(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<StatusCode, StatusCode> {
    state
        .capture_store
        .delete_capture(&id)
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(StatusCode::NO_CONTENT)
}

/// Compare multiple captures
#[utoipa::path(
    post,
    path = "/api/v1/captures/compare",
    tag = "captures",
    request_body = CompareRequest,
    responses(
        (status = 200, description = "Comparison result"),
        (status = 400, description = "Invalid request"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn compare_captures(
    State(state): State<AppState>,
    Json(request): Json<CompareRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if request.capture_ids.len() < 2 {
        return Err(StatusCode::BAD_REQUEST);
    }

    // Load capture data (simplified - would need actual packet data)
    let captures: Vec<CaptureData> = request
        .capture_ids
        .iter()
        .map(|id| {
            let metadata = state
                .capture_store
                .load_metadata(id)
                .map_err(|_| StatusCode::NOT_FOUND)?;

            // Create simplified capture data
            // In production, this would load actual packet data from PCAP
            Ok(CaptureData {
                capture_id: id.clone(),
                packets: vec![], // TODO: Load from PCAP
                devices: vec![], // TODO: Extract from packets
            })
        })
        .collect::<Result<Vec<_>, StatusCode>>()?;

    let result = ComparisonEngine::compare_captures(captures)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let json = serde_json::to_value(&result).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capture_response_conversion() {
        use chrono::Utc;
        use ubertooth_platform::CaptureMetadata;

        let metadata = CaptureMetadata {
            capture_id: "test-123".to_string(),
            timestamp: Utc::now(),
            capture_type: "btle_scan".to_string(),
            packet_count: 100,
            duration_sec: Some(60),
            file_size_bytes: 1024,
            pcap_path: "/path/to/file.pcap".to_string(),
            tags: vec!["test".to_string()],
            description: "Test capture".to_string(),
            category: None,
            notes: None,
        };

        let response: CaptureResponse = metadata.into();
        assert_eq!(response.capture_id, "test-123");
        assert_eq!(response.packet_count, 100);
    }
}
