//! Session management for saving and restoring application state.
//!
//! Allows users to save their current work and resume later without losing context.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use ubertooth_core::error::{Result, UbertoothError};
use uuid::Uuid;

/// Session state that can be saved and restored
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    /// Unique session identifier
    pub session_id: String,
    /// When the session was created
    pub created_at: DateTime<Utc>,
    /// When the session was last updated
    pub updated_at: DateTime<Utc>,
    /// User-provided session name
    pub name: String,
    /// Current tool being used
    pub current_tool: Option<String>,
    /// Tool parameters
    pub parameters: HashMap<String, serde_json::Value>,
    /// Current view state (which screen, selections, etc)
    pub view_state: ViewState,
    /// Active filters
    pub filters: Vec<PacketFilter>,
    /// Open captures
    pub open_captures: Vec<String>,
    /// Bookmarked packets (capture_id -> packet indices)
    pub bookmarks: HashMap<String, Vec<usize>>,
    /// User notes
    pub notes: String,
}

/// View state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewState {
    /// Current state name (e.g., "MainMenu", "Results", "LiveCapture")
    pub state_name: String,
    /// Selected indices per view
    pub selections: HashMap<String, usize>,
    /// Scroll positions per view
    pub scroll_positions: HashMap<String, usize>,
    /// Expanded items (e.g., packet indices)
    pub expanded_items: Vec<usize>,
    /// Active view mode (e.g., "List", "Statistics", "Timeline")
    pub view_mode: Option<String>,
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            state_name: "MainMenu".to_string(),
            selections: HashMap::new(),
            scroll_positions: HashMap::new(),
            expanded_items: Vec::new(),
            view_mode: None,
        }
    }
}

/// Packet filter information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PacketFilter {
    /// Filter field (e.g., "packet_type", "mac_address", "rssi")
    pub field: String,
    /// Filter operator (e.g., "equals", "contains", "greater_than")
    pub operator: String,
    /// Filter value
    pub value: serde_json::Value,
}

impl SessionState {
    /// Create a new session state
    pub fn new(name: String) -> Self {
        let now = Utc::now();
        Self {
            session_id: Self::generate_session_id(),
            created_at: now,
            updated_at: now,
            name,
            current_tool: None,
            parameters: HashMap::new(),
            view_state: ViewState::default(),
            filters: Vec::new(),
            open_captures: Vec::new(),
            bookmarks: HashMap::new(),
            notes: String::new(),
        }
    }

    /// Generate a unique session ID
    pub fn generate_session_id() -> String {
        format!("session-{}", Uuid::new_v4())
    }

    /// Update the last modified timestamp
    pub fn touch(&mut self) {
        self.updated_at = Utc::now();
    }

    /// Get a human-readable session description
    pub fn description(&self) -> String {
        let tool = self.current_tool.as_ref().map(|t| t.as_str()).unwrap_or("None");
        let captures = self.open_captures.len();
        let filters = self.filters.len();

        format!(
            "{} | Tool: {} | {} capture(s) | {} filter(s) | Updated: {}",
            self.name,
            tool,
            captures,
            filters,
            self.updated_at.format("%Y-%m-%d %H:%M:%S")
        )
    }
}

/// Session metadata for listing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_id: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub current_tool: Option<String>,
    pub captures_count: usize,
}

impl From<&SessionState> for SessionMetadata {
    fn from(state: &SessionState) -> Self {
        Self {
            session_id: state.session_id.clone(),
            name: state.name.clone(),
            created_at: state.created_at,
            updated_at: state.updated_at,
            current_tool: state.current_tool.clone(),
            captures_count: state.open_captures.len(),
        }
    }
}

