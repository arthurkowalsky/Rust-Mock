use super::common::{TestServer, BASE_URL};
use serde_json::json;

const PROXY_TARGET: &str = "https://httpbin.org";

#[tokio::test]
async fn test_proxy_config_endpoints() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let resp = client
        .get(format!("{}/__mock/proxy", BASE_URL))
        .send()
        .await
        .expect("Failed to get proxy config");

    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["enabled"], false);
    assert_eq!(body["proxy_url"], serde_json::Value::Null);

    let resp = client
        .post(format!("{}/__mock/proxy", BASE_URL))
        .json(&json!({"url": PROXY_TARGET}))
        .send()
        .await
        .expect("Failed to set proxy");

    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["enabled"], true);
    assert_eq!(body["proxy_url"], PROXY_TARGET);

    let resp = client
        .get(format!("{}/__mock/proxy", BASE_URL))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["enabled"], true);
    assert_eq!(body["proxy_url"], PROXY_TARGET);

    let resp = client
        .delete(format!("{}/__mock/proxy", BASE_URL))
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["deleted"], true);

    let resp = client
        .get(format!("{}/__mock/proxy", BASE_URL))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["enabled"], false);
}

#[tokio::test]
async fn test_endpoint_with_proxy_url() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "GET",
            "path": "/anything/objects",
            "response": {"mock": "this should not be returned"},
            "status": 200,
            "proxy_url": PROXY_TARGET
        }))
        .send()
        .await
        .expect("Failed to add endpoint");

    assert!(resp.status().is_success());

    let resp = client
        .get(format!("{}/anything/objects", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_object(), "Expected object response from httpbin");
}

#[tokio::test]
async fn test_default_proxy_mode() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/__mock/proxy", BASE_URL))
        .json(&json!({"url": PROXY_TARGET}))
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success());

    let resp = client
        .get(format!("{}/anything/objects", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.is_object(), "Expected object from proxied request");
}

#[tokio::test]
async fn test_mixed_mock_and_proxy() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/__mock/proxy", BASE_URL))
        .json(&json!({"url": PROXY_TARGET}))
        .send()
        .await
        .unwrap();

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "GET",
            "path": "/mock/users",
            "response": {"users": [{"id": 1, "name": "Mock User"}]},
            "status": 200
        }))
        .send()
        .await
        .unwrap();

    let resp1 = client
        .get(format!("{}/mock/users", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp1.status().as_u16(), 200);
    let body1: serde_json::Value = resp1.json().await.unwrap();
    assert_eq!(body1["users"][0]["name"], "Mock User");

    let resp2 = client
        .get(format!("{}/anything/objects", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp2.status().as_u16(), 200);
    let body2: serde_json::Value = resp2.json().await.unwrap();
    assert!(body2.is_object(), "Expected proxied response");
}

#[tokio::test]
async fn test_proxy_with_query_params() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "GET",
            "path": "/anything/objects",
            "response": {},
            "status": 200,
            "proxy_url": PROXY_TARGET
        }))
        .send()
        .await
        .unwrap();

    let resp = client
        .get(format!("{}/anything/objects?id=1&id=2", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["args"].is_object(), "Expected query params in args");
}

#[tokio::test]
async fn test_proxy_post_with_body() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "POST",
            "path": "/anything/objects",
            "response": {},
            "status": 200,
            "proxy_url": PROXY_TARGET
        }))
        .send()
        .await
        .unwrap();

    let resp = client
        .post(format!("{}/anything/objects", BASE_URL))
        .json(&json!({
            "name": "Test Object",
            "data": {"year": 2025, "price": 99.99}
        }))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body.get("json").is_some(), "Expected json echo from httpbin");
    assert_eq!(body["json"]["name"], "Test Object");
}

#[tokio::test]
async fn test_proxy_failure_returns_502() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "GET",
            "path": "/test/fail",
            "response": {},
            "status": 200,
            "proxy_url": "http://invalid-host-that-does-not-exist-12345.com"
        }))
        .send()
        .await
        .unwrap();

    let resp = client
        .get(format!("{}/test/fail", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 502);
    let body: serde_json::Value = resp.json().await.unwrap();
    assert!(body["error"].as_str().unwrap().contains("Proxy request failed"));
}

#[tokio::test]
async fn test_proxy_does_not_forward_accept_encoding() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/__mock/proxy", BASE_URL))
        .json(&json!({"url": PROXY_TARGET}))
        .send()
        .await
        .unwrap();

    let resp = client
        .get(format!("{}/anything/objects", BASE_URL))
        .header("accept-encoding", "gzip, deflate, br, zstd")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
}
