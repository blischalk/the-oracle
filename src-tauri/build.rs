static APP_COMMANDS: &[&str] = &[
    "list_campaigns",
    "create_campaign",
    "get_campaign",
    "get_campaign_state",
    "get_rpg_system",
    "archive_campaign",
    "delete_campaign",
    "update_campaign_name",
    "get_messages",
    "send_chat_message",
    "request_gm_greeting",
    "extract_character_data",
    "suggest_campaign_name",
    "list_providers",
    "validate_api_key",
    "get_settings",
    "save_settings",
    "save_api_key",
    "get_api_key",
    "delete_api_key",
];

fn main() {
    tauri_build::try_build(
        tauri_build::Attributes::new()
            .app_manifest(tauri_build::AppManifest::new().commands(APP_COMMANDS)),
    )
    .expect("failed to run tauri build");
}
