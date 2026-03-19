//! Device information endpoints

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::state::AppState;

/// Device response
#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct DeviceResponse {
    pub mac_address: String,
    pub name: Option<String>,
    pub first_seen: String,
    pub last_seen: String,
    pub packet_count: usize,
    pub captures: Vec<String>,
}

/// Device list response
#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceListResponse {
    pub devices: Vec<DeviceResponse>,
    pub total: usize,
}

/// List all discovered devices
#[utoipa::path(
    get,
    path = "/api/v1/devices",
    tag = "devices",
    responses(
        (status = 200, description = "List of devices"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn list_devices(
    State(state): State<AppState>,
) -> Result<Json<DeviceListResponse>, StatusCode> {
    // Load all captures
    let captures = state
        .capture_store
        .list_captures()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // TODO: Extract unique devices from captures
    // For now, return empty list
    let devices = vec![];

    Ok(Json(DeviceListResponse {
        devices,
        total: 0,
    }))
}

/// Get device by MAC address
#[utoipa::path(
    get,
    path = "/api/v1/devices/{mac}",
    tag = "devices",
    params(
        ("mac" = String, Path, description = "Device MAC address")
    ),
    responses(
        (status = 200, description = "Device details", body = DeviceResponse),
        (status = 404, description = "Device not found"),
        (status = 500, description = "Internal server error")
    )
)]
pub async fn get_device(
    State(_state): State<AppState>,
    Path(_mac): Path<String>,
) -> Result<Json<DeviceResponse>, StatusCode> {
    // TODO: Implement device lookup
    Err(StatusCode::NOT_IMPLEMENTED)
}
