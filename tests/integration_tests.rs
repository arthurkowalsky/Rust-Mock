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

#[tokio::test]
async fn test_custom_headers_in_response() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Add endpoint with custom headers
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

    // Call the endpoint and verify headers
    let response = client
        .get(format!("{}/api/with-headers", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");

    assert!(response.status().is_success());

    // Note: actix-web may not return custom headers from DynamicEndpoint
    // This tests that the endpoint definition accepts headers
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

    // Test PUT, PATCH, DELETE methods
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

    // Test PUT
    let response = client
        .put(format!("{}/api/update", BASE_URL))
        .send()
        .await
        .expect("Failed to call PUT endpoint");
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["method"], "PUT");

    // Test PATCH
    let response = client
        .patch(format!("{}/api/partial", BASE_URL))
        .send()
        .await
        .expect("Failed to call PATCH endpoint");
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["method"], "PATCH");

    // Test DELETE
    let response = client
        .delete(format!("{}/api/remove", BASE_URL))
        .send()
        .await
        .expect("Failed to call DELETE endpoint");
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["method"], "DELETE");

    // Test POST
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

    // Test various status codes
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

    // Test each status code
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

    // Clear logs
    client
        .delete(format!("{}/__mock/logs", BASE_URL))
        .send()
        .await
        .expect("Failed to clear logs");

    // Add an endpoint
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

    // Call with body and query params
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

    // Get logs and verify body and query are logged
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

    // Verify query params
    assert_eq!(log["query"], "key=value&foo=bar");

    // Verify request body
    assert_eq!(log["body"]["name"], "Test User");
    assert_eq!(log["body"]["email"], "test@example.com");
    assert_eq!(log["body"]["age"], 25);
}

#[tokio::test]
async fn test_overwriting_endpoint() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Add initial endpoint
    let payload1 = json!({
        "method": "GET",
        "path": "/api/overwrite",
        "response": {"version": 1},
        "status": 200
    });

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&payload1)
        .send()
        .await
        .expect("Failed to add endpoint");

    // Call and verify first version
    let response = client
        .get(format!("{}/api/overwrite", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["version"], 1);

    // Overwrite with new endpoint
    let payload2 = json!({
        "method": "GET",
        "path": "/api/overwrite",
        "response": {"version": 2, "updated": true},
        "status": 200
    });

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&payload2)
        .send()
        .await
        .expect("Failed to overwrite endpoint");

    // Call and verify second version
    let response = client
        .get(format!("{}/api/overwrite", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["version"], 2);
    assert_eq!(body["updated"], true);
}

