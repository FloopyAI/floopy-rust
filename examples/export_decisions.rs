//! Stream the JSONL decision export and capture the trailer.
//!
//! Run: `FLOOPY_API_KEY=fl_... cargo run --example export_decisions`

use floopy::types::ExportDecisionsParams;
use floopy::Floopy;
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = Floopy::builder(std::env::var("FLOOPY_API_KEY")?);
    if let Ok(base) = std::env::var("FLOOPY_BASE_URL") {
        builder = builder.base_url(base);
    }
    let client = builder.build()?;

    let params = ExportDecisionsParams::new("2026-05-01T00:00:00Z", "2026-06-01T00:00:00Z");

    let mut stream = client.export().decisions_with_trailer(params, None);
    let mut count = 0usize;
    while let Some(row) = stream.next().await {
        row?;
        count += 1;
    }
    println!("exported {count} rows");
    if let Some(trailer) = stream.trailer() {
        println!(
            "trailer: emitted={} truncated={}",
            trailer.rows_emitted, trailer.truncated
        );
    }
    Ok(())
}
