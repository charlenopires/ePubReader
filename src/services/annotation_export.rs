use std::collections::HashMap;
use anyhow::Result;
use chrono::{DateTime, Utc};
use serde_json::Value;

use crate::models::annotation::{
    Annotation, Bookmark, ExportFormat, ExportOptions, AnnotationSortBy
};

pub struct AnnotationExporter;

impl AnnotationExporter {
    /// Export annotations to specified format
    pub fn export_annotations(
        annotations: &[Annotation],
        bookmarks: &[Bookmark],
        options: &ExportOptions,
        book_title: Option<&str>,
    ) -> Result<String> {
        // Filter annotations based on options
        let filtered_annotations = Self::filter_annotations(annotations, options);
        let filtered_bookmarks = if options.include_bookmarks {
            bookmarks.to_vec()
        } else {
            Vec::new()
        };

        // Sort annotations
        let sorted_annotations = Self::sort_annotations(filtered_annotations, &options.sort_by);

        match options.format {
            ExportFormat::Json => Self::export_as_json(&sorted_annotations, &filtered_bookmarks, options, book_title),
            ExportFormat::Csv => Self::export_as_csv(&sorted_annotations, &filtered_bookmarks, options),
            ExportFormat::Markdown => Self::export_as_markdown(&sorted_annotations, &filtered_bookmarks, options, book_title),
            ExportFormat::Html => Self::export_as_html(&sorted_annotations, &filtered_bookmarks, options, book_title),
            ExportFormat::Txt => Self::export_as_txt(&sorted_annotations, &filtered_bookmarks, options, book_title),
            ExportFormat::Pdf => Err(anyhow::anyhow!("PDF export not yet implemented")),
        }
    }

    /// Filter annotations based on export options
    fn filter_annotations(annotations: &[Annotation], options: &ExportOptions) -> Vec<Annotation> {
        annotations
            .iter()
            .filter(|annotation| {
                // Filter by type
                match annotation.annotation_type {
                    crate::models::annotation::AnnotationType::Highlight => options.include_highlights,
                    crate::models::annotation::AnnotationType::Note => options.include_notes,
                    _ => true,
                }
            })
            .cloned()
            .collect()
    }

    /// Sort annotations based on sort criteria
    fn sort_annotations(mut annotations: Vec<Annotation>, sort_by: &AnnotationSortBy) -> Vec<Annotation> {
        match sort_by {
            AnnotationSortBy::CreatedAt => {
                annotations.sort_by(|a, b| a.created_at.cmp(&b.created_at));
            }
            AnnotationSortBy::PageNumber => {
                annotations.sort_by(|a, b| a.page_number.cmp(&b.page_number));
            }
            AnnotationSortBy::Type => {
                annotations.sort_by(|a, b| {
                    a.annotation_type.to_display_name().cmp(&b.annotation_type.to_display_name())
                });
            }
            AnnotationSortBy::Color => {
                annotations.sort_by(|a, b| a.color.to_name().cmp(&b.color.to_name()));
            }
            AnnotationSortBy::Category => {
                annotations.sort_by(|a, b| a.category.cmp(&b.category));
            }
            AnnotationSortBy::Tag => {
                annotations.sort_by(|a, b| {
                    let a_tags = a.tags.join(", ");
                    let b_tags = b.tags.join(", ");
                    a_tags.cmp(&b_tags)
                });
            }
        }
        annotations
    }

