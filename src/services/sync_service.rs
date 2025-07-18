use std::path::Path;
use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use chrono::{DateTime, Utc};
use tokio::sync::RwLock;
use tokio::fs;
use serde_json;

use crate::models::sync::{
    SyncData, BookProgress, ReadingSession, SyncConflict, SyncConflictType,
    ConflictVersion, ConflictResolution, SyncHistoryEntry, SyncType, SyncStatus,
    SyncStatistics, CloudSyncConfig, UserPreferences,
};
use crate::models::annotation::{Annotation, Bookmark};
use crate::models::library::{Collection, ReadingStatus};
use crate::services::annotation_service::AnnotationService;
use crate::services::library_service::LibraryService;

/// Synchronization service for managing reading progress and data sync
pub struct SyncService {
    local_data: Arc<RwLock<SyncData>>,
    sync_statistics: Arc<RwLock<SyncStatistics>>,
    cloud_config: Arc<RwLock<Option<CloudSyncConfig>>>,
    annotation_service: Arc<AnnotationService>,
    library_service: Arc<LibraryService>,
    device_id: String,
    device_name: String,
}

impl SyncService {
    pub fn new(
        annotation_service: Arc<AnnotationService>,
        library_service: Arc<LibraryService>,
        device_id: String,
        device_name: String,
    ) -> Self {
        let local_data = SyncData::new(device_id.clone(), device_name.clone());
        
        Self {
            local_data: Arc::new(RwLock::new(local_data)),
            sync_statistics: Arc::new(RwLock::new(SyncStatistics::default())),
            cloud_config: Arc::new(RwLock::new(None)),
            annotation_service,
            library_service,
            device_id,
            device_name,
        }
    }

    /// Initialize sync service
    pub async fn initialize(&self) -> Result<()> {
        // Load local sync data if exists
        let sync_file_path = self.get_sync_file_path();
        if sync_file_path.exists() {
            self.sync_from_file(&sync_file_path).await?;
        }

        // Initialize with current data
        self.collect_local_data().await?;

        Ok(())
    }

    /// Sync data to file
    pub async fn sync_to_file(&self, path: &Path) -> Result<()> {
        let data = self.local_data.read().await;
        let json_data = serde_json::to_string_pretty(&*data)?;
        fs::write(path, json_data).await?;

        // Update sync statistics
        self.update_sync_statistics(SyncType::Manual, SyncStatus::Success, 0).await;

        Ok(())
    }

    /// Sync data from file
    pub async fn sync_from_file(&self, path: &Path) -> Result<()> {
        let json_data = fs::read_to_string(path).await?;
        let remote_data: SyncData = serde_json::from_str(&json_data)?;

        // Merge with local data
        self.merge_sync_data(remote_data).await?;

        // Update sync statistics
        self.update_sync_statistics(SyncType::Manual, SyncStatus::Success, 0).await;

        Ok(())
    }

    /// Merge conflicts between local and remote data
    pub async fn merge_conflicts(&self, local: SyncData, remote: SyncData) -> Result<SyncData> {
        let mut merged = local.clone();
        let mut conflicts = Vec::new();

        // Merge book progress
        for (book_id, remote_progress) in remote.books_progress {
            if let Some(local_progress) = local.books_progress.get(&book_id) {
                let conflict_result = self.resolve_progress_conflict(local_progress, &remote_progress).await;
                match conflict_result {
                    Ok(resolved_progress) => {
                        merged.books_progress.insert(book_id, resolved_progress);
                    }
                    Err(conflict) => {
                        conflicts.push(conflict);
                    }
                }
            } else {
                merged.books_progress.insert(book_id, remote_progress);
            }
        }

        // Merge annotations
        let annotation_conflicts = self.merge_annotations(&local.annotations, &remote.annotations).await?;
        merged.annotations = annotation_conflicts.0;
        conflicts.extend(annotation_conflicts.1);

        // Merge bookmarks
        let bookmark_conflicts = self.merge_bookmarks(&local.bookmarks, &remote.bookmarks).await?;
        merged.bookmarks = bookmark_conflicts.0;
        conflicts.extend(bookmark_conflicts.1);

        // Merge collections
        let collection_conflicts = self.merge_collections(&local.collections, &remote.collections).await?;
        merged.collections = collection_conflicts.0;
        conflicts.extend(collection_conflicts.1);

        // Merge preferences
        if remote.preferences.reading_preferences.last_updated > local.preferences.reading_preferences.last_updated {
            merged.preferences = remote.preferences;
        }

        // Merge reading sessions
        merged.reading_sessions = self.merge_reading_sessions(&local.reading_sessions, &remote.reading_sessions).await?;

        // Update sync metadata
        merged.sync_metadata.sync_conflicts = conflicts;
        merged.last_sync = Utc::now();

        Ok(merged)
    }

