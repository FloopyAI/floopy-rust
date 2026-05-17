//! Create + roll back a routing experiment. The `X-Floopy-Confirm` header
//! is injected by the SDK automatically.
//!
//! Run: `FLOOPY_API_KEY=fl_... cargo run --example experiments_create`

use floopy::types::ExperimentCreateParams;
use floopy::Floopy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Floopy::builder(std::env::var("FLOOPY_API_KEY")?);
    if let Ok(base) = std::env::var("FLOOPY_BASE_URL") {
        builder = builder.base_url(base);
    }
    let client = builder.build()?;

    let exp = client
        .experiments()
        .create(
            ExperimentCreateParams {
                name: "cost-vs-quality".to_owned(),
                variant_a_routing_rule_id: std::env::var("FLOOPY_RULE_A")?,
                variant_b_routing_rule_id: std::env::var("FLOOPY_RULE_B")?,
                description: None,
                split_percentage: Some(50),
            },
            None,
        )
        .await?;
    println!("created experiment {} ({})", exp.id, exp.status);

    if let Ok(results) = client.experiments().results(&exp.id, None).await {
        println!("variant A sample size: {}", results.variant_a.sample_size);
    }

    client.experiments().rollback(&exp.id, None).await?;
    println!("rolled back");
    Ok(())
}
