use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Book {
    pub id: String,
    pub title: String,
    pub author: String,
    pub cover_path: Option<String>,
    pub original_path: String,
    pub language: String,
    pub target_language: Option<String>,
    pub translation_status: TranslationStatus,
    pub added_at: DateTime<Utc>,
    pub last_read: Option<DateTime<Utc>>,
    pub progress: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TranslationStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub book_id: String,
    pub title: String,
    pub content: String,
    pub translated_content: Option<String>,
    pub order: i32,
    pub images: Vec<ImageReference>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageReference {
    pub id: String,
    pub original_path: String,
    pub local_path: String,
    pub alt_text: Option<String>,
    pub position: ImagePosition,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePosition {
    pub chapter_id: String,
    pub paragraph_index: i32,
    pub position_type: PositionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PositionType {
    Before,
    After,
    Inline,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Language {
    pub code: String,
    pub name: String,
    pub native_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStatus {
    pub is_running: bool,
    pub available_models: Vec<String>,
    pub recommended_model: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TranslationRequest {
    pub text: String,
    pub source_language: String,
    pub target_language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookContent {
    pub book: Book,
    pub chapters: Vec<Chapter>,
    pub current_chapter: i32,
}

impl Default for TranslationStatus {
    fn default() -> Self {
        TranslationStatus::NotStarted
    }
}