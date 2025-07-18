use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

use crate::models::annotation::{Annotation, Bookmark};
use crate::models::reading_theme::ReadingThemePreferences;
use crate::models::library::{Collection, ReadingStatus};

/// Main synchronization data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncData {
    pub device_id: String,
    pub device_name: String,
    pub last_sync: DateTime<Utc>,
    pub sync_version: u32,
    pub books_progress: HashMap<String, BookProgress>,
    pub annotations: Vec<Annotation>,
    pub bookmarks: Vec<Bookmark>,
    pub collections: Vec<Collection>,
    pub preferences: UserPreferences,
    pub reading_sessions: Vec<ReadingSession>,
    pub sync_metadata: SyncMetadata,
}

/// Book reading progress
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookProgress {
    pub book_id: String,
    pub current_page: u32,
    pub total_pages: u32,
    pub current_chapter: Option<String>,
    pub position_in_chapter: Option<usize>,
    pub reading_percentage: f32,
    pub reading_status: ReadingStatus,
    pub last_read_at: DateTime<Utc>,
    pub reading_time_minutes: u32,
    pub notes: Option<String>,
    pub rating: Option<u8>,
    pub started_at: Option<DateTime<Utc>>,
    pub finished_at: Option<DateTime<Utc>>,
    pub reading_goals: Option<BookReadingGoals>,
}

/// User preferences for synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserPreferences {
    pub reading_preferences: ReadingThemePreferences,
    pub app_settings: AppSettings,
    pub sync_settings: SyncSettings,
    pub privacy_settings: PrivacySettings,
}

/// Application settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub language: String,
    pub timezone: String,
    pub auto_save_interval: u32,
    pub backup_frequency: String,
    pub notification_settings: NotificationSettings,
    pub library_settings: LibrarySettings,
    pub reading_behavior: ReadingBehaviorSettings,
}

/// Synchronization settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncSettings {
    pub auto_sync: bool,
    pub sync_interval_minutes: u32,
    pub sync_on_startup: bool,
    pub sync_on_close: bool,
    pub sync_wifi_only: bool,
    pub max_conflict_resolution_attempts: u32,
    pub sync_categories: SyncCategories,
}

/// Privacy settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrivacySettings {
    pub sync_reading_progress: bool,
    pub sync_annotations: bool,
    pub sync_bookmarks: bool,
    pub sync_collections: bool,
    pub sync_preferences: bool,
    pub sync_reading_sessions: bool,
    pub anonymize_data: bool,
    pub data_retention_days: u32,
}

/// Notification settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationSettings {
    pub reading_reminders: bool,
    pub goal_progress: bool,
    pub sync_status: bool,
    pub new_books: bool,
    pub updates: bool,
    pub quiet_hours: Option<QuietHours>,
}

/// Library settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibrarySettings {
    pub default_view: String,
    pub sort_by: String,
    pub sort_direction: String,
    pub show_covers: bool,
    pub grid_size: u32,
    pub auto_import: bool,
    pub duplicate_detection: bool,
    pub metadata_update: bool,
}

/// Reading behavior settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingBehaviorSettings {
    pub auto_bookmark: bool,
    pub auto_night_mode: bool,
    pub page_turn_animation: bool,
    pub reading_timer: bool,
    pub progress_tracking: bool,
    pub chapter_progress: bool,
    pub reading_goals: bool,
}

/// Quiet hours configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuietHours {
    pub enabled: bool,
    pub start_time: String, // HH:MM format
    pub end_time: String,   // HH:MM format
    pub days: Vec<String>,  // Days of week
}

/// Sync categories configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncCategories {
    pub progress: bool,
    pub annotations: bool,
    pub bookmarks: bool,
    pub collections: bool,
    pub preferences: bool,
    pub reading_sessions: bool,
    pub library_metadata: bool,
}

/// Reading session tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingSession {
    pub id: String,
    pub book_id: String,
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration_minutes: u32,
    pub pages_read: u32,
    pub start_page: u32,
    pub end_page: u32,
    pub start_chapter: Option<String>,
    pub end_chapter: Option<String>,
    pub device_id: String,
    pub reading_environment: ReadingEnvironment,
    pub interruptions: Vec<ReadingInterruption>,
    pub notes: Option<String>,
}

