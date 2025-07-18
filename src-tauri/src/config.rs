use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result, anyhow};
use crate::file_manager::get_app_directory;
use tracing::{info, error, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub target_language: String,
    pub google_api_key: String,
    pub auto_translate: bool,
    pub font_size: u32,
    pub theme: String,
}

impl Default for Settings {
    fn default() -> Self {
        // Try to get API key from environment variable as fallback
        let google_api_key = std::env::var("GOOGLE_TRANSLATE_API_KEY")
            .unwrap_or_else(|_| String::new());
        
        Self {
            target_language: "pt-BR".to_string(),
            google_api_key,
            auto_translate: true,
            font_size: 16,
            theme: "light".to_string(),
        }
    }
}

pub async fn load_settings() -> Result<Settings> {
    debug!("Loading application settings");
    let app_dir = get_app_directory()?;
    let settings_file = app_dir.join("settings.json");
    
    if !settings_file.exists() {
        info!("Settings file does not exist, creating default settings");
        let default_settings = Settings::default();
        save_settings(default_settings.clone()).await?;
        return Ok(default_settings);
    }
    
    debug!("Reading settings from: {}", settings_file.display());
    let settings_content = fs::read_to_string(&settings_file)
        .map_err(|e| {
            error!("Failed to read settings from '{}': {}", settings_file.display(), e);
            anyhow!("Failed to read settings: {}", e)
        })?;
    
    let mut settings: Settings = serde_json::from_str(&settings_content)
        .map_err(|e| {
            error!("Failed to parse settings file: {}", e);
            anyhow!("Failed to parse settings: {}", e)
        })?;
    
    // If API key is empty, try to get it from environment variable
    if settings.google_api_key.is_empty() {
        if let Ok(env_api_key) = std::env::var("GOOGLE_TRANSLATE_API_KEY") {
            if !env_api_key.is_empty() {
                debug!("Using Google Translate API key from environment variable");
                settings.google_api_key = env_api_key;
            }
        }
    }
    
    info!("Settings loaded successfully");
    Ok(settings)
}

pub async fn save_settings(settings: Settings) -> Result<()> {
    info!("Saving application settings");
    let app_dir = get_app_directory()?;
    let settings_file = app_dir.join("settings.json");
    
    // Ensure directory exists
    debug!("Ensuring app directory exists: {}", app_dir.display());
    fs::create_dir_all(&app_dir)
        .map_err(|e| {
            error!("Failed to create app directory '{}': {}", app_dir.display(), e);
            anyhow!("Failed to create app directory: {}", e)
        })?;
    
    let settings_json = serde_json::to_string_pretty(&settings)
        .map_err(|e| {
            error!("Failed to serialize settings: {}", e);
            anyhow!("Failed to serialize settings: {}", e)
        })?;
    
    debug!("Writing settings to: {} ({} bytes)", settings_file.display(), settings_json.len());
    fs::write(&settings_file, settings_json)
        .map_err(|e| {
            error!("Failed to write settings to '{}': {}", settings_file.display(), e);
            anyhow!("Failed to write settings: {}", e)
        })?;
    
    info!("Settings saved successfully");
    Ok(())
}