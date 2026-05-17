//! Read + full-replace the org spend/rate constraints.
//!
//! Run: `FLOOPY_API_KEY=fl_... cargo run --example constraints`

use floopy::types::OrgConstraints;
use floopy::Floopy;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Floopy::builder(std::env::var("FLOOPY_API_KEY")?);
    if let Ok(base) = std::env::var("FLOOPY_BASE_URL") {
        builder = builder.base_url(base);
    }
    let client = builder.build()?;

    let current = client.constraints().get(None).await?;
    println!("current monthly cap: {:?}", current.cost_limit_monthly_usd);

    // `put` is full-replace: any None field is cleared server-side.
    let updated = client
        .constraints()
        .put(
            &OrgConstraints {
                cost_limit_monthly_usd: Some(100.0),
                max_requests_per_minute: Some(120),
                ..Default::default()
            },
            None,
        )
        .await?;
    println!("new monthly cap: {:?}", updated.cost_limit_monthly_usd);
    Ok(())
}
