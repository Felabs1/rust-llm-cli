pub fn is_safe_prompt(prompt: &str) -> bool {
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

#[cfg(test)]
mod tests {
    use super::*;

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
