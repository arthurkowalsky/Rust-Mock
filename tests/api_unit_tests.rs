use actix_web::{test, web, App};
use serde_json::json;
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;

mod app {
    pub use actix_web::{web, HttpRequest, HttpResponse, Responder};
    pub use serde::{Deserialize, Serialize};
    pub use serde_json::{json, Value};
    pub use std::collections::{HashMap, HashSet};
    pub use std::sync::Mutex;
    pub use openapiv3::OpenAPI;

    #[derive(Clone)]
    pub struct DynamicEndpoint {
        pub response: Value,
        pub status: u16,
        pub headers: Option<HashMap<String, String>>,
    }

    pub struct AppState {
        pub dynamic: Mutex<HashMap<(String, String), DynamicEndpoint>>,
        pub removed_spec: Mutex<HashSet<(String, String)>>,
        pub spec: Option<OpenAPI>,
        pub raw_spec: Option<Value>,
        pub logs: Mutex<Vec<RequestLog>>,
    }

    #[derive(Serialize, Clone)]
    pub struct RequestLog {
        pub method: String,
        pub path: String,
        pub headers: HashMap<String, String>,
        pub query: String,
        pub body: Option<Value>,
        pub timestamp: String,
        pub status: u16,
    }

    #[derive(Deserialize)]
    pub struct EndpointConfig {
        pub method: String,
        pub path: String,
        pub response: Value,
        pub status: Option<u16>,
        pub headers: Option<HashMap<String, String>>,
    }

    #[derive(Deserialize)]
    pub struct RemoveConfig {
        pub method: String,
        pub path: String,
    }

    pub async fn add_endpoint(data: web::Data<AppState>, cfg: web::Json<EndpointConfig>) -> impl Responder {
        let status = cfg.status.unwrap_or(200);
        let ep = DynamicEndpoint {
            response: cfg.response.clone(),
            status,
            headers: cfg.headers.clone()
        };
        data.dynamic.lock().unwrap().insert((cfg.method.clone(), cfg.path.clone()), ep);
        HttpResponse::Ok().json(json!({"added": true}))
    }

    pub async fn remove_endpoint(data: web::Data<AppState>, cfg: web::Json<RemoveConfig>) -> impl Responder {
        let mut dyn_map = data.dynamic.lock().unwrap();
        let mut rem_spec = data.removed_spec.lock().unwrap();
        let key = (cfg.method.clone(), cfg.path.clone());
        let removed = if dyn_map.remove(&key).is_some() {
            true
        } else {
            rem_spec.insert(key)
        };
        HttpResponse::Ok().json(json!({"removed": removed}))
    }

    pub async fn get_config(data: web::Data<AppState>) -> impl Responder {
        let mut list = Vec::new();
        let dyn_map = data.dynamic.lock().unwrap();
        for ((m, p), ep) in dyn_map.iter() {
            list.push(json!({
                "method": m,
                "path": p,
                "response": ep.response,
                "status": ep.status,
                "headers": ep.headers
            }));
        }
        HttpResponse::Ok().json(list)
    }

    pub async fn get_logs(data: web::Data<AppState>) -> impl Responder {
        let logs = data.logs.lock().unwrap();
        HttpResponse::Ok().json(&*logs)
    }

    pub async fn clear_logs(data: web::Data<AppState>) -> impl Responder {
        data.logs.lock().unwrap().clear();
        HttpResponse::Ok().json(json!({"cleared": true}))
    }
}

fn create_test_app_state() -> web::Data<app::AppState> {
    web::Data::new(app::AppState {
        dynamic: Mutex::new(HashMap::new()),
        removed_spec: Mutex::new(HashSet::new()),
        spec: None,
        raw_spec: None,
        logs: Mutex::new(vec![]),
    })
}

