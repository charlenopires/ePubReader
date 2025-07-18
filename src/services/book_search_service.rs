use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use regex::Regex;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};

use crate::models::book::Book;

/// Search result within a book
#[derive(Debug, Clone)]
pub struct SearchResult {
    pub page_number: u32,
    pub position: usize,
    pub match_text: String,
    pub before_text: String,
    pub after_text: String,
    pub context: String,
    pub chapter_title: String,
    pub chapter_id: Option<String>,
    pub relevance_score: f32,
}

/// Search options
#[derive(Debug, Clone)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub whole_words: bool,
    pub regex_mode: bool,
    pub context_length: usize,
    pub max_results: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_words: false,
            regex_mode: false,
            context_length: 50,
            max_results: 100,
        }
    }
}

/// Search statistics
#[derive(Debug, Clone)]
pub struct SearchStats {
    pub total_results: usize,
    pub pages_searched: usize,
    pub search_duration_ms: u64,
    pub query: String,
    pub options: SearchOptions,
}

/// Book content representation for search
#[derive(Debug, Clone)]
pub struct BookContent {
    pub book_id: String,
    pub chapters: Vec<Chapter>,
    pub total_pages: u32,
    pub last_indexed: DateTime<Utc>,
}

/// Chapter content
#[derive(Debug, Clone)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub content: String,
    pub page_start: u32,
    pub page_end: u32,
    pub word_count: usize,
}

/// Search index for efficient searching
#[derive(Debug, Clone)]
pub struct SearchIndex {
    pub book_id: String,
    pub word_positions: HashMap<String, Vec<WordPosition>>,
    pub chapter_titles: HashMap<String, String>,
    pub page_content: HashMap<u32, String>,
    pub created_at: DateTime<Utc>,
}

/// Word position in the book
#[derive(Debug, Clone)]
pub struct WordPosition {
    pub chapter_id: String,
    pub page_number: u32,
    pub position: usize,
    pub word: String,
}

/// Book search service
pub struct BookSearchService {
    content_cache: Arc<RwLock<HashMap<String, BookContent>>>,
    search_indices: Arc<RwLock<HashMap<String, SearchIndex>>>,
    search_history: Arc<RwLock<Vec<SearchStats>>>,
}

