mod client;
mod commands;
mod config;
mod models;

use models::Message;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = config::api_key()?;
    let command = commands::parse_commands()?;

    match command {
        commands::Commands::Ask => {
            let mut history: Vec<Message> = Vec::new();

            loop {
                let prompt = commands::read_prompt()?;

                if prompt.eq_ignore_ascii_case("exit") {
                    break
                }

                let user_message = Message{
                    role: "user".to_string(),
                    content: prompt
                };

                history.push(user_message);

                let reply = client::ask(&api_key, &history)?;

                let assistant_content = reply
                .choices
                .into_iter()
                .next()
                .ok_or("Api returned no choices")?
                .message
                .content;

                let assistant_message = Message {
                    role: "assistant".to_string(),
                    content: assistant_content,
                };

                println!("AI: {}", assistant_message.content);

                history.push(assistant_message);
            }
        }
    }

    Ok(())
}
