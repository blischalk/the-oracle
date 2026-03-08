use std::path::PathBuf;
use std::sync::Arc;

use tauri::Manager;

use commands::campaign_commands::{
    archive_campaign, create_campaign, get_campaign, get_messages, list_campaigns,
};
use commands::keychain_commands::{delete_api_key, get_api_key, save_api_key};
use commands::llm_commands::{list_providers, send_chat_message, validate_api_key};
use commands::settings_commands::{get_settings, save_settings};

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
            archive_campaign,
            get_messages,
            send_chat_message,
            list_providers,
            validate_api_key,
            get_settings,
            save_settings,
            save_api_key,
            get_api_key,
            delete_api_key,
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

    let keychain_service = Arc::new(KeychainService::new());

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
    let resource_directory = app_handle
        .path()
        .resource_dir()
        .unwrap_or_else(|_| PathBuf::from("."));

    let systems_directory = resource_directory.join("rpg-systems");

    RpgSystemRegistry::load(&systems_directory).unwrap_or_else(|error| {
        eprintln!("Warning: could not load RPG systems from {systems_directory:?}: {error}");
        RpgSystemRegistry::load(std::path::Path::new("/nonexistent")).unwrap()
    })
}
