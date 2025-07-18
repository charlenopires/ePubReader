#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod epub_parser;
mod translation;
mod file_manager;
mod config;
mod logger;

use std::collections::HashMap;
use tracing::{info, error, warn, debug};
use logger::init_logging;

#[tauri::command]
async fn open_epub(path: String) -> Result<epub_parser::EpubInfo, String> {
    info!("Opening ePub file: {}", path);
    match epub_parser::parse_epub(&path).await {
        Ok(result) => {
            info!("Successfully parsed ePub: {}", result.title);
            Ok(result)
        }
        Err(e) => {
            error!("Failed to open ePub file '{}': {}", path, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn translate_text(text: String, target_lang: String, api_key: String) -> Result<String, String> {
    debug!("Translating text to {}: {} chars", target_lang, text.len());
    match translation::translate_preserving_code_and_images(&text, &target_lang, &api_key).await {
        Ok(result) => {
            info!("Smart translation successful: {} -> {} ({} chars)", target_lang, target_lang, result.len());
            Ok(result)
        }
        Err(e) => {
            error!("Smart translation failed for language '{}': {}", target_lang, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn save_translated_epub(epub_info: epub_parser::EpubInfo, translated_content: HashMap<String, String>) -> Result<String, String> {
    info!("Saving translated ePub: {}", epub_info.title);
    match file_manager::save_translated_epub(epub_info, translated_content).await {
        Ok(book_id) => {
            info!("Successfully saved translated ePub with ID: {}", book_id);
            Ok(book_id)
        }
        Err(e) => {
            error!("Failed to save translated ePub: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn get_saved_books() -> Result<Vec<file_manager::SavedBook>, String> {
    debug!("Loading saved books list");
    match file_manager::get_saved_books().await {
        Ok(books) => {
            info!("Loaded {} saved books", books.len());
            Ok(books)
        }
        Err(e) => {
            error!("Failed to load saved books: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn load_saved_book(book_id: String) -> Result<file_manager::SavedBookContent, String> {
    info!("Loading saved book: {}", book_id);
    match file_manager::load_saved_book(&book_id).await {
        Ok(book) => {
            info!("Successfully loaded saved book: {}", book.book_info.title);
            Ok(book)
        }
        Err(e) => {
            error!("Failed to load saved book '{}': {}", book_id, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn get_settings() -> Result<config::Settings, String> {
    debug!("Loading application settings");
    match config::load_settings().await {
        Ok(settings) => {
            info!("Settings loaded successfully");
            Ok(settings)
        }
        Err(e) => {
            warn!("Failed to load settings, using defaults: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn save_settings(settings: config::Settings) -> Result<(), String> {
    info!("Saving application settings");
    match config::save_settings(settings).await {
        Ok(_) => {
            info!("Settings saved successfully");
            Ok(())
        }
        Err(e) => {
            error!("Failed to save settings: {}", e);
            Err(e.to_string())
        }
    }
}

fn main() {
    // Load environment variables from .env file
    if let Err(e) = dotenvy::dotenv() {
        eprintln!("Warning: Could not load .env file: {}", e);
    }
    
    // Initialize logging first
    if let Err(e) = init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
    }
    
    info!("Starting ePubReader application");
    
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
        .setup(|_app| {
            info!("Initializing application setup");
            
            // Initialize app directory
            match file_manager::init_app_directory() {
                Ok(_) => info!("App directory initialized successfully"),
                Err(e) => error!("Failed to initialize app directory: {}", e),
            }
            
            info!("Application setup completed");
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|e| {
            error!("Critical error while running Tauri application: {}", e);
            panic!("Application failed to start: {}", e);
        });
}