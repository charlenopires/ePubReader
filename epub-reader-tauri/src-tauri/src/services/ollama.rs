use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::json;
use crate::models::{OllamaRequest, OllamaResponse};

pub struct OllamaService {
    client: Client,
    base_url: String,
}

impl OllamaService {
    pub fn new(base_url: String) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }

    pub async fn check_server_status(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.base_url);
        
        match self.client.get(&url).send().await {
            Ok(response) => Ok(response.status().is_success()),
            Err(_) => Ok(false),
        }
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        let url = format!("{}/api/tags", self.base_url);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to fetch models from Ollama"));
        }

        let json: serde_json::Value = response.json().await?;
        let models = json["models"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|model| {
                model["name"].as_str().map(|name| name.to_string())
            })
            .collect();

        Ok(models)
    }

    pub async fn translate_text(
        &self,
        text: &str,
        source_lang: &str,
        target_lang: &str,
        model: &str,
    ) -> Result<String> {
        let prompt = self.create_translation_prompt(text, source_lang, target_lang);
        
        let request = OllamaRequest {
            model: model.to_string(),
            prompt,
            stream: false,
        };

        let url = format!("{}/api/generate", self.base_url);
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Translation request failed: {}", response.status()));
        }

        let ollama_response: OllamaResponse = response.json().await?;
        
        // Clean up the response to extract just the translation
        let translated = self.extract_translation_from_response(&ollama_response.response);
        
        Ok(translated)
    }

    fn create_translation_prompt(&self, text: &str, source_lang: &str, target_lang: &str) -> String {
        format!(
            r#"You are a professional translator. Translate the following text from {} to {}. 
            
Rules:
1. Provide ONLY the translation, no explanations or additional text
2. Maintain the original formatting and structure
3. Preserve proper nouns when appropriate
4. Keep the same tone and style as the original
5. If there are technical terms, translate them appropriately for the target language

Text to translate:
{}

Translation:"#,
            source_lang, target_lang, text
        )
    }

    fn extract_translation_from_response(&self, response: &str) -> String {
        // Remove common prefixes that the model might add
        let prefixes_to_remove = [
            "Translation:",
            "Here is the translation:",
            "The translation is:",
            "Translated text:",
        ];

        let mut cleaned = response.trim().to_string();
        
        for prefix in &prefixes_to_remove {
            if cleaned.starts_with(prefix) {
                cleaned = cleaned[prefix.len()..].trim().to_string();
                break;
            }
        }

        // Remove quotes if the entire response is wrapped in them
        if cleaned.starts_with('"') && cleaned.ends_with('"') && cleaned.len() > 2 {
            cleaned = cleaned[1..cleaned.len()-1].to_string();
        }

        cleaned
    }

    pub async fn check_model_availability(&self, model: &str) -> Result<bool> {
        let models = self.list_models().await?;
        Ok(models.iter().any(|m| m.contains(model)))
    }

    pub async fn pull_model(&self, model: &str) -> Result<()> {
        let url = format!("{}/api/pull", self.base_url);
        
        let request = json!({
            "name": model,
            "stream": false
        });

        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to pull model: {}", response.status()));
        }

        Ok(())
    }
}