// src/cache.rs

use std::collections::HashMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::client::LanguageModel;
use crate::models::{Message, Usage};

// We make this public so main.rs can create the HashMap
pub type ResponseCache = HashMap<u64, (String, Option<Usage>)>;

pub fn ask_with_cache<M: LanguageModel>(
    client: &M,
    cache: &mut ResponseCache,
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
    let (reply, usage) = client.ask(api_key, model, history)?;

    // 4. Store the result so next time it's instant
    cache.insert(cache_key, (reply.clone(), usage.clone()));

    Ok((reply, usage))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::Message;

    struct MockClient;

    impl LanguageModel for MockClient {
       fn ask(
        &self,
        _api_key: &str,
        _model: &str,
        _messages: &[Message],
       ) -> Result<(String, Option<Usage>), Box<dyn std::error::Error>> {
        Ok(("This is a fake mocked response.".to_string(), None))
       }
    }

    // 3. Test the cache using the MockClient
    #[test]
    fn cache_returns_stored_response_on_second_call() {
        let mut cache = HashMap::new();
        let client = MockClient;
        
        let history = vec![Message {
            role: "user".to_string(),
            content: "hello".to_string(),
        }];

        // First call: cache miss, hits the mock
        let (reply1, _) = ask_with_cache(&client, &mut cache, "fake_key", "fake_model", &history).unwrap();
        assert_eq!(reply1, "This is a fake mocked response.");
        assert_eq!(cache.len(), 1); // Cache now has 1 item

        // Second call: cache hit, returns stored value WITHOUT calling the mock
        let (reply2, _) = ask_with_cache(&client, &mut cache, "fake_key", "fake_model", &history).unwrap();
        assert_eq!(reply2, "This is a fake mocked response.");
        assert_eq!(cache.len(), 1); // Cache STILL has exactly 1 item
    }

}