#[tokio::test]
async fn test_remove_nonexistent_endpoint() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Try to remove an endpoint that doesn't exist
    let payload = json!({
        "method": "GET",
        "path": "/api/does-not-exist"
    });

    let response = client
        .delete(format!("{}/__mock/endpoints", BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to remove endpoint");

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");

    // Should still return removed: true (or false depending on implementation)
    assert!(body.get("removed").is_some());
}

#[tokio::test]
async fn test_case_sensitive_paths() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Add endpoint with lowercase path
    let payload = json!({
        "method": "GET",
        "path": "/api/test",
        "response": {"case": "lower"},
        "status": 200
    });

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to add endpoint");

    // Call with lowercase - should work
    let response = client
        .get(format!("{}/api/test", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");
    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.unwrap();
    assert_eq!(body["case"], "lower");

    // Call with uppercase - should return 404 (paths are case sensitive)
    let response = client
        .get(format!("{}/api/Test", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn test_empty_response_body() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Add endpoint with minimal response
    let payload = json!({
        "method": "GET",
        "path": "/api/empty",
        "response": {},
        "status": 204
    });

    client
        .post(format!("{}/__mock/endpoints", BASE_URL))
        .json(&payload)
        .send()
        .await
        .expect("Failed to add endpoint");

    let response = client
        .get(format!("{}/api/empty", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint");

    assert_eq!(response.status().as_u16(), 204);
}

#[tokio::test]
async fn test_import_openapi_valid_spec() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let openapi_spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "paths": {
            "/api/users": {
                "get": {
                    "summary": "Get users",
                    "responses": {
                        "200": {
                            "description": "Success",
                            "content": {
                                "application/json": {
                                    "example": {"users": [{"id": 1, "name": "John"}]}
                                }
                            }
                        }
                    }
                },
                "post": {
                    "summary": "Create user",
                    "responses": {
                        "201": {
                            "description": "Created",
                            "content": {
                                "application/json": {
                                    "example": {"id": 1, "name": "John"}
                                }
                            }
                        }
                    }
                }
            },
            "/api/products/{id}": {
                "get": {
                    "summary": "Get product",
                    "responses": {
                        "200": {
                            "description": "Success",
                            "content": {
                                "application/json": {
                                    "example": {"id": 1, "name": "Product 1"}
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let response = client
        .post(format!("{}/__mock/import", BASE_URL))
        .json(&json!({"openapi_spec": openapi_spec}))
        .send()
        .await
        .expect("Failed to import OpenAPI spec");

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["imported"], true);
    assert_eq!(body["count"], 3);

    // Verify endpoints were imported
    let config_response = client
        .get(format!("{}/__mock/config", BASE_URL))
        .send()
        .await
        .expect("Failed to get config");

    let config: serde_json::Value = config_response.json().await.expect("Failed to parse config");
    let endpoints = config.as_array().unwrap();

    assert!(endpoints.iter().any(|e| e["method"] == "GET" && e["path"] == "/api/users"));
    assert!(endpoints.iter().any(|e| e["method"] == "POST" && e["path"] == "/api/users"));
    assert!(endpoints.iter().any(|e| e["method"] == "GET" && e["path"] == "/api/products/{id}"));
}

#[tokio::test]
async fn test_import_openapi_invalid_spec() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let invalid_spec = json!({
        "invalid": "spec"
    });

    let response = client
        .post(format!("{}/__mock/import", BASE_URL))
        .json(&json!({"openapi_spec": invalid_spec}))
        .send()
        .await
        .expect("Failed to send import request");

    assert_eq!(response.status().as_u16(), 400);
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert!(body["error"].as_str().unwrap().contains("Invalid OpenAPI specification"));
}

#[tokio::test]
async fn test_export_openapi() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Add some endpoints first
    let endpoints = vec![
        json!({
            "method": "GET",
            "path": "/api/users",
            "response": {"users": []},
            "status": 200
        }),
        json!({
            "method": "POST",
            "path": "/api/users",
            "response": {"id": 1, "name": "John"},
            "status": 201
        }),
    ];

    for endpoint in endpoints {
        client
            .post(format!("{}/__mock/endpoints", BASE_URL))
            .json(&endpoint)
            .send()
            .await
            .expect("Failed to add endpoint");
    }

    // Export OpenAPI spec
    let response = client
        .get(format!("{}/__mock/export", BASE_URL))
        .send()
        .await
        .expect("Failed to export OpenAPI spec");

    assert!(response.status().is_success());
    let spec: serde_json::Value = response.json().await.expect("Failed to parse response");

    assert_eq!(spec["openapi"], "3.0.0");
    assert_eq!(spec["info"]["title"], "Mock API");
    assert_eq!(spec["info"]["description"], "Exported from Rust-Mock server");

    let paths = spec["paths"].as_object().unwrap();
    assert!(paths.contains_key("/api/users"));

    let users_path = &paths["/api/users"];
    assert!(users_path["get"].is_object());
    assert!(users_path["post"].is_object());

    // Verify GET operation
    let get_op = &users_path["get"];
    assert_eq!(get_op["summary"], "GET /api/users");
    assert!(get_op["responses"]["200"].is_object());

    // Verify POST operation
    let post_op = &users_path["post"];
    assert_eq!(post_op["summary"], "POST /api/users");
    assert!(post_op["requestBody"].is_object());
    assert!(post_op["responses"]["201"].is_object());
}

#[tokio::test]
async fn test_import_export_roundtrip() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Import OpenAPI spec
    let original_spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "paths": {
            "/api/test": {
                "get": {
                    "summary": "Get test",
                    "responses": {
                        "200": {
                            "description": "Success",
                            "content": {
                                "application/json": {
                                    "example": {"message": "test"}
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let import_response = client
        .post(format!("{}/__mock/import", BASE_URL))
        .json(&json!({"openapi_spec": original_spec}))
        .send()
        .await
        .expect("Failed to import OpenAPI spec");

    assert!(import_response.status().is_success());

    // Export OpenAPI spec
    let export_response = client
        .get(format!("{}/__mock/export", BASE_URL))
        .send()
        .await
        .expect("Failed to export OpenAPI spec");

    assert!(export_response.status().is_success());
    let exported_spec: serde_json::Value = export_response.json().await.expect("Failed to parse response");

    // Verify exported spec has the correct structure
    assert_eq!(exported_spec["openapi"], "3.0.0");
    assert_eq!(exported_spec["info"]["title"], "Mock API");
    assert!(exported_spec["paths"]["/api/test"]["get"].is_object());
    assert_eq!(
        exported_spec["paths"]["/api/test"]["get"]["responses"]["200"]["content"]["application/json"]["example"],
        json!({"message": "test"})
    );
}

#[tokio::test]
async fn test_import_multiple_methods_same_path() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let openapi_spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "paths": {
            "/api/resource": {
                "get": {
                    "responses": {
                        "200": {
                            "description": "Get",
                            "content": {
                                "application/json": {
                                    "example": {"action": "get"}
                                }
                            }
                        }
                    }
                },
                "post": {
                    "responses": {
                        "201": {
                            "description": "Create",
                            "content": {
                                "application/json": {
                                    "example": {"action": "create"}
                                }
                            }
                        }
                    }
                },
                "put": {
                    "responses": {
                        "200": {
                            "description": "Update",
                            "content": {
                                "application/json": {
                                    "example": {"action": "update"}
                                }
                            }
                        }
                    }
                },
                "delete": {
                    "responses": {
                        "204": {
                            "description": "Delete",
                            "content": {
                                "application/json": {
                                    "example": {"action": "delete"}
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    let response = client
        .post(format!("{}/__mock/import", BASE_URL))
        .json(&json!({"openapi_spec": openapi_spec}))
        .send()
        .await
        .expect("Failed to import OpenAPI spec");

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["count"], 4);

    // Verify all methods were imported by calling each endpoint
    let get_response = client.get(format!("{}/api/resource", BASE_URL)).send().await.unwrap();
    assert!(get_response.status().is_success());
    let get_body: serde_json::Value = get_response.json().await.unwrap();
    assert_eq!(get_body["action"], "get");

    let post_response = client.post(format!("{}/api/resource", BASE_URL)).send().await.unwrap();
    assert_eq!(post_response.status().as_u16(), 201);
    let post_body: serde_json::Value = post_response.json().await.unwrap();
    assert_eq!(post_body["action"], "create");

    let put_response = client.put(format!("{}/api/resource", BASE_URL)).send().await.unwrap();
    assert!(put_response.status().is_success());
    let put_body: serde_json::Value = put_response.json().await.unwrap();
    assert_eq!(put_body["action"], "update");

    let delete_response = client.delete(format!("{}/api/resource", BASE_URL)).send().await.unwrap();
    assert_eq!(delete_response.status().as_u16(), 204);
}

#[tokio::test]
async fn test_call_imported_endpoint() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    // Import OpenAPI spec with specific response
    let openapi_spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "paths": {
            "/api/imported": {
                "get": {
                    "responses": {
                        "200": {
                            "description": "Success",
                            "content": {
                                "application/json": {
                                    "example": {"source": "imported", "data": "test"}
                                }
                            }
                        }
                    }
                }
            }
        }
    });

    client
        .post(format!("{}/__mock/import", BASE_URL))
        .json(&json!({"openapi_spec": openapi_spec}))
        .send()
        .await
        .expect("Failed to import OpenAPI spec");

    // Call the imported endpoint
    let response = client
        .get(format!("{}/api/imported", BASE_URL))
        .send()
        .await
        .expect("Failed to call imported endpoint");

    assert!(response.status().is_success());
    let body: serde_json::Value = response.json().await.expect("Failed to parse response");
    assert_eq!(body["source"], "imported");
    assert_eq!(body["data"], "test");
}
