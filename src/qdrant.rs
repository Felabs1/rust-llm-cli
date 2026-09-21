use reqwest::blocking::Client;
use serde_json::json;
use std::fs;

use crate::retriever::{RetrievalResult, Retriever};

pub struct QdrantClient {
    pub base_url: String,
}

#[derive(serde::Deserialize, Debug)]
pub struct SearchResponse {
    pub result: Vec<SearchResult>,
}

#[derive(serde::Deserialize, Debug)]
pub struct SearchResult {
    pub id: u64,
    pub score: f32,
    pub payload: SearchResultPayload,
}

#[derive(serde::Deserialize, Debug)]
pub struct SearchResultPayload {
    pub text: String,
}

#[derive(serde::Deserialize)]
pub struct StoredEmbedding {
    data: String,
    vector: Vec<f32>,
}

impl QdrantClient {
    /// Create a new Qdrant client pointing to the local Docker instance.
    pub fn new() -> Self {
        Self {
            base_url: "http://localhost:6333".to_string(),
        }
    }

    /// Create a collection in Qdrant.
    pub fn create_collection(
        &self,
        collection_name: &str,
        vector_size: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::new();
        let url = format!("{}/collections/{}", self.base_url, collection_name);

        let payload = serde_json::json!({
            "vectors": {
                "size": vector_size,
                "distance": "Cosine"
            }
        });

        let response = client.put(&url).json(&payload).send()?;
        let status = response.status();

        // 200/201 = created, 409 = already exists (both are fine!)
        if status.is_success() || status.as_u16() == 409 {
            println!("✅ Collection '{}' ready.", collection_name);
            Ok(())
        } else {
            // Some other error occurred, so we return it
            Err(response.error_for_status().unwrap_err().into())
        }
    }

    /// Insert embedded points into Qdrant.
    pub fn insert_points(
        &self,
        collection_name: &str,
        embeddings: &[(String, Vec<f32>)],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::blocking::Client::new();

        let points: Vec<serde_json::Value> = embeddings
            .iter()
            .enumerate()
            .map(|(i, (text, vector))| {
                serde_json::json!({
                    "id": (i + 1) as u64,
                    "vector": vector,
                    "payload": { "text": text }
                })
            })
            .collect();

        let url = format!("{}/collections/{}/points", self.base_url, collection_name);

        let payload = serde_json::json!({ "points": points });

        client.put(&url).json(&payload).send()?.error_for_status()?;

        println!(
            "✅ Indexed {} points into '{}'.",
            points.len(),
            collection_name
        );
        Ok(())
    }

    /// Load embeddings from a JSON file and index them into Qdrant.
    pub fn insert_points_from_file(
        &self,
        collection_name: &str,
        file_path: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let json_data = std::fs::read_to_string(file_path)?;
        let stored: Vec<StoredEmbedding> = serde_json::from_str(&json_data)?;

        let embeddings: Vec<(String, Vec<f32>)> =
            stored.into_iter().map(|e| (e.data, e.vector)).collect();

        self.insert_points(collection_name, &embeddings)
    }
}

pub fn search_points(
    collection_name: &str,
    query_vector: Vec<f32>,
    limit: usize,
    keyword_filter: Option<&str>,
) -> Result<Vec<SearchResult>, Box<dyn std::error::Error>> {
    let client = reqwest::blocking::Client::new();

    // Build the search URL
    let url = format!(
        "http://localhost:6333/collections/{}/points/search",
        collection_name
    );

    // Build the request body
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
        })
    }

    // Send the POST request
    let response = client
        .post(&url)
        .json(&payload)
        .send()?
        .error_for_status()?;

    // Parse the JSON response into our structs
    let search_response: SearchResponse = response.json()?;

    Ok(search_response.result)
}
