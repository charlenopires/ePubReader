use crate::epub_parser::EpubInfo;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, anyhow};
use uuid::Uuid;
use tracing::{info, error, warn, debug};
use std::io::Write;

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
    info!("Initializing app directory: {}", app_dir.display());
    let ebooks_dir = app_dir.join("ebooks");
    
    fs::create_dir_all(&ebooks_dir)
        .map_err(|e| {
            error!("Failed to create app directory '{}': {}", ebooks_dir.display(), e);
            anyhow!("Failed to create app directory: {}", e)
        })?;
    
    info!("App directory initialized successfully");
    Ok(())
}

pub fn get_app_directory() -> Result<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| {
            error!("Could not find home directory");
            anyhow!("Could not find home directory")
        })?;
    
    let app_dir = home_dir.join(".epubreader");
    debug!("App directory path: {}", app_dir.display());
    Ok(app_dir)
}

pub async fn save_translated_epub(
    mut epub_info: EpubInfo,
    translated_content: HashMap<String, String>,
) -> Result<String> {
    info!("Saving translated ePub: '{}' by {}", epub_info.title, epub_info.author);
    let app_dir = get_app_directory()?;
    let ebooks_dir = app_dir.join("ebooks");
    
    // Generate unique ID for this book
    let book_id = Uuid::new_v4().to_string();
    debug!("Generated book ID: {}", book_id);
    let book_dir = ebooks_dir.join(&book_id);
    
    debug!("Creating book directory: {}", book_dir.display());
    fs::create_dir_all(&book_dir)
        .map_err(|e| {
            error!("Failed to create book directory '{}': {}", book_dir.display(), e);
            anyhow!("Failed to create book directory: {}", e)
        })?;
    
    // Update chapters with translated content
    for chapter in &mut epub_info.chapters {
        if let Some(translated) = translated_content.get(&chapter.id) {
            debug!("Updating chapter {} with translated content", chapter.id);
            chapter.content = translated.clone();
        }
    }
    
    // Update metadata to reflect translation
    epub_info.language = "pt-BR".to_string();
    epub_info.metadata.insert("language".to_string(), "pt-BR".to_string());
    epub_info.title = format!("{} (Traduzido)", epub_info.title);
    epub_info.metadata.insert("title".to_string(), epub_info.title.clone());
    
    // Create a simple ePub structure
    let epub_file_path = book_dir.join("translated.epub");
    create_epub_file(&epub_info, &epub_file_path).await?;
    
    // Save book metadata
    let saved_book = SavedBook {
        id: book_id.clone(),
        title: epub_info.title.clone(),
        author: epub_info.author.clone(),
        original_language: "en".to_string(), // Original was English
        translated_language: "pt-BR".to_string(), // Portuguese (Brazil) target language
        saved_date: chrono::Utc::now().to_rfc3339(),
        file_path: epub_file_path.to_string_lossy().to_string(),
    };
    
    // Save book content as JSON for compatibility
    let book_content = SavedBookContent {
        book_info: saved_book.clone(),
        epub_info,
        translated_content,
    };
    
    let content_file = book_dir.join("content.json");
    debug!("Serializing book content to: {}", content_file.display());
    let content_json = serde_json::to_string_pretty(&book_content)
        .map_err(|e| {
            error!("Failed to serialize book content: {}", e);
            anyhow!("Failed to serialize book content: {}", e)
        })?;
    
    debug!("Writing {} bytes to content file", content_json.len());
    fs::write(&content_file, content_json)
        .map_err(|e| {
            error!("Failed to write book content to '{}': {}", content_file.display(), e);
            anyhow!("Failed to write book content: {}", e)
        })?;
    
    // Update index
    debug!("Updating books index for new book");
    update_books_index(&saved_book).await?;
    
    info!("Successfully saved translated ePub with ID: {}", book_id);
    Ok(book_id)
}

