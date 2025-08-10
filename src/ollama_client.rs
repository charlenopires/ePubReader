use crate::models::*;
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::{json, Value};
use tracing::{info, error, warn};

pub struct OllamaClient {
    client: Client,
    base_url: String,
}

impl OllamaClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:11434".to_string(),
        }
    }
    
    pub async fn check_status(&self) -> Result<OllamaStatus> {
        info!("Checking Ollama status");
        
        match self.client.get(&format!("{}/api/tags", self.base_url)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let data: Value = response.json().await?;
                    let models: Vec<String> = data["models"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|m| m["name"].as_str().map(|s| s.to_string()))
                        .collect();
                    
                    let recommended_model = self.find_recommended_model(&models);
                    
                    Ok(OllamaStatus {
                        is_running: true,
                        available_models: models,
                        recommended_model,
                    })
                } else {
                    Ok(OllamaStatus {
                        is_running: false,
                        available_models: vec![],
                        recommended_model: None,
                    })
                }
            }
            Err(e) => {
                warn!("Failed to connect to Ollama: {}", e);
                Ok(OllamaStatus {
                    is_running: false,
                    available_models: vec![],
                    recommended_model: None,
                })
            }
        }
    }
    
    fn find_recommended_model(&self, models: &[String]) -> Option<String> {
        // Priority order for translation models
        let preferred_models = [
            "llama3.1:8b",
            "llama3:8b",
            "llama2:7b",
            "mistral:7b",
            "codellama:7b",
        ];
        
        for preferred in &preferred_models {
            if models.iter().any(|m| m.contains(preferred)) {
                return models.iter().find(|m| m.contains(preferred)).cloned();
            }
        }
        
        // If no preferred model found, return the first available
        models.first().cloned()
    }
    
    pub async fn translate_text(&self, request: TranslationRequest) -> Result<String> {
        let status = self.check_status().await?;
        if !status.is_running {
            return Err(anyhow!("Ollama is not running"));
        }
        
        let model = status.recommended_model
            .ok_or_else(|| anyhow!("No suitable model available for translation"))?;
        
        let prompt = self.create_translation_prompt(&request);
        
        info!("Translating text from {} to {} using model {}", 
              request.source_language, request.target_language, model);
        
        let payload = json!({
            "model": model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.3,
                "top_p": 0.9,
                "max_tokens": 2048
            }
        });
        
        let response = self.client
            .post(&format!("{}/api/generate", self.base_url))
            .json(&payload)
            .send()
            .await?;
        
        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("Ollama API error: {}", error_text));
        }
        
        let data: Value = response.json().await?;
        let translated_text = data["response"]
            .as_str()
            .ok_or_else(|| anyhow!("Invalid response from Ollama"))?;
        
        // Clean up the response (remove any extra formatting)
        let cleaned_text = self.clean_translation_response(translated_text);
        
        Ok(cleaned_text)
    }
    
    fn create_translation_prompt(&self, request: &TranslationRequest) -> String {
        format!(
            r#"You are a professional translator. Translate the following text from {} to {}. 
            
Rules:
1. Maintain the original formatting and structure
2. Preserve any HTML tags if present
3. Keep proper names unchanged unless they have established translations
4. Maintain the tone and style of the original text
5. Only return the translated text, no explanations or additional comments

Text to translate:
{}

Translation:"#,
            request.source_language,
            request.target_language,
            request.text
        )
    }
    
    fn clean_translation_response(&self, response: &str) -> String {
        // Remove common prefixes that models might add
        let cleaned = response
            .trim()
            .strip_prefix("Translation:")
            .unwrap_or(response)
            .strip_prefix("Here is the translation:")
            .unwrap_or(response)
            .trim();
        
        cleaned.to_string()
    }
    
    pub async fn pull_recommended_model(&self) -> Result<String> {
        info!("Pulling recommended model for translation");
        
        let model_name = "llama3.1:8b"; // Default recommended model
        
        let payload = json!({
            "name": model_name
        });
        
        let response = self.client
            .post(&format!("{}/api/pull", self.base_url))
            .json(&payload)
            .send()
            .await?;
        
        if response.status().is_success() {
            info!("Successfully pulled model: {}", model_name);
            Ok(model_name.to_string())
        } else {
            let error_text = response.text().await?;
            Err(anyhow!("Failed to pull model: {}", error_text))
        }
    }
}

pub fn get_supported_languages() -> Vec<Language> {
    vec![
        Language {
            code: "en".to_string(),
            name: "English".to_string(),
            native_name: "English".to_string(),
        },
        Language {
            code: "pt".to_string(),
            name: "Portuguese".to_string(),
            native_name: "Português".to_string(),
        },
        Language {
            code: "es".to_string(),
            name: "Spanish".to_string(),
            native_name: "Español".to_string(),
        },
        Language {
            code: "fr".to_string(),
            name: "French".to_string(),
            native_name: "Français".to_string(),
        },
        Language {
            code: "de".to_string(),
            name: "German".to_string(),
            native_name: "Deutsch".to_string(),
        },
        Language {
            code: "it".to_string(),
            name: "Italian".to_string(),
            native_name: "Italiano".to_string(),
        },
        Language {
            code: "ja".to_string(),
            name: "Japanese".to_string(),
            native_name: "日本語".to_string(),
        },
        Language {
            code: "ko".to_string(),
            name: "Korean".to_string(),
            native_name: "한국어".to_string(),
        },
        Language {
            code: "zh".to_string(),
            name: "Chinese".to_string(),
            native_name: "中文".to_string(),
        },
        Language {
            code: "ru".to_string(),
            name: "Russian".to_string(),
            native_name: "Русский".to_string(),
        },
        Language {
            code: "ar".to_string(),
            name: "Arabic".to_string(),
            native_name: "العربية".to_string(),
        },
        Language {
            code: "hi".to_string(),
            name: "Hindi".to_string(),
            native_name: "हिन्दी".to_string(),
        },
    ]
}