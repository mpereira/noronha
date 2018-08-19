use std::net::SocketAddr;
use std::str;
use std::thread::{self, JoinHandle};

use actix_web::{
    http::{self, StatusCode},
    server, App, Error, HttpRequest, HttpResponse,
};
use serde_json;

use components::configuration::Configuration;
use http_utils::{json_body, make_handler_for_request_with_body};
use keyspace::Keyspace;
use object::{Object, ObjectData};
use storage::Outcome::*;
use utils::make_id_string;

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

fn handle_create_or_update_keyspace(
    request: &HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut storage = components::storage::STATE.write().unwrap();

    let keyspace_name: String = request.match_info().query("keyspace")?;
    let keyspace = Keyspace::make(&keyspace_name);

    match storage.create_or_update_keyspace(keyspace) {
        Ok(outcome) => match outcome {
            KeyspaceCreated(keyspace) => {
                let response_body = json!(keyspace.metadata);

                Ok(HttpResponse::build(StatusCode::CREATED)
                    .content_type("application/json")
                    .body(json_body(&response_body)))
            }
            KeyspaceUpdated(keyspace) => {
                let response_body = json!(keyspace.metadata);

                Ok(HttpResponse::build(StatusCode::OK)
                    .content_type("application/json")
                    .body(json_body(&response_body)))
            }
            _ => Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("application/json")
                .finish()),
        },
        Err(_error) => Ok(HttpResponse::build(
            StatusCode::INTERNAL_SERVER_ERROR,
        ).content_type("application/json")
            .finish()),
    }
}

fn handle_get_keyspace(request: &HttpRequest) -> Result<HttpResponse, Error> {
    let mut storage = components::storage::STATE.write().unwrap();

    let keyspace_name: String = request.match_info().query("keyspace")?;

    match storage.read_keyspace(keyspace_name) {
        Ok(outcome) => match outcome {
            KeyspaceFound(keyspace) => {
                let response_body = json!(keyspace.metadata);

                Ok(HttpResponse::build(StatusCode::OK)
                    .content_type("application/json")
                    .body(json_body(&response_body)))
            }
            KeyspaceNotFound(_keyspace_name) => {
                Ok(HttpResponse::build(StatusCode::NOT_FOUND)
                    .content_type("application/json")
                    .finish())
            }
            _ => Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("application/json")
                .finish()),
        },
        Err(_error) => Ok(HttpResponse::build(
            StatusCode::INTERNAL_SERVER_ERROR,
        ).content_type("application/json")
            .finish()),
    }
}

fn handle_create_or_update_keyspace_object(
    request: &HttpRequest,
    body: serde_json::Value,
) -> Result<HttpResponse, Error> {
    let mut storage = components::storage::STATE.write().unwrap();

    let keyspace_name: String = request.match_info().query("keyspace")?;
    let object_id: String = request
        .match_info()
        .get("object_id")
        .map(str::to_string)
        .or(Some(make_id_string()))
        .unwrap();
    let object_data: ObjectData = serde_json::from_value(body).unwrap();
    let object = Object::make(&object_id, object_data);

    match storage.create_or_update_keyspace_object(keyspace_name, object) {
        Ok(outcome) => match outcome {
            KeyspaceObjectCreated(object) => {
                let response_body = json!(object.data);

                Ok(HttpResponse::build(StatusCode::CREATED)
                    .content_type("application/json")
                    .body(json_body(&response_body)))
            }
            KeyspaceObjectUpdated(object) => {
                let response_body = json!(object.data);

                Ok(HttpResponse::build(StatusCode::OK)
                    .content_type("application/json")
                    .body(json_body(&response_body)))
            }
            KeyspaceNotFound(_keyspace_name) => {
                Ok(HttpResponse::build(StatusCode::NOT_FOUND)
                    .content_type("application/json")
                    .finish())
            }
            _ => Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("application/json")
                .finish()),
        },
        Err(_error) => Ok(HttpResponse::build(
            StatusCode::INTERNAL_SERVER_ERROR,
        ).content_type("application/json")
            .finish()),
    }
}

fn handle_get_keyspace_object(
    request: &HttpRequest,
) -> Result<HttpResponse, Error> {
    let mut storage = components::storage::STATE.write().unwrap();

    let keyspace_name: String = request.match_info().query("keyspace")?;
    let object_id: String = request
        .match_info()
        .get("object_id")
        .map(str::to_string)
        .or(Some(make_id_string()))
        .unwrap();

    match storage.read_keyspace_object(keyspace_name, object_id) {
        Ok(outcome) => match outcome {
            KeyspaceObjectFound(object) => {
                let response_body = json!(object.data);

                Ok(HttpResponse::build(StatusCode::OK)
                    .content_type("application/json")
                    .body(json_body(&response_body)))
            }
            KeyspaceNotFound(_keyspace_name) => {
                Ok(HttpResponse::build(StatusCode::NOT_FOUND)
                    .content_type("application/json")
                    .finish())
            }
            KeyspaceObjectNotFound(_object_id) => {
                Ok(HttpResponse::build(StatusCode::NOT_FOUND)
                    .content_type("application/json")
                    .finish())
            }
            _ => Ok(HttpResponse::build(StatusCode::INTERNAL_SERVER_ERROR)
                .content_type("application/json")
                .finish()),
        },
        Err(_error) => Ok(HttpResponse::build(
            StatusCode::INTERNAL_SERVER_ERROR,
        ).content_type("application/json")
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
        .resource("/{keyspace}", |r| {
            r.method(http::Method::PUT)
                .f(handle_create_or_update_keyspace);
            r.method(http::Method::GET).f(handle_get_keyspace);
            r.method(http::Method::POST).with(|request: HttpRequest| {
                make_handler_for_request_with_body(
                    &handle_create_or_update_keyspace_object,
                )(request)
            });
        })
        .resource("/{keyspace}/{object_id}", |r| {
            r.method(http::Method::PUT).with(|request: HttpRequest| {
                make_handler_for_request_with_body(
                    &handle_create_or_update_keyspace_object,
                )(request)
            });
            r.method(http::Method::GET).f(handle_get_keyspace_object);
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
