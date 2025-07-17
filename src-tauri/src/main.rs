#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod epub_parser;
mod translation;
mod file_manager;
mod config;

use std::collections::HashMap;

#[tauri::command]
async fn open_epub(path: String) -> Result<epub_parser::EpubInfo, String> {
    epub_parser::parse_epub(&path).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn translate_text(text: String, target_lang: String, api_key: String) -> Result<String, String> {
    translation::translate_with_google(&text, &target_lang, &api_key).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_translated_epub(epub_info: epub_parser::EpubInfo, translated_content: HashMap<String, String>) -> Result<String, String> {
    file_manager::save_translated_epub(epub_info, translated_content).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_saved_books() -> Result<Vec<file_manager::SavedBook>, String> {
    file_manager::get_saved_books().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn load_saved_book(book_id: String) -> Result<file_manager::SavedBookContent, String> {
    file_manager::load_saved_book(&book_id).await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn get_settings() -> Result<config::Settings, String> {
    config::load_settings().await.map_err(|e| e.to_string())
}

#[tauri::command]
async fn save_settings(settings: config::Settings) -> Result<(), String> {
    config::save_settings(settings).await.map_err(|e| e.to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            open_epub,
            translate_text,
            save_translated_epub,
            get_saved_books,
            load_saved_book,
            get_settings,
            save_settings
        ])
        .setup(|app| {
            // Initialize app directory
            let _ = file_manager::init_app_directory();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}