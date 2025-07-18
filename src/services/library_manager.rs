use std::sync::Arc;
use std::collections::HashMap;
use anyhow::Result;
use slint::{ComponentHandle, Model, VecModel, ModelRc};
use tokio::sync::RwLock;

use crate::models::library::{
    Collection, SmartCollectionRules, SmartRule, SmartRuleField, SmartRuleOperator, MatchType,
    Category, ReadingStatus, LibraryStats, LibraryFilter, LibrarySortBy, SortDirection,
    Author, Genre, Tag, LibraryOrganizer,
};
use crate::services::library_service::LibraryService;

/// Slint-compatible library statistics
#[derive(Clone, Default)]
pub struct SlintLibraryStats {
    pub total_books: i32,
    pub want_to_read: i32,
    pub currently_reading: i32,
    pub finished: i32,
    pub total_collections: i32,
    pub total_authors: i32,
    pub total_tags: i32,
}

/// Slint-compatible collection model
#[derive(Clone)]
pub struct SlintCollection {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub count: i32,
    pub is_smart: bool,
    pub is_favorite: bool,
}

/// Slint-compatible author model
#[derive(Clone)]
pub struct SlintAuthor {
    pub id: String,
    pub name: String,
    pub count: i32,
    pub is_favorite: bool,
}

/// Slint-compatible tag model
#[derive(Clone)]
pub struct SlintTag {
    pub id: String,
    pub name: String,
    pub color: String,
    pub count: i32,
}

/// Slint-compatible smart rule model
#[derive(Clone)]
pub struct SlintSmartRule {
    pub field: String,
    pub operator: String,
    pub value: String,
}

/// Slint-compatible collection editor data
#[derive(Clone)]
pub struct SlintCollectionEditor {
    pub id: String,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub color: String,
    pub is_smart: bool,
    pub smart_rules: Vec<SlintSmartRule>,
    pub match_type: String,
}

/// Library manager for handling UI interactions
pub struct LibraryManager {
    service: LibraryService,
    stats: Arc<RwLock<SlintLibraryStats>>,
    collections: Arc<RwLock<Vec<SlintCollection>>>,
    authors: Arc<RwLock<Vec<SlintAuthor>>>,
    tags: Arc<RwLock<Vec<SlintTag>>>,
    current_category: Arc<RwLock<String>>,
    current_filter: Arc<RwLock<Option<LibraryFilter>>>,
    
    // Slint models
    collections_model: ModelRc<SlintCollection>,
    authors_model: ModelRc<SlintAuthor>,
    tags_model: ModelRc<SlintTag>,
}

impl LibraryManager {
    pub fn new(service: LibraryService) -> Self {
        Self {
            service,
            stats: Arc::new(RwLock::new(SlintLibraryStats::default())),
            collections: Arc::new(RwLock::new(Vec::new())),
            authors: Arc::new(RwLock::new(Vec::new())),
            tags: Arc::new(RwLock::new(Vec::new())),
            current_category: Arc::new(RwLock::new("all".to_string())),
            current_filter: Arc::new(RwLock::new(None)),
            collections_model: ModelRc::new(VecModel::default()),
            authors_model: ModelRc::new(VecModel::default()),
            tags_model: ModelRc::new(VecModel::default()),
        }
    }

    /// Get collections model for Slint UI
    pub fn get_collections_model(&self) -> ModelRc<SlintCollection> {
        self.collections_model.clone()
    }

    /// Get authors model for Slint UI
    pub fn get_authors_model(&self) -> ModelRc<SlintAuthor> {
        self.authors_model.clone()
    }

    /// Get tags model for Slint UI
    pub fn get_tags_model(&self) -> ModelRc<SlintTag> {
        self.tags_model.clone()
    }

    /// Load all library data
    pub async fn load_library_data(&self) -> Result<()> {
        // Load library statistics
        let stats = self.service.get_library_stats().await?;
        self.update_stats(&stats).await;

        // Load collections
        let collections = self.service.get_all_collections().await?;
        self.update_collections(&collections).await;

        // Load authors
        let authors = self.service.get_all_authors().await?;
        self.update_authors(&authors).await;

        // Load tags
        let tags = self.service.get_all_tags().await?;
        self.update_tags(&tags).await;

        Ok(())
    }

