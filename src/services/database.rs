use sqlx::{SqlitePool, Row};
use std::path::{Path, PathBuf};
use anyhow::Result;
use chrono::{DateTime, Utc};
use tracing::{info, error};

use crate::models::{Book, BookCollection, BookFormat};
use crate::models::library::ReadingStatus;
use crate::services::book_service::{BookFilter, BookSort, SortField, SortOrder};
use crate::services::database_initializer::{DatabaseInitializer, DatabaseInitError};
use crate::services::path_resolver::PathResolver;

/// Database service for managing SQLite operations
pub struct DatabaseService {
    pool: SqlitePool,
}

impl DatabaseService {
    /// Create a new database service instance with automatic initialization
    pub async fn new() -> Result<Self> {
        Self::new_with_path(None).await
    }
    
    /// Create a new database service instance with custom path
    pub async fn new_with_path(custom_path: Option<PathBuf>) -> Result<Self> {
        info!("Initializing database service");
        
        // Determine database path
        let database_path = match custom_path {
            Some(path) => {
                info!("Using custom database path: {}", path.display());
                path
            }
            None => {
                PathResolver::resolve_database_path_with_fallback()
                    .map_err(|e| anyhow::anyhow!("Failed to resolve database path: {}", e))?
            }
        };
        
        // Initialize database
        let initializer = DatabaseInitializer::new(database_path.clone());
        let validated_path = initializer.ensure_database_ready().await
            .map_err(|e| anyhow::anyhow!("Database initialization failed: {}", e))?;
        
        // Connect to database
        let database_url = format!("sqlite://{}?mode=rwc", validated_path.display());
        let pool = SqlitePool::connect(&database_url).await
            .map_err(|e| anyhow::anyhow!("Failed to connect to database: {}", e))?;
        
        let service = Self { pool };
        
        // Initialize schema
        service.initialize_schema().await
            .map_err(|e| anyhow::anyhow!("Failed to initialize database schema: {}", e))?;
        
        info!("Database service initialized successfully");
        Ok(service)
    }
    
    /// Create database service for testing with in-memory database
    #[cfg(test)]
    pub async fn new_in_memory() -> Result<Self> {
        let pool = SqlitePool::connect("sqlite::memory:").await?;
        let service = Self { pool };
        service.initialize_schema().await?;
        Ok(service)
    }

