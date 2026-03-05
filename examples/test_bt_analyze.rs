use ubertooth_platform::RustUsbBackend;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    println!("Testing bt_analyze...\n");

    let backend = RustUsbBackend::new()?;

    let result = backend
        .call_method(
            "bt_analyze",
            json!({
                "capture_id": "cap-btle-06b8b707-431f-4b7c-8eda-fb02b7e253d3"
            }),
        )
        .await?;

    println!("\n=== Result ===");
    println!("{}", serde_json::to_string_pretty(&result)?);

    Ok(())
}
