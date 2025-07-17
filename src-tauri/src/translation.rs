use reqwest;
use serde::{Deserialize, Serialize};
use anyhow::{Result, anyhow};

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
    if text.trim().is_empty() {
        return Ok(String::new());
    }
    
    let client = reqwest::Client::new();
    let url = format!("https://translation.googleapis.com/language/translate/v2?key={}", api_key);
    
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
        .map_err(|e| anyhow!("Failed to send translation request: {}", e))?;
    
    if !response.status().is_success() {
        return Err(anyhow!("Translation API error: {}", response.status()));
    }
    
    let translate_response: GoogleTranslateResponse = response
        .json()
        .await
        .map_err(|e| anyhow!("Failed to parse translation response: {}", e))?;
    
    translate_response
        .data
        .translations
        .first()
        .map(|t| t.translated_text.clone())
        .ok_or_else(|| anyhow!("No translation found in response"))
}

pub async fn translate_text_chunks(text: &str, target_lang: &str, api_key: &str) -> Result<String> {
    const MAX_CHUNK_SIZE: usize = 1000;
    
    if text.len() <= MAX_CHUNK_SIZE {
        return translate_with_google(text, target_lang, api_key).await;
    }
    
    let chunks = split_text_into_chunks(text, MAX_CHUNK_SIZE);
    let mut translated_chunks = Vec::new();
    
    for chunk in chunks {
        let translated = translate_with_google(&chunk, target_lang, api_key).await?;
        translated_chunks.push(translated);
        
        // Add small delay to avoid rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    Ok(translated_chunks.join(" "))
}

fn split_text_into_chunks(text: &str, max_size: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let sentences: Vec<&str> = text.split(". ").collect();
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
    
    chunks
}