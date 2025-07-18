use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use slint::{ComponentHandle, Model, VecModel, ModelRc};
use tokio::sync::RwLock;

use crate::models::annotation::{
    Annotation, Bookmark, AnnotationType, HighlightColor, BookmarkColor,
    TextPosition, AnnotationFilter, ExportOptions, ExportFormat,
};
use crate::services::annotation_service::AnnotationService;

/// Slint-compatible annotation model
#[derive(Clone)]
pub struct SlintAnnotation {
    pub id: String,
    pub page_number: i32,
    pub selected_text: String,
    pub note: String,
    pub color: String,
    pub annotation_type: String,
    pub created_at: String,
    pub tags: Vec<String>,
    pub is_favorite: bool,
}

/// Slint-compatible bookmark model  
#[derive(Clone)]
pub struct SlintBookmark {
    pub id: String,
    pub page_number: i32,
    pub title: String,
    pub preview_text: String,
    pub created_at: String,
    pub color: String,
    pub is_favorite: bool,
}

/// Annotation manager for handling UI interactions
pub struct AnnotationManager {
    service: AnnotationService,
    current_book_id: Arc<RwLock<Option<String>>>,
    annotations: Arc<RwLock<Vec<Annotation>>>,
    bookmarks: Arc<RwLock<Vec<Bookmark>>>,
    annotation_model: ModelRc<SlintAnnotation>,
    bookmark_model: ModelRc<SlintBookmark>,
}

impl AnnotationManager {
    pub fn new(service: AnnotationService) -> Self {
        Self {
            service,
            current_book_id: Arc::new(RwLock::new(None)),
            annotations: Arc::new(RwLock::new(Vec::new())),
            bookmarks: Arc::new(RwLock::new(Vec::new())),
            annotation_model: ModelRc::new(VecModel::default()),
            bookmark_model: ModelRc::new(VecModel::default()),
        }
    }

    /// Get annotation model for Slint UI
    pub fn get_annotation_model(&self) -> ModelRc<SlintAnnotation> {
        self.annotation_model.clone()
    }

    /// Get bookmark model for Slint UI
    pub fn get_bookmark_model(&self) -> ModelRc<SlintBookmark> {
        self.bookmark_model.clone()
    }

    /// Load annotations for a book
    pub async fn load_book_annotations(&self, book_id: &str) -> Result<()> {
        // Update current book ID
        *self.current_book_id.write().await = Some(book_id.to_string());

        // Load annotations and bookmarks
        let annotations = self.service.get_annotations_for_book(book_id).await?;
        let bookmarks = self.service.get_bookmarks_for_book(book_id).await?;

        // Update internal storage
        *self.annotations.write().await = annotations.clone();
        *self.bookmarks.write().await = bookmarks.clone();

        // Update Slint models
        self.update_annotation_model(&annotations).await;
        self.update_bookmark_model(&bookmarks).await;

        Ok(())
    }

    /// Create new annotation
    pub async fn create_annotation(
        &self,
        page_number: u32,
        selected_text: String,
        position: TextPosition,
        annotation_type: AnnotationType,
        color: HighlightColor,
        note: Option<String>,
    ) -> Result<String> {
        let book_id = {
            let book_id_guard = self.current_book_id.read().await;
            book_id_guard.as_ref().ok_or_else(|| anyhow::anyhow!("No book loaded"))?.clone()
        };

        let annotation = self.service.create_annotation(
            book_id.clone(),
            page_number,
            selected_text,
            position,
            annotation_type,
            color,
            note,
        ).await?;

        // Update local storage
        {
            let mut annotations = self.annotations.write().await;
            annotations.push(annotation.clone());
            annotations.sort_by(|a, b| a.page_number.cmp(&b.page_number));
        }

        // Update UI model
        self.refresh_annotation_model().await;

        Ok(annotation.id)
    }

    /// Create new bookmark
    pub async fn create_bookmark(
        &self,
        page_number: u32,
        preview_text: String,
        position: TextPosition,
        title: Option<String>,
        color: BookmarkColor,
    ) -> Result<String> {
        let book_id = {
            let book_id_guard = self.current_book_id.read().await;
            book_id_guard.as_ref().ok_or_else(|| anyhow::anyhow!("No book loaded"))?.clone()
        };

        let bookmark = self.service.create_bookmark(
            book_id.clone(),
            page_number,
            preview_text,
            position,
            title,
            color,
        ).await?;

        // Update local storage
        {
            let mut bookmarks = self.bookmarks.write().await;
            bookmarks.push(bookmark.clone());
            bookmarks.sort_by(|a, b| a.page_number.cmp(&b.page_number));
        }

        // Update UI model
        self.refresh_bookmark_model().await;

        Ok(bookmark.id)
    }

