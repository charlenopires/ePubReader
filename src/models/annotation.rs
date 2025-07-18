use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

/// Annotation model for storing user annotations and highlights
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Annotation {
    pub id: String,
    pub book_id: String,
    pub page_number: u32,
    pub selected_text: String,
    pub note: Option<String>,
    pub color: HighlightColor,
    pub created_at: DateTime<Utc>,
    pub modified_at: DateTime<Utc>,
    pub position: TextPosition,
    pub tags: Vec<String>,
    pub category: Option<String>,
    pub annotation_type: AnnotationType,
    pub formatting: Option<TextFormatting>,
    pub is_favorite: bool,
    pub cross_references: Vec<String>, // IDs of related annotations
}

/// Bookmark model for storing page bookmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: String,
    pub book_id: String,
    pub page_number: u32,
    pub title: Option<String>,
    pub description: Option<String>,
    pub preview_text: String,
    pub created_at: DateTime<Utc>,
    pub position: TextPosition,
    pub color: BookmarkColor,
    pub is_favorite: bool,
}

/// Text position within a page/document
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextPosition {
    pub start_offset: usize,
    pub end_offset: usize,
    pub paragraph_index: usize,
    pub chapter_id: Option<String>,
    pub line_number: Option<u32>,
    pub column_number: Option<u32>,
}

/// Highlight colors for annotations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum HighlightColor {
    Yellow,
    Green,
    Blue,
    Pink,
    Orange,
    Purple,
    Red,
    Gray,
    Custom(String), // Hex color
}

/// Bookmark colors
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BookmarkColor {
    Red,
    Blue,
    Green,
    Yellow,
    Purple,
    Orange,
    Gray,
    Custom(String), // Hex color
}

/// Annotation types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnnotationType {
    Highlight,
    Note,
    Bookmark,
    Underline,
    Strikethrough,
    Question,
    Important,
    Reference,
}

/// Text formatting options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextFormatting {
    pub bold: bool,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
    pub font_size: Option<u16>,
    pub font_color: Option<String>,
    pub background_color: Option<String>,
}

/// Annotation category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationCategory {
    pub id: String,
    pub name: String,
    pub color: String,
    pub icon: Option<String>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// Annotation tag
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationTag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub usage_count: u32,
    pub created_at: DateTime<Utc>,
}

/// Annotation collection for organizing annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationCollection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub annotation_ids: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub is_shared: bool,
}

/// Export format for annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Csv,
    Markdown,
    Html,
    Pdf,
    Txt,
}

/// Annotation export options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportOptions {
    pub format: ExportFormat,
    pub include_highlights: bool,
    pub include_notes: bool,
    pub include_bookmarks: bool,
    pub include_timestamps: bool,
    pub include_page_numbers: bool,
    pub include_context: bool,
    pub group_by_chapter: bool,
    pub sort_by: AnnotationSortBy,
}

/// Sorting options for annotations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum AnnotationSortBy {
    CreatedAt,
    PageNumber,
    Type,
    Color,
    Category,
    Tag,
}

/// Annotation search filter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationFilter {
    pub book_id: Option<String>,
    pub annotation_type: Option<AnnotationType>,
    pub color: Option<HighlightColor>,
    pub category: Option<String>,
    pub tags: Vec<String>,
    pub date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub text_search: Option<String>,
    pub is_favorite: Option<bool>,
}

/// Annotation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationStats {
    pub total_annotations: u32,
    pub highlights_count: u32,
    pub notes_count: u32,
    pub bookmarks_count: u32,
    pub favorite_count: u32,
    pub categories_count: u32,
    pub tags_count: u32,
    pub color_distribution: HashMap<HighlightColor, u32>,
    pub daily_activity: HashMap<String, u32>, // Date -> count
    pub most_used_tags: Vec<(String, u32)>,
    pub reading_patterns: ReadingPatterns,
}

/// Reading patterns derived from annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingPatterns {
    pub most_annotated_books: Vec<(String, u32)>, // Book title, annotation count
    pub preferred_colors: Vec<HighlightColor>,
    pub annotation_frequency: f32, // Annotations per page
    pub average_note_length: f32,
    pub most_active_hours: Vec<u8>, // Hours of day
    pub annotation_clusters: Vec<AnnotationCluster>,
}

/// Cluster of related annotations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationCluster {
    pub id: String,
    pub annotations: Vec<String>, // Annotation IDs
    pub topic: Option<String>,
    pub relevance_score: f32,
}

