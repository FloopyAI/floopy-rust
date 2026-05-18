//! Batch + Files API integration tests against a mock gateway (wiremock).

use floopy::types::{BatchCreateParams, BatchListParams, FileListParams, FileUploadParams};
use floopy::{Error, Floopy, RequestOptions};
use serde_json::json;
use wiremock::matchers::{body_string_contains, header, method, path, query_param};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> Floopy {
    Floopy::builder("fl_test")
        .base_url(format!("{}/v1", server.uri()))
        .max_retries(0)
        .build()
        .expect("client builds")
}

#[tokio::test]
async fn files_upload_multipart_with_provider() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/files"))
        .and(header("floopy-provider", "openai"))
        .and(body_string_contains("name=\"purpose\""))
        .and(body_string_contains("batch"))
        .and(body_string_contains("filename=\"in.jsonl\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "file-1", "object": "file", "purpose": "batch", "status": "processed"
        })))
        .mount(&server)
        .await;

    let c = client(&server);
    let res = c
        .files()
        .upload(
            FileUploadParams {
                file: b"{\"x\":1}\n".to_vec(),
                filename: Some("in.jsonl".to_owned()),
                purpose: "batch".to_owned(),
            },
            RequestOptions::new().provider("openai"),
        )
        .await
        .expect("upload ok");
    assert_eq!(res.id, "file-1");
    assert_eq!(res.purpose.as_deref(), Some("batch"));
}

#[tokio::test]
async fn files_upload_default_filename() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/files"))
        .and(body_string_contains("filename=\"file\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "file-2" })))
        .mount(&server)
        .await;
    let c = client(&server);
    let res = c
        .files()
        .upload(
            FileUploadParams {
                file: b"x".to_vec(),
                filename: None,
                purpose: "batch".to_owned(),
            },
            None,
        )
        .await
        .expect("upload ok");
    assert_eq!(res.id, "file-2");
}

#[tokio::test]
async fn files_list_get_content_delete() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/files"))
        .and(query_param("purpose", "batch"))
        .and(query_param("limit", "10"))
        .and(query_param("after", "f0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list", "data": [{ "id": "f1" }]
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/files/file-1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "file-1" })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/files/file-1/content"))
        .respond_with(ResponseTemplate::new(200).set_body_string("{\"a\":1}\n"))
        .mount(&server)
        .await;
    Mock::given(method("DELETE"))
        .and(path("/v1/files/file-1"))
        .respond_with(
            ResponseTemplate::new(200).set_body_json(json!({ "id": "file-1", "deleted": true })),
        )
        .mount(&server)
        .await;

    let c = client(&server);
    let page = c
        .files()
        .list(
            FileListParams {
                purpose: Some("batch".to_owned()),
                limit: Some(10),
                after: Some("f0".to_owned()),
            },
            None,
        )
        .await
        .expect("list ok");
    assert_eq!(page.data.len(), 1);
    assert_eq!(page.data[0].id, "f1");

    let got = c.files().retrieve("file-1", None).await.expect("get ok");
    assert_eq!(got.id, "file-1");

    let body = c
        .files()
        .content("file-1", RequestOptions::new().provider("openai"))
        .await
        .expect("content ok");
    assert_eq!(body, b"{\"a\":1}\n");

    let del = c.files().delete("file-1", None).await.expect("delete ok");
    assert_eq!(del.deleted, Some(true));
}

#[tokio::test]
async fn files_list_empty_query() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/files"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list", "data": []
        })))
        .mount(&server)
        .await;
    let c = client(&server);
    let page = c
        .files()
        .list(FileListParams::default(), None)
        .await
        .expect("list ok");
    assert!(page.data.is_empty());
    assert_eq!(page.object.as_deref(), Some("list"));
}

