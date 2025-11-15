use super::common::{TestServer, BASE_URL};
use serde_json::json;

#[tokio::test]
async fn test_server_starts() {
    let _server = TestServer::start().await;

    let client = reqwest::Client::new();
    let response = client
        .get(format!("{}/__mock/config", BASE_URL))
        .send()
        .await
        .expect("Failed to get config");

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_add_endpoint_integration() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let response = client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "GET",
            "path": "/test",
            "response": {"message": "test"},
            "status": 200
        }))
        .send()
        .await
        .expect("Failed to add endpoint");

    assert!(response.status().is_success());

    let config_response = client
        .get(format!("{}/__mock/config", BASE_URL))
        .send()
        .await
        .expect("Failed to get config");

    let config: serde_json::Value = config_response.json().await.expect("Failed to parse config");
    let endpoints = config.as_array().unwrap();

    assert!(endpoints.iter().any(|e| e["path"] == "/test"));
}

#[tokio::test]
async fn test_call_dynamic_endpoint() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "GET",
            "path": "/hello",
            "response": {"message": "Hello World"},
            "status": 200
        }))
        .send()
        .await
        .expect("Failed to add endpoint");

    let response = client
        .get(format!("{}/hello", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");

    assert_eq!(response.status().as_u16(), 200);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["message"], "Hello World");
}

#[tokio::test]
async fn test_remove_endpoint_integration() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "GET",
            "path": "/temp",
            "response": {"data": "temporary"},
            "status": 200
        }))
        .send()
        .await
        .expect("Failed to add endpoint");

    let delete_response = client
        .delete(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({
            "method": "GET",
            "path": "/temp"
        }))
        .send()
        .await
        .expect("Failed to delete endpoint");

    assert!(delete_response.status().is_success());

    let call_response = client
        .get(format!("{}/temp", BASE_URL))
        .send()
        .await
        .expect("Failed to call removed endpoint");

    assert_eq!(call_response.status().as_u16(), 404);
}

#[tokio::test]
async fn test_not_found_integration() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/nonexistent", BASE_URL))
        .send()
        .await
        .expect("Failed to make request");

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn test_multiple_endpoints_integration() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client.post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({"method": "GET", "path": "/one", "response": {"id": 1}, "status": 200}))
        .send().await.expect("Failed");

    client.post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({"method": "GET", "path": "/two", "response": {"id": 2}, "status": 200}))
        .send().await.expect("Failed");

    client.post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({"method": "POST", "path": "/three", "response": {"id": 3}, "status": 201}))
        .send().await.expect("Failed");

    let resp1 = client.get(format!("{}/one", BASE_URL)).send().await.unwrap();
    let body1: serde_json::Value = resp1.json().await.unwrap();
    assert_eq!(body1["id"], 1);

    let resp2 = client.get(format!("{}/two", BASE_URL)).send().await.unwrap();
    let body2: serde_json::Value = resp2.json().await.unwrap();
    assert_eq!(body2["id"], 2);

    let resp3 = client.post(format!("{}/three", BASE_URL)).send().await.unwrap();
    assert_eq!(resp3.status().as_u16(), 201);
}

#[tokio::test]
async fn test_overwriting_endpoint() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client.post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({"method": "GET", "path": "/data", "response": {"version": 1}, "status": 200}))
        .send().await.unwrap();

    client.post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({"method": "GET", "path": "/data", "response": {"version": 2}, "status": 200}))
        .send().await.unwrap();

    let resp = client.get(format!("{}/data", BASE_URL)).send().await.unwrap();
    let body: serde_json::Value = resp.json().await.unwrap();
    assert_eq!(body["version"], 2);
}

#[tokio::test]
async fn test_remove_nonexistent_endpoint() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let response = client
        .delete(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({"method": "GET", "path": "/does-not-exist"}))
        .send()
        .await
        .unwrap();

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["removed"], true);
}

#[tokio::test]
async fn test_case_sensitive_paths() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client.post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({"method": "GET", "path": "/Test", "response": {"case": "upper"}, "status": 200}))
        .send().await.unwrap();

    let resp_upper = client.get(format!("{}/Test", BASE_URL)).send().await.unwrap();
    assert_eq!(resp_upper.status().as_u16(), 200);

    let resp_lower = client.get(format!("{}/test", BASE_URL)).send().await.unwrap();
    assert_eq!(resp_lower.status().as_u16(), 404);
}

#[tokio::test]
async fn test_empty_response_body() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client.post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&json!({"method": "DELETE", "path": "/item/123", "response": {}, "status": 204}))
        .send().await.unwrap();

    let resp = client.delete(format!("{}/item/123", BASE_URL)).send().await.unwrap();
    assert_eq!(resp.status().as_u16(), 204);
}

#[tokio::test]
async fn test_path_parameters_in_mock_endpoint() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let payload = json!({
        "method": "GET",
        "path": "/users/{user_id}",
        "response": {"id": 123, "name": "Test User"},
        "status": 200
    });

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to add endpoint");

    let response = client
        .get(format!("{}/users/42", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["id"], 123);
    assert_eq!(body["name"], "Test User");

    let response2 = client
        .get(format!("{}/users/999", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");

    assert!(response2.status().is_success());
    let body2: serde_json::Value = response2.json().await.unwrap();
    assert_eq!(body2["id"], 123);
}
