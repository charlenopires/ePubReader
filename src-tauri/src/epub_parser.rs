use epub::doc::EpubDoc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::{Result, anyhow};
use base64::{Engine as _, engine::general_purpose};
use tracing::{info, error, warn, debug};
use regex::Regex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EpubInfo {
    pub title: String,
    pub author: String,
    pub language: String,
    pub chapters: Vec<Chapter>,
    pub metadata: HashMap<String, String>,
    pub cover_image: Option<String>, // Base64 encoded cover image
    pub images: HashMap<String, String>, // Image path -> Base64 data
    pub css_files: HashMap<String, String>, // CSS path -> content
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    pub content: String,
    pub raw_html: String, // Original HTML with image references
    pub order: usize,
    pub image_refs: Vec<String>, // List of image paths referenced in this chapter
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
    
    // Extract all images and CSS files from the ePub
    let mut images = HashMap::new();
    let mut css_files = HashMap::new();
    
    debug!("Extracting images and CSS files from ePub resources");
    let resources = doc.resources.clone();
    for (_, (path, mime_type)) in resources.iter() {
        let path_str = path.to_string_lossy().to_string();
        
        if mime_type.starts_with("image/") {
            debug!("Found image resource: {} ({})", path_str, mime_type);
            if let Some((img_data, _)) = doc.get_resource(&path_str) {
                let base64_image = general_purpose::STANDARD.encode(&img_data);
                let data_url = format!("data:{};base64,{}", mime_type, base64_image);
                images.insert(path_str.clone(), data_url);
                debug!("Extracted image: {} ({} bytes)", path_str, img_data.len());
            }
        } else if mime_type == "text/css" {
            debug!("Found CSS resource: {}", path_str);
            if let Some((css_data, _)) = doc.get_resource_str(&path_str) {
                let css_len = css_data.len();
                css_files.insert(path_str.clone(), css_data);
                debug!("Extracted CSS: {} ({} chars)", path_str, css_len);
            }
        }
    }
    
    info!("Extracted {} images and {} CSS files", images.len(), css_files.len());
    
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
        
        if let Some((raw_html, _)) = content_result {
            // Extract image references from this chapter
            let image_refs = extract_image_references(&raw_html);
            debug!("Found {} image references in chapter {}", image_refs.len(), order + 1);
            
            // Extract clean text content for translation
            let text_content = extract_text_from_html_advanced(&raw_html);
            debug!("Extracted {} characters from chapter {}", text_content.len(), order + 1);
            
            if text_content.trim().len() > 10 { // Lower threshold to capture more content
                let content_len = text_content.len();
                let image_refs_len = image_refs.len();
                let chapter = Chapter {
                    id: item_identifier,
                    title: extract_chapter_title(&raw_html, chapters.len() + 1),
                    content: text_content,
                    raw_html: raw_html.clone(),
                    order: chapters.len(), // Use the actual chapter index
                    image_refs,
                };
                chapters.push(chapter);
                debug!("Added chapter {} with {} characters and {} images", chapters.len(), content_len, image_refs_len);
            } else {
                debug!("Skipping spine item {} - insufficient content ({} chars)", order + 1, text_content.len());
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
        images,
        css_files,
    })
}

fn extract_image_references(html: &str) -> Vec<String> {
    let mut image_refs = Vec::new();
    
    // Regex to find img tags and extract src attributes
    if let Ok(img_regex) = Regex::new(r#"<img[^>]+src=["']([^"']+)["'][^>]*>"#) {
        for cap in img_regex.captures_iter(html) {
            if let Some(src) = cap.get(1) {
                let img_path = src.as_str().trim().to_string();
                if !img_path.is_empty() && !image_refs.contains(&img_path) {
                    image_refs.push(img_path);
                }
            }
        }
    }
    
    debug!("Extracted {} unique image references", image_refs.len());
    image_refs
}

fn extract_chapter_title(html: &str, default_number: usize) -> String {
    // Try to find title in h1, h2, or title tags
    let title_patterns = vec![
        r"<h1[^>]*>([^<]+)</h1>",
        r"<h2[^>]*>([^<]+)</h2>",
        r"<title[^>]*>([^<]+)</title>",
    ];
    
    for pattern in title_patterns {
        if let Ok(regex) = Regex::new(pattern) {
            if let Some(cap) = regex.captures(html) {
                if let Some(title_match) = cap.get(1) {
                    let title = title_match.as_str().trim();
                    if !title.is_empty() && title.len() < 100 {
                        return title.to_string();
                    }
                }
            }
        }
    }
    
    // Fallback to default chapter numbering
    format!("Chapter {}", default_number)
}

