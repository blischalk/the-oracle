use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub active_provider_id: String,
    pub active_model_id: String,
    pub theme: String,
    pub is_fullscreen: bool,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            active_provider_id: "anthropic".to_string(),
            active_model_id: "claude-opus-4-5".to_string(),
            theme: "default".to_string(),
            is_fullscreen: false,
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
        assert_eq!(settings.active_model_id, "claude-opus-4-5");
        assert_eq!(settings.theme, "default");
        assert!(!settings.is_fullscreen);
    }
}
