//! Batch embeddings, delegated to `async-openai`.
//!
//! Run: `FLOOPY_API_KEY=fl_... cargo run --example embeddings`

use floopy::async_openai::types::embeddings::CreateEmbeddingRequestArgs;
use floopy::Floopy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Floopy::builder(std::env::var("FLOOPY_API_KEY")?);
    if let Ok(base) = std::env::var("FLOOPY_BASE_URL") {
        builder = builder.base_url(base);
    }
    let client = builder.build()?;

    let request = CreateEmbeddingRequestArgs::default()
        .model("text-embedding-3-small")
        .input(vec!["floopy gateway".to_owned(), "rust sdk".to_owned()])
        .build()?;

    let response = client.openai().embeddings().create(request).await?;
    for (i, e) in response.data.iter().enumerate() {
        println!("embedding {i}: {} dimensions", e.embedding.len());
    }
    Ok(())
}
