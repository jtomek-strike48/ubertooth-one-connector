//! Application state shared across handlers.

use std::sync::Arc;
use tokio::sync::RwLock;
use ubertooth_core::error::Result;
use ubertooth_platform::{CaptureStore, StreamingBuffer};

/// Shared application state
#[derive(Clone)]
pub struct AppState {
    /// Capture store for persisted captures
    pub capture_store: Arc<CaptureStore>,
    /// Active streaming buffer
    pub streaming_buffer: Arc<RwLock<Option<StreamingBuffer>>>,
}

impl AppState {
    /// Create new application state
    pub async fn new() -> Result<Self> {
        let capture_store = Arc::new(CaptureStore::new()?);
        let streaming_buffer = Arc::new(RwLock::new(None));

        Ok(Self {
            capture_store,
            streaming_buffer,
        })
    }
}
