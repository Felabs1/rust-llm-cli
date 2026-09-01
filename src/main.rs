mod cache;
mod client;
mod commands;
mod config;
mod cost;
mod history;
mod models;
mod safety;

use cache::{ResponseCache, ask_with_cache};
use cost::{calculate_cost, estimate_cost};
use history::{estimate_tokens, print_history, truncate_history};
use models::Message;
use models::Pricing;
use models::Usage;
use safety::is_safe_prompt;
use std::collections::HashMap;
use client::{LanguageModel, OpenRouterClient};

const MAX_TOKENS: usize = 500;

fn process_turn<M: LanguageModel>(
    client: &M,
    cache: &mut ResponseCache,
    api_key: &str,
    pricing: &Pricing,
    history: &mut Vec<Message>,
    prompt: String,
) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Add user message
    let user_message = Message {
        role: "user".to_string(),
        content: prompt,
    };
    history.push(user_message);

    // 2. Truncate if needed
    history::truncate_history(history, MAX_TOKENS);

    // 3. Estimate tokens and call API (via cache)
    let input_tokens = history::estimate_tokens(history);
    let (reply, usage) = ask_with_cache(client, cache, api_key, &pricing.model, history)?;

    // 4. Calculate usage and cost
    let output_tokens = reply.len() / 4;
    let _total_tokens = input_tokens + output_tokens;
    let (_total_cost, used_actual_usage) =
        calculate_cost(usage.as_ref(), input_tokens, output_tokens, pricing);

    // 5. Print response and cost source
    println!("\n\nComplete response:");
    println!("{reply}");
    println!("pricing model: {}", pricing.model);
    println!(
        "cost source: {}",
        if used_actual_usage {
            "provider usage"
        } else {
            "estimated tokens"
        }
    );

    // 6. Add assistant response to history
    history.push(Message {
        role: "assistant".to_string(),
        content: reply,
    });

    // Optional: Print history after every turn
    // history::print_history(history);

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cache: ResponseCache = HashMap::new();
    let api_key = config::api_key()?;
    let cli = commands::parse_commands()?;
    let system_prompt = config::system_prompt(&cli.system_prompt)?;
    let pricing = config::pricing()?;

    let llm = OpenRouterClient;

    match cli.command {
        commands::Commands::Ask { prompt } => {
            let mut history: Vec<Message> = vec![Message {
                role: "system".to_string(),
                content: system_prompt,
            }];

            let mut redo_stack: Vec<Message> = Vec::new();

            process_turn(&llm, &mut cache, &api_key, &pricing, &mut history, prompt)?;

            // print_history(&history);

            loop {
                let prompt = commands::read_prompt()?;

                if prompt.eq_ignore_ascii_case("exit") {
                    break;
                }

                if !is_safe_prompt(&prompt) {
                    println!("\n BLOCKED: Prompt injection detected.\n");
                    continue;
                }

                if prompt.eq_ignore_ascii_case("undo") {
                    // handle undo logic
                    if history.len() >= 3 {
                        // pop assistant first if it's on top
                        if let Some(assistant_msg) = history.pop() {
                            if let Some(user_msg) = history.pop() {
                                // push them onto redo_stack (assistant first, then user)
                                redo_stack.push(assistant_msg);
                                redo_stack.push(user_msg);
                                println!("undid last turn");
                            } else {
                                history.push(assistant_msg);
                                println!("nothing to undo");
                            }
                        }
                    } else {
                        println!("Nothing to undo.");
                    }
                    print_history(&history);

                    continue;
                }

                if prompt.eq_ignore_ascii_case("redo") {
                    if redo_stack.len() >= 2 {
                        // popo user first (since is't on top of the redo stack)
                        if let Some(user_msg) = redo_stack.pop() {
                            // pop assistant next
                            if let Some(assistant_msg) = redo_stack.pop() {
                                // push them back on to history (user first, then assistant)
                                history.push(user_msg);
                                history.push(assistant_msg);
                                println!("REdid last turn.");
                            } else {
                                // edge case: put user back if assistant pop failed
                                redo_stack.push(user_msg);
                                println!("nothing to redo");
                            }
                        }
                    } else {
                        println!("Nothing to redo.");
                    }

                    print_history(&history);

                    continue;
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
                        ask_with_cache(&llm, &mut cache, &api_key, &pricing.model, &test_history)?;

                    println!("\nCache test: second call");
                    let (second_reply, _) =
                        ask_with_cache(&llm, &mut cache, &api_key, &pricing.model, &test_history)?;

                    println!("\nCache test complete.");
                    println!("Same response? {}", first_reply == second_reply);
                    println!("Second reply: {second_reply}");

                    continue;
                }

                redo_stack.clear();

                process_turn(&llm, &mut cache, &api_key, &pricing, &mut history, prompt)?;
            }
        }
    }

    Ok(())
}