/// Reading environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingEnvironment {
    pub theme: String,
    pub font_size: u16,
    pub brightness: f32,
    pub location: Option<String>,
    pub time_of_day: String,
}

/// Reading interruption tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingInterruption {
    pub timestamp: DateTime<Utc>,
    pub duration_seconds: u32,
    pub reason: Option<String>,
}

/// Book reading goals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookReadingGoals {
    pub target_completion_date: Option<DateTime<Utc>>,
    pub daily_pages_goal: Option<u32>,
    pub daily_minutes_goal: Option<u32>,
    pub current_streak: u32,
    pub longest_streak: u32,
}

/// Sync metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMetadata {
    pub checksum: String,
    pub file_size: u64,
    pub compression_used: bool,
    pub encryption_used: bool,
    pub sync_conflicts: Vec<SyncConflict>,
    pub sync_history: Vec<SyncHistoryEntry>,
}

/// Sync conflict information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConflict {
    pub id: String,
    pub conflict_type: SyncConflictType,
    pub local_version: ConflictVersion,
    pub remote_version: ConflictVersion,
    pub resolution: Option<ConflictResolution>,
    pub created_at: DateTime<Utc>,
    pub resolved_at: Option<DateTime<Utc>>,
}

/// Types of sync conflicts
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SyncConflictType {
    BookProgress,
    Annotation,
    Bookmark,
    Collection,
    Preferences,
    ReadingSession,
}

/// Conflict version information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictVersion {
    pub timestamp: DateTime<Utc>,
    pub device_id: String,
    pub data_hash: String,
    pub version_number: u32,
}

/// Conflict resolution strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConflictResolution {
    UseLocal,
    UseRemote,
    Merge,
    Manual(String), // JSON representation of merged data
}

/// Sync history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncHistoryEntry {
    pub timestamp: DateTime<Utc>,
    pub sync_type: SyncType,
    pub status: SyncStatus,
    pub items_synced: u32,
    pub conflicts_resolved: u32,
    pub error_message: Option<String>,
    pub duration_ms: u64,
}

/// Types of sync operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncType {
    Manual,
    Automatic,
    Startup,
    Shutdown,
    Periodic,
}

/// Sync operation status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncStatus {
    Success,
    Partial,
    Failed,
    Cancelled,
}

/// Sync statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncStatistics {
    pub total_syncs: u32,
    pub successful_syncs: u32,
    pub failed_syncs: u32,
    pub last_successful_sync: Option<DateTime<Utc>>,
    pub last_failed_sync: Option<DateTime<Utc>>,
    pub average_sync_duration_ms: u64,
    pub total_conflicts: u32,
    pub resolved_conflicts: u32,
    pub data_transferred_bytes: u64,
    pub sync_efficiency: f32,
}

/// Cloud sync configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudSyncConfig {
    pub provider: CloudProvider,
    pub endpoint: String,
    pub credentials: CloudCredentials,
    pub encryption_key: Option<String>,
    pub sync_path: String,
    pub retry_attempts: u32,
    pub timeout_seconds: u32,
}

/// Cloud providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProvider {
    None,
    Custom,
    GoogleDrive,
    Dropbox,
    OneDrive,
    ICloud,
    AmazonS3,
}

/// Cloud credentials
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudCredentials {
    pub access_token: Option<String>,
    pub refresh_token: Option<String>,
    pub api_key: Option<String>,
    pub secret_key: Option<String>,
    pub expires_at: Option<DateTime<Utc>>,
}

impl SyncData {
    /// Create new sync data for a device
    pub fn new(device_id: String, device_name: String) -> Self {
        Self {
            device_id,
            device_name,
            last_sync: Utc::now(),
            sync_version: 1,
            books_progress: HashMap::new(),
            annotations: Vec::new(),
            bookmarks: Vec::new(),
            collections: Vec::new(),
            preferences: UserPreferences::default(),
            reading_sessions: Vec::new(),
            sync_metadata: SyncMetadata::default(),
        }
    }

    /// Update book progress
    pub fn update_book_progress(&mut self, book_id: String, progress: BookProgress) {
        self.books_progress.insert(book_id, progress);
        self.last_sync = Utc::now();
    }

