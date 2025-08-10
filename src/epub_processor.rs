use crate::models::*;
use crate::database::Database;
use anyhow::{Result, anyhow};
use epub::doc::EpubDoc;
use std::path::{Path, PathBuf};
use std::fs;
use uuid::Uuid;
use image::ImageFormat;
use base64::{Engine as _, engine::general_purpose};
use html2text::from_read;
use regex::Regex;
use tracing::{info, error, warn};

pub struct EpubProcessor {
    base_path: PathBuf,
}

impl EpubProcessor {
    pub fn new(base_path: PathBuf) -> Self {
        Self { base_path }
    }
    
    pub async fn process_epub(&self, epub_path: &str, db: &mut Database) -> Result<Book> {
        info!("Processing EPUB file: {}", epub_path);
        
        let mut doc = EpubDoc::new(epub_path)
            .map_err(|e| anyhow!("Failed to open EPUB: {}", e))?;
        
        // Extract metadata
        let title = doc.mdata("title")
            .unwrap_or_else(|| "Unknown Title".to_string());
        let author = doc.mdata("creator")
            .or_else(|| doc.mdata("author"))
            .unwrap_or_else(|| "Unknown Author".to_string());
        let language = doc.mdata("language")
            .unwrap_or_else(|| "en".to_string());
        
        info!("Book metadata - Title: {}, Author: {}, Language: {}", title, author, language);
        
        // Add book to database
        let book = db.add_book(title.clone(), author, epub_path.to_string(), language).await?;
        
        // Create book directory
        let book_dir = self.base_path.join(&title);
        let images_dir = book_dir.join("images");
        fs::create_dir_all(&images_dir)?;
        
        // Extract cover image
        if let Some((cover_data, _)) = doc.get_cover() {
            let cover_path = images_dir.join("cover.jpg");
            fs::write(&cover_path, cover_data)?;
            info!("Extracted cover image to: {}", cover_path.display());
        }
        
        // Process chapters
        let spine = doc.spine.clone();
        for (index, spine_item) in spine.iter().enumerate() {
            if let Some(path) = &spine_item.id {
                match doc.get_resource_str(path) {
                    Some((content, _)) => {
                        let chapter_title = self.extract_chapter_title(&content, index);
                        let cleaned_content = self.clean_html_content(&content);
                        
                        let chapter = db.add_chapter(
                            book.id.clone(),
                            chapter_title,
                            cleaned_content,
                            index as i32,
                        ).await?;
                        
                        // Extract and save images from this chapter
                        self.extract_chapter_images(&content, &chapter.id, &images_dir, &mut doc).await?;
                        
                        info!("Processed chapter {}: {}", index + 1, chapter.title);
                    }
                    None => {
                        warn!("Could not read content for spine item: {}", path);
                    }
                }
            }
        }
        
        info!("Successfully processed EPUB: {}", title);
        Ok(book)
    }
    
    fn extract_chapter_title(&self, html_content: &str, index: usize) -> String {
        // Try to extract title from HTML
        let title_regex = Regex::new(r"<h[1-6][^>]*>(.*?)</h[1-6]>").unwrap();
        if let Some(captures) = title_regex.captures(html_content) {
            let title = captures.get(1).unwrap().as_str();
            let clean_title = html2text::from_read(title.as_bytes(), 100);
            if !clean_title.trim().is_empty() {
                return clean_title.trim().to_string();
            }
        }
        
        // Fallback to generic chapter name
        format!("Chapter {}", index + 1)
    }
    
