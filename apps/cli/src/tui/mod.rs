//! TUI module - Interactive Terminal User Interface
//!
//! Provides menu-driven access to all 36 Ubertooth tools.

mod app;
mod events;
mod ui;
mod views;

pub use app::{App, AppState, DeviceStatus};
pub use events::EventHandler;

use anyhow::Result;

/// Run the TUI application
pub async fn run() -> Result<()> {
    let mut app = App::new().await?;
    app.run().await
}
