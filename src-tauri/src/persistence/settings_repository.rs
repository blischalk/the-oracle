use std::sync::{Arc, Mutex};

use anyhow::Context;
use rusqlite::{params, Connection};

use crate::domain::settings::AppSettings;

const KEY_ACTIVE_PROVIDER_ID: &str = "active_provider_id";
const KEY_ACTIVE_MODEL_ID: &str = "active_model_id";
const KEY_THEME: &str = "theme";
const KEY_IS_FULLSCREEN: &str = "is_fullscreen";
const KEY_NARRATION_ENABLED: &str = "narration_enabled";
const KEY_NARRATION_RATE: &str = "narration_rate";
const KEY_NARRATION_VOICE_URI: &str = "narration_voice_uri";
const KEY_TTS_PROVIDER: &str = "tts_provider";
const KEY_TTS_OPENAI_VOICE: &str = "tts_openai_voice";

pub struct SettingsRepository {
    connection: Arc<Mutex<Connection>>,
}

impl SettingsRepository {
    pub fn new(connection: Arc<Mutex<Connection>>) -> Self {
        Self { connection }
    }

    pub fn get(&self) -> anyhow::Result<AppSettings> {
        let defaults = AppSettings::default();
        let connection = self.connection.lock().unwrap();

        let active_provider_id =
            read_key(&connection, KEY_ACTIVE_PROVIDER_ID)?.unwrap_or(defaults.active_provider_id);

        let active_model_id =
            read_key(&connection, KEY_ACTIVE_MODEL_ID)?.unwrap_or(defaults.active_model_id);

        let theme = read_key(&connection, KEY_THEME)?.unwrap_or(defaults.theme);

        let is_fullscreen_str = read_key(&connection, KEY_IS_FULLSCREEN)?.unwrap_or_else(|| {
            if defaults.is_fullscreen {
                "true".to_string()
            } else {
                "false".to_string()
            }
        });

        let is_fullscreen = is_fullscreen_str == "true";

        let narration_enabled_str =
            read_key(&connection, KEY_NARRATION_ENABLED)?.unwrap_or_else(|| "false".to_string());
        let narration_enabled = narration_enabled_str == "true";

        let narration_rate_str =
            read_key(&connection, KEY_NARRATION_RATE)?.unwrap_or_else(|| "1.0".to_string());
        let narration_rate = narration_rate_str.parse::<f32>().unwrap_or(1.0);

        let narration_voice_uri =
            read_key(&connection, KEY_NARRATION_VOICE_URI)?.unwrap_or_default();

        let tts_provider = read_key(&connection, KEY_TTS_PROVIDER)?
            .unwrap_or_else(|| "system".to_string());

        let tts_openai_voice = read_key(&connection, KEY_TTS_OPENAI_VOICE)?
            .unwrap_or_else(|| "nova".to_string());

        Ok(AppSettings {
            active_provider_id,
            active_model_id,
            theme,
            is_fullscreen,
            narration_enabled,
            narration_rate,
            narration_voice_uri,
            tts_provider,
            tts_openai_voice,
        })
    }

    pub fn save(&self, settings: &AppSettings) -> anyhow::Result<()> {
        let connection = self.connection.lock().unwrap();

        upsert_key(
            &connection,
            KEY_ACTIVE_PROVIDER_ID,
            &settings.active_provider_id,
        )?;
        upsert_key(&connection, KEY_ACTIVE_MODEL_ID, &settings.active_model_id)?;
        upsert_key(&connection, KEY_THEME, &settings.theme)?;
        upsert_key(
            &connection,
            KEY_IS_FULLSCREEN,
            if settings.is_fullscreen {
                "true"
            } else {
                "false"
            },
        )?;
        upsert_key(
            &connection,
            KEY_NARRATION_ENABLED,
            if settings.narration_enabled {
                "true"
            } else {
                "false"
            },
        )?;
        upsert_key(
            &connection,
            KEY_NARRATION_RATE,
            &settings.narration_rate.to_string(),
        )?;
        upsert_key(
            &connection,
            KEY_NARRATION_VOICE_URI,
            &settings.narration_voice_uri,
        )?;
        upsert_key(&connection, KEY_TTS_PROVIDER, &settings.tts_provider)?;
        upsert_key(&connection, KEY_TTS_OPENAI_VOICE, &settings.tts_openai_voice)?;

        Ok(())
    }
}

