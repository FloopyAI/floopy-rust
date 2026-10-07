use floopy::async_openai::types::responses::CreateResponse;
use floopy::{Floopy, FloopyOptions};
use futures::StreamExt;
use serde_json::json;
use wiremock::matchers::{body_partial_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

#[tokio::test]
async fn native_response_and_stream_use_gateway_headers() {
    let server = MockServer::start().await;
    let output = json!({"id":"resp_test","object":"response","created_at":1,"model":"gpt-6-luna","status":"completed","output":[{"id":"msg_1","type":"message","role":"assistant","status":"completed","content":[{"type":"output_text","text":"Olá","annotations":[]}]}]});
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .and(header("authorization", "Bearer fl_test"))
        .and(header("floopy-llm-security-enabled", "true"))
        .and(body_partial_json(json!({"input":"hi","store":false})))
        .respond_with(ResponseTemplate::new(200).set_body_json(output.clone()))
        .expect(1)
        .mount(&server)
        .await;
    let client = Floopy::builder("fl_test")
        .base_url(format!("{}/v1", server.uri()))
        .options(FloopyOptions {
            llm_security_enabled: Some(true),
            ..Default::default()
        })
        .build()
        .unwrap();
    let request: CreateResponse =
        serde_json::from_value(json!({"model":"gpt-6-luna","input":"hi","store":false})).unwrap();
    let result = client.responses().create(request).await.unwrap();
    assert_eq!(result.id, "resp_test");
    server.reset().await;
    let frames = format!(
        "event: response.completed\ndata: {}\n\n",
        json!({"type":"response.completed","sequence_number":0,"response":output})
    );
    Mock::given(method("POST"))
        .and(path("/v1/responses"))
        .and(body_partial_json(json!({"stream":true})))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(frames),
        )
        .expect(1)
        .mount(&server)
        .await;
    let request: CreateResponse =
        serde_json::from_value(json!({"model":"gpt-6-luna","input":"hi"})).unwrap();
    let mut stream = client.responses().create_stream(request).await.unwrap();
    let event = stream.next().await.unwrap().unwrap();
    assert_eq!(
        serde_json::to_value(event).unwrap()["type"],
        "response.completed"
    );
}
