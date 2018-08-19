use std::net::SocketAddr;
use std::str;
use std::thread::{self, JoinHandle};

use actix_web::{
    http::{self, StatusCode},
    server, App, Error, HttpRequest, HttpResponse,
};
use im::hashmap::Entry::{Occupied, Vacant};
use serde_json;

use components::configuration::Configuration;
use http_utils::{json_body, make_handler_for_request_with_body};
use namespace::Namespace;
use object::{Object, ObjectData};
use types::Outcome::*;
use utils::make_id_string;
use storage;

use cluster::Cluster;

use components;

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
    match components::cluster::STATE.read().unwrap().as_ref() {
        Some(cluster) => Ok(HttpResponse::Ok()
                            .content_type("application/json")
                            .body(json_body(&cluster_information(cluster)))),
        None => Ok(HttpResponse::build(StatusCode::SERVICE_UNAVAILABLE)
                   .content_type("application/json")
                   .finish()),
    }
}

fn handle_cluster_state(_request: &HttpRequest) -> Result<HttpResponse, Error> {
    match components::cluster::STATE.read().unwrap().as_ref() {
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
    let mut namespaces = components::storage::STATE.write().unwrap();

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
            let namespace = Namespace::make(&namespace_name);
            let response_body = json!(namespace.metadata);
            entry.insert(namespace);

            Ok(HttpResponse::build(http::StatusCode::CREATED)
               .content_type("application/json")
               .body(json_body(&response_body)))
        }
    }
    // let namespace = Namespace::make(&namespace_name);
    // match storage::create_namespace(namespace) {
    //     Ok(outcome) => match outcome {
    //         Created(namespace) => (),
    //         AlreadyExisted(error) => (),
    //     },
    //     Err(error) => (),
    // }
}

fn handle_get_namespace(request: &HttpRequest) -> Result<HttpResponse, Error> {
    let namespaces = components::storage::STATE.read().unwrap();

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

    // match storage::get_namespace(namespace_name) {
    //     Some(namespace) => {
    //         let response_body = json!(namespace.metadata);

    //         Ok(HttpResponse::build(http::StatusCode::OK)
    //            .content_type("application/json")
    //            .body(json_body(&response_body)))
    //     }
    //     None => Ok(HttpResponse::build(http::StatusCode::NOT_FOUND)
    //                .content_type("application/json")
    //                .finish()),
    // }
}

fn handle_create_or_update_namespace_object(
    request: &HttpRequest,
    body: serde_json::Value,
) -> Result<HttpResponse, Error> {
    let mut namespaces = components::storage::STATE.write().unwrap();

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
            let outcome = namespace
                .create_or_update_object(Object::make(&object_id, object_data));

            match outcome {
                Created(object) => {
                    let response_body = json!(object.data);

                    Ok(HttpResponse::build(http::StatusCode::CREATED)
                       .content_type("application/json")
                       .body(json_body(&response_body)))
                }
                Updated(object) => {
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

    // let object_data: ObjectData = serde_json::from_value(body).unwrap();
    // let object = Object::make(&object_id, object_data);
    // match storage::create_namespace_object(namespace_name, object) {
    //     Ok(outcome) => {
    //         match outcome {
    //             NamespaceObjectCreated { namespace, object } => {
    //                 let response_body = json!(object.data);

    //                 Ok(HttpResponse::build(http::StatusCode::CREATED)
    //                    .content_type("application/json")
    //                    .body(json_body(&response_body)))
    //             },
    //             NamespaceObjectUpdated { namespace, object } => {
    //                 let response_body = json!(object.data);

    //                 Ok(HttpResponse::build(http::StatusCode::OK)
    //                    .content_type("application/json")
    //                    .body(json_body(&response_body)))
    //             },
    //             NamespaceNotFound => { namespace_name } => {
    //                 Ok(HttpResponse::build(http::StatusCode::NOT_FOUND)
    //                    .content_type("application/json")
    //                    .finish())
    //             }
    //         }
    //     }
    //     Err(error) => {
    //         Ok(HttpResponse::build(StatusCode::SERVICE_UNAVAILABLE)
    //            .content_type("application/json")
    //            .finish())
    //     }
    // }
}

fn handle_get_namespace_object(
    request: &HttpRequest,
) -> Result<HttpResponse, Error> {
    let namespaces = components::storage::STATE.read().unwrap();

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

pub fn start() -> () {
    let c = Configuration::read();

    let http_resources_address: SocketAddr =
        format!("{}:{}", c.bind_host, c.http_resources_port)
        .parse()
        .unwrap();

    let http_resources_server = server::new(application)
        .workers(c.http_resources_workers)
        .bind(http_resources_address)
        .unwrap();

    info!("Starting resources server");
    http_resources_server.run()
}

pub fn spawn() -> JoinHandle<()> {
    thread::spawn(start)
}
