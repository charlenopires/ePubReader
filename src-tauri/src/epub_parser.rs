use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubInfo {
    pub title: String,
    pub author: String,
    pub language: String,
    pub chapters: Vec<Chapter>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub content: String,
    pub order: usize,
}

pub async fn parse_epub(path: &str) -> Result<EpubInfo> {
    let mut doc = EpubDoc::new(path).map_err(|e| anyhow!("Failed to open ePub: {}", e))?;
    
    // Extract metadata
    let title = doc.mdata("title").unwrap_or_else(|| "Unknown Title".to_string());
    let author = doc.mdata("creator").unwrap_or_else(|| "Unknown Author".to_string());
    let language = doc.mdata("language").unwrap_or_else(|| "en".to_string());
    
    let mut metadata = HashMap::new();
    metadata.insert("title".to_string(), title.clone());
    metadata.insert("author".to_string(), author.clone());
    metadata.insert("language".to_string(), language.clone());
    
    // Extract chapters
    let mut chapters = Vec::new();
    let spine = doc.spine.clone();
    
    for (order, spine_item) in spine.iter().enumerate() {
        if let Some(ref item_id) = spine_item.id {
            if let Some((content, _)) = doc.get_resource_str(item_id) {
                // Simple HTML content extraction (remove tags)
                let text_content = extract_text_from_html(&content);
                
                let chapter = Chapter {
                    id: item_id.clone(),
                    title: format!("Chapter {}", order + 1),
                    content: text_content,
                    order,
                };
                chapters.push(chapter);
            }
        }
    }
    
    Ok(EpubInfo {
        title,
        author,
        language,
        chapters,
        metadata,
    })
}

fn extract_text_from_html(html: &str) -> String {
    // Simple HTML tag removal - in a real implementation, use a proper HTML parser
    let mut result = String::new();
    let mut in_tag = false;
    
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => result.push(ch),
            _ => {}
        }
    }
    
    // Clean up whitespace
    result
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
        .trim()
        .to_string()
}