impl Annotation {
    /// Create a new annotation
    pub fn new(
        book_id: String,
        page_number: u32,
        selected_text: String,
        position: TextPosition,
        annotation_type: AnnotationType,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            book_id,
            page_number,
            selected_text,
            note: None,
            color: HighlightColor::Yellow,
            created_at: Utc::now(),
            modified_at: Utc::now(),
            position,
            tags: Vec::new(),
            category: None,
            annotation_type,
            formatting: None,
            is_favorite: false,
            cross_references: Vec::new(),
        }
    }

    /// Add a note to the annotation
    pub fn add_note(&mut self, note: String) {
        self.note = Some(note);
        self.modified_at = Utc::now();
    }

    /// Add a tag to the annotation
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.modified_at = Utc::now();
        }
    }

    /// Remove a tag from the annotation
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.modified_at = Utc::now();
    }

    /// Set the highlight color
    pub fn set_color(&mut self, color: HighlightColor) {
        self.color = color;
        self.modified_at = Utc::now();
    }

    /// Set the category
    pub fn set_category(&mut self, category: Option<String>) {
        self.category = category;
        self.modified_at = Utc::now();
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
        self.modified_at = Utc::now();
    }

    /// Add cross-reference to another annotation
    pub fn add_cross_reference(&mut self, annotation_id: String) {
        if !self.cross_references.contains(&annotation_id) {
            self.cross_references.push(annotation_id);
            self.modified_at = Utc::now();
        }
    }

    /// Get preview text (truncated if too long)
    pub fn get_preview_text(&self, max_length: usize) -> String {
        if self.selected_text.len() <= max_length {
            self.selected_text.clone()
        } else {
            format!("{}...", &self.selected_text[..max_length])
        }
    }

    /// Check if annotation matches filter
    pub fn matches_filter(&self, filter: &AnnotationFilter) -> bool {
        if let Some(book_id) = &filter.book_id {
            if self.book_id != *book_id {
                return false;
            }
        }

        if let Some(annotation_type) = &filter.annotation_type {
            if self.annotation_type != *annotation_type {
                return false;
            }
        }

        if let Some(color) = &filter.color {
            if self.color != *color {
                return false;
            }
        }

        if let Some(category) = &filter.category {
            if self.category.as_ref() != Some(category) {
                return false;
            }
        }

        if !filter.tags.is_empty() {
            if !filter.tags.iter().any(|tag| self.tags.contains(tag)) {
                return false;
            }
        }

        if let Some((start, end)) = &filter.date_range {
            if self.created_at < *start || self.created_at > *end {
                return false;
            }
        }

        if let Some(text_search) = &filter.text_search {
            let search_lower = text_search.to_lowercase();
            let matches_text = self.selected_text.to_lowercase().contains(&search_lower);
            let matches_note = self.note.as_ref()
                .map(|note| note.to_lowercase().contains(&search_lower))
                .unwrap_or(false);
            
            if !matches_text && !matches_note {
                return false;
            }
        }

        if let Some(is_favorite) = filter.is_favorite {
            if self.is_favorite != is_favorite {
                return false;
            }
        }

        true
    }
}

impl Bookmark {
    /// Create a new bookmark
    pub fn new(
        book_id: String,
        page_number: u32,
        preview_text: String,
        position: TextPosition,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            book_id,
            page_number,
            title: None,
            description: None,
            preview_text,
            created_at: Utc::now(),
            position,
            color: BookmarkColor::Red,
            is_favorite: false,
        }
    }

    /// Set bookmark title
    pub fn set_title(&mut self, title: String) {
        self.title = Some(title);
    }

    /// Set bookmark description
    pub fn set_description(&mut self, description: String) {
        self.description = Some(description);
    }

    /// Set bookmark color
    pub fn set_color(&mut self, color: BookmarkColor) {
        self.color = color;
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
    }

    /// Get display title (uses title or generates from preview)
    pub fn get_display_title(&self) -> String {
        if let Some(title) = &self.title {
            title.clone()
        } else {
            // Generate title from preview text
            let words: Vec<&str> = self.preview_text.split_whitespace().take(5).collect();
            if words.is_empty() {
                format!("Bookmark - Page {}", self.page_number)
            } else {
                format!("{}...", words.join(" "))
            }
        }
    }
}

impl HighlightColor {
    /// Get hex color value
    pub fn to_hex(&self) -> String {
        match self {
            HighlightColor::Yellow => "#FFD700".to_string(),
            HighlightColor::Green => "#90EE90".to_string(),
            HighlightColor::Blue => "#87CEEB".to_string(),
            HighlightColor::Pink => "#FFB6C1".to_string(),
            HighlightColor::Orange => "#FFA500".to_string(),
            HighlightColor::Purple => "#DDA0DD".to_string(),
            HighlightColor::Red => "#FF6B6B".to_string(),
            HighlightColor::Gray => "#D3D3D3".to_string(),
            HighlightColor::Custom(hex) => hex.clone(),
        }
    }

