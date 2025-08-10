// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod database;
mod epub_processor;
mod ollama_client;
mod models;

use commands::*;
use database::Database;
use ollama_client::OllamaClient;
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{info, error};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Mutex<Database>>,
    pub ollama: Arc<OllamaClient>,
}

#[tokio::main]
async fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("Starting ePub Reader Library");

    // Initialize database
    let db = match Database::new().await {
        Ok(db) => Arc::new(Mutex::new(db)),
        Err(e) => {
            error!("Failed to initialize database: {}", e);
            std::process::exit(1);
        }
    };

    // Initialize Ollama client
    let ollama = Arc::new(OllamaClient::new());

    let app_state = AppState { db, ollama };

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_http::init())
        .manage(app_state)
        .invoke_handler(tauri::generate_handler![
            check_ollama_status,
            get_books,
            add_book,
            get_book_content,
            translate_text,
            get_supported_languages,
            set_target_language,
            translate_book,
            generate_book_html,
            pull_translation_model
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}