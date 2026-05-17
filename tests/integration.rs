//! Integration tests against a mock gateway (wiremock), mirroring the
//! Go/Python suites.

use floopy::types::{
    DecisionListParams, EvaluationCreateParams, ExperimentCreateParams, ExportDecisionsParams,
    FeedbackSubmitParams, OrgConstraints, RoutingExplainParams,
};
use floopy::{CacheOptions, Error, Floopy, FloopyOptions, RequestOptions};
use futures::StreamExt;
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn client(server: &MockServer) -> Floopy {
    Floopy::builder("fl_test")
        .base_url(format!("{}/v1", server.uri()))
        .max_retries(0)
        .build()
        .expect("client builds")
}

fn decision_wire(rid: &str) -> serde_json::Value {
    json!({
        "request_id": rid, "session_id": null, "request_created_at": "2026-05-10T00:00:00Z",
        "provider": null, "model": null, "status": "ok", "latency_ms": null,
        "cost_micro_usd": null, "cache_enabled": null, "threat": null,
        "decision_trace": null, "confidence": null, "confidence_reason": null,
        "explanation": null
    })
}

fn experiment_wire(eid: &str) -> serde_json::Value {
    json!({
        "id": eid, "name": "test", "description": null, "status": "active",
        "variant_a_routing_rule_id": "rule_a", "variant_b_routing_rule_id": "rule_b",
        "split_percentage": 50, "created_at": "2026-05-10T00:00:00Z", "rolled_back_at": null
    })
}

#[tokio::test]
async fn new_rejects_empty_api_key() {
    let err = Floopy::new("").expect_err("empty api key must fail");
    assert!(matches!(err, Error::Config(_)));
}

#[tokio::test]
async fn lazy_openai_delegate_is_reused() {
    let c = Floopy::new("fl_test").expect("client builds");
    let a = c.openai() as *const _;
    let b = c.openai() as *const _;
    assert_eq!(a, b, "delegate must be built once and reused");
}

#[tokio::test]
async fn forwards_floopy_and_auth_headers() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/decisions/abc"))
        .and(header("authorization", "Bearer fl_test"))
        .and(header("floopy-cache-enabled", "true"))
        .and(header("floopy-cache-bucket-max-size", "4"))
        .and(header("floopy-prompt-id", "p1"))
        .and(header("floopy-llm-security-enabled", "true"))
        .respond_with(ResponseTemplate::new(200).set_body_json(decision_wire("abc")))
        .mount(&server)
        .await;

    let c = Floopy::builder("fl_test")
        .base_url(format!("{}/v1", server.uri()))
        .options(FloopyOptions {
            cache: Some(CacheOptions {
                enabled: Some(true),
                bucket_max_size: Some(4),
            }),
            prompt_id: Some("p1".to_owned()),
            llm_security_enabled: Some(true),
            ..Default::default()
        })
        .build()
        .expect("client builds");

    let decision = c.decisions().get("abc", None).await.expect("request ok");
    assert_eq!(decision.request_id, "abc");
}

#[tokio::test]
async fn maps_429_into_rate_limit_error() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/decisions"))
        .respond_with(
            ResponseTemplate::new(429)
                .insert_header("Retry-After", "5")
                .insert_header("X-Request-Id", "req_x")
                .set_body_json(json!({"error": {"code": "rate_limited", "message": "slow down"}})),
        )
        .mount(&server)
        .await;

    let c = client(&server);
    let err = c
        .decisions()
        .list(&DecisionListParams::default(), None)
        .await
        .expect_err("must be rate limited");

    assert!(matches!(err, Error::RateLimit(_)));
    assert_eq!(err.status(), Some(429));
    assert_eq!(err.request_id(), Some("req_x"));
    assert_eq!(err.retry_after_seconds(), Some(5));
}

#[tokio::test]
async fn retries_5xx_until_success() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/decisions"))
        .respond_with(ResponseTemplate::new(503))
        .up_to_n_times(2)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/decisions"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"items": [], "next_cursor": null, "has_more": false})),
        )
        .mount(&server)
        .await;

    let c = Floopy::builder("fl_test")
        .base_url(format!("{}/v1", server.uri()))
        .max_retries(2)
        .build()
        .expect("client builds");

    let page = c
        .decisions()
        .list(&DecisionListParams::default(), None)
        .await
        .expect("eventually succeeds");
    assert!(page.items.is_empty());
}

