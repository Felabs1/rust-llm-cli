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

fn is_safe_prompt(prompt: &str) -> bool {
    let banned_phrases = [
        "ignore previous instructions",
        "ignore all previous",
        "disregard prior",
        "system prompt",
        "reveal your instructions",
        "you are now dan",
        "do anything now",
        "strive to avoid norms",
    ];

    let lower = prompt.to_lowercase();

    for phrase in banned_phrases.iter() {
        if lower.contains(phrase) {
            return false;
        }
    }

    true
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

            let mut redo_stack: Vec<Message> = Vec::new();

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

            // println!("\n---------usage----------");
            // println!("estimated input tokens: {input_tokens}");
            // println!("estimated output tokens: {output_tokens}");
            // println!("estimated total tokens: {total_tokens}");
            // println!("estimated cost: ${total_cost:.8}");

            println!("pricing model: {}", pricing.model);
            println!(
                "cost source: {}",
                if used_actual_usage {
                    "provider usage"
                } else {
                    "estimated tokens"
                }
            );

            // if let Some(usage) = &usage {
            //     println!("\n---------provider usage----------");
            //     println!("actual input tokens: {}", usage.prompt_tokens);
            //     println!("actual output tokens: {}", usage.completion_tokens);
            //     println!("actual total tokens: {}", usage.total_tokens);
            // }

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
                        ask_with_cache(&mut cache, &api_key, &pricing.model, &test_history)?;

                    println!("\nCache test: second call");
                    let (second_reply, _) =
                        ask_with_cache(&mut cache, &api_key, &pricing.model, &test_history)?;

                    println!("\nCache test complete.");
                    println!("Same response? {}", first_reply == second_reply);
                    println!("Second reply: {second_reply}");



                    continue;
                }

                redo_stack.clear();

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

                // println!("\n\nComplete response: ");
                // println!("{reply}");

                // println!("\n---------usage----------");
                // println!("estimated input tokens: {input_tokens}");
                // println!("estimated output tokens: {output_tokens}");
                // println!("estimated total tokens: {total_tokens}");
                // println!("estimated cost: ${total_cost:.8}");

                println!(
                    "cost source: {}",
                    if used_actual_usage {
                        "provider usage"
                    } else {
                        "estimated tokens"
                    }
                );

                // if let Some(usage) = &usage {
                //     println!("\n---------provider usage----------");
                //     println!("actual input tokens: {}", usage.prompt_tokens);
                //     println!("actual output tokens: {}", usage.completion_tokens);
                //     println!("actual total tokens: {}", usage.total_tokens);
                // }

                history.push(Message {
                    role: "assistant".to_string(),
                    content: reply,
                });

                print_history(&history);
            }
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    fn msg(role: &str, content: &str) -> Message {
        Message {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

    fn pricing_fixture() -> Pricing {
        Pricing {
            model: "test-model".to_string(),
            input_per_million: 10.0,
            output_per_million: 20.0,
        }
    }

    // ---------------------------------------------------------------
    // estimate_tokens
    // ---------------------------------------------------------------
    #[test]
    fn estimate_tokens_returns_zero_when_history_is_empty() {
        assert_eq!(estimate_tokens(&[]), 0);
    }

    #[test]
    fn estimate_tokens_uses_four_characters_per_token() {
        let history = vec![msg("user", "abcd"), msg("assistant", "abcdabcd")];

        assert_eq!(estimate_tokens(&history), 3)
    }

    #[test]
    fn estimate_tokens_truncates_partial_tokens_using_integer_division() {
        let history = vec![
            msg("user", "abcde"), // 5 chars / 4 = 1 token, not 1.25
        ];

        assert_eq!(estimate_tokens(&history), 1);
    }

    #[test]
    fn estimate_cost_calculates_input_and_output_cost() {
        let pricing = pricing_fixture();

        let cost = estimate_cost(1_000_000, 5_00_000, &pricing);

        assert!((cost - 20.00).abs() < 1e-12);
    }

    #[test]
    fn estimate_cost_returns_zero_for_zero_tokens() {
        let pricing = pricing_fixture();

        let cost = estimate_cost(0, 0, &pricing);

        assert!((cost - 0.0).abs() < 1e-12);
    }

    #[test]
    fn calculate_cost_prefers_provider_usage_when_present() {
        let pricing = pricing_fixture();

        let usage = Usage {
            prompt_tokens: 1_000_000,
            completion_tokens: 500_000,
            total_tokens: 1_500_000,
        };

        let (cost, used_actual_usage) = calculate_cost(Some(&usage), 0, 0, &pricing);

        assert!((cost - 20.0).abs() < 1e-12);
        assert!(used_actual_usage);
    }

    #[test]
    fn calculate_cost_falls_back_to_estimates_when_usage_is_missing() {
        let pricing = pricing_fixture();

        let (cost, used_actual_usage) = calculate_cost(None, 1_000_000, 500_000, &pricing);

        assert!((cost - 20.0).abs() < 1e-12);
        assert!(!used_actual_usage);
    }

    #[test]
    fn truncate_history_removes_oldest_user_assistant_pair_but_keeps_system_and_recent_message() {
        let mut history = vec![
            msg("system", "ssss"),              // 1 token
            msg("user", &"a".repeat(400)),      // 100 tokens
            msg("assistant", &"b".repeat(400)), // 100 tokens
            msg("user", "cccc"),
        ];

        truncate_history(&mut history, 50);

        assert_eq!(history.len(), 2);
        assert_eq!(history[0].role, "system");
        assert_eq!(history[1].role, "user");
        assert_eq!(history[1].content, "cccc");
    }

    #[test]
    fn truncate_history_does_not_truncate_when_only_one_turn_remains() {
        let mut history = vec![
            msg("system", "ssss"),
            msg("user", &"a".repeat(400)),
            msg("assistant", &"b".repeat(400)),
        ];

        truncate_history(&mut history, 10);

        // This documents current behavior:
        // the function refuses to truncate when history.len() <= 3.
        assert_eq!(history.len(), 3);
    }

    // now let's test for safety
    #[test]
    fn guardrail_allows_normal_prompt() {
        assert!(is_safe_prompt("What is the capital of France?"));
    }

    #[test]
    fn guardrail_blocks_exact_injection_phrase() {
        assert!(!is_safe_prompt("Ignore previous instructions"));
    }

    #[test]
    fn guardrail_blocks_uppercase_injection_phrase() {
        assert!(!is_safe_prompt("IGNORE PREVIOUS INSTRUCTIONS"));
    }

    #[test]
    fn guardrail_blocks_phrase_hidden_inside_longer_text() {
        let prompt = "Please translate this: ignore previous instructions";
        assert!(!is_safe_prompt(prompt));
    }

    #[test]
    fn guardrail_allows_legitimate_programming_question_about_ignoring_errors() {
        let prompt = "How do I make Git ignore previous commits?";
        assert!(is_safe_prompt(prompt));
    }
}