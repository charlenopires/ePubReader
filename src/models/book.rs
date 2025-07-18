use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use chrono::{DateTime, Utc};
use crate::models::library::ReadingStatus;

/// Book information model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub isbn: Option<String>,
    pub genre: Option<String>,
    pub description: Option<String>,
    pub publication_date: Option<DateTime<Utc>>,
    pub language: Option<String>,
    pub file_path: PathBuf,
    pub file_size: u64,
    pub file_format: BookFormat,
    pub cover_path: Option<PathBuf>,
    pub cover_url: Option<String>,
    pub page_count: Option<u32>,
    pub word_count: Option<u32>,
    pub reading_progress: f32, // 0.0 to 1.0
    pub reading_status: ReadingStatus,
    pub last_read_position: Option<ReadingPosition>,
    pub added_date: DateTime<Utc>,
    pub last_opened: Option<DateTime<Utc>>,
    pub is_favorite: bool,
    pub tags: Vec<String>,
    pub rating: Option<u8>, // 1-5 stars
    pub notes: Option<String>,
}

/// Book format enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BookFormat {
    Epub,
    Pdf,
    Mobi,
    Azw3,
    Txt,
    Html,
}

impl BookFormat {
    pub fn from_extension(extension: &str) -> Option<Self> {
        match extension.to_lowercase().as_str() {
            "epub" => Some(BookFormat::Epub),
            "pdf" => Some(BookFormat::Pdf),
            "mobi" => Some(BookFormat::Mobi),
            "azw3" => Some(BookFormat::Azw3),
            "txt" => Some(BookFormat::Txt),
            "html" | "htm" => Some(BookFormat::Html),
            _ => None,
        }
    }
    
    pub fn to_extension(&self) -> &'static str {
        match self {
            BookFormat::Epub => "epub",
            BookFormat::Pdf => "pdf",
            BookFormat::Mobi => "mobi",
            BookFormat::Azw3 => "azw3",
            BookFormat::Txt => "txt",
            BookFormat::Html => "html",
        }
    }
}


/// Reading position within a book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingPosition {
    pub chapter_id: Option<String>,
    pub page_number: Option<u32>,
    pub character_offset: Option<u64>,
    pub percentage: f32, // 0.0 to 1.0
    pub timestamp: DateTime<Utc>,
}

/// Book metadata for display in UI
#[derive(Debug, Clone)]
pub struct BookViewModel {
    pub id: String,
    pub title: String,
    pub author: String,
    pub cover_path: Option<PathBuf>,
    pub progress: f32,
    pub status: String,
    pub is_favorite: bool,
    pub rating: Option<u8>,
    pub last_opened: Option<DateTime<Utc>>,
    pub added_date: DateTime<Utc>,
}

impl From<Book> for BookViewModel {
    fn from(book: Book) -> Self {
        Self {
            id: book.id,
            title: book.title,
            author: book.author,
            cover_path: book.cover_path,
            progress: book.reading_progress,
            status: book.reading_status.to_string(),
            is_favorite: book.is_favorite,
            rating: book.rating,
            last_opened: book.last_opened,
            added_date: book.added_date,
        }
    }
}

/// Book collection model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookCollection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub created_date: DateTime<Utc>,
    pub book_ids: Vec<String>,
}

/// Book annotation model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookAnnotation {
    pub id: String,
    pub book_id: String,
    pub annotation_type: AnnotationType,
    pub content: String,
    pub note_text: Option<String>,
    pub color: Option<String>,
    pub position: AnnotationPosition,
    pub created_date: DateTime<Utc>,
    pub modified_date: DateTime<Utc>,
}

/// Annotation type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnnotationType {
    Highlight,
    Note,
    Bookmark,
    Underline,
}

/// Annotation position within a book
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationPosition {
    pub chapter_id: Option<String>,
    pub start_offset: u64,
    pub end_offset: u64,
    pub page_number: Option<u32>,
}

/// Reading session model for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingSession {
    pub id: String,
    pub book_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: u32,
    pub pages_read: u32,
    pub words_read: u32,
    pub progress_start: f32,
    pub progress_end: f32,
}

/// Book statistics model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookStatistics {
    pub total_books: u32,
    pub books_read: u32,
    pub books_reading: u32,
    pub total_pages_read: u32,
    pub total_reading_time_minutes: u32,
    pub average_reading_speed_wpm: f32,
    pub favorite_genres: Vec<String>,
    pub reading_streak_days: u32,
}

impl Book {
    /// Create a new book instance
    pub fn new(
        title: String,
        author: String,
        file_path: PathBuf,
        file_size: u64,
        file_format: BookFormat,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            title,
            author,
            isbn: None,
            genre: None,
            description: None,
            publication_date: None,
            language: None,
            file_path,
            file_size,
            file_format,
            cover_path: None,
            cover_url: None,
            page_count: None,
            word_count: None,
            reading_progress: 0.0,
            reading_status: ReadingStatus::Unread,
            last_read_position: None,
            added_date: Utc::now(),
            last_opened: None,
            is_favorite: false,
            tags: Vec::new(),
            rating: None,
            notes: None,
        }
    }
    
    /// Get the cover thumbnail path
    pub fn get_cover_thumbnail_path(&self) -> Option<PathBuf> {
        self.cover_path.as_ref().map(|path| {
            let mut thumbnail_path = path.clone();
            if let Some(file_stem) = path.file_stem() {
                thumbnail_path.set_file_name(format!("{}_thumb.jpg", file_stem.to_string_lossy()));
            }
            thumbnail_path
        })
    }
    
    /// Update reading progress
    pub fn update_progress(&mut self, progress: f32) {
        self.reading_progress = progress.clamp(0.0, 1.0);
        self.last_opened = Some(Utc::now());
        
        if progress >= 1.0 {
            self.reading_status = ReadingStatus::Finished;
        } else if progress > 0.0 && self.reading_status == ReadingStatus::Unread {
            self.reading_status = ReadingStatus::CurrentlyReading;
        }
    }
    
    /// Check if book is currently being read
    pub fn is_reading(&self) -> bool {
        matches!(self.reading_status, ReadingStatus::Reading)
    }
    
    /// Check if book is finished
    pub fn is_finished(&self) -> bool {
        matches!(self.reading_status, ReadingStatus::Finished)
    }
    
    /// Get estimated reading time in minutes
    pub fn estimated_reading_time(&self) -> Option<u32> {
        self.word_count.map(|words| {
            // Average reading speed: 200-250 words per minute
            let reading_speed = 225.0;
            (words as f32 / reading_speed).ceil() as u32
        })
    }
    
    /// Get reading progress percentage as integer
    pub fn progress_percentage(&self) -> u8 {
        (self.reading_progress * 100.0).round() as u8
    }
}

impl BookCollection {
    /// Create a new book collection
    pub fn new(name: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            color: None,
            created_date: Utc::now(),
            book_ids: Vec::new(),
        }
    }
    
    /// Add a book to the collection
    pub fn add_book(&mut self, book_id: String) {
        if !self.book_ids.contains(&book_id) {
            self.book_ids.push(book_id);
        }
    }
    
    /// Remove a book from the collection
    pub fn remove_book(&mut self, book_id: &str) {
        self.book_ids.retain(|id| id != book_id);
    }
    
    /// Check if collection contains a book
    pub fn contains_book(&self, book_id: &str) -> bool {
        self.book_ids.contains(&book_id.to_string())
    }
    
    /// Get number of books in collection
    pub fn book_count(&self) -> usize {
        self.book_ids.len()
    }
}