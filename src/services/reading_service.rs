use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use anyhow::Result;
use regex::Regex;

use crate::models::{Book, ThemeManager};
use crate::models::reading_theme::{ReadingTheme, ReadingThemePreferences};

/// Reading service for managing book content and reading experience
pub struct ReadingService {
    theme_manager: Arc<RwLock<ThemeManager>>,
    content_cache: Arc<RwLock<HashMap<String, BookContent>>>,
    pagination_cache: Arc<RwLock<HashMap<String, Vec<Page>>>>,
}

/// Book content structure
#[derive(Debug, Clone)]
pub struct BookContent {
    pub book_id: String,
    pub title: String,
    pub author: String,
    pub chapters: Vec<Chapter>,
    pub total_word_count: usize,
    pub estimated_reading_time: u32, // in minutes
}

/// Chapter structure
#[derive(Debug, Clone)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub content: String,
    pub word_count: usize,
    pub order: usize,
}

/// Page structure for pagination
#[derive(Debug, Clone)]
pub struct Page {
    pub page_number: usize,
    pub content: String,
    pub word_count: usize,
    pub chapter_id: String,
    pub start_position: usize,
    pub end_position: usize,
}

/// Pagination settings
#[derive(Debug, Clone)]
pub struct PaginationSettings {
    pub viewport_width: u32,
    pub viewport_height: u32,
    pub font_size: u16,
    pub line_height: f32,
    pub margin_horizontal: u16,
    pub margin_vertical: u16,
    pub font_family: String,
    pub two_column_mode: bool,
}

/// Page transition animation
#[derive(Debug, Clone)]
pub struct PageTransition {
    pub transition_type: TransitionType,
    pub duration: u32, // in milliseconds
    pub easing: EasingType,
}

/// Transition types
#[derive(Debug, Clone, PartialEq)]
pub enum TransitionType {
    Fade,
    Slide,
    Flip,
    Curl,
    None,
}

/// Easing types
#[derive(Debug, Clone, PartialEq)]
pub enum EasingType {
    Linear,
    EaseIn,
    EaseOut,
    EaseInOut,
}

/// Text selection and highlighting
#[derive(Debug, Clone)]
pub struct TextSelection {
    pub start_position: usize,
    pub end_position: usize,
    pub selected_text: String,
    pub chapter_id: String,
    pub page_number: usize,
}

/// Highlight annotation
#[derive(Debug, Clone)]
pub struct Highlight {
    pub id: String,
    pub selection: TextSelection,
    pub color: String,
    pub note: Option<String>,
    pub created_at: chrono::DateTime<chrono::Utc>,
}

