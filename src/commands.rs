use clap::{Parser, Subcommand};
use std::io::{self, Write};

#[derive(Parser)]
#[command(name = "llm-cli")]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Ask,
}

pub fn parse_commands() -> Result<Commands, Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    Ok(cli.command)
}

pub fn read_prompt() -> Result<String, Box<dyn std::error::Error>> {
    print!("You: ");
    io::stdout().flush()?;
    let mut input = String::new();

    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}
