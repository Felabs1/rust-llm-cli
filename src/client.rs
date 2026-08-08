use crate::models::ChatResponse;
use reqwest::blocking::Client;
use serde_json::json;

pub fn ask(api_key: &str, prompt: &str) -> Result<ChatResponse, Box<dyn std::error::Error>> {
    let client = Client::new();

    let payload = json!({
        "model": "openrouter/free",
        "messages": [
            {
                "role": "user",
                "content": prompt.trim()
            }
        ]
    });

    let chat: ChatResponse = client
        .post("https://openrouter.ai/api/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()?
        .error_for_status()?
        .json()?;

    Ok(chat)
}
