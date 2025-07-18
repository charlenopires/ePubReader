use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// User preferences model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub library: LibraryPreferences,
    pub reading: ReadingPreferences,
    pub ui: UiPreferences,
    pub sync: SyncPreferences,
    pub privacy: PrivacyPreferences,
}

/// Library management preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryPreferences {
    pub default_library_path: PathBuf,
    pub auto_import: bool,
    pub watch_folders: Vec<PathBuf>,
    pub duplicate_handling: DuplicateHandling,
    pub metadata_sources: Vec<MetadataSource>,
    pub cover_download: bool,
    pub organize_by_author: bool,
    pub organize_by_genre: bool,
}

/// Reading experience preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingPreferences {
    pub default_font_family: String,
    pub default_font_size: u16,
    pub default_line_spacing: f32,
    pub default_margin: u16,
    pub default_theme: String,
    pub page_turn_animation: bool,
    pub reading_progress_sync: bool,
    pub auto_bookmark: bool,
    pub highlight_color: String,
    pub note_color: String,
}

/// UI preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiPreferences {
    pub theme: String,
    pub language: String,
    pub view_mode: ViewMode,
    pub grid_columns: u8,
    pub sort_by: SortBy,
    pub sort_order: SortOrder,
    pub show_reading_progress: bool,
    pub show_cover_thumbnails: bool,
    pub animation_enabled: bool,
    pub compact_mode: bool,
}

/// Synchronization preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncPreferences {
    pub enabled: bool,
    pub provider: SyncProvider,
    pub sync_library: bool,
    pub sync_reading_progress: bool,
    pub sync_annotations: bool,
    pub sync_preferences: bool,
    pub auto_sync: bool,
    pub sync_interval_minutes: u32,
}

/// Privacy preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacyPreferences {
    pub analytics_enabled: bool,
    pub crash_reporting: bool,
    pub usage_statistics: bool,
    pub personalized_recommendations: bool,
    pub data_retention_days: u32,
}

/// Duplicate handling strategies
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DuplicateHandling {
    Ask,
    Skip,
    Replace,
    KeepBoth,
}

/// Metadata sources
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MetadataSource {
    GoogleBooks,
    OpenLibrary,
    Goodreads,
    LocalFile,
}

/// View modes for the library
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ViewMode {
    Grid,
    List,
    LargeCover,
    Details,
}

/// Sort criteria
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortBy {
    Title,
    Author,
    DateAdded,
    DateModified,
    ReadingProgress,
    Rating,
    FileSize,
    PublicationDate,
}

/// Sort order
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortOrder {
    Ascending,
    Descending,
}

/// Sync providers
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncProvider {
    None,
    GoogleDrive,
    Dropbox,
    OneDrive,
    ICloud,
    Custom(String),
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            library: LibraryPreferences::default(),
            reading: ReadingPreferences::default(),
            ui: UiPreferences::default(),
            sync: SyncPreferences::default(),
            privacy: PrivacyPreferences::default(),
        }
    }
}

impl Default for LibraryPreferences {
    fn default() -> Self {
        Self {
            default_library_path: dirs::document_dir()
                .unwrap_or_else(|| dirs::home_dir().unwrap_or_default())
                .join("Ebook Library"),
            auto_import: true,
            watch_folders: Vec::new(),
            duplicate_handling: DuplicateHandling::Ask,
            metadata_sources: vec![
                MetadataSource::LocalFile,
                MetadataSource::GoogleBooks,
                MetadataSource::OpenLibrary,
            ],
            cover_download: true,
            organize_by_author: false,
            organize_by_genre: false,
        }
    }
}

impl Default for ReadingPreferences {
    fn default() -> Self {
        Self {
            default_font_family: "System".to_string(),
            default_font_size: 16,
            default_line_spacing: 1.5,
            default_margin: 40,
            default_theme: "light".to_string(),
            page_turn_animation: true,
            reading_progress_sync: true,
            auto_bookmark: true,
            highlight_color: "#FFD700".to_string(),
            note_color: "#87CEEB".to_string(),
        }
    }
}

