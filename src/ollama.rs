#![allow(dead_code)]
use crate::client::LanguageModel; // ← ONLY the trait. Not OpenRouterClient.
use crate::models::{Message, Usage};
use reqwest::blocking::Client;
use serde::Deserialize;
use serde_json::json;
use std::io::Read;
use std::time::Instant;

pub struct OllamaClient;

#[derive(Deserialize)]
struct OllamaResponse {
    message: OllamaMessage,
    eval_count: Option<i32>,
    done: bool,
}

#[derive(Deserialize)]
struct OllamaMessage {
    content: String,
}

impl LanguageModel for OllamaClient {
    fn ask(
        &self,
        _api_key: &str, // Ollama is local, no key needed
        model: &str,
        messages: &[Message],
    ) -> Result<(String, Option<Usage>), Box<dyn std::error::Error>> {
        let client = Client::new();

        let payload = json!({
            "model": model,
            "messages": messages,
            "stream": true,
            "options": {
                "num_predict": 2048
            }
        });

        let start_time = Instant::now();
        let first_token_time: Option<Instant> = None;

        let mut response = client
            .post("http://localhost:11434/api/chat")
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

            // process complete lines (each line is a JSON object) in ollama's NDJSON format
            while let Some(position) = stream_buffer.find('\n') {
                let line = stream_buffer[..position].trim_end_matches('\r').to_string();
                stream_buffer.drain(..=position);

                if line.is_empty() {
                    continue;
                }

                // parse the JSON line
                let chunk: OllamaResponse = serde_json::from_str(&line)?;

                // if this chunk has a message with content, print and accumulate it
                if !chunk.message.content.is_empty() {
                    print!("{}", chunk.message.content);
                    full_response.push_str(&chunk.message.content);
                }

                // if this is the final chunk, capture the usage stats
                if chunk.done {
                    if let Some(tokens) = chunk.eval_count {
                        usage = Some(Usage {
                            prompt_tokens: 0,
                            completion_tokens: tokens as usize,
                            total_tokens: tokens as usize,
                        });
                    }

                    break;
                }
            }
        }

        println!();

        // ⏱️ PRINT METRICS
        let total_time = start_time.elapsed();
        let ttft = first_token_time
            .unwrap_or(start_time)
            .duration_since(start_time);

        let tokens_generated = usage.as_ref().map(|u| u.completion_tokens).unwrap_or(0);
        let generation_time = total_time.saturating_sub(ttft);
        let tps = if generation_time.as_secs_f64() > 0.0 {
            tokens_generated as f64 / generation_time.as_secs_f64()
        } else {
            0.0
        };

        println!("\n⏱️  Performance Metrics (Local Ollama):");
        println!("  Time to first token:  {:.3}s", ttft.as_secs_f64());
        println!("  Tokens generated:     {}", tokens_generated);
        println!("  Tokens per second:    {:.1} tok/s", tps);
        println!("  Total time:           {:.3}s", total_time.as_secs_f64());

        Ok((full_response, usage))
    }
}