#[actix_web::test]
async fn test_add_endpoint() {
    let app_state = create_test_app_state();
    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/__mock/endpoints", web::post().to(app::add_endpoint))
    ).await;

    let payload = json!({
        "method": "GET",
        "path": "/test-path",
        "response": {"message": "test response"},
        "status": 200
    });

    let req = test::TestRequest::post()
        .uri("/__mock/endpoints")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["added"], true);
}

#[actix_web::test]
async fn test_remove_endpoint() {
    let app_state = create_test_app_state();

    {
        let mut dynamic = app_state.dynamic.lock().unwrap();
        dynamic.insert(
            ("GET".to_string(), "/test-remove".to_string()),
            app::DynamicEndpoint {
                response: json!({"message": "test"}),
                status: 200,
                headers: None,
            }
        );
    }

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/__mock/endpoints", web::delete().to(app::remove_endpoint))
    ).await;

    let payload = json!({
        "method": "GET",
        "path": "/test-remove"
    });

    let req = test::TestRequest::delete()
        .uri("/__mock/endpoints")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["removed"], true);
}

#[actix_web::test]
async fn test_get_config() {
    let app_state = create_test_app_state();

    {
        let mut dynamic = app_state.dynamic.lock().unwrap();
        dynamic.insert(
            ("POST".to_string(), "/api/test".to_string()),
            app::DynamicEndpoint {
                response: json!({"data": "value"}),
                status: 201,
                headers: None,
            }
        );
    }

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/__mock/config", web::get().to(app::get_config))
    ).await;

    let req = test::TestRequest::get()
        .uri("/__mock/config")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
    let endpoints = body.as_array().unwrap();
    assert_eq!(endpoints.len(), 1);
    assert_eq!(endpoints[0]["method"], "POST");
    assert_eq!(endpoints[0]["path"], "/api/test");
}

#[actix_web::test]
async fn test_get_logs() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/__mock/logs", web::get().to(app::get_logs))
    ).await;

    let req = test::TestRequest::get()
        .uri("/__mock/logs")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(body.is_array());
}

#[actix_web::test]
async fn test_clear_logs() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/__mock/logs", web::delete().to(app::clear_logs))
    ).await;

    let req = test::TestRequest::delete()
        .uri("/__mock/logs")
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["cleared"], true);
}

#[actix_web::test]
async fn test_endpoint_with_custom_status() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/__mock/endpoints", web::post().to(app::add_endpoint))
            .route("/__mock/config", web::get().to(app::get_config))
    ).await;

    let payload = json!({
        "method": "POST",
        "path": "/api/resource",
        "response": {"id": 1},
        "status": 201
    });

    let req = test::TestRequest::post()
        .uri("/__mock/endpoints")
        .set_json(&payload)
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let req2 = test::TestRequest::get()
        .uri("/__mock/config")
        .to_request();

    let resp2 = test::call_service(&app, req2).await;
    let body: serde_json::Value = test::read_body_json(resp2).await;
    let endpoints = body.as_array().unwrap();

    let added_endpoint = endpoints.iter()
        .find(|e| e["path"] == "/api/resource")
        .expect("Should find added endpoint");

    assert_eq!(added_endpoint["status"], 201);
}

#[actix_web::test]
async fn test_multiple_endpoints() {
    let app_state = create_test_app_state();

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/__mock/endpoints", web::post().to(app::add_endpoint))
            .route("/__mock/config", web::get().to(app::get_config))
    ).await;

    for i in 1..=3 {
        let payload = json!({
            "method": "GET",
            "path": format!("/api/endpoint{}", i),
            "response": {"id": i},
            "status": 200
        });

        let req = test::TestRequest::post()
            .uri("/__mock/endpoints")
            .set_json(&payload)
            .to_request();

        test::call_service(&app, req).await;
    }

    let req = test::TestRequest::get()
        .uri("/__mock/config")
        .to_request();

    let resp = test::call_service(&app, req).await;
    let body: serde_json::Value = test::read_body_json(resp).await;
    let endpoints = body.as_array().unwrap();

    assert_eq!(endpoints.len(), 3);
}