    /// Get current library statistics
    pub async fn get_stats(&self) -> SlintLibraryStats {
        self.stats.read().await.clone()
    }

    /// Update library statistics
    async fn update_stats(&self, stats: &LibraryStats) {
        let slint_stats = SlintLibraryStats {
            total_books: stats.total_books as i32,
            want_to_read: stats.want_to_read as i32,
            currently_reading: stats.currently_reading as i32,
            finished: stats.finished as i32,
            total_collections: stats.total_collections as i32,
            total_authors: stats.total_authors as i32,
            total_tags: stats.total_tags as i32,
        };

        *self.stats.write().await = slint_stats;
    }

    /// Update collections
    async fn update_collections(&self, collections: &Vec<Collection>) {
        let slint_collections: Vec<SlintCollection> = collections
            .iter()
            .map(|c| SlintCollection {
                id: c.id.clone(),
                name: c.name.clone(),
                description: c.description.clone().unwrap_or_default(),
                icon: c.icon.clone(),
                color: c.color.clone(),
                count: c.book_count() as i32,
                is_smart: c.is_smart,
                is_favorite: c.is_favorite,
            })
            .collect();

        *self.collections.write().await = slint_collections.clone();
        
        // Atualizar o modelo diretamente não é possível com ModelRc
        // Vamos usar uma abordagem diferente
    }

    /// Update authors
    async fn update_authors(&self, authors: &[Author]) {
        let slint_authors: Vec<SlintAuthor> = authors
            .iter()
            .map(|a| SlintAuthor {
                id: a.id.clone(),
                name: a.name.clone(),
                count: a.book_count as i32,
                is_favorite: false, // TODO: Implement favorite authors
            })
            .collect();

        *self.authors.write().await = slint_authors.clone();
        
        // Atualizar o modelo diretamente não é possível com ModelRc
        // Vamos usar uma abordagem diferente
    }

    /// Update tags
    async fn update_tags(&self, tags: &[Tag]) {
        let slint_tags: Vec<SlintTag> = tags
            .iter()
            .map(|t| SlintTag {
                id: t.id.clone(),
                name: t.name.clone(),
                color: t.color.clone().unwrap_or_else(|| "#007AFF".to_string()),
                count: t.book_count as i32,
            })
            .collect();

        *self.tags.write().await = slint_tags.clone();
        
        // Atualizar o modelo diretamente não é possível com ModelRc
        // Vamos usar uma abordagem diferente
    }

    /// Create a new collection
    pub async fn create_collection(&self, name: String, icon: String, color: String) -> Result<String> {
        let collection = self.service.create_collection(name, icon, color).await?;
        
        // Refresh collections
        let collections = self.service.get_all_collections().await?;
        self.update_collections(&collections).await;
        
        // Refresh stats
        let stats = self.service.get_library_stats().await?;
        self.update_stats(&stats).await;
        
        Ok(collection.id)
    }

    /// Create a smart collection
    pub async fn create_smart_collection(
        &self,
        name: String,
        rules: Vec<SlintSmartRule>,
        match_type: String,
    ) -> Result<String> {
        let smart_rules = SmartCollectionRules {
            rules: rules.into_iter().map(|r| SmartRule {
                field: self.parse_rule_field(&r.field),
                operator: self.parse_rule_operator(&r.operator),
                value: r.value,
            }).collect(),
            match_type: if match_type == "any" { MatchType::Any } else { MatchType::All },
        };

        let collection = self.service.create_smart_collection(name, smart_rules).await?;
        
        // Refresh collections
        let collections = self.service.get_all_collections().await?;
        self.update_collections(&collections).await;
        
        // Refresh stats
        let stats = self.service.get_library_stats().await?;
        self.update_stats(&stats).await;
        
        Ok(collection.id)
    }

