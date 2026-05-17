//! Official Floopy AI Gateway SDK for Rust.
//!
//! `floopy-sdk` wraps the official [`async_openai`] crate and points it at
//! the Floopy gateway, so `chat`/`embeddings`/`models` stay a 1:1 drop-in
//! replacement, and adds typed Floopy-only resources (feedback, decisions,
//! experiments, constraints, decision export, evaluations, routing dry-run,
//! sessions) on top. It mirrors the Node, Python and Go SDKs so behaviour
//! stays in lockstep across languages.
//!
//! ```no_run
//! use floopy::Floopy;
//! use async_openai::types::chat::{
//!     ChatCompletionRequestUserMessageArgs, CreateChatCompletionRequestArgs,
//! };
//!
//! # async fn run() -> Result<(), Box<dyn std::error::Error>> {
//! let client = Floopy::new(std::env::var("FLOOPY_API_KEY")?)?;
//!
//! let request = CreateChatCompletionRequestArgs::default()
//!     .model("gpt-4o")
//!     .messages(vec![ChatCompletionRequestUserMessageArgs::default()
//!         .content("Hello from Floopy!")
//!         .build()?
//!         .into()])
//!     .build()?;
//!
//! let response = client.openai().chat().create(request).await?;
//! println!("{:?}", response.choices[0].message.content);
//! # Ok(())
//! # }
//! ```
//!
//! Every Floopy-only call returns [`Result<T, Error>`](Error). Errors from
//! the OpenAI-compatible surface come from [`async_openai`] instead.

#![deny(missing_docs)]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod client;
mod constants;
mod error;
mod headers;
mod http;
mod openai_delegate;
mod options;
pub mod resources;
pub mod types;

pub use client::{Floopy, FloopyBuilder};
pub use constants::{CONFIRM_EXPERIMENTS, DEFAULT_BASE_URL, DEFAULT_MAX_RETRIES, DEFAULT_TIMEOUT};
pub use error::{Error, ErrorDetails, Result};
pub use options::{CacheOptions, FloopyOptions, RequestOptions};

// Re-export the wrapped OpenAI crate so consumers can build chat/embedding
// requests without a separate `async-openai` dependency declaration.
pub use async_openai;
