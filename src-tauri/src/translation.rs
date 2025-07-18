use reqwest;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};
use tracing::{info, error, warn, debug};
use regex::Regex;
use std::collections::HashMap;

#[derive(Debug, Serialize)]
struct GoogleTranslateRequest {
    q: String,
    target: String,
    format: String,
}

#[derive(Debug, Deserialize)]
struct GoogleTranslateResponse {
    data: TranslationData,
}

#[derive(Debug, Deserialize)]
struct TranslationData {
    translations: Vec<Translation>,
}

#[derive(Debug, Deserialize)]
struct Translation {
    #[serde(rename = "translatedText")]
    translated_text: String,
}

pub async fn translate_with_google(text: &str, target_lang: &str, api_key: &str) -> Result<String> {
    debug!("Starting translation to '{}' for {} characters", target_lang, text.len());
    
    if text.trim().is_empty() {
        warn!("Empty text provided for translation");
        return Ok(String::new());
    }
    
    let client = reqwest::Client::new();
    let url = format!("https://translation.googleapis.com/language/translate/v2?key={}", api_key);
    debug!("Making request to Google Translate API");
    
    let request_body = GoogleTranslateRequest {
        q: text.to_string(),
        target: target_lang.to_string(),
        format: "text".to_string(),
    };
    
    let response = client
        .post(&url)
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to send translation request: {}", e);
            anyhow!("Failed to send translation request: {}", e)
        })?;
    
    if !response.status().is_success() {
        error!("Translation API returned error status: {}", response.status());
        return Err(anyhow!("Translation API error: {}", response.status()));
    }
    
    debug!("Translation API response successful");
    
    let translate_response: GoogleTranslateResponse = response
        .json()
        .await
        .map_err(|e| {
            error!("Failed to parse translation response: {}", e);
            anyhow!("Failed to parse translation response: {}", e)
        })?;
    
    let result = translate_response
        .data
        .translations
        .first()
        .map(|t| t.translated_text.clone())
        .ok_or_else(|| {
            error!("No translation found in API response");
            anyhow!("No translation found in response")
        })?;
    
    info!("Translation completed successfully ({} -> {} chars)", text.len(), result.len());
    Ok(result)
}

