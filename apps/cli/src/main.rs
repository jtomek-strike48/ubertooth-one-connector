//! Ubertooth One CLI - Standalone command-line interface.
//!
//! Supports both traditional CLI commands and interactive TUI mode.

mod tui;

use clap::Parser;

#[derive(Parser)]
#[command(
    name = "ubertooth-cli",
    about = "Standalone CLI for Ubertooth One operations"
)]
struct Args {
    /// Launch interactive TUI mode
    #[arg(long)]
    tui: bool,

    /// Command to execute (for non-TUI mode)
    command: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.tui {
        // Run TUI mode
        tui::run().await?;
    } else {
        // Traditional CLI mode (not yet implemented)
        println!("ubertooth-cli v{}", env!("CARGO_PKG_VERSION"));
        println!();
        println!("Traditional CLI not yet implemented.");
        println!("Use 'ubertooth-cli --tui' for interactive mode.");
        println!("Use 'ubertooth-agent' for connector operations.");
        println!();
    }

    Ok(())
}
