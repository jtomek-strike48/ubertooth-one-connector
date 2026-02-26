//! Logging initialization for the Ubertooth One connector.

use tracing_subscriber::{fmt, prelude::*, EnvFilter};

/// Initialize logging with the specified level.
///
/// # Arguments
///
/// * `level` - Log level: trace, debug, info, warn, error
pub fn init_logging(level: &str) {
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new(level));

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer())
        .init();
}