    /// Update a collection
    pub async fn update_collection(&self, editor_data: SlintCollectionEditor) -> Result<()> {
        if let Some(mut collection) = self.service.get_collection(&editor_data.id).await? {
            collection.update_info(
                Some(editor_data.name),
                Some(editor_data.description),
                Some(editor_data.icon),
                Some(editor_data.color),
            );

            if editor_data.is_smart {
                let smart_rules = SmartCollectionRules {
                    rules: editor_data.smart_rules.into_iter().map(|r| SmartRule {
                        field: self.parse_rule_field(&r.field),
                        operator: self.parse_rule_operator(&r.operator),
                        value: r.value,
                    }).collect(),
                    match_type: if editor_data.match_type == "any" { MatchType::Any } else { MatchType::All },
                };
                collection.smart_rules = Some(smart_rules);
                collection.is_smart = true;
            } else {
                collection.is_smart = false;
                collection.smart_rules = None;
            }

            self.service.update_collection(&collection).await?;
            
            // Refresh collections
            let collections = self.service.get_all_collections().await?;
            self.update_collections(&collections).await;
        }

        Ok(())
    }

    /// Delete a collection
    pub async fn delete_collection(&self, collection_id: &str) -> Result<()> {
        self.service.delete_collection(collection_id).await?;
        
        // Refresh collections
        let collections = self.service.get_all_collections().await?;
        self.update_collections(&collections).await;
        
        // Refresh stats
        let stats = self.service.get_library_stats().await?;
        self.update_stats(&stats).await;
        
        Ok(())
    }

    /// Toggle collection favorite status
    pub async fn toggle_collection_favorite(&self, collection_id: &str) -> Result<()> {
        if let Some(mut collection) = self.service.get_collection(collection_id).await? {
            collection.toggle_favorite();
            self.service.update_collection(&collection).await?;
            
            // Refresh collections
            let collections = self.service.get_all_collections().await?;
            self.update_collections(&collections).await;
        }

        Ok(())
    }

    /// Add book to collection
    pub async fn add_book_to_collection(&self, book_id: String, collection_id: String) -> Result<()> {
        self.service.add_to_collection(book_id, collection_id).await?;
        
        // Refresh collections to update counts
        let collections = self.service.get_all_collections().await?;
        self.update_collections(&collections).await;
        
        Ok(())
    }

    /// Remove book from collection
    pub async fn remove_book_from_collection(&self, book_id: String, collection_id: String) -> Result<()> {
        self.service.remove_from_collection(book_id, collection_id).await?;
        
        // Refresh collections to update counts
        let collections = self.service.get_all_collections().await?;
        self.update_collections(&collections).await;
        
        Ok(())
    }

    /// Get books for a category
    pub async fn get_books_for_category(&self, category: &str) -> Result<Vec<String>> {
        let parsed_category = self.parse_category(category);
        self.service.get_books_by_category(parsed_category).await
    }

    /// Update reading status
    pub async fn update_reading_status(&self, book_id: &str, status: &str) -> Result<()> {
        let reading_status = match status {
            "want-to-read" => ReadingStatus::WantToRead,
            "currently-reading" => ReadingStatus::CurrentlyReading,
            "finished" => ReadingStatus::Finished,
            "dnf" => ReadingStatus::DNF,
            "reference" => ReadingStatus::Reference,
            _ => ReadingStatus::WantToRead,
        };

        self.service.update_reading_status(book_id, reading_status).await?;
        
        // Refresh stats
        let stats = self.service.get_library_stats().await?;
        self.update_stats(&stats).await;
        
        Ok(())
    }

    /// Get reading status for a book
    pub async fn get_reading_status(&self, book_id: &str) -> Result<Option<String>> {
        if let Some(status) = self.service.get_reading_status(book_id).await? {
            let status_str = match status {
                ReadingStatus::Unread => "unread",
                ReadingStatus::WantToRead => "want-to-read",
                ReadingStatus::CurrentlyReading => "currently-reading",
                ReadingStatus::OnHold => "on-hold",
                ReadingStatus::Finished => "finished",
                ReadingStatus::DNF => "dnf",
                ReadingStatus::Reference => "reference",
            };
            Ok(Some(status_str.to_string()))
        } else {
            Ok(None)
        }
    }

    /// Search library
    pub async fn search_library(&self, query: &str) -> Result<Vec<String>> {
        let filter = LibraryFilter {
            search_query: Some(query.to_string()),
            ..Default::default()
        };

        self.service.filter_books(&filter).await
    }

    /// Set current category
    pub async fn set_current_category(&self, category: String) {
        *self.current_category.write().await = category;
    }

    /// Get current category
    pub async fn get_current_category(&self) -> String {
        self.current_category.read().await.clone()
    }

