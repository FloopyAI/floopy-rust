use std::sync::Arc;

use reqwest::multipart::{Form, Part};
use reqwest::Method;

use crate::constants::{file_by_id, file_content, ENDPOINT_FILES};
use crate::error::Result;
use crate::http::HttpTransport;
use crate::options::RequestOptions;
use crate::types::{FileList, FileListParams, FileObject, FileUploadParams};

use super::require;

/// Files API — upload/list/retrieve/delete files and download content.
///
/// v1 targets the `batch` purpose (JSONL input + output files). Select the
/// upstream with [`RequestOptions::provider`] (the `floopy-provider`
/// header).
pub struct Files {
    t: Arc<HttpTransport>,
}

impl Files {
    pub(crate) fn new(t: Arc<HttpTransport>) -> Self {
        Self { t }
    }

    /// Upload a file as `multipart/form-data`.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn upload(
        &self,
        params: FileUploadParams,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<FileObject> {
        let filename = params.filename.unwrap_or_else(|| "file".to_owned());
        let part = Part::bytes(params.file).file_name(filename);
        let form = Form::new()
            .text("purpose", params.purpose)
            .part("file", part);
        let (data, _) = self
            .t
            .request_multipart(ENDPOINT_FILES, form, req.into().as_ref())
            .await?;
        require(data)
    }

    /// List files, optionally filtered by purpose.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn list(
        &self,
        params: FileListParams,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<FileList> {
        let (data, _) = self
            .t
            .request(
                Method::GET,
                ENDPOINT_FILES,
                None,
                &params.query(),
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }

    /// Retrieve a single file's metadata.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn retrieve(
        &self,
        file_id: &str,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<FileObject> {
        let (data, _) = self
            .t
            .request(
                Method::GET,
                &file_by_id(file_id),
                None,
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }

    /// Download raw file content (e.g. a batch output/error JSONL).
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn content(
        &self,
        file_id: &str,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<Vec<u8>> {
        let (bytes, _) = self
            .t
            .request_bytes(
                Method::GET,
                &file_content(file_id),
                &[],
                req.into().as_ref(),
            )
            .await?;
        Ok(bytes)
    }

    /// Delete a file.
    ///
    /// # Errors
    /// Returns an [`Error`](crate::Error) on a non-2xx response or transport
    /// failure.
    pub async fn delete(
        &self,
        file_id: &str,
        req: impl Into<Option<RequestOptions>>,
    ) -> Result<FileObject> {
        let (data, _) = self
            .t
            .request(
                Method::DELETE,
                &file_by_id(file_id),
                None,
                &[],
                req.into().as_ref(),
            )
            .await?;
        require(data)
    }
}