    /// Resolve book progress conflict
    async fn resolve_progress_conflict(
        &self,
        local: &BookProgress,
        remote: &BookProgress,
    ) -> Result<BookProgress, SyncConflict> {
        // Use the most recent progress
        if local.last_read_at > remote.last_read_at {
            Ok(local.clone())
        } else if remote.last_read_at > local.last_read_at {
            Ok(remote.clone())
        } else {
            // If timestamps are equal, use the furthest progress
            if local.current_page >= remote.current_page {
                Ok(local.clone())
            } else {
                Ok(remote.clone())
            }
        }
    }

    /// Merge annotations
    async fn merge_annotations(
        &self,
        local: &[Annotation],
        remote: &[Annotation],
    ) -> Result<(Vec<Annotation>, Vec<SyncConflict>)> {
        let mut merged = Vec::new();
        let mut conflicts = Vec::new();
        let mut processed_ids = std::collections::HashSet::new();

        // Process local annotations
        for local_annotation in local {
            if let Some(remote_annotation) = remote.iter().find(|a| a.id == local_annotation.id) {
                // Conflict resolution
                if local_annotation.modified_at > remote_annotation.modified_at {
                    merged.push(local_annotation.clone());
                } else {
                    merged.push(remote_annotation.clone());
                }
            } else {
                merged.push(local_annotation.clone());
            }
            processed_ids.insert(local_annotation.id.clone());
        }

        // Process remote annotations not in local
        for remote_annotation in remote {
            if !processed_ids.contains(&remote_annotation.id) {
                merged.push(remote_annotation.clone());
            }
        }

        Ok((merged, conflicts))
    }

    /// Merge bookmarks
    async fn merge_bookmarks(
        &self,
        local: &[Bookmark],
        remote: &[Bookmark],
    ) -> Result<(Vec<Bookmark>, Vec<SyncConflict>)> {
        let mut merged = Vec::new();
        let mut conflicts = Vec::new();
        let mut processed_ids = std::collections::HashSet::new();

        // Process local bookmarks
        for local_bookmark in local {
            if let Some(remote_bookmark) = remote.iter().find(|b| b.id == local_bookmark.id) {
                // Use the most recent bookmark
                if local_bookmark.created_at > remote_bookmark.created_at {
                    merged.push(local_bookmark.clone());
                } else {
                    merged.push(remote_bookmark.clone());
                }
            } else {
                merged.push(local_bookmark.clone());
            }
            processed_ids.insert(local_bookmark.id.clone());
        }

        // Process remote bookmarks not in local
        for remote_bookmark in remote {
            if !processed_ids.contains(&remote_bookmark.id) {
                merged.push(remote_bookmark.clone());
            }
        }

        Ok((merged, conflicts))
    }

    /// Merge collections
    async fn merge_collections(
        &self,
        local: &[Collection],
        remote: &[Collection],
    ) -> Result<(Vec<Collection>, Vec<SyncConflict>)> {
        let mut merged = Vec::new();
        let mut conflicts = Vec::new();
        let mut processed_ids = std::collections::HashSet::new();

        // Process local collections
        for local_collection in local {
            if let Some(remote_collection) = remote.iter().find(|c| c.id == local_collection.id) {
                // Use the most recent collection
                if local_collection.updated_at > remote_collection.updated_at {
                    merged.push(local_collection.clone());
                } else {
                    merged.push(remote_collection.clone());
                }
            } else {
                merged.push(local_collection.clone());
            }
            processed_ids.insert(local_collection.id.clone());
        }

        // Process remote collections not in local
        for remote_collection in remote {
            if !processed_ids.contains(&remote_collection.id) {
                merged.push(remote_collection.clone());
            }
        }

        Ok((merged, conflicts))
    }

    /// Merge reading sessions
    async fn merge_reading_sessions(
        &self,
        local: &[ReadingSession],
        remote: &[ReadingSession],
    ) -> Result<Vec<ReadingSession>> {
        let mut merged = Vec::new();
        let mut processed_ids = std::collections::HashSet::new();

        // Add all local sessions
        for session in local {
            merged.push(session.clone());
            processed_ids.insert(session.id.clone());
        }

        // Add remote sessions not in local
        for session in remote {
            if !processed_ids.contains(&session.id) {
                merged.push(session.clone());
            }
        }

        // Sort by start time
        merged.sort_by(|a, b| a.start_time.cmp(&b.start_time));

        Ok(merged)
    }

