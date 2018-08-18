use std::net::SocketAddr;
use std::thread::{self, JoinHandle};

use actix_web::{
    http::{Method, StatusCode},
    server, App, Error as HttpError, HttpRequest, HttpResponse,
};
use serde_json;

use components::configuration::Configuration;
use http_utils::{json_body, json_error, make_handler_for_request_with_body};
use node::Node;

use components;

fn handle_ping(
    _request: &HttpRequest,
    body: serde_json::Value,
) -> Result<HttpResponse, HttpError> {
    match components::cluster::STATE.write().unwrap().as_mut() {
        Some(cluster) => match serde_json::from_value::<Node>(body) {
            Ok(node) => {
                info!("Pinged by {}, sending pong", node.name);
                cluster.register_ping(node);

                Ok(HttpResponse::Ok()
                    .content_type("application/json")
                    .body(json_body(&json!(cluster.make_pong()))))
            }
            Err(error) => {
                error!("Error handling ping, not sending pong: {:?}", error);
                Ok(HttpResponse::build(StatusCode::BAD_REQUEST)
                    .content_type("application/json")
                    .body(json_error(error)))
            }
        },
        None => Ok(HttpResponse::build(StatusCode::SERVICE_UNAVAILABLE)
            .content_type("application/json")
            .finish()),
    }
}

pub fn application() -> App {
    App::new().resource("/ping", |r| {
        r.method(Method::POST).with(|request: HttpRequest| {
            make_handler_for_request_with_body(&handle_ping)(request)
        });
    })
}

pub fn start() -> () {
    let c = Configuration::read();

    let http_transport_address: SocketAddr =
        format!("{}:{}", &c.bind_host, &c.http_transport_port)
            .parse()
            .unwrap();

    let http_transport_server = server::new(application)
        .workers(c.http_transport_workers)
        .bind(http_transport_address)
        .unwrap();

    info!("Starting cluster server");
    http_transport_server.run();
}

pub fn spawn() -> JoinHandle<()> {
    thread::spawn(start)
}
