use serde::{Deserialize, Serialize};
use std::fs;
use anyhow::{Result, anyhow};
use crate::file_manager::get_app_directory;

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
        Self {
            target_language: "en".to_string(),
            google_api_key: String::new(),
            auto_translate: true,
            font_size: 16,
            theme: "light".to_string(),
        }
    }
}

pub async fn load_settings() -> Result<Settings> {
    let app_dir = get_app_directory()?;
    let settings_file = app_dir.join("settings.json");
    
    if !settings_file.exists() {
        let default_settings = Settings::default();
        save_settings(default_settings.clone()).await?;
        return Ok(default_settings);
    }
    
    let settings_content = fs::read_to_string(&settings_file)
        .map_err(|e| anyhow!("Failed to read settings: {}", e))?;
    
    let settings: Settings = serde_json::from_str(&settings_content)
        .map_err(|e| anyhow!("Failed to parse settings: {}", e))?;
    
    Ok(settings)
}

pub async fn save_settings(settings: Settings) -> Result<()> {
    let app_dir = get_app_directory()?;
    let settings_file = app_dir.join("settings.json");
    
    // Ensure directory exists
    fs::create_dir_all(&app_dir)
        .map_err(|e| anyhow!("Failed to create app directory: {}", e))?;
    
    let settings_json = serde_json::to_string_pretty(&settings)
        .map_err(|e| anyhow!("Failed to serialize settings: {}", e))?;
    
    fs::write(&settings_file, settings_json)
        .map_err(|e| anyhow!("Failed to write settings: {}", e))?;
    
    Ok(())
}