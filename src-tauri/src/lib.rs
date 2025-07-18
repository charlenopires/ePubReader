mod epub_parser;
mod translation;
mod file_manager;
mod config;
mod logger;

use std::collections::HashMap;
use tracing::{info, error, debug};

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
async fn translate_epub_with_images(epub_info: epub_parser::EpubInfo, target_lang: String, api_key: String) -> Result<String, String> {
    info!("Starting ePub translation with image preservation for: {}", epub_info.title);
    
    let mut translated_content = std::collections::HashMap::new();
    
    for chapter in &epub_info.chapters {
        info!("Translating chapter {}: {}", chapter.order + 1, chapter.title);
        
        match translation::translate_chapter_with_image_preservation(
            &chapter.raw_html,
            &target_lang,
            &api_key,
            &epub_info.images
        ).await {
            Ok(translated_html) => {
                // Extract text content from translated HTML for storage
                let translated_text = crate::epub_parser::extract_text_from_html_advanced(&translated_html);
                translated_content.insert(chapter.id.clone(), translated_text);
                debug!("Successfully translated chapter {}", chapter.id);
            }
            Err(e) => {
                error!("Failed to translate chapter {}: {}", chapter.id, e);
                return Err(format!("Translation failed for chapter {}: {}", chapter.id, e));
            }
        }
        
        // Small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }
    
    info!("All chapters translated successfully, saving ePub");
    
    match file_manager::save_translated_epub(epub_info, translated_content).await {
        Ok(saved_path) => {
            info!("Translated ePub saved successfully: {}", saved_path);
            Ok(saved_path)
        }
        Err(e) => {
            error!("Failed to save translated ePub: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn save_translated_epub(epub_info: epub_parser::EpubInfo, translated_content: HashMap<String, String>) -> Result<String, String> {
    info!("Saving translated ePub: {}", epub_info.title);
    match file_manager::save_translated_epub(epub_info, translated_content).await {
        Ok(saved_path) => {
            info!("Translated ePub saved successfully: {}", saved_path);
            Ok(saved_path)
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
    debug!("Loading saved book: {}", book_id);
    match file_manager::load_saved_book(&book_id).await {
        Ok(book_content) => {
            info!("Successfully loaded saved book: {}", book_content.book_info.title);
            Ok(book_content)
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
            error!("Failed to load settings: {}", e);
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

#[tauri::command]
async fn delete_saved_book(book_id: String) -> Result<(), String> {
    info!("Deleting saved book: {}", book_id);
    match file_manager::delete_saved_book(&book_id).await {
        Ok(_) => {
            info!("Book '{}' deleted successfully", book_id);
            Ok(())
        }
        Err(e) => {
            error!("Failed to delete book '{}': {}", book_id, e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn delete_all_saved_books() -> Result<(), String> {
    info!("Deleting all saved books");
    match file_manager::delete_all_saved_books().await {
        Ok(_) => {
            info!("All books deleted successfully");
            Ok(())
        }
        Err(e) => {
            error!("Failed to delete all books: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
async fn open_file_dialog(app: tauri::AppHandle) -> Result<Option<String>, String> {
    info!("Opening file dialog through backend command");
    
    use tauri_plugin_dialog::DialogExt;
    use std::sync::mpsc;
    
    // Create a channel to communicate between the callback and this async function
    let (tx, rx) = mpsc::channel();
    
    // Use the dialog plugin directly from the backend
    app.dialog()
        .file()
        .add_filter("ePub Files", &["epub"])
        .pick_file(move |file_path| {
            let result = match file_path {
                Some(path) => {
                    let path_str = path.to_string();
                    Some(path_str)
                }
                None => None,
            };
            let _ = tx.send(result);
        });
    
    // Wait for the callback to complete
    match rx.recv() {
        Ok(result) => {
            if let Some(path) = &result {
                info!("File selected: {}", path);
            } else {
                info!("No file selected");
            }
            Ok(result)
        }
        Err(e) => {
            let error_msg = format!("Failed to receive file dialog result: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // Initialize logging first
    if let Err(e) = logger::init_logging() {
        eprintln!("Failed to initialize logging: {}", e);
    }
    
    info!("Starting ePubReader application");
    
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![
            open_epub,
            translate_text,
            translate_epub_with_images,
            save_translated_epub,
            get_saved_books,
            load_saved_book,
            get_settings,
            save_settings,
            delete_saved_book,
            delete_all_saved_books,
            open_file_dialog
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
        .expect("error while running tauri application");
}