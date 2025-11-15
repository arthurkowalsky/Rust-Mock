use super::common::{TestServer, BASE_URL};
use serde_json::json;

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

    let get_op = &users_path["get"];
    assert_eq!(get_op["summary"], "GET /api/users");
    assert!(get_op["responses"]["200"].is_object());

    let post_op = &users_path["post"];
    assert_eq!(post_op["summary"], "POST /api/users");
    assert!(post_op["requestBody"].is_object());
    assert!(post_op["responses"]["201"].is_object());
}

#[tokio::test]
async fn test_import_export_roundtrip() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

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

    let export_response = client
        .get(format!("{}/__mock/export", BASE_URL))
        .send()
        .await
        .expect("Failed to export OpenAPI spec");

    assert!(export_response.status().is_success());
    let exported_spec: serde_json::Value = export_response.json().await.expect("Failed to parse response");

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

#[tokio::test]
async fn test_import_openapi_with_path_parameters() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let openapi_spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Test API",
            "version": "1.0.0"
        },
        "paths": {
            "/update-plan/{request_hash}": {
                "post": {
                    "parameters": [
                        {
                            "name": "request_hash",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Success",
                            "content": {
                                "application/json": {
                                    "example": {
                                        "status": "updated",
                                        "request_hash": "abc123"
                                    }
                                }
                            }
                        }
                    }
                }
            },
            "/users/{user_id}/posts/{post_id}": {
                "get": {
                    "parameters": [
                        {
                            "name": "user_id",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        },
                        {
                            "name": "post_id",
                            "in": "path",
                            "required": true,
                            "schema": {
                                "type": "string"
                            }
                        }
                    ],
                    "responses": {
                        "200": {
                            "description": "Success",
                            "content": {
                                "application/json": {
                                    "example": {
                                        "user_id": "123",
                                        "post_id": "456",
                                        "title": "Test Post"
                                    }
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
        .json(&json!({"openapi_spec": openapi_spec}))
        .send()
        .await
        .expect("Failed to import OpenAPI spec");

    assert!(import_response.status().is_success());
    let body: serde_json::Value = import_response.json().await.expect("Failed to parse response");
    assert_eq!(body["count"], 2);

    let response1 = client
        .post(format!("{}/update-plan/abc123", BASE_URL))
        .json(&json!({"some": "data"}))
        .send()
        .await
        .expect("Failed to call endpoint with path parameter");

    assert!(response1.status().is_success(), "Expected success but got: {}", response1.status());
    let body1: serde_json::Value = response1.json().await.expect("Failed to parse response");
    assert_eq!(body1["status"], "updated");
    assert_eq!(body1["request_hash"], "abc123");

    let response2 = client
        .post(format!("{}/update-plan/xyz789", BASE_URL))
        .json(&json!({"some": "data"}))
        .send()
        .await
        .expect("Failed to call endpoint with different path parameter");

    assert!(response2.status().is_success());
    let body2: serde_json::Value = response2.json().await.expect("Failed to parse response");
    assert_eq!(body2["status"], "updated");

    let response3 = client
        .get(format!("{}/users/123/posts/456", BASE_URL))
        .send()
        .await
        .expect("Failed to call endpoint with multiple path parameters");

    assert!(response3.status().is_success(), "Expected success but got: {}", response3.status());
    let body3: serde_json::Value = response3.json().await.expect("Failed to parse response");
    assert_eq!(body3["user_id"], "123");
    assert_eq!(body3["post_id"], "456");
    assert_eq!(body3["title"], "Test Post");

    let response4 = client
        .get(format!("{}/users/123/comments/456", BASE_URL))
        .send()
        .await
        .expect("Failed to call non-existent endpoint");

    assert_eq!(response4.status().as_u16(), 404);
}

#[tokio::test]
async fn test_import_comprehensive_openapi_spec() {
    let _server = TestServer::start().await;
    let client = reqwest::Client::new();

    let spec_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/openapi-test.json");
    let spec_content = std::fs::read_to_string(&spec_path)
        .expect("Failed to read openapi-test.json");
    let openapi_spec: serde_json::Value = serde_json::from_str(&spec_content)
        .expect("Failed to parse openapi-test.json");

    let import_response = client
        .post(format!("{}/__mock/import", BASE_URL))
        .json(&json!({"openapi_spec": openapi_spec}))
        .send()
        .await
        .expect("Failed to import comprehensive OpenAPI spec");

    assert!(import_response.status().is_success());
    let import_body: serde_json::Value = import_response.json().await.expect("Failed to parse import response");
    println!("Imported {} endpoints", import_body["count"]);
    assert!(import_body["count"].as_i64().unwrap() > 10, "Expected at least 10 endpoints");

    let resp1 = client.get(format!("{}/api/health", BASE_URL)).send().await.unwrap();
    assert!(resp1.status().is_success());
    let body1: serde_json::Value = resp1.json().await.unwrap();
    assert_eq!(body1["status"], "healthy");

    let resp2 = client.get(format!("{}/api/users/42", BASE_URL)).send().await.unwrap();
    assert!(resp2.status().is_success());
    let body2: serde_json::Value = resp2.json().await.unwrap();
    assert!(body2["id"].is_number() || body2["id"].is_string());

    let resp3 = client.get(format!("{}/api/users/1/posts/5", BASE_URL)).send().await.unwrap();
    assert!(resp3.status().is_success());
    let body3: serde_json::Value = resp3.json().await.unwrap();
    assert!(body3["id"].is_number() || body3["user_id"].is_number());

    let resp4 = client
        .post(format!("{}/api/users", BASE_URL))
        .json(&json!({"name": "Test User", "email": "test@example.com"}))
        .send()
        .await
        .unwrap();
    assert_eq!(resp4.status().as_u16(), 201);
    let body4: serde_json::Value = resp4.json().await.unwrap();
    assert!(body4["id"].is_number() || body4["name"].is_string());

    let resp5 = client.delete(format!("{}/api/users/999", BASE_URL)).send().await.unwrap();
    assert_eq!(resp5.status().as_u16(), 204);

    let resp6 = client
        .put(format!("{}/api/users/123", BASE_URL))
        .json(&json!({"name": "Updated Name"}))
        .send()
        .await
        .unwrap();
    assert!(resp6.status().is_success());

    let resp7 = client
        .patch(format!("{}/api/orders/order-123/items/item-456", BASE_URL))
        .json(&json!({"quantity": 5}))
        .send()
        .await
        .unwrap();
    assert!(resp7.status().is_success());
    let body7: serde_json::Value = resp7.json().await.unwrap();
    assert_eq!(body7["order_id"], "order-456");
    assert_eq!(body7["item_id"], "item-789");

    let export_resp = client
        .get(format!("{}/__mock/export", BASE_URL))
        .send()
        .await
        .unwrap();
    assert!(export_resp.status().is_success());
    let exported: serde_json::Value = export_resp.json().await.unwrap();
    assert_eq!(exported["openapi"], "3.0.0");
    let paths = exported["paths"].as_object().unwrap();
    println!("Exported paths count: {}", paths.len());
    println!("Exported paths: {:?}", paths.keys().collect::<Vec<_>>());

    assert!(paths.len() >= 8, "Exported spec should contain imported endpoints, got {} paths", paths.len());
    assert!(paths.contains_key("/api/health"));
    assert!(paths.contains_key("/api/users/{user_id}"));
    assert!(paths.contains_key("/api/users/{user_id}/posts/{post_id}"));
}
