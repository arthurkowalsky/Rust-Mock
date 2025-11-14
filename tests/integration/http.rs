use super::common::{TestServer, BASE_URL};
use serde_json::json;

#[tokio::test]
async fn test_custom_headers_in_response() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let payload = json!({
        "method": "GET",
        "path": "/api/with-headers",
        "response": {"message": "success"},
        "status": 200,
        "headers": {
            "X-Custom-Header": "custom-value",
            "X-Rate-Limit": "100"
        }
    });

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to add endpoint");

    let response = client
        .get(format!("{}/api/with-headers", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");

    assert!(response.status().is_success());

    let config_response = client
        .get(format!("{}/__mock/config", BASE_URL))
        .send()
        .await
        .expect("Failed to get config");

    let config: serde_json::Value = config_response.json().await.expect("Failed to parse config");
    let endpoints = config.as_array().unwrap();
    let endpoint = endpoints.iter()
        .find(|e| e["path"] == "/api/with-headers")
        .expect("Endpoint not found");

    assert_eq!(endpoint["headers"]["X-Custom-Header"], "custom-value");
    assert_eq!(endpoint["headers"]["X-Rate-Limit"], "100");
}

#[tokio::test]
async fn test_different_http_methods() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let methods = vec![
        ("PUT", "/api/update"),
        ("PATCH", "/api/partial"),
        ("DELETE", "/api/remove"),
        ("POST", "/api/create"),
    ];

    for (method, path) in methods.iter() {
        let payload = json!({
            "method": method,
            "path": path,
            "response": {"method": method, "success": true},
            "status": 200
        });

        client
            .post(format!("{}/__mock/endpoints", BASE_URL))
            .json(&payload)
            .send()
            .await
            .expect("Failed to add endpoint");
    }

    let response = client
        .put(format!("{}/api/update", BASE_URL))
        .send()
        .await
        .expect("Failed to call PUT endpoint");
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["method"], "PUT");

    let response = client
        .patch(format!("{}/api/partial", BASE_URL))
        .send()
        .await
        .expect("Failed to call PATCH endpoint");
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["method"], "PATCH");

    let response = client
        .delete(format!("{}/api/remove", BASE_URL))
        .send()
        .await
        .expect("Failed to call DELETE endpoint");
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["method"], "DELETE");

    let response = client
        .post(format!("{}/api/create", BASE_URL))
        .send()
        .await
        .expect("Failed to call POST endpoint");
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["method"], "POST");
}

#[tokio::test]
async fn test_different_status_codes() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let status_codes = vec![
        (200, "/api/ok"),
        (201, "/api/created"),
        (204, "/api/no-content"),
        (400, "/api/bad-request"),
        (401, "/api/unauthorized"),
        (403, "/api/forbidden"),
        (500, "/api/server-error"),
        (503, "/api/unavailable"),
    ];

    for (status, path) in status_codes.iter() {
        let payload = json!({
            "method": "GET",
            "path": path,
            "response": {"status": status},
            "status": status
        });

        client
            .post(format!("{}/__mock/endpoints", BASE_URL))
            .json(&payload)
            .send()
            .await
            .expect("Failed to add endpoint");
    }

    for (expected_status, path) in status_codes.iter() {
        let response = client
            .get(format!("{}{}", BASE_URL, path))
            .send()
            .await
            .expect("Failed to call endpoint");

        assert_eq!(response.status().as_u16(), *expected_status as u16,
            "Expected status {} for path {}", expected_status, path);
    }
}

#[tokio::test]
async fn test_request_body_and_query_params_in_logs() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client
        .delete(format!("{}/__mock/logs", BASE_URL))
        .send()
        .await
        .expect("Failed to clear logs");

    let payload = json!({
        "method": "POST",
        "path": "/api/data",
        "response": {"received": true},
        "status": 200
    });

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to add endpoint");

    let request_body = json!({
        "name": "Test User",
        "email": "test@example.com",
        "age": 25
    });

    client
        .post(format!("{}/api/data?key=value&foo=bar", BASE_URL))
        .json(&request_body)
        .send()
        .await
        .expect("Failed to call endpoint");

    let logs_response = client
        .get(format!("{}/__mock/logs", BASE_URL))
        .send()
        .await
        .expect("Failed to get logs");

    let logs: serde_json::Value = logs_response.json().await.expect("Failed to parse logs");
    let log_entries = logs.as_array().unwrap();

    let log = log_entries.iter()
        .find(|l| l["path"] == "/api/data")
        .expect("Log entry not found");

    assert_eq!(log["query"], "key=value&foo=bar");

    assert_eq!(log["request_body"]["name"], "Test User");
    assert_eq!(log["request_body"]["email"], "test@example.com");
    assert_eq!(log["request_body"]["age"], 25);
}
