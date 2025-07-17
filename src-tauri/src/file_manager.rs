use crate::epub_parser::EpubInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedBook {
    pub id: String,
    pub title: String,
    pub author: String,
    pub original_language: String,
    pub translated_language: String,
    pub saved_date: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedBookContent {
    pub book_info: SavedBook,
    pub epub_info: EpubInfo,
    pub translated_content: HashMap<String, String>,
}

pub fn init_app_directory() -> Result<()> {
    let app_dir = get_app_directory()?;
    let ebooks_dir = app_dir.join("ebooks");
    
    fs::create_dir_all(&ebooks_dir)
        .map_err(|e| anyhow!("Failed to create app directory: {}", e))?;
    
    Ok(())
}

pub fn get_app_directory() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("Could not find home directory"))?;
    
    Ok(home_dir.join(".epubreader"))
}

pub async fn save_translated_epub(
    epub_info: EpubInfo,
    translated_content: HashMap<String, String>,
) -> Result<String> {
    let app_dir = get_app_directory()?;
    let ebooks_dir = app_dir.join("ebooks");
    
    // Generate unique ID for this book
    let book_id = Uuid::new_v4().to_string();
    let book_dir = ebooks_dir.join(&book_id);
    
    fs::create_dir_all(&book_dir)
        .map_err(|e| anyhow!("Failed to create book directory: {}", e))?;
    
    // Save book metadata
    let saved_book = SavedBook {
        id: book_id.clone(),
        title: epub_info.title.clone(),
        author: epub_info.author.clone(),
        original_language: epub_info.language.clone(),
        translated_language: "en".to_string(), // Default target language
        saved_date: chrono::Utc::now().to_rfc3339(),
        file_path: book_dir.to_string_lossy().to_string(),
    };
    
    // Save book content
    let book_content = SavedBookContent {
        book_info: saved_book.clone(),
        epub_info,
        translated_content,
    };
    
    let content_file = book_dir.join("content.json");
    let content_json = serde_json::to_string_pretty(&book_content)
        .map_err(|e| anyhow!("Failed to serialize book content: {}", e))?;
    
    fs::write(&content_file, content_json)
        .map_err(|e| anyhow!("Failed to write book content: {}", e))?;
    
    // Update index
    update_books_index(&saved_book).await?;
    
    Ok(book_id)
}

pub async fn get_saved_books() -> Result<Vec<SavedBook>> {
    let app_dir = get_app_directory()?;
    let index_file = app_dir.join("books_index.json");
    
    if !index_file.exists() {
        return Ok(Vec::new());
    }
    
    let index_content = fs::read_to_string(&index_file)
        .map_err(|e| anyhow!("Failed to read books index: {}", e))?;
    
    let books: Vec<SavedBook> = serde_json::from_str(&index_content)
        .map_err(|e| anyhow!("Failed to parse books index: {}", e))?;
    
    Ok(books)
}

pub async fn load_saved_book(book_id: &str) -> Result<SavedBookContent> {
    let app_dir = get_app_directory()?;
    let book_dir = app_dir.join("ebooks").join(book_id);
    let content_file = book_dir.join("content.json");
    
    if !content_file.exists() {
        return Err(anyhow!("Book not found: {}", book_id));
    }
    
    let content_json = fs::read_to_string(&content_file)
        .map_err(|e| anyhow!("Failed to read book content: {}", e))?;
    
    let book_content: SavedBookContent = serde_json::from_str(&content_json)
        .map_err(|e| anyhow!("Failed to parse book content: {}", e))?;
    
    Ok(book_content)
}

async fn update_books_index(new_book: &SavedBook) -> Result<()> {
    let app_dir = get_app_directory()?;
    let index_file = app_dir.join("books_index.json");
    
    let mut books = if index_file.exists() {
        let index_content = fs::read_to_string(&index_file)
            .map_err(|e| anyhow!("Failed to read books index: {}", e))?;
        
        serde_json::from_str::<Vec<SavedBook>>(&index_content)
            .unwrap_or_else(|_| Vec::new())
    } else {
        Vec::new()
    };
    
    // Remove existing book with same ID if exists
    books.retain(|book| book.id != new_book.id);
    
    // Add new book
    books.push(new_book.clone());
    
    // Sort by saved date (newest first)
    books.sort_by(|a, b| b.saved_date.cmp(&a.saved_date));
    
    let index_json = serde_json::to_string_pretty(&books)
        .map_err(|e| anyhow!("Failed to serialize books index: {}", e))?;
    
    fs::write(&index_file, index_json)
        .map_err(|e| anyhow!("Failed to write books index: {}", e))?;
    
    Ok(())
}