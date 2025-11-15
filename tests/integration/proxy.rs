use super::common::{TestServer, BASE_URL};
use serde_json::json;

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
        .json(&json!({"url": "https://httpbin.org"}))
        .send()
        .await
        .expect("Failed to set proxy");

    assert!(resp.status().is_success());
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["enabled"], true);
    assert_eq!(body["proxy_url"], "https://httpbin.org");

    let resp = client
        .get(format!("{}/__mock/proxy", BASE_URL))
        .send()
        .await
        .unwrap();

    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["enabled"], true);
    assert_eq!(body["proxy_url"], "https://httpbin.org");

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
            "path": "/test/proxy",
            "response": {"mock": "this should not be returned"},
            "status": 200,
            "proxy_url": "https://httpbin.org"
        }))
        .send()
        .await
        .expect("Failed to add endpoint");

    assert!(resp.status().is_success());

    let resp = client
        .get(format!("{}/test/proxy", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 404);
}

#[tokio::test]
async fn test_endpoint_proxy_to_httpbin_status() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "GET",
            "path": "/status/201",
            "response": {},
            "status": 200,
            "proxy_url": "https://httpbin.org"
        }))
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success());

    let resp = client
        .get(format!("{}/status/201", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 201);
}

#[tokio::test]
async fn test_default_proxy_mode() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let resp = client
        .post(format!("{}/__mock/proxy", BASE_URL))
        .json(&json!({"url": "https://httpbin.org"}))
        .send()
        .await
        .unwrap();

    assert!(resp.status().is_success());

    let resp = client
        .get(format!("{}/get", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    assert!(body.get("url").is_some() || body.get("headers").is_some());
}

#[tokio::test]
async fn test_mixed_mock_and_proxy() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/__mock/proxy", BASE_URL))
        .json(&json!({"url": "https://httpbin.org"}))
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

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "GET",
            "path": "/uuid",
            "response": {},
            "status": 200,
            "proxy_url": "https://httpbin.org"
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
        .get(format!("{}/uuid", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp2.status().as_u16(), 200);
    let body2: serde_json::Value = resp2.json().await.unwrap();
    assert!(body2.get("uuid").is_some());

    let resp3 = client
        .get(format!("{}/headers", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp3.status().as_u16(), 200);
    let body3: serde_json::Value = resp3.json().await.unwrap();
    assert!(body3.get("headers").is_some());
}

#[tokio::test]
async fn test_proxy_with_query_params() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "GET",
            "path": "/get",
            "response": {},
            "status": 200,
            "proxy_url": "https://httpbin.org"
        }))
        .send()
        .await
        .unwrap();

    let resp = client
        .get(format!("{}/get?foo=bar&baz=qux", BASE_URL))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    assert_eq!(body["args"]["foo"], "bar");
    assert_eq!(body["args"]["baz"], "qux");
}

#[tokio::test]
async fn test_proxy_post_with_body() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "POST",
            "path": "/post",
            "response": {},
            "status": 200,
            "proxy_url": "https://httpbin.org"
        }))
        .send()
        .await
        .unwrap();

    let resp = client
        .post(format!("{}/post", BASE_URL))
        .json(&json!({"test": "data", "number": 42}))
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    assert_eq!(body["json"]["test"], "data");
    assert_eq!(body["json"]["number"], 42);
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
        .json(&json!({"url": "https://httpbin.org"}))
        .send()
        .await
        .unwrap();

    let resp = client
        .get(format!("{}/headers", BASE_URL))
        .header("accept-encoding", "gzip, deflate, br, zstd")
        .send()
        .await
        .unwrap();

    assert_eq!(resp.status().as_u16(), 200);
    let body: serde_json::Value = resp.json().await.unwrap();

    let headers = body["headers"].as_object().unwrap();
    assert!(headers.get("Accept-Encoding").is_none() ||
            !headers.get("Accept-Encoding").unwrap().as_str().unwrap().contains("zstd"),
            "accept-encoding should not be forwarded to upstream");
}