    /// Export as JSON
    fn export_as_json(
        annotations: &[Annotation],
        bookmarks: &[Bookmark],
        options: &ExportOptions,
        book_title: Option<&str>,
    ) -> Result<String> {
        let mut export_data = serde_json::Map::new();
        
        // Metadata
        export_data.insert("book_title".to_string(), Value::String(book_title.unwrap_or("Unknown").to_string()));
        export_data.insert("exported_at".to_string(), Value::String(Utc::now().to_rfc3339()));
        export_data.insert("total_annotations".to_string(), Value::Number(annotations.len().into()));
        export_data.insert("total_bookmarks".to_string(), Value::Number(bookmarks.len().into()));
        
        // Export options
        export_data.insert("export_options".to_string(), serde_json::to_value(options)?);
        
        // Annotations
        if options.include_highlights || options.include_notes {
            let annotation_data: Vec<serde_json::Value> = annotations
                .iter()
                .map(|a| {
                    let mut annotation_obj = serde_json::Map::new();
                    annotation_obj.insert("id".to_string(), Value::String(a.id.clone()));
                    annotation_obj.insert("type".to_string(), Value::String(a.annotation_type.to_display_name()));
                    annotation_obj.insert("page_number".to_string(), Value::Number(a.page_number.into()));
                    annotation_obj.insert("selected_text".to_string(), Value::String(a.selected_text.clone()));
                    
                    if let Some(note) = &a.note {
                        annotation_obj.insert("note".to_string(), Value::String(note.clone()));
                    }
                    
                    annotation_obj.insert("color".to_string(), Value::String(a.color.to_hex()));
                    annotation_obj.insert("tags".to_string(), Value::Array(a.tags.iter().map(|t| Value::String(t.clone())).collect()));
                    
                    if options.include_timestamps {
                        annotation_obj.insert("created_at".to_string(), Value::String(a.created_at.to_rfc3339()));
                        annotation_obj.insert("modified_at".to_string(), Value::String(a.modified_at.to_rfc3339()));
                    }
                    
                    annotation_obj.insert("is_favorite".to_string(), Value::Bool(a.is_favorite));
                    
                    if let Some(category) = &a.category {
                        annotation_obj.insert("category".to_string(), Value::String(category.clone()));
                    }
                    
                    Value::Object(annotation_obj)
                })
                .collect();
            
            export_data.insert("annotations".to_string(), Value::Array(annotation_data));
        }
        
        // Bookmarks
        if options.include_bookmarks {
            let bookmark_data: Vec<serde_json::Value> = bookmarks
                .iter()
                .map(|b| {
                    let mut bookmark_obj = serde_json::Map::new();
                    bookmark_obj.insert("id".to_string(), Value::String(b.id.clone()));
                    bookmark_obj.insert("page_number".to_string(), Value::Number(b.page_number.into()));
                    bookmark_obj.insert("title".to_string(), Value::String(b.get_display_title()));
                    bookmark_obj.insert("preview_text".to_string(), Value::String(b.preview_text.clone()));
                    bookmark_obj.insert("color".to_string(), Value::String(b.color.to_hex()));
                    bookmark_obj.insert("is_favorite".to_string(), Value::Bool(b.is_favorite));
                    
                    if options.include_timestamps {
                        bookmark_obj.insert("created_at".to_string(), Value::String(b.created_at.to_rfc3339()));
                    }
                    
                    if let Some(description) = &b.description {
                        bookmark_obj.insert("description".to_string(), Value::String(description.clone()));
                    }
                    
                    Value::Object(bookmark_obj)
                })
                .collect();
            
            export_data.insert("bookmarks".to_string(), Value::Array(bookmark_data));
        }
        
        Ok(serde_json::to_string_pretty(&export_data)?)
    }

    /// Export as CSV
    fn export_as_csv(
        annotations: &[Annotation],
        bookmarks: &[Bookmark],
        options: &ExportOptions,
    ) -> Result<String> {
        let mut csv_data = String::new();
        
        // Headers
        let mut headers = vec!["Type", "Page"];
        if options.include_page_numbers {
            headers.push("Page Number");
        }
        headers.extend_from_slice(&["Text", "Note", "Color"]);
        if options.include_timestamps {
            headers.push("Created At");
        }
        headers.push("Favorite");
        
        csv_data.push_str(&headers.join(","));
        csv_data.push('\n');
        
        // Annotations
        for annotation in annotations {
            let mut row = vec![
                annotation.annotation_type.to_display_name(),
                annotation.page_number.to_string(),
            ];
            
            if options.include_page_numbers {
                row.push(annotation.page_number.to_string());
            }
            
            row.push(Self::escape_csv_field(&annotation.selected_text));
            row.push(Self::escape_csv_field(&annotation.note.clone().unwrap_or_default()));
            row.push(annotation.color.to_name());
            
            if options.include_timestamps {
                row.push(annotation.created_at.format("%Y-%m-%d %H:%M:%S").to_string());
            }
            
            row.push(annotation.is_favorite.to_string());
            
            csv_data.push_str(&row.join(","));
            csv_data.push('\n');
        }
        
        // Bookmarks
        if options.include_bookmarks {
            for bookmark in bookmarks {
                let mut row = vec![
                    "Bookmark".to_string(),
                    bookmark.page_number.to_string(),
                ];
                
                if options.include_page_numbers {
                    row.push(bookmark.page_number.to_string());
                }
                
                row.push(Self::escape_csv_field(&bookmark.preview_text));
                row.push(Self::escape_csv_field(&bookmark.description.clone().unwrap_or_default()));
                row.push(bookmark.color.to_name());
                
                if options.include_timestamps {
                    row.push(bookmark.created_at.format("%Y-%m-%d %H:%M:%S").to_string());
                }
                
                row.push(bookmark.is_favorite.to_string());
                
                csv_data.push_str(&row.join(","));
                csv_data.push('\n');
            }
        }
        
        Ok(csv_data)
    }

