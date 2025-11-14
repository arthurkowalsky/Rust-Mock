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
    // Request data
    pub method: String,
    pub path: String,
    pub request_headers: HashMap<String, String>,
    pub query: String,
    pub request_body: Option<Value>,

    // Response data
    pub status: u16,
    pub response_body: Option<Value>,
    pub response_headers: HashMap<String, String>,

    // Metadata
    pub timestamp: String,
    pub matched_endpoint: Option<String>,
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
    // Try common success status codes
    for status_code in [200, 201, 204, 202] {
        if let Some(item) = op.responses.responses.get(&StatusCode::Code(status_code)) {
            if let ReferenceOr::Item(resp) = item {
                if let Some(media) = resp.content.get("application/json") {
                    if let Some(example) = &media.example {
                        return Some(example.clone());
                    }
                }
            }
        }
    }
    None
}

fn extract_example_response_for_status(op: &Operation, status: u16) -> Option<Value> {
    if let Some(item) = op.responses.responses.get(&StatusCode::Code(status)) {
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

fn matches_path_template(template: &str, actual_path: &str) -> bool {
    // Convert OpenAPI path template to regex pattern
    // e.g., "/update-plan/{request_hash}" -> "/update-plan/(?P<request_hash>[^/]+)"
    let regex_pattern = template.replace('{', "(?P<").replace('}', ">[^/]+)");
    match Regex::new(&format!("^{}$", regex_pattern)) {
        Ok(re) => re.is_match(actual_path),
        Err(_) => false,
    }
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
                    // Extract status code (default to 200)
                    let status = if op.responses.responses.contains_key(&StatusCode::Code(201)) {
                        201
                    } else if op.responses.responses.contains_key(&StatusCode::Code(204)) {
                        204
                    } else if op.responses.responses.contains_key(&StatusCode::Code(202)) {
                        202
                    } else {
                        200
                    };

                    // Extract response example for the detected status code
                    let response = extract_example_response_for_status(op, status)
                        .unwrap_or_else(|| json!({"message": "OK"}));

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
    let request_headers = req.headers().iter().map(|(k,v)| (k.to_string(), v.to_str().unwrap_or("").to_string())).collect::<HashMap<_,_>>();
    let query = req.query_string().to_string();
    let request_body = serde_json::from_slice::<Value>(&body).ok();
    info!("Request {} {} headers={:?} query={} body={:?}", method, path, request_headers, query, request_body);

    // Try exact match first in dynamic endpoints
    let mut matched_endpoint: Option<DynamicEndpoint> = None;
    let mut matched_pattern: Option<String> = None;
    {
        let dyn_map = data.dynamic.lock().unwrap();
        if let Some(ep) = dyn_map.get(&(method.clone(), path.clone())) {
            matched_endpoint = Some(ep.clone());
            matched_pattern = Some(path.clone());
        } else {
            // Try path template matching for dynamic endpoints with parameters
            for ((m, p), ep) in dyn_map.iter() {
                if m == &method && matches_path_template(p, &path) {
                    matched_endpoint = Some(ep.clone());
                    matched_pattern = Some(format!("{} (template)", p));
                    info!("Matched path template: {} matches {}", p, path);
                    break;
                }
            }
        }
    }

    // Capture response data for logging
    let mut response_body: Option<Value> = None;
    let mut response_headers = HashMap::new();
    let status: u16;

    let response = if let Some(ep) = matched_endpoint {
        status = ep.status;
        response_body = Some(ep.response.clone());

        // Add custom headers if present
        if let Some(custom_headers) = &ep.headers {
            response_headers.extend(custom_headers.clone());
        }
        response_headers.insert("content-type".to_string(), "application/json".to_string());

        HttpResponse::build(actix_web::http::StatusCode::from_u16(ep.status).unwrap()).json(&ep.response)
    } else if let Some(spec) = &data.spec {
        if let Some(op) = get_operation(spec, &method, &path) {
            if let Some(example) = extract_example_response(&op) {
                status = 200;
                response_body = Some(example.clone());
                response_headers.insert("content-type".to_string(), "application/json".to_string());
                matched_pattern = Some("OpenAPI spec".to_string());
                HttpResponse::Ok().content_type("application/json").body(example.to_string())
            } else {
                status = 200;
                HttpResponse::Ok().finish()
            }
        } else {
            status = 404;
            HttpResponse::NotFound().finish()
        }
    } else {
        status = 404;
        HttpResponse::NotFound().finish()
    };

    info!("Responded {} {} -> {}", method, path, status);

    // Log with full request and response data
    data.logs.lock().unwrap().push(RequestLog {
        method,
        path,
        request_headers,
        query,
        request_body,
        status,
        response_body,
        response_headers,
        timestamp,
        matched_endpoint: matched_pattern,
    });

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