#[tokio::test]
async fn connection_error_is_wrapped() {
    // Nothing is listening on this port.
    let c = Floopy::builder("fl_test")
        .base_url("http://127.0.0.1:1/v1")
        .max_retries(0)
        .build()
        .expect("client builds");
    let err = c
        .decisions()
        .list(&DecisionListParams::default(), None)
        .await
        .expect_err("must fail to connect");
    assert!(matches!(err, Error::Connection(_)));
}

#[tokio::test]
async fn feedback_posts_body() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/feedback"))
        .and(wiremock::matchers::body_json(
            json!({"score": 9, "useful": true, "session_id": "s1"}),
        ))
        .respond_with(
            ResponseTemplate::new(200)
                .set_body_json(json!({"duplicate": false, "session_id": "s1"})),
        )
        .mount(&server)
        .await;

    let c = client(&server);
    let res = c
        .feedback()
        .submit(
            FeedbackSubmitParams {
                score: 9,
                useful: true,
                session_id: Some("s1".to_owned()),
            },
            None,
        )
        .await
        .expect("feedback ok");
    assert!(!res.duplicate);
    assert_eq!(res.session_id.as_deref(), Some("s1"));
}

#[tokio::test]
async fn decisions_paginate_via_pages_stream() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/decisions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [decision_wire("req_1")], "next_cursor": "cur_1", "has_more": true
        })))
        .up_to_n_times(1)
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/v1/decisions"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "items": [decision_wire("req_2")], "next_cursor": null, "has_more": false
        })))
        .mount(&server)
        .await;

    let c = client(&server);
    let mut ids = Vec::new();
    let stream = c.decisions().pages(DecisionListParams::default(), None);
    futures::pin_mut!(stream);
    while let Some(page) = stream.next().await {
        for d in page.expect("page ok").items {
            ids.push(d.request_id);
        }
    }
    assert_eq!(ids, vec!["req_1", "req_2"]);
}

