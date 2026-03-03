//! Event handling for TUI

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent, MouseEvent};
use std::time::Duration;

/// Terminal events
#[derive(Clone, Copy, Debug)]
pub enum Event {
    /// Terminal tick
    Tick,
    /// Key press
    Key(KeyEvent),
    /// Mouse click/scroll
    Mouse(MouseEvent),
    /// Terminal resize
    Resize(u16, u16),
}

/// Event handler
pub struct EventHandler {
    /// Tick rate in milliseconds
    tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler with given tick rate (ms)
    pub fn new(tick_rate_ms: u64) -> Self {
        Self {
            tick_rate: Duration::from_millis(tick_rate_ms),
        }
    }

    /// Get next event (blocking with timeout)
    pub fn next(&self) -> Result<Option<CrosstermEvent>> {
        if event::poll(self.tick_rate)? {
            Ok(Some(event::read()?))
        } else {
            Ok(None)
        }
    }
}
