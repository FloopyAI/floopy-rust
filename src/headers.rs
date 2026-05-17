//! Translate [`FloopyOptions`] into wire headers and merge header layers
//! with a deterministic precedence (later layers win).

use std::collections::HashMap;

use crate::constants::{
    HEADER_CACHE_BUCKET_MAX_SIZE, HEADER_CACHE_ENABLED, HEADER_LLM_SECURITY_ENABLED,
    HEADER_PROMPT_ID, HEADER_PROMPT_VERSION,
};
use crate::options::FloopyOptions;

pub(crate) fn build_floopy_headers(options: Option<&FloopyOptions>) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    let Some(options) = options else {
        return headers;
    };
    if let Some(cache) = &options.cache {
        if let Some(enabled) = cache.enabled {
            headers.insert(HEADER_CACHE_ENABLED.to_owned(), bool_str(enabled));
        }
        if let Some(size) = cache.bucket_max_size {
            headers.insert(HEADER_CACHE_BUCKET_MAX_SIZE.to_owned(), size.to_string());
        }
    }
    if let Some(prompt_id) = &options.prompt_id {
        headers.insert(HEADER_PROMPT_ID.to_owned(), prompt_id.clone());
    }
    if let Some(prompt_version) = &options.prompt_version {
        headers.insert(HEADER_PROMPT_VERSION.to_owned(), prompt_version.clone());
    }
    if let Some(enabled) = options.llm_security_enabled {
        headers.insert(HEADER_LLM_SECURITY_ENABLED.to_owned(), bool_str(enabled));
    }
    headers
}

/// Merge header layers with later layers winning. Each layer overwrites keys
/// already present.
pub(crate) fn merge_headers<'a, I>(layers: I) -> HashMap<String, String>
where
    I: IntoIterator<Item = &'a HashMap<String, String>>,
{
    let mut merged = HashMap::new();
    for layer in layers {
        for (k, v) in layer {
            merged.insert(k.clone(), v.clone());
        }
    }
    merged
}

/// Match the JS `String(boolean)` rendering used by the other SDKs.
fn bool_str(value: bool) -> String {
    if value { "true" } else { "false" }.to_owned()
}
