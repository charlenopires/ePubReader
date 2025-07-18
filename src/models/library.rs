use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use anyhow::Result;

/// Reading status for books
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ReadingStatus {
    Unread,
    WantToRead,
    CurrentlyReading,
    Finished,
    OnHold,
    DNF, // Did Not Finish
    Reference,
}

/// Library category types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Category {
    All,
    WantToRead,
    CurrentlyReading,
    Finished,
    Collection(String),
    Author(String),
    Tag(String),
    Genre(String),
    Publisher(String),
    Year(u32),
    Language(String),
    Rating(u8), // 1-5 stars
}

/// User-created collection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Collection {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub icon: String,
    pub color: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub book_ids: Vec<String>,
    pub is_smart: bool,
    pub smart_rules: Option<SmartCollectionRules>,
    pub is_favorite: bool,
    pub sort_order: u32,
}

/// Smart collection rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartCollectionRules {
    pub rules: Vec<SmartRule>,
    pub match_type: MatchType, // All or Any
}

/// Individual smart rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartRule {
    pub field: SmartRuleField,
    pub operator: SmartRuleOperator,
    pub value: String,
}

/// Fields that can be used in smart rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SmartRuleField {
    Title,
    Author,
    Genre,
    Publisher,
    Language,
    PublishDate,
    AddedDate,
    ReadingStatus,
    Rating,
    Tags,
    FileSize,
    PageCount,
    Progress,
}

/// Operators for smart rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SmartRuleOperator {
    Equals,
    NotEquals,
    Contains,
    NotContains,
    StartsWith,
    EndsWith,
    GreaterThan,
    LessThan,
    GreaterThanOrEqual,
    LessThanOrEqual,
    IsEmpty,
    IsNotEmpty,
    InLast, // For dates - "in last X days"
    Before,
    After,
}

/// Match type for smart collections
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MatchType {
    All, // All rules must match
    Any, // Any rule must match
}

/// Library statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryStats {
    pub total_books: u32,
    pub want_to_read: u32,
    pub currently_reading: u32,
    pub finished: u32,
    pub total_collections: u32,
    pub total_authors: u32,
    pub total_tags: u32,
    pub total_genres: u32,
    pub reading_streak: u32,
    pub books_read_this_month: u32,
    pub books_read_this_year: u32,
    pub total_reading_time: u64, // in minutes
    pub average_rating: f32,
    pub favorite_genres: Vec<(String, u32)>,
    pub top_authors: Vec<(String, u32)>,
    pub reading_goals: Option<ReadingGoals>,
}

/// Reading goals
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingGoals {
    pub yearly_goal: u32,
    pub monthly_goal: u32,
    pub current_year_progress: u32,
    pub current_month_progress: u32,
    pub streak_goal: u32,
    pub current_streak: u32,
}

/// Library filter options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct LibraryFilter {
    pub category: Option<Category>,
    pub author: Option<String>,
    pub genre: Option<String>,
    pub language: Option<String>,
    pub publisher: Option<String>,
    pub year_range: Option<(u32, u32)>,
    pub rating_range: Option<(u8, u8)>,
    pub reading_status: Option<ReadingStatus>,
    pub tags: Vec<String>,
    pub search_query: Option<String>,
    pub has_cover: Option<bool>,
    pub file_format: Option<String>,
    pub added_date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
    pub read_date_range: Option<(DateTime<Utc>, DateTime<Utc>)>,
}

/// Library sort options
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LibrarySortBy {
    Title,
    Author,
    AddedDate,
    ReadDate,
    Rating,
    PublishDate,
    FileSize,
    PageCount,
    Progress,
    LastOpened,
    Genre,
    Publisher,
    Language,
}

/// Sort direction
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

/// Library view mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LibraryViewMode {
    Grid,
    List,
    Compact,
    Cover,
    Table,
}

/// Author information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Author {
    pub id: String,
    pub name: String,
    pub bio: Option<String>,
    pub birth_date: Option<DateTime<Utc>>,
    pub death_date: Option<DateTime<Utc>>,
    pub nationality: Option<String>,
    pub photo_url: Option<String>,
    pub website: Option<String>,
    pub book_count: u32,
    pub average_rating: f32,
    pub genres: Vec<String>,
    pub aliases: Vec<String>,
}

/// Genre information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Genre {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub parent_genre: Option<String>,
    pub book_count: u32,
    pub average_rating: f32,
    pub color: String,
    pub icon: String,
}

/// Tag information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub book_count: u32,
    pub created_at: DateTime<Utc>,
    pub description: Option<String>,
}

