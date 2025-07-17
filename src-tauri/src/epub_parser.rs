use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubInfo {
    pub title: String,
    pub author: String,
    pub language: String,
    pub chapters: Vec<Chapter>,
    pub metadata: HashMap<String, String>,
    pub cover_image: Option<String>, // Base64 encoded cover image
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
    
    // Extract cover image
    let cover_image = extract_cover_image(&mut doc);
    
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
        cover_image,
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

fn extract_cover_image(doc: &mut EpubDoc<std::io::BufReader<std::fs::File>>) -> Option<String> {
    // Try to get cover image
    if let Some((cover_data, _)) = doc.get_cover() {
        // Convert to base64
        let base64_image = general_purpose::STANDARD.encode(&cover_data);
        return Some(format!("data:image/jpeg;base64,{}", base64_image));
    }
    
    // Fallback: try to find any image in the manifest
    let resources = doc.resources.clone();
    for (_, (path, mime_type)) in resources.iter() {
        if mime_type.starts_with("image/") {
            if let Some((img_data, _)) = doc.get_resource(&path.to_string_lossy()) {
                let base64_image = general_purpose::STANDARD.encode(&img_data);
                return Some(format!("data:{};base64,{}", mime_type, base64_image));
            }
        }
    }
    
    None
}