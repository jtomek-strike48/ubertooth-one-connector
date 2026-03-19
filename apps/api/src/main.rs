//! Ubertooth One REST API Server
//!
//! Provides HTTP/WebSocket access to Ubertooth functionality.

mod handlers;
mod state;

use axum::{
    routing::{delete, get, post},
    Router,
};
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::handlers::{captures, devices, health, streaming};
use crate::state::AppState;

#[derive(OpenApi)]
#[openapi(
    paths(
        health::health_check,
        captures::list_captures,
        captures::get_capture,
        captures::delete_capture,
        captures::compare_captures,
        devices::list_devices,
        streaming::stream_packets,
    ),
    components(
        schemas(
            handlers::captures::CaptureResponse,
            handlers::captures::CaptureListResponse,
            handlers::captures::CompareRequest,
            handlers::devices::DeviceResponse,
            handlers::streaming::StreamConfig,
        )
    ),
    tags(
        (name = "health", description = "Health check endpoints"),
        (name = "captures", description = "Capture management endpoints"),
        (name = "devices", description = "Device information endpoints"),
        (name = "streaming", description = "Real-time packet streaming"),
    )
)]
struct ApiDoc;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ubertooth_api=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize application state
    let state = AppState::new().await?;

    // Build router
    let app = Router::new()
        // Health check
        .route("/health", get(health::health_check))
        // Captures
        .route("/api/v1/captures", get(captures::list_captures))
        .route("/api/v1/captures/:id", get(captures::get_capture))
        .route("/api/v1/captures/:id", delete(captures::delete_capture))
        .route("/api/v1/captures/compare", post(captures::compare_captures))
        // Devices
        .route("/api/v1/devices", get(devices::list_devices))
        .route("/api/v1/devices/:mac", get(devices::get_device))
        // Streaming
        .route("/api/v1/stream", get(streaming::stream_packets))
        // Swagger UI
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDoc::openapi()))
        // Add middleware
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .layer(TraceLayer::new_for_http())
        // Add state
        .with_state(state);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Starting Ubertooth API server on {}", addr);
    tracing::info!("Swagger UI available at http://localhost:3000/swagger-ui");

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
