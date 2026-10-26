use axum::body::Body;
use axum::http::{Request, StatusCode};
use mobile_api::{build_router, AppState, CbsClient, KeyStore};
use serde_json::json;
use tower::ServiceExt;
use wiremock::matchers::{body_json, header, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

fn test_state(cbs_url: &str) -> AppState {
    let cbs = CbsClient::new(cbs_url, "cbs-key", 5).unwrap();
    let keys = KeyStore::from_pairs(vec![("test-key", "dev")]);
    AppState::for_tests(cbs, keys)
}

#[tokio::test]
async fn unknown_key_is_unauthorized_with_an_empty_body() {
    let app = build_router(test_state("http://127.0.0.1:9"));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/deposit/balance")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"accountNumber":"1"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    assert!(body.is_empty());
}

#[tokio::test]
async fn tenant_mismatch_is_bad_request() {
    let app = build_router(test_state("http://127.0.0.1:9"));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/deposit/balance")
                .header("authorization", "test-key")
                .header("tenant", "other")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"accountNumber":"1"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
    let body = axum::body::to_bytes(response.into_body(), 1024).await.unwrap();
    assert!(body.is_empty());
}

#[tokio::test]
async fn balance_forwards_the_key_and_returns_trimmed_json() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mbank/balance"))
        .and(header("x-api-key", "cbs-key"))
        .and(header("tenant", "dev"))
        .and(body_json(json!({"branchId": "01", "accountNumber": "100"})))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "availableBalance": 10,
            "ledgerBalance": 12,
            "extra": "ignored"
        })))
        .mount(&server)
        .await;

    let app = build_router(test_state(&server.uri()));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/deposit/balance")
                .header("authorization", "test-key")
                .header("tenant", "dev")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"branchId":"01","accountNumber":"100"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::OK);
    assert!(response.headers().get("x-request-id").is_some());
    let body = axum::body::to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["availableBalance"], 10);
    assert_eq!(json["ledgerBalance"], 12);
    assert_eq!(json["isoResponseCode"], "00");
    assert!(json.get("extra").is_none());
}

#[tokio::test]
async fn balance_maps_insufficient_funds_to_iso_51() {
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/api/mbank/balance"))
        .respond_with(ResponseTemplate::new(500).set_body_json(json!({"message": "InsufficientFund"})))
        .mount(&server)
        .await;

    let app = build_router(test_state(&server.uri()));
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/deposit/balance")
                .header("authorization", "test-key")
                .header("tenant", "dev")
                .header("content-type", "application/json")
                .body(Body::from(r#"{"accountNumber":"100"}"#))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
    let body = axum::body::to_bytes(response.into_body(), 64 * 1024).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["message"], "InsufficientFund");
    assert_eq!(json["errorCode"], 500);
    assert_eq!(json["isoResponseCode"], "51");
}