    /// Add annotation
    pub fn add_annotation(&mut self, annotation: Annotation) {
        self.annotations.push(annotation);
        self.last_sync = Utc::now();
    }

    /// Add bookmark
    pub fn add_bookmark(&mut self, bookmark: Bookmark) {
        self.bookmarks.push(bookmark);
        self.last_sync = Utc::now();
    }

    /// Add collection
    pub fn add_collection(&mut self, collection: Collection) {
        self.collections.push(collection);
        self.last_sync = Utc::now();
    }

    /// Add reading session
    pub fn add_reading_session(&mut self, session: ReadingSession) {
        self.reading_sessions.push(session);
        self.last_sync = Utc::now();
    }

    /// Get sync data size in bytes
    pub fn get_size(&self) -> usize {
        serde_json::to_string(self).unwrap_or_default().len()
    }

    /// Get data checksum
    pub fn get_checksum(&self) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        let data_str = serde_json::to_string(self).unwrap_or_default();
        data_str.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Check if data is valid
    pub fn is_valid(&self) -> bool {
        !self.device_id.is_empty() && self.sync_version > 0
    }

    /// Clean old data
    pub fn clean_old_data(&mut self, retention_days: u32) {
        let cutoff_date = Utc::now() - chrono::Duration::days(retention_days as i64);

        // Remove old reading sessions
        self.reading_sessions.retain(|session| session.start_time > cutoff_date);

        // Remove old sync history
        self.sync_metadata.sync_history.retain(|entry| entry.timestamp > cutoff_date);
    }
}

impl Default for UserPreferences {
    fn default() -> Self {
        Self {
            reading_preferences: ReadingThemePreferences::default(),
            app_settings: AppSettings::default(),
            sync_settings: SyncSettings::default(),
            privacy_settings: PrivacySettings::default(),
        }
    }
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            language: "en".to_string(),
            timezone: "UTC".to_string(),
            auto_save_interval: 30,
            backup_frequency: "daily".to_string(),
            notification_settings: NotificationSettings::default(),
            library_settings: LibrarySettings::default(),
            reading_behavior: ReadingBehaviorSettings::default(),
        }
    }
}

impl Default for SyncSettings {
    fn default() -> Self {
        Self {
            auto_sync: true,
            sync_interval_minutes: 15,
            sync_on_startup: true,
            sync_on_close: true,
            sync_wifi_only: false,
            max_conflict_resolution_attempts: 3,
            sync_categories: SyncCategories::default(),
        }
    }
}

impl Default for PrivacySettings {
    fn default() -> Self {
        Self {
            sync_reading_progress: true,
            sync_annotations: true,
            sync_bookmarks: true,
            sync_collections: true,
            sync_preferences: true,
            sync_reading_sessions: true,
            anonymize_data: false,
            data_retention_days: 365,
        }
    }
}

impl Default for NotificationSettings {
    fn default() -> Self {
        Self {
            reading_reminders: true,
            goal_progress: true,
            sync_status: false,
            new_books: true,
            updates: true,
            quiet_hours: None,
        }
    }
}

impl Default for LibrarySettings {
    fn default() -> Self {
        Self {
            default_view: "grid".to_string(),
            sort_by: "title".to_string(),
            sort_direction: "asc".to_string(),
            show_covers: true,
            grid_size: 3,
            auto_import: true,
            duplicate_detection: true,
            metadata_update: true,
        }
    }
}

impl Default for ReadingBehaviorSettings {
    fn default() -> Self {
        Self {
            auto_bookmark: true,
            auto_night_mode: false,
            page_turn_animation: true,
            reading_timer: true,
            progress_tracking: true,
            chapter_progress: true,
            reading_goals: true,
        }
    }
}

impl Default for SyncCategories {
    fn default() -> Self {
        Self {
            progress: true,
            annotations: true,
            bookmarks: true,
            collections: true,
            preferences: true,
            reading_sessions: true,
            library_metadata: true,
        }
    }
}

impl Default for SyncMetadata {
    fn default() -> Self {
        Self {
            checksum: String::new(),
            file_size: 0,
            compression_used: false,
            encryption_used: false,
            sync_conflicts: Vec::new(),
            sync_history: Vec::new(),
        }
    }
}