    fn clean_html_content(&self, html_content: &str) -> String {
        // Remove script and style tags
        let script_regex = Regex::new(r"<script[^>]*>.*?</script>").unwrap();
        let style_regex = Regex::new(r"<style[^>]*>.*?</style>").unwrap();
        
        let mut cleaned = script_regex.replace_all(html_content, "").to_string();
        cleaned = style_regex.replace_all(&cleaned, "").to_string();
        
        // Keep basic formatting tags but remove complex attributes
        let tag_regex = Regex::new(r#"<(\w+)[^>]*>"#).unwrap();
        cleaned = tag_regex.replace_all(&cleaned, "<$1>").to_string();
        
        cleaned
    }
    
    async fn extract_chapter_images(
        &self,
        html_content: &str,
        chapter_id: &str,
        images_dir: &Path,
        doc: &mut EpubDoc<std::io::BufReader<std::fs::File>>,
    ) -> Result<()> {
        let img_regex = Regex::new(r#"<img[^>]+src=["']([^"']+)["'][^>]*>"#).unwrap();
        
        for (paragraph_index, captures) in img_regex.captures_iter(html_content).enumerate() {
            if let Some(src_match) = captures.get(1) {
                let src = src_match.as_str();
                
                // Get image data from EPUB
                if let Some((image_data, media_type)) = doc.get_resource(src) {
                    let image_id = Uuid::new_v4().to_string();
                    let extension = self.get_image_extension(&media_type);
                    let local_filename = format!("{}_{}.{}", chapter_id, paragraph_index, extension);
                    let local_path = images_dir.join(&local_filename);
                    
                    // Save image file
                    fs::write(&local_path, &image_data)?;
                    
                    info!("Extracted image: {} -> {}", src, local_path.display());
                }
            }
        }
        
        Ok(())
    }
    
    fn get_image_extension(&self, media_type: &str) -> &str {
        match media_type {
            "image/jpeg" => "jpg",
            "image/png" => "png",
            "image/gif" => "gif",
            "image/webp" => "webp",
            "image/svg+xml" => "svg",
            _ => "jpg", // Default fallback
        }
    }
    
    pub async fn generate_html_book(&self, book_id: &str, db: &Database) -> Result<PathBuf> {
        let book = db.get_book_by_id(book_id).await?
            .ok_or_else(|| anyhow!("Book not found"))?;
        
        let chapters = db.get_book_chapters(book_id).await?;
        
        let book_dir = self.base_path.join(&book.title);
        let html_dir = book_dir.join("html");
        fs::create_dir_all(&html_dir)?;
        
        // Generate index.html (main book file)
        self.generate_index_html(&book, &chapters, &html_dir).await?;
        
        // Generate individual chapter HTML files
        for chapter in &chapters {
            self.generate_chapter_html(&book, chapter, &html_dir).await?;
        }
        
        // Copy CSS and JS files
        self.generate_book_assets(&html_dir).await?;
        
        Ok(html_dir.join("index.html"))
    }
    
    async fn generate_index_html(&self, book: &Book, chapters: &[Chapter], html_dir: &Path) -> Result<()> {
        let cover_path = self.base_path.join(&book.title).join("images").join("cover.jpg");
        let cover_exists = cover_path.exists();
        
        let html_content = format!(
            r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <div class="book-container">
        <div class="book-cover">
            {}
            <h1 class="book-title">{}</h1>
            <h2 class="book-author">{}</h2>
        </div>
        
        <div class="table-of-contents">
            <h3>Table of Contents</h3>
            <ul>
                {}
            </ul>
        </div>
        
        <div class="navigation">
            <button onclick="startReading()" class="start-reading-btn">Start Reading</button>
        </div>
    </div>
    
    <script src="book.js"></script>
</body>
</html>"#,
            book.language,
            book.title,
            if cover_exists {
                format!(r#"<img src="../images/cover.jpg" alt="Book Cover" class="cover-image">"#)
            } else {
                String::new()
            },
            book.title,
            book.author,
            chapters.iter()
                .map(|ch| format!(r#"<li><a href="chapter_{}.html">{}</a></li>"#, ch.order, ch.title))
                .collect::<Vec<_>>()
                .join("\n                ")
        );
        
        fs::write(html_dir.join("index.html"), html_content)?;
        Ok(())
    }
    
    async fn generate_chapter_html(&self, book: &Book, chapter: &Chapter, html_dir: &Path) -> Result<()> {
        let content = if let Some(translated) = &chapter.translated_content {
            translated
        } else {
            &chapter.content
        };
        
        let html_content = format!(
            r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{} - {}</title>
    <link rel="stylesheet" href="styles.css">
</head>
<body>
    <div class="chapter-container">
        <div class="chapter-header">
            <h1 class="chapter-title">{}</h1>
            <div class="chapter-navigation">
                <button onclick="previousChapter()" class="nav-btn">← Previous</button>
                <button onclick="goToIndex()" class="nav-btn">Contents</button>
                <button onclick="nextChapter()" class="nav-btn">Next →</button>
            </div>
        </div>
        
        <div class="chapter-content">
            {}
        </div>
        
        <div class="chapter-footer">
            <div class="progress-bar">
                <div class="progress-fill" style="width: {}%"></div>
            </div>
        </div>
    </div>
    
    <script src="book.js"></script>
    <script>
        window.currentChapter = {};
        window.bookTitle = "{}";
    </script>
</body>
</html>"#,
            book.language,
            chapter.title,
            book.title,
            chapter.title,
            content,
            (chapter.order as f64 / 10.0 * 100.0).min(100.0), // Simple progress calculation
            chapter.order,
            book.title
        );
        
        fs::write(html_dir.join(format!("chapter_{}.html", chapter.order)), html_content)?;
        Ok(())
    }
    
    async fn generate_book_assets(&self, html_dir: &Path) -> Result<()> {
        // Generate CSS
        let css_content = include_str!("../assets/styles.css");
        fs::write(html_dir.join("styles.css"), css_content)?;
        
        // Generate JavaScript
        let js_content = include_str!("../assets/book.js");
        fs::write(html_dir.join("book.js"), js_content)?;
        
        Ok(())
    }
}