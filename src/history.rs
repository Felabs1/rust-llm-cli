use crate::models::Message;
const MESSAGES_PER_TURN: usize = 2;

pub fn estimate_tokens(messages: &[Message]) -> usize {
    messages
        .iter()
        .map(|message| message.content.len() / 4)
        .sum()
}

pub fn truncate_history(history: &mut Vec<Message>, max_tokens: usize) {
    while estimate_tokens(history) > max_tokens && history.len() > MESSAGES_PER_TURN + 1 {
        history.drain(1..=MESSAGES_PER_TURN);
    }
}

pub fn print_history(history: &[Message]) {
    println!("\n-------History--------");
    for (index, message) in history.iter().enumerate() {
        println!("[{}] {}: {}", index, message.role, message.content)
    }
    println!("\n----------------------------\n")
}

// Notice how the tests moved WITH the code!
#[cfg(test)]
mod tests {
    use super::*;

    fn msg(role: &str, content: &str) -> Message {
        Message {
            role: role.to_string(),
            content: content.to_string(),
        }
    }

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
}
