use anyhow::{Result, anyhow};
use std::path::{Path, PathBuf};
use std::fs;
use crate::models::{Book, BookMetadata, Chapter, AppSettings};
use crate::services::{EpubParser, DatabaseService, TranslationService};

pub struct BookManager {
    books_dir: PathBuf,
    settings: AppSettings,
}

impl BookManager {
    pub fn new(settings: AppSettings) -> Result<Self> {
        let books_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?
            .join(".epubreader")
            .join("ebooks");

        fs::create_dir_all(&books_dir)?;

        Ok(Self {
            books_dir,
            settings,
        })
    }

    pub async fn add_book(&self, epub_path: &Path, target_language: &str) -> Result<Book> {
        // Parse metadata
        let metadata = EpubParser::parse_metadata(epub_path)?;
        
        // Create book instance
        let mut book = Book::new(
            metadata.title.clone(),
            metadata.author.clone(),
            metadata.language.clone(),
            target_language.to_string(),
            epub_path.to_path_buf(),
        );

        // Create book directory structure
        let book_dir = book.get_base_directory();
        fs::create_dir_all(&book_dir)?;
        fs::create_dir_all(&book.images_dir_path)?;

        // Save cover image if available
        if let Some(cover_data) = metadata.cover_image {
            let cover_path = book_dir.join("cover.jpg");
            fs::write(&cover_path, cover_data)?;
            book.cover_path = Some(cover_path);
        }

        // Extract chapters
        let chapters = EpubParser::extract_chapters(
            epub_path,
            &book.id,
            &book.images_dir_path,
        )?;

        book.total_chapters = chapters.len() as u32;

        // Initialize database for this book
        let db = DatabaseService::new(&book.translated_db_path).await?;
        
        // Save chapters to database
        for chapter in &chapters {
            db.insert_chapter(chapter).await?;
        }

        // Generate HTML file
        self.generate_book_html(&book, &chapters).await?;

        // Save book metadata
        self.save_book_metadata(&book).await?;

        Ok(book)
    }

    pub async fn get_all_books(&self) -> Result<Vec<Book>> {
        let mut books = Vec::new();
        
        if !self.books_dir.exists() {
            return Ok(books);
        }

        for entry in fs::read_dir(&self.books_dir)? {
            let entry = entry?;
            let path = entry.path();
            
            if path.is_dir() {
                let metadata_path = path.join("metadata.json");
                if metadata_path.exists() {
                    if let Ok(book) = self.load_book_metadata(&metadata_path).await {
                        books.push(book);
                    }
                }
            }
        }

        // Sort by added date (newest first)
        books.sort_by(|a, b| b.added_date.cmp(&a.added_date));

        Ok(books)
    }

    pub async fn translate_book(&self, book_id: &str) -> Result<()> {
        let book = self.get_book_by_id(book_id).await?
            .ok_or_else(|| anyhow!("Book not found"))?;

        let translation_service = TranslationService::new(
            self.settings.ollama_url.clone(),
            self.settings.ollama_model.clone(),
        );

        // Ensure Ollama model is available
        translation_service.ensure_model_available().await?;

        let db = DatabaseService::new(&book.translated_db_path).await?;
        let chapters = db.get_chapters_by_book(book_id).await?;

        for chapter in chapters {
            if !chapter.is_translated {
                tracing::info!("Translating chapter {}: {}", chapter.chapter_number, chapter.title);
                
                let translated_content = translation_service.translate_chapter_content(
                    &chapter.original_content,
                    &book.original_language,
                    &book.target_language,
                ).await?;

                db.update_chapter_translation(&chapter.id, &translated_content).await?;
            }
        }

        // Update book HTML with translations
        let updated_chapters = db.get_chapters_by_book(book_id).await?;
        self.generate_book_html(&book, &updated_chapters).await?;

        Ok(())
    }

