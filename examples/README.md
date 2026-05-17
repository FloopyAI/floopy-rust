# `floopy-sdk` examples

Runnable Cargo examples for every public surface of the SDK. They are
`exclude`d from the published crate.

## Setup

```sh
# from floopy-rust/
cp examples/.env.example examples/.env
set -a; source examples/.env; set +a
cargo run --example chat
```

By default examples talk to `https://api.floopy.ai/v1`. To point at a local
gateway, set `FLOOPY_BASE_URL=http://localhost:8000/v1`.

## Files

| Example | What it shows |
| --- | --- |
| `chat` | Basic chat completion (drop-in for `async-openai`) |
| `chat_stream` | Streaming response |
| `embeddings` | Batch embeddings |
| `feedback` | Submit NPS-style feedback |
| `decisions_list` | List + paginate decisions (`Stream`) |
| `export_decisions` | Stream the JSONL export + trailer |
| `experiments_create` | Create + roll back an experiment |
| `constraints` | Read + full-replace org constraints |
| `routing_explain` | Routing dry-run (Pro plan) |
