use std::collections::HashMap;
use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::net::SocketAddr;
use std::sync::Mutex;

extern crate actix_web;
use actix_web::{http, server, App, Error, HttpRequest, HttpResponse};

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate log4rs;

#[macro_use]
extern crate serde_json;

type Namespace = HashMap<String, String>;

fn make_namespace(name: &String) -> Namespace {
    let mut namespace: Namespace = HashMap::new();
    namespace.insert("name".to_string(), (*name).clone());
    namespace
}

lazy_static! {
    static ref NAMESPACES: Mutex<HashMap<String, Namespace>> = Mutex::new(HashMap::new());
}

fn json_body(content: &serde_json::Value) -> String {
    format!("{}\n", serde_json::to_string_pretty(content).unwrap())
}

fn handle_cluster_information(_request: &HttpRequest) -> Result<HttpResponse, Error> {
    let cluster_information = json!({
        "cluster_name" : "noronha",
        "node_name" : "noronha-0",
        "noronha_version" : "0.1.0-SNAPSHOT"
    });

    Ok(HttpResponse::Ok()
       .content_type("application/json")
       .body(json_body(&cluster_information)))
}

fn handle_create_namespace(request: &HttpRequest) -> Result<HttpResponse, Error> {
    let mut namespaces = NAMESPACES.lock().unwrap();

    let namespace_name: String = request.match_info().query("namespace")?;

    match namespaces.entry(namespace_name.clone()) {
        Occupied(entry) => {
            let namespace = entry.get();
            let response_body = json!(namespace);

            Ok(HttpResponse::build(http::StatusCode::OK)
               .content_type("application/json")
               .body(json_body(&response_body)))
        },
        Vacant(entry) => {
            let namespace = entry.insert(make_namespace(&namespace_name));
            let response_body = json!(namespace);

            Ok(HttpResponse::build(http::StatusCode::CREATED)
               .content_type("application/json")
               .body(json_body(&response_body)))
        }
    }
}

fn handle_get_namespace(request: &HttpRequest) -> Result<HttpResponse, Error> {
    let namespaces = NAMESPACES.lock().unwrap();

    let namespace_name: String = request.match_info().query("namespace")?;

    match namespaces.get(&namespace_name) {
        Some(namespace) => {
            let response_body = json!(namespace);

            Ok(HttpResponse::build(http::StatusCode::OK)
               .content_type("application/json")
               .body(json_body(&response_body)))
        },
        None => {
            Ok(HttpResponse::build(http::StatusCode::NOT_FOUND)
               .content_type("application/json")
               .finish())
        }
    }
}

fn http_api_application() -> App {
    App::new()
        .resource("/", |r| {
            r.method(http::Method::GET).f(handle_cluster_information)
        })
        .resource("/{namespace}", |r| {
            r.method(http::Method::PUT).f(handle_create_namespace);
            r.method(http::Method::GET).f(handle_get_namespace);
        })
}

fn main() {
    let host = "0.0.0.0";
    let port = 6500;
    let http_workers = 10;
    let log_config_file = "config/log4rs.yml";

    log4rs::init_file(log_config_file, Default::default()).unwrap();

    let address: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    let http_api = server::new(http_api_application)
        .workers(http_workers)
        .bind(address)
        .unwrap();

    info!("Starting");
    http_api.run();
}