    async fn generate_book_html(&self, book: &Book, chapters: &[Chapter]) -> Result<()> {
        let mut html_content = String::new();
        
        // HTML header
        html_content.push_str(&format!(
            r#"<!DOCTYPE html>
<html lang="{}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{}</title>
    <style>
        body {{
            font-family: '{}', serif;
            font-size: {}px;
            line-height: {};
            background-color: #1a1a1a;
            color: #e0e0e0;
            margin: 0;
            padding: 20px;
            max-width: 800px;
            margin: 0 auto;
        }}
        
        .cover {{
            text-align: center;
            padding: 50px 0;
            border-bottom: 2px solid #333;
            margin-bottom: 40px;
        }}
        
        .cover img {{
            max-width: 300px;
            max-height: 400px;
            border-radius: 8px;
            box-shadow: 0 4px 8px rgba(0,0,0,0.3);
        }}
        
        .cover h1 {{
            font-size: 2.5em;
            margin: 20px 0 10px 0;
            color: #fff;
        }}
        
        .cover .author {{
            font-size: 1.2em;
            color: #ccc;
            font-style: italic;
        }}
        
        .chapter {{
            margin-bottom: 60px;
            padding: 20px 0;
        }}
        
        .chapter h2 {{
            font-size: 1.8em;
            color: #fff;
            border-bottom: 1px solid #444;
            padding-bottom: 10px;
            margin-bottom: 30px;
        }}
        
        .chapter-content {{
            text-align: justify;
            text-indent: 2em;
        }}
        
        .chapter-content p {{
            margin-bottom: {}em;
        }}
        
        .chapter-image {{
            text-align: center;
            margin: 20px 0;
        }}
        
        .chapter-image img {{
            max-width: 100%;
            height: auto;
            border-radius: 4px;
        }}
        
        .navigation {{
            position: fixed;
            top: 20px;
            right: 20px;
            background: rgba(0,0,0,0.8);
            padding: 10px;
            border-radius: 8px;
        }}
        
        .navigation a {{
            color: #fff;
            text-decoration: none;
            margin: 0 10px;
            padding: 5px 10px;
            border-radius: 4px;
            background: #333;
        }}
        
        .navigation a:hover {{
            background: #555;
        }}
    </style>
</head>
<body>
"#,
            book.target_language,
            book.title,
            self.settings.reading_settings.font_family,
            self.settings.reading_settings.font_size,
            self.settings.reading_settings.line_height,
            self.settings.reading_settings.paragraph_spacing
        ));

        // Navigation
        html_content.push_str(r#"<div class="navigation">"#);
        html_content.push_str(r#"<a href="#cover">Cover</a>"#);
        for (i, chapter) in chapters.iter().enumerate() {
            html_content.push_str(&format!(
                r#"<a href="#chapter-{}">{}</a>"#,
                i + 1,
                if chapter.title.len() > 20 {
                    format!("{}...", &chapter.title[..20])
                } else {
                    chapter.title.clone()
                }
            ));
        }
        html_content.push_str("</div>");

        // Cover page
        html_content.push_str(r#"<div id="cover" class="cover">"#);
        if let Some(cover_path) = &book.cover_path {
            if cover_path.exists() {
                html_content.push_str(&format!(
                    r#"<img src="{}" alt="Book Cover">"#,
                    cover_path.file_name().unwrap().to_string_lossy()
                ));
            }
        }
        html_content.push_str(&format!(r#"<h1>{}</h1>"#, book.title));
        html_content.push_str(&format!(r#"<div class="author">{}</div>"#, book.author));
        html_content.push_str("</div>");

        // Chapters
        for (i, chapter) in chapters.iter().enumerate() {
            html_content.push_str(&format!(
                r#"<div id="chapter-{}" class="chapter">"#,
                i + 1
            ));
            html_content.push_str(&format!(r#"<h2>{}</h2>"#, chapter.title));
            
            let content = if let Some(ref translated) = chapter.translated_content {
                translated
            } else {
                &chapter.original_content
            };

            // Insert images at appropriate positions
            let content_with_images = self.insert_images_in_content(content, &chapter.images);
            
            html_content.push_str(&format!(
                r#"<div class="chapter-content">{}</div>"#,
                content_with_images.replace('\n', "</p><p>")
            ));
            html_content.push_str("</div>");
        }

        html_content.push_str("</body></html>");

        // Write HTML file
        fs::write(&book.html_path, html_content)?;

        Ok(())
    }

    fn insert_images_in_content(&self, content: &str, images: &[crate::models::ChapterImage]) -> String {
        let mut result = content.to_string();
        
        // Sort images by position (reverse order to maintain positions when inserting)
        let mut sorted_images = images.to_vec();
        sorted_images.sort_by(|a, b| b.position_in_text.cmp(&a.position_in_text));
        
        for image in sorted_images {
            let img_html = format!(
                r#"<div class="chapter-image"><img src="{}" alt="{}"></div>"#,
                image.local_path.file_name().unwrap().to_string_lossy(),
                image.alt_text.as_deref().unwrap_or("Chapter image")
            );
            
            // Insert image at the specified position
            if (image.position_in_text as usize) < result.len() {
                result.insert_str(image.position_in_text as usize, &img_html);
            } else {
                result.push_str(&img_html);
            }
        }
        
        result
    }

    async fn save_book_metadata(&self, book: &Book) -> Result<()> {
        let metadata_path = book.get_base_directory().join("metadata.json");
        let json = serde_json::to_string_pretty(book)?;
        fs::write(metadata_path, json)?;
        Ok(())
    }

    async fn load_book_metadata(&self, metadata_path: &Path) -> Result<Book> {
        let json = fs::read_to_string(metadata_path)?;
        let book: Book = serde_json::from_str(&json)?;
        Ok(book)
    }

    pub async fn get_book_by_id(&self, book_id: &str) -> Result<Option<Book>> {
        let books = self.get_all_books().await?;
        Ok(books.into_iter().find(|book| book.id == book_id))
    }

    pub async fn get_translation_progress(&self, book_id: &str) -> Result<f64> {
        let book = self.get_book_by_id(book_id).await?
            .ok_or_else(|| anyhow!("Book not found"))?;
        
        let db = DatabaseService::new(&book.translated_db_path).await?;
        db.get_translation_progress(book_id).await
    }
}