//! Streaming chat completion. Streaming is delegated to `async-openai`.
//!
//! Run: `FLOOPY_API_KEY=fl_... cargo run --example chat_stream`

use std::io::Write;

use floopy::async_openai::types::chat::{
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use floopy::Floopy;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = build_client()?;

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .messages(vec![ChatCompletionRequestUserMessageArgs::default()
            .content("Write a short haiku about gateways.")
            .build()?
            .into()])
        .stream(true)
        .build()?;

    let mut stream = client.openai().chat().create_stream(request).await?;
    while let Some(item) = stream.next().await {
        let chunk = item?;
        for choice in &chunk.choices {
            if let Some(content) = &choice.delta.content {
                print!("{content}");
                std::io::stdout().flush().ok();
            }
        }
    }
    println!();
    Ok(())
}

fn build_client() -> Result<Floopy, Box<dyn std::error::Error>> {
    let mut builder = Floopy::builder(std::env::var("FLOOPY_API_KEY")?);
    if let Ok(base) = std::env::var("FLOOPY_BASE_URL") {
        builder = builder.base_url(base);
    }
    Ok(builder.build()?)
}
