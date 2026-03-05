// Simple test for bt_analyze - compile with: rustc --edition 2021 test_bt_analyze_simple.rs
// Or just run directly

fn main() {
    println!("Testing bt_analyze Phase 2 Implementation");
    println!("==========================================\n");

    let capture_id = "cap-btle-06b8b707-431f-4b7c-8eda-fb02b7e253d3";

    // Use the built CLI to analyze
    let status = std::process::Command::new("./target/debug/ubertooth-cli")
        .stdin(std::process::Stdio::piped())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn();

    match status {
        Ok(mut child) => {
            use std::io::Write;
            if let Some(mut stdin) = child.stdin.take() {
                // Try to send the analyze command
                let input = format!("analyze {}\n", capture_id);
                let _ = stdin.write_all(input.as_bytes());
            }

            let output = child.wait_with_output().expect("Failed to read output");
            println!("Output:");
            println!("{}", String::from_utf8_lossy(&output.stdout));

            if !output.stderr.is_empty() {
                eprintln!("Errors:");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
            }
        }
        Err(e) => {
            eprintln!("Failed to launch CLI: {}", e);
        }
    }
}
