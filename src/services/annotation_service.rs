use std::collections::HashMap;
use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool, sqlite::SqliteRow};
use uuid::Uuid;

use crate::models::annotation::{
    Annotation, Bookmark, AnnotationType, HighlightColor, BookmarkColor,
    TextPosition, AnnotationFilter, AnnotationStats, ExportOptions,
    ExportFormat, AnnotationSortBy, ReadingPatterns, TextFormatting,
};

#[derive(Clone)]
pub struct AnnotationService {
    pool: SqlitePool,
}

impl AnnotationService {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    /// Initialize annotation tables
    pub async fn init_tables(&self) -> Result<()> {
        // Create annotations table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS annotations (
                id TEXT PRIMARY KEY,
                book_id TEXT NOT NULL,
                page_number INTEGER NOT NULL,
                selected_text TEXT NOT NULL,
                note TEXT,
                color TEXT NOT NULL,
                created_at TEXT NOT NULL,
                modified_at TEXT NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                paragraph_index INTEGER NOT NULL,
                chapter_id TEXT,
                line_number INTEGER,
                column_number INTEGER,
                tags TEXT, -- JSON array
                category TEXT,
                annotation_type TEXT NOT NULL,
                formatting TEXT, -- JSON object
                is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
                cross_references TEXT, -- JSON array
                FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create bookmarks table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS bookmarks (
                id TEXT PRIMARY KEY,
                book_id TEXT NOT NULL,
                page_number INTEGER NOT NULL,
                title TEXT,
                description TEXT,
                preview_text TEXT NOT NULL,
                created_at TEXT NOT NULL,
                start_offset INTEGER NOT NULL,
                end_offset INTEGER NOT NULL,
                paragraph_index INTEGER NOT NULL,
                chapter_id TEXT,
                line_number INTEGER,
                column_number INTEGER,
                color TEXT NOT NULL,
                is_favorite BOOLEAN NOT NULL DEFAULT FALSE,
                FOREIGN KEY (book_id) REFERENCES books (id) ON DELETE CASCADE
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create annotation categories table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS annotation_categories (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT NOT NULL,
                icon TEXT,
                description TEXT,
                created_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create annotation tags table
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS annotation_tags (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                color TEXT,
                usage_count INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );
            "#,
        )
        .execute(&self.pool)
        .await?;

        // Create indexes for better performance
        sqlx::query("CREATE INDEX IF NOT EXISTS idx_annotations_book_id ON annotations(book_id);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_annotations_page_number ON annotations(page_number);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_annotations_created_at ON annotations(created_at);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_bookmarks_book_id ON bookmarks(book_id);")
            .execute(&self.pool)
            .await?;

        sqlx::query("CREATE INDEX IF NOT EXISTS idx_bookmarks_page_number ON bookmarks(page_number);")
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Save annotation to database
    pub async fn save_annotation(&self, annotation: &Annotation) -> Result<()> {
        let tags_json = serde_json::to_string(&annotation.tags)?;
        let cross_references_json = serde_json::to_string(&annotation.cross_references)?;
        let formatting_json = annotation.formatting.as_ref()
            .map(|f| serde_json::to_string(f))
            .transpose()?;

        sqlx::query(
            r#"
            INSERT OR REPLACE INTO annotations (
                id, book_id, page_number, selected_text, note, color,
                created_at, modified_at, start_offset, end_offset,
                paragraph_index, chapter_id, line_number, column_number,
                tags, category, annotation_type, formatting, is_favorite,
                cross_references
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&annotation.id)
        .bind(&annotation.book_id)
        .bind(annotation.page_number as i64)
        .bind(&annotation.selected_text)
        .bind(&annotation.note)
        .bind(&annotation.color.to_name())
        .bind(annotation.created_at.to_rfc3339())
        .bind(annotation.modified_at.to_rfc3339())
        .bind(annotation.position.start_offset as i64)
        .bind(annotation.position.end_offset as i64)
        .bind(annotation.position.paragraph_index as i64)
        .bind(&annotation.position.chapter_id)
        .bind(annotation.position.line_number.map(|n| n as i64))
        .bind(annotation.position.column_number.map(|n| n as i64))
        .bind(&tags_json)
        .bind(&annotation.category)
        .bind(&annotation.annotation_type.to_display_name())
        .bind(&formatting_json)
        .bind(annotation.is_favorite)
        .bind(&cross_references_json)
        .execute(&self.pool)
        .await?;

        // Update tag usage counts
        for tag in &annotation.tags {
            self.update_tag_usage(tag).await?;
        }

        Ok(())
    }

    /// Get annotation by ID
    pub async fn get_annotation(&self, id: &str) -> Result<Option<Annotation>> {
        let row = sqlx::query("SELECT * FROM annotations WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        match row {
            Some(row) => Ok(Some(self.row_to_annotation(row)?)),
            None => Ok(None),
        }
    }

    /// Get all annotations for a book
    pub async fn get_annotations_for_book(&self, book_id: &str) -> Result<Vec<Annotation>> {
        let rows = sqlx::query("SELECT * FROM annotations WHERE book_id = ? ORDER BY page_number, start_offset")
            .bind(book_id)
            .fetch_all(&self.pool)
            .await?;

        let mut annotations = Vec::new();
        for row in rows {
            annotations.push(self.row_to_annotation(row)?);
        }

        Ok(annotations)
    }

    /// Get annotations with filter
    pub async fn get_annotations_filtered(&self, filter: &AnnotationFilter) -> Result<Vec<Annotation>> {
        // For now, implement a simplified version without dynamic parameters
        // In a real implementation, you would use SQLx query builder or separate queries
        let rows = sqlx::query("SELECT * FROM annotations ORDER BY created_at DESC")
            .fetch_all(&self.pool)
            .await?;
        
        // TODO: Apply filtering in Rust code for now
        // In production, this should be done in SQL for better performance

        let mut annotations = Vec::new();
        for row in rows {
            let annotation = self.row_to_annotation(row)?;
            
            // Apply tag filtering (can't be done in SQL easily)
            if !filter.tags.is_empty() {
                if !filter.tags.iter().any(|tag| annotation.tags.contains(tag)) {
                    continue;
                }
            }

            annotations.push(annotation);
        }

        Ok(annotations)
    }

    /// Update annotation
    pub async fn update_annotation(&self, annotation: &Annotation) -> Result<()> {
        self.save_annotation(annotation).await
    }

    /// Delete annotation
    pub async fn delete_annotation(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM annotations WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Save bookmark to database
    pub async fn save_bookmark(&self, bookmark: &Bookmark) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO bookmarks (
                id, book_id, page_number, title, description, preview_text,
                created_at, start_offset, end_offset, paragraph_index,
                chapter_id, line_number, column_number, color, is_favorite
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&bookmark.id)
        .bind(&bookmark.book_id)
        .bind(bookmark.page_number as i64)
        .bind(&bookmark.title)
        .bind(&bookmark.description)
        .bind(&bookmark.preview_text)
        .bind(bookmark.created_at.to_rfc3339())
        .bind(bookmark.position.start_offset as i64)
        .bind(bookmark.position.end_offset as i64)
        .bind(bookmark.position.paragraph_index as i64)
        .bind(&bookmark.position.chapter_id)
        .bind(bookmark.position.line_number.map(|n| n as i64))
        .bind(bookmark.position.column_number.map(|n| n as i64))
        .bind(&bookmark.color.to_name())
        .bind(bookmark.is_favorite)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get all bookmarks for a book
    pub async fn get_bookmarks_for_book(&self, book_id: &str) -> Result<Vec<Bookmark>> {
        let rows = sqlx::query("SELECT * FROM bookmarks WHERE book_id = ? ORDER BY page_number")
            .bind(book_id)
            .fetch_all(&self.pool)
            .await?;

        let mut bookmarks = Vec::new();
        for row in rows {
            bookmarks.push(self.row_to_bookmark(row)?);
        }

        Ok(bookmarks)
    }

    /// Delete bookmark
    pub async fn delete_bookmark(&self, id: &str) -> Result<()> {
        sqlx::query("DELETE FROM bookmarks WHERE id = ?")
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Get annotation statistics
    pub async fn get_annotation_stats(&self, book_id: Option<&str>) -> Result<AnnotationStats> {
        let book_filter = if let Some(book_id) = book_id {
            format!("WHERE book_id = '{}'", book_id)
        } else {
            String::new()
        };

        // Get total counts
        let total_annotations: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM annotations {}", book_filter
        ))
        .fetch_one(&self.pool)
        .await?;

        let highlights_count: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM annotations {} AND annotation_type = 'Highlight'", 
            if book_filter.is_empty() { "WHERE 1=1" } else { &book_filter }
        ))
        .fetch_one(&self.pool)
        .await?;

        let notes_count: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM annotations {} AND note IS NOT NULL AND note != ''", 
            if book_filter.is_empty() { "WHERE 1=1" } else { &book_filter }
        ))
        .fetch_one(&self.pool)
        .await?;