    /// Apply library filter
    pub async fn apply_filter(&self, filter: LibraryFilter) -> Result<Vec<String>> {
        *self.current_filter.write().await = Some(filter.clone());
        self.service.filter_books(&filter).await
    }

    /// Clear library filter
    pub async fn clear_filter(&self) -> Result<()> {
        *self.current_filter.write().await = None;
        Ok(())
    }

    /// Sort books
    pub async fn sort_books(&self, book_ids: &[String], sort_by: &str, direction: &str) -> Result<Vec<String>> {
        let sort_by = match sort_by {
            "title" => LibrarySortBy::Title,
            "author" => LibrarySortBy::Author,
            "added-date" => LibrarySortBy::AddedDate,
            "read-date" => LibrarySortBy::ReadDate,
            "rating" => LibrarySortBy::Rating,
            "publish-date" => LibrarySortBy::PublishDate,
            "file-size" => LibrarySortBy::FileSize,
            "page-count" => LibrarySortBy::PageCount,
            "progress" => LibrarySortBy::Progress,
            "last-opened" => LibrarySortBy::LastOpened,
            "genre" => LibrarySortBy::Genre,
            "publisher" => LibrarySortBy::Publisher,
            "language" => LibrarySortBy::Language,
            _ => LibrarySortBy::Title,
        };

        let sort_direction = match direction {
            "desc" => SortDirection::Descending,
            _ => SortDirection::Ascending,
        };

        self.service.sort_books(book_ids, sort_by, sort_direction).await
    }

    /// Get collection editor data
    pub async fn get_collection_editor_data(&self, collection_id: &str) -> Result<Option<SlintCollectionEditor>> {
        if let Some(collection) = self.service.get_collection(collection_id).await? {
            let (smart_rules, match_type) = if let Some(ref rules) = collection.smart_rules {
                let rules_vec = rules.rules.iter().map(|rule| SlintSmartRule {
                    field: rule.field.to_display_name(),
                    operator: rule.operator.to_display_name(),
                    value: rule.value.clone(),
                }).collect();
                
                let match_type_str = match rules.match_type {
                    MatchType::All => "all",
                    MatchType::Any => "any",
                }.to_string();
                
                (rules_vec, match_type_str)
            } else {
                (Vec::new(), "all".to_string())
            };

            Ok(Some(SlintCollectionEditor {
                id: collection.id,
                name: collection.name,
                description: collection.description.unwrap_or_default(),
                icon: collection.icon,
                color: collection.color,
                is_smart: collection.is_smart,
                smart_rules,
                match_type,
            }))
        } else {
            Ok(None)
        }
    }

    /// Preview smart collection results
    pub async fn preview_smart_collection(&self, rules: Vec<SlintSmartRule>, match_type: String) -> Result<Vec<String>> {
        let smart_rules = SmartCollectionRules {
            rules: rules.into_iter().map(|r| SmartRule {
                field: self.parse_rule_field(&r.field),
                operator: self.parse_rule_operator(&r.operator),
                value: r.value,
            }).collect(),
            match_type: if match_type == "any" { MatchType::Any } else { MatchType::All },
        };

        // Create temporary collection for preview
        let temp_collection = Collection::new_smart(
            "temp".to_string(),
            "⚡".to_string(),
            "#007AFF".to_string(),
            smart_rules,
        );

        self.service.evaluate_smart_collection(&temp_collection.id).await
    }

    /// Parse category string to Category enum
    fn parse_category(&self, category: &str) -> Category {
        match category {
            "all" => Category::All,
            "want-to-read" => Category::WantToRead,
            "currently-reading" => Category::CurrentlyReading,
            "finished" => Category::Finished,
            _ => {
                if let Some(collection_id) = category.strip_prefix("collection:") {
                    Category::Collection(collection_id.to_string())
                } else if let Some(author_id) = category.strip_prefix("author:") {
                    Category::Author(author_id.to_string())
                } else if let Some(tag_id) = category.strip_prefix("tag:") {
                    Category::Tag(tag_id.to_string())
                } else {
                    Category::All
                }
            }
        }
    }

