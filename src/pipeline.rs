use crate::embeddings;
use crate::qdrant::{self, QdrantClient};
use crate::retriever::Retriever;
use std::fs;

// the full pipeline, read, chunk, embedd store in qdrant
pub fn run_pipeline(
    file_path: &str,
    collection_name: &str,
    api_key: &str,
) -> Result<usize, Box<dyn std::error::Error>> {
    println!("Reading file: {}", file_path);
    let content = fs::read_to_string(file_path)?;

    let chunks: Vec<&str> = content
        .split("\n\n")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    println!("Found {} chunks to embed.", chunks.len());

    // embedding each chunk
    let mut embeddings: Vec<(String, Vec<f32>)> = Vec::new();
    for (i, chunk) in chunks.iter().enumerate() {
        println!("  Embedding chunk {}/{}...", i + 1, chunks.len());
        let vector = embeddings::get_embedding(api_key, chunk)?;
        embeddings.push((chunk.to_string(), vector));
    }

    // Ensure the collection exists in Qdrant
    println!("📦 Setting up Qdrant collection: {}", collection_name);
    let client = QdrantClient::new();
    client.create_collection(collection_name, 1536)?;

    // Step 5: Build and upsert the points
    println!("🚀 Indexing {} points into Qdrant...", embeddings.len());
    client.insert_points(collection_name, &embeddings)?;

    println!(
        "✅ Pipeline complete! {} chunks are now searchable.",
        chunks.len()
    );
    Ok(chunks.len())
}