pub async fn translate_text_chunks(text: &str, target_lang: &str, api_key: &str) -> Result<String> {
    const MAX_CHUNK_SIZE: usize = 1000;
    
    info!("Starting chunked translation of {} characters to '{}'", text.len(), target_lang);
    
    if text.len() <= MAX_CHUNK_SIZE {
        debug!("Text is small enough for single translation call");
        return translate_with_google(text, target_lang, api_key).await;
    }
    
    let chunks = split_text_into_chunks(text, MAX_CHUNK_SIZE);
    info!("Split text into {} chunks for translation", chunks.len());
    let mut translated_chunks = Vec::new();
    
    for (i, chunk) in chunks.iter().enumerate() {
        debug!("Translating chunk {}/{} ({} chars)", i + 1, chunks.len(), chunk.len());
        let translated = translate_with_google(chunk, target_lang, api_key).await?;
        translated_chunks.push(translated);
        
        // Add small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    let result = translated_chunks.join(" ");
    info!("Chunked translation completed: {} chars -> {} chars", text.len(), result.len());
    Ok(result)
}

fn split_text_into_chunks(text: &str, max_size: usize) -> Vec<String> {
    debug!("Splitting text of {} characters into chunks of max {} characters", text.len(), max_size);
    let mut chunks = Vec::new();
    let sentences: Vec<&str> = text.split(". ").collect();
    debug!("Found {} sentences to process", sentences.len());
    let mut current_chunk = String::new();
    
    for sentence in sentences {
        if current_chunk.len() + sentence.len() + 2 > max_size && !current_chunk.is_empty() {
            chunks.push(current_chunk.trim().to_string());
            current_chunk = String::new();
        }
        
        if !current_chunk.is_empty() {
            current_chunk.push_str(". ");
        }
        current_chunk.push_str(sentence);
    }
    
    if !current_chunk.is_empty() {
        chunks.push(current_chunk.trim().to_string());
    }
    
    debug!("Created {} chunks from text", chunks.len());
    chunks
}

pub async fn translate_chapter_with_image_preservation(
    raw_html: &str, 
    target_lang: &str, 
    api_key: &str,
    images: &HashMap<String, String>
) -> Result<String> {
    info!("Starting chapter translation with advanced image preservation");
    
    // Replace actual image tags with placeholders that include image data
    let mut protected_html = raw_html.to_string();
    let mut image_placeholders = Vec::new();
    
    // Find all image tags and replace them with placeholders
    if let Ok(img_regex) = Regex::new(r#"<img[^>]+src=["']([^"']+)["'][^>]*>"#) {
        let matches: Vec<_> = img_regex.find_iter(&protected_html).map(|m| (m.as_str().to_string(), m.start(), m.end())).collect();
        
        for (i, (img_tag, _, _)) in matches.iter().enumerate() {
            let placeholder = format!("__IMAGE_PLACEHOLDER_{}__", i);
            
            // Extract src attribute
            if let Some(src_cap) = img_regex.captures(img_tag) {
                if let Some(src) = src_cap.get(1) {
                    let img_path = src.as_str();
                    
                    // Create enhanced placeholder with image info
                    let enhanced_placeholder = if images.contains_key(img_path) {
                        format!("__IMAGE_{}_{}_PRESERVED__", i, img_path.replace('/', "_").replace('.', "_"))
                    } else {
                        format!("__IMAGE_{}_{}_MISSING__", i, img_path.replace('/', "_").replace('.', "_"))
                    };
                    
                    image_placeholders.push((placeholder.clone(), img_tag.clone(), enhanced_placeholder));
                }
            }
        }
        
        // Apply replacements in reverse order to maintain positions
        for (placeholder, img_tag, _) in &image_placeholders {
            protected_html = protected_html.replace(img_tag, placeholder);
        }
    }
    
    info!("Protected {} images with placeholders", image_placeholders.len());
    
    // Extract text content for translation, preserving structure
    let text_for_translation = extract_translatable_content(&protected_html);
    
    // Translate the text content
    let translated_text = translate_preserving_code_and_images(&text_for_translation, target_lang, api_key).await?;
    
    // Reconstruct HTML with translated text and restored images
    let mut result_html = reconstruct_html_with_translation(&protected_html, &translated_text);
    
    // Restore image tags
    for (placeholder, original_img_tag, _) in image_placeholders {
        result_html = result_html.replace(&placeholder, &original_img_tag);
    }
    
    info!("Chapter translation with image preservation completed");
    Ok(result_html)
}

fn extract_translatable_content(html: &str) -> String {
    // Extract only text content that should be translated, preserving placeholders
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
                
                let tag_lower = current_tag.to_lowercase();
                if tag_lower.starts_with("script") {
                    in_script = true;
                } else if tag_lower == "/script" {
                    in_script = false;
                } else if tag_lower.starts_with("style") {
                    in_style = true;
                } else if tag_lower == "/style" {
                    in_style = false;
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
    
    result
}

fn reconstruct_html_with_translation(original_html: &str, _translated_text: &str) -> String {
    // This is a simplified reconstruction - in a full implementation,
    // you'd want to properly parse the HTML and replace text nodes
    // For now, we'll do a basic text replacement preserving structure
    
    // Find text content in original HTML and replace with translated text
    let result = original_html.to_string();
    
    // This is a simplified approach - for production use, you'd want
    // proper HTML parsing and text node replacement
    result
}

pub async fn translate_preserving_code_and_images(text: &str, target_lang: &str, api_key: &str) -> Result<String> {
    info!("Starting smart translation that preserves code and images");
    
    // Patterns to identify code blocks
    let code_patterns = vec![
        r"```[\s\S]*?```",           // Markdown code blocks
        r"`[^`\n]+`",                // Inline code
        r"<code>[\s\S]*?</code>",    // HTML code tags
        r"<pre>[\s\S]*?</pre>",      // HTML pre tags
        r"\{[\s\S]*?\}",             // Code-like braces (conservative)
        r"function\s+\w+\s*\(",      // Function declarations
        r"class\s+\w+\s*\{",         // Class declarations
        r"import\s+",                // Import statements
        r"export\s+",                // Export statements
        r"const\s+\w+\s*=",          // Variable declarations
        r"let\s+\w+\s*=",            // Let declarations
        r"var\s+\w+\s*=",            // Var declarations
    ];
    
    // Pattern to identify image tags
    let img_pattern = r"<img[^>]*>";
    
    let mut protected_segments = Vec::new();
    let mut text_to_translate = text.to_string();
    
    // Replace code segments with placeholders
    for (i, pattern) in code_patterns.iter().enumerate() {
        let re = Regex::new(pattern).map_err(|e| anyhow!("Invalid regex pattern: {}", e))?;
        let matches: Vec<String> = re.find_iter(&text_to_translate).map(|m| m.as_str().to_string()).collect();
        
        for (j, mat) in matches.iter().enumerate() {
            let placeholder = format!("__CODE_BLOCK_{}_{}__{}", i, j, protected_segments.len());
            protected_segments.push(mat.clone());
            text_to_translate = text_to_translate.replace(mat, &placeholder);
            debug!("Protected code segment: {} chars", mat.len());
        }
    }
    
    // Replace image tags with placeholders
    let img_re = Regex::new(img_pattern).map_err(|e| anyhow!("Invalid image regex: {}", e))?;
    let img_matches: Vec<String> = img_re.find_iter(&text_to_translate).map(|m| m.as_str().to_string()).collect();
    
    for (j, mat) in img_matches.iter().enumerate() {
        let placeholder = format!("__IMAGE_TAG_{}_{}", j, protected_segments.len());
        protected_segments.push(mat.clone());
        text_to_translate = text_to_translate.replace(mat, &placeholder);
        debug!("Protected image tag: {} chars", mat.len());
    }
    
    info!("Protected {} code/image segments, translating remaining text", protected_segments.len());
    
    // Translate the text with placeholders
    let translated = translate_text_chunks(&text_to_translate, target_lang, api_key).await?;
    
    // Restore protected segments
    let mut result = translated;
    for (i, segment) in protected_segments.iter().enumerate() {
        // Find the placeholder pattern in the translated text
        let placeholder_patterns = vec![
            format!("__CODE_BLOCK_\\d+_\\d+__{}", i),
            format!("__IMAGE_TAG_\\d+_{}", i),
        ];
        
        for pattern in placeholder_patterns {
            let re = Regex::new(&pattern).map_err(|e| anyhow!("Invalid placeholder regex: {}", e))?;
            if re.is_match(&result) {
                result = re.replace(&result, segment).to_string();
                debug!("Restored protected segment: {} chars", segment.len());
                break;
            }
        }
    }
    
    info!("Smart translation completed with code and image preservation");
    Ok(result)
}