    /// Get color name
    pub fn to_name(&self) -> String {
        match self {
            HighlightColor::Yellow => "Yellow".to_string(),
            HighlightColor::Green => "Green".to_string(),
            HighlightColor::Blue => "Blue".to_string(),
            HighlightColor::Pink => "Pink".to_string(),
            HighlightColor::Orange => "Orange".to_string(),
            HighlightColor::Purple => "Purple".to_string(),
            HighlightColor::Red => "Red".to_string(),
            HighlightColor::Gray => "Gray".to_string(),
            HighlightColor::Custom(_) => "Custom".to_string(),
        }
    }

    /// Get all available colors
    pub fn all_colors() -> Vec<HighlightColor> {
        vec![
            HighlightColor::Yellow,
            HighlightColor::Green,
            HighlightColor::Blue,
            HighlightColor::Pink,
            HighlightColor::Orange,
            HighlightColor::Purple,
            HighlightColor::Red,
            HighlightColor::Gray,
        ]
    }
}

impl BookmarkColor {
    /// Get hex color value
    pub fn to_hex(&self) -> String {
        match self {
            BookmarkColor::Red => "#FF6B6B".to_string(),
            BookmarkColor::Blue => "#4ECDC4".to_string(),
            BookmarkColor::Green => "#45B7D1".to_string(),
            BookmarkColor::Yellow => "#F9CA24".to_string(),
            BookmarkColor::Purple => "#6C5CE7".to_string(),
            BookmarkColor::Orange => "#FD79A8".to_string(),
            BookmarkColor::Gray => "#636E72".to_string(),
            BookmarkColor::Custom(hex) => hex.clone(),
        }
    }

    /// Get color name
    pub fn to_name(&self) -> String {
        match self {
            BookmarkColor::Red => "Red".to_string(),
            BookmarkColor::Blue => "Blue".to_string(),
            BookmarkColor::Green => "Green".to_string(),
            BookmarkColor::Yellow => "Yellow".to_string(),
            BookmarkColor::Purple => "Purple".to_string(),
            BookmarkColor::Orange => "Orange".to_string(),
            BookmarkColor::Gray => "Gray".to_string(),
            BookmarkColor::Custom(_) => "Custom".to_string(),
        }
    }
}

impl AnnotationType {
    /// Get display name
    pub fn to_display_name(&self) -> String {
        match self {
            AnnotationType::Highlight => "Highlight".to_string(),
            AnnotationType::Note => "Note".to_string(),
            AnnotationType::Bookmark => "Bookmark".to_string(),
            AnnotationType::Underline => "Underline".to_string(),
            AnnotationType::Strikethrough => "Strikethrough".to_string(),
            AnnotationType::Question => "Question".to_string(),
            AnnotationType::Important => "Important".to_string(),
            AnnotationType::Reference => "Reference".to_string(),
        }
    }

    /// Get icon for annotation type
    pub fn to_icon(&self) -> String {
        match self {
            AnnotationType::Highlight => "🖍️".to_string(),
            AnnotationType::Note => "📝".to_string(),
            AnnotationType::Bookmark => "🔖".to_string(),
            AnnotationType::Underline => "📏".to_string(),
            AnnotationType::Strikethrough => "❌".to_string(),
            AnnotationType::Question => "❓".to_string(),
            AnnotationType::Important => "⭐".to_string(),
            AnnotationType::Reference => "🔗".to_string(),
        }
    }
}

impl Default for TextFormatting {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            underline: false,
            strikethrough: false,
            font_size: None,
            font_color: None,
            background_color: None,
        }
    }
}

impl Default for ExportOptions {
    fn default() -> Self {
        Self {
            format: ExportFormat::Json,
            include_highlights: true,
            include_notes: true,
            include_bookmarks: true,
            include_timestamps: true,
            include_page_numbers: true,
            include_context: true,
            group_by_chapter: false,
            sort_by: AnnotationSortBy::CreatedAt,
        }
    }
}

impl Default for AnnotationFilter {
    fn default() -> Self {
        Self {
            book_id: None,
            annotation_type: None,
            color: None,
            category: None,
            tags: Vec::new(),
            date_range: None,
            text_search: None,
            is_favorite: None,
        }
    }
}