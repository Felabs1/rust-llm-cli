use crate::ollama;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::fs;

// 1. A struct to bundle the text and its numbers together
// The <T> tells Rust: "This struct holds some type T, which we will define later."
#[derive(Serialize, Deserialize, Debug)]
pub struct Embedding<T> {
    pub data: T,          // this can be a custom struct, a string and anything
    pub vector: Vec<f32>, // the actual mathematical embedding
}

// A custom struct representing a richer document
#[derive(Serialize, Deserialize, Debug)]
pub struct Document {
    pub title: String,
    pub body: String,
}

#[derive(Serialize, Debug)]
struct EmbeddingRequest {
    model: String,
    input: String,
}

#[derive(Deserialize, Debug)]
struct EmbeddingResponse {
    data: Vec<EmbeddingData>,
    // Note: the api also returns model ans usage info
    // but we can ignore them for now to keep it simple
}

#[derive(Deserialize, Debug)]
struct EmbeddingData {
    // this is the embedding, a list of floating point numbers
    embedding: Vec<f32>,
    index: usize,
}

pub fn get_embedding(api_key: &str, text: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let client = Client::new();

    // 1. Changed model name to OpenRouter's format
    let payload = json!({
        "model": "openai/text-embedding-3-small",
        "input": text
    });

    // 2. Changed URL to OpenRouter's embeddings endpoint
    let response = client
        .post("https://openrouter.ai/api/v1/embeddings")
        .header("Authorization", format!("Bearer {}", api_key)) // Uses your existing OPENROUTER_API_KEY!
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()?
        .error_for_status()?;

    let embedding_response: EmbeddingResponse = response.json()?;

    let embedding_vector = embedding_response
        .data
        .into_iter()
        .next()
        .ok_or("No embedding data returned from API")?
        .embedding;

    Ok(embedding_vector)
}

pub fn chunk_text_by_paragraph(text: &str) -> Vec<String> {
    text.split("\n\n")
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty()) // remove empty chunks
        .collect()
}

// the master function to process the whole file
// 3. The master function to process a whole file
pub fn ingest_file(
    api_key: &str,
    file_path: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Reading file: {}", file_path);
    let content = fs::read_to_string(file_path)?;

    let chunks = chunk_text_by_paragraph(&content);
    println!("Split into {} chunks.", chunks.len());

    let mut embeddings_list: Vec<Embedding<String>> = Vec::new();

    // Loop through each chunk and get its embedding
    for (i, chunk) in chunks.iter().enumerate() {
        println!("🧠 [{}/{}] Generating embedding...", i + 1, chunks.len());
        let embedding_vector = get_embedding(api_key, chunk)?;

        embeddings_list.push(Embedding {
            data: chunk.clone(),
            vector: embedding_vector,
        });
    }

    let json_data = serde_json::to_string_pretty(&embeddings_list)?;
    fs::write(output_path, json_data)?;
    println!(
        "✅ Saved {} embeddings to {}",
        embeddings_list.len(),
        output_path
    );

    Ok(())
}

/// calculate the cosine similarity between two vectors
/// returns a value between -1.0 and 1.0
pub fn cosine_similarity(vec_a: &[f32], vec_b: &[f32]) -> f32 {
    // safety check: vectors must be the same length to be compared
    if vec_a.len() != vec_b.len() {
        return 0.0; // or panic, but 0.0 is safer for search ranking
    }

    // calculate the dot product
    // we zip two iterators together, multiply the pairs and sum it up
    let dot_product: f32 = vec_a.iter().zip(vec_b.iter()).map(|(a, b)| a * b).sum();

    // calculate magnitude of A
    // square each number, sum them then take squareroot
    let magnitude_a: f32 = vec_a.iter().map(|x| x * x).sum::<f32>().sqrt();

    // calculating the magnitude of b
    // square each number, sum them then take square_root
    let magnitude_b: f32 = vec_b.iter().map(|x| x * x).sum::<f32>().sqrt();

    // the final calculation
    // we check for zero to avoid dividing by zero which caused NAN errors
    let denominator = magnitude_a * magnitude_b;
    if denominator == 0.0 {
        return 0.0;
    }

    dot_product / denominator
}

pub fn benchmark_models(
    api_key: &str,
    query: &str,
    documents: &[&str],
) -> Result<(), Box<dyn std::error::Error>> {
    println!("Benchmarking Query: \"{}\"\n", query);
    // 1. Get the query embeddings for both models
    println!("Embedding query via API...");
    let api_query_vec = get_embedding(api_key, query)?;
    println!("Embedding query via Ollama...");
    let local_query_vec = ollama::get_local_embedding("nomic-embed-text", query)?;

    // 2. Embed all documents and calculate similarity scores
    let mut api_scores: Vec<(usize, f32)> = Vec::new();
    let mut local_scores: Vec<(usize, f32)> = Vec::new();

    for (i, doc) in documents.iter().enumerate() {
        let api_doc_vec = get_embedding(api_key, doc)?;
        let local_doc_vec = ollama::get_local_embedding("nomic-embed-text", doc)?;

        let api_score = cosine_similarity(&api_query_vec, &api_doc_vec);
        let local_score = cosine_similarity(&local_query_vec, &local_doc_vec);

        api_scores.push((i, api_score));
        local_scores.push((i, local_score));
    }

    // 3. Sort the scores from highest to lowest
    api_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
    local_scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // 4. Print the results
    println!("API Rankings (OpenRouter):");
    for (rank, (idx, score)) in api_scores.iter().enumerate() {
        println!("  {}. [Score: {:.3}] {}", rank + 1, score, documents[*idx]);
    }

    println!("\nLocal Rankings (Ollama):");
    for (rank, (idx, score)) in local_scores.iter().enumerate() {
        println!("  {}. [Score: {:.3}] {}", rank + 1, score, documents[*idx]);
    }
    println!("--------------------------------------------------\n");

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generic_embedding_works_with_custom_struct() {
        // create our custom document
        let doc = Document {
            title: "Rust Generics".to_string(),
            body: "generics allow flexible code".to_string(),
        };

        // 2. Wrap it in our generic Embedding struct!
        // Here, T is explicitly a Document.
        let my_embedding = Embedding {
            data: doc,
            vector: vec![0.1, 0.2, 0.3],
        };

        // 3. Prove it works
        assert_eq!(my_embedding.data.title, "Rust Generics");
        assert_eq!(my_embedding.vector.len(), 3);
    }

    #[test]
    fn cosine_similarity_identical_direction_is_one() {
        let v1 = vec![1.0, 2.0, 3.0];
        let v2 = vec![2.0, 4.0, 6.0]; // Same direction, just longer

        let similarity = cosine_similarity(&v1, &v2);

        // It should be exactly 1.0 (or extremely close due to float math)
        assert!((similarity - 1.0).abs() < 1e-5);
    }

    #[test]
    fn cosine_similarity_perpendicular_is_zero() {
        // In 2D space, [1, 0] and [0, 1] are perpendicular (90 degrees)
        let v1 = vec![1.0, 0.0];
        let v2 = vec![0.0, 1.0];

        let similarity = cosine_similarity(&v1, &v2);

        // Cosine of 90 degrees is 0
        assert!((similarity - 0.0).abs() < 1e-5);
    }
}