fn read_key(connection: &Connection, key: &str) -> anyhow::Result<Option<String>> {
    let mut statement = connection
        .prepare("SELECT value FROM app_settings WHERE key = ?1")
        .context("Failed to prepare read_key statement")?;

    let mut rows = statement
        .query_map(params![key], |row| row.get(0))
        .context("Failed to query settings key")?;

    match rows.next() {
        Some(result) => Ok(Some(result.context("Failed to read settings value")?)),
        None => Ok(None),
    }
}

fn upsert_key(connection: &Connection, key: &str, value: &str) -> anyhow::Result<()> {
    connection
        .execute(
            "INSERT INTO app_settings (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .context("Failed to upsert settings key")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::database::Database;

    fn build_repository() -> SettingsRepository {
        let database = Database::open_in_memory().unwrap();
        SettingsRepository::new(database.connection)
    }

    #[test]
    fn get_returns_defaults_when_no_settings_are_stored() {
        let repository = build_repository();
        let settings = repository.get().unwrap();
        let defaults = AppSettings::default();

        assert_eq!(settings.active_provider_id, defaults.active_provider_id);
        assert_eq!(settings.active_model_id, defaults.active_model_id);
        assert_eq!(settings.theme, defaults.theme);
        assert_eq!(settings.is_fullscreen, defaults.is_fullscreen);
        assert_eq!(settings.narration_enabled, defaults.narration_enabled);
        assert_eq!(settings.narration_rate, defaults.narration_rate);
        assert_eq!(settings.narration_voice_uri, defaults.narration_voice_uri);
        assert_eq!(settings.tts_provider, defaults.tts_provider);
        assert_eq!(settings.tts_openai_voice, defaults.tts_openai_voice);
    }

    #[test]
    fn save_and_get_round_trips_settings() {
        let repository = build_repository();
        let settings = AppSettings {
            active_provider_id: "anthropic".to_string(),
            active_model_id: "claude-sonnet-4-5".to_string(),
            theme: "dark".to_string(),
            is_fullscreen: true,
            narration_enabled: false,
            narration_rate: 1.0,
            narration_voice_uri: "".to_string(),
            tts_provider: "system".to_string(),
            tts_openai_voice: "nova".to_string(),
        };

        repository.save(&settings).unwrap();
        let retrieved = repository.get().unwrap();

        assert_eq!(retrieved.active_provider_id, "anthropic");
        assert_eq!(retrieved.active_model_id, "claude-sonnet-4-5");
        assert_eq!(retrieved.theme, "dark");
        assert!(retrieved.is_fullscreen);
        assert!(!retrieved.narration_enabled);
        assert_eq!(retrieved.narration_rate, 1.0);
        assert_eq!(retrieved.narration_voice_uri, "");
    }

    #[test]
    fn save_is_idempotent() {
        let repository = build_repository();
        let settings = AppSettings {
            active_provider_id: "anthropic".to_string(),
            active_model_id: "claude-opus-4-5".to_string(),
            theme: "default".to_string(),
            is_fullscreen: false,
            narration_enabled: false,
            narration_rate: 1.0,
            narration_voice_uri: "".to_string(),
            tts_provider: "system".to_string(),
            tts_openai_voice: "nova".to_string(),
        };

        repository.save(&settings).unwrap();
        repository.save(&settings).unwrap();

        let retrieved = repository.get().unwrap();
        assert_eq!(retrieved.theme, "default");
    }
}
