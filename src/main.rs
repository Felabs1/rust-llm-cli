mod client;
mod commands;
mod config;
mod models;

use models::Message;

const MESSAGES_PER_TURN: usize = 2;
const MAX_TOKENS: usize = 50;

fn estimate_tokens(messages: &[Message]) -> usize {
    messages
        .iter()
        .map(|message| message.content.len() / 4)
        .sum()
}

fn truncate_history(history: &mut Vec<Message>, max_tokens: usize) {
    while estimate_tokens(history) > max_tokens && history.len() > MESSAGES_PER_TURN {
        history.drain(0..MESSAGES_PER_TURN);
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = config::api_key()?;
    let command = commands::parse_commands()?;

    match command {
        commands::Commands::Ask { prompt } => {
            let mut history: Vec<Message> = Vec::new();

            let user_message = Message {
                role: "user".to_string(),
                content: prompt,
            };

            history.push(user_message);

            truncate_history(&mut history, MAX_TOKENS);

            let reply = client::ask(&api_key, &history)?;

            println!("\n\nComplete response");
            println!("{reply}");
            history.push(Message {
                role: "assistant".to_string(),
                content: reply,
            });

            loop {
                let prompt = commands::read_prompt()?;

                if prompt.eq_ignore_ascii_case("exit") {
                    break;
                }

                let user_message = Message {
                    role: "user".to_string(),
                    content: prompt,
                };

                history.push(user_message);

                truncate_history(&mut history, MAX_TOKENS);

                let reply = client::ask(&api_key, &history)?;

                println!("\n\nComplete response: ");
                println!("{reply}");
                history.push(Message {
                    role: "assistant".to_string(),
                    content: reply,
                });
            }
        }
    }

    Ok(())
}