        let bookmarks_count: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM bookmarks {}", 
            book_filter.replace("annotations", "bookmarks")
        ))
        .fetch_one(&self.pool)
        .await?;

        let favorite_count: i64 = sqlx::query_scalar(&format!(
            "SELECT COUNT(*) FROM annotations {} AND is_favorite = 1", 
            if book_filter.is_empty() { "WHERE 1=1" } else { &book_filter }
        ))
        .fetch_one(&self.pool)
        .await?;

        // Get color distribution
        let color_rows = sqlx::query(&format!(
            "SELECT color, COUNT(*) as count FROM annotations {} GROUP BY color", 
            book_filter
        ))
        .fetch_all(&self.pool)
        .await?;

        let mut color_distribution = HashMap::new();
        for row in color_rows {
            let color_name: String = row.get(0);
            let count: i64 = row.get(1);
            
            let color = match color_name.as_str() {
                "Yellow" => HighlightColor::Yellow,
                "Green" => HighlightColor::Green,
                "Blue" => HighlightColor::Blue,
                "Pink" => HighlightColor::Pink,
                "Orange" => HighlightColor::Orange,
                "Purple" => HighlightColor::Purple,
                "Red" => HighlightColor::Red,
                "Gray" => HighlightColor::Gray,
                _ => HighlightColor::Custom(color_name),
            };
            
            color_distribution.insert(color, count as u32);
        }

        // Get daily activity (simplified)
        let daily_activity = HashMap::new(); // Would need proper date grouping

        // Get most used tags
        let tag_rows = sqlx::query("SELECT name, usage_count FROM annotation_tags ORDER BY usage_count DESC LIMIT 10")
            .fetch_all(&self.pool)
            .await?;

        let mut most_used_tags = Vec::new();
        for row in tag_rows {
            let name: String = row.get(0);
            let count: i64 = row.get(1);
            most_used_tags.push((name, count as u32));
        }

        Ok(AnnotationStats {
            total_annotations: total_annotations as u32,
            highlights_count: highlights_count as u32,
            notes_count: notes_count as u32,
            bookmarks_count: bookmarks_count as u32,
            favorite_count: favorite_count as u32,
            categories_count: 0, // Would need to count categories
            tags_count: most_used_tags.len() as u32,
            color_distribution,
            daily_activity,
            most_used_tags,
            reading_patterns: ReadingPatterns {
                most_annotated_books: Vec::new(),
                preferred_colors: Vec::new(),
                annotation_frequency: 0.0,
                average_note_length: 0.0,
                most_active_hours: Vec::new(),
                annotation_clusters: Vec::new(),
            },
        })
    }

    /// Export annotations
    pub async fn export_annotations(&self, book_id: &str, options: &ExportOptions) -> Result<String> {
        let annotations = self.get_annotations_for_book(book_id).await?;
        let bookmarks = if options.include_bookmarks {
            self.get_bookmarks_for_book(book_id).await?
        } else {
            Vec::new()
        };

        match options.format {
            ExportFormat::Json => {
                let export_data = serde_json::json!({
                    "annotations": annotations,
                    "bookmarks": bookmarks,
                    "exported_at": Utc::now().to_rfc3339(),
                    "options": options,
                });
                Ok(serde_json::to_string_pretty(&export_data)?)
            }
            ExportFormat::Csv => {
                let mut csv_data = String::new();
                csv_data.push_str("Type,Page,Text,Note,Color,Created\n");
                
                for annotation in annotations {
                    if !options.include_highlights && annotation.annotation_type == AnnotationType::Highlight {
                        continue;
                    }
                    if !options.include_notes && annotation.note.is_some() {
                        continue;
                    }
                    
                    csv_data.push_str(&format!(
                        "{},{},{},{},{},{}\n",
                        annotation.annotation_type.to_display_name(),
                        annotation.page_number,
                        annotation.selected_text.replace('"', "\"\""),
                        annotation.note.unwrap_or_default().replace('"', "\"\""),
                        annotation.color.to_name(),
                        annotation.created_at.format("%Y-%m-%d %H:%M:%S")
                    ));
                }
                
                Ok(csv_data)
            }
            ExportFormat::Markdown => {
                let mut md_data = String::new();
                md_data.push_str("# Annotations Export\n\n");
                
                for annotation in annotations {
                    md_data.push_str(&format!(
                        "## Page {} - {}\n\n",
                        annotation.page_number,
                        annotation.annotation_type.to_display_name()
                    ));
                    
                    md_data.push_str(&format!("> {}\n\n", annotation.selected_text));
                    
                    if let Some(note) = &annotation.note {
                        md_data.push_str(&format!("**Note:** {}\n\n", note));
                    }
                    
                    if options.include_timestamps {
                        md_data.push_str(&format!(
                            "*Created: {}*\n\n",
                            annotation.created_at.format("%Y-%m-%d %H:%M:%S")
                        ));
                    }
                    
                    md_data.push_str("---\n\n");
                }
                
                Ok(md_data)
            }
            _ => Err(anyhow::anyhow!("Export format not yet implemented")),
        }
    }

    /// Create new annotation
    pub async fn create_annotation(
        &self,
        book_id: String,
        page_number: u32,
        selected_text: String,
        position: TextPosition,
        annotation_type: AnnotationType,
        color: HighlightColor,
        note: Option<String>,
    ) -> Result<Annotation> {
        let mut annotation = Annotation::new(
            book_id,
            page_number,
            selected_text,
            position,
            annotation_type,
        );
        
        annotation.color = color;
        annotation.note = note;
        
        self.save_annotation(&annotation).await?;
        Ok(annotation)
    }

    /// Create new bookmark
    pub async fn create_bookmark(
        &self,
        book_id: String,
        page_number: u32,
        preview_text: String,
        position: TextPosition,
        title: Option<String>,
        color: BookmarkColor,
    ) -> Result<Bookmark> {
        let mut bookmark = Bookmark::new(book_id, page_number, preview_text, position);
        bookmark.title = title;
        bookmark.color = color;
        
        self.save_bookmark(&bookmark).await?;
        Ok(bookmark)
    }

    /// Update tag usage count
    async fn update_tag_usage(&self, tag: &str) -> Result<()> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO annotation_tags (id, name, usage_count, created_at)
            VALUES (
                COALESCE((SELECT id FROM annotation_tags WHERE name = ?), ?),
                ?,
                COALESCE((SELECT usage_count FROM annotation_tags WHERE name = ?), 0) + 1,
                COALESCE((SELECT created_at FROM annotation_tags WHERE name = ?), ?)
            )
            "#,
        )
        .bind(tag)
        .bind(Uuid::new_v4().to_string())
        .bind(tag)
        .bind(tag)
        .bind(tag)
        .bind(Utc::now().to_rfc3339())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Convert database row to Annotation
    fn row_to_annotation(&self, row: SqliteRow) -> Result<Annotation> {
        let tags_json: String = row.get("tags");
        let tags: Vec<String> = serde_json::from_str(&tags_json)?;
        
        let cross_references_json: String = row.get("cross_references");
        let cross_references: Vec<String> = serde_json::from_str(&cross_references_json)?;
        
        let formatting_json: Option<String> = row.get("formatting");
        let formatting: Option<TextFormatting> = formatting_json
            .map(|json| serde_json::from_str(&json))
            .transpose()?;

        let color_name: String = row.get("color");
        let color = match color_name.as_str() {
            "Yellow" => HighlightColor::Yellow,
            "Green" => HighlightColor::Green,
            "Blue" => HighlightColor::Blue,
            "Pink" => HighlightColor::Pink,
            "Orange" => HighlightColor::Orange,
            "Purple" => HighlightColor::Purple,
            "Red" => HighlightColor::Red,
            "Gray" => HighlightColor::Gray,
            _ => HighlightColor::Custom(color_name),
        };

        let annotation_type_name: String = row.get("annotation_type");
        let annotation_type = match annotation_type_name.as_str() {
            "Highlight" => AnnotationType::Highlight,
            "Note" => AnnotationType::Note,
            "Bookmark" => AnnotationType::Bookmark,
            "Underline" => AnnotationType::Underline,
            "Strikethrough" => AnnotationType::Strikethrough,
            "Question" => AnnotationType::Question,
            "Important" => AnnotationType::Important,
            "Reference" => AnnotationType::Reference,
            _ => AnnotationType::Highlight,
        };

        let created_at_str: String = row.get("created_at");
        let modified_at_str: String = row.get("modified_at");

        Ok(Annotation {
            id: row.get("id"),
            book_id: row.get("book_id"),
            page_number: row.get::<i64, _>("page_number") as u32,
            selected_text: row.get("selected_text"),
            note: row.get("note"),
            color,
            created_at: DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
            modified_at: DateTime::parse_from_rfc3339(&modified_at_str)?.with_timezone(&Utc),
            position: TextPosition {
                start_offset: row.get::<i64, _>("start_offset") as usize,
                end_offset: row.get::<i64, _>("end_offset") as usize,
                paragraph_index: row.get::<i64, _>("paragraph_index") as usize,
                chapter_id: row.get("chapter_id"),
                line_number: row.get::<Option<i64>, _>("line_number").map(|n| n as u32),
                column_number: row.get::<Option<i64>, _>("column_number").map(|n| n as u32),
            },
            tags,
            category: row.get("category"),
            annotation_type,
            formatting,
            is_favorite: row.get("is_favorite"),
            cross_references,
        })
    }

    /// Convert database row to Bookmark
    fn row_to_bookmark(&self, row: SqliteRow) -> Result<Bookmark> {
        let color_name: String = row.get("color");
        let color = match color_name.as_str() {
            "Red" => BookmarkColor::Red,
            "Blue" => BookmarkColor::Blue,
            "Green" => BookmarkColor::Green,
            "Yellow" => BookmarkColor::Yellow,
            "Purple" => BookmarkColor::Purple,
            "Orange" => BookmarkColor::Orange,
            "Gray" => BookmarkColor::Gray,
            _ => BookmarkColor::Custom(color_name),
        };

        let created_at_str: String = row.get("created_at");

        Ok(Bookmark {
            id: row.get("id"),
            book_id: row.get("book_id"),
            page_number: row.get::<i64, _>("page_number") as u32,
            title: row.get("title"),
            description: row.get("description"),
            preview_text: row.get("preview_text"),
            created_at: DateTime::parse_from_rfc3339(&created_at_str)?.with_timezone(&Utc),
            position: TextPosition {
                start_offset: row.get::<i64, _>("start_offset") as usize,
                end_offset: row.get::<i64, _>("end_offset") as usize,
                paragraph_index: row.get::<i64, _>("paragraph_index") as usize,
                chapter_id: row.get("chapter_id"),
                line_number: row.get::<Option<i64>, _>("line_number").map(|n| n as u32),
                column_number: row.get::<Option<i64>, _>("column_number").map(|n| n as u32),
            },
            color,
            is_favorite: row.get("is_favorite"),
        })
    }
}