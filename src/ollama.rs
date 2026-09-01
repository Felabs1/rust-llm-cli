// src/ollama.rs
use crate::client::LanguageModel;   // ← ONLY the trait. Not OpenRouterClient.
use crate::models::{Message, Usage};

pub struct OllamaClient;

impl LanguageModel for OllamaClient {
    fn ask(
        &self,
        _api_key: &str,             // Ollama is local, no key needed
        model: &str,
        messages: &[Message],
    ) -> Result<(String, Option<Usage>), Box<dyn std::error::Error>> {
        // Ollama-specific code: call http://localhost:11434/api/chat
        // ... totally different from OpenRouter, and that's fine ...
        todo!()
    }
}