    /// Update book progress
    pub async fn update_book_progress(&self, book_id: String, progress: BookProgress) -> Result<()> {
        let mut data = self.local_data.write().await;
        data.update_book_progress(book_id, progress);
        
        // Auto-save if enabled
        if self.should_auto_save().await {
            self.save_local_data().await?;
        }

        Ok(())
    }

    /// Get book progress
    pub async fn get_book_progress(&self, book_id: &str) -> Option<BookProgress> {
        let data = self.local_data.read().await;
        data.books_progress.get(book_id).cloned()
    }

    /// Start reading session
    pub async fn start_reading_session(&self, book_id: String) -> Result<String> {
        let session = ReadingSession::new(book_id, self.device_id.clone());
        let session_id = session.id.clone();
        
        let mut data = self.local_data.write().await;
        data.add_reading_session(session);
        
        Ok(session_id)
    }

    /// End reading session
    pub async fn end_reading_session(&self, session_id: &str, end_page: u32) -> Result<()> {
        let mut data = self.local_data.write().await;
        
        if let Some(session) = data.reading_sessions.iter_mut().find(|s| s.id == session_id) {
            session.end_session(end_page);
        }
        
        Ok(())
    }

    /// Get reading sessions for a book
    pub async fn get_reading_sessions(&self, book_id: &str) -> Vec<ReadingSession> {
        let data = self.local_data.read().await;
        data.reading_sessions
            .iter()
            .filter(|s| s.book_id == book_id)
            .cloned()
            .collect()
    }

    /// Get reading statistics
    pub async fn get_reading_statistics(&self, book_id: Option<&str>) -> HashMap<String, u32> {
        let data = self.local_data.read().await;
        let mut stats = HashMap::new();

        let sessions: Vec<_> = if let Some(book_id) = book_id {
            data.reading_sessions
                .iter()
                .filter(|s| s.book_id == book_id)
                .collect()
        } else {
            data.reading_sessions.iter().collect()
        };

        // Total reading time
        let total_time: u32 = sessions.iter().map(|s| s.duration_minutes).sum();
        stats.insert("total_reading_time".to_string(), total_time);

        // Total pages read
        let total_pages: u32 = sessions.iter().map(|s| s.pages_read).sum();
        stats.insert("total_pages_read".to_string(), total_pages);

        // Number of sessions
        stats.insert("total_sessions".to_string(), sessions.len() as u32);

        // Average session length
        if !sessions.is_empty() {
            stats.insert("average_session_length".to_string(), total_time / sessions.len() as u32);
        }

        // Books in different states
        let want_to_read = data.books_progress.values().filter(|p| p.reading_status == ReadingStatus::WantToRead).count();
        let currently_reading = data.books_progress.values().filter(|p| p.reading_status == ReadingStatus::CurrentlyReading).count();
        let finished = data.books_progress.values().filter(|p| p.reading_status == ReadingStatus::Finished).count();

        stats.insert("want_to_read".to_string(), want_to_read as u32);
        stats.insert("currently_reading".to_string(), currently_reading as u32);
        stats.insert("finished".to_string(), finished as u32);

        stats
    }

    /// Get sync statistics
    pub async fn get_sync_statistics(&self) -> SyncStatistics {
        let stats = self.sync_statistics.read().await;
        stats.clone()
    }

    /// Update user preferences
    pub async fn update_preferences(&self, preferences: UserPreferences) -> Result<()> {
        let mut data = self.local_data.write().await;
        data.preferences = preferences;
        data.last_sync = Utc::now();
        
        // Auto-save if enabled
        if self.should_auto_save().await {
            self.save_local_data().await?;
        }

        Ok(())
    }

    /// Get user preferences
    pub async fn get_preferences(&self) -> UserPreferences {
        let data = self.local_data.read().await;
        data.preferences.clone()
    }

    /// Perform full sync
    pub async fn perform_full_sync(&self) -> Result<()> {
        let start_time = std::time::Instant::now();
        
        // Collect local data
        self.collect_local_data().await?;
        
        // Save to file
        let sync_file_path = self.get_sync_file_path();
        self.sync_to_file(&sync_file_path).await?;
        
        let duration = start_time.elapsed().as_millis() as u64;
        
        // Update statistics
        self.update_sync_statistics(SyncType::Manual, SyncStatus::Success, duration).await;
        
        Ok(())
    }

    /// Collect local data from services
    async fn collect_local_data(&self) -> Result<()> {
        let mut data = self.local_data.write().await;
        
        // Collect annotations
        // This would integrate with your annotation service
        // data.annotations = self.annotation_service.get_all_annotations().await?;
        
        // Collect bookmarks  
        // data.bookmarks = self.annotation_service.get_all_bookmarks().await?;
        
        // Collect collections
        // data.collections = self.library_service.get_all_collections().await?;
        
        Ok(())
    }

