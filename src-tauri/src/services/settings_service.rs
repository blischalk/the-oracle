use std::sync::Arc;

use crate::domain::settings::AppSettings;
use crate::keychain::keychain_service::KeychainService;
use crate::persistence::settings_repository::SettingsRepository;

pub struct SettingsService {
    settings_repository: Arc<SettingsRepository>,
    #[allow(dead_code)]
    keychain: Arc<KeychainService>,
}

impl SettingsService {
    pub fn new(
        settings_repository: Arc<SettingsRepository>,
        keychain: Arc<KeychainService>,
    ) -> Self {
        Self {
            settings_repository,
            keychain,
        }
    }

    pub fn get_settings(&self) -> anyhow::Result<AppSettings> {
        self.settings_repository.get()
    }

    pub fn save_settings(&self, settings: &AppSettings) -> anyhow::Result<()> {
        self.settings_repository.save(settings)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::database::Database;
    use crate::persistence::settings_repository::SettingsRepository;

    fn build_service() -> SettingsService {
        let database = Database::open_in_memory().unwrap();
        let settings_repo = Arc::new(SettingsRepository::new(database.connection));
        let keychain = Arc::new(KeychainService::new());
        SettingsService::new(settings_repo, keychain)
    }

    #[test]
    fn get_settings_returns_defaults_on_first_run() {
        let service = build_service();
        let settings = service.get_settings().unwrap();
        let defaults = AppSettings::default();

        assert_eq!(settings.active_provider_id, defaults.active_provider_id);
        assert_eq!(settings.theme, defaults.theme);
    }

    #[test]
    fn save_and_get_settings_round_trips() {
        let service = build_service();
        let updated = AppSettings {
            active_provider_id: "anthropic".to_string(),
            active_model_id: "claude-sonnet-4-5".to_string(),
            theme: "dark".to_string(),
            is_fullscreen: true,
        };

        service.save_settings(&updated).unwrap();

        let retrieved = service.get_settings().unwrap();
        assert_eq!(retrieved.theme, "dark");
        assert!(retrieved.is_fullscreen);
    }
}
