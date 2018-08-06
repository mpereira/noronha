use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::str;
use std::sync::Mutex;

extern crate actix_web;
use actix_web::{
    http, server, App, AsyncResponder, Error, HttpMessage, HttpRequest,
    HttpResponse,
};

extern crate bytes;
use bytes::Bytes;

extern crate futures;
use futures::future::Future;

#[macro_use]
extern crate lazy_static;

#[macro_use]
extern crate log;
extern crate log4rs;

#[macro_use]
extern crate serde_json;

extern crate uuid;
use uuid::Uuid;

type Bag<T> = HashMap<String, T>;

type Metadata = Bag<String>;

type ObjectData = Bag<String>;

#[derive(Clone, Debug)]
struct Object {
    metadata: Metadata,
    data: ObjectData,
}

#[derive(Debug)]
struct Namespace {
    metadata: Metadata,
    objects: Bag<Object>,
}

type Namespaces = Bag<Namespace>;

enum Operation<Object> {
    Created(Object),
    Updated(Object),
}

fn make_id() -> String {
    Uuid::new_v4().to_string()
}

fn make_object(mut data: ObjectData, id: &str) -> Object {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("id".to_string(), id.to_string());

    data.insert("id".to_string(), id.to_string());

    Object {
        metadata: metadata,
        data: data,
    }
}

fn make_namespace(name: &str) -> Namespace {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("id".to_string(), make_id());
    metadata.insert("name".to_string(), name.to_string());

    Namespace {
        metadata: metadata,
        objects: HashMap::new(),
    }
}

fn json_body(content: &serde_json::Value) -> String {
    format!("{}\n", serde_json::to_string_pretty(content).unwrap())
}

fn handle_cluster_information(
    _request: &HttpRequest,
) -> Result<HttpResponse, Error> {
    let cluster_information = json!({
        "cluster_name" : "noronha",
        "node_name" : "noronha-0",
        "noronha_version" : "0.1.0-SNAPSHOT"
    });

    Ok(HttpResponse::Ok()
       .content_type("application/json")
       .body(json_body(&cluster_information)))
}

fn handle_create_namespace(
    request: &HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut namespaces = NAMESPACES.lock().unwrap();

    let namespace_name: String = request.match_info().query("namespace")?;

    match namespaces.entry(namespace_name.clone()) {
        Occupied(entry) => {
            let namespace = entry.get();
            let response_body = json!(namespace.metadata);

            Ok(HttpResponse::build(http::StatusCode::OK)
               .content_type("application/json")
               .body(json_body(&response_body)))
        }
        Vacant(entry) => {
            let namespace = make_namespace(&namespace_name);
            let response_body = json!(namespace.metadata);
            entry.insert(namespace);

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
            let response_body = json!(namespace.metadata);

            Ok(HttpResponse::build(http::StatusCode::OK)
               .content_type("application/json")
               .body(json_body(&response_body)))
        }
        None => Ok(HttpResponse::build(http::StatusCode::NOT_FOUND)
                   .content_type("application/json")
                   .finish()),
    }
}

fn create_or_update_namespace_object<'a>(
    Namespace {
        ref mut objects, ..
    }: &'a mut Namespace,
    object: Object,
) -> Operation<Object> {
    let object_id = object.data.get("id").unwrap().to_string();

    match objects.entry(object_id) {
        Occupied(mut entry) => {
            entry.insert(object.clone());
            Operation::Updated(object)
        },
        Vacant(entry) => {
            entry.insert(object.clone());
            Operation::Created(object)
        }
    }
}

fn handle_create_or_update_namespace_object(
    request: &HttpRequest,
    body: serde_json::Value,
) -> Result<HttpResponse, Error> {
    let mut namespaces = NAMESPACES.lock().unwrap();

    let namespace_name: String = request.match_info().query("namespace")?;
    let object_id: String = request
        .match_info()
        .get("object_id")
        .map(str::to_string)
        .or(Some(make_id()))
        .unwrap();

    match namespaces.get_mut(&namespace_name) {
        Some(namespace) => {
            let object_data: ObjectData = serde_json::from_value(body).unwrap();
            let operation = create_or_update_namespace_object(
                namespace,
                make_object(object_data, &object_id),
            );

            match operation {
                Operation::Created(object) => {
                    let response_body = json!(object.data);

                    Ok(HttpResponse::build(http::StatusCode::CREATED)
                       .content_type("application/json")
                       .body(json_body(&response_body)))
                }
                Operation::Updated(object) => {
                    let response_body = json!(object.data);

                    Ok(HttpResponse::build(http::StatusCode::OK)
                       .content_type("application/json")
                       .body(json_body(&response_body)))
                }
            }
        }
        None => Ok(HttpResponse::build(http::StatusCode::NOT_FOUND)
                   .content_type("application/json")
                   .finish()),
    }
}

fn handle_get_namespace_object(
    request: &HttpRequest,
) -> Result<HttpResponse, Error> {
    let namespaces = NAMESPACES.lock().unwrap();

    let namespace_name: String = request.match_info().query("namespace")?;
    let object_id: String = request
        .match_info()
        .get("object_id")
        .map(str::to_string)
        .or(Some(make_id()))
        .unwrap();

    match namespaces.get(&namespace_name) {
        Some(namespace) => match namespace.objects.get(&object_id) {
            Some(object) => {
                let response_body = json!(object.data);

                Ok(HttpResponse::build(http::StatusCode::OK)
                   .content_type("application/json")
                   .body(json_body(&response_body)))
            }
            None => Ok(HttpResponse::build(http::StatusCode::NOT_FOUND)
                       .content_type("application/json")
                       .finish()),
        },
        None => Ok(HttpResponse::build(http::StatusCode::NOT_FOUND)
                   .content_type("application/json")
                   .finish()),
    }
}

fn make_handler_for_request_with_body(
    handler: &'static for<'r> Fn(&HttpRequest, serde_json::Value)
                                 -> Result<HttpResponse, Error>,
) -> Box<Fn(HttpRequest) -> Box<Future<Item = HttpResponse, Error = Error>>> {
    Box::new(move |request: HttpRequest| {
        request
            .body()
            .from_err()
            .and_then(move |bytes: Bytes| -> Result<HttpResponse, Error> {
                let body: serde_json::Value = serde_json::from_str(
                    str::from_utf8(&bytes).unwrap(),
                ).unwrap();
                handler(&request, body)
            })
            .responder()
    })
}

fn http_api_application() -> App {
    App::new()
        .resource("/", |r| {
            r.method(http::Method::GET).f(handle_cluster_information)
        })
        .resource("/{namespace}", |r| {
            r.method(http::Method::PUT).f(handle_create_namespace);
            r.method(http::Method::GET).f(handle_get_namespace);
            r.method(http::Method::POST).with(|request: HttpRequest| {
                make_handler_for_request_with_body(
                    &handle_create_or_update_namespace_object,
                )(request)
            });
        })
        .resource("/{namespace}/{object_id}", |r| {
            r.method(http::Method::PUT).with(|request: HttpRequest| {
                make_handler_for_request_with_body(
                    &handle_create_or_update_namespace_object,
                )(request)
            });
            r.method(http::Method::GET).f(handle_get_namespace_object);
        })
}

lazy_static! {
    static ref NAMESPACES: Mutex<Namespaces> = Mutex::new(HashMap::new());
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