/// Session manager for saving and loading sessions
pub struct SessionManager {
    sessions_dir: PathBuf,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| UbertoothError::BackendError("Could not determine home directory".to_string()))?;

        let sessions_dir = home.join(".ubertooth").join("sessions");
        fs::create_dir_all(&sessions_dir)?;

        Ok(Self { sessions_dir })
    }

    /// Get the sessions directory path
    pub fn sessions_dir(&self) -> &PathBuf {
        &self.sessions_dir
    }

    /// Save a session
    pub fn save_session(&self, state: &SessionState) -> Result<()> {
        let path = self.sessions_dir.join(format!("{}.json", state.session_id));
        let json = serde_json::to_string_pretty(state)?;
        fs::write(path, json)?;
        Ok(())
    }

    /// Load a session by ID
    pub fn load_session(&self, session_id: &str) -> Result<SessionState> {
        let path = self.sessions_dir.join(format!("{}.json", session_id));

        if !path.exists() {
            return Err(UbertoothError::BackendError(format!("Session not found: {}", session_id)));
        }

        let json = fs::read_to_string(path)?;
        let state: SessionState = serde_json::from_str(&json)?;
        Ok(state)
    }

    /// List all sessions, sorted by most recently updated
    pub fn list_sessions(&self) -> Result<Vec<SessionMetadata>> {
        let mut sessions = Vec::new();

        for entry in fs::read_dir(&self.sessions_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().and_then(|s| s.to_str()) == Some("json") {
                let json = fs::read_to_string(&path)?;
                if let Ok(state) = serde_json::from_str::<SessionState>(&json) {
                    sessions.push(SessionMetadata::from(&state));
                }
            }
        }

        // Sort by most recently updated
        sessions.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));

        Ok(sessions)
    }

    /// Delete a session
    pub fn delete_session(&self, session_id: &str) -> Result<()> {
        let path = self.sessions_dir.join(format!("{}.json", session_id));

        if !path.exists() {
            return Err(UbertoothError::BackendError(format!("Session not found: {}", session_id)));
        }

        fs::remove_file(path)?;
        Ok(())
    }

    /// Get the most recent session
    pub fn get_recent_session(&self) -> Result<Option<SessionState>> {
        let sessions = self.list_sessions()?;

        if let Some(metadata) = sessions.first() {
            Ok(Some(self.load_session(&metadata.session_id)?))
        } else {
            Ok(None)
        }
    }

    /// Auto-save functionality - save with auto-generated name
    pub fn auto_save(&self, state: &SessionState) -> Result<()> {
        let mut state = state.clone();
        if state.name.is_empty() || state.name.starts_with("Auto-save") {
            state.name = format!("Auto-save {}", state.updated_at.format("%Y-%m-%d %H:%M:%S"));
        }
        state.touch();
        self.save_session(&state)
    }

    /// Clean up old auto-save sessions (keep last N)
    pub fn cleanup_auto_saves(&self, keep_count: usize) -> Result<usize> {
        let sessions = self.list_sessions()?;

        let auto_saves: Vec<_> = sessions
            .iter()
            .filter(|s| s.name.starts_with("Auto-save"))
            .collect();

        let mut deleted = 0;

        // Delete all but the most recent N auto-saves
        for session in auto_saves.iter().skip(keep_count) {
            self.delete_session(&session.session_id)?;
            deleted += 1;
        }

        Ok(deleted)
    }

    /// Get total number of sessions
    pub fn session_count(&self) -> Result<usize> {
        Ok(self.list_sessions()?.len())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new().expect("Failed to create session manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_manager() -> (SessionManager, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        std::env::set_var("HOME", temp_dir.path());
        let manager = SessionManager::new().unwrap();
        (manager, temp_dir)
    }

    fn create_test_session() -> SessionState {
        let mut session = SessionState::new("Test Session".to_string());
        session.current_tool = Some("bt_decode".to_string());
        session.parameters.insert("channel".to_string(), serde_json::json!(37));
        session.open_captures.push("cap-123".to_string());
        session
    }

    #[test]
    fn test_session_creation() {
        let session = SessionState::new("My Session".to_string());
        assert_eq!(session.name, "My Session");
        assert!(session.session_id.starts_with("session-"));
        assert_eq!(session.open_captures.len(), 0);
    }

    #[test]
    fn test_manager_creation() {
        let (manager, _temp) = create_test_manager();
        assert!(manager.sessions_dir().exists());
    }

    #[test]
    fn test_save_and_load_session() {
        let (manager, _temp) = create_test_manager();
        let session = create_test_session();

        manager.save_session(&session).unwrap();

        let loaded = manager.load_session(&session.session_id).unwrap();
        assert_eq!(loaded.session_id, session.session_id);
        assert_eq!(loaded.name, "Test Session");
        assert_eq!(loaded.current_tool, Some("bt_decode".to_string()));
    }

    #[test]
    fn test_list_sessions() {
        let (manager, _temp) = create_test_manager();

        let session1 = create_test_session();
        let session2 = SessionState::new("Another Session".to_string());

        manager.save_session(&session1).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        manager.save_session(&session2).unwrap();

        let sessions = manager.list_sessions().unwrap();
        assert_eq!(sessions.len(), 2);

        // Most recent should be first
        assert_eq!(sessions[0].name, "Another Session");
        assert_eq!(sessions[1].name, "Test Session");
    }

    #[test]
    fn test_delete_session() {
        let (manager, _temp) = create_test_manager();
        let session = create_test_session();

        manager.save_session(&session).unwrap();
        assert_eq!(manager.session_count().unwrap(), 1);

        manager.delete_session(&session.session_id).unwrap();
        assert_eq!(manager.session_count().unwrap(), 0);
    }

    #[test]
    fn test_get_recent_session() {
        let (manager, _temp) = create_test_manager();

        let recent = manager.get_recent_session().unwrap();
        assert!(recent.is_none());

        let session = create_test_session();
        manager.save_session(&session).unwrap();

        let recent = manager.get_recent_session().unwrap();
        assert!(recent.is_some());
        assert_eq!(recent.unwrap().session_id, session.session_id);
    }

    #[test]
    fn test_auto_save() {
        let (manager, _temp) = create_test_manager();
        let mut session = create_test_session();
        session.name = String::new();

        manager.auto_save(&session).unwrap();

        let sessions = manager.list_sessions().unwrap();
        assert_eq!(sessions.len(), 1);
        assert!(sessions[0].name.starts_with("Auto-save"));
    }

    #[test]
    fn test_cleanup_auto_saves() {
        let (manager, _temp) = create_test_manager();

        // Create 5 auto-save sessions
        for i in 0..5 {
            let mut session = create_test_session();
            session.name = format!("Auto-save {}", i);
            manager.save_session(&session).unwrap();
            std::thread::sleep(std::time::Duration::from_millis(10));
        }

        assert_eq!(manager.session_count().unwrap(), 5);

        // Keep only 2 most recent
        let deleted = manager.cleanup_auto_saves(2).unwrap();
        assert_eq!(deleted, 3);
        assert_eq!(manager.session_count().unwrap(), 2);
    }

    #[test]
    fn test_session_metadata() {
        let session = create_test_session();
        let metadata = SessionMetadata::from(&session);

        assert_eq!(metadata.session_id, session.session_id);
        assert_eq!(metadata.name, "Test Session");
        assert_eq!(metadata.captures_count, 1);
    }

    #[test]
    fn test_session_touch() {
        let mut session = create_test_session();
        let original_time = session.updated_at;

        std::thread::sleep(std::time::Duration::from_millis(10));
        session.touch();

        assert!(session.updated_at > original_time);
    }
}