#[tokio::test]
async fn experiments_inject_confirm_header() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/experiments"))
        .and(header("x-floopy-confirm", "experiments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(experiment_wire("exp_1")))
        .mount(&server)
        .await;
    Mock::given(method("POST"))
        .and(path("/v1/experiments/exp_1/rollback"))
        .and(header("x-floopy-confirm", "experiments"))
        .respond_with(ResponseTemplate::new(200).set_body_json(experiment_wire("exp_1")))
        .mount(&server)
        .await;

    let c = client(&server);
    c.experiments()
        .create(
            ExperimentCreateParams {
                name: "test".to_owned(),
                variant_a_routing_rule_id: "rule_a".to_owned(),
                variant_b_routing_rule_id: "rule_b".to_owned(),
                description: None,
                split_percentage: None,
            },
            None,
        )
        .await
        .expect("create ok (confirm header matched)");
    c.experiments()
        .rollback("exp_1", None)
        .await
        .expect("rollback ok (confirm header matched)");
}

#[tokio::test]
async fn constraints_put_sends_all_keys() {
    let server = MockServer::start().await;
    Mock::given(method("PUT"))
        .and(path("/v1/constraints"))
        .and(wiremock::matchers::body_json(json!({
            "cost_limit_monthly_usd": 100.0,
            "token_window_seconds": null,
            "max_tokens_per_window": null,
            "max_requests_per_minute": 60
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "cost_limit_monthly_usd": 100.0, "token_window_seconds": null,
            "max_tokens_per_window": null, "max_requests_per_minute": 60
        })))
        .mount(&server)
        .await;

    let c = client(&server);
    let res = c
        .constraints()
        .put(
            &OrgConstraints {
                cost_limit_monthly_usd: Some(100.0),
                max_requests_per_minute: Some(60),
                ..Default::default()
            },
            None,
        )
        .await
        .expect("put ok (full-replace body matched)");
    assert_eq!(res.cost_limit_monthly_usd, Some(100.0));
    assert_eq!(res.token_window_seconds, None);
}

#[tokio::test]
async fn export_skips_trailer() {
    let server = MockServer::start().await;
    let body = [
        json!({"request_id":"req_1","session_id":null,"organization_id":"org_1","provider":"openai","model":"gpt-4o","status":"ok","latency_ms":100,"cost_micro_usd":1000,"cache_enabled":false,"threat":null,"created_at":"2026-05-10T00:00:00Z"}),
        json!({"request_id":"req_2","session_id":null,"organization_id":"org_1","provider":"openai","model":"gpt-4o","status":"ok","latency_ms":200,"cost_micro_usd":2000,"cache_enabled":false,"threat":null,"created_at":"2026-05-10T00:01:00Z"}),
        json!({"trailer":true,"rows_emitted":2,"truncated":false,"reason":null}),
    ]
    .iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<_>>()
    .join("\n");
    Mock::given(method("GET"))
        .and(path("/v1/export/decisions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let c = client(&server);
    let stream = c
        .export()
        .decisions(ExportDecisionsParams::new("a", "b"), None);
    futures::pin_mut!(stream);
    let mut ids = Vec::new();
    while let Some(row) = stream.next().await {
        ids.push(row.expect("row ok").request_id);
    }
    assert_eq!(ids, vec!["req_1", "req_2"]);
}

#[tokio::test]
async fn export_with_trailer_capture() {
    let server = MockServer::start().await;
    let body = [
        json!({"request_id":"req_1","session_id":null,"organization_id":"org_1","provider":null,"model":null,"status":"ok","latency_ms":null,"cost_micro_usd":null,"cache_enabled":null,"threat":null,"created_at":"2026-05-10T00:00:00Z"}),
        json!({"trailer":true,"rows_emitted":1,"truncated":true,"reason":"deadline"}),
    ]
    .iter()
    .map(std::string::ToString::to_string)
    .collect::<Vec<_>>()
    .join("\n");
    Mock::given(method("GET"))
        .and(path("/v1/export/decisions"))
        .respond_with(ResponseTemplate::new(200).set_body_string(body))
        .mount(&server)
        .await;

    let c = client(&server);
    let mut stream = c
        .export()
        .decisions_with_trailer(ExportDecisionsParams::new("a", "b"), None);
    let mut count = 0;
    while let Some(row) = stream.next().await {
        row.expect("row ok");
        count += 1;
    }
    assert_eq!(count, 1);
    let trailer = stream.trailer().expect("trailer captured");
    assert!(trailer.truncated);
    assert_eq!(trailer.reason.as_deref(), Some("deadline"));
}

#[tokio::test]
async fn routing_explain_maps_wire() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/routing/explain"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "would_select": {"provider": "openai", "model": "gpt-4o-mini"},
            "firewall_decision": "allow", "reasoning": null, "routing_rule_id": "rule_1"
        })))
        .mount(&server)
        .await;

    let c = client(&server);
    let res = c
        .routing()
        .explain(RoutingExplainParams::new("gpt-4o", vec![]), None)
        .await
        .expect("explain ok");
    assert_eq!(res.firewall_decision, "allow");
    assert_eq!(
        res.would_select.and_then(|m| m.get("model").cloned()),
        Some("gpt-4o-mini".to_owned())
    );
}

#[tokio::test]
async fn evaluations_create_omits_optional_fields() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/evaluations"))
        .and(wiremock::matchers::body_json(
            json!({"dataset_id": "ds_1", "model": "gpt-4o"}),
        ))
        .respond_with(ResponseTemplate::new(201).set_body_json(json!({
            "id": "eval_1", "dataset_id": "ds_1", "model": "gpt-4o", "prompt_id": null,
            "status": "pending", "config": null, "created_at": "2026-05-10T00:00:00Z",
            "started_at": null, "finished_at": null
        })))
        .mount(&server)
        .await;

    let c = client(&server);
    let run = c
        .evaluations()
        .create(
            EvaluationCreateParams {
                dataset_id: "ds_1".to_owned(),
                model: "gpt-4o".to_owned(),
                prompt_id: None,
                config: None,
            },
            None,
        )
        .await
        .expect("create ok (optional fields omitted from body)");
    assert_eq!(run.id, "eval_1");
}

#[tokio::test]
async fn sessions_get_percent_encodes_path_and_sends_per_call_header() {
    let server = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/v1/session/sess%2F1"))
        .and(header("x-trace", "abc"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "session_id": "sess/1",
            "messages": [{"role": "user", "content": "hi"}],
            "turn_count": 1,
            "turns": [{"request_id": "r1", "created_at": "2026-05-17T10:00:00Z", "model": "gpt-4o", "provider": "openai"}]
        })))
        .mount(&server)
        .await;

    let c = client(&server);
    let s = c
        .sessions()
        .get("sess/1", RequestOptions::new().header("x-trace", "abc"))
        .await
        .expect("session ok (encoded path + header matched)");
    assert_eq!(s.session_id, "sess/1");
    assert_eq!(s.turn_count, 1);
    assert_eq!(s.turns[0].request_id, "r1");
}
