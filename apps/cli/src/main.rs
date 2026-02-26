//! Ubertooth One CLI - Standalone command-line interface.
//!
//! Phase 1: Placeholder for future standalone CLI tool.

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "ubertooth-cli",
    about = "Standalone CLI for Ubertooth One operations"
)]
struct Args {
    /// Command to execute
    command: Option<String>,
}

fn main() {
    let _args = Args::parse();

    println!("ubertooth-cli v{}", env!("CARGO_PKG_VERSION"));
    println!();
    println!("Standalone CLI not yet implemented.");
    println!("Use 'ubertooth-agent' for connector operations.");
    println!();
    println!("Planned for Phase 2.");
}
