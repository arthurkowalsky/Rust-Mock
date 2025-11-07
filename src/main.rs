use actix_files::Files;
use actix_web::{middleware::Logger, guard, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use chrono::Local;
use clap::Parser;
use env_logger::Builder;
use log::{info, LevelFilter};
use openapiv3::{OpenAPI, Operation, ReferenceOr, StatusCode};
use regex::Regex;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::{HashMap, HashSet}, env, fs, sync::Mutex};

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

#[derive(Parser)]
struct Config {
    #[clap(long, default_value = "0.0.0.0")]
    host: String,
    #[clap(long, default_value = "8090")]
    port: u16,
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

fn get_operation(spec: &OpenAPI, method: &str, req_path: &str) -> Option<Operation> {
    for (tpl, item) in &spec.paths.paths {
        if let ReferenceOr::Item(path_item) = item {
            let re = Regex::new(&format!("^{}$", tpl.replace('{', "(?P<").replace('}', ">[^/]+)"))).unwrap();
            if re.is_match(req_path) {
                let op = match method {
                    "GET" => &path_item.get,
                    "POST" => &path_item.post,
                    "PUT" => &path_item.put,
                    "PATCH" => &path_item.patch,
                    "DELETE" => &path_item.delete,
                    _ => &None,
                };
                if let Some(o) = op.clone() {
                    return Some(o);
                }
            }
        }
    }
    None
}

fn get_request_schema(raw_spec: &Value, method: &str, path: &str) -> Option<Value> {
    raw_spec.get("paths")?.get(path)?.get(&method.to_lowercase())?
        .get("requestBody")?.get("content")?.get("application/json")?
        .get("schema").cloned()
}

fn extract_example_response(op: &Operation) -> Option<Value> {
    if let Some(item) = op.responses.responses.get(&StatusCode::Code(200)) {
        if let ReferenceOr::Item(resp) = item {
            if let Some(media) = resp.content.get("application/json") {
                if let Some(example) = &media.example {
                    return Some(example.clone());
                }
            }
        }
    }
    None
}

pub async fn add_endpoint(data: web::Data<AppState>, cfg: web::Json<EndpointConfig>) -> impl Responder {
    let status = cfg.status.unwrap_or(200);
    let ep = DynamicEndpoint { response: cfg.response.clone(), status, headers: cfg.headers.clone() };
    data.dynamic.lock().unwrap().insert((cfg.method.clone(), cfg.path.clone()), ep);
    info!("Added endpoint {} {}", cfg.method, cfg.path);
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
    info!("Removed endpoint {} {}: {}", cfg.method, cfg.path, removed);
    HttpResponse::Ok().json(json!({"removed": removed}))
}

pub async fn get_config(data: web::Data<AppState>) -> impl Responder {
    let mut list = Vec::new();
    if let Some(spec) = &data.spec {
        let rem = data.removed_spec.lock().unwrap();
        if let Some(raw) = &data.raw_spec {
            for (tpl, item) in &spec.paths.paths {
                if let ReferenceOr::Item(path_item) = item {
                    for (m, op_opt) in ["GET", "POST", "PUT", "PATCH", "DELETE"].iter()
                        .filter_map(|&m| Some((m, match m {"GET"=>&path_item.get,"POST"=>&path_item.post,"PUT"=>&path_item.put,"PATCH"=>&path_item.patch,"DELETE"=>&path_item.delete,_=>&None})))
                    {
                        if op_opt.is_some() && !rem.contains(&(m.to_string(), tpl.clone())) {
                            let schema = get_request_schema(raw, m, tpl);
                            let example = extract_example_response(op_opt.as_ref().unwrap());
                            list.push(json!({"method": m, "path": tpl, "request_schema": schema, "response_example": example}));
                        }
                    }
                }
            }
        }
    }
    let dyn_map = data.dynamic.lock().unwrap();
    for ((m,p), ep) in dyn_map.iter() {
        list.push(json!({"method": m, "path": p, "request_schema": null, "response": ep.response, "status": ep.status, "headers": ep.headers}));
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

#[derive(Deserialize)]
pub struct ImportRequest {
    pub openapi_spec: Value,
}

pub async fn import_openapi(data: web::Data<AppState>, req: web::Json<ImportRequest>) -> impl Responder {
    // Validate and parse OpenAPI spec
    let spec = match serde_json::from_value::<OpenAPI>(req.openapi_spec.clone()) {
        Ok(s) => s,
        Err(e) => {
            return HttpResponse::BadRequest().json(json!({
                "error": format!("Invalid OpenAPI specification: {}", e)
            }));
        }
    };

    let mut imported_count = 0;
    let mut endpoints = Vec::new();
    let mut dyn_map = data.dynamic.lock().unwrap();

    // Iterate through all paths and operations
    for (path, item) in &spec.paths.paths {
        if let ReferenceOr::Item(path_item) = item {
            // Process each HTTP method
            let methods = [
                ("GET", &path_item.get),
                ("POST", &path_item.post),
                ("PUT", &path_item.put),
                ("PATCH", &path_item.patch),
                ("DELETE", &path_item.delete),
            ];

            for (method, op_opt) in methods {
                if let Some(op) = op_opt {
                    // Extract response example or use default
                    let response = extract_example_response(op).unwrap_or_else(|| json!({"message": "OK"}));

                    // Extract status code (default to 200)
                    let status = if op.responses.responses.contains_key(&StatusCode::Code(201)) {
                        201
                    } else if op.responses.responses.contains_key(&StatusCode::Code(204)) {
                        204
                    } else {
                        200
                    };

                    let endpoint = DynamicEndpoint {
                        response,
                        status,
                        headers: Some(HashMap::from([
                            ("Content-Type".to_string(), "application/json".to_string()),
                        ])),
                    };

                    dyn_map.insert((method.to_string(), path.clone()), endpoint.clone());
                    endpoints.push(json!({
                        "method": method,
                        "path": path,
                        "status": status
                    }));
                    imported_count += 1;
                    info!("Imported endpoint {} {} with status {}", method, path, status);
                }
            }
        }
    }

    HttpResponse::Ok().json(json!({
        "imported": true,
        "count": imported_count,
        "endpoints": endpoints
    }))
}

pub async fn export_openapi(data: web::Data<AppState>) -> impl Responder {
    let mut paths_map = serde_json::Map::new();

    // Export dynamic endpoints
    let dyn_map = data.dynamic.lock().unwrap();
    for ((method, path), endpoint) in dyn_map.iter() {
        // Get or create path item
        if !paths_map.contains_key(path) {
            paths_map.insert(path.clone(), json!({}));
        }

        let path_obj = paths_map.get_mut(path).unwrap();
        let path_item = path_obj.as_object_mut().unwrap();

        // Build operation object
        let mut operation = serde_json::Map::new();
        operation.insert("summary".to_string(), json!(format!("{} {}", method, path)));
        operation.insert("operationId".to_string(), json!(format!("{}_{}", method.to_lowercase(), path.replace('/', "_").trim_matches('_'))));

        // Add request body if method supports it
        if matches!(method.as_str(), "POST" | "PUT" | "PATCH") {
            operation.insert("requestBody".to_string(), json!({
                "content": {
                    "application/json": {
                        "schema": {
                            "type": "object"
                        }
                    }
                }
            }));
        }

        // Add response
        let mut responses = serde_json::Map::new();
        responses.insert(endpoint.status.to_string(), json!({
            "description": format!("Successful response with status {}", endpoint.status),
            "content": {
                "application/json": {
                    "example": endpoint.response,
                    "schema": {
                        "type": "object"
                    }
                }
            }
        }));

        operation.insert("responses".to_string(), json!(responses));

        // Add operation to path item
        path_item.insert(method.to_lowercase(), json!(operation));
    }

    // Build OpenAPI specification
    let openapi_spec = json!({
        "openapi": "3.0.0",
        "info": {
            "title": "Mock API",
            "description": "Exported from Rust-Mock server",
            "version": "1.0.0"
        },
        "paths": paths_map
    });

    info!("Exported {} endpoints to OpenAPI format", dyn_map.len());
    HttpResponse::Ok()
        .content_type("application/json")
        .json(openapi_spec)
}

pub async fn dispatch(req: HttpRequest, body: web::Bytes, data: web::Data<AppState>) -> impl Responder {
    let method = req.method().as_str().to_uppercase();
    let path = req.path().to_string();
    let timestamp = Local::now().to_rfc3339();
    let headers = req.headers().iter().map(|(k,v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect::<HashMap<_,_>>();
    let query = req.query_string().to_string();
    let body_json = serde_json::from_slice::<Value>(&body).ok();
    info!("Request {} {} headers={:?} query={} body={:?}", method, path, headers, query, body_json);
    let response = if let Some(ep) = data.dynamic.lock().unwrap().get(&(method.clone(), path.clone())) {
        HttpResponse::build(actix_web::http::StatusCode::from_u16(ep.status).unwrap()).json(&ep.response)
    } else if let Some(spec) = &data.spec {
        if let Some(op) = get_operation(spec, &method, &path) {
            if let Some(example) = extract_example_response(&op) {
                HttpResponse::Ok().content_type("application/json").body(example.to_string())
            } else {
                HttpResponse::Ok().finish()
            }
        } else {
            HttpResponse::NotFound().finish()
        }
    } else {
        HttpResponse::NotFound().finish()
    };
    let status = response.status().as_u16();
    info!("Responded {} {} -> {}", method, path, status);
    data.logs.lock().unwrap().push(RequestLog { method, path, headers, query, body: body_json, timestamp, status });
    response
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    Builder::new().filter(None, LevelFilter::Info).init();
    let cfg = Config::parse();
    info!("Starting server host={} port={}", cfg.host, cfg.port);
    let raw = env::var("OPENAPI_FILE").ok()
        .and_then(|p| fs::read_to_string(&p).ok())
        .and_then(|s| serde_json::from_str::<Value>(&s).ok());
    let spec = raw.as_ref().and_then(|_v| serde_json::from_value::<OpenAPI>(_v.clone()).ok());
    if let (Some(_v), Some(o)) = (raw.as_ref(), spec.as_ref()) {
        let mut list = Vec::new();
        for (tpl, item) in &o.paths.paths {
            if let ReferenceOr::Item(pi) = item {
                for m in ["GET","POST","PUT","PATCH","DELETE"] {
                    let op = match m {"GET"=>&pi.get,"POST"=>&pi.post,"PUT"=>&pi.put,"PATCH"=>&pi.patch,"DELETE"=>&pi.delete,_=>&None};
                    if op.is_some() {
                        list.push(format!("{} {}", m, tpl));
                    }
                }
            }
        }
        info!("Registered OpenAPI endpoints at startup: {:?}", list);
    } else if raw.is_none() {
        info!("No OPENAPI_FILE specified");
    }
    let state = web::Data::new(AppState { dynamic: Mutex::new(HashMap::new()), removed_spec: Mutex::new(HashSet::new()), spec, raw_spec: raw, logs: Mutex::new(vec![]) });
    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .wrap(Logger::default())
            .service(web::scope("/__mock")
                .route("/endpoints", web::post().to(add_endpoint))
                .route("/endpoints", web::delete().to(remove_endpoint))
                .route("/config", web::get().to(get_config))
                .route("/logs", web::get().to(get_logs))
                .route("/logs", web::delete().to(clear_logs))
                .route("/import", web::post().to(import_openapi))
                .route("/export", web::get().to(export_openapi)))
            .service(web::scope("")
                .guard(guard::Get())
                .service(Files::new("/", "./ui/dist").index_file("index.html").default_handler(web::route().to(dispatch))))
            .default_service(web::route().to(dispatch))
    })
        .bind((cfg.host, cfg.port))?
        .run()
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, App};

    fn create_test_app_state() -> web::Data<AppState> {
        web::Data::new(AppState {
            dynamic: Mutex::new(HashMap::new()),
            removed_spec: Mutex::new(HashSet::new()),
            spec: None,
            raw_spec: None,
            logs: Mutex::new(vec![]),
        })
    }

    #[actix_web::test]
    async fn test_import_openapi_valid_spec() {
        let state = create_test_app_state();
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/import", web::post().to(import_openapi))
        ).await;

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

        let req = test::TestRequest::post()
            .uri("/import")
            .set_json(json!({"openapi_spec": openapi_spec}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["imported"], true);
        assert_eq!(body["count"], 3);

        // Verify endpoints were imported
        let dyn_map = state.dynamic.lock().unwrap();
        assert!(dyn_map.contains_key(&("GET".to_string(), "/api/users".to_string())));
        assert!(dyn_map.contains_key(&("POST".to_string(), "/api/users".to_string())));
        assert!(dyn_map.contains_key(&("GET".to_string(), "/api/products/{id}".to_string())));
    }

    #[actix_web::test]
    async fn test_import_openapi_invalid_spec() {
        let state = create_test_app_state();
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/import", web::post().to(import_openapi))
        ).await;

        let invalid_spec = json!({
            "invalid": "spec"
        });

        let req = test::TestRequest::post()
            .uri("/import")
            .set_json(json!({"openapi_spec": invalid_spec}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);

        let body: Value = test::read_body_json(resp).await;
        assert!(body["error"].as_str().unwrap().contains("Invalid OpenAPI specification"));
    }

    #[actix_web::test]
    async fn test_export_openapi_empty() {
        let state = create_test_app_state();
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/export", web::get().to(export_openapi))
        ).await;

        let req = test::TestRequest::get()
            .uri("/export")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["openapi"], "3.0.0");
        assert_eq!(body["info"]["title"], "Mock API");
        assert_eq!(body["paths"].as_object().unwrap().len(), 0);
    }

