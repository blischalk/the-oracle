use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub active_provider_id: String,
    pub active_model_id: String,
    pub theme: String,
    pub is_fullscreen: bool,
    pub narration_enabled: bool,
    pub narration_rate: f32,
    pub narration_voice_uri: String,
    pub tts_provider: String,
    pub tts_openai_voice: String,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            active_provider_id: "anthropic".to_string(),
            active_model_id: "claude-sonnet-4-6".to_string(),
            theme: "ink".to_string(),
            is_fullscreen: false,
            narration_enabled: false,
            narration_rate: 1.0,
            narration_voice_uri: String::new(),
            tts_provider: "system".to_string(),
            tts_openai_voice: "nova".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_settings_point_to_anthropic_claude_opus() {
        let settings = AppSettings::default();

        assert_eq!(settings.active_provider_id, "anthropic");
        assert_eq!(settings.active_model_id, "claude-sonnet-4-6");
        assert_eq!(settings.theme, "ink");
        assert!(!settings.is_fullscreen);
        assert!(!settings.narration_enabled);
        assert_eq!(settings.narration_rate, 1.0);
        assert_eq!(settings.narration_voice_uri, "");
        assert_eq!(settings.tts_provider, "system");
        assert_eq!(settings.tts_openai_voice, "nova");
    }
}
