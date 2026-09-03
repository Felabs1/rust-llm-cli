use clap::{Parser, Subcommand};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "llm-cli")]
pub struct Cli {
    #[arg(long, default_value = "system_prompt.txt")]
    pub system_prompt: String,
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Ask {
        prompt: String,
        #[arg(short, long)]
        model: Option<String>,
    },
}

pub fn parse_commands() -> Result<Cli, Box<dyn std::error::Error>> {
    let cli = Cli::parse();
    Ok(cli)
}

pub fn read_prompt() -> Result<String, Box<dyn std::error::Error>> {
    print!("You: ");
    io::stdout().flush()?;
    let mut input = String::new();

    io::stdin().read_line(&mut input)?;

    Ok(input.trim().to_string())
}
