use anyhow::Result;
use crate::models::{TranslationRequest, TranslationResponse};
use crate::services::OllamaService;

pub struct TranslationService {
    ollama: OllamaService,
    model: String,
}

impl TranslationService {
    pub fn new(ollama_url: String, model: String) -> Self {
        Self {
            ollama: OllamaService::new(ollama_url),
            model,
        }
    }

    pub async fn translate(&self, request: TranslationRequest) -> Result<TranslationResponse> {
        let translated_text = self.ollama.translate_text(
            &request.text,
            &request.source_language,
            &request.target_language,
            &self.model,
        ).await?;

        Ok(TranslationResponse {
            translated_text,
            confidence: 0.95, // Placeholder - could be enhanced with actual confidence scoring
            source_language: request.source_language,
            target_language: request.target_language,
        })
    }

    pub async fn translate_chapter_content(
        &self,
        content: &str,
        source_lang: &str,
        target_lang: &str,
    ) -> Result<String> {
        // Split content into smaller chunks for better translation
        let chunks = self.split_content_into_chunks(content, 1000); // ~1000 words per chunk
        let mut translated_chunks = Vec::new();

        for chunk in chunks {
            if chunk.trim().is_empty() {
                translated_chunks.push(chunk);
                continue;
            }

            let translated_chunk = self.ollama.translate_text(
                &chunk,
                source_lang,
                target_lang,
                &self.model,
            ).await?;

            translated_chunks.push(translated_chunk);
            
            // Small delay to avoid overwhelming the API
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }

        Ok(translated_chunks.join(" "))
    }

    fn split_content_into_chunks(&self, content: &str, max_words: usize) -> Vec<String> {
        let words: Vec<&str> = content.split_whitespace().collect();
        let mut chunks = Vec::new();
        let mut current_chunk = Vec::new();

        for word in words {
            current_chunk.push(word);
            
            if current_chunk.len() >= max_words {
                chunks.push(current_chunk.join(" "));
                current_chunk.clear();
            }
        }

        // Add remaining words as the last chunk
        if !current_chunk.is_empty() {
            chunks.push(current_chunk.join(" "));
        }

        chunks
    }

    pub async fn check_ollama_status(&self) -> Result<bool> {
        self.ollama.check_server_status().await
    }

    pub async fn get_available_models(&self) -> Result<Vec<String>> {
        self.ollama.list_models().await
    }

    pub async fn ensure_model_available(&self) -> Result<()> {
        if !self.ollama.check_model_availability(&self.model).await? {
            tracing::info!("Model {} not found, attempting to pull...", self.model);
            self.ollama.pull_model(&self.model).await?;
        }
        Ok(())
    }
}