/// Library organization service trait
#[async_trait::async_trait]
pub trait LibraryOrganizer {
    /// Collections
    async fn create_collection(&self, name: String, icon: String, color: String) -> anyhow::Result<Collection>;
    async fn update_collection(&self, collection: &Collection) -> anyhow::Result<()>;
    async fn delete_collection(&self, collection_id: &str) -> anyhow::Result<()>;
    async fn get_collection(&self, collection_id: &str) -> anyhow::Result<Option<Collection>>;
    async fn get_all_collections(&self) -> anyhow::Result<Vec<Collection>>;
    
    /// Collection management
    async fn add_to_collection(&self, book_id: String, collection_id: String) -> anyhow::Result<()>;
    async fn remove_from_collection(&self, book_id: String, collection_id: String) -> anyhow::Result<()>;
    async fn get_books_by_category(&self, category: Category) -> anyhow::Result<Vec<String>>;
    
    /// Smart collections
    async fn create_smart_collection(&self, name: String, rules: SmartCollectionRules) -> anyhow::Result<Collection>;
    async fn get_smart_collections(&self) -> anyhow::Result<Vec<Collection>>;
    async fn update_smart_collection_books(&self, collection_id: &str) -> anyhow::Result<()>;
    
    /// Library statistics
    async fn get_library_stats(&self) -> anyhow::Result<LibraryStats>;
    
    /// Reading status
    async fn update_reading_status(&self, book_id: &str, status: ReadingStatus) -> anyhow::Result<()>;
    async fn get_reading_status(&self, book_id: &str) -> anyhow::Result<Option<ReadingStatus>>;
    
    /// Authors, genres, tags
    async fn get_all_authors(&self) -> anyhow::Result<Vec<Author>>;
    async fn get_all_genres(&self) -> anyhow::Result<Vec<Genre>>;
    async fn get_all_tags(&self) -> anyhow::Result<Vec<Tag>>;
    
    /// Filtering and sorting
    async fn filter_books(&self, filter: &LibraryFilter) -> anyhow::Result<Vec<String>>;
    async fn sort_books(&self, book_ids: &[String], sort_by: LibrarySortBy, direction: SortDirection) -> anyhow::Result<Vec<String>>;
}

impl Collection {
    /// Create a new collection
    pub fn new(name: String, icon: String, color: String) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            icon,
            color,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            book_ids: Vec::new(),
            is_smart: false,
            smart_rules: None,
            is_favorite: false,
            sort_order: 0,
        }
    }

    /// Create a new smart collection
    pub fn new_smart(name: String, icon: String, color: String, rules: SmartCollectionRules) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            description: None,
            icon,
            color,
            created_at: Utc::now(),
            updated_at: Utc::now(),
            book_ids: Vec::new(),
            is_smart: true,
            smart_rules: Some(rules),
            is_favorite: false,
            sort_order: 0,
        }
    }

    /// Add a book to the collection
    pub fn add_book(&mut self, book_id: String) {
        if !self.book_ids.contains(&book_id) {
            self.book_ids.push(book_id);
            self.updated_at = Utc::now();
        }
    }

    /// Remove a book from the collection
    pub fn remove_book(&mut self, book_id: &str) {
        if let Some(pos) = self.book_ids.iter().position(|id| id == book_id) {
            self.book_ids.remove(pos);
            self.updated_at = Utc::now();
        }
    }

    /// Get book count
    pub fn book_count(&self) -> usize {
        self.book_ids.len()
    }

    /// Toggle favorite status
    pub fn toggle_favorite(&mut self) {
        self.is_favorite = !self.is_favorite;
        self.updated_at = Utc::now();
    }

    /// Update collection info
    pub fn update_info(&mut self, name: Option<String>, description: Option<String>, icon: Option<String>, color: Option<String>) {
        if let Some(name) = name {
            self.name = name;
        }
        if let Some(description) = description {
            self.description = Some(description);
        }
        if let Some(icon) = icon {
            self.icon = icon;
        }
        if let Some(color) = color {
            self.color = color;
        }
        self.updated_at = Utc::now();
    }
}

impl SmartCollectionRules {
    /// Create new smart collection rules
    pub fn new(match_type: MatchType) -> Self {
        Self {
            rules: Vec::new(),
            match_type,
        }
    }

    /// Add a rule
    pub fn add_rule(&mut self, field: SmartRuleField, operator: SmartRuleOperator, value: String) {
        self.rules.push(SmartRule {
            field,
            operator,
            value,
        });
    }

    /// Remove a rule
    pub fn remove_rule(&mut self, index: usize) {
        if index < self.rules.len() {
            self.rules.remove(index);
        }
    }

    /// Check if rules are empty
    pub fn is_empty(&self) -> bool {
        self.rules.is_empty()
    }
}

