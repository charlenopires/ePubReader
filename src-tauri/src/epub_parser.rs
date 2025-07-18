use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use tracing::{info, error, warn, debug};

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
    info!("Starting ePub parsing for file: {}", path);
    let mut doc = EpubDoc::new(path).map_err(|e| {
        error!("Failed to open ePub file '{}': {}", path, e);
        anyhow!("Failed to open ePub: {}", e)
    })?;
    
    // Extract metadata
    let title = doc.mdata("title").unwrap_or_else(|| {
        warn!("No title found in ePub metadata, using default");
        "Unknown Title".to_string()
    });
    let author = doc.mdata("creator").unwrap_or_else(|| {
        warn!("No author found in ePub metadata, using default");
        "Unknown Author".to_string()
    });
    let language = doc.mdata("language").unwrap_or_else(|| {
        warn!("No language found in ePub metadata, defaulting to 'en'");
        "en".to_string()
    });
    
    debug!("Extracted metadata - Title: '{}', Author: '{}', Language: '{}'", title, author, language);
    
    let mut metadata = HashMap::new();
    metadata.insert("title".to_string(), title.clone());
    metadata.insert("author".to_string(), author.clone());
    metadata.insert("language".to_string(), language.clone());
    
    // Extract cover image
    debug!("Extracting cover image from ePub");
    let cover_image = extract_cover_image(&mut doc);
    if cover_image.is_some() {
        info!("Successfully extracted cover image");
    } else {
        warn!("No cover image found in ePub");
    }
    
    // Extract chapters
    let mut chapters = Vec::new();
    let spine = doc.spine.clone();
    
    for (order, spine_item) in spine.iter().enumerate() {
        debug!("Processing spine item {}: linear={}", order, spine_item.linear);
        
        let item_identifier = if let Some(ref item_id) = spine_item.id {
            debug!("Spine item {} has ID: {}", order, item_id);
            item_id.clone()
        } else {
            debug!("Spine item {} has no ID, using order-based identifier", order);
            format!("item_{}", order)
        };
        
        // First, try to get content by spine item's href/path directly
        let manifest_path = &spine_item.idref;
        debug!("Trying to get resource by manifest ID: {}", manifest_path);
        
        let content_result = doc.get_resource_str(manifest_path);
        
        if let Some((content, _)) = content_result {
            // Simple HTML content extraction (remove tags)
            let text_content = extract_text_from_html(&content);
            debug!("Extracted {} characters from chapter {}", text_content.len(), order + 1);
            
            if text_content.trim().len() > 50 { // Only include chapters with substantial content
                let chapter = Chapter {
                    id: item_identifier,
                    title: format!("Chapter {}", order + 1),
                    content: text_content,
                    order,
                };
                chapters.push(chapter);
            } else {
                debug!("Skipping chapter {} - insufficient content ({} chars)", order + 1, text_content.len());
            }
        } else {
            warn!("Failed to extract content from spine item {}", order);
        }
    }
    
    info!("Successfully parsed ePub '{}' with {} chapters", title, chapters.len());
    
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
    // Preserve HTML structure for translation while extracting meaningful content
    // This function now preserves images and code blocks
    let mut result = String::new();
    let mut in_tag = false;
    let mut current_tag = String::new();
    let mut tag_content = String::new();
    
    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        
        match ch {
            '<' => {
                in_tag = true;
                current_tag.clear();
                tag_content.clear();
            },
            '>' => {
                in_tag = false;
                
                // Check if this is an image tag - preserve it
                if current_tag.starts_with("img ") || current_tag == "img" {
                    result.push('<');
                    result.push_str(&current_tag);
                    result.push('>');
                }
                // Check for code tags - preserve content but mark for no translation
                else if current_tag.starts_with("code") || current_tag.starts_with("pre") {
                    // We'll handle code preservation in the translation phase
                    result.push('<');
                    result.push_str(&current_tag);
                    result.push('>');
                }
                
                current_tag.clear();
            },
            _ if in_tag => {
                current_tag.push(ch);
            },
            _ => {
                result.push(ch);
            }
        }
        i += 1;
    }
    
    // Clean up excessive whitespace but preserve structure
    let lines: Vec<&str> = result.lines().collect();
    let cleaned_lines: Vec<String> = lines.iter()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_string())
        .collect();
    
    cleaned_lines.join("\n")
}

fn extract_cover_image(doc: &mut EpubDoc<std::io::BufReader<std::fs::File>>) -> Option<String> {
    // Try to get cover image
    debug!("Attempting to extract cover image from ePub");
    if let Some((cover_data, _)) = doc.get_cover() {
        debug!("Found cover image, converting to base64 ({} bytes)", cover_data.len());
        // Convert to base64
        let base64_image = general_purpose::STANDARD.encode(&cover_data);
        return Some(format!("data:image/jpeg;base64,{}", base64_image));
    }
    
    // Fallback: try to find any image in the manifest
    debug!("No cover found, searching for any image in manifest");
    let resources = doc.resources.clone();
    for (_, (path, mime_type)) in resources.iter() {
        if mime_type.starts_with("image/") {
            debug!("Found image resource: {} ({})", path.display(), mime_type);
            if let Some((img_data, _)) = doc.get_resource(&path.to_string_lossy()) {
                debug!("Successfully extracted image ({} bytes)", img_data.len());
                let base64_image = general_purpose::STANDARD.encode(&img_data);
                return Some(format!("data:{};base64,{}", mime_type, base64_image));
            } else {
                warn!("Failed to extract image resource: {}", path.display());
            }
        }
    }
    
    debug!("No cover image found in ePub");
    None
}