    /// Export as Markdown
    fn export_as_markdown(
        annotations: &[Annotation],
        bookmarks: &[Bookmark],
        options: &ExportOptions,
        book_title: Option<&str>,
    ) -> Result<String> {
        let mut md_data = String::new();
        
        // Title
        md_data.push_str(&format!("# Annotations - {}\n\n", book_title.unwrap_or("Unknown Book")));
        
        // Statistics
        md_data.push_str("## Summary\n\n");
        md_data.push_str(&format!("- **Total Annotations**: {}\n", annotations.len()));
        md_data.push_str(&format!("- **Total Bookmarks**: {}\n", bookmarks.len()));
        
        if options.include_timestamps {
            md_data.push_str(&format!("- **Exported**: {}\n", Utc::now().format("%Y-%m-%d %H:%M:%S")));
        }
        
        md_data.push_str("\n---\n\n");
        
        // Group by chapter if requested
        if options.group_by_chapter {
            let mut chapters: HashMap<String, Vec<&Annotation>> = HashMap::new();
            
            for annotation in annotations {
                let chapter = annotation.position.chapter_id.clone().unwrap_or_else(|| "Unknown Chapter".to_string());
                chapters.entry(chapter).or_insert_with(Vec::new).push(annotation);
            }
            
            for (chapter, chapter_annotations) in chapters {
                md_data.push_str(&format!("## Chapter: {}\n\n", chapter));
                
                for annotation in chapter_annotations {
                    Self::write_annotation_markdown(&mut md_data, annotation, options);
                }
                
                md_data.push_str("\n---\n\n");
            }
        } else {
            // Regular format
            if !annotations.is_empty() {
                md_data.push_str("## Annotations\n\n");
                
                for annotation in annotations {
                    Self::write_annotation_markdown(&mut md_data, annotation, options);
                }
            }
        }
        
        // Bookmarks
        if options.include_bookmarks && !bookmarks.is_empty() {
            md_data.push_str("## Bookmarks\n\n");
            
            for bookmark in bookmarks {
                md_data.push_str(&format!("### {} - Page {}\n\n", bookmark.get_display_title(), bookmark.page_number));
                
                if !bookmark.preview_text.is_empty() {
                    md_data.push_str(&format!("> {}\n\n", bookmark.preview_text));
                }
                
                if let Some(description) = &bookmark.description {
                    md_data.push_str(&format!("**Description:** {}\n\n", description));
                }
                
                if options.include_timestamps {
                    md_data.push_str(&format!("*Created: {}*\n\n", bookmark.created_at.format("%Y-%m-%d %H:%M:%S")));
                }
                
                md_data.push_str("---\n\n");
            }
        }
        
        Ok(md_data)
    }

    /// Write annotation in Markdown format
    fn write_annotation_markdown(md_data: &mut String, annotation: &Annotation, options: &ExportOptions) {
        md_data.push_str(&format!("### {} - Page {}\n\n", annotation.annotation_type.to_display_name(), annotation.page_number));
        
        // Selected text
        md_data.push_str(&format!("> {}\n\n", annotation.selected_text));
        
        // Note
        if let Some(note) = &annotation.note {
            md_data.push_str(&format!("**Note:** {}\n\n", note));
        }
        
        // Tags
        if !annotation.tags.is_empty() {
            md_data.push_str(&format!("**Tags:** {}\n\n", annotation.tags.join(", ")));
        }
        
        // Color
        md_data.push_str(&format!("**Color:** {}\n\n", annotation.color.to_name()));
        
        // Timestamps
        if options.include_timestamps {
            md_data.push_str(&format!("*Created: {}*\n\n", annotation.created_at.format("%Y-%m-%d %H:%M:%S")));
        }
        
        md_data.push_str("---\n\n");
    }

