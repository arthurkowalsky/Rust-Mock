use super::common::{TestServer, BASE_URL};
use serde_json::json;

#[tokio::test]
async fn test_logs_integration() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    client
        .delete(format!("{}/__mock/logs", BASE_URL))
        .send()
        .await
        .expect("Failed to clear logs");

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

    let logs_response = client
        .get(format!("{}/__mock/logs", BASE_URL))
        .send()
        .await
        .expect("Failed to get logs");

    assert!(logs_response.status().is_success());

    let logs: serde_json::Value = logs_response.json().await.expect("Failed to parse logs");
    assert!(logs.is_array());

    let log_entries = logs.as_array().unwrap();
    let found_log = log_entries.iter().any(|log| {
        log["method"] == "GET" && log["path"] == "/api/logtest"
    });
    assert!(found_log, "Expected log entry not found");
}

#[tokio::test]
async fn test_clear_logs_integration() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

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

    let response = client
        .delete(format!("{}/__mock/logs", BASE_URL))
        .send()
        .await
        .expect("Failed to clear logs");

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["cleared"], true);

    let logs_response = client
        .get(format!("{}/__mock/logs", BASE_URL))
        .send()
        .await
        .expect("Failed to get logs");

    let logs: serde_json::Value = logs_response.json().await.expect("Failed to parse logs");
    let log_entries = logs.as_array().unwrap();

    assert_eq!(log_entries.len(), 0, "Expected logs to be empty after clearing");
}
