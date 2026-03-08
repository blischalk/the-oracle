use keyring::Entry;
use thiserror::Error;

const KEYCHAIN_SERVICE_NAME: &str = "the-oracle";

#[derive(Debug, Error)]
pub enum KeychainError {
    #[error("Failed to access keychain: {0}")]
    AccessError(String),
    #[error("API key not found for provider: {0}")]
    NotFound(String),
}

pub struct KeychainService;

impl KeychainService {
    pub fn new() -> Self {
        Self
    }

    pub fn save_api_key(&self, provider_id: &str, key: &str) -> Result<(), KeychainError> {
        let entry = open_keychain_entry(provider_id)?;
        entry
            .set_password(key)
            .map_err(|error| KeychainError::AccessError(error.to_string()))
    }

    pub fn get_api_key(&self, provider_id: &str) -> Result<Option<String>, KeychainError> {
        let entry = open_keychain_entry(provider_id)?;

        match entry.get_password() {
            Ok(key) => Ok(Some(key)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(KeychainError::AccessError(error.to_string())),
        }
    }

    pub fn delete_api_key(&self, provider_id: &str) -> Result<(), KeychainError> {
        let entry = open_keychain_entry(provider_id)?;

        match entry.delete_credential() {
            Ok(()) => Ok(()),
            Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(KeychainError::AccessError(error.to_string())),
        }
    }
}

impl Default for KeychainService {
    fn default() -> Self {
        Self::new()
    }
}

fn open_keychain_entry(provider_id: &str) -> Result<Entry, KeychainError> {
    Entry::new(KEYCHAIN_SERVICE_NAME, provider_id)
        .map_err(|error| KeychainError::AccessError(error.to_string()))
}