pub async fn get_saved_books() -> Result<Vec<SavedBook>> {
    debug!("Loading saved books list");
    let app_dir = get_app_directory()?;
    let index_file = app_dir.join("books_index.json");
    
    if !index_file.exists() {
        debug!("Books index file does not exist, returning empty list");
        return Ok(Vec::new());
    }
    
    debug!("Reading books index from: {}", index_file.display());
    let index_content = fs::read_to_string(&index_file)
        .map_err(|e| {
            error!("Failed to read books index from '{}': {}", index_file.display(), e);
            anyhow!("Failed to read books index: {}", e)
        })?;
    
    let books: Vec<SavedBook> = serde_json::from_str(&index_content)
        .map_err(|e| {
            error!("Failed to parse books index: {}", e);
            anyhow!("Failed to parse books index: {}", e)
        })?;
    
    info!("Loaded {} saved books from index", books.len());
    Ok(books)
}

pub async fn load_saved_book(book_id: &str) -> Result<SavedBookContent> {
    info!("Loading saved book with ID: {}", book_id);
    let app_dir = get_app_directory()?;
    let book_dir = app_dir.join("ebooks").join(book_id);
    let content_file = book_dir.join("content.json");
    
    if !content_file.exists() {
        error!("Book content file not found: {}", content_file.display());
        return Err(anyhow!("Book not found: {}", book_id));
    }
    
    debug!("Reading book content from: {}", content_file.display());
    let content_json = fs::read_to_string(&content_file)
        .map_err(|e| {
            error!("Failed to read book content from '{}': {}", content_file.display(), e);
            anyhow!("Failed to read book content: {}", e)
        })?;
    
    let book_content: SavedBookContent = serde_json::from_str(&content_json)
        .map_err(|e| {
            error!("Failed to parse book content for '{}': {}", book_id, e);
            anyhow!("Failed to parse book content: {}", e)
        })?;
    
    info!("Successfully loaded saved book: '{}'", book_content.epub_info.title);
    Ok(book_content)
}

async fn update_books_index(new_book: &SavedBook) -> Result<()> {
    debug!("Updating books index with new book: {}", new_book.title);
    let app_dir = get_app_directory()?;
    let index_file = app_dir.join("books_index.json");
    
    let mut books = if index_file.exists() {
        debug!("Reading existing books index");
        let index_content = fs::read_to_string(&index_file)
            .map_err(|e| {
                error!("Failed to read books index: {}", e);
                anyhow!("Failed to read books index: {}", e)
            })?;
        
        serde_json::from_str::<Vec<SavedBook>>(&index_content)
            .unwrap_or_else(|e| {
                warn!("Failed to parse existing books index, starting fresh: {}", e);
                Vec::new()
            })
    } else {
        debug!("Creating new books index");
        Vec::new()
    };
    
    // Remove existing book with same ID if exists
    let original_count = books.len();
    books.retain(|book| book.id != new_book.id);
    if books.len() < original_count {
        debug!("Removed existing book entry for ID: {}", new_book.id);
    }
    
    // Add new book
    books.push(new_book.clone());
    debug!("Added new book to index, total books: {}", books.len());
    
    // Sort by saved date (newest first)
    books.sort_by(|a, b| b.saved_date.cmp(&a.saved_date));
    
    let index_json = serde_json::to_string_pretty(&books)
        .map_err(|e| {
            error!("Failed to serialize books index: {}", e);
            anyhow!("Failed to serialize books index: {}", e)
        })?;
    
    debug!("Writing updated books index ({} bytes)", index_json.len());
    fs::write(&index_file, index_json)
        .map_err(|e| {
            error!("Failed to write books index to '{}': {}", index_file.display(), e);
            anyhow!("Failed to write books index: {}", e)
        })?;
    
    info!("Books index updated successfully");
    Ok(())
}

