//! Stores API keys in the app's SQLite database under the `app_settings` table.
//!
//! Keys are stored with the prefix `api_key_<provider_id>` so they coexist
//! safely with other settings rows. This avoids macOS Keychain code-signing
//! requirements during development while keeping keys in the app's
//! user-specific data directory (protected by macOS filesystem permissions).
//!
//! When the app is properly code-signed for distribution the same interface
//! can be backed by the system keychain without changing any call sites.

use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};
use thiserror::Error;

const KEY_PREFIX: &str = "api_key_";

#[derive(Debug, Error)]
pub enum KeychainError {
    #[error("Failed to access credential store: {0}")]
    AccessError(String),
}

pub struct KeychainService {
    connection: Arc<Mutex<Connection>>,
}

impl KeychainService {
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    pub fn save_api_key(&self, provider_id: &str, key: &str) -> Result<(), KeychainError> {
        let db_key = settings_key_for(provider_id);
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![db_key, key],
        )
        .map_err(|e| KeychainError::AccessError(e.to_string()))?;
        Ok(())
    }

    pub fn get_api_key(&self, provider_id: &str) -> Result<Option<String>, KeychainError> {
        let db_key = settings_key_for(provider_id);
        let conn = self.connection.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT value FROM app_settings WHERE key = ?1")
            .map_err(|e| KeychainError::AccessError(e.to_string()))?;

        let mut rows = stmt
            .query_map(params![db_key], |row| row.get::<_, String>(0))
            .map_err(|e| KeychainError::AccessError(e.to_string()))?;

        match rows.next() {
            Some(Ok(value)) => Ok(Some(value)),
            Some(Err(e)) => Err(KeychainError::AccessError(e.to_string())),
            None => Ok(None),
        }
    }

    pub fn delete_api_key(&self, provider_id: &str) -> Result<(), KeychainError> {
        let db_key = settings_key_for(provider_id);
        let conn = self.connection.lock().unwrap();
        conn.execute(
            "DELETE FROM app_settings WHERE key = ?1",
            params![db_key],
        )
        .map_err(|e| KeychainError::AccessError(e.to_string()))?;
        Ok(())
    }
}

fn settings_key_for(provider_id: &str) -> String {
    format!("{KEY_PREFIX}{provider_id}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::database::Database;

    fn build_service() -> KeychainService {
        let db = Database::open_in_memory().unwrap();
        KeychainService::new(db.connection)
    }

    #[test]
    fn save_and_get_round_trips_api_key() {
        let service = build_service();
        service.save_api_key("anthropic", "sk-test-123").unwrap();
        let key = service.get_api_key("anthropic").unwrap();
        assert_eq!(key, Some("sk-test-123".to_string()));
    }

    #[test]
    fn get_returns_none_when_no_key_stored() {
        let service = build_service();
        let key = service.get_api_key("openai").unwrap();
        assert_eq!(key, None);
    }

    #[test]
    fn delete_removes_stored_key() {
        let service = build_service();
        service.save_api_key("gemini", "gm-key").unwrap();
        service.delete_api_key("gemini").unwrap();
        let key = service.get_api_key("gemini").unwrap();
        assert_eq!(key, None);
    }

    #[test]
    fn delete_is_idempotent_when_key_missing() {
        let service = build_service();
        // Should not error even if the key was never saved
        service.delete_api_key("ollama").unwrap();
    }

    #[test]
    fn keys_for_different_providers_are_independent() {
        let service = build_service();
        service.save_api_key("anthropic", "key-a").unwrap();
        service.save_api_key("openai", "key-b").unwrap();
        assert_eq!(service.get_api_key("anthropic").unwrap(), Some("key-a".to_string()));
        assert_eq!(service.get_api_key("openai").unwrap(), Some("key-b".to_string()));
    }
}
