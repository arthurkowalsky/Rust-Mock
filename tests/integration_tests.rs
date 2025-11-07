use reqwest;
use serde_json::json;
use std::process::{Child, Command, Stdio};
use std::time::Duration;
use tokio::time::sleep;

const TEST_PORT: u16 = 18090;
const BASE_URL: &str = "http://127.0.0.1:18090";

struct TestServer {
    process: Child,
}

impl TestServer {
    async fn start() -> Self {
        // Build the application first
        let build_status = Command::new("cargo")
            .args(&["build", "--release"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .expect("Failed to build application");

        assert!(build_status.success(), "Build failed");

        // Start the server
        let process = Command::new("./target/release/RustMock")
            .args(&["--port", &TEST_PORT.to_string()])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("Failed to start server");

        let client = reqwest::Client::new();

        // Wait for server to start using async
        for _ in 0..50 {
            if client.get(format!("{}/__mock/config", BASE_URL))
                .send()
                .await
                .is_ok()
            {
                println!("Server started successfully on port {}", TEST_PORT);
                return TestServer { process };
            }
            sleep(Duration::from_millis(100)).await;
        }

        panic!("Server failed to start within timeout");
    }
}

impl Drop for TestServer {
    fn drop(&mut self) {
        let _ = self.process.kill();
        let _ = self.process.wait();
        println!("Server stopped");
    }
}

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

    // Clear any existing endpoints by getting config first
    let _ = client.delete(format!("{}/__mock/logs", BASE_URL)).send().await;

    // Add an endpoint
    let payload = json!({
        "method": "GET",
        "path": "/api/test",
        "response": {"message": "Hello from integration test"},
        "status": 200
    });

    let response = client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to add endpoint");

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["added"], true);

    // Verify the endpoint was added
    let config_response = client
        .get(format!("{}/__mock/config", BASE_URL))
        .send()
        .await
        .expect("Failed to get config");

    let config: serde_json::Value = config_response.json().await.expect("Failed to parse config");
    assert!(config.is_array());

    let endpoints = config.as_array().unwrap();
    let found = endpoints.iter().any(|e| {
        e["method"] == "GET" && e["path"] == "/api/test"
    });
    assert!(found, "Added endpoint not found in config");
}

#[tokio::test]
async fn test_call_dynamic_endpoint() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Add a dynamic endpoint
    let endpoint_payload = json!({
        "method": "POST",
        "path": "/api/users",
        "response": {"id": 42, "name": "Integration Test User"},
        "status": 201
    });

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&endpoint_payload)
        .send()
        .await
        .expect("Failed to add endpoint");

    // Call the dynamic endpoint
    let response = client
        .post(format!("{}/api/users", BASE_URL))
        .json(&json!({"name": "Test User"}))
        .send()
        .await
        .expect("Failed to call dynamic endpoint");

    assert_eq!(response.status().as_u16(), 201);

    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["id"], 42);
    assert_eq!(body["name"], "Integration Test User");
}

#[tokio::test]
async fn test_remove_endpoint_integration() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Add an endpoint
    let add_payload = json!({
        "method": "DELETE",
        "path": "/api/resource",
        "response": {"deleted": true},
        "status": 200
    });

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&add_payload)
        .send()
        .await
        .expect("Failed to add endpoint");

    // Remove the endpoint
    let remove_payload = json!({
        "method": "DELETE",
        "path": "/api/resource"
    });

    let response = client
        .delete(format!("{}/__mock/endpoints", BASE_URL))
        .json(&remove_payload)
        .send()
        .await
        .expect("Failed to remove endpoint");

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["removed"], true);
}

#[tokio::test]
async fn test_logs_integration() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Clear logs first
    client
        .delete(format!("{}/__mock/logs", BASE_URL))
        .send()
        .await
        .expect("Failed to clear logs");

    // Add and call an endpoint to generate logs
    let endpoint_payload = json!({
        "method": "GET",
        "path": "/api/logtest",
        "response": {"test": "log"},
        "status": 200
    });

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&endpoint_payload)
        .send()
        .await
        .expect("Failed to add endpoint");

    client
        .get(format!("{}/api/logtest", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");

    // Get logs
    let logs_response = client
        .get(format!("{}/__mock/logs", BASE_URL))
        .send()
        .await
        .expect("Failed to get logs");

    assert!(logs_response.status().is_success());

    let logs: serde_json::Value = logs_response.json().await.expect("Failed to parse logs");
    assert!(logs.is_array());

    let log_entries = logs.as_array().unwrap();
    // Should have at least the GET request to /api/logtest
    let found_log = log_entries.iter().any(|log| {
        log["method"] == "GET" && log["path"] == "/api/logtest"
    });
    assert!(found_log, "Expected log entry not found");
}

#[tokio::test]
async fn test_clear_logs_integration() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Add an endpoint and call it to generate logs
    let endpoint_payload = json!({
        "method": "GET",
        "path": "/api/cleartest",
        "response": {"test": "data"},
        "status": 200
    });

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&endpoint_payload)
        .send()
        .await
        .expect("Failed to add endpoint");

    client
        .get(format!("{}/api/cleartest", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");

    // Clear logs
    let response = client
        .delete(format!("{}/__mock/logs", BASE_URL))
        .send()
        .await
        .expect("Failed to clear logs");

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["cleared"], true);

    // Verify logs are cleared
    let logs_response = client
        .get(format!("{}/__mock/logs", BASE_URL))
        .send()
        .await
        .expect("Failed to get logs");

    let logs: serde_json::Value = logs_response.json().await.expect("Failed to parse logs");
    let log_entries = logs.as_array().unwrap();

    // Should be completely empty - admin endpoints (/__mock/*) are not logged
    assert_eq!(log_entries.len(), 0, "Expected logs to be empty after clearing");
}

#[tokio::test]
async fn test_not_found_integration() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let response = client
        .get(format!("{}/nonexistent-path", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");

    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn test_multiple_endpoints_integration() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Add multiple endpoints
    for i in 1..=3 {
        let payload = json!({
            "method": "GET",
            "path": format!("/api/endpoint{}", i),
            "response": {"endpoint": i, "name": format!("Endpoint {}", i)},
            "status": 200
        });

        client
            .post(format!("{}/__mock/endpoints", BASE_URL))
            .json(&payload)
            .send()
            .await
            .expect("Failed to add endpoint");
    }

    // Verify all endpoints are in config
    let config_response = client
        .get(format!("{}/__mock/config", BASE_URL))
        .send()
        .await
        .expect("Failed to get config");

    let config: serde_json::Value = config_response.json().await.expect("Failed to parse config");
    let endpoints = config.as_array().unwrap();

    for i in 1..=3 {
        let found = endpoints.iter().any(|e| {
            e["path"] == format!("/api/endpoint{}", i)
        });
        assert!(found, "Endpoint {} not found", i);
    }

    // Call each endpoint and verify response
    for i in 1..=3 {
        let response = client
            .get(format!("{}/api/endpoint{}", BASE_URL, i))
            .send()
            .await
            .expect("Failed to call endpoint");

        assert!(response.status().is_success());
        let body: serde_json::Value = response.json().await.expect("Failed to parse response");
        assert_eq!(body["endpoint"], i);
    }
}