impl BookSearchService {
    pub fn new() -> Self {
        Self {
            content_cache: Arc::new(RwLock::new(HashMap::new())),
            search_indices: Arc::new(RwLock::new(HashMap::new())),
            search_history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Search within a book
    pub async fn search_in_book(
        &self,
        book_id: &str,
        query: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let start_time = std::time::Instant::now();
        
        // Get book content
        let book_content = self.get_book_content(book_id).await?;
        
        // Perform search
        let results = if options.regex_mode {
            self.regex_search(&book_content, query, options).await?
        } else {
            self.text_search(&book_content, query, options).await?
        };
        
        // Calculate search duration
        let duration = start_time.elapsed();
        
        // Store search statistics
        let stats = SearchStats {
            total_results: results.len(),
            pages_searched: book_content.total_pages as usize,
            search_duration_ms: duration.as_millis() as u64,
            query: query.to_string(),
            options: options.clone(),
        };
        
        self.add_search_stats(stats).await;
        
        Ok(results)
    }

    /// Text-based search
    async fn text_search(
        &self,
        book_content: &BookContent,
        query: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        let search_query = if options.case_sensitive {
            query.to_string()
        } else {
            query.to_lowercase()
        };

        for chapter in &book_content.chapters {
            let chapter_content = if options.case_sensitive {
                chapter.content.clone()
            } else {
                chapter.content.to_lowercase()
            };

            let matches = if options.whole_words {
                self.find_whole_word_matches(&chapter_content, &search_query)
            } else {
                self.find_substring_matches(&chapter_content, &search_query)
            };

            for match_pos in matches {
                if results.len() >= options.max_results {
                    break;
                }

                let result = self.create_search_result(
                    chapter,
                    match_pos,
                    query,
                    options.context_length,
                )?;

                results.push(result);
            }

            if results.len() >= options.max_results {
                break;
            }
        }

        // Sort results by relevance
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        Ok(results)
    }

    /// Regex-based search
    async fn regex_search(
        &self,
        book_content: &BookContent,
        pattern: &str,
        options: &SearchOptions,
    ) -> Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        
        let regex_flags = if options.case_sensitive {
            ""
        } else {
            "(?i)"
        };
        
        let full_pattern = format!("{}{}", regex_flags, pattern);
        let regex = Regex::new(&full_pattern)?;

        for chapter in &book_content.chapters {
            let matches: Vec<_> = regex.find_iter(&chapter.content).collect();

            for regex_match in matches {
                if results.len() >= options.max_results {
                    break;
                }

                let result = self.create_search_result(
                    chapter,
                    regex_match.start(),
                    regex_match.as_str(),
                    options.context_length,
                )?;

                results.push(result);
            }

            if results.len() >= options.max_results {
                break;
            }
        }

        // Sort results by relevance
        results.sort_by(|a, b| b.relevance_score.partial_cmp(&a.relevance_score).unwrap());

        Ok(results)
    }

    /// Find whole word matches
    fn find_whole_word_matches(&self, text: &str, query: &str) -> Vec<usize> {
        let mut positions = Vec::new();
        let words: Vec<&str> = text.split_whitespace().collect();
        let mut current_pos = 0;

        for word in words {
            if word == query {
                positions.push(current_pos);
            }
            current_pos += word.len() + 1; // +1 for space
        }

        positions
    }

    /// Find substring matches
    fn find_substring_matches(&self, text: &str, query: &str) -> Vec<usize> {
        let mut positions = Vec::new();
        let mut start = 0;

        while let Some(pos) = text[start..].find(query) {
            let absolute_pos = start + pos;
            positions.push(absolute_pos);
            start = absolute_pos + 1;
        }

        positions
    }

    /// Create search result from match
    fn create_search_result(
        &self,
        chapter: &Chapter,
        position: usize,
        match_text: &str,
        context_length: usize,
    ) -> Result<SearchResult> {
        let content = &chapter.content;
        let match_len = match_text.len();

        // Calculate context boundaries
        let before_start = if position >= context_length {
            position - context_length
        } else {
            0
        };
        
        let after_end = if position + match_len + context_length < content.len() {
            position + match_len + context_length
        } else {
            content.len()
        };

        // Extract context text
        let before_text = content[before_start..position].trim().to_string();
        let after_text = content[position + match_len..after_end].trim().to_string();
        let full_context = content[before_start..after_end].trim().to_string();

        // Calculate page number (simplified - would need proper page calculation)
        let page_number = self.calculate_page_number(chapter, position);

        // Calculate relevance score
        let relevance_score = self.calculate_relevance_score(match_text, &full_context);

        Ok(SearchResult {
            page_number,
            position,
            match_text: match_text.to_string(),
            before_text,
            after_text,
            context: full_context,
            chapter_title: chapter.title.clone(),
            chapter_id: Some(chapter.id.clone()),
            relevance_score,
        })
    }

    /// Calculate page number from chapter and position
    fn calculate_page_number(&self, chapter: &Chapter, position: usize) -> u32 {
        // Simplified calculation - in a real implementation, this would use proper page layout
        let chars_per_page = 2000; // Approximate characters per page
        let page_offset = position / chars_per_page;
        chapter.page_start + page_offset as u32
    }

    /// Calculate relevance score
    fn calculate_relevance_score(&self, match_text: &str, context: &str) -> f32 {
        let mut score = 1.0;

        // Boost score for exact matches
        if match_text.len() > 3 {
            score += 0.5;
        }

        // Boost score for matches at word boundaries
        if context.contains(&format!(" {} ", match_text)) {
            score += 0.3;
        }

        // Boost score for matches at the beginning of sentences
        if context.contains(&format!(". {}", match_text)) {
            score += 0.2;
        }

        score
    }

    /// Get book content (with caching)
    async fn get_book_content(&self, book_id: &str) -> Result<BookContent> {
        let cache = self.content_cache.read().await;
        
        if let Some(content) = cache.get(book_id) {
            return Ok(content.clone());
        }

        drop(cache);

        // Load content from book (this would integrate with your book loading system)
        let content = self.load_book_content(book_id).await?;
        
        // Cache the content
        let mut cache = self.content_cache.write().await;
        cache.insert(book_id.to_string(), content.clone());

        Ok(content)
    }

    /// Load book content from storage
    async fn load_book_content(&self, book_id: &str) -> Result<BookContent> {
        // This would integrate with your existing book loading system
        // For now, return a placeholder
        Ok(BookContent {
            book_id: book_id.to_string(),
            chapters: vec![
                Chapter {
                    id: "chapter1".to_string(),
                    title: "Chapter 1".to_string(),
                    content: "Sample chapter content for searching...".to_string(),
                    page_start: 1,
                    page_end: 10,
                    word_count: 1000,
                }
            ],
            total_pages: 100,
            last_indexed: Utc::now(),
        })
    }

    /// Build search index for a book
    pub async fn build_search_index(&self, book_id: &str) -> Result<()> {
        let book_content = self.get_book_content(book_id).await?;
        let mut word_positions = HashMap::new();
        let mut chapter_titles = HashMap::new();
        let mut page_content = HashMap::new();

        for chapter in &book_content.chapters {
            chapter_titles.insert(chapter.id.clone(), chapter.title.clone());

            // Extract words and their positions
            let words: Vec<&str> = chapter.content.split_whitespace().collect();
            let mut current_pos = 0;

            for word in words {
                let clean_word = word.to_lowercase()
                    .chars()
                    .filter(|c| c.is_alphanumeric())
                    .collect::<String>();

                if !clean_word.is_empty() {
                    let position = WordPosition {
                        chapter_id: chapter.id.clone(),
                        page_number: self.calculate_page_number(chapter, current_pos),
                        position: current_pos,
                        word: clean_word.clone(),
                    };

                    word_positions.entry(clean_word)
                        .or_insert_with(Vec::new)
                        .push(position);
                }

                current_pos += word.len() + 1;
            }

            // Index page content
            for page in chapter.page_start..=chapter.page_end {
                page_content.insert(page, chapter.content.clone());
            }
        }

        let index = SearchIndex {
            book_id: book_id.to_string(),
            word_positions,
            chapter_titles,
            page_content,
            created_at: Utc::now(),
        };

        let mut indices = self.search_indices.write().await;
        indices.insert(book_id.to_string(), index);

        Ok(())
    }

    /// Get search suggestions
    pub async fn get_search_suggestions(&self, book_id: &str, partial_query: &str) -> Result<Vec<String>> {
        let indices = self.search_indices.read().await;
        
        if let Some(index) = indices.get(book_id) {
            let mut suggestions = Vec::new();
            let query_lower = partial_query.to_lowercase();

            for word in index.word_positions.keys() {
                if word.starts_with(&query_lower) && word.len() > partial_query.len() {
                    suggestions.push(word.clone());
                }
            }

            suggestions.sort();
            suggestions.truncate(10); // Limit suggestions
            Ok(suggestions)
        } else {
            Ok(Vec::new())
        }
    }

    /// Get search history
    pub async fn get_search_history(&self) -> Vec<SearchStats> {
        let history = self.search_history.read().await;
        history.clone()
    }

    /// Add search statistics
    async fn add_search_stats(&self, stats: SearchStats) {
        let mut history = self.search_history.write().await;
        history.push(stats);
        
        // Keep only last 100 searches
        if history.len() > 100 {
            history.remove(0);
        }
    }

    /// Clear search cache
    pub async fn clear_cache(&self) {
        let mut cache = self.content_cache.write().await;
        cache.clear();
        
        let mut indices = self.search_indices.write().await;
        indices.clear();
    }

    /// Get cached book content
    pub async fn get_cached_books(&self) -> Vec<String> {
        let cache = self.content_cache.read().await;
        cache.keys().cloned().collect()
    }

    /// Remove book from cache
    pub async fn remove_from_cache(&self, book_id: &str) {
        let mut cache = self.content_cache.write().await;
        cache.remove(book_id);
        
        let mut indices = self.search_indices.write().await;
        indices.remove(book_id);
    }

    /// Get word frequency for a book
    pub async fn get_word_frequency(&self, book_id: &str) -> Result<HashMap<String, usize>> {
        let indices = self.search_indices.read().await;
        
        if let Some(index) = indices.get(book_id) {
            let mut frequency = HashMap::new();
            
            for (word, positions) in &index.word_positions {
                frequency.insert(word.clone(), positions.len());
            }
            
            Ok(frequency)
        } else {
            Ok(HashMap::new())
        }
    }

    /// Search across multiple books
    pub async fn search_across_books(
        &self,
        book_ids: &[String],
        query: &str,
        options: &SearchOptions,
    ) -> Result<HashMap<String, Vec<SearchResult>>> {
        let mut results = HashMap::new();
        
        for book_id in book_ids {
            let book_results = self.search_in_book(book_id, query, options).await?;
            results.insert(book_id.clone(), book_results);
        }
        
        Ok(results)
    }

    /// Get search result context with highlighting
    pub async fn get_highlighted_context(
        &self,
        book_id: &str,
        page_number: u32,
        position: usize,
        query: &str,
        context_length: usize,
    ) -> Result<String> {
        let book_content = self.get_book_content(book_id).await?;
        
        // Find the chapter for this page
        for chapter in &book_content.chapters {
            if page_number >= chapter.page_start && page_number <= chapter.page_end {
                let content = &chapter.content;
                
                // Calculate context boundaries
                let start = if position >= context_length {
                    position - context_length
                } else {
                    0
                };
                
                let end = if position + query.len() + context_length < content.len() {
                    position + query.len() + context_length
                } else {
                    content.len()
                };
                
                let context = &content[start..end];
                
                // Add HTML highlighting
                let highlighted = context.replace(
                    query,
                    &format!("<mark>{}</mark>", query)
                );
                
                return Ok(highlighted);
            }
        }
        
        Ok(String::new())
    }
}

impl Default for BookSearchService {
    fn default() -> Self {
        Self::new()
    }
}