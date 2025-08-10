use crate::{AppState, models::*, epub_processor::EpubProcessor};
use tauri::State;
use anyhow::Result;
use tracing::{info, error};

#[tauri::command]
pub async fn check_ollama_status(state: State<'_, AppState>) -> Result<OllamaStatus, String> {
    info!("Checking Ollama status");
    
    match state.ollama.check_status().await {
        Ok(status) => {
            info!("Ollama status: running={}, models={}", status.is_running, status.available_models.len());
            Ok(status)
        }
        Err(e) => {
            error!("Failed to check Ollama status: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_books(state: State<'_, AppState>) -> Result<Vec<Book>, String> {
    info!("Getting all books");
    
    let db = state.db.lock().await;
    match db.get_all_books().await {
        Ok(books) => {
            info!("Retrieved {} books", books.len());
            Ok(books)
        }
        Err(e) => {
            error!("Failed to get books: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn add_book(
    file_path: String,
    state: State<'_, AppState>,
) -> Result<Book, String> {
    info!("Adding book from path: {}", file_path);
    
    let mut db = state.db.lock().await;
    let base_path = db.get_base_path().clone();
    drop(db); // Release the lock before processing
    
    let processor = EpubProcessor::new(base_path);
    
    match processor.process_epub(&file_path, &mut *state.db.lock().await).await {
        Ok(book) => {
            info!("Successfully added book: {}", book.title);
            Ok(book)
        }
        Err(e) => {
            error!("Failed to add book: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_book_content(
    book_id: String,
    state: State<'_, AppState>,
) -> Result<BookContent, String> {
    info!("Getting content for book: {}", book_id);
    
    let db = state.db.lock().await;
    
    let book = match db.get_book_by_id(&book_id).await {
        Ok(Some(book)) => book,
        Ok(None) => return Err("Book not found".to_string()),
        Err(e) => {
            error!("Failed to get book: {}", e);
            return Err(e.to_string());
        }
    };
    
    let chapters = match db.get_book_chapters(&book_id).await {
        Ok(chapters) => chapters,
        Err(e) => {
            error!("Failed to get chapters: {}", e);
            return Err(e.to_string());
        }
    };
    
    let content = BookContent {
        book,
        chapters,
        current_chapter: 0,
    };
    
    info!("Retrieved book content with {} chapters", content.chapters.len());
    Ok(content)
}

#[tauri::command]
pub async fn translate_text(
    text: String,
    source_language: String,
    target_language: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Translating text from {} to {}", source_language, target_language);
    
    let request = TranslationRequest {
        text,
        source_language,
        target_language,
    };
    
    match state.ollama.translate_text(request).await {
        Ok(translated) => {
            info!("Translation completed successfully");
            Ok(translated)
        }
        Err(e) => {
            error!("Translation failed: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn get_supported_languages() -> Result<Vec<Language>, String> {
    info!("Getting supported languages");
    Ok(crate::ollama_client::get_supported_languages())
}

#[tauri::command]
pub async fn set_target_language(
    book_id: String,
    language_code: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Setting target language for book {}: {}", book_id, language_code);
    
    // This would update the book's target language in the database
    // For now, we'll just log it
    info!("Target language set successfully");
    Ok(())
}

#[tauri::command]
pub async fn translate_book(
    book_id: String,
    target_language: String,
    state: State<'_, AppState>,
) -> Result<(), String> {
    info!("Starting translation of book {} to {}", book_id, target_language);
    
    let mut db = state.db.lock().await;
    
    // Update book status to in progress
    if let Err(e) = db.update_book_translation_status(&book_id, TranslationStatus::InProgress).await {
        error!("Failed to update book status: {}", e);
        return Err(e.to_string());
    }
    
    // Get book and chapters
    let book = match db.get_book_by_id(&book_id).await {
        Ok(Some(book)) => book,
        Ok(None) => return Err("Book not found".to_string()),
        Err(e) => return Err(e.to_string()),
    };
    
    let chapters = match db.get_book_chapters(&book_id).await {
        Ok(chapters) => chapters,
        Err(e) => return Err(e.to_string()),
    };
    
    drop(db); // Release lock for translation
    
    // Translate each chapter
    for chapter in chapters {
        if chapter.translated_content.is_some() {
            continue; // Skip already translated chapters
        }
        
        let request = TranslationRequest {
            text: chapter.content.clone(),
            source_language: book.language.clone(),
            target_language: target_language.clone(),
        };
        
        match state.ollama.translate_text(request).await {
            Ok(translated) => {
                let mut db = state.db.lock().await;
                if let Err(e) = db.update_chapter_translation(&chapter.id, translated).await {
                    error!("Failed to save translation for chapter {}: {}", chapter.id, e);
                }
            }
            Err(e) => {
                error!("Failed to translate chapter {}: {}", chapter.id, e);
                let mut db = state.db.lock().await;
                let _ = db.update_book_translation_status(&book_id, TranslationStatus::Failed).await;
                return Err(format!("Translation failed: {}", e));
            }
        }
    }
    
    // Update book status to completed
    let mut db = state.db.lock().await;
    if let Err(e) = db.update_book_translation_status(&book_id, TranslationStatus::Completed).await {
        error!("Failed to update book status to completed: {}", e);
        return Err(e.to_string());
    }
    
    info!("Book translation completed successfully");
    Ok(())
}

#[tauri::command]
pub async fn generate_book_html(
    book_id: String,
    state: State<'_, AppState>,
) -> Result<String, String> {
    info!("Generating HTML for book: {}", book_id);
    
    let db = state.db.lock().await;
    let base_path = db.get_base_path().clone();
    
    let processor = EpubProcessor::new(base_path);
    
    match processor.generate_html_book(&book_id, &*db).await {
        Ok(html_path) => {
            info!("HTML generated at: {}", html_path.display());
            Ok(html_path.to_string_lossy().to_string())
        }
        Err(e) => {
            error!("Failed to generate HTML: {}", e);
            Err(e.to_string())
        }
    }
}

#[tauri::command]
pub async fn pull_translation_model(state: State<'_, AppState>) -> Result<String, String> {
    info!("Pulling recommended translation model");
    
    match state.ollama.pull_recommended_model().await {
        Ok(model_name) => {
            info!("Successfully pulled model: {}", model_name);
            Ok(model_name)
        }
        Err(e) => {
            error!("Failed to pull model: {}", e);
            Err(e.to_string())
        }
    }
}