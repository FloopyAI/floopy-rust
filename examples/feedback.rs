//! Submit NPS-style feedback for a completed chat request.
//!
//! Run: `FLOOPY_API_KEY=fl_... cargo run --example feedback`

use floopy::async_openai::types::chat::{
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};
use floopy::types::FeedbackSubmitParams;
use floopy::Floopy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Floopy::builder(std::env::var("FLOOPY_API_KEY")?);
    if let Ok(base) = std::env::var("FLOOPY_BASE_URL") {
        builder = builder.base_url(base);
    }
    let client = builder.build()?;

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o-mini")
        .messages(vec![ChatCompletionRequestUserMessageArgs::default()
            .content("ping")
            .build()?
            .into()])
        .build()?;
    let response = client.openai().chat().create(request).await?;

    let fb = client
        .feedback()
        .submit(
            FeedbackSubmitParams {
                score: 9,
                useful: true,
                session_id: Some(response.id),
            },
            None,
        )
        .await?;
    println!(
        "feedback recorded (duplicate={} session={:?})",
        fb.duplicate, fb.session_id
    );
    Ok(())
}
