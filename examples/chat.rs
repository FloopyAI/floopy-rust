//! Basic chat completion — a drop-in for the `async-openai` crate.
//!
//! Run: `FLOOPY_API_KEY=fl_... cargo run --example chat`

use floopy::async_openai::types::chat::{
    ChatCompletionRequestSystemMessageArgs, ChatCompletionRequestUserMessageArgs,
    CreateChatCompletionRequestArgs,
};
use floopy::{CacheOptions, Floopy, FloopyOptions};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Floopy::builder(std::env::var("FLOOPY_API_KEY")?).options(FloopyOptions {
        cache: Some(CacheOptions {
            enabled: Some(true),
            bucket_max_size: Some(3),
        }),
        llm_security_enabled: Some(true),
        ..Default::default()
    });
    if let Ok(base) = std::env::var("FLOOPY_BASE_URL") {
        builder = builder.base_url(base);
    }
    let client = builder.build()?;

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .messages(vec![
            ChatCompletionRequestSystemMessageArgs::default()
                .content("You are a concise assistant.")
                .build()?
                .into(),
            ChatCompletionRequestUserMessageArgs::default()
                .content("Say hi from Floopy in one sentence.")
                .build()?
                .into(),
        ])
        .build()?;

    let response = client.openai().chat().create(request).await?;
    println!("{:?}", response.choices[0].message.content);
    Ok(())
}
