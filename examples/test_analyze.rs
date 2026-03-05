//! Quick test of bt_analyze Phase 2 implementation

use serde_json::json;
use ubertooth_platform::backend::UbertoothBackendProvider;
use ubertooth_platform::sidecar::SidecarManager;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("info")
        .init();

    println!("========================================");
    println!("Testing bt_analyze Phase 2 Implementation");
    println!("========================================\n");

    let backend = SidecarManager::new();

    let capture_id = "cap-btle-06b8b707-431f-4b7c-8eda-fb02b7e253d3";
    println!("Analyzing capture: {}\n", capture_id);

    let params = json!({
        "capture_id": capture_id,
        "analysis_type": "auto"
    });

    match backend.execute_tool("bt_analyze", params).await {
        Ok(result) => {
            println!("✅ Analysis complete!\n");
            println!("{}", serde_json::to_string_pretty(&result)?);
        }
        Err(e) => {
            eprintln!("❌ Analysis failed: {}", e);
            return Err(e.into());
        }
    }

    println!("\n========================================");
    println!("Test Complete");
    println!("========================================");

    Ok(())
}
