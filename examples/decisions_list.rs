//! List + paginate the decision audit trail.
//!
//! Run: `FLOOPY_API_KEY=fl_... cargo run --example decisions_list`

use floopy::types::DecisionListParams;
use floopy::Floopy;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Floopy::builder(std::env::var("FLOOPY_API_KEY")?);
    if let Ok(base) = std::env::var("FLOOPY_BASE_URL") {
        builder = builder.base_url(base);
    }
    let client = builder.build()?;

    let params = DecisionListParams {
        limit: Some(20),
        ..Default::default()
    };

    // One page:
    let page = client.decisions().list(&params, None).await?;
    println!(
        "first page: {} decisions (has_more={})",
        page.items.len(),
        page.has_more
    );

    // Every decision across all pages:
    let mut count = 0usize;
    let stream = client.decisions().iter(DecisionListParams::default(), None);
    futures::pin_mut!(stream);
    while let Some(decision) = stream.next().await {
        decision?;
        count += 1;
    }
    println!("total decisions: {count}");
    Ok(())
}
