use crate::models::{Message, StreamResponse, Usage};
use reqwest::blocking::Client;
use serde_json::json;
use std::io::Read;

pub trait LanguageModel {
    fn ask(
        &self,
        api_key: &str,
        model: &str,
        messages: &[Message],
    ) -> Result<(String, Option<Usage>), Box<dyn std::error::Error>>;
}

pub struct OpenRouterClient;

impl LanguageModel for OpenRouterClient {
    fn ask(
        &self,
        api_key: &str,
        model: &str,
        history: &[Message],
    ) -> Result<(String, Option<Usage>), Box<dyn std::error::Error>> {
        let client = Client::new();

        let payload = json!({
            "model": model,
            "messages": history,
            "stream": true,
            "usage": {
                "include": true,
            }
        });

        let mut response = client
            .post("https://openrouter.ai/api/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()?
            .error_for_status()?;

        let mut buffer = [0u8; 1024];
        let mut stream_buffer = String::new();
        let mut full_response = String::new();
        let mut usage: Option<Usage> = None;

        loop {
            let bytes_read = response.read(&mut buffer)?;

            if bytes_read == 0 {
                break;
            }

            let chunk = String::from_utf8_lossy(&buffer[..bytes_read]);

            stream_buffer.push_str(&chunk);

            while let Some(position) = stream_buffer.find('\n') {
                let line = stream_buffer[..position].trim_end_matches('\r').to_string();

                stream_buffer.drain(..=position);

                if line.is_empty() {
                    continue;
                }

                if line == "data: [DONE]" {
                    break;
                }

                if let Some(json_data) = line.strip_prefix("data: ") {
                    let chunk: StreamResponse = serde_json::from_str(json_data)?;
                    if chunk.usage.is_some() {
                        usage = chunk.usage;
                    }
                    let choice = chunk.choices.first();
                    if let Some(choice) = choice {
                        if let Some(content) = &choice.delta.content {
                            print!("{content}");
                            full_response.push_str(content);
                        }
                    }
                }
            }
        }

        Ok((full_response, usage))
    }
}
