use tauri::State;
use std::sync::Mutex;
use crate::models::{Book, Language, AppSettings};
use crate::services::{BookManager, TranslationService};
use anyhow::Result;

pub type AppState = Mutex<AppSettings>;

#[tauri::command]
pub async fn check_ollama_status(settings: State<'_, AppState>) -> Result<bool, String> {
    let settings = settings.lock().unwrap().clone();
    let translation_service = TranslationService::new(
        settings.ollama_url,
        settings.ollama_model,
    );
    
    translation_service.check_ollama_status()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_supported_languages() -> Result<Vec<Language>, String> {
    Ok(Language::get_supported_languages())
}

#[tauri::command]
pub async fn get_all_books(settings: State<'_, AppState>) -> Result<Vec<Book>, String> {
    let settings = settings.lock().unwrap().clone();
    let book_manager = BookManager::new(settings)
        .map_err(|e| e.to_string())?;
    
    book_manager.get_all_books()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn add_book(
    epub_path: String,
    target_language: String,
    settings: State<'_, AppState>,
) -> Result<Book, String> {
    let settings = settings.lock().unwrap().clone();
    let book_manager = BookManager::new(settings)
        .map_err(|e| e.to_string())?;
    
    book_manager.add_book(
        std::path::Path::new(&epub_path),
        &target_language,
    )
    .await
    .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn translate_book(
    book_id: String,
    settings: State<'_, AppState>,
) -> Result<(), String> {
    let settings = settings.lock().unwrap().clone();
    let book_manager = BookManager::new(settings)
        .map_err(|e| e.to_string())?;
    
    book_manager.translate_book(&book_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_translation_progress(
    book_id: String,
    settings: State<'_, AppState>,
) -> Result<f64, String> {
    let settings = settings.lock().unwrap().clone();
    let book_manager = BookManager::new(settings)
        .map_err(|e| e.to_string())?;
    
    book_manager.get_translation_progress(&book_id)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_settings(
    new_settings: AppSettings,
    settings: State<'_, AppState>,
) -> Result<(), String> {
    let mut settings_guard = settings.lock().unwrap();
    *settings_guard = new_settings;
    
    // Save settings to file
    save_settings_to_file(&*settings_guard)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_settings(settings: State<'_, AppState>) -> Result<AppSettings, String> {
    Ok(settings.lock().unwrap().clone())
}

#[tauri::command]
pub async fn open_book_reader(book_id: String, settings: State<'_, AppState>) -> Result<String, String> {
    let settings = settings.lock().unwrap().clone();
    let book_manager = BookManager::new(settings)
        .map_err(|e| e.to_string())?;
    
    let book = book_manager.get_book_by_id(&book_id)
        .await
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Book not found".to_string())?;
    
    // Return the path to the HTML file
    Ok(book.html_path.to_string_lossy().to_string())
}

fn save_settings_to_file(settings: &AppSettings) -> Result<()> {
    let config_dir = dirs::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .join(".epubreader");
    
    std::fs::create_dir_all(&config_dir)?;
    
    let config_path = config_dir.join("settings.json");
    let json = serde_json::to_string_pretty(settings)?;
    std::fs::write(config_path, json)?;
    
    Ok(())
}

pub fn load_settings_from_file() -> AppSettings {
    let config_path = dirs::home_dir()
        .unwrap_or_default()
        .join(".epubreader")
        .join("settings.json");
    
    if config_path.exists() {
        if let Ok(json) = std::fs::read_to_string(&config_path) {
            if let Ok(settings) = serde_json::from_str::<AppSettings>(&json) {
                return settings;
            }
        }
    }
    
    AppSettings::default()
}