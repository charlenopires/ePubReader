use crate::models::*;
use anyhow::{Result, anyhow};
use sqlx::{SqlitePool, Row};
use std::path::PathBuf;
use uuid::Uuid;
use chrono::Utc;
use tracing::{info, error};

pub struct Database {
    pool: SqlitePool,
    base_path: PathBuf,
}

impl Database {
    pub async fn new() -> Result<Self> {
        let home_dir = dirs::home_dir()
            .ok_or_else(|| anyhow!("Could not find home directory"))?;
        
        let base_path = home_dir.join(".epubreader").join("ebooks");
        std::fs::create_dir_all(&base_path)?;
        
        let db_path = base_path.join("library.db");
        let database_url = format!("sqlite:{}", db_path.display());
        
        info!("Connecting to database at: {}", database_url);
        
        let pool = SqlitePool::connect(&database_url).await?;
        
        let db = Self { pool, base_path };
        db.initialize_schema().await?;
        
        Ok(db)
    }
    
    async fn initialize_schema(&self) -> Result<()> {
        info!("Initializing database schema");
        
        // Books table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS books (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                author TEXT NOT NULL,
                cover_path TEXT,
                original_path TEXT NOT NULL,
                language TEXT NOT NULL,
                target_language TEXT,
                translation_status TEXT NOT NULL DEFAULT 'NotStarted',
                added_at TEXT NOT NULL,
                last_read TEXT,
                progress REAL NOT NULL DEFAULT 0.0
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        // Chapters table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS chapters (
                id TEXT PRIMARY KEY,
                book_id TEXT NOT NULL,
                title TEXT NOT NULL,
                content TEXT NOT NULL,
                translated_content TEXT,
                chapter_order INTEGER NOT NULL,
                FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        // Images table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS images (
                id TEXT PRIMARY KEY,
                chapter_id TEXT NOT NULL,
                original_path TEXT NOT NULL,
                local_path TEXT NOT NULL,
                alt_text TEXT,
                paragraph_index INTEGER NOT NULL,
                position_type TEXT NOT NULL,
                FOREIGN KEY (chapter_id) REFERENCES chapters (id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        // Settings table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;
        
        info!("Database schema initialized successfully");
        Ok(())
    }
    
    pub async fn add_book(&mut self, title: String, author: String, original_path: String, language: String) -> Result<Book> {
        let id = Uuid::new_v4().to_string();
        let now = Utc::now();
        
        let book = Book {
            id: id.clone(),
            title: title.clone(),
            author,
            cover_path: None,
            original_path,
            language,
            target_language: None,
            translation_status: TranslationStatus::NotStarted,
            added_at: now,
            last_read: None,
            progress: 0.0,
        };
        
        sqlx::query(
            r#"
            INSERT INTO books (id, title, author, cover_path, original_path, language, target_language, translation_status, added_at, last_read, progress)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&book.id)
        .bind(&book.title)
        .bind(&book.author)
        .bind(&book.cover_path)
        .bind(&book.original_path)
        .bind(&book.language)
        .bind(&book.target_language)
        .bind("NotStarted")
        .bind(book.added_at.to_rfc3339())
        .bind(book.last_read.map(|d| d.to_rfc3339()))
        .bind(book.progress)
        .execute(&self.pool)
        .await?;
        
        // Create book directory
        let book_dir = self.base_path.join(&title);
        std::fs::create_dir_all(&book_dir)?;
        std::fs::create_dir_all(book_dir.join("images"))?;
        
        info!("Added book: {} by {}", title, book.author);
        Ok(book)
    }
    
    pub async fn get_all_books(&self) -> Result<Vec<Book>> {
        let rows = sqlx::query("SELECT * FROM books ORDER BY added_at DESC")
            .fetch_all(&self.pool)
            .await?;
        
        let mut books = Vec::new();
        for row in rows {
            let translation_status = match row.get::<String, _>("translation_status").as_str() {
                "NotStarted" => TranslationStatus::NotStarted,
                "InProgress" => TranslationStatus::InProgress,
                "Completed" => TranslationStatus::Completed,
                "Failed" => TranslationStatus::Failed,
                _ => TranslationStatus::NotStarted,
            };
            
            let book = Book {
                id: row.get("id"),
                title: row.get("title"),
                author: row.get("author"),
                cover_path: row.get("cover_path"),
                original_path: row.get("original_path"),
                language: row.get("language"),
                target_language: row.get("target_language"),
                translation_status,
                added_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("added_at"))?.with_timezone(&Utc),
                last_read: row.get::<Option<String>, _>("last_read")
                    .map(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .flatten()
                    .map(|dt| dt.with_timezone(&Utc)),
                progress: row.get("progress"),
            };
            books.push(book);
        }
        
        Ok(books)
    }
    
    pub async fn add_chapter(&mut self, book_id: String, title: String, content: String, order: i32) -> Result<Chapter> {
        let id = Uuid::new_v4().to_string();
        
        let chapter = Chapter {
            id: id.clone(),
            book_id: book_id.clone(),
            title: title.clone(),
            content: content.clone(),
            translated_content: None,
            order,
            images: Vec::new(),
        };
        
        sqlx::query(
            r#"
            INSERT INTO chapters (id, book_id, title, content, translated_content, chapter_order)
            VALUES (?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&chapter.id)
        .bind(&chapter.book_id)
        .bind(&chapter.title)
        .bind(&chapter.content)
        .bind(&chapter.translated_content)
        .bind(chapter.order)
        .execute(&self.pool)
        .await?;
        
        Ok(chapter)
    }
    
    pub async fn get_book_chapters(&self, book_id: &str) -> Result<Vec<Chapter>> {
        let rows = sqlx::query("SELECT * FROM chapters WHERE book_id = ? ORDER BY chapter_order")
            .bind(book_id)
            .fetch_all(&self.pool)
            .await?;
        
        let mut chapters = Vec::new();
        for row in rows {
            let chapter_id: String = row.get("id");
            let images = self.get_chapter_images(&chapter_id).await?;
            
            let chapter = Chapter {
                id: chapter_id,
                book_id: row.get("book_id"),
                title: row.get("title"),
                content: row.get("content"),
                translated_content: row.get("translated_content"),
                order: row.get("chapter_order"),
                images,
            };
            chapters.push(chapter);
        }
        
        Ok(chapters)
    }
    
    async fn get_chapter_images(&self, chapter_id: &str) -> Result<Vec<ImageReference>> {
        let rows = sqlx::query("SELECT * FROM images WHERE chapter_id = ? ORDER BY paragraph_index")
            .bind(chapter_id)
            .fetch_all(&self.pool)
            .await?;
        
        let mut images = Vec::new();
        for row in rows {
            let position_type = match row.get::<String, _>("position_type").as_str() {
                "Before" => PositionType::Before,
                "After" => PositionType::After,
                "Inline" => PositionType::Inline,
                _ => PositionType::Inline,
            };
            
            let image = ImageReference {
                id: row.get("id"),
                original_path: row.get("original_path"),
                local_path: row.get("local_path"),
                alt_text: row.get("alt_text"),
                position: ImagePosition {
                    chapter_id: chapter_id.to_string(),
                    paragraph_index: row.get("paragraph_index"),
                    position_type,
                },
            };
            images.push(image);
        }
        
        Ok(images)
    }
    
    pub async fn update_chapter_translation(&mut self, chapter_id: &str, translated_content: String) -> Result<()> {
        sqlx::query("UPDATE chapters SET translated_content = ? WHERE id = ?")
            .bind(translated_content)
            .bind(chapter_id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    pub async fn update_book_translation_status(&mut self, book_id: &str, status: TranslationStatus) -> Result<()> {
        let status_str = match status {
            TranslationStatus::NotStarted => "NotStarted",
            TranslationStatus::InProgress => "InProgress",
            TranslationStatus::Completed => "Completed",
            TranslationStatus::Failed => "Failed",
        };
        
        sqlx::query("UPDATE books SET translation_status = ? WHERE id = ?")
            .bind(status_str)
            .bind(book_id)
            .execute(&self.pool)
            .await?;
        
        Ok(())
    }
    
    pub async fn get_book_by_id(&self, book_id: &str) -> Result<Option<Book>> {
        let row = sqlx::query("SELECT * FROM books WHERE id = ?")
            .bind(book_id)
            .fetch_optional(&self.pool)
            .await?;
        
        if let Some(row) = row {
            let translation_status = match row.get::<String, _>("translation_status").as_str() {
                "NotStarted" => TranslationStatus::NotStarted,
                "InProgress" => TranslationStatus::InProgress,
                "Completed" => TranslationStatus::Completed,
                "Failed" => TranslationStatus::Failed,
                _ => TranslationStatus::NotStarted,
            };
            
            let book = Book {
                id: row.get("id"),
                title: row.get("title"),
                author: row.get("author"),
                cover_path: row.get("cover_path"),
                original_path: row.get("original_path"),
                language: row.get("language"),
                target_language: row.get("target_language"),
                translation_status,
                added_at: chrono::DateTime::parse_from_rfc3339(&row.get::<String, _>("added_at"))?.with_timezone(&Utc),
                last_read: row.get::<Option<String>, _>("last_read")
                    .map(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .flatten()
                    .map(|dt| dt.with_timezone(&Utc)),
                progress: row.get("progress"),
            };
            Ok(Some(book))
        } else {
            Ok(None)
        }
    }
    
    pub fn get_base_path(&self) -> &PathBuf {
        &self.base_path
    }
}