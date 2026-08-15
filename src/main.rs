mod client;
mod commands;
mod config;
mod models;

use models::Message;
use models::Pricing;
use models::Usage;

const MESSAGES_PER_TURN: usize = 2;
const MAX_TOKENS: usize = 50;

fn estimate_tokens(messages: &[Message]) -> usize {
    messages
        .iter()
        .map(|message| message.content.len() / 4)
        .sum()
}

fn estimate_cost(input_tokens: usize, output_tokens: usize, pricing: &Pricing) -> f64 {
    let input_cost = (input_tokens as f64 / 1_000_000.0) * pricing.input_per_million;
    let output_cost = (output_tokens as f64 / 1_000_000.0) * pricing.output_per_million;

    input_cost + output_cost
}

fn truncate_history(history: &mut Vec<Message>, max_tokens: usize) {
    while estimate_tokens(history) > max_tokens && history.len() > MESSAGES_PER_TURN + 1 {
        history.drain(1..=MESSAGES_PER_TURN);
    }
}

fn calculate_cost(
    usage: Option<&Usage>,
    input_tokens: usize,
    output_tokens: usize,
    config: &Pricing,
) -> (f64, bool) {
    match usage {
        Some(usage) => {
            let cost = estimate_cost(usage.prompt_tokens, usage.completion_tokens, config);

            (cost, true)
        }
        None => {
            let cost = estimate_cost(input_tokens, output_tokens, config);

            (cost, false)
        }
    }
}

fn print_history(history: &[Message]) {
    println!("\n-------History--------");

    for (index, message) in history.iter().enumerate() {
        println!("[{}] {}: {}", index, message.role, message.content)
    }

    println!("\n----------------------------\n")
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = config::api_key()?;
    let cli = commands::parse_commands()?;
    let system_prompt = config::system_prompt(&cli.system_prompt)?;
    let pricing = config::pricing()?;

    match cli.command {
        commands::Commands::Ask { prompt } => {
            let mut history: Vec<Message> = vec![Message {
                role: "system".to_string(),
                content: system_prompt,
            }];

            let user_message = Message {
                role: "user".to_string(),
                content: prompt,
            };

            history.push(user_message);

            truncate_history(&mut history, MAX_TOKENS);

            let input_tokens = estimate_tokens(&history);
            let (reply, usage) = client::ask(&api_key, &pricing.model, &history)?;
            let output_tokens = reply.len() / 4;
            let total_tokens = input_tokens + output_tokens;

            let (total_cost, used_actual_usage) =
                calculate_cost(usage.as_ref(), input_tokens, output_tokens, &pricing);

            println!("\n\nComplete response");
            println!("{reply}");

            println!("\n---------usage----------");
            println!("estimated input tokens: {input_tokens}");
            println!("estimated output tokens: {output_tokens}");
            println!("estimated total tokens: {total_tokens}");
            println!("estimated cost: ${total_cost:.8}");

            println!("pricing model: {}", pricing.model);
            println!(
                "cost source: {}",
                if used_actual_usage {
                    "provider usage"
                } else {
                    "estimated tokens"
                }
            );

            if let Some(usage) = &usage {
                println!("\n---------provider usage----------");
                println!("actual input tokens: {}", usage.prompt_tokens);
                println!("actual output tokens: {}", usage.completion_tokens);
                println!("actual total tokens: {}", usage.total_tokens);
            }

            history.push(Message {
                role: "assistant".to_string(),
                content: reply,
            });

            // print_history(&history);

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

                let input_tokens = estimate_tokens(&history);

                let (reply, usage) = client::ask(&api_key, &pricing.model, &history)?;

                let output_tokens = reply.len() / 4;
                let total_tokens = input_tokens + output_tokens;

                let (total_cost, used_actual_usage) =
                    calculate_cost(usage.as_ref(), input_tokens, output_tokens, &pricing);

                println!("\n\nComplete response: ");
                println!("{reply}");

                println!("\n---------usage----------");
                println!("estimated input tokens: {input_tokens}");
                println!("estimated output tokens: {output_tokens}");
                println!("estimated total tokens: {total_tokens}");
                println!("estimated cost: ${total_cost:.8}");

                println!(
                    "cost source: {}",
                    if used_actual_usage {
                        "provider usage"
                    } else {
                        "estimated tokens"
                    }
                );

                if let Some(usage) = &usage {
                    println!("\n---------provider usage----------");
                    println!("actual input tokens: {}", usage.prompt_tokens);
                    println!("actual output tokens: {}", usage.completion_tokens);
                    println!("actual total tokens: {}", usage.total_tokens);
                }

                history.push(Message {
                    role: "assistant".to_string(),
                    content: reply,
                });

                // print_history(&history);
            }
        }
    }

    Ok(())
}