    /// Export as HTML
    fn export_as_html(
        annotations: &[Annotation],
        bookmarks: &[Bookmark],
        options: &ExportOptions,
        book_title: Option<&str>,
    ) -> Result<String> {
        let mut html_data = String::new();
        
        // HTML Header
        html_data.push_str(&format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Annotations - {}</title>
    <style>
        body {{ font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; margin: 40px; line-height: 1.6; }}
        .annotation {{ border-left: 4px solid #007AFF; padding: 15px; margin: 20px 0; background: #f8f9fa; }}
        .bookmark {{ border-left: 4px solid #FF6B6B; padding: 15px; margin: 20px 0; background: #fff5f5; }}
        .page-number {{ color: #666; font-size: 0.9em; }}
        .note {{ font-style: italic; color: #333; margin-top: 10px; }}
        .tags {{ margin-top: 10px; }}
        .tag {{ background: #e3f2fd; padding: 3px 8px; border-radius: 12px; font-size: 0.8em; margin-right: 5px; }}
        .timestamp {{ color: #999; font-size: 0.8em; }}
        .summary {{ background: #e8f5e8; padding: 20px; border-radius: 8px; margin-bottom: 30px; }}
        h1 {{ color: #333; }}
        h2 {{ color: #555; }}
        blockquote {{ border-left: 3px solid #ddd; padding-left: 15px; margin: 15px 0; font-style: italic; }}
    </style>
</head>
<body>
    <h1>Annotations - {}</h1>
"#,
            book_title.unwrap_or("Unknown Book"),
            book_title.unwrap_or("Unknown Book")
        ));
        
        // Summary
        html_data.push_str(&format!(
            r#"<div class="summary">
        <h2>Summary</h2>
        <p><strong>Total Annotations:</strong> {}</p>
        <p><strong>Total Bookmarks:</strong> {}</p>
        <p><strong>Exported:</strong> {}</p>
    </div>"#,
            annotations.len(),
            bookmarks.len(),
            Utc::now().format("%Y-%m-%d %H:%M:%S")
        ));
        
        // Annotations
        if !annotations.is_empty() {
            html_data.push_str("<h2>Annotations</h2>");
            
            for annotation in annotations {
                html_data.push_str(&format!(
                    r#"<div class="annotation">
                <div class="page-number">Page {} - {}</div>
                <blockquote>{}</blockquote>"#,
                    annotation.page_number,
                    annotation.annotation_type.to_display_name(),
                    html_escape::encode_text(&annotation.selected_text)
                ));
                
                if let Some(note) = &annotation.note {
                    html_data.push_str(&format!(
                        r#"<div class="note">Note: {}</div>"#,
                        html_escape::encode_text(note)
                    ));
                }
                
                if !annotation.tags.is_empty() {
                    html_data.push_str(r#"<div class="tags">"#);
                    for tag in &annotation.tags {
                        html_data.push_str(&format!(
                            r#"<span class="tag">{}</span>"#,
                            html_escape::encode_text(tag)
                        ));
                    }
                    html_data.push_str("</div>");
                }
                
                if options.include_timestamps {
                    html_data.push_str(&format!(
                        r#"<div class="timestamp">Created: {}</div>"#,
                        annotation.created_at.format("%Y-%m-%d %H:%M:%S")
                    ));
                }
                
                html_data.push_str("</div>");
            }
        }
        
        // Bookmarks
        if options.include_bookmarks && !bookmarks.is_empty() {
            html_data.push_str("<h2>Bookmarks</h2>");
            
            for bookmark in bookmarks {
                html_data.push_str(&format!(
                    r#"<div class="bookmark">
                <div class="page-number">Page {} - {}</div>
                <blockquote>{}</blockquote>"#,
                    bookmark.page_number,
                    html_escape::encode_text(&bookmark.get_display_title()),
                    html_escape::encode_text(&bookmark.preview_text)
                ));
                
                if let Some(description) = &bookmark.description {
                    html_data.push_str(&format!(
                        r#"<div class="note">Description: {}</div>"#,
                        html_escape::encode_text(description)
                    ));
                }
                
                if options.include_timestamps {
                    html_data.push_str(&format!(
                        r#"<div class="timestamp">Created: {}</div>"#,
                        bookmark.created_at.format("%Y-%m-%d %H:%M:%S")
                    ));
                }
                
                html_data.push_str("</div>");
            }
        }
        
        // HTML Footer
        html_data.push_str(r#"</body></html>"#);
        
        Ok(html_data)
    }

    /// Export as plain text
    fn export_as_txt(
        annotations: &[Annotation],
        bookmarks: &[Bookmark],
        options: &ExportOptions,
        book_title: Option<&str>,
    ) -> Result<String> {
        let mut txt_data = String::new();
        
        // Title
        txt_data.push_str(&format!("ANNOTATIONS - {}\n", book_title.unwrap_or("UNKNOWN BOOK")));
        txt_data.push_str("=".repeat(50).as_str());
        txt_data.push_str("\n\n");
        
        // Summary
        txt_data.push_str("SUMMARY\n");
        txt_data.push_str("-".repeat(20).as_str());
        txt_data.push_str("\n");
        txt_data.push_str(&format!("Total Annotations: {}\n", annotations.len()));
        txt_data.push_str(&format!("Total Bookmarks: {}\n", bookmarks.len()));
        
        if options.include_timestamps {
            txt_data.push_str(&format!("Exported: {}\n", Utc::now().format("%Y-%m-%d %H:%M:%S")));
        }
        
        txt_data.push_str("\n");
        txt_data.push_str("=".repeat(50).as_str());
        txt_data.push_str("\n\n");
        
        // Annotations
        if !annotations.is_empty() {
            txt_data.push_str("ANNOTATIONS\n");
            txt_data.push_str("-".repeat(20).as_str());
            txt_data.push_str("\n\n");
            
            for (i, annotation) in annotations.iter().enumerate() {
                txt_data.push_str(&format!("{}. {} - Page {}\n", i + 1, annotation.annotation_type.to_display_name(), annotation.page_number));
                txt_data.push_str(&format!("   \"{}\"\n", annotation.selected_text));
                
                if let Some(note) = &annotation.note {
                    txt_data.push_str(&format!("   Note: {}\n", note));
                }
                
                if !annotation.tags.is_empty() {
                    txt_data.push_str(&format!("   Tags: {}\n", annotation.tags.join(", ")));
                }
                
                txt_data.push_str(&format!("   Color: {}\n", annotation.color.to_name()));
                
                if options.include_timestamps {
                    txt_data.push_str(&format!("   Created: {}\n", annotation.created_at.format("%Y-%m-%d %H:%M:%S")));
                }
                
                txt_data.push_str("\n");
            }
        }
        
        // Bookmarks
        if options.include_bookmarks && !bookmarks.is_empty() {
            txt_data.push_str("BOOKMARKS\n");
            txt_data.push_str("-".repeat(20).as_str());
            txt_data.push_str("\n\n");
            
            for (i, bookmark) in bookmarks.iter().enumerate() {
                txt_data.push_str(&format!("{}. {} - Page {}\n", i + 1, bookmark.get_display_title(), bookmark.page_number));
                txt_data.push_str(&format!("   \"{}\"\n", bookmark.preview_text));
                
                if let Some(description) = &bookmark.description {
                    txt_data.push_str(&format!("   Description: {}\n", description));
                }
                
                if options.include_timestamps {
                    txt_data.push_str(&format!("   Created: {}\n", bookmark.created_at.format("%Y-%m-%d %H:%M:%S")));
                }
                
                txt_data.push_str("\n");
            }
        }
        
        Ok(txt_data)
    }

    /// Escape CSV field
    fn escape_csv_field(field: &str) -> String {
        if field.contains(',') || field.contains('"') || field.contains('\n') {
            format!("\"{}\"", field.replace('"', "\"\""))
        } else {
            field.to_string()
        }
    }
}