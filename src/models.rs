use serde::Deserialize;
use serde::Serialize;


#[derive(Debug, Deserialize, Serialize, Hash, PartialEq, Eq, Clone)]
pub struct Message {
    pub role: String,
    pub content: String,
}

#[derive(Debug, Deserialize)]
pub struct StreamResponse {
    pub choices: Vec<StreamChoice>,
    pub usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
pub struct StreamChoice {
    pub delta: StreamDelta,
}

#[derive(Debug, Deserialize)]
pub struct StreamDelta {
    pub content: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Pricing {
    pub model: String,
    pub input_per_million: f64,
    pub output_per_million: f64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Usage {
    pub prompt_tokens: usize,
    pub completion_tokens: usize,
    #[allow(dead_code)]
    pub total_tokens: usize,
}