impl ReadingStatus {
    /// Get display name
    pub fn to_display_name(&self) -> String {
        match self {
            ReadingStatus::Unread => "Unread".to_string(),
            ReadingStatus::WantToRead => "Want to Read".to_string(),
            ReadingStatus::CurrentlyReading => "Currently Reading".to_string(),
            ReadingStatus::Finished => "Finished".to_string(),
            ReadingStatus::OnHold => "On Hold".to_string(),
            ReadingStatus::DNF => "Did Not Finish".to_string(),
            ReadingStatus::Reference => "Reference".to_string(),
        }
    }

    /// Get icon
    pub fn to_icon(&self) -> String {
        match self {
            ReadingStatus::Unread => "📚".to_string(),
            ReadingStatus::WantToRead => "📚".to_string(),
            ReadingStatus::CurrentlyReading => "📖".to_string(),
            ReadingStatus::Finished => "✅".to_string(),
            ReadingStatus::OnHold => "⏸️".to_string(),
            ReadingStatus::DNF => "❌".to_string(),
            ReadingStatus::Reference => "📑".to_string(),
        }
    }

    /// Get color
    pub fn to_color(&self) -> String {
        match self {
            ReadingStatus::Unread => "#95a5a6".to_string(),
            ReadingStatus::WantToRead => "#3498db".to_string(),
            ReadingStatus::CurrentlyReading => "#f39c12".to_string(),
            ReadingStatus::Finished => "#27ae60".to_string(),
            ReadingStatus::OnHold => "#8e44ad".to_string(),
            ReadingStatus::DNF => "#e74c3c".to_string(),
            ReadingStatus::Reference => "#9b59b6".to_string(),
        }
    }

    /// Get all statuses
    pub fn all() -> Vec<ReadingStatus> {
        vec![
            ReadingStatus::Unread,
            ReadingStatus::WantToRead,
            ReadingStatus::CurrentlyReading,
            ReadingStatus::Finished,
            ReadingStatus::OnHold,
            ReadingStatus::DNF,
            ReadingStatus::Reference,
        ]
    }

    /// Convert from string representation
    pub fn from_string(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "unread" => Some(ReadingStatus::Unread),
            "want_to_read" | "wanttoread" => Some(ReadingStatus::WantToRead),
            "currently_reading" | "currentlyreading" | "reading" => Some(ReadingStatus::CurrentlyReading),
            "finished" => Some(ReadingStatus::Finished),
            "on_hold" | "onhold" => Some(ReadingStatus::OnHold),
            "dnf" | "did_not_finish" => Some(ReadingStatus::DNF),
            "reference" => Some(ReadingStatus::Reference),
            _ => None,
        }
    }

    /// Convert to string representation
    pub fn to_string(&self) -> String {
        match self {
            ReadingStatus::Unread => "unread".to_string(),
            ReadingStatus::WantToRead => "want_to_read".to_string(),
            ReadingStatus::CurrentlyReading => "currently_reading".to_string(),
            ReadingStatus::Finished => "finished".to_string(),
            ReadingStatus::OnHold => "on_hold".to_string(),
            ReadingStatus::DNF => "dnf".to_string(),
            ReadingStatus::Reference => "reference".to_string(),
        }
    }
}

impl Category {
    /// Get display name
    pub fn to_display_name(&self) -> String {
        match self {
            Category::All => "All Books".to_string(),
            Category::WantToRead => "Want to Read".to_string(),
            Category::CurrentlyReading => "Currently Reading".to_string(),
            Category::Finished => "Finished".to_string(),
            Category::Collection(name) => name.clone(),
            Category::Author(name) => format!("Author: {}", name),
            Category::Tag(name) => format!("Tag: {}", name),
            Category::Genre(name) => format!("Genre: {}", name),
            Category::Publisher(name) => format!("Publisher: {}", name),
            Category::Year(year) => format!("Year: {}", year),
            Category::Language(lang) => format!("Language: {}", lang),
            Category::Rating(rating) => format!("{} Stars", rating),
        }
    }

    /// Get icon
    pub fn to_icon(&self) -> String {
        match self {
            Category::All => "📚".to_string(),
            Category::WantToRead => "📚".to_string(),
            Category::CurrentlyReading => "📖".to_string(),
            Category::Finished => "✅".to_string(),
            Category::Collection(_) => "📂".to_string(),
            Category::Author(_) => "👤".to_string(),
            Category::Tag(_) => "🏷️".to_string(),
            Category::Genre(_) => "🎭".to_string(),
            Category::Publisher(_) => "🏢".to_string(),
            Category::Year(_) => "📅".to_string(),
            Category::Language(_) => "🌐".to_string(),
            Category::Rating(_) => "⭐".to_string(),
        }
    }
}

