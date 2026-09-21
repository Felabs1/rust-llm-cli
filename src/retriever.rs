use crate::qdrant::{QdrantClient, SearchResponse};

/// The result of a search operation.
/// We define this here instead of in qdrant.rs because
/// ANY backend (Qdrant, Pinecone, Weaviate) will return this same shape.

#[derive(Debug)]
pub struct RetrievalResult {
    pub text: String,
    pub score: f32,
}

/// The contract that any vector database backend must fulfill.
/// Think of this as the wall socket. Any plug that fits can be used.
pub trait Retriever {
    fn search(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        keyword_filter: Option<&str>,
    ) -> Result<Vec<RetrievalResult>, Box<dyn std::error::Error>>;
}

impl Retriever for QdrantClient {
    fn search(
        &self,
        collection_name: &str,
        query_vector: Vec<f32>,
        limit: usize,
        keyword_filter: Option<&str>,
    ) -> Result<Vec<RetrievalResult>, Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::new();
        let url = format!(
            "{}/collections/{}/points/search",
            self.base_url, collection_name
        );
        let mut payload = serde_json::json!({
            "vector": query_vector,
            "limit": limit,
            "with_payload": true
        });

        if let Some(keyword) = keyword_filter {
            payload["filter"] = serde_json::json!({
                "must": [
                    {
                        "key": "text",
                        "match": {
                            "text": keyword
                        }
                    }
                ]
            });
        }

        let response = client
            .post(&url)
            .json(&payload)
            .send()?
            .error_for_status()?;

        let search_response: SearchResponse = response.json()?;

        let results = search_response
            .result
            .into_iter()
            .map(|sr| RetrievalResult {
                text: sr.payload.text,
                score: sr.score,
            })
            .collect();

        Ok(results)
    }
}