impl BookProgress {
    /// Create new book progress
    pub fn new(book_id: String) -> Self {
        Self {
            book_id,
            current_page: 1,
            total_pages: 1,
            current_chapter: None,
            position_in_chapter: None,
            reading_percentage: 0.0,
            reading_status: ReadingStatus::WantToRead,
            last_read_at: Utc::now(),
            reading_time_minutes: 0,
            notes: None,
            rating: None,
            started_at: None,
            finished_at: None,
            reading_goals: None,
        }
    }

    /// Update reading progress
    pub fn update_progress(&mut self, page: u32, total_pages: u32) {
        self.current_page = page;
        self.total_pages = total_pages;
        self.reading_percentage = (page as f32 / total_pages as f32) * 100.0;
        self.last_read_at = Utc::now();
    }

    /// Mark as started
    pub fn start_reading(&mut self) {
        if self.started_at.is_none() {
            self.started_at = Some(Utc::now());
        }
        self.reading_status = ReadingStatus::CurrentlyReading;
    }

    /// Mark as finished
    pub fn finish_reading(&mut self) {
        self.finished_at = Some(Utc::now());
        self.reading_status = ReadingStatus::Finished;
        self.reading_percentage = 100.0;
    }

    /// Add reading time
    pub fn add_reading_time(&mut self, minutes: u32) {
        self.reading_time_minutes += minutes;
        self.last_read_at = Utc::now();
    }

    /// Calculate reading speed (pages per minute)
    pub fn get_reading_speed(&self) -> f32 {
        if self.reading_time_minutes > 0 {
            self.current_page as f32 / self.reading_time_minutes as f32
        } else {
            0.0
        }
    }

    /// Get estimated time to finish
    pub fn get_estimated_time_to_finish(&self) -> Option<u32> {
        let reading_speed = self.get_reading_speed();
        if reading_speed > 0.0 {
            let remaining_pages = self.total_pages - self.current_page;
            Some((remaining_pages as f32 / reading_speed) as u32)
        } else {
            None
        }
    }
}

impl ReadingSession {
    /// Create new reading session
    pub fn new(book_id: String, device_id: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            book_id,
            start_time: Utc::now(),
            end_time: None,
            duration_minutes: 0,
            pages_read: 0,
            start_page: 1,
            end_page: 1,
            start_chapter: None,
            end_chapter: None,
            device_id,
            reading_environment: ReadingEnvironment::default(),
            interruptions: Vec::new(),
            notes: None,
        }
    }

    /// End reading session
    pub fn end_session(&mut self, end_page: u32) {
        self.end_time = Some(Utc::now());
        self.end_page = end_page;
        self.pages_read = end_page - self.start_page + 1;
        
        if let Some(end_time) = self.end_time {
            let duration = end_time - self.start_time;
            self.duration_minutes = duration.num_minutes() as u32;
        }
    }

    /// Add interruption
    pub fn add_interruption(&mut self, duration_seconds: u32, reason: Option<String>) {
        self.interruptions.push(ReadingInterruption {
            timestamp: Utc::now(),
            duration_seconds,
            reason,
        });
    }

    /// Get effective reading time (total time minus interruptions)
    pub fn get_effective_reading_time(&self) -> u32 {
        let total_interruption_time: u32 = self.interruptions
            .iter()
            .map(|i| i.duration_seconds)
            .sum();
        
        if self.duration_minutes * 60 > total_interruption_time {
            self.duration_minutes * 60 - total_interruption_time
        } else {
            0
        }
    }
}

impl Default for ReadingEnvironment {
    fn default() -> Self {
        Self {
            theme: "light".to_string(),
            font_size: 16,
            brightness: 1.0,
            location: None,
            time_of_day: "day".to_string(),
        }
    }
}

impl SyncConflict {
    /// Create new sync conflict
    pub fn new(
        conflict_type: SyncConflictType,
        local_version: ConflictVersion,
        remote_version: ConflictVersion,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            conflict_type,
            local_version,
            remote_version,
            resolution: None,
            created_at: Utc::now(),
            resolved_at: None,
        }
    }

    /// Resolve conflict
    pub fn resolve(&mut self, resolution: ConflictResolution) {
        self.resolution = Some(resolution);
        self.resolved_at = Some(Utc::now());
    }

    /// Check if conflict is resolved
    pub fn is_resolved(&self) -> bool {
        self.resolution.is_some()
    }
}