impl Default for UiPreferences {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            language: "en".to_string(),
            view_mode: ViewMode::Grid,
            grid_columns: 6,
            sort_by: SortBy::DateAdded,
            sort_order: SortOrder::Descending,
            show_reading_progress: true,
            show_cover_thumbnails: true,
            animation_enabled: true,
            compact_mode: false,
        }
    }
}

impl Default for SyncPreferences {
    fn default() -> Self {
        Self {
            enabled: false,
            provider: SyncProvider::None,
            sync_library: true,
            sync_reading_progress: true,
            sync_annotations: true,
            sync_preferences: true,
            auto_sync: true,
            sync_interval_minutes: 15,
        }
    }
}

impl Default for PrivacyPreferences {
    fn default() -> Self {
        Self {
            analytics_enabled: false,
            crash_reporting: true,
            usage_statistics: false,
            personalized_recommendations: true,
            data_retention_days: 365,
        }
    }
}

impl ViewMode {
    pub fn to_string(&self) -> String {
        match self {
            ViewMode::Grid => "grid".to_string(),
            ViewMode::List => "list".to_string(),
            ViewMode::LargeCover => "large-cover".to_string(),
            ViewMode::Details => "details".to_string(),
        }
    }
    
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "grid" => Some(ViewMode::Grid),
            "list" => Some(ViewMode::List),
            "large-cover" => Some(ViewMode::LargeCover),
            "details" => Some(ViewMode::Details),
            _ => None,
        }
    }
}

impl SortBy {
    pub fn to_string(&self) -> String {
        match self {
            SortBy::Title => "title".to_string(),
            SortBy::Author => "author".to_string(),
            SortBy::DateAdded => "date_added".to_string(),
            SortBy::DateModified => "date_modified".to_string(),
            SortBy::ReadingProgress => "reading_progress".to_string(),
            SortBy::Rating => "rating".to_string(),
            SortBy::FileSize => "file_size".to_string(),
            SortBy::PublicationDate => "publication_date".to_string(),
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            SortBy::Title => "Title",
            SortBy::Author => "Author",
            SortBy::DateAdded => "Date Added",
            SortBy::DateModified => "Date Modified",
            SortBy::ReadingProgress => "Reading Progress",
            SortBy::Rating => "Rating",
            SortBy::FileSize => "File Size",
            SortBy::PublicationDate => "Publication Date",
        }
    }
}

impl SortOrder {
    pub fn to_string(&self) -> String {
        match self {
            SortOrder::Ascending => "asc".to_string(),
            SortOrder::Descending => "desc".to_string(),
        }
    }
    
    pub fn display_name(&self) -> &'static str {
        match self {
            SortOrder::Ascending => "Ascending",
            SortOrder::Descending => "Descending",
        }
    }
}

impl SyncProvider {
    pub fn to_string(&self) -> String {
        match self {
            SyncProvider::None => "none".to_string(),
            SyncProvider::GoogleDrive => "google_drive".to_string(),
            SyncProvider::Dropbox => "dropbox".to_string(),
            SyncProvider::OneDrive => "onedrive".to_string(),
            SyncProvider::iCloud => "icloud".to_string(),
            SyncProvider::Custom(name) => format!("custom_{}", name),
        }
    }
    
    pub fn display_name(&self) -> String {
        match self {
            SyncProvider::None => "None".to_string(),
            SyncProvider::GoogleDrive => "Google Drive".to_string(),
            SyncProvider::Dropbox => "Dropbox".to_string(),
            SyncProvider::OneDrive => "OneDrive".to_string(),
            SyncProvider::iCloud => "iCloud".to_string(),
            SyncProvider::Custom(name) => name.clone(),
        }
    }
}