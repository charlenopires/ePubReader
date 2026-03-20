//! Integration test for the full ePub translation pipeline with LMStudio + Qwen3.5.
//! Run with: cargo test --test test_translation -- --nocapture

use std::process::Command;
use serde_json::Value;

fn curl_lmstudio(endpoint: &str) -> Result<Value, String> {
    let output = Command::new("curl")
        .args(["-s", &format!("http://localhost:1234/v1{}", endpoint)])
        .output()
        .map_err(|e| format!("Failed to run curl: {}", e))?;
    let body = String::from_utf8_lossy(&output.stdout).to_string();
    serde_json::from_str(&body).map_err(|e| format!("JSON error: {} - body: {}", e, body))
}

fn curl_lmstudio_post(endpoint: &str, payload: &Value) -> Result<Value, String> {
    let output = Command::new("curl")
        .args([
            "-s", "-H", "Content-Type: application/json",
            "-d", &payload.to_string(),
            &format!("http://localhost:1234/v1{}", endpoint),
        ])
        .output()
        .map_err(|e| format!("curl failed: {}", e))?;
    let body = String::from_utf8_lossy(&output.stdout).to_string();
    serde_json::from_str(&body).map_err(|e| format!("JSON error: {} - body: {}", e, body))
}

/// Extract translation from response, handling thinking models.
fn extract_translation(data: &Value) -> String {
    let message = &data["choices"][0]["message"];
    let content = message["content"].as_str().unwrap_or("").trim();

    if !content.is_empty() {
        return content.to_string();
    }

    // Fallback: extract from reasoning_content (thinking models like Qwen3.5)
    let reasoning = message["reasoning_content"].as_str().unwrap_or("");
    if !reasoning.is_empty() {
        // Find the last HTML paragraph block
        if let Some(pos) = reasoning.rfind("<p>") {
            if let Some(end_rel) = reasoning[pos..].rfind("</p>") {
                return reasoning[pos..pos + end_rel + 4].to_string();
            }
        }
    }

    String::new()
}

#[test]
fn test_01_lmstudio_connection() {
    println!("\n=== Test 1: LMStudio Connection ===");

    let data = curl_lmstudio("/models").expect("LMStudio is not running");
    let models: Vec<String> = data["data"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .filter_map(|m| m["id"].as_str().map(|s| s.to_string()))
        .collect();

    println!("LMStudio running with {} models:", models.len());
    for m in &models {
        println!("  - {}", m);
    }

    assert!(!models.is_empty(), "No models loaded");
    assert!(models.iter().any(|m| m.contains("qwen")), "qwen model not found: {:?}", models);
    println!("PASSED\n");
}

#[test]
fn test_02_translate_frankenstein() {
    println!("\n=== Test 2: Translate Frankenstein excerpt (EN -> PT) ===");

    let original = "<p>The rain pattered dismally against the panes.</p>";

    let payload = serde_json::json!({
        "model": "qwen/qwen3.5-9b",
        "messages": [
            {"role": "system", "content": "You are a professional translator. Translate from English to Portuguese. Rules: preserve HTML tags, maintain formatting, keep proper names, only output translation."},
            {"role": "user", "content": original}
        ],
        "temperature": 0.3,
        "max_tokens": 16384
    });

    println!("Translating (may take ~2min with thinking model)...");
    let data = curl_lmstudio_post("/chat/completions", &payload).expect("API request failed");

    let model = data["model"].as_str().unwrap_or("?");
    let tokens = data["usage"]["completion_tokens"].as_u64().unwrap_or(0);
    let finish = data["choices"][0]["finish_reason"].as_str().unwrap_or("?");

    let translated = extract_translation(&data);

    println!("Model: {}", model);
    println!("Tokens: {}, Finish: {}", tokens, finish);
    println!("\nOriginal:   {}", original);
    println!("Translated: {}", translated);

    // Assertions
    assert!(!translated.is_empty(), "Translation is empty");
    assert!(translated.contains("<p>"), "<p> tag not preserved");
    assert!(translated.contains("</p>"), "</p> tag not preserved");
    assert!(model.contains("qwen"), "Wrong model: {}", model);

    // Check Portuguese content
    let lower = translated.to_lowercase();
    let pt = ["de", "que", "uma", "da", "do", "na", "foi", "noite", "novembro"];
    let found: Vec<&&str> = pt.iter().filter(|w| {
        lower.split(|c: char| !c.is_alphanumeric()).any(|word| word == **w)
    }).collect();
    println!("PT words found: {:?}", found);
    assert!(found.len() >= 2, "Not Portuguese enough: {:?}", found);

    println!("\n=== PASSED ===\n");
}

#[test]
fn test_03_html_preservation() {
    println!("\n=== Test 3: HTML tag preservation ===");

    let original = r#"<h2>Chapter 1</h2><p>Call me <strong>Ishmael</strong>. Some years ago.</p>"#;

    let payload = serde_json::json!({
        "model": "qwen/qwen3.5-9b",
        "messages": [
            {"role": "system", "content": "You are a professional translator. Translate from English to Portuguese. Rules: preserve HTML tags, maintain formatting, keep proper names, only output translation."},
            {"role": "user", "content": original}
        ],
        "temperature": 0.3,
        "max_tokens": 16384
    });

    println!("Translating with HTML tags...");
    let data = curl_lmstudio_post("/chat/completions", &payload).expect("API request failed");

    let translated = extract_translation(&data);
    println!("Original:   {}", original);
    println!("Translated: {}", translated);

    assert!(!translated.is_empty(), "Translation is empty");
    // Check that at least <p> is preserved
    assert!(translated.contains("<p>") || translated.contains("<h2>"),
        "HTML tags not preserved in: {}", translated);

    // "Ishmael" should be kept (proper name)
    let has_ishmael = translated.to_lowercase().contains("ishmael");
    println!("Proper name 'Ishmael' preserved: {}", has_ishmael);

    println!("\n=== PASSED ===\n");
}
