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
struct RequestLog {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    query: String,
    body: Option<Value>,
    timestamp: String,
    status: u16,
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
struct EndpointConfig {
    method: String,
    path: String,
    response: Value,
    status: Option<u16>,
    headers: Option<HashMap<String, String>>,
}

#[derive(Deserialize)]
struct RemoveConfig {
    method: String,
    path: String,
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
                .route("/logs", web::delete().to(clear_logs)))
            .service(web::scope("")
                .guard(guard::Get())
                .service(Files::new("/", "./ui/dist").index_file("index.html").default_handler(web::route().to(dispatch))))
            .default_service(web::route().to(dispatch))
    })
        .bind((cfg.host, cfg.port))?
        .run()
        .await
}
