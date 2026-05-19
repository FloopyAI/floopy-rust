# `floopy-sdk` (Rust)

> Official Floopy AI Gateway SDK for Rust. **Drop-in wrapper around the
> [`async-openai`](https://crates.io/crates/async-openai) crate** with
> Floopy's cache, audit, experiments, routing, and security on top.

[![crates.io](https://img.shields.io/crates/v/floopy-sdk?color=22c55e&label=crates.io)](https://crates.io/crates/floopy-sdk)
[![docs.rs](https://img.shields.io/docsrs/floopy-sdk?label=docs.rs)](https://docs.rs/floopy-sdk)
[![docs](https://img.shields.io/badge/docs-floopy.ai-blue)](https://floopy.ai/docs/guides/sdks/floopy-sdk-rust)

## Why

`floopy-sdk` wraps the official `async-openai` crate and points it at the
Floopy gateway, so:

- **Zero migration cost** for chat and embeddings — same types, same
  methods, via `client.openai()`.
- **Upstream updates** to `async-openai` reach you on `cargo update`
  without forks or parity drift.
- **Floopy-only features** (audit, experiments, constraints, decision
  export, feedback, routing dry-run, sessions) get **first-class typed,
  async methods** instead of hand-rolled `reqwest` calls.

## Install

```sh
cargo add floopy-sdk async-openai tokio --features tokio/macros,tokio/rt-multi-thread
```

The crate is published as `floopy-sdk` and imported as `floopy`. Requires
Rust `>= 1.82`. It is fully async (Tokio).

## Quick start

```rust,no_run
use floopy::Floopy;
use floopy::async_openai::types::chat::{
    ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Floopy::new(std::env::var("FLOOPY_API_KEY")?)?;

    let request = CreateChatCompletionRequestArgs::default()
        .model("gpt-4o")
        .messages(vec![ChatCompletionRequestUserMessageArgs::default()
            .content("Hello from Floopy!")
            .build()?
            .into()])
        .build()?;

    let response = client.openai().chat().create(request).await?;
    println!("{:?}", response.choices[0].message.content);
    Ok(())
}
```

`client.openai()` returns a lazily-built `async_openai::Client` pre-pointed
at the gateway; chat/embeddings/models behave exactly like upstream. The
wrapped crate is re-exported as `floopy::async_openai`, so you don't declare
it separately unless you want to.

### Migrating from `async-openai`

```diff
- use async_openai::{Client, config::OpenAIConfig};
- let client = Client::with_config(
-     OpenAIConfig::new().with_api_key(std::env::var("OPENAI_API_KEY")?),
- );
+ use floopy::Floopy;
+ let fl = Floopy::new(std::env::var("FLOOPY_API_KEY")?)?;
+ let client = fl.openai();

  let response = client.chat().create(request).await?;
```

## Floopy options (cache, prompt versioning, security firewall)

```rust,no_run
use floopy::{Floopy, FloopyOptions, CacheOptions};

# fn main() -> Result<(), Box<dyn std::error::Error>> {
let client = Floopy::builder(std::env::var("FLOOPY_API_KEY")?)
    .options(FloopyOptions {
        cache: Some(CacheOptions { enabled: Some(true), bucket_max_size: Some(3) }),
        prompt_id: Some("cd4249d5-44d5-46c8-8961-9eb3861e1f7e".into()),
        prompt_version: Some("1".into()),
        llm_security_enabled: Some(true),
    })
    .build()?;
# let _ = client;
# Ok(())
# }
```

These map to `Floopy-*` headers forwarded to **every** request (both
OpenAI-compat calls and Floopy-only ones). Per-call overrides go through a
trailing `RequestOptions` argument on every resource method.

| Option | Header | Purpose |
| --- | --- | --- |
| `cache.enabled` | `Floopy-Cache-Enabled` | Toggle exact + semantic cache |
| `cache.bucket_max_size` | `Floopy-Cache-Bucket-Max-Size` | Max entries per semantic bucket |
| `prompt_id` | `Floopy-Prompt-Id` | Stored prompt to resolve |
| `prompt_version` | `Floopy-Prompt-Version` | Pinned version for `prompt_id` |
| `llm_security_enabled` | `floopy-llm-security-enabled` | LLM firewall pre-check |

## Floopy-only resources

Each resource maps to a public `/v1/*` gateway endpoint and is typed
end-to-end. Errors are [`floopy::Error`] variants (see below). Pagination
and export are `futures::Stream`s.

```rust,no_run
# use floopy::Floopy;
# use floopy::types::*;
# use futures::StreamExt;
# async fn demo(client: Floopy) -> Result<(), Box<dyn std::error::Error>> {
// Feedback
client.feedback().submit(FeedbackSubmitParams { score: 9, useful: true, session_id: None }, None).await?;

// Decisions (paginated Streams)
let d = client.decisions().get("req_123", None).await?;
let stream = client.decisions().iter(DecisionListParams::default(), None);
futures::pin_mut!(stream);
while let Some(decision) = stream.next().await { let _ = decision?; }

// Experiments (auto X-Floopy-Confirm: experiments header)
let exp = client.experiments().create(ExperimentCreateParams {
    name: "cost-vs-quality".into(),
    variant_a_routing_rule_id: "rule_a".into(),
    variant_b_routing_rule_id: "rule_b".into(),
    description: None, split_percentage: None,
}, None).await?;
client.experiments().rollback(&exp.id, None).await?;

// Constraints (full-replace PUT)
client.constraints().put(&OrgConstraints { cost_limit_monthly_usd: Some(100.0), ..Default::default() }, None).await?;

// Export (streamed JSONL + trailer)
let mut export = client.export().decisions_with_trailer(ExportDecisionsParams::new("a", "b"), None);
while let Some(row) = export.next().await { let _ = row?; }
println!("{:?}", export.trailer());

// Evaluations
let run = client.evaluations().create(EvaluationCreateParams {
    dataset_id: "ds_1".into(), model: "gpt-4o".into(), prompt_id: None, config: None,
}, None).await?;
let _ = client.evaluations().results(&run.id, Some(100), None, None).await?;

// Routing dry-run (Pro plan)
let explain = client.routing().explain(RoutingExplainParams::new("gpt-4o", vec![]), None).await?;
let _ = (explain.would_select, explain.firewall_decision);

// Sessions — restore a stored conversation
let session = client.sessions().get("sess_1", None).await?;
let _ = session.messages;
# Ok(()) }
```

### Batch + Files

OpenAI-shaped Batch + Files passthrough. A batch carries no model up
front, so select the upstream with `RequestOptions::new().provider(...)`
(the `floopy-provider` header) — optional when the key has one provider.

```no_run
# use floopy::Floopy;
# use floopy::types::{BatchCreateParams, FileUploadParams};
# use floopy::RequestOptions;
# async fn run(client: Floopy) -> Result<(), Box<dyn std::error::Error>> {
let file = client.files().upload(
    FileUploadParams {
        file: std::fs::read("requests.jsonl")?,
        filename: Some("requests.jsonl".into()),
        purpose: "batch".into(),
    },
    RequestOptions::new().provider("openai"),
).await?;

let batch = client.batches().create(
    BatchCreateParams {
        input_file_id: file.id,
        endpoint: "/v1/chat/completions".into(),
        completion_window: "24h".into(),
        metadata: None,
    },
    RequestOptions::new().provider("openai"),
).await?;

let done = client.batches().retrieve(&batch.id, RequestOptions::new().provider("openai")).await?;
if done.status.as_deref() == Some("completed") {
    if let Some(out) = done.output_file_id {
        let bytes = client.files().content(&out, RequestOptions::new().provider("openai")).await?;
        let _ = bytes;
    }
}
# Ok(()) }
```

`files().list/retrieve/delete` and `batches().list/cancel` are also
available.

## Error handling

Every Floopy-only call returns `Result<T, floopy::Error>`:

```rust,no_run
# use floopy::{Error, Floopy};
# use floopy::types::DecisionListParams;
# async fn demo(client: Floopy) {
match client.decisions().list(&DecisionListParams::default(), None).await {
    Ok(page) => { let _ = page; }
    Err(Error::RateLimit(d)) => {
        let secs = d.retry_after_seconds.unwrap_or(1);
        tokio::time::sleep(std::time::Duration::from_secs(secs)).await;
    }
    Err(Error::Plan(d)) => eprintln!("upgrade plan: feature {:?} not in plan", d.feature),
    Err(e) => eprintln!("{e}"),
}
# }
```

`Error::status()`, `Error::request_id()`, `Error::feature()` and
`Error::retry_after_seconds()` are available on every variant. Errors from
chat/embeddings are emitted by `async-openai`
(`async_openai::error::OpenAIError`), not this crate.

## Security

- The API key is only ever sent in the `Authorization` header and is never
  rendered by `Debug`; the SDK never logs request or response bodies.
- TLS certificate verification is on by default (rustls + webpki-roots).
- Releases are immutable, signed crates.io publishes via **Trusted
  Publishing (OIDC)** — no long-lived registry token in this repo.

## Self-hosting / custom base URL

```rust,no_run
# use floopy::Floopy;
# fn main() -> Result<(), Box<dyn std::error::Error>> {
let client = Floopy::builder(std::env::var("FLOOPY_API_KEY")?)
    .base_url("https://gateway.internal.acme.com/v1")
    .build()?;
# let _ = client; Ok(()) }
```

## Links

- Full SDK guide: <https://floopy.ai/docs/guides/sdks/floopy-sdk-rust>
  ([Português](https://floopy.ai/pt/docs/guides/sdks/floopy-sdk-rust))
- API docs: <https://docs.rs/floopy-sdk>
- API reference: <https://floopy.ai/docs/guides/api-reference>
- Changelog: [`CHANGELOG.md`](./CHANGELOG.md)

## License

Apache-2.0 © Floopy