impl ReadingService {
    /// Create a new reading service
    pub fn new() -> Self {
        Self {
            theme_manager: Arc::new(RwLock::new(ThemeManager::new())),
            content_cache: Arc::new(RwLock::new(HashMap::new())),
            pagination_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Load book content for reading
    pub async fn load_book_content(&self, book: &Book) -> Result<BookContent> {
        // Check cache first
        {
            let cache = self.content_cache.read().await;
            if let Some(content) = cache.get(&book.id) {
                return Ok(content.clone());
            }
        }

        // Parse book content based on format
        let content = match book.file_format {
            crate::models::BookFormat::Epub => {
                self.parse_epub_content(book).await?
            }
            crate::models::BookFormat::Pdf => {
                self.parse_pdf_content(book).await?
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported book format"));
            }
        };

        // Cache the content
        {
            let mut cache = self.content_cache.write().await;
            cache.insert(book.id.clone(), content.clone());
        }

        Ok(content)
    }

    /// Parse EPUB content
    async fn parse_epub_content(&self, book: &Book) -> Result<BookContent> {
        use epub::doc::EpubDoc;
        
        let mut doc = EpubDoc::new(&book.file_path)?;
        let mut chapters = Vec::new();
        let mut total_word_count = 0;

        // Get spine (reading order)
        let spine = doc.spine.clone();
        
        for (order, spine_item) in spine.iter().enumerate() {
            if let Some(id) = &spine_item.id {
                if let Some((content, _)) = doc.get_resource_str(id) {
                    let cleaned_content = self.clean_html_content(&content);
                    let word_count = self.count_words(&cleaned_content);
                    
                    let chapter = Chapter {
                        id: id.clone(),
                    title: format!("Chapter {}", order + 1),
                    content: cleaned_content,
                    word_count,
                    order,
                };
                
                    total_word_count += word_count;
                    chapters.push(chapter);
                }
            }
        }

        // Estimate reading time (average 200 words per minute)
        let estimated_reading_time = (total_word_count as f32 / 200.0).ceil() as u32;

        Ok(BookContent {
            book_id: book.id.clone(),
            title: book.title.clone(),
            author: book.author.clone(),
            chapters,
            total_word_count,
            estimated_reading_time,
        })
    }

    /// Parse PDF content
    async fn parse_pdf_content(&self, book: &Book) -> Result<BookContent> {
        // For now, return a placeholder
        // In a real implementation, you would use pdf-extract or similar
        let content = format!("PDF content from {}", book.title);
        let word_count = self.count_words(&content);

        let chapter = Chapter {
            id: "pdf_content".to_string(),
            title: "PDF Content".to_string(),
            content,
            word_count,
            order: 0,
        };

        Ok(BookContent {
            book_id: book.id.clone(),
            title: book.title.clone(),
            author: book.author.clone(),
            chapters: vec![chapter],
            total_word_count: word_count,
            estimated_reading_time: (word_count as f32 / 200.0).ceil() as u32,
        })
    }

    /// Clean HTML content for reading
    fn clean_html_content(&self, html: &str) -> String {
        // Remove HTML tags but preserve structure
        let re = Regex::new(r"<[^>]+>").unwrap();
        let cleaned = re.replace_all(html, " ");
        
        // Clean up whitespace
        let re = Regex::new(r"\s+").unwrap();
        let cleaned = re.replace_all(&cleaned, " ");
        
        // Decode HTML entities
        let cleaned = html_escape::decode_html_entities(&cleaned);
        
        cleaned.trim().to_string()
    }

    /// Count words in text
    fn count_words(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }

    /// Generate pagination for book content
    pub async fn paginate_content(
        &self,
        content: &BookContent,
        settings: &PaginationSettings,
    ) -> Result<Vec<Page>> {
        let cache_key = format!("{}_{}", content.book_id, self.pagination_cache_key(settings));
        
        // Check cache first
        {
            let cache = self.pagination_cache.read().await;
            if let Some(pages) = cache.get(&cache_key) {
                return Ok(pages.clone());
            }
        }

        let pages = self.calculate_pagination(content, settings).await?;

        // Cache the pagination
        {
            let mut cache = self.pagination_cache.write().await;
            cache.insert(cache_key, pages.clone());
        }

        Ok(pages)
    }

    /// Calculate pagination based on viewport and font settings
    async fn calculate_pagination(
        &self,
        content: &BookContent,
        settings: &PaginationSettings,
    ) -> Result<Vec<Page>> {
        let mut pages = Vec::new();
        let mut page_number = 1;

        // Calculate available text area
        let available_width = settings.viewport_width - (settings.margin_horizontal * 2) as u32;
        let available_height = settings.viewport_height - (settings.margin_vertical * 2) as u32;

        // Estimate characters per line and lines per page
        let char_width = (settings.font_size as f32 * 0.6) as u32; // Rough estimate
        let line_height = (settings.font_size as f32 * settings.line_height) as u32;
        
        let chars_per_line = available_width / char_width;
        let lines_per_page = available_height / line_height;
        let chars_per_page = chars_per_line * lines_per_page;

        // Handle two-column mode
        let effective_chars_per_page = if settings.two_column_mode {
            chars_per_page * 2
        } else {
            chars_per_page
        };

        for chapter in &content.chapters {
            let chapter_content = &chapter.content;
            let mut start_position = 0;

            while start_position < chapter_content.len() {
                let end_position = std::cmp::min(
                    start_position + effective_chars_per_page as usize,
                    chapter_content.len(),
                );

                // Find a good break point (end of sentence or word)
                let actual_end = self.find_break_point(chapter_content, start_position, end_position);
                
                let page_content = chapter_content[start_position..actual_end].to_string();
                let word_count = self.count_words(&page_content);

                pages.push(Page {
                    page_number,
                    content: page_content,
                    word_count,
                    chapter_id: chapter.id.clone(),
                    start_position,
                    end_position: actual_end,
                });

                page_number += 1;
                start_position = actual_end;
            }
        }

        Ok(pages)
    }

    /// Find a good break point for pagination
    fn find_break_point(&self, text: &str, start: usize, max_end: usize) -> usize {
        if max_end >= text.len() {
            return text.len();
        }

        // Try to find end of sentence
        for i in (start..max_end).rev() {
            if let Some(ch) = text.chars().nth(i) {
                if ch == '.' || ch == '!' || ch == '?' {
                    return i + 1;
                }
            }
        }

        // Try to find end of word
        for i in (start..max_end).rev() {
            if let Some(ch) = text.chars().nth(i) {
                if ch.is_whitespace() {
                    return i;
                }
            }
        }

        // Return max_end if no good break point found
        max_end
    }

    /// Generate cache key for pagination
    fn pagination_cache_key(&self, settings: &PaginationSettings) -> String {
        format!(
            "{}x{}_{}_{}_{}_{}_{}_{}",
            settings.viewport_width,
            settings.viewport_height,
            settings.font_size,
            settings.line_height,
            settings.margin_horizontal,
            settings.margin_vertical,
            settings.font_family,
            settings.two_column_mode
        )
    }

    /// Get theme manager
    pub async fn get_theme_manager(&self) -> Arc<RwLock<ThemeManager>> {
        self.theme_manager.clone()
    }

    /// Apply theme to content
    pub async fn apply_theme(&self, theme_name: &str) -> Result<()> {
        let mut theme_manager = self.theme_manager.write().await;
        theme_manager.switch_theme(theme_name);
        Ok(())
    }

    /// Get current reading preferences
    pub async fn get_reading_preferences(&self) -> ReadingThemePreferences {
        let theme_manager = self.theme_manager.read().await;
        theme_manager.get_preferences().clone()
    }

    /// Update reading preferences
    pub async fn update_reading_preferences(&self, preferences: ReadingThemePreferences) -> Result<()> {
        let mut theme_manager = self.theme_manager.write().await;
        theme_manager.update_preferences(preferences);
        Ok(())
    }

    /// Search text in book content
    pub async fn search_in_content(
        &self,
        content: &BookContent,
        query: &str,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let query_lower = query.to_lowercase();

        for chapter in &content.chapters {
            let chapter_content_lower = chapter.content.to_lowercase();
            let mut start = 0;

            while let Some(pos) = chapter_content_lower[start..].find(&query_lower) {
                let absolute_pos = start + pos;
                let context_start = absolute_pos.saturating_sub(50);
                let context_end = std::cmp::min(absolute_pos + query.len() + 50, chapter.content.len());
                
                let context = &chapter.content[context_start..context_end];
                
                results.push(SearchResult {
                    chapter_id: chapter.id.clone(),
                    chapter_title: chapter.title.clone(),
                    position: absolute_pos,
                    context: context.to_string(),
                    matched_text: query.to_string(),
                });

                start = absolute_pos + 1;
            }
        }

        Ok(results)
    }

    /// Create highlight annotation
    pub async fn create_highlight(
        &self,
        selection: TextSelection,
        color: String,
        note: Option<String>,
    ) -> Result<Highlight> {
        let highlight = Highlight {
            id: uuid::Uuid::new_v4().to_string(),
            selection,
            color,
            note,
            created_at: chrono::Utc::now(),
        };

        // In a real implementation, you would save this to database
        // For now, just return the highlight
        Ok(highlight)
    }

    /// Calculate reading statistics
    pub async fn calculate_reading_stats(
        &self,
        content: &BookContent,
        current_position: usize,
        reading_speed_wpm: u32,
    ) -> ReadingStats {
        let total_words = content.total_word_count;
        let words_read = (current_position as f32 / content.chapters.iter().map(|c| c.content.len()).sum::<usize>() as f32 * total_words as f32) as usize;
        let progress = words_read as f32 / total_words as f32;
        
        let remaining_words = total_words - words_read;
        let estimated_time_remaining = (remaining_words as f32 / reading_speed_wpm as f32).ceil() as u32;

        ReadingStats {
            total_words,
            words_read,
            progress,
            estimated_time_remaining,
            estimated_total_time: content.estimated_reading_time,
        }
    }

    /// Clear caches
    pub async fn clear_caches(&self) {
        let mut content_cache = self.content_cache.write().await;
        content_cache.clear();

        let mut pagination_cache = self.pagination_cache.write().await;
        pagination_cache.clear();
    }
}

/// Search result structure
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub chapter_id: String,
    pub chapter_title: String,
    pub position: usize,
    pub context: String,
    pub matched_text: String,
}

/// Reading statistics
#[derive(Debug, Clone)]
pub struct ReadingStats {
    pub total_words: usize,
    pub words_read: usize,
    pub progress: f32,
    pub estimated_time_remaining: u32, // in minutes
    pub estimated_total_time: u32,     // in minutes
}

impl Default for PaginationSettings {
    fn default() -> Self {
        Self {
            viewport_width: 800,
            viewport_height: 600,
            font_size: 16,
            line_height: 1.5,
            margin_horizontal: 40,
            margin_vertical: 40,
            font_family: "Default".to_string(),
            two_column_mode: false,
        }
    }
}

impl Default for PageTransition {
    fn default() -> Self {
        Self {
            transition_type: TransitionType::Fade,
            duration: 300,
            easing: EasingType::EaseOut,
        }
    }
}

impl Default for ReadingService {
    fn default() -> Self {
        Self::new()
    }
}