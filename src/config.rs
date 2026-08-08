use dotenvy::dotenv;
use std::env;

pub fn api_key() -> Result<String, env::VarError> {
    dotenv().ok();

    env::var("OPENROUTER_API_KEY")
}
