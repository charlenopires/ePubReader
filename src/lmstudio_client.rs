use crate::models::*;
use anyhow::{Result, anyhow};
use reqwest::Client;
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};
use tracing::{info, warn};

pub struct LmStudioClient {
    client: Client,
    base_url: String,
    current_model: Arc<Mutex<Option<String>>>,
}

impl LmStudioClient {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            base_url: "http://localhost:1234/v1".to_string(),
            current_model: Arc::new(Mutex::new(None)),
        }
    }

    pub async fn check_status(&self) -> Result<LmStudioStatus> {
        info!("Checking LM Studio status");

        match self.client.get(&format!("{}/models", self.base_url)).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let data: Value = response.json().await?;
                    let models: Vec<String> = data["data"]
                        .as_array()
                        .unwrap_or(&vec![])
                        .iter()
                        .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
                        .collect();

                    let recommended_model = models.first().cloned();

                    Ok(LmStudioStatus {
                        is_running: true,
                        available_models: models,
                        recommended_model,
                    })
                } else {
                    Ok(LmStudioStatus {
                        is_running: false,
                        available_models: vec![],
                        recommended_model: None,
                    })
                }
            }
            Err(e) => {
                warn!("Failed to connect to LM Studio: {}", e);
                Ok(LmStudioStatus {
                    is_running: false,
                    available_models: vec![],
                    recommended_model: None,
                })
            }
        }
    }

    pub async fn list_models(&self) -> Result<Vec<String>> {
        info!("Listing available LM Studio models");

        let response = self.client
            .get(&format!("{}/models", self.base_url))
            .send()
            .await?;

        if !response.status().is_success() {
            return Err(anyhow!("Failed to get models from LM Studio"));
        }

        let data: Value = response.json().await?;
        let models: Vec<String> = data["data"]
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
            .collect();

        Ok(models)
    }

    pub async fn translate_text(&self, request: TranslationRequest) -> Result<String> {
        let status = self.check_status().await?;
        if !status.is_running {
            return Err(anyhow!("LM Studio is not running"));
        }

        // Use current model if set, otherwise use recommended model
        let model = {
            let current = self.current_model.lock().unwrap();
            current.clone()
        }.or(status.recommended_model)
            .ok_or_else(|| anyhow!("No suitable model available for translation"))?;

        info!(
            "Translating text from {} to {} using model {}",
            request.source_language, request.target_language, model
        );

        let system_prompt = format!(
            "You are a professional translator. Translate from {} to {}. \
             Rules: preserve HTML tags, maintain formatting, keep proper names, only output translation.",
            request.source_language, request.target_language
        );

        let payload = json!({
            "model": model,
            "messages": [
                {"role": "system", "content": system_prompt},
                {"role": "user", "content": request.text}
            ],
            "temperature": 0.3,
            "max_tokens": 16384
        });

        let response = self.client
            .post(&format!("{}/chat/completions", self.base_url))
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            return Err(anyhow!("LM Studio API error: {}", error_text));
        }

        let data: Value = response.json().await?;
        let message = data["choices"]
            .get(0)
            .and_then(|choice| choice.get("message"))
            .ok_or_else(|| anyhow!("Invalid response from LM Studio"))?;

        // Try "content" first; if empty, fall back to "reasoning_content"
        // (some models like Qwen3.5 use thinking mode and may put the
        // final answer in content only after reasoning, or sometimes
        // the translation ends up in reasoning_content)
        let translated_text = message["content"]
            .as_str()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| {
                message["reasoning_content"]
                    .as_str()
                    .and_then(|r| self.extract_translation_from_reasoning(r))
            })
            .ok_or_else(|| anyhow!("No translation content in LM Studio response"))?;

        // Clean up the response (remove any extra formatting)
        let cleaned_text = self.clean_translation_response(translated_text);

        Ok(cleaned_text)
    }

    /// When thinking models put the translation inside reasoning_content,
    /// try to extract the last HTML block or the text after "Final" / "Output" markers.
    fn extract_translation_from_reasoning<'a>(&self, reasoning: &'a str) -> Option<&'a str> {
        // Look for the last occurrence of an HTML tag block as the final output
        if let Some(pos) = reasoning.rfind("<p>") {
            // Find the end of the last HTML block
            if let Some(end) = reasoning[pos..].rfind("</p>") {
                let end_pos = pos + end + 4; // "</p>".len()
                let candidate = reasoning[pos..end_pos].trim();
                if !candidate.is_empty() {
                    return Some(candidate);
                }
            }
        }
        None
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

    pub async fn set_model(&self, model: String) -> Result<()> {
        info!("Setting current model to: {}", model);

        // Verify the model exists
        let available_models = self.list_models().await?;
        if !available_models.contains(&model) {
            return Err(anyhow!("Model '{}' is not available", model));
        }

        // Set the current model
        {
            let mut current = self.current_model.lock().unwrap();
            *current = Some(model.clone());
        }

        info!("Successfully set current model to: {}", model);
        Ok(())
    }

    pub async fn get_current_model(&self) -> Result<String> {
        let current_model = {
            let current = self.current_model.lock().unwrap();
            current.clone()
        };

        match current_model {
            Some(model) => Ok(model),
            None => {
                // If no model is set, return the recommended model
                let status = self.check_status().await?;
                status.recommended_model
                    .ok_or_else(|| anyhow!("No model is currently set and no recommended model available"))
            }
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
            native_name: "Portugu\u{00ea}s".to_string(),
        },
        Language {
            code: "es".to_string(),
            name: "Spanish".to_string(),
            native_name: "Espa\u{00f1}ol".to_string(),
        },
        Language {
            code: "fr".to_string(),
            name: "French".to_string(),
            native_name: "Fran\u{00e7}ais".to_string(),
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
            native_name: "\u{65e5}\u{672c}\u{8a9e}".to_string(),
        },
        Language {
            code: "ko".to_string(),
            name: "Korean".to_string(),
            native_name: "\u{d55c}\u{ad6d}\u{c5b4}".to_string(),
        },
        Language {
            code: "zh".to_string(),
            name: "Chinese".to_string(),
            native_name: "\u{4e2d}\u{6587}".to_string(),
        },
        Language {
            code: "ru".to_string(),
            name: "Russian".to_string(),
            native_name: "\u{0420}\u{0443}\u{0441}\u{0441}\u{043a}\u{0438}\u{0439}".to_string(),
        },
        Language {
            code: "ar".to_string(),
            name: "Arabic".to_string(),
            native_name: "\u{0627}\u{0644}\u{0639}\u{0631}\u{0628}\u{064a}\u{0629}".to_string(),
        },
        Language {
            code: "hi".to_string(),
            name: "Hindi".to_string(),
            native_name: "\u{0939}\u{093f}\u{0928}\u{094d}\u{0926}\u{0940}".to_string(),
        },
    ]
}
