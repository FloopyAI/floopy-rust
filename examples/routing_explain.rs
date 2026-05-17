//! Routing dry-run (Pro plan): see which provider/model the gateway would
//! pick — and the firewall verdict — without calling a provider.
//!
//! Run: `FLOOPY_API_KEY=fl_... cargo run --example routing_explain`

use floopy::async_openai::types::chat::ChatCompletionRequestUserMessageArgs;
use floopy::types::RoutingExplainParams;
use floopy::Floopy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Floopy::builder(std::env::var("FLOOPY_API_KEY")?);
    if let Ok(base) = std::env::var("FLOOPY_BASE_URL") {
        builder = builder.base_url(base);
    }
    let client = builder.build()?;

    let messages = vec![ChatCompletionRequestUserMessageArgs::default()
        .content("Summarise War and Peace in one line.")
        .build()?
        .into()];

    let res = client
        .routing()
        .explain(RoutingExplainParams::new("gpt-4o", messages), None)
        .await?;
    println!("firewall: {}", res.firewall_decision);
    match res.would_select {
        Some(sel) => println!(
            "would route to: {} / {}",
            sel.get("provider").map_or("?", String::as_str),
            sel.get("model").map_or("?", String::as_str)
        ),
        None => println!("blocked by firewall"),
    }
    Ok(())
}