async fn create_epub_file(epub_info: &EpubInfo, output_path: &PathBuf) -> Result<()> {
    info!("Creating ePub file: {}", output_path.display());
    
    // Create temporary directory for ePub structure
    let temp_dir = std::env::temp_dir().join(format!("epub_temp_{}", Uuid::new_v4()));
    fs::create_dir_all(&temp_dir)?;
    
    // Create standard ePub directory structure
    let meta_inf_dir = temp_dir.join("META-INF");
    let oebps_dir = temp_dir.join("OEBPS");
    let text_dir = oebps_dir.join("Text");
    let images_dir = oebps_dir.join("Images");
    
    fs::create_dir_all(&meta_inf_dir)?;
    fs::create_dir_all(&text_dir)?;
    fs::create_dir_all(&images_dir)?;
    
    // Create mimetype file
    fs::write(temp_dir.join("mimetype"), "application/epub+zip")?;
    
    // Create container.xml
    let container_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<container version="1.0" xmlns="urn:oasis:names:tc:opendocument:xmlns:container">
  <rootfiles>
    <rootfile full-path="OEBPS/content.opf" media-type="application/oebps-package+xml"/>
  </rootfiles>
</container>"#;
    fs::write(meta_inf_dir.join("container.xml"), container_xml)?;
    
    // Create content.opf (package document)
    let mut manifest_items = String::new();
    let mut spine_items = String::new();
    
    // Add chapters to manifest and spine
    for (i, chapter) in epub_info.chapters.iter().enumerate() {
        let file_name = format!("chapter_{:03}.xhtml", i + 1);
        manifest_items.push_str(&format!(
            r#"    <item id="chapter_{}" href="Text/{}" media-type="application/xhtml+xml"/>
"#, i + 1, file_name
        ));
        spine_items.push_str(&format!(
            r#"    <itemref idref="chapter_{}"/>
"#, i + 1
        ));
        
        // Create chapter XHTML file
        let chapter_content = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml">
<head>
    <title>{}</title>
    <meta charset="UTF-8"/>
</head>
<body>
    <h1>{}</h1>
    <div>
        {}
    </div>
</body>
</html>"#, chapter.title, chapter.title, chapter.content);
        
        fs::write(text_dir.join(&file_name), chapter_content)?;
    }
    
    let content_opf = format!(r#"<?xml version="1.0" encoding="UTF-8"?>
<package version="3.0" xmlns="http://www.idpf.org/2007/opf" unique-identifier="uid">
  <metadata xmlns:dc="http://purl.org/dc/elements/1.1/">
    <dc:identifier id="uid">{}</dc:identifier>
    <dc:title>{}</dc:title>
    <dc:creator>{}</dc:creator>
    <dc:language>{}</dc:language>
    <meta property="dcterms:modified">{}</meta>
  </metadata>
  <manifest>
{}  </manifest>
  <spine>
{}  </spine>
</package>"#, 
        Uuid::new_v4().to_string(),
        escape_xml(&epub_info.title),
        escape_xml(&epub_info.author),
        epub_info.language,
        chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ"),
        manifest_items,
        spine_items
    );
    
    fs::write(oebps_dir.join("content.opf"), content_opf)?;
    
    // Create the ZIP file (ePub is essentially a ZIP)
    create_zip_from_directory(&temp_dir, output_path)?;
    
    // Clean up temporary directory
    if let Err(e) = fs::remove_dir_all(&temp_dir) {
        warn!("Failed to clean up temporary directory: {}", e);
    }
    
    info!("Successfully created ePub file: {}", output_path.display());
    Ok(())
}

fn escape_xml(input: &str) -> String {
    input
        .replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace("\"", "&quot;")
        .replace("'", "&apos;")
}

fn create_zip_from_directory(source_dir: &PathBuf, output_path: &PathBuf) -> Result<()> {
    use zip::write::{FileOptions, ZipWriter};
    
    let file = fs::File::create(output_path)?;
    let mut zip = ZipWriter::new(file);
    let options = FileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    
    add_dir_to_zip(&mut zip, source_dir, source_dir, &options)?;
    
    zip.finish()?;
    Ok(())
}

fn add_dir_to_zip<W: std::io::Write + std::io::Seek>(
    zip: &mut zip::ZipWriter<W>,
    dir: &PathBuf,
    base_dir: &PathBuf,
    options: &zip::write::FileOptions,
) -> Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let relative_path = path.strip_prefix(base_dir)?;
        
        if path.is_dir() {
            // Add directory (with trailing slash)
            let dir_name = format!("{}/", relative_path.to_string_lossy());
            zip.start_file(dir_name, *options)?;
            add_dir_to_zip(zip, &path, base_dir, options)?;
        } else {
            // Add file
            let file_name = relative_path.to_string_lossy().to_string();
            zip.start_file(file_name, *options)?;
            let file_content = fs::read(&path)?;
            zip.write_all(&file_content)?;
        }
    }
    Ok(())
}