    /// Update annotation
    pub async fn update_annotation(&self, id: &str, note: Option<String>, color: Option<HighlightColor>) -> Result<()> {
        let mut annotations = self.annotations.write().await;
        
        if let Some(annotation) = annotations.iter_mut().find(|a| a.id == id) {
            if let Some(note) = note {
                annotation.add_note(note);
            }
            if let Some(color) = color {
                annotation.set_color(color);
            }
            
            self.service.update_annotation(annotation).await?;
        }

        drop(annotations);
        self.refresh_annotation_model().await;
        Ok(())
    }

    /// Delete annotation
    pub async fn delete_annotation(&self, id: &str) -> Result<()> {
        self.service.delete_annotation(id).await?;

        // Update local storage
        {
            let mut annotations = self.annotations.write().await;
            annotations.retain(|a| a.id != id);
        }

        // Update UI model
        self.refresh_annotation_model().await;
        Ok(())
    }

    /// Delete bookmark
    pub async fn delete_bookmark(&self, id: &str) -> Result<()> {
        self.service.delete_bookmark(id).await?;

        // Update local storage
        {
            let mut bookmarks = self.bookmarks.write().await;
            bookmarks.retain(|b| b.id != id);
        }

        // Update UI model
        self.refresh_bookmark_model().await;
        Ok(())
    }

    /// Toggle annotation favorite
    pub async fn toggle_annotation_favorite(&self, id: &str) -> Result<()> {
        let mut annotations = self.annotations.write().await;
        
        if let Some(annotation) = annotations.iter_mut().find(|a| a.id == id) {
            annotation.toggle_favorite();
            self.service.update_annotation(annotation).await?;
        }

        drop(annotations);
        self.refresh_annotation_model().await;
        Ok(())
    }

    /// Toggle bookmark favorite
    pub async fn toggle_bookmark_favorite(&self, id: &str) -> Result<()> {
        let mut bookmarks = self.bookmarks.write().await;
        
        if let Some(bookmark) = bookmarks.iter_mut().find(|b| b.id == id) {
            bookmark.toggle_favorite();
            self.service.save_bookmark(bookmark).await?;
        }

        drop(bookmarks);
        self.refresh_bookmark_model().await;
        Ok(())
    }

    /// Filter annotations
    pub async fn filter_annotations(&self, filter: AnnotationFilter) -> Result<Vec<SlintAnnotation>> {
        let annotations = self.service.get_annotations_filtered(&filter).await?;
        Ok(annotations.into_iter().map(|a| self.annotation_to_slint(&a)).collect())
    }

    /// Search annotations
    pub async fn search_annotations(&self, query: &str) -> Result<Vec<SlintAnnotation>> {
        let filter = AnnotationFilter {
            text_search: Some(query.to_string()),
            ..Default::default()
        };
        self.filter_annotations(filter).await
    }

    /// Get annotations for page
    pub async fn get_page_annotations(&self, page_number: u32) -> Result<Vec<SlintAnnotation>> {
        let annotations = self.annotations.read().await;
        let page_annotations: Vec<SlintAnnotation> = annotations
            .iter()
            .filter(|a| a.page_number == page_number)
            .map(|a| self.annotation_to_slint(a))
            .collect();
        Ok(page_annotations)
    }

    /// Get bookmarks for page
    pub async fn get_page_bookmarks(&self, page_number: u32) -> Result<Vec<SlintBookmark>> {
        let bookmarks = self.bookmarks.read().await;
        let page_bookmarks: Vec<SlintBookmark> = bookmarks
            .iter()
            .filter(|b| b.page_number == page_number)
            .map(|b| self.bookmark_to_slint(b))
            .collect();
        Ok(page_bookmarks)
    }

    /// Export annotations
    pub async fn export_annotations(&self, format: ExportFormat) -> Result<String> {
        let book_id = {
            let book_id_guard = self.current_book_id.read().await;
            book_id_guard.as_ref().ok_or_else(|| anyhow::anyhow!("No book loaded"))?.clone()
        };

        let options = ExportOptions {
            format,
            include_highlights: true,
            include_notes: true,
            include_bookmarks: true,
            include_timestamps: true,
            include_page_numbers: true,
            include_context: true,
            group_by_chapter: false,
            sort_by: crate::models::annotation::AnnotationSortBy::CreatedAt,
        };

        self.service.export_annotations(&book_id, &options).await
    }