impl Default for LibraryStats {
    fn default() -> Self {
        Self {
            total_books: 0,
            want_to_read: 0,
            currently_reading: 0,
            finished: 0,
            total_collections: 0,
            total_authors: 0,
            total_tags: 0,
            total_genres: 0,
            reading_streak: 0,
            books_read_this_month: 0,
            books_read_this_year: 0,
            total_reading_time: 0,
            average_rating: 0.0,
            favorite_genres: Vec::new(),
            top_authors: Vec::new(),
            reading_goals: None,
        }
    }
}

impl SmartRuleField {
    /// Get display name
    pub fn to_display_name(&self) -> String {
        match self {
            SmartRuleField::Title => "Title".to_string(),
            SmartRuleField::Author => "Author".to_string(),
            SmartRuleField::Genre => "Genre".to_string(),
            SmartRuleField::Publisher => "Publisher".to_string(),
            SmartRuleField::Language => "Language".to_string(),
            SmartRuleField::PublishDate => "Publish Date".to_string(),
            SmartRuleField::AddedDate => "Added Date".to_string(),
            SmartRuleField::ReadingStatus => "Reading Status".to_string(),
            SmartRuleField::Rating => "Rating".to_string(),
            SmartRuleField::Tags => "Tags".to_string(),
            SmartRuleField::FileSize => "File Size".to_string(),
            SmartRuleField::PageCount => "Page Count".to_string(),
            SmartRuleField::Progress => "Progress".to_string(),
        }
    }

    /// Get available operators for this field
    pub fn available_operators(&self) -> Vec<SmartRuleOperator> {
        match self {
            SmartRuleField::Title | SmartRuleField::Author | SmartRuleField::Genre | 
            SmartRuleField::Publisher | SmartRuleField::Language | SmartRuleField::Tags => {
                vec![
                    SmartRuleOperator::Equals,
                    SmartRuleOperator::NotEquals,
                    SmartRuleOperator::Contains,
                    SmartRuleOperator::NotContains,
                    SmartRuleOperator::StartsWith,
                    SmartRuleOperator::EndsWith,
                    SmartRuleOperator::IsEmpty,
                    SmartRuleOperator::IsNotEmpty,
                ]
            }
            SmartRuleField::PublishDate | SmartRuleField::AddedDate => {
                vec![
                    SmartRuleOperator::Equals,
                    SmartRuleOperator::NotEquals,
                    SmartRuleOperator::Before,
                    SmartRuleOperator::After,
                    SmartRuleOperator::InLast,
                    SmartRuleOperator::IsEmpty,
                    SmartRuleOperator::IsNotEmpty,
                ]
            }
            SmartRuleField::Rating | SmartRuleField::FileSize | SmartRuleField::PageCount | SmartRuleField::Progress => {
                vec![
                    SmartRuleOperator::Equals,
                    SmartRuleOperator::NotEquals,
                    SmartRuleOperator::GreaterThan,
                    SmartRuleOperator::LessThan,
                    SmartRuleOperator::GreaterThanOrEqual,
                    SmartRuleOperator::LessThanOrEqual,
                    SmartRuleOperator::IsEmpty,
                    SmartRuleOperator::IsNotEmpty,
                ]
            }
            SmartRuleField::ReadingStatus => {
                vec![
                    SmartRuleOperator::Equals,
                    SmartRuleOperator::NotEquals,
                ]
            }
        }
    }
}

impl SmartRuleOperator {
    /// Get display name
    pub fn to_display_name(&self) -> String {
        match self {
            SmartRuleOperator::Equals => "equals".to_string(),
            SmartRuleOperator::NotEquals => "does not equal".to_string(),
            SmartRuleOperator::Contains => "contains".to_string(),
            SmartRuleOperator::NotContains => "does not contain".to_string(),
            SmartRuleOperator::StartsWith => "starts with".to_string(),
            SmartRuleOperator::EndsWith => "ends with".to_string(),
            SmartRuleOperator::GreaterThan => "greater than".to_string(),
            SmartRuleOperator::LessThan => "less than".to_string(),
            SmartRuleOperator::GreaterThanOrEqual => "greater than or equal".to_string(),
            SmartRuleOperator::LessThanOrEqual => "less than or equal".to_string(),
            SmartRuleOperator::IsEmpty => "is empty".to_string(),
            SmartRuleOperator::IsNotEmpty => "is not empty".to_string(),
            SmartRuleOperator::InLast => "in last".to_string(),
            SmartRuleOperator::Before => "before".to_string(),
            SmartRuleOperator::After => "after".to_string(),
        }
    }
}