    /// Parse rule field string to SmartRuleField enum
    fn parse_rule_field(&self, field: &str) -> SmartRuleField {
        match field {
            "Title" => SmartRuleField::Title,
            "Author" => SmartRuleField::Author,
            "Genre" => SmartRuleField::Genre,
            "Publisher" => SmartRuleField::Publisher,
            "Language" => SmartRuleField::Language,
            "Publish Date" => SmartRuleField::PublishDate,
            "Added Date" => SmartRuleField::AddedDate,
            "Reading Status" => SmartRuleField::ReadingStatus,
            "Rating" => SmartRuleField::Rating,
            "Tags" => SmartRuleField::Tags,
            "File Size" => SmartRuleField::FileSize,
            "Page Count" => SmartRuleField::PageCount,
            "Progress" => SmartRuleField::Progress,
            _ => SmartRuleField::Title,
        }
    }

    /// Parse rule operator string to SmartRuleOperator enum
    fn parse_rule_operator(&self, operator: &str) -> SmartRuleOperator {
        match operator {
            "equals" => SmartRuleOperator::Equals,
            "does not equal" => SmartRuleOperator::NotEquals,
            "contains" => SmartRuleOperator::Contains,
            "does not contain" => SmartRuleOperator::NotContains,
            "starts with" => SmartRuleOperator::StartsWith,
            "ends with" => SmartRuleOperator::EndsWith,
            "greater than" => SmartRuleOperator::GreaterThan,
            "less than" => SmartRuleOperator::LessThan,
            "greater than or equal" => SmartRuleOperator::GreaterThanOrEqual,
            "less than or equal" => SmartRuleOperator::LessThanOrEqual,
            "is empty" => SmartRuleOperator::IsEmpty,
            "is not empty" => SmartRuleOperator::IsNotEmpty,
            "in last" => SmartRuleOperator::InLast,
            "before" => SmartRuleOperator::Before,
            "after" => SmartRuleOperator::After,
            _ => SmartRuleOperator::Equals,
        }
    }

    /// Refresh all data
    pub async fn refresh_all(&self) -> Result<()> {
        self.load_library_data().await
    }

    /// Get available rule fields
    pub fn get_available_rule_fields(&self) -> Vec<String> {
        vec![
            "Title".to_string(),
            "Author".to_string(),
            "Genre".to_string(),
            "Publisher".to_string(),
            "Language".to_string(),
            "Publish Date".to_string(),
            "Added Date".to_string(),
            "Reading Status".to_string(),
            "Rating".to_string(),
            "Tags".to_string(),
            "File Size".to_string(),
            "Page Count".to_string(),
            "Progress".to_string(),
        ]
    }

    /// Get available rule operators
    pub fn get_available_rule_operators(&self) -> Vec<String> {
        vec![
            "equals".to_string(),
            "does not equal".to_string(),
            "contains".to_string(),
            "does not contain".to_string(),
            "starts with".to_string(),
            "ends with".to_string(),
            "greater than".to_string(),
            "less than".to_string(),
            "greater than or equal".to_string(),
            "less than or equal".to_string(),
            "is empty".to_string(),
            "is not empty".to_string(),
            "in last".to_string(),
            "before".to_string(),
            "after".to_string(),
        ]
    }

    /// Get available collection icons
    pub fn get_available_icons(&self) -> Vec<String> {
        vec![
            "📂".to_string(), "📚".to_string(), "⭐".to_string(), "🎯".to_string(),
            "💡".to_string(), "📖".to_string(), "🏆".to_string(), "🎨".to_string(),
            "🔥".to_string(), "💎".to_string(), "🌟".to_string(), "🎪".to_string(),
            "🎵".to_string(), "🎬".to_string(), "🎮".to_string(), "🏃".to_string(),
            "🍎".to_string(), "🌸".to_string(), "🌊".to_string(), "🏔️".to_string(),
        ]
    }

    /// Get available collection colors
    pub fn get_available_colors(&self) -> Vec<String> {
        vec![
            "#007AFF".to_string(), "#FF6B6B".to_string(), "#4ECDC4".to_string(),
            "#45B7D1".to_string(), "#F9CA24".to_string(), "#6C5CE7".to_string(),
            "#FD79A8".to_string(), "#636E72".to_string(), "#00B894".to_string(),
            "#FDCB6E".to_string(), "#E17055".to_string(), "#74B9FF".to_string(),
            "#A29BFE".to_string(), "#FD79A8".to_string(), "#FDCB6E".to_string(),
        ]
    }
}