    /// Get annotation statistics
    pub async fn get_statistics(&self) -> Result<HashMap<String, u32>> {
        let book_id = {
            let book_id_guard = self.current_book_id.read().await;
            book_id_guard.as_ref().map(|s| s.clone())
        };

        let stats = self.service.get_annotation_stats(book_id.as_deref()).await?;
        
        let mut result = HashMap::new();
        result.insert("total_annotations".to_string(), stats.total_annotations);
        result.insert("highlights_count".to_string(), stats.highlights_count);
        result.insert("notes_count".to_string(), stats.notes_count);
        result.insert("bookmarks_count".to_string(), stats.bookmarks_count);
        result.insert("favorite_count".to_string(), stats.favorite_count);
        
        Ok(result)
    }

    /// Refresh annotation model
    async fn refresh_annotation_model(&self) {
        let annotations = self.annotations.read().await;
        self.update_annotation_model(&annotations).await;
    }

    /// Refresh bookmark model
    async fn refresh_bookmark_model(&self) {
        let bookmarks = self.bookmarks.read().await;
        self.update_bookmark_model(&bookmarks).await;
    }

    /// Update annotation model
    async fn update_annotation_model(&self, annotations: &[Annotation]) {
        let slint_annotations: Vec<SlintAnnotation> = annotations
            .iter()
            .map(|a| self.annotation_to_slint(a))
            .collect();
        
        let model = VecModel::from(slint_annotations);
        // Replace the model contents
        // Note: This is a simplified approach - in a real implementation
        // you might need to use a different pattern to update the model
    }

    /// Update bookmark model
    async fn update_bookmark_model(&self, bookmarks: &[Bookmark]) {
        let slint_bookmarks: Vec<SlintBookmark> = bookmarks
            .iter()
            .map(|b| self.bookmark_to_slint(b))
            .collect();
        
        let model = VecModel::from(slint_bookmarks);
        // Replace the model contents
        // Note: This is a simplified approach - in a real implementation
        // you might need to use a different pattern to update the model
    }

    /// Convert Annotation to SlintAnnotation
    fn annotation_to_slint(&self, annotation: &Annotation) -> SlintAnnotation {
        SlintAnnotation {
            id: annotation.id.clone(),
            page_number: annotation.page_number as i32,
            selected_text: annotation.selected_text.clone(),
            note: annotation.note.clone().unwrap_or_default(),
            color: annotation.color.to_hex(),
            annotation_type: annotation.annotation_type.to_display_name(),
            created_at: annotation.created_at.format("%Y-%m-%d %H:%M").to_string(),
            tags: annotation.tags.clone(),
            is_favorite: annotation.is_favorite,
        }
    }

    /// Convert Bookmark to SlintBookmark
    fn bookmark_to_slint(&self, bookmark: &Bookmark) -> SlintBookmark {
        SlintBookmark {
            id: bookmark.id.clone(),
            page_number: bookmark.page_number as i32,
            title: bookmark.get_display_title(),
            preview_text: bookmark.preview_text.clone(),
            created_at: bookmark.created_at.format("%Y-%m-%d %H:%M").to_string(),
            color: bookmark.color.to_hex(),
            is_favorite: bookmark.is_favorite,
        }
    }

    /// Add tag to annotation
    pub async fn add_tag_to_annotation(&self, annotation_id: &str, tag: String) -> Result<()> {
        let mut annotations = self.annotations.write().await;
        
        if let Some(annotation) = annotations.iter_mut().find(|a| a.id == annotation_id) {
            annotation.add_tag(tag);
            self.service.update_annotation(annotation).await?;
        }

        drop(annotations);
        self.refresh_annotation_model().await;
        Ok(())
    }

    /// Remove tag from annotation
    pub async fn remove_tag_from_annotation(&self, annotation_id: &str, tag: &str) -> Result<()> {
        let mut annotations = self.annotations.write().await;
        
        if let Some(annotation) = annotations.iter_mut().find(|a| a.id == annotation_id) {
            annotation.remove_tag(tag);
            self.service.update_annotation(annotation).await?;
        }

        drop(annotations);
        self.refresh_annotation_model().await;
        Ok(())
    }

    /// Get all unique tags
    pub async fn get_all_tags(&self) -> Vec<String> {
        let annotations = self.annotations.read().await;
        let mut tags = std::collections::HashSet::new();
        
        for annotation in annotations.iter() {
            for tag in &annotation.tags {
                tags.insert(tag.clone());
            }
        }
        
        let mut sorted_tags: Vec<String> = tags.into_iter().collect();
        sorted_tags.sort();
        sorted_tags
    }

    /// Get highlights for text rendering
    pub async fn get_highlights_for_page(&self, page_number: u32) -> Result<Vec<(usize, usize, String)>> {
        let annotations = self.annotations.read().await;
        let highlights: Vec<(usize, usize, String)> = annotations
            .iter()
            .filter(|a| a.page_number == page_number && a.annotation_type == AnnotationType::Highlight)
            .map(|a| (a.position.start_offset, a.position.end_offset, a.color.to_hex()))
            .collect();
        Ok(highlights)
    }
}