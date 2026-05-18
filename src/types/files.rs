use serde::Deserialize;

/// A file stored on the upstream provider. v1 targets the `batch` purpose
/// (JSONL input + output files). Mirrors the OpenAI file shape; the gateway
/// forwards file traffic verbatim.
#[derive(Debug, Clone, Deserialize)]
pub struct FileObject {
    /// Provider-issued file id.
    pub id: String,
    /// Object type (usually `"file"`).
    #[serde(default)]
    pub object: Option<String>,
    /// File size in bytes.
    #[serde(default)]
    pub bytes: Option<i64>,
    /// Unix creation timestamp.
    #[serde(default)]
    pub created_at: Option<i64>,
    /// Original filename.
    #[serde(default)]
    pub filename: Option<String>,
    /// Upload purpose (e.g. `"batch"`).
    #[serde(default)]
    pub purpose: Option<String>,
    /// Processing status.
    #[serde(default)]
    pub status: Option<String>,
    /// Present (and `true`) on a delete response.
    #[serde(default)]
    pub deleted: Option<bool>,
}

/// Response of [`crate::resources::Files::list`].
#[derive(Debug, Clone, Deserialize)]
pub struct FileList {
    /// Object type (usually `"list"`).
    #[serde(default)]
    pub object: Option<String>,
    /// The files.
    #[serde(default)]
    pub data: Vec<FileObject>,
}

/// Arguments for [`crate::resources::Files::upload`].
#[derive(Debug, Clone)]
pub struct FileUploadParams {
    /// Raw file content (forwarded verbatim as a multipart part).
    pub file: Vec<u8>,
    /// Filename used in the multipart part. Defaults to `"file"`.
    pub filename: Option<String>,
    /// Upload purpose. Use `"batch"` for batch input files.
    pub purpose: String,
}

/// Filters for [`crate::resources::Files::list`].
#[derive(Debug, Clone, Default)]
pub struct FileListParams {
    /// Restrict to a single purpose.
    pub purpose: Option<String>,
    /// Page size.
    pub limit: Option<u32>,
    /// Cursor: return files after this id.
    pub after: Option<String>,
}

impl FileListParams {
    pub(crate) fn query(&self) -> Vec<(String, String)> {
        let mut q = Vec::new();
        if let Some(p) = &self.purpose {
            q.push(("purpose".to_owned(), p.clone()));
        }
        if let Some(l) = self.limit {
            q.push(("limit".to_owned(), l.to_string()));
        }
        if let Some(a) = &self.after {
            q.push(("after".to_owned(), a.clone()));
        }
        q
    }
}