#[tokio::test]
async fn batches_create_list_get_cancel() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/batches"))
        .and(header("floopy-provider", "openai"))
        .and(body_string_contains("\"metadata\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": "batch_1", "status": "validating",
            "request_counts": { "total": 2, "completed": 0, "failed": 0 }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/batches"))
        .and(query_param("limit", "5"))
        .and(query_param("after", "batch_0"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list", "data": [{ "id": "batch_1" }], "has_more": false
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/batches/batch_1"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "id": "batch_1", "status": "completed" })),
        )
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/batches/batch_1/cancel"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({ "id": "batch_1", "status": "cancelling" })),
        )
        .mount(&server)
        .await;

    let c = client(&server);
    let mut meta = std::collections::HashMap::new();
    meta.insert("k".to_owned(), "v".to_owned());
    let b = c
        .batches()
        .create(
            BatchCreateParams {
                input_file_id: "file-1".to_owned(),
                endpoint: "/v1/chat/completions".to_owned(),
                completion_window: "24h".to_owned(),
                metadata: Some(meta),
            },
            RequestOptions::new().provider("openai"),
        )
        .await
        .expect("create ok");
    assert_eq!(b.id, "batch_1");
    assert_eq!(b.request_counts.and_then(|r| r.total), Some(2));

    let page = c
        .batches()
        .list(
            BatchListParams {
                limit: Some(5),
                after: Some("batch_0".to_owned()),
            },
            None,
        )
        .await
        .expect("list ok");
    assert_eq!(page.has_more, Some(false));
    assert_eq!(page.data[0].id, "batch_1");

    let got = c.batches().retrieve("batch_1", None).await.expect("get ok");
    assert_eq!(got.status.as_deref(), Some("completed"));

    let cancelled = c
        .batches()
        .cancel("batch_1", RequestOptions::new().provider("openai"))
        .await
        .expect("cancel ok");
    assert_eq!(cancelled.status.as_deref(), Some("cancelling"));
}

#[tokio::test]
async fn batches_create_without_metadata_and_empty_list_query() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/batches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({ "id": "batch_2" })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/batches"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "object": "list", "data": []
        })))
        .mount(&server)
        .await;
    let c = client(&server);
    let b = c
        .batches()
        .create(
            BatchCreateParams {
                input_file_id: "file-1".to_owned(),
                endpoint: "/v1/chat/completions".to_owned(),
                completion_window: "24h".to_owned(),
                metadata: None,
            },
            None,
        )
        .await
        .expect("create ok");
    assert_eq!(b.id, "batch_2");
    let page = c
        .batches()
        .list(BatchListParams::default(), None)
        .await
        .expect("list ok");
    assert!(page.data.is_empty());
}

#[tokio::test]
async fn batch_and_files_errors_propagate() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/files"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": { "message": "bad" }
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/files/f1/content"))
        .respond_with(ResponseTemplate::new(404).set_body_string("nope"))
        .mount(&server)
        .await;

    let c = client(&server);
    let up = c
        .files()
        .upload(
            FileUploadParams {
                file: b"x".to_vec(),
                filename: None,
                purpose: "batch".to_owned(),
            },
            None,
        )
        .await;
    assert!(matches!(up, Err(Error::Validation(_))), "got {up:?}");

    let content = c.files().content("f1", None).await;
    assert!(
        matches!(content, Err(Error::NotFound(_))),
        "got {content:?}"
    );
}

#[tokio::test]
async fn all_methods_surface_non_2xx() {
    let server = MockServer::start().await;
    // Catch-all 400 for every batch/files endpoint.
    Mock::given(wiremock::matchers::path_regex(r"^/v1/(files|batches).*$"))
        .respond_with(ResponseTemplate::new(400).set_body_json(json!({
            "error": { "message": "bad" }
        })))
        .mount(&server)
        .await;
    let c = client(&server);

    assert!(c
        .files()
        .list(FileListParams::default(), None)
        .await
        .is_err());
    assert!(c.files().retrieve("f1", None).await.is_err());
    assert!(c.files().delete("f1", None).await.is_err());
    assert!(c
        .batches()
        .create(
            BatchCreateParams {
                input_file_id: "f1".to_owned(),
                endpoint: "/v1/chat/completions".to_owned(),
                completion_window: "24h".to_owned(),
                metadata: None,
            },
            None,
        )
        .await
        .is_err());
    assert!(c
        .batches()
        .list(BatchListParams::default(), None)
        .await
        .is_err());
    assert!(c.batches().retrieve("b1", None).await.is_err());
    assert!(c.batches().cancel("b1", None).await.is_err());
}