    /// Initialize database schema
    async fn initialize_schema(&self) -> Result<()> {
        // Create books table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS books (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                author TEXT,
                isbn TEXT,
                genre TEXT,
                description TEXT,
                publication_date TEXT,
                language TEXT,
                file_path TEXT NOT NULL,
                file_size INTEGER NOT NULL,
                file_format TEXT NOT NULL,
                cover_path TEXT,
                cover_url TEXT,
                page_count INTEGER,
                word_count INTEGER,
                reading_progress REAL DEFAULT 0.0,
                reading_status TEXT DEFAULT 'new',
                last_read_position TEXT,
                added_date TEXT NOT NULL,
                last_opened TEXT,
                is_favorite INTEGER DEFAULT 0,
                rating INTEGER,
                notes TEXT,
                tags TEXT
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create collections table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                color TEXT,
                created_date TEXT NOT NULL
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create book_collections junction table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS book_collections (
                book_id TEXT NOT NULL,
                collection_id TEXT NOT NULL,
                PRIMARY KEY (book_id, collection_id),
                FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE,
                FOREIGN KEY (collection_id) REFERENCES collections(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create annotations table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS annotations (
                id TEXT PRIMARY KEY,
                book_id TEXT NOT NULL,
                annotation_type TEXT NOT NULL,
                content TEXT NOT NULL,
                note_text TEXT,
                color TEXT,
                position_data TEXT NOT NULL,
                created_date TEXT NOT NULL,
                modified_date TEXT NOT NULL,
                FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create reading_sessions table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS reading_sessions (
                id TEXT PRIMARY KEY,
                book_id TEXT NOT NULL,
                start_time TEXT NOT NULL,
                end_time TEXT,
                duration_minutes INTEGER DEFAULT 0,
                pages_read INTEGER DEFAULT 0,
                words_read INTEGER DEFAULT 0,
                progress_start REAL DEFAULT 0.0,
                progress_end REAL DEFAULT 0.0,
                FOREIGN KEY (book_id) REFERENCES books(id) ON DELETE CASCADE
            )
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Add missing columns for existing databases (simple migration)
        let _ = sqlx::query("ALTER TABLE books ADD COLUMN cover_url TEXT")
            .execute(&self.pool)
            .await; // Ignore error if column already exists

        // Create indexes for better query performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_books_title ON books(title)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_books_author ON books(author)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_books_added_date ON books(added_date)")
            .execute(&self.pool)
            .await?;
        
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_books_reading_status ON books(reading_status)")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Insert a new book
    pub async fn insert_book(&self, book: &Book) -> Result<()> {
        let tags_json = serde_json::to_string(&book.tags)?;
        
        sqlx::query(
            r#"
            INSERT INTO books (
                id, title, author, isbn, genre, description, publication_date, language,
                file_path, file_size, file_format, cover_path, cover_url, page_count, word_count,
                reading_progress, reading_status, added_date, last_opened, is_favorite,
                rating, notes, tags
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&book.id)
        .bind(&book.title)
        .bind(&book.author)
        .bind(&book.isbn)
        .bind(&book.genre)
        .bind(&book.description)
        .bind(book.publication_date.map(|d| d.to_rfc3339()))
        .bind(&book.language)
        .bind(book.file_path.to_string_lossy().to_string())
        .bind(book.file_size as i64)
        .bind(book.file_format.to_extension())
        .bind(book.cover_path.as_ref().map(|p| p.to_string_lossy().to_string()))
        .bind(&book.cover_url)        .bind(book.page_count.map(|p| p as i64))
        .bind(book.word_count.map(|w| w as i64))
        .bind(book.reading_progress)
        .bind(book.reading_status.to_string())
        .bind(book.added_date.to_rfc3339())
        .bind(book.last_opened.map(|d| d.to_rfc3339()))
        .bind(if book.is_favorite { 1 } else { 0 })
        .bind(book.rating.map(|r| r as i64))
        .bind(&book.notes)
        .bind(tags_json)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Update an existing book
    pub async fn update_book(&self, book: &Book) -> Result<()> {
        let tags_json = serde_json::to_string(&book.tags)?;
        
        sqlx::query(
            r#"
            UPDATE books SET
                title = ?, author = ?, isbn = ?, genre = ?, description = ?,
                publication_date = ?, language = ?, file_path = ?, file_size = ?,
                file_format = ?, cover_path = ?, cover_url = ?, page_count = ?, word_count = ?,
                reading_progress = ?, reading_status = ?, last_opened = ?,
                is_favorite = ?, rating = ?, notes = ?, tags = ?
            WHERE id = ?
            "#,
        )
        .bind(&book.title)
        .bind(&book.author)
        .bind(&book.isbn)
        .bind(&book.genre)
        .bind(&book.description)
        .bind(book.publication_date.map(|d| d.to_rfc3339()))
        .bind(&book.language)
        .bind(book.file_path.to_string_lossy().to_string())
        .bind(book.file_size as i64)
        .bind(book.file_format.to_extension())
        .bind(book.cover_path.as_ref().map(|p| p.to_string_lossy().to_string()))
        .bind(&book.cover_url)        .bind(book.page_count.map(|p| p as i64))
        .bind(book.word_count.map(|w| w as i64))
        .bind(book.reading_progress)
        .bind(book.reading_status.to_string())
        .bind(book.last_opened.map(|d| d.to_rfc3339()))
        .bind(if book.is_favorite { 1 } else { 0 })
        .bind(book.rating.map(|r| r as i64))
        .bind(&book.notes)
        .bind(tags_json)
        .bind(&book.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get a book by ID
    pub async fn get_book_by_id(&self, book_id: &str) -> Result<Book> {
        let row = sqlx::query("SELECT * FROM books WHERE id = ?")
            .bind(book_id)
            .fetch_one(&self.pool)
            .await?;

        self.row_to_book(row)
    }

    /// Get all books
    pub async fn get_all_books(&self) -> Result<Vec<Book>> {
        let rows = sqlx::query("SELECT * FROM books ORDER BY added_date DESC")
            .fetch_all(&self.pool)
            .await?;

        let mut books = Vec::new();
        for row in rows {
            books.push(self.row_to_book(row)?);
        }

        Ok(books)
    }

    /// Get books with filtering and sorting
    pub async fn get_filtered_books(
        &self,
        filter: &BookFilter,
        sort: &BookSort,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<Book>> {
        let mut query = "SELECT * FROM books WHERE 1=1".to_string();
        let mut params = Vec::new();

        // Apply filters
        if let Some(status) = &filter.status {
            query.push_str(" AND reading_status = ?");
            params.push(status.to_string());
        }

        if let Some(author) = &filter.author {
            query.push_str(" AND author LIKE ?");
            params.push(format!("%{}%", author));
        }

        if let Some(genre) = &filter.genre {
            query.push_str(" AND genre = ?");
            params.push(genre.clone());
        }

        if let Some(is_favorite) = filter.is_favorite {
            query.push_str(" AND is_favorite = ?");
            params.push(if is_favorite { "1".to_string() } else { "0".to_string() });
        }

        if let Some(rating_min) = filter.rating_min {
            query.push_str(" AND rating >= ?");
            params.push(rating_min.to_string());
        }

        if let Some(rating_max) = filter.rating_max {
            query.push_str(" AND rating <= ?");
            params.push(rating_max.to_string());
        }

        // Apply sorting
        match sort.field {
            SortField::Title => query.push_str(" ORDER BY title"),
            SortField::Author => query.push_str(" ORDER BY author"),
            SortField::DateAdded => query.push_str(" ORDER BY added_date"),
            SortField::LastOpened => query.push_str(" ORDER BY last_opened"),
            SortField::ReadingProgress => query.push_str(" ORDER BY reading_progress"),
            SortField::Rating => query.push_str(" ORDER BY rating"),
        }

        match sort.order {
            SortOrder::Ascending => query.push_str(" ASC"),
            SortOrder::Descending => query.push_str(" DESC"),
        }

        // Apply pagination
        if let Some(limit) = limit {
            query.push_str(&format!(" LIMIT {}", limit));
            if let Some(offset) = offset {
                query.push_str(&format!(" OFFSET {}", offset));
            }
        }

        let mut sql_query = sqlx::query(&query);
        for param in params {
            sql_query = sql_query.bind(param);
        }

        let rows = sql_query.fetch_all(&self.pool).await?;
        let mut books = Vec::new();
        for row in rows {
            books.push(self.row_to_book(row)?);
        }

        Ok(books)
    }

    /// Search books by query
    pub async fn search_books(&self, query: &str) -> Result<Vec<Book>> {
        let search_query = format!("%{}%", query);
        
        let rows = sqlx::query(
            r#"
            SELECT * FROM books 
            WHERE title LIKE ? OR author LIKE ? OR description LIKE ? OR tags LIKE ?
            ORDER BY 
                CASE 
                    WHEN title LIKE ? THEN 1
                    WHEN author LIKE ? THEN 2
                    WHEN description LIKE ? THEN 3
                    ELSE 4
                END,
                title
            "#,
        )
        .bind(&search_query)
        .bind(&search_query)
        .bind(&search_query)
        .bind(&search_query)
        .bind(&search_query)
        .bind(&search_query)
        .bind(&search_query)
        .fetch_all(&self.pool)
        .await?;

        let mut books = Vec::new();
        for row in rows {
            books.push(self.row_to_book(row)?);
        }

        Ok(books)
    }

    /// Delete a book
    pub async fn delete_book(&self, book_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM books WHERE id = ?")
            .bind(book_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Check if book exists by file path
    pub async fn book_exists_by_path(&self, file_path: &Path) -> Result<bool> {
        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM books WHERE file_path = ?")
            .bind(file_path.to_string_lossy().to_string())
            .fetch_one(&self.pool)
            .await?;

        Ok(count > 0)
    }

    /// Get books by status
    pub async fn get_books_by_status(&self, status: ReadingStatus) -> Result<Vec<Book>> {
        let rows = sqlx::query("SELECT * FROM books WHERE reading_status = ? ORDER BY last_opened DESC")
            .bind(status.to_string())
            .fetch_all(&self.pool)
            .await?;

        let mut books = Vec::new();
        for row in rows {
            books.push(self.row_to_book(row)?);
        }

        Ok(books)
    }

    /// Get recently added books
    pub async fn get_recently_added_books(&self, limit: usize) -> Result<Vec<Book>> {
        let rows = sqlx::query("SELECT * FROM books ORDER BY added_date DESC LIMIT ?")
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;

        let mut books = Vec::new();
        for row in rows {
            books.push(self.row_to_book(row)?);
        }

        Ok(books)
    }

    /// Insert a new collection
    pub async fn insert_collection(&self, collection: &BookCollection) -> Result<()> {
        sqlx::query(
            "INSERT INTO collections (id, name, description, color, created_date) VALUES (?, ?, ?, ?, ?)"
        )
        .bind(&collection.id)
        .bind(&collection.name)
        .bind(&collection.description)
        .bind(&collection.color)
        .bind(collection.created_date.to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all collections
    pub async fn get_all_collections(&self) -> Result<Vec<BookCollection>> {
        let rows = sqlx::query("SELECT * FROM collections ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        let mut collections = Vec::new();
        for row in rows {
            let collection = BookCollection {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                color: row.get("color"),
                created_date: DateTime::parse_from_rfc3339(&row.get::<String, _>("created_date"))
                    .unwrap()
                    .with_timezone(&Utc),
                book_ids: self.get_collection_book_ids(&row.get::<String, _>("id")).await?,
            };
            collections.push(collection);
        }

        Ok(collections)
    }

    /// Add book to collection
    pub async fn add_book_to_collection(&self, book_id: &str, collection_id: &str) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO book_collections (book_id, collection_id) VALUES (?, ?)"
        )
        .bind(book_id)
        .bind(collection_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get book IDs for a collection
    async fn get_collection_book_ids(&self, collection_id: &str) -> Result<Vec<String>> {
        let rows = sqlx::query("SELECT book_id FROM book_collections WHERE collection_id = ?")
            .bind(collection_id)
            .fetch_all(&self.pool)
            .await?;

        let book_ids = rows.into_iter().map(|row| row.get("book_id")).collect();
        Ok(book_ids)
    }

    /// Convert database row to Book struct
    fn row_to_book(&self, row: sqlx::sqlite::SqliteRow) -> Result<Book> {
        let tags_json: String = row.get("tags");
        let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
        
        let publication_date = row.get::<Option<String>, _>("publication_date")
            .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
            .map(|d| d.with_timezone(&Utc));

        let last_opened = row.get::<Option<String>, _>("last_opened")
            .and_then(|d| DateTime::parse_from_rfc3339(&d).ok())
            .map(|d| d.with_timezone(&Utc));

        let added_date = DateTime::parse_from_rfc3339(&row.get::<String, _>("added_date"))
            .unwrap()
            .with_timezone(&Utc);

        let file_format = BookFormat::from_extension(&row.get::<String, _>("file_format"))
            .unwrap_or(BookFormat::Epub);

        let reading_status = ReadingStatus::from_string(&row.get::<String, _>("reading_status"))
            .unwrap_or(ReadingStatus::Unread);

        Ok(Book {
            id: row.get("id"),
            title: row.get("title"),
            author: row.get("author"),
            isbn: row.get("isbn"),
            genre: row.get("genre"),
            description: row.get("description"),
            publication_date,
            language: row.get("language"),
            file_path: row.get::<String, _>("file_path").into(),
            file_size: row.get::<i64, _>("file_size") as u64,
            file_format,
            cover_path: row.get::<Option<String>, _>("cover_path").map(|p| p.into()),
            cover_url: row.get::<Option<String>, _>("cover_url"),
            page_count: row.get::<Option<i64>, _>("page_count").map(|p| p as u32),
            word_count: row.get::<Option<i64>, _>("word_count").map(|w| w as u32),
            reading_progress: row.get("reading_progress"),
            reading_status,
            last_read_position: None, // TODO: Implement position parsing
            added_date,
            last_opened,
            is_favorite: row.get::<i64, _>("is_favorite") != 0,
            tags,
            rating: row.get::<Option<i64>, _>("rating").map(|r| r as u8),
            notes: row.get("notes"),
        })
    }
}