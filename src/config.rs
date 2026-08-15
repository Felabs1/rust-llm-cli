use crate::models::Pricing;
use dotenvy::dotenv;
use std::env;

pub fn api_key() -> Result<String, env::VarError> {
    dotenv().ok();

    env::var("OPENROUTER_API_KEY")
}

pub fn system_prompt(path: &str) -> Result<String, Box<dyn std::error::Error>> {
    let prompt = std::fs::read_to_string(path)?;
    Ok(prompt)
}

pub fn pricing() -> Result<Pricing, Box<dyn std::error::Error>> {
    let contents = std::fs::read_to_string("pricing.json")?;
    let pricing: Pricing = serde_json::from_str(&contents)?;
    Ok(pricing)
}
