use std::collections::HashMap;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration, Datelike};
use sqlx::{Row, SqlitePool, sqlite::SqliteRow};
use uuid::Uuid;

use crate::models::library::{
    Collection, SmartCollectionRules, SmartRule, SmartRuleField, SmartRuleOperator, MatchType,
    Category, ReadingStatus, LibraryStats, LibraryFilter, LibrarySortBy, SortDirection,
    Author, Genre, Tag, LibraryOrganizer,
};
use crate::models::book::Book;

#[derive(Clone)]
pub struct LibraryService {
    pool: SqlitePool,
}

impl LibraryService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Initialize library tables
    pub async fn init_tables(&self) -> Result<()> {
        // Create collections table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS collections (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                description TEXT,
                icon TEXT NOT NULL,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                is_smart BOOLEAN NOT NULL DEFAULT FALSE,
                smart_rules TEXT, -- JSON
                is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
                sort_order INTEGER NOT NULL DEFAULT 0
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create collection_books table (many-to-many relationship)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS collection_books (
                collection_id TEXT NOT NULL,
                book_id TEXT NOT NULL,
                added_at TEXT NOT NULL,
                PRIMARY KEY (collection_id, book_id),
                FOREIGN KEY (collection_id) REFERENCES collections (id) ON DELETE CASCADE,
                FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create reading_status table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS reading_status (
                book_id TEXT PRIMARY KEY,
                status TEXT NOT NULL,
                started_at TEXT,
                finished_at TEXT,
                progress REAL DEFAULT 0.0,
                notes TEXT,
                FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create authors table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS authors (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                bio TEXT,
                birth_date TEXT,
                death_date TEXT,
                nationality TEXT,
                photo_url TEXT,
                website TEXT,
                created_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create genres table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS genres (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                description TEXT,
                parent_genre TEXT,
                color TEXT NOT NULL,
                icon TEXT NOT NULL,
                created_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create tags table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT,
                description TEXT,
                created_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create book_tags table (many-to-many relationship)
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS book_tags (
                book_id TEXT NOT NULL,
                tag_id TEXT NOT NULL,
                added_at TEXT NOT NULL,
                PRIMARY KEY (book_id, tag_id),
                FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE CASCADE,
                FOREIGN KEY (tag_id) REFERENCES tags (id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for better performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_collections_sort_order ON collections(sort_order);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_collection_books_collection_id ON collection_books(collection_id);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_collection_books_book_id ON collection_books(book_id);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_reading_status_status ON reading_status(status);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_book_tags_book_id ON book_tags(book_id);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_book_tags_tag_id ON book_tags(tag_id);")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get library statistics
    pub async fn get_library_stats(&self) -> Result<LibraryStats> {
        // Get total books
        let total_books: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM books")
            .fetch_one(&self.pool)
            .await?;

        // Get reading status counts
        let want_to_read: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM reading_status WHERE status = 'WantToRead'"
        )
        .fetch_one(&self.pool)
        .await?;

        let currently_reading: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM reading_status WHERE status = 'CurrentlyReading'"
        )
        .fetch_one(&self.pool)
        .await?;

        let finished: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM reading_status WHERE status = 'Finished'"
        )
        .fetch_one(&self.pool)
        .await?;

        // Get other counts
        let total_collections: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM collections")
            .fetch_one(&self.pool)
            .await?;

        let total_authors: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM authors")
            .fetch_one(&self.pool)
            .await?;

        let total_tags: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM tags")
            .fetch_one(&self.pool)
            .await?;

        let total_genres: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM genres")
            .fetch_one(&self.pool)
            .await?;

        // Get reading streak and monthly/yearly stats
        let reading_streak = self.calculate_reading_streak().await?;
        let books_read_this_month = self.get_books_read_this_period("month").await?;
        let books_read_this_year = self.get_books_read_this_period("year").await?;

        // Get average rating
        let average_rating: Option<f64> = sqlx::query_scalar(
            "SELECT AVG(rating) FROM books WHERE rating > 0"
        )
        .fetch_one(&self.pool)
        .await?;

        // Get favorite genres
        let favorite_genres = self.get_favorite_genres().await?;

        // Get top authors
        let top_authors = self.get_top_authors().await?;

        Ok(LibraryStats {
            total_books: total_books as u32,
            want_to_read: want_to_read as u32,
            currently_reading: currently_reading as u32,
            finished: finished as u32,
            total_collections: total_collections as u32,
            total_authors: total_authors as u32,
            total_tags: total_tags as u32,
            total_genres: total_genres as u32,
            reading_streak,
            books_read_this_month,
            books_read_this_year,
            total_reading_time: 0, // TODO: Implement reading time tracking
            average_rating: average_rating.unwrap_or(0.0) as f32,
            favorite_genres,
            top_authors,
            reading_goals: None, // TODO: Implement reading goals
        })
    }

    /// Calculate reading streak
    async fn calculate_reading_streak(&self) -> Result<u32> {
        let finished_books = sqlx::query(
            "SELECT finished_at FROM reading_status WHERE status = 'Finished' AND finished_at IS NOT NULL ORDER BY finished_at DESC"
        )
        .fetch_all(&self.pool)
        .await?;

        let mut streak = 0u32;
        let mut current_date = Utc::now().date_naive();

        for row in finished_books {
            let finished_at_str: String = row.get(0);
            if let Ok(finished_at) = DateTime::parse_from_rfc3339(&finished_at_str) {
                let finished_date = finished_at.date_naive();
                
                if finished_date == current_date || finished_date == current_date - Duration::days(1) {
                    streak += 1;
                    current_date = finished_date - Duration::days(1);
                } else {
                    break;
                }
            }
        }

        Ok(streak)
    }

    /// Get books read in a specific period
    async fn get_books_read_this_period(&self, period: &str) -> Result<u32> {
        let (start_date, _) = match period {
            "month" => {
                let now = Utc::now();
                let start_of_month = now.date_naive().with_day(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                (start_of_month, now.naive_utc())
            }
            "year" => {
                let now = Utc::now();
                let start_of_year = now.date_naive().with_ordinal(1).unwrap().and_hms_opt(0, 0, 0).unwrap();
                (start_of_year, now.naive_utc())
            }
            _ => return Ok(0),
        };

        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM reading_status WHERE status = 'Finished' AND finished_at >= ?"
        )
        .bind(start_date.format("%Y-%m-%d %H:%M:%S").to_string())
        .fetch_one(&self.pool)
        .await?;

        Ok(count as u32)
    }

    /// Get favorite genres
    async fn get_favorite_genres(&self) -> Result<Vec<(String, u32)>> {
        let rows = sqlx::query(
            r#"
            SELECT b.genre, COUNT(*) as count
            FROM books b
            WHERE b.genre IS NOT NULL AND b.genre != ''
            GROUP BY b.genre
            ORDER BY count DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut genres = Vec::new();
        for row in rows {
            let genre: String = row.get(0);
            let count: i64 = row.get(1);
            genres.push((genre, count as u32));
        }

        Ok(genres)
    }

    /// Get top authors
    async fn get_top_authors(&self) -> Result<Vec<(String, u32)>> {
        let rows = sqlx::query(
            r#"
            SELECT b.author, COUNT(*) as count
            FROM books b
            WHERE b.author IS NOT NULL AND b.author != ''
            GROUP BY b.author
            ORDER BY count DESC
            LIMIT 10
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        let mut authors = Vec::new();
        for row in rows {
            let author: String = row.get(0);
            let count: i64 = row.get(1);
            authors.push((author, count as u32));
        }

        Ok(authors)
    }

    /// Evaluate smart collection rules
    pub async fn evaluate_smart_collection(&self, collection_id: &str) -> Result<Vec<String>> {
        let collection = self.get_collection(collection_id).await?;
        
        if let Some(collection) = collection {
            if let Some(rules) = &collection.smart_rules {
                return self.evaluate_smart_rules(rules).await;
            }
        }
        
        Ok(Vec::new())
    }

    /// Evaluate smart rules against all books
    async fn evaluate_smart_rules(&self, rules: &SmartCollectionRules) -> Result<Vec<String>> {
        if rules.rules.is_empty() {
            return Ok(Vec::new());
        }

        let all_books = sqlx::query("SELECT * FROM books")
            .fetch_all(&self.pool)
            .await?;

        let mut matching_books = Vec::new();

        for row in all_books {
            let book_id: String = row.get("id");
            let matches = self.evaluate_rules_for_book(&row, rules).await?;

            let book_matches = match rules.match_type {
                MatchType::All => matches.iter().all(|&m| m),
                MatchType::Any => matches.iter().any(|&m| m),
            };

            if book_matches {
                matching_books.push(book_id);
            }
        }

        Ok(matching_books)
    }

    /// Evaluate rules for a specific book
    async fn evaluate_rules_for_book(&self, book_row: &SqliteRow, rules: &SmartCollectionRules) -> Result<Vec<bool>> {
        let mut results = Vec::new();

        for rule in &rules.rules {
            let matches = self.evaluate_single_rule(book_row, rule).await?;
            results.push(matches);
        }

        Ok(results)
    }

    /// Evaluate a single rule
    async fn evaluate_single_rule(&self, book_row: &SqliteRow, rule: &SmartRule) -> Result<bool> {
        let field_value = self.get_field_value(book_row, &rule.field).await?;
        
        match rule.operator {
            SmartRuleOperator::Equals => Ok(field_value == rule.value),
            SmartRuleOperator::NotEquals => Ok(field_value != rule.value),
            SmartRuleOperator::Contains => Ok(field_value.to_lowercase().contains(&rule.value.to_lowercase())),
            SmartRuleOperator::NotContains => Ok(!field_value.to_lowercase().contains(&rule.value.to_lowercase())),
            SmartRuleOperator::StartsWith => Ok(field_value.to_lowercase().starts_with(&rule.value.to_lowercase())),
            SmartRuleOperator::EndsWith => Ok(field_value.to_lowercase().ends_with(&rule.value.to_lowercase())),
            SmartRuleOperator::IsEmpty => Ok(field_value.is_empty()),
            SmartRuleOperator::IsNotEmpty => Ok(!field_value.is_empty()),
            SmartRuleOperator::GreaterThan => {
                if let (Ok(field_num), Ok(rule_num)) = (field_value.parse::<f64>(), rule.value.parse::<f64>()) {
                    Ok(field_num > rule_num)
                } else {
                    Ok(false)
                }
            }
            SmartRuleOperator::LessThan => {
                if let (Ok(field_num), Ok(rule_num)) = (field_value.parse::<f64>(), rule.value.parse::<f64>()) {
                    Ok(field_num < rule_num)
                } else {
                    Ok(false)
                }
            }
            SmartRuleOperator::GreaterThanOrEqual => {
                if let (Ok(field_num), Ok(rule_num)) = (field_value.parse::<f64>(), rule.value.parse::<f64>()) {
                    Ok(field_num >= rule_num)
                } else {
                    Ok(false)
                }
            }
            SmartRuleOperator::LessThanOrEqual => {
                if let (Ok(field_num), Ok(rule_num)) = (field_value.parse::<f64>(), rule.value.parse::<f64>()) {
                    Ok(field_num <= rule_num)
                } else {
                    Ok(false)
                }
            }
            SmartRuleOperator::Before => {
                if let (Ok(field_date), Ok(rule_date)) = (
                    DateTime::parse_from_rfc3339(&field_value),
                    DateTime::parse_from_rfc3339(&rule.value)
                ) {
                    Ok(field_date < rule_date)
                } else {
                    Ok(false)
                }
            }
            SmartRuleOperator::After => {
                if let (Ok(field_date), Ok(rule_date)) = (
                    DateTime::parse_from_rfc3339(&field_value),
                    DateTime::parse_from_rfc3339(&rule.value)
                ) {
                    Ok(field_date > rule_date)
                } else {
                    Ok(false)
                }
            }
            SmartRuleOperator::InLast => {
                if let (Ok(field_date), Ok(days)) = (
                    DateTime::parse_from_rfc3339(&field_value),
                    rule.value.parse::<i64>()
                ) {
                    let cutoff_date = Utc::now() - Duration::days(days);
                    Ok(field_date.with_timezone(&Utc) >= cutoff_date)
                } else {
                    Ok(false)
                }
            }
        }
    }

    /// Get field value from book row
    async fn get_field_value(&self, book_row: &SqliteRow, field: &SmartRuleField) -> Result<String> {
        match field {
            SmartRuleField::Title => Ok(book_row.get::<Option<String>, _>("title").unwrap_or_default()),
            SmartRuleField::Author => Ok(book_row.get::<Option<String>, _>("author").unwrap_or_default()),
            SmartRuleField::Genre => Ok(book_row.get::<Option<String>, _>("genre").unwrap_or_default()),
            SmartRuleField::Publisher => Ok(book_row.get::<Option<String>, _>("publisher").unwrap_or_default()),
            SmartRuleField::Language => Ok(book_row.get::<Option<String>, _>("language").unwrap_or_default()),
            SmartRuleField::PublishDate => Ok(book_row.get::<Option<String>, _>("publish_date").unwrap_or_default()),
            SmartRuleField::AddedDate => Ok(book_row.get::<String, _>("added_date")),
            SmartRuleField::Rating => Ok(book_row.get::<Option<f64>, _>("rating").unwrap_or(0.0).to_string()),
            SmartRuleField::FileSize => Ok(book_row.get::<Option<i64>, _>("file_size").unwrap_or(0).to_string()),
            SmartRuleField::PageCount => Ok(book_row.get::<Option<i64>, _>("page_count").unwrap_or(0).to_string()),
            SmartRuleField::ReadingStatus => {
                let book_id: String = book_row.get("id");
                let status = self.get_reading_status(&book_id).await?;
                Ok(status.map(|s| s.to_display_name()).unwrap_or_default())
            }
            SmartRuleField::Tags => {
                let book_id: String = book_row.get("id");
                let tags = self.get_book_tags(&book_id).await?;
                Ok(tags.join(", "))
            }
            SmartRuleField::Progress => {
                let book_id: String = book_row.get("id");
                let progress = self.get_book_progress(&book_id).await?;
                Ok(progress.to_string())
            }
        }
    }

    /// Get book tags
    async fn get_book_tags(&self, book_id: &str) -> Result<Vec<String>> {
        let rows = sqlx::query(
            "SELECT t.name FROM tags t JOIN book_tags bt ON t.id = bt.tag_id WHERE bt.book_id = ?"
        )
        .bind(book_id)
        .fetch_all(&self.pool)
        .await?;

        let mut tags = Vec::new();
        for row in rows {
            let tag: String = row.get(0);
            tags.push(tag);
        }

        Ok(tags)
    }

    /// Get book progress
    async fn get_book_progress(&self, book_id: &str) -> Result<f64> {
        let progress: Option<f64> = sqlx::query_scalar(
            "SELECT progress FROM reading_status WHERE book_id = ?"
        )
        .bind(book_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(progress.unwrap_or(0.0))
    }

    /// Convert database row to Collection
    fn row_to_collection(&self, row: SqliteRow) -> Result<Collection> {
        let smart_rules_json: Option<String> = row.get("smart_rules");
        let smart_rules: Option<SmartCollectionRules> = if let Some(json) = smart_rules_json {
            serde_json::from_str(&json).ok()
        } else {
            None
        };

        let created_at_str: String = row.get("created_at");
        let updated_at_str: String = row.get("updated_at");

        Ok(Collection {
            id: row.get("id"),
            name: row.get("name"),
            description: row.get("description"),
            icon: row.get("icon"),
            color: row.get("color"),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
            updated_at: DateTime::parse_from_rfc3339(&updated_at_str)?.with_timezone(&Utc),
            book_ids: Vec::new(), // Will be populated separately
            is_smart: row.get("is_smart"),
            smart_rules,
            is_favorite: row.get("is_favorite"),
            sort_order: row.get::<i64, _>("sort_order") as u32,
        })
    }

    /// Load book IDs for a collection
    async fn load_collection_books(&self, collection: &mut Collection) -> Result<()> {
        if collection.is_smart {
            // For smart collections, evaluate rules
            collection.book_ids = self.evaluate_smart_collection(&collection.id).await?;
        } else {
            // For regular collections, load from database
            let rows = sqlx::query(
                "SELECT book_id FROM collection_books WHERE collection_id = ? ORDER BY added_at"
            )
            .bind(&collection.id)
            .fetch_all(&self.pool)
            .await?;

            collection.book_ids = rows.into_iter().map(|row| row.get(0)).collect();
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl LibraryOrganizer for LibraryService {
    async fn create_collection(&self, name: String, icon: String, color: String) -> Result<Collection> {
        let collection = Collection::new(name, icon, color);
        
        sqlx::query(
            r#"
            INSERT INTO collections (id, name, description, icon, color, created_at, updated_at, is_smart, is_favorite, sort_order)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&collection.id)
        .bind(&collection.name)
        .bind(&collection.description)
        .bind(&collection.icon)
        .bind(&collection.color)
        .bind(collection.created_at.to_rfc3339())
        .bind(collection.updated_at.to_rfc3339())
        .bind(collection.is_smart)
        .bind(collection.is_favorite)
        .bind(collection.sort_order as i64)
        .execute(&self.pool)
        .await?;

        Ok(collection)
    }

    async fn update_collection(&self, collection: &Collection) -> Result<()> {
        let smart_rules_json = collection.smart_rules.as_ref()
            .map(|rules| serde_json::to_string(rules))
            .transpose()?;

        sqlx::query(
            r#"
            UPDATE collections SET
                name = ?, description = ?, icon = ?, color = ?, updated_at = ?,
                is_smart = ?, smart_rules = ?, is_favorite = ?, sort_order = ?
            WHERE id = ?
            "#
        )
        .bind(&collection.name)
        .bind(&collection.description)
        .bind(&collection.icon)
        .bind(&collection.color)
        .bind(collection.updated_at.to_rfc3339())
        .bind(collection.is_smart)
        .bind(&smart_rules_json)
        .bind(collection.is_favorite)
        .bind(collection.sort_order as i64)
        .bind(&collection.id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn delete_collection(&self, collection_id: &str) -> Result<()> {
        sqlx::query("DELETE FROM collections WHERE id = ?")
            .bind(collection_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_collection(&self, collection_id: &str) -> Result<Option<Collection>> {
        let row = sqlx::query("SELECT * FROM collections WHERE id = ?")
            .bind(collection_id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => {
                let mut collection = self.row_to_collection(row)?;
                self.load_collection_books(&mut collection).await?;
                Ok(Some(collection))
            }
            None => Ok(None),
        }
    }

    async fn get_all_collections(&self) -> Result<Vec<Collection>> {
        let rows = sqlx::query("SELECT * FROM collections ORDER BY sort_order, name")
            .fetch_all(&self.pool)
            .await?;

        let mut collections = Vec::new();
        for row in rows {
            let mut collection = self.row_to_collection(row)?;
            self.load_collection_books(&mut collection).await?;
            collections.push(collection);
        }

        Ok(collections)
    }

    async fn add_to_collection(&self, book_id: String, collection_id: String) -> Result<()> {
        sqlx::query(
            "INSERT OR IGNORE INTO collection_books (collection_id, book_id, added_at) VALUES (?, ?, ?)"
        )
        .bind(&collection_id)
        .bind(&book_id)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn remove_from_collection(&self, book_id: String, collection_id: String) -> Result<()> {
        sqlx::query("DELETE FROM collection_books WHERE collection_id = ? AND book_id = ?")
            .bind(&collection_id)
            .bind(&book_id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn get_books_by_category(&self, category: Category) -> Result<Vec<String>> {
        match category {
            Category::All => {
                let rows = sqlx::query("SELECT id FROM books ORDER BY title")
                    .fetch_all(&self.pool)
                    .await?;
                Ok(rows.into_iter().map(|row| row.get(0)).collect())
            }
            Category::WantToRead => {
                let rows = sqlx::query(
                    "SELECT book_id FROM reading_status WHERE status = 'WantToRead' ORDER BY book_id"
                )
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.into_iter().map(|row| row.get(0)).collect())
            }
            Category::CurrentlyReading => {
                let rows = sqlx::query(
                    "SELECT book_id FROM reading_status WHERE status = 'CurrentlyReading' ORDER BY book_id"
                )
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.into_iter().map(|row| row.get(0)).collect())
            }
            Category::Finished => {
                let rows = sqlx::query(
                    "SELECT book_id FROM reading_status WHERE status = 'Finished' ORDER BY finished_at DESC"
                )
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.into_iter().map(|row| row.get(0)).collect())
            }
            Category::Collection(collection_id) => {
                if let Some(collection) = self.get_collection(&collection_id).await? {
                    Ok(collection.book_ids)
                } else {
                    Ok(Vec::new())
                }
            }
            Category::Author(author_name) => {
                let rows = sqlx::query("SELECT id FROM books WHERE author = ? ORDER BY title")
                    .bind(&author_name)
                    .fetch_all(&self.pool)
                    .await?;
                Ok(rows.into_iter().map(|row| row.get(0)).collect())
            }
            Category::Tag(tag_name) => {
                let rows = sqlx::query(
                    r#"
                    SELECT b.id FROM books b
                    JOIN book_tags bt ON b.id = bt.book_id
                    JOIN tags t ON bt.tag_id = t.id
                    WHERE t.name = ?
                    ORDER BY b.title
                    "#
                )
                .bind(&tag_name)
                .fetch_all(&self.pool)
                .await?;
                Ok(rows.into_iter().map(|row| row.get(0)).collect())
            }
            Category::Genre(genre_name) => {
                let rows = sqlx::query("SELECT id FROM books WHERE genre = ? ORDER BY title")
                    .bind(&genre_name)
                    .fetch_all(&self.pool)
                    .await?;
                Ok(rows.into_iter().map(|row| row.get(0)).collect())
            }
            Category::Publisher(publisher_name) => {
                let rows = sqlx::query("SELECT id FROM books WHERE publisher = ? ORDER BY title")
                    .bind(&publisher_name)
                    .fetch_all(&self.pool)
                    .await?;
                Ok(rows.into_iter().map(|row| row.get(0)).collect())
            }
            Category::Year(year) => {
                let rows = sqlx::query("SELECT id FROM books WHERE strftime('%Y', publish_date) = ? ORDER BY title")
                    .bind(year.to_string())
                    .fetch_all(&self.pool)
                    .await?;
                Ok(rows.into_iter().map(|row| row.get(0)).collect())
            }
            Category::Language(language) => {
                let rows = sqlx::query("SELECT id FROM books WHERE language = ? ORDER BY title")
                    .bind(&language)
                    .fetch_all(&self.pool)
                    .await?;
                Ok(rows.into_iter().map(|row| row.get(0)).collect())
            }
            Category::Rating(rating) => {
                let rows = sqlx::query("SELECT id FROM books WHERE rating = ? ORDER BY title")
                    .bind(rating as f64)
                    .fetch_all(&self.pool)
                    .await?;
                Ok(rows.into_iter().map(|row| row.get(0)).collect())
            }
        }
    }

    async fn create_smart_collection(&self, name: String, rules: SmartCollectionRules) -> Result<Collection> {
        let collection = Collection::new_smart(name, "⚡".to_string(), "#007AFF".to_string(), rules);
        
        let smart_rules_json = serde_json::to_string(&collection.smart_rules)?;

        sqlx::query(
            r#"
            INSERT INTO collections (id, name, description, icon, color, created_at, updated_at, is_smart, smart_rules, is_favorite, sort_order)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(&collection.id)
        .bind(&collection.name)
        .bind(&collection.description)
        .bind(&collection.icon)
        .bind(&collection.color)
        .bind(collection.created_at.to_rfc3339())
        .bind(collection.updated_at.to_rfc3339())
        .bind(collection.is_smart)
        .bind(&smart_rules_json)
        .bind(collection.is_favorite)
        .bind(collection.sort_order as i64)
        .execute(&self.pool)
        .await?;

        Ok(collection)
    }

    async fn get_smart_collections(&self) -> Result<Vec<Collection>> {
        let rows = sqlx::query("SELECT * FROM collections WHERE is_smart = 1 ORDER BY sort_order, name")
            .fetch_all(&self.pool)
            .await?;

        let mut collections = Vec::new();
        for row in rows {
            let mut collection = self.row_to_collection(row)?;
            self.load_collection_books(&mut collection).await?;
            collections.push(collection);
        }

        Ok(collections)
    }

    async fn update_smart_collection_books(&self, collection_id: &str) -> Result<()> {
        // Smart collections don't store books directly - they're calculated on-demand
        Ok(())
    }

    async fn get_library_stats(&self) -> Result<LibraryStats> {
        Ok(LibraryService::get_library_stats(self).await?)
    }

    async fn update_reading_status(&self, book_id: &str, status: ReadingStatus) -> Result<()> {
        let now = Utc::now();
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO reading_status (book_id, status, started_at, finished_at, progress)
            VALUES (?, ?, 
                CASE WHEN ? = 'CurrentlyReading' THEN ? ELSE (SELECT started_at FROM reading_status WHERE book_id = ?) END,
                CASE WHEN ? = 'Finished' THEN ? ELSE NULL END,
                CASE WHEN ? = 'Finished' THEN 1.0 ELSE (SELECT COALESCE(progress, 0.0) FROM reading_status WHERE book_id = ?) END
            )
            "#
        )
        .bind(book_id)
        .bind(status.to_display_name())
        .bind(status.to_display_name())
        .bind(now.to_rfc3339())
        .bind(book_id)
        .bind(status.to_display_name())
        .bind(now.to_rfc3339())
        .bind(status.to_display_name())
        .bind(book_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    async fn get_reading_status(&self, book_id: &str) -> Result<Option<ReadingStatus>> {
        let status_str: Option<String> = sqlx::query_scalar(
            "SELECT status FROM reading_status WHERE book_id = ?"
        )
        .bind(book_id)
        .fetch_optional(&self.pool)
        .await?;

        if let Some(status_str) = status_str {
            let status = match status_str.as_str() {
                "WantToRead" => ReadingStatus::WantToRead,
                "CurrentlyReading" => ReadingStatus::CurrentlyReading,
                "Finished" => ReadingStatus::Finished,
                "DNF" => ReadingStatus::DNF,
                "Reference" => ReadingStatus::Reference,
                _ => ReadingStatus::WantToRead,
            };
            Ok(Some(status))
        } else {
            Ok(None)
        }
    }

    async fn get_all_authors(&self) -> Result<Vec<Author>> {
        let rows = sqlx::query("SELECT * FROM authors ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        let mut authors = Vec::new();
        for row in rows {
            let created_at_str: String = row.get("created_at");
            
            authors.push(Author {
                id: row.get("id"),
                name: row.get("name"),
                bio: row.get("bio"),
                birth_date: row.get::<Option<String>, _>("birth_date")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                death_date: row.get::<Option<String>, _>("death_date")
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                nationality: row.get("nationality"),
                photo_url: row.get("photo_url"),
                website: row.get("website"),
                book_count: 0, // TODO: Calculate from books
                average_rating: 0.0, // TODO: Calculate from books
                genres: Vec::new(), // TODO: Calculate from books
                aliases: Vec::new(), // TODO: Implement aliases
            });
        }

        Ok(authors)
    }

    async fn get_all_genres(&self) -> Result<Vec<Genre>> {
        let rows = sqlx::query("SELECT * FROM genres ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        let mut genres = Vec::new();
        for row in rows {
            let created_at_str: String = row.get("created_at");
            
            genres.push(Genre {
                id: row.get("id"),
                name: row.get("name"),
                description: row.get("description"),
                parent_genre: row.get("parent_genre"),
                color: row.get("color"),
                icon: row.get("icon"),
                book_count: 0, // TODO: Calculate from books
                average_rating: 0.0, // TODO: Calculate from books
            });
        }

        Ok(genres)
    }

    async fn get_all_tags(&self) -> Result<Vec<Tag>> {
        let rows = sqlx::query("SELECT * FROM tags ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        let mut tags = Vec::new();
        for row in rows {
            let created_at_str: String = row.get("created_at");
            
            tags.push(Tag {
                id: row.get("id"),
                name: row.get("name"),
                color: row.get("color"),
                description: row.get("description"),
                book_count: 0, // TODO: Calculate from books
                created_at: DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
            });
        }

        Ok(tags)
    }

    async fn filter_books(&self, filter: &LibraryFilter) -> Result<Vec<String>> {
        let mut query = String::from("SELECT DISTINCT b.id FROM books b");
        let mut joins = Vec::new();
        let mut conditions = Vec::new();
        let mut params: Vec<Box<dyn sqlx::Encode<'_, sqlx::Sqlite> + Send + Sync>> = Vec::new();

        // Add joins based on filter criteria
        if !filter.tags.is_empty() {
            joins.push("JOIN book_tags bt ON b.id = bt.book_id");
            joins.push("JOIN tags t ON bt.tag_id = t.id");
        }

        if filter.reading_status.is_some() {
            joins.push("LEFT JOIN reading_status rs ON b.id = rs.book_id");
        }

        // Add conditions
        if let Some(ref author) = filter.author {
            conditions.push("b.author = ?");
            params.push(Box::new(author.clone()));
        }

        if let Some(ref genre) = filter.genre {
            conditions.push("b.genre = ?");
            params.push(Box::new(genre.clone()));
        }

        if let Some(ref language) = filter.language {
            conditions.push("b.language = ?");
            params.push(Box::new(language.clone()));
        }

        if let Some(ref publisher) = filter.publisher {
            conditions.push("b.publisher = ?");
            params.push(Box::new(publisher.clone()));
        }

        if let Some((min_year, max_year)) = filter.year_range {
            conditions.push("strftime('%Y', b.publish_date) BETWEEN ? AND ?");
            params.push(Box::new(min_year.to_string()));
            params.push(Box::new(max_year.to_string()));
        }

        if let Some((min_rating, max_rating)) = filter.rating_range {
            conditions.push("b.rating BETWEEN ? AND ?");
            params.push(Box::new(min_rating as f64));
            params.push(Box::new(max_rating as f64));
        }

        if let Some(ref status) = filter.reading_status {
            conditions.push("rs.status = ?");
            params.push(Box::new(status.to_display_name()));
        }

        if !filter.tags.is_empty() {
            // For now, just use a simple approach - can be improved later
            conditions.push("t.name IS NOT NULL");
            for tag in &filter.tags {
                params.push(Box::new(tag.clone()));
            }
        }

        if let Some(ref search_query) = filter.search_query {
            conditions.push("(b.title LIKE ? OR b.author LIKE ? OR b.description LIKE ?)");
            let search_pattern = format!("%{}%", search_query);
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern.clone()));
            params.push(Box::new(search_pattern));
        }

        if let Some(has_cover) = filter.has_cover {
            if has_cover {
                conditions.push("b.cover_image IS NOT NULL AND b.cover_image != ''");
            } else {
                conditions.push("(b.cover_image IS NULL OR b.cover_image = '')");
            }
        }

        if let Some(ref file_format) = filter.file_format {
            conditions.push("b.file_path LIKE ?");
            params.push(Box::new(format!("%.{}", file_format)));
        }

        // Build final query
        if !joins.is_empty() {
            query.push_str(" ");
            query.push_str(&joins.join(" "));
        }

        if !conditions.is_empty() {
            query.push_str(" WHERE ");
            query.push_str(&conditions.join(" AND "));
        }

        query.push_str(" ORDER BY b.title");

        // Execute query
        // For now, use a simple query without dynamic binding
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|row| row.get(0)).collect())
    }

    async fn sort_books(&self, book_ids: &[String], sort_by: LibrarySortBy, direction: SortDirection) -> Result<Vec<String>> {
        if book_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders = book_ids.iter().map(|_| "?").collect::<Vec<_>>().join(", ");
        let order_direction = match direction {
            SortDirection::Ascending => "ASC",
            SortDirection::Descending => "DESC",
        };

        let (order_field, additional_joins) = match sort_by {
            LibrarySortBy::Title => ("b.title", ""),
            LibrarySortBy::Author => ("b.author", ""),
            LibrarySortBy::AddedDate => ("b.added_date", ""),
            LibrarySortBy::Rating => ("b.rating", ""),
            LibrarySortBy::PublishDate => ("b.publish_date", ""),
            LibrarySortBy::FileSize => ("b.file_size", ""),
            LibrarySortBy::PageCount => ("b.page_count", ""),
            LibrarySortBy::Genre => ("b.genre", ""),
            LibrarySortBy::Publisher => ("b.publisher", ""),
            LibrarySortBy::Language => ("b.language", ""),
            LibrarySortBy::ReadDate => ("rs.finished_at", "LEFT JOIN reading_status rs ON b.id = rs.book_id"),
            LibrarySortBy::Progress => ("rs.progress", "LEFT JOIN reading_status rs ON b.id = rs.book_id"),
            LibrarySortBy::LastOpened => ("b.last_opened", ""),
        };

        let query = format!(
            "SELECT b.id FROM books b {} WHERE b.id IN ({}) ORDER BY {} {}",
            additional_joins, placeholders, order_field, order_direction
        );

        // For now, use a simple query without dynamic binding
        let rows = sqlx::query(&query)
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(|row| row.get(0)).collect())
    }
}