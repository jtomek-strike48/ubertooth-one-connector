//! Ubertooth One Connector — Headless Agent
//!
//! When STRIKE48_URL is set, runs in production mode via ConnectorRunner.
//! Otherwise, runs a local smoke test: connect → status → disconnect.

use clap::Parser;
use serde_json::json;
use std::sync::Arc;
use strike48_connector::{BaseConnector, ConnectorConfig, ConnectorRunner};
use ubertooth_core::connector::UbertoothConnector;
use ubertooth_core::events::ToolEvent;
use ubertooth_platform::{CaptureStore, SidecarManager};
use ubertooth_tools::create_tool_registry;

#[derive(Parser)]
#[command(
    name = "ubertooth-agent",
    about = "Ubertooth One connector for Prospector Studio"
)]
struct Args {
    /// Prospector Studio server URL (e.g. wss://studio.example.com:443)
    #[arg(long, short = 's')]
    server_url: Option<String>,

    /// Skip TLS certificate verification (for self-signed certs)
    #[arg(long, short = 'k')]
    insecure: bool,

    /// Tenant ID
    #[arg(long, short = 't')]
    tenant_id: Option<String>,

    /// Authentication token
    #[arg(long)]
    auth_token: Option<String>,

    /// Log level (trace, debug, info, warn, error)
    #[arg(long, default_value = "info")]
    log_level: String,

    /// Backend to use: 'python' (default) or 'rust'
    #[arg(long, default_value = "python")]
    backend: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    ubertooth_core::logging::init_logging(&args.log_level);

    // CLI flags override env vars (set before ConnectorConfig::from_env)
    if let Some(url) = &args.server_url {
        std::env::set_var("STRIKE48_URL", url);
    }
    if args.insecure {
        std::env::set_var("MATRIX_TLS_INSECURE", "true");
    }
    if let Some(tid) = &args.tenant_id {
        std::env::set_var("TENANT_ID", tid);
    }
    if let Some(token) = &args.auth_token {
        std::env::set_var("AUTH_TOKEN", token);
    }

    tracing::info!("ubertooth-agent starting (v{})", env!("CARGO_PKG_VERSION"));

    // Create backend based on selection
    let backend_choice = std::env::var("UBERTOOTH_BACKEND")
        .unwrap_or_else(|_| args.backend.clone())
        .to_lowercase();

    let backend: Arc<dyn ubertooth_platform::UbertoothBackendProvider> = match backend_choice.as_str() {
        "rust" => {
            tracing::info!("Backend: Rust USB (Phase 3 - not yet implemented)");
            return Err(anyhow::anyhow!(
                "Rust USB backend not yet implemented. Please use 'python' backend."
            ));
        }
        "python" | _ => {
            if backend_choice != "python" && !backend_choice.is_empty() {
                tracing::warn!(
                    "Unknown backend '{}', defaulting to Python",
                    backend_choice
                );
            }
            tracing::info!("Backend: Python sidecar (ubertooth-tools)");
            SidecarManager::new()
        }
    };

    // Create capture store (~/.ubertooth/)
    let store = Arc::new(CaptureStore::new()?);
    tracing::info!("Capture store initialized at ~/.ubertooth/");

    // Create tool registry and connector
    let tools = create_tool_registry(backend);
    tracing::info!("Registered {} tools:", tools.tools().len());
    for name in tools.names() {
        tracing::info!("  - {}", name);
    }

    let connector = Arc::new(UbertoothConnector::new(tools));

    // Subscribe to tool events and log them in a background task
    let mut event_rx = connector.subscribe_events();
    tokio::spawn(async move {
        while let Ok(event) = event_rx.recv().await {
            match event {
                ToolEvent::Started { tool_name, .. } => {
                    tracing::info!(tool = %tool_name, "Tool started");
                }
                ToolEvent::Progress { tool_name, data } => {
                    if let Some(message) = data.get("message").and_then(|v| v.as_str()) {
                        tracing::debug!(tool = %tool_name, message, "Tool progress");
                    }
                }
                ToolEvent::Completed {
                    tool_name,
                    duration_ms,
                    success,
                    ..
                } => {
                    tracing::info!(
                        tool = %tool_name,
                        duration_ms,
                        success,
                        "Tool completed"
                    );
                }
                ToolEvent::Failed {
                    tool_name,
                    duration_ms,
                    error,
                } => {
                    tracing::warn!(
                        tool = %tool_name,
                        duration_ms,
                        error = %error,
                        "Tool failed"
                    );
                }
            }
        }
    });

    if std::env::var("STRIKE48_URL").is_ok() {
        run_production(connector).await
    } else {
        run_local_smoke_test(connector).await
    }
}

/// Production mode: connect to Strike48 server via ConnectorRunner.
async fn run_production(connector: Arc<UbertoothConnector>) -> anyhow::Result<()> {
    let mut config = ConnectorConfig::from_env();
    config.connector_type = "ubertooth".to_string();
    config.version = env!("CARGO_PKG_VERSION").to_string();
    config.max_concurrent_requests = 1;

    // Use INSTANCE_ID from env if set, otherwise use generated one
    if let Ok(instance_id) = std::env::var("INSTANCE_ID") {
        config.instance_id = instance_id;
    }

    // Use CONNECTOR_DISPLAY_NAME from env, or default to "Ubertooth One"
    if config.display_name.is_none() {
        config.display_name = Some("Ubertooth One".to_string());
    }

    tracing::info!("Connector configuration:");
    tracing::info!("  Type: {}", config.connector_type);
    tracing::info!("  Instance ID: {}", config.instance_id);
    tracing::info!("  Version: {}", config.version);
    tracing::info!("  Display Name: {:?}", config.display_name);
    tracing::info!("  Host: {}", config.host);
    tracing::info!("  Tenant: {}", config.tenant_id);
    tracing::info!("  Transport: {:?}", config.transport_type);
    tracing::info!("  TLS: {}", config.use_tls);

    let runner = ConnectorRunner::new(config, connector);
    let shutdown = runner.shutdown_handle();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c");
        tracing::info!("Shutdown signal received");
        shutdown.shutdown();
    });

    tracing::info!("Starting ConnectorRunner (press Ctrl+C to stop)...");
    if let Err(e) = runner.run().await {
        tracing::error!("ConnectorRunner exited with error: {}", e);
    }

    tracing::info!("ubertooth-agent shut down");
    Ok(())
}

/// Local smoke test: exercise tools without a Strike48 server.
async fn run_local_smoke_test(connector: Arc<UbertoothConnector>) -> anyhow::Result<()> {
    tracing::info!("No STRIKE48_URL set — running local smoke test");

    // List capabilities
    let caps = connector.capabilities();
    tracing::info!("Capabilities ({}):", caps.len());
    for cap in &caps {
        tracing::info!(
            "  [{}] {} — {}",
            cap.category,
            cap.task_type_id,
            cap.description
        );
    }

    // Connect to Ubertooth One
    tracing::info!("--- device_connect ---");
    let result = connector
        .execute(json!({"tool": "device_connect", "parameters": {}}), None)
        .await?;
    println!("{}", serde_json::to_string_pretty(&result)?);

    let success = result
        .get("success")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);

    if !success {
        let error = result
            .get("error")
            .and_then(|v| v.as_str())
            .unwrap_or("unknown error");
        tracing::error!("device_connect failed: {}", error);
        tracing::info!(
            "Hint: is the Ubertooth One plugged in? Are ubertooth-tools installed? (apt install ubertooth)"
        );
        return Ok(());
    }

    tracing::info!("Local smoke test complete ✓");
    Ok(())
}
