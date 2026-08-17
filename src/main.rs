mod client;
mod commands;
mod config;
mod models;

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use models::Message;
use models::Pricing;
use models::Usage;

const MESSAGES_PER_TURN: usize = 2;
const MAX_TOKENS: usize = 500;

type ResponseCache = HashMap<u64, (String, Option<Usage>)>;


fn ask_with_cache(
    cache: &mut HashMap<u64, (String, Option<Usage>)>,
    api_key: &str,
    model: &str,
    history: &[Message],
) -> Result<(String, Option<Usage>), Box<dyn std::error::Error>> {
    // 1. Turn the entire conversation history into a unique key
    let mut hasher = DefaultHasher::new();
    history.hash(&mut hasher);
    let cache_key = hasher.finish();

    // 2. CACHE HIT: we've seen this exact conversation before
    if let Some((reply, usage)) = cache.get(&cache_key) {
        println!("\n⚡ [CACHE HIT] serving from memory\n");
        return Ok((reply.clone(), usage.clone()));
    }

    // 3. CACHE MISS: hit the real API
    println!("\n🌐 [CACHE MISS] calling API\n");
    let (reply, usage) = client::ask(api_key, model, history)?;

    // 4. Store the result so next time it's instant
    cache.insert(cache_key, (reply.clone(), usage.clone()));

    Ok((reply, usage))
}

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
    let mut cache: ResponseCache = HashMap::new();
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
            let (reply, usage) = ask_with_cache(&mut cache, &api_key, &pricing.model, &history)?;
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

                if prompt.eq_ignore_ascii_case("cache-test") {
                    let test_history = vec![
                        Message {
                            role: "system".to_string(),
                            content: "You are a cache test bot. Reply briefly.".to_string(),
                        },
                        Message {
                            role: "user".to_string(),
                            content: "Say exactly: CACHE OK".to_string(),
                        },
                    ];

                    println!("Cache test: first call");
                    let (first_reply, _) =
                        ask_with_cache(&mut cache, &api_key, &pricing.model, &test_history)?;

                    println!("\nCache test: second call");
                    let (second_reply, _) =
                        ask_with_cache(&mut cache, &api_key, &pricing.model, &test_history)?;

                    println!("\nCache test complete.");
                    println!("Same response? {}", first_reply == second_reply);
                    println!("Second reply: {second_reply}");

                    continue;
                }

                let user_message = Message {
                    role: "user".to_string(),
                    content: prompt,
                };

                history.push(user_message);

                truncate_history(&mut history, MAX_TOKENS);

                let input_tokens = estimate_tokens(&history);

                let (reply, usage) =
                    ask_with_cache(&mut cache, &api_key, &pricing.model, &history)?;

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