pub fn extract_text_from_html_advanced(html: &str) -> String {
    // More sophisticated HTML processing that preserves structure and content
    let mut result = String::new();
    let mut in_tag = false;
    let mut current_tag = String::new();
    let mut in_script = false;
    let mut in_style = false;
    let mut tag_depth: i32 = 0;
    
    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        
        match ch {
            '<' => {
                in_tag = true;
                current_tag.clear();
                tag_depth += 1;
            },
            '>' => {
                in_tag = false;
                tag_depth = tag_depth.saturating_sub(1);
                
                // Handle different tag types
                let tag_lower = current_tag.to_lowercase();
                
                if tag_lower.starts_with("script") {
                    in_script = true;
                } else if tag_lower == "/script" {
                    in_script = false;
                } else if tag_lower.starts_with("style") {
                    in_style = true;
                } else if tag_lower == "/style" {
                    in_style = false;
                } else if tag_lower.starts_with("img ") || tag_lower == "img" {
                    // Preserve image placeholders for translation
                    result.push_str("[IMAGE]");
                } else if tag_lower.starts_with("code") || tag_lower.starts_with("pre") || 
                          tag_lower == "/code" || tag_lower == "/pre" {
                    // Preserve code boundaries
                    result.push('<');
                    result.push_str(&current_tag);
                    result.push('>');
                } else if tag_lower == "p" || tag_lower == "/p" || 
                          tag_lower == "br" || tag_lower == "br/" ||
                          tag_lower.starts_with("h1") || tag_lower.starts_with("h2") || 
                          tag_lower.starts_with("h3") || tag_lower.starts_with("h4") ||
                          tag_lower.starts_with("h5") || tag_lower.starts_with("h6") ||
                          tag_lower == "/h1" || tag_lower == "/h2" || 
                          tag_lower == "/h3" || tag_lower == "/h4" ||
                          tag_lower == "/h5" || tag_lower == "/h6" {
                    // Add line breaks for paragraph and heading tags
                    result.push('\n');
                } else if tag_lower == "div" || tag_lower == "/div" {
                    // Add line breaks for div tags
                    result.push('\n');
                } else if tag_lower == "li" || tag_lower == "/li" {
                    result.push('\n');
                } else if tag_lower == "ul" || tag_lower == "/ul" || 
                          tag_lower == "ol" || tag_lower == "/ol" {
                    result.push('\n');
                }
                
                current_tag.clear();
            },
            _ if in_tag => {
                current_tag.push(ch);
            },
            _ if !in_script && !in_style && tag_depth == 0 => {
                result.push(ch);
            },
            _ => {
                // Skip script and style content
            }
        }
        i += 1;
    }
    
    // Clean up excessive whitespace while preserving paragraph breaks
    let lines: Vec<&str> = result.lines().collect();
    let mut cleaned_lines: Vec<String> = Vec::new();
    
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            cleaned_lines.push(trimmed.to_string());
        } else if !cleaned_lines.is_empty() && !cleaned_lines.last().unwrap().is_empty() {
            // Add empty line only if the last line wasn't empty (preserve paragraph breaks)
            cleaned_lines.push(String::new());
        }
    }
    
    cleaned_lines.join("\n").trim().to_string()
}

fn extract_text_from_html(html: &str) -> String {
    // More sophisticated HTML processing that preserves structure and content
    let mut result = String::new();
    let mut in_tag = false;
    let mut current_tag = String::new();
    let mut in_script = false;
    let mut in_style = false;
    
    let chars: Vec<char> = html.chars().collect();
    let mut i = 0;
    
    while i < chars.len() {
        let ch = chars[i];
        
        match ch {
            '<' => {
                in_tag = true;
                current_tag.clear();
            },
            '>' => {
                in_tag = false;
                
                // Handle different tag types
                if current_tag.starts_with("script") {
                    in_script = true;
                } else if current_tag == "/script" {
                    in_script = false;
                } else if current_tag.starts_with("style") {
                    in_style = true;
                } else if current_tag == "/style" {
                    in_style = false;
                } else if current_tag.starts_with("img ") || current_tag == "img" {
                    // Preserve image tags
                    result.push('<');
                    result.push_str(&current_tag);
                    result.push('>');
                } else if current_tag.starts_with("code") || current_tag.starts_with("pre") || current_tag == "/code" || current_tag == "/pre" {
                    // Preserve code tags
                    result.push('<');
                    result.push_str(&current_tag);
                    result.push('>');
                } else if current_tag == "p" || current_tag == "/p" || 
                          current_tag == "br" || current_tag == "br/" ||
                          current_tag.starts_with("h1") || current_tag.starts_with("h2") || 
                          current_tag.starts_with("h3") || current_tag.starts_with("h4") ||
                          current_tag.starts_with("h5") || current_tag.starts_with("h6") ||
                          current_tag == "/h1" || current_tag == "/h2" || 
                          current_tag == "/h3" || current_tag == "/h4" ||
                          current_tag == "/h5" || current_tag == "/h6" {
                    // Add line breaks for paragraph and heading tags
                    result.push('\n');
                } else if current_tag == "div" || current_tag == "/div" {
                    // Add line breaks for div tags
                    result.push('\n');
                }
                
                current_tag.clear();
            },
            _ if in_tag => {
                current_tag.push(ch);
            },
            _ if !in_script && !in_style => {
                result.push(ch);
            },
            _ => {
                // Skip script and style content
            }
        }
        i += 1;
    }
    
    // Clean up excessive whitespace while preserving paragraph breaks
    let lines: Vec<&str> = result.lines().collect();
    let mut cleaned_lines: Vec<String> = Vec::new();
    
    for line in lines {
        let trimmed = line.trim();
        if !trimmed.is_empty() {
            cleaned_lines.push(trimmed.to_string());
        } else if !cleaned_lines.is_empty() && !cleaned_lines.last().unwrap().is_empty() {
            // Add empty line only if the last line wasn't empty (preserve paragraph breaks)
            cleaned_lines.push(String::new());
        }
    }
    
    cleaned_lines.join("\n").trim().to_string()
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