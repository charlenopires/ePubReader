use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::{Result, anyhow};
use chrono::Utc;
use image::imageops::FilterType;
use uuid::Uuid;

use crate::models::{Book, BookViewModel, BookFormat, BookCollection};
use crate::models::library::ReadingStatus;
use crate::services::database::DatabaseService;
use crate::utils::image_cache::ImageCache;

/// Book service for managing book operations
pub struct BookService {
    database: Arc<DatabaseService>,
    image_cache: Arc<ImageCache>,
    book_cache: Arc<RwLock<HashMap<String, Book>>>,
    collections_cache: Arc<RwLock<HashMap<String, BookCollection>>>,
}

impl BookService {
    /// Create a new book service instance
    pub fn new(database: Arc<DatabaseService>, image_cache: Arc<ImageCache>) -> Self {
        Self {
            database,
            image_cache,
            book_cache: Arc::new(RwLock::new(HashMap::new())),
            collections_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get all books in the library
    pub async fn get_library_books(&self) -> Result<Vec<BookViewModel>> {
        let books = self.database.get_all_books().await?;
        let mut view_models = Vec::new();
        
        for book in books {
            let cover_path = if let Some(cover) = &book.cover_path {
                // Check if thumbnail exists, create if not
                let thumbnail_path = self.get_or_create_thumbnail(&book.id, cover).await?;
                Some(thumbnail_path)
            } else {
                None
            };
            
            view_models.push(BookViewModel {
                id: book.id.clone(),
                title: book.title.clone(),
                author: book.author.clone(),
                cover_path,
                progress: book.reading_progress,
                status: book.reading_status.to_string(),
                is_favorite: book.is_favorite,
                rating: book.rating,
                last_opened: book.last_opened,
                added_date: book.added_date,
            });
        }
        
        Ok(view_models)
    }

    /// Get books with filtering and sorting
    pub async fn get_filtered_books(
        &self,
        filter: &BookFilter,
        sort: &BookSort,
        limit: Option<usize>,
        offset: Option<usize>,
    ) -> Result<Vec<BookViewModel>> {
        let books = self.database.get_filtered_books(filter, sort, limit, offset).await?;
        let mut view_models = Vec::new();
        
        for book in books {
            let cover_path = if let Some(cover) = &book.cover_path {
                let thumbnail_path = self.get_or_create_thumbnail(&book.id, cover).await?;
                Some(thumbnail_path)
            } else {
                None
            };
            
            view_models.push(BookViewModel::from(book));
        }
        
        Ok(view_models)
    }

    /// Search books by query
    pub async fn search_books(&self, query: &str) -> Result<Vec<BookViewModel>> {
        let books = self.database.search_books(query).await?;
        let mut view_models = Vec::new();
        
        for book in books {
            let cover_path = if let Some(cover) = &book.cover_path {
                let thumbnail_path = self.get_or_create_thumbnail(&book.id, cover).await?;
                Some(thumbnail_path)
            } else {
                None
            };
            
            view_models.push(BookViewModel::from(book));
        }
        
        Ok(view_models)
    }

    /// Add a new book to the library
    pub async fn add_book(&self, file_path: &Path) -> Result<String> {
        // Check if book already exists
        if self.database.book_exists_by_path(file_path).await? {
            return Err(anyhow!("Book already exists in library"));
        }

        // Parse book metadata
        let mut book = self.parse_book_metadata(file_path).await?;
        
        // Generate unique ID
        book.id = Uuid::new_v4().to_string();
        
        // Extract and cache cover if available
        if let Some(cover_data) = self.extract_cover_data(&book).await? {
            let cover_path = self.image_cache.save_cover(&book.id, &cover_data).await?;
            book.cover_path = Some(cover_path);
        }

        // Save to database
        self.database.insert_book(&book).await?;
        
        // Update cache
        let mut cache = self.book_cache.write().await;
        cache.insert(book.id.clone(), book.clone());
        
        Ok(book.id)
    }

    /// Update book information
    pub async fn update_book(&self, book_id: &str, updated_book: &Book) -> Result<()> {
        self.database.update_book(updated_book).await?;
        
        // Update cache
        let mut cache = self.book_cache.write().await;
        cache.insert(book_id.to_string(), updated_book.clone());
        
        Ok(())
    }

    /// Delete a book from the library
    pub async fn delete_book(&self, book_id: &str) -> Result<()> {
        // Get book info before deletion
        let book = self.get_book_by_id(book_id).await?;
        
        // Delete from database
        self.database.delete_book(book_id).await?;
        
        // Remove from cache
        let mut cache = self.book_cache.write().await;
        cache.remove(book_id);
        
        // Clean up cover image
        if let Some(cover_path) = book.cover_path {
            let _ = self.image_cache.remove_cover(&cover_path).await;
        }
        
        Ok(())
    }

    /// Get a book by ID
    pub async fn get_book_by_id(&self, book_id: &str) -> Result<Book> {
        // Check cache first
        {
            let cache = self.book_cache.read().await;
            if let Some(book) = cache.get(book_id) {
                return Ok(book.clone());
            }
        }
        
        // Load from database
        let book = self.database.get_book_by_id(book_id).await?;
        
        // Update cache
        let mut cache = self.book_cache.write().await;
        cache.insert(book_id.to_string(), book.clone());
        
        Ok(book)
    }

    /// Update reading progress for a book
    pub async fn update_reading_progress(
        &self,
        book_id: &str,
        progress: f32,
    ) -> Result<()> {
        let mut book = self.get_book_by_id(book_id).await?;
        book.update_progress(progress);
        
        self.database.update_book(&book).await?;
        
        // Update cache
        let mut cache = self.book_cache.write().await;
        cache.insert(book_id.to_string(), book);
        
        Ok(())
    }

    /// Toggle favorite status for a book
    pub async fn toggle_favorite(&self, book_id: &str) -> Result<bool> {
        let mut book = self.get_book_by_id(book_id).await?;
        book.is_favorite = !book.is_favorite;
        
        self.database.update_book(&book).await?;
        
        // Update cache
        let mut cache = self.book_cache.write().await;
        cache.insert(book_id.to_string(), book.clone());
        
        Ok(book.is_favorite)
    }

    /// Get book collections
    pub async fn get_collections(&self) -> Result<Vec<BookCollection>> {
        let collections = self.database.get_all_collections().await?;
        
        // Update cache
        let mut cache = self.collections_cache.write().await;
        cache.clear();
        for collection in &collections {
            cache.insert(collection.id.clone(), collection.clone());
        }
        
        Ok(collections)
    }

    /// Create a new collection
    pub async fn create_collection(&self, name: String) -> Result<String> {
        let collection = BookCollection::new(name);
        let collection_id = collection.id.clone();
        
        self.database.insert_collection(&collection).await?;
        
        // Update cache
        let mut cache = self.collections_cache.write().await;
        cache.insert(collection_id.clone(), collection);
        
        Ok(collection_id)
    }

    /// Add book to collection
    pub async fn add_book_to_collection(
        &self,
        book_id: &str,
        collection_id: &str,
    ) -> Result<()> {
        self.database.add_book_to_collection(book_id, collection_id).await?;
        
        // Update cache
        let mut cache = self.collections_cache.write().await;
        if let Some(collection) = cache.get_mut(collection_id) {
            collection.add_book(book_id.to_string());
        }
        
        Ok(())
    }

    /// Get recently added books
    pub async fn get_recently_added(&self, limit: usize) -> Result<Vec<BookViewModel>> {
        let books = self.database.get_recently_added_books(limit).await?;
        let mut view_models = Vec::new();
        
        for book in books {
            let cover_path = if let Some(cover) = &book.cover_path {
                let thumbnail_path = self.get_or_create_thumbnail(&book.id, cover).await?;
                Some(thumbnail_path)
            } else {
                None
            };
            
            view_models.push(BookViewModel::from(book));
        }
        
        Ok(view_models)
    }

    /// Get currently reading books
    pub async fn get_currently_reading(&self) -> Result<Vec<BookViewModel>> {
        let books = self.database.get_books_by_status(ReadingStatus::CurrentlyReading).await?;
        let mut view_models = Vec::new();
        
        for book in books {
            let cover_path = if let Some(cover) = &book.cover_path {
                let thumbnail_path = self.get_or_create_thumbnail(&book.id, cover).await?;
                Some(thumbnail_path)
            } else {
                None
            };
            
            view_models.push(BookViewModel::from(book));
        }
        
        Ok(view_models)
    }

    /// Parse book metadata from file
    async fn parse_book_metadata(&self, file_path: &Path) -> Result<Book> {
        let file_size = std::fs::metadata(file_path)?.len();
        let format = BookFormat::from_extension(
            file_path.extension()
                .and_then(|ext| ext.to_str())
                .unwrap_or("")
        ).ok_or_else(|| anyhow!("Unsupported file format"))?;

        let mut book = Book::new(
            "Unknown Title".to_string(),
            "Unknown Author".to_string(),
            file_path.to_path_buf(),
            file_size,
            format.clone(),
        );

        // Parse metadata based on format
        match format {
            BookFormat::Epub => {
                self.parse_epub_metadata(&mut book).await?;
            }
            BookFormat::Pdf => {
                self.parse_pdf_metadata(&mut book).await?;
            }
            _ => {
                // For other formats, try to extract basic info from filename
                if let Some(file_stem) = file_path.file_stem() {
                    book.title = file_stem.to_string_lossy().to_string();
                }
            }
        }

        Ok(book)
    }

    /// Parse EPUB metadata
    async fn parse_epub_metadata(&self, book: &mut Book) -> Result<()> {
        // Implementation would use epub crate to extract metadata
        // This is a placeholder for the actual implementation
        Ok(())
    }

    /// Parse PDF metadata
    async fn parse_pdf_metadata(&self, book: &mut Book) -> Result<()> {
        // Implementation would use pdf-extract crate to extract metadata
        // This is a placeholder for the actual implementation
        Ok(())
    }

    /// Extract cover data from book
    async fn extract_cover_data(&self, book: &Book) -> Result<Option<Vec<u8>>> {
        match book.file_format {
            BookFormat::Epub => {
                // Extract cover from EPUB
                // This is a placeholder for the actual implementation
                Ok(None)
            }
            BookFormat::Pdf => {
                // Extract first page as cover from PDF
                // This is a placeholder for the actual implementation
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    /// Get or create thumbnail for book cover
    async fn get_or_create_thumbnail(
        &self,
        book_id: &str,
        cover_path: &Path,
    ) -> Result<PathBuf> {
        let thumbnail_path = self.image_cache.get_thumbnail_path(book_id);
        
        if !thumbnail_path.exists() {
            // Create thumbnail
            let img = image::open(cover_path)?;
            let thumbnail = img.resize(200, 300, FilterType::Lanczos3);
            thumbnail.save(&thumbnail_path)?;
        }
        
        Ok(thumbnail_path)
    }

    /// Clear all caches
    pub async fn clear_caches(&self) {
        let mut book_cache = self.book_cache.write().await;
        book_cache.clear();
        
        let mut collections_cache = self.collections_cache.write().await;
        collections_cache.clear();
    }
}

/// Book filtering options
#[derive(Debug, Clone)]
pub struct BookFilter {
    pub status: Option<ReadingStatus>,
    pub author: Option<String>,
    pub genre: Option<String>,
    pub tags: Vec<String>,
    pub is_favorite: Option<bool>,
    pub rating_min: Option<u8>,
    pub rating_max: Option<u8>,
}

/// Book sorting options
#[derive(Debug, Clone)]
pub struct BookSort {
    pub field: SortField,
    pub order: SortOrder,
}

#[derive(Debug, Clone)]
pub enum SortField {
    Title,
    Author,
    DateAdded,
    LastOpened,
    ReadingProgress,
    Rating,
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Ascending,
    Descending,
}

impl Default for BookFilter {
    fn default() -> Self {
        Self {
            status: None,
            author: None,
            genre: None,
            tags: Vec::new(),
            is_favorite: None,
            rating_min: None,
            rating_max: None,
        }
    }
}

impl Default for BookSort {
    fn default() -> Self {
        Self {
            field: SortField::DateAdded,
            order: SortOrder::Descending,
        }
    }
}