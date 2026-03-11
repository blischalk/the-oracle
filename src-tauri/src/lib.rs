use std::path::{Path, PathBuf};
use std::sync::Arc;

use tauri::path::BaseDirectory;
use tauri::Manager;

use commands::campaign_commands::{
    archive_campaign, create_campaign, delete_campaign, get_campaign, get_campaign_state,
    get_messages, get_rpg_system, list_campaigns, list_rpg_systems, patch_character_data,
    update_campaign_name,
};
use commands::keychain_commands::{delete_api_key, get_api_key, save_api_key};
use commands::llm_commands::{
    extract_character_data, list_providers, request_gm_greeting, send_chat_message,
    suggest_campaign_name, validate_api_key,
};
use commands::settings_commands::{get_settings, open_user_systems_folder, save_settings};
use commands::tts_commands::synthesize_speech;

use keychain::keychain_service::KeychainService;
use persistence::campaign_repository::{CampaignRepository, MessageRepository};
use persistence::database::Database;
use persistence::settings_repository::SettingsRepository;
use services::campaign_service::CampaignService;
use services::llm_service::LlmService;
use services::rpg_system_registry::RpgSystemRegistry;
use services::settings_service::SettingsService;

pub mod commands;
pub mod domain;
pub mod keychain;
pub mod persistence;
pub mod providers;
pub mod services;
pub mod tools;

pub struct AppState {
    pub campaign_service: Arc<CampaignService>,
    pub llm_service: Arc<LlmService>,
    pub settings_service: Arc<SettingsService>,
    pub keychain_service: Arc<KeychainService>,
    pub rpg_registry: Arc<RpgSystemRegistry>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let app_state = build_app_state(app.handle())?;
            app.manage(app_state);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            list_campaigns,
            create_campaign,
            get_campaign,
            get_campaign_state,
            get_rpg_system,
            archive_campaign,
            delete_campaign,
            update_campaign_name,
            get_messages,
            send_chat_message,
            request_gm_greeting,
            extract_character_data,
            suggest_campaign_name,
            list_providers,
            validate_api_key,
            get_settings,
            save_settings,
            open_user_systems_folder,
            list_rpg_systems,
            patch_character_data,
            save_api_key,
            get_api_key,
            delete_api_key,
            synthesize_speech,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn build_app_state(app_handle: &tauri::AppHandle) -> anyhow::Result<AppState> {
    let database = open_application_database(app_handle)?;

    let campaign_repository = Arc::new(CampaignRepository::new(database.connection.clone()));
    let message_repository = Arc::new(MessageRepository::new(database.connection.clone()));
    let settings_repository = Arc::new(SettingsRepository::new(database.connection.clone()));

    let rpg_registry = Arc::new(load_rpg_registry(app_handle));

    let keychain_service = Arc::new(KeychainService::new(database.connection.clone()));

    let campaign_service = Arc::new(CampaignService::new(
        campaign_repository,
        message_repository,
        rpg_registry.clone(),
    ));

    let llm_service = Arc::new(LlmService::new(keychain_service.clone()));

    let settings_service = Arc::new(SettingsService::new(
        settings_repository,
        keychain_service.clone(),
    ));

    Ok(AppState {
        campaign_service,
        llm_service,
        settings_service,
        keychain_service,
        rpg_registry,
    })
}

fn open_application_database(app_handle: &tauri::AppHandle) -> anyhow::Result<Database> {
    let data_directory = app_handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| PathBuf::from("."));

    std::fs::create_dir_all(&data_directory)?;

    let db_path = data_directory.join("oracle.db");
    Database::open(db_path.to_str().unwrap_or("oracle.db"))
}

fn load_rpg_registry(app_handle: &tauri::AppHandle) -> RpgSystemRegistry {
    let bundled_dir = bundled_systems_directory(app_handle);
    let user_dir = user_systems_directory(app_handle);

    let dirs: Vec<&Path> = [&bundled_dir, &user_dir]
        .iter()
        .filter_map(|opt| opt.as_deref())
        .collect();

    RpgSystemRegistry::load_from_directories(&dirs).unwrap_or_else(|error| {
        eprintln!("Warning: could not load RPG systems: {error}");
        RpgSystemRegistry::load_from_directories(&[]).unwrap()
    })
}

fn bundled_systems_directory(app_handle: &tauri::AppHandle) -> Option<PathBuf> {
    let resource_path = app_handle
        .path()
        .resolve("rpg-systems", BaseDirectory::Resource)
        .ok()
        .filter(|p| p.exists());

    resource_path.or_else(|| {
        // Dev fallback: rpg-systems at project root
        let dev_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("rpg-systems");
        dev_path.exists().then_some(dev_path)
    })
}

fn user_systems_directory(app_handle: &tauri::AppHandle) -> Option<PathBuf> {
    let dir = app_handle
        .path()
        .app_data_dir()
        .ok()?
        .join("rpg-systems");

    // Create it so users can find it, but don't fail if we can't
    let _ = std::fs::create_dir_all(&dir);
    Some(dir)
}