    /// Merge sync data
    async fn merge_sync_data(&self, remote_data: SyncData) -> Result<()> {
        let local_data = self.local_data.read().await.clone();
        drop(local_data);
        
        let merged_data = self.merge_conflicts(local_data, remote_data).await?;
        
        let mut data = self.local_data.write().await;
        *data = merged_data;
        
        Ok(())
    }

    /// Save local data
    async fn save_local_data(&self) -> Result<()> {
        let sync_file_path = self.get_sync_file_path();
        self.sync_to_file(&sync_file_path).await
    }

    /// Get sync file path
    fn get_sync_file_path(&self) -> std::path::PathBuf {
        let mut path = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("."));
        path.push("ebook-reader");
        path.push("sync");
        path.push("sync_data.json");
        path
    }

    /// Check if auto-save is enabled
    async fn should_auto_save(&self) -> bool {
        let data = self.local_data.read().await;
        data.preferences.app_settings.auto_save_interval > 0
    }

    /// Update sync statistics
    async fn update_sync_statistics(
        &self,
        sync_type: SyncType,
        status: SyncStatus,
        duration_ms: u64,
    ) {
        let mut stats = self.sync_statistics.write().await;
        
        stats.total_syncs += 1;
        
        match status {
            SyncStatus::Success => {
                stats.successful_syncs += 1;
                stats.last_successful_sync = Some(Utc::now());
            }
            SyncStatus::Failed => {
                stats.failed_syncs += 1;
                stats.last_failed_sync = Some(Utc::now());
            }
            _ => {}
        }
        
        // Update average duration
        let total_duration = stats.average_sync_duration_ms * (stats.total_syncs - 1) as u64 + duration_ms;
        stats.average_sync_duration_ms = total_duration / stats.total_syncs as u64;
        
        // Add to history
        let mut data = self.local_data.write().await;
        data.sync_metadata.sync_history.push(SyncHistoryEntry {
            timestamp: Utc::now(),
            sync_type,
            status,
            items_synced: 0, // Would be calculated based on actual sync
            conflicts_resolved: 0,
            error_message: None,
            duration_ms,
        });
        
        // Keep only last 100 history entries
        if data.sync_metadata.sync_history.len() > 100 {
            data.sync_metadata.sync_history.remove(0);
        }
    }

    /// Export sync data
    pub async fn export_sync_data(&self, path: &Path) -> Result<()> {
        let data = self.local_data.read().await;
        let json_data = serde_json::to_string_pretty(&*data)?;
        fs::write(path, json_data).await?;
        Ok(())
    }

    /// Import sync data
    pub async fn import_sync_data(&self, path: &Path) -> Result<()> {
        let json_data = fs::read_to_string(path).await?;
        let imported_data: SyncData = serde_json::from_str(&json_data)?;
        
        // Merge with existing data
        self.merge_sync_data(imported_data).await?;
        
        Ok(())
    }

    /// Clean old sync data
    pub async fn clean_old_data(&self, retention_days: u32) -> Result<()> {
        let mut data = self.local_data.write().await;
        data.clean_old_data(retention_days);
        Ok(())
    }

    /// Get device info
    pub async fn get_device_info(&self) -> (String, String) {
        (self.device_id.clone(), self.device_name.clone())
    }

    /// Check sync data integrity
    pub async fn check_data_integrity(&self) -> Result<bool> {
        let data = self.local_data.read().await;
        Ok(data.is_valid())
    }

    /// Get sync conflicts
    pub async fn get_sync_conflicts(&self) -> Vec<SyncConflict> {
        let data = self.local_data.read().await;
        data.sync_metadata.sync_conflicts.clone()
    }

    /// Resolve sync conflict
    pub async fn resolve_sync_conflict(&self, conflict_id: &str, resolution: ConflictResolution) -> Result<()> {
        let mut data = self.local_data.write().await;
        
        if let Some(conflict) = data.sync_metadata.sync_conflicts.iter_mut().find(|c| c.id == conflict_id) {
            conflict.resolve(resolution);
        }
        
        Ok(())
    }
}

impl Default for SyncStatistics {
    fn default() -> Self {
        Self {
            total_syncs: 0,
            successful_syncs: 0,
            failed_syncs: 0,
            last_successful_sync: None,
            last_failed_sync: None,
            average_sync_duration_ms: 0,
            total_conflicts: 0,
            resolved_conflicts: 0,
            data_transferred_bytes: 0,
            sync_efficiency: 0.0,
        }
    }
}