    #[actix_web::test]
    async fn test_export_openapi_with_endpoints() {
        let state = create_test_app_state();

        // Add some endpoints
        {
            let mut dyn_map = state.dynamic.lock().unwrap();
            dyn_map.insert(
                ("GET".to_string(), "/api/users".to_string()),
                DynamicEndpoint {
                    response: json!({"users": []}),
                    status: 200,
                    headers: Some(HashMap::from([
                        ("Content-Type".to_string(), "application/json".to_string()),
                    ])),
                }
            );
            dyn_map.insert(
                ("POST".to_string(), "/api/users".to_string()),
                DynamicEndpoint {
                    response: json!({"id": 1, "name": "John"}),
                    status: 201,
                    headers: Some(HashMap::from([
                        ("Content-Type".to_string(), "application/json".to_string()),
                    ])),
                }
            );
        }

        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/export", web::get().to(export_openapi))
        ).await;

        let req = test::TestRequest::get()
            .uri("/export")
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["openapi"], "3.0.0");
        assert_eq!(body["info"]["title"], "Mock API");

        let paths = body["paths"].as_object().unwrap();
        assert_eq!(paths.len(), 1);
        assert!(paths.contains_key("/api/users"));

        let users_path = &paths["/api/users"];
        assert!(users_path["get"].is_object());
        assert!(users_path["post"].is_object());

        // Verify GET operation
        let get_op = &users_path["get"];
        assert_eq!(get_op["summary"], "GET /api/users");
        assert!(get_op["responses"]["200"].is_object());
        assert_eq!(get_op["responses"]["200"]["content"]["application/json"]["example"], json!({"users": []}));

        // Verify POST operation
        let post_op = &users_path["post"];
        assert_eq!(post_op["summary"], "POST /api/users");
        assert!(post_op["requestBody"].is_object());
        assert!(post_op["responses"]["201"].is_object());
        assert_eq!(post_op["responses"]["201"]["content"]["application/json"]["example"], json!({"id": 1, "name": "John"}));
    }

    #[actix_web::test]
    async fn test_import_export_roundtrip() {
        let state = create_test_app_state();

        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/import", web::post().to(import_openapi))
                .route("/export", web::get().to(export_openapi))
        ).await;

        // Step 1: Import OpenAPI spec
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

        let import_req = test::TestRequest::post()
            .uri("/import")
            .set_json(json!({"openapi_spec": original_spec}))
            .to_request();

        let import_resp = test::call_service(&app, import_req).await;
        assert_eq!(import_resp.status(), 200);

        // Step 2: Export OpenAPI spec
        let export_req = test::TestRequest::get()
            .uri("/export")
            .to_request();

        let export_resp = test::call_service(&app, export_req).await;
        assert_eq!(export_resp.status(), 200);

        let exported_spec: Value = test::read_body_json(export_resp).await;

        // Verify exported spec has the correct structure
        assert_eq!(exported_spec["openapi"], "3.0.0");
        assert_eq!(exported_spec["info"]["title"], "Mock API");
        assert!(exported_spec["paths"]["/api/test"]["get"].is_object());
        assert_eq!(
            exported_spec["paths"]["/api/test"]["get"]["responses"]["200"]["content"]["application/json"]["example"],
            json!({"message": "test"})
        );
    }

    #[actix_web::test]
    async fn test_import_multiple_methods_same_path() {
        let state = create_test_app_state();
        let app = test::init_service(
            App::new()
                .app_data(state.clone())
                .route("/import", web::post().to(import_openapi))
        ).await;

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

        let req = test::TestRequest::post()
            .uri("/import")
            .set_json(json!({"openapi_spec": openapi_spec}))
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: Value = test::read_body_json(resp).await;
        assert_eq!(body["count"], 4);

        // Verify all methods were imported
        let dyn_map = state.dynamic.lock().unwrap();
        assert!(dyn_map.contains_key(&("GET".to_string(), "/api/resource".to_string())));
        assert!(dyn_map.contains_key(&("POST".to_string(), "/api/resource".to_string())));
        assert!(dyn_map.contains_key(&("PUT".to_string(), "/api/resource".to_string())));
        assert!(dyn_map.contains_key(&("DELETE".to_string(), "/api/resource".to_string())));

        // Verify correct status codes
        assert_eq!(dyn_map.get(&("GET".to_string(), "/api/resource".to_string())).unwrap().status, 200);
        assert_eq!(dyn_map.get(&("POST".to_string(), "/api/resource".to_string())).unwrap().status, 201);
        assert_eq!(dyn_map.get(&("PUT".to_string(), "/api/resource".to_string())).unwrap().status, 200);
        assert_eq!(dyn_map.get(&("DELETE".to_string(), "/api/resource".to_string())).unwrap().status, 204);
    }
}