pub async fn delete_saved_book(book_id: &str) -> Result<()> {
    info!("Deleting saved book with ID: {}", book_id);
    
    let app_dir = get_app_directory()?;
    let ebooks_dir = app_dir.join("ebooks");
    let book_dir = ebooks_dir.join(book_id);
    
    // Check if book directory exists
    if !book_dir.exists() {
        warn!("Book directory does not exist: {}", book_dir.display());
        return Err(anyhow!("Book with ID '{}' not found", book_id));
    }
    
    // Remove the entire book directory
    debug!("Removing book directory: {}", book_dir.display());
    fs::remove_dir_all(&book_dir)
        .map_err(|e| {
            error!("Failed to remove book directory '{}': {}", book_dir.display(), e);
            anyhow!("Failed to delete book files: {}", e)
        })?;
    
    info!("Successfully deleted book directory: {}", book_dir.display());
    
    // Update the books index by removing the book entry
    let index_file = app_dir.join("books_index.json");
    
    if index_file.exists() {
        debug!("Updating books index after deletion");
        
        // Load existing books
        let index_content = fs::read_to_string(&index_file)
            .map_err(|e| {
                error!("Failed to read books index '{}': {}", index_file.display(), e);
                anyhow!("Failed to read books index: {}", e)
            })?;
        
        let mut books: Vec<SavedBook> = serde_json::from_str(&index_content)
            .unwrap_or_else(|e| {
                warn!("Failed to parse books index, starting fresh: {}", e);
                Vec::new()
            });
        
        // Remove the deleted book from the index
        let original_count = books.len();
        books.retain(|book| book.id != book_id);
        
        if books.len() < original_count {
            debug!("Removed book '{}' from index", book_id);
            
            // Save updated index
            let index_json = serde_json::to_string_pretty(&books)
                .map_err(|e| {
                    error!("Failed to serialize updated books index: {}", e);
                    anyhow!("Failed to serialize books index: {}", e)
                })?;
            
            fs::write(&index_file, index_json)
                .map_err(|e| {
                    error!("Failed to write updated books index to '{}': {}", index_file.display(), e);
                    anyhow!("Failed to write books index: {}", e)
                })?;
            
            info!("Books index updated successfully after deletion");
        } else {
            warn!("Book '{}' was not found in the index", book_id);
        }
    } else {
        debug!("Books index file does not exist, no need to update");
    }
    
    info!("Successfully deleted saved book: {}", book_id);
    Ok(())
}

pub async fn delete_all_saved_books() -> Result<()> {
    info!("Deleting all saved books");
    
    let app_dir = get_app_directory()?;
    let ebooks_dir = app_dir.join("ebooks");
    
    if !ebooks_dir.exists() {
        debug!("Ebooks directory does not exist, nothing to delete");
        return Ok(());
    }
    
    // Get list of all book directories
    let mut deleted_count = 0;
    let entries = fs::read_dir(&ebooks_dir)
        .map_err(|e| {
            error!("Failed to read ebooks directory '{}': {}", ebooks_dir.display(), e);
            anyhow!("Failed to read ebooks directory: {}", e)
        })?;
    
    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_dir() {
            debug!("Removing book directory: {}", path.display());
            fs::remove_dir_all(&path)
                .map_err(|e| {
                    error!("Failed to remove directory '{}': {}", path.display(), e);
                    anyhow!("Failed to remove directory: {}", e)
                })?;
            deleted_count += 1;
        }
    }
    
    // Clear the books index
    let index_file = app_dir.join("books_index.json");
    if index_file.exists() {
        debug!("Clearing books index");
        let empty_books: Vec<SavedBook> = Vec::new();
        let index_json = serde_json::to_string_pretty(&empty_books)
            .map_err(|e| {
                error!("Failed to serialize empty books index: {}", e);
                anyhow!("Failed to serialize books index: {}", e)
            })?;
        
        fs::write(&index_file, index_json)
            .map_err(|e| {
                error!("Failed to write empty books index to '{}': {}", index_file.display(), e);
                anyhow!("Failed to write books index: {}", e)
            })?;
        
        info!("Books index cleared successfully");
    }
    
    info!("Successfully deleted {} saved books", deleted_count);
    Ok(())
}