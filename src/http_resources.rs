use std::str;

use actix_web::{
    http::{self, StatusCode},
    App, Error, HttpRequest, HttpResponse,
};
use im::hashmap::Entry::{Occupied, Vacant};
use serde_json;

use http_utils::{json_body, make_handler_for_request_with_body};
use namespace;
use object::{self, ObjectData};
use types::Operation;
use utils::make_id_string;

use cluster::Cluster;

use noronha_state::CLUSTER;
use noronha_state::NAMESPACES;

fn cluster_information(cluster: &Cluster) -> serde_json::Value {
    json!({
        "cluster_name" : cluster.name,
        "node_name" : cluster.node.name,
        "noronha_version" : cluster.noronha_version
    })
}

fn handle_cluster_information(
    _request: &HttpRequest,
) -> Result<HttpResponse, Error> {
    match CLUSTER.read().unwrap().as_ref() {
        Some(cluster) => Ok(HttpResponse::Ok()
                            .content_type("application/json")
                            .body(json_body(&cluster_information(cluster)))),
        None => Ok(HttpResponse::build(StatusCode::SERVICE_UNAVAILABLE)
                   .content_type("application/json")
                   .finish()),
    }
}

fn handle_cluster_state(_request: &HttpRequest) -> Result<HttpResponse, Error> {
    match CLUSTER.read().unwrap().as_ref() {
        Some(cluster) => Ok(HttpResponse::Ok()
                            .content_type("application/json")
                            .body(json_body(&serde_json::to_value(cluster).unwrap()))),
        None => Ok(HttpResponse::build(StatusCode::SERVICE_UNAVAILABLE)
                   .content_type("application/json")
                   .finish()),
    }
}

fn handle_create_namespace(
    request: &HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut namespaces = NAMESPACES.write().unwrap();

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
    let namespaces = NAMESPACES.read().unwrap();

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
    let mut namespaces = NAMESPACES.write().unwrap();

    let namespace_name: String = request.match_info().query("namespace")?;
    let object_id: String = request
        .match_info()
        .get("object_id")
        .map(str::to_string)
        .or(Some(make_id_string()))
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
    let namespaces = NAMESPACES.read().unwrap();

    let namespace_name: String = request.match_info().query("namespace")?;
    let object_id: String = request
        .match_info()
        .get("object_id")
        .map(str::to_string)
        .or(Some(make_id_string()))
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
        .scope("/_cluster/", |s| {
            s.resource("state", |r| {
                r.method(http::Method::GET).f(handle_cluster_state)
            })
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
