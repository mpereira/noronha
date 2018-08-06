use std::collections::hash_map::Entry::{Occupied, Vacant};
use std::collections::HashMap;
use std::str;
use std::sync::Mutex;

use actix_web::{http, App, Error, HttpRequest, HttpResponse};
use serde_json;

use http_utils::{json_body, make_handler_for_request_with_body};
use namespace::{self, Namespace};
use object::{self, ObjectData};
use types::{Bag, Operation};
use utils::make_id;

type Namespaces = Bag<Namespace>;

lazy_static! {
    static ref NAMESPACES: Mutex<Namespaces> = Mutex::new(HashMap::new());
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
            let namespace = namespace::make(&namespace_name);
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
            let operation = namespace::create_or_update_object(
                namespace,
                object::make_object(object_data, &object_id),
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

pub fn application() -> App {
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
