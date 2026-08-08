use clap::{Parser, Subcommand};

#[derive(Parser)]
pub struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Ask { prompt: String },
    Version,
    Models,
}

pub fn parse_commands() -> Result<String, Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ask { prompt } => {
            Ok(prompt.to_string())
        }

        Commands::Version => {
            Ok("v1.0".to_string())
        }

        Commands::Models => {
            Ok("listing models...".to_string())
        }
    }
}
