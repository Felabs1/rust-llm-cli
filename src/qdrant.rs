use reqwest::blocking::Client;
use serde_json::json;
use std::fs;

#[derive(serde::Deserialize)]
struct StoredEmbedding {
    data: String,
    vector: Vec<f32>,
}

pub fn create_collection(collection_name: &str, vector_size: usize) -> Result<(), Box<dyn std::error::Error>> {
    // creating a http client
    let client = Client::new();

    // building the url
    let url = format!(
        "http://localhost:6333/collections/{}",
        collection_name
    );

    // building the json payload
    let payload = json!({
        "vectors": {
            "size": vector_size,
            "distance": "Cosine"
        }
    });

    // send the put request to check the response
    client
        .put(&url)
        .json(&payload)
        .send()?
        .error_for_status()?;

    println!("✅ Collection '{}' created successfully.", collection_name);
    Ok(())
}

pub fn insert_points( file_path: &str, collection_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();

    // 1. Read and parse the JSON file
    println!("📖 Reading embeddings from {}...", file_path);
    let json_data = fs::read_to_string(file_path)?;

    let embeddings: Vec<StoredEmbedding> = serde_json::from_str(&json_data)?;
    // 2. Transform into Qdrant Points
    // .enumerate() gives us the index (i) and the item (emb)

    let points: Vec<serde_json::Value> = embeddings.into_iter().enumerate().map(|(i, emb)|{
        json!({
            "id": (i + 1) as u64, // qdrants need id > 0
            "vector": emb.vector,
            "payload": {"text": emb.data}
        })
    }).collect();

    // 3. Send the PUT request to Qdrant
    let url = format!("http://localhost:6333/collections/{}/points", collection_name);
    let payload = json!({ "points": points });

    println!("🚀 Pushing {} points to Qdrant...", points.len());
    client.put(&url).json(&payload).send()?.error_for_status()?;

    println!("✅ Successfully indexed {} points into '{}'", points.len(), collection_name);
    Ok(())
}