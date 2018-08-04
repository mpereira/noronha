use std::net::SocketAddr;

extern crate log;
extern crate log4rs;

#[macro_use]
extern crate serde_json;

extern crate actix_web;
use actix_web::{http, server, App, Error, HttpRequest, HttpResponse};

fn json_body(content: &serde_json::Value) -> String {
    format!("{}\n", serde_json::to_string_pretty(content).unwrap())
}

fn cluster_information(_request: &HttpRequest) -> Result<HttpResponse, Error> {
    let cluster_information = json!({
        "cluster_name" : "noronha",
        "node_name" : "noronha-0",
        "noronha_version" : "0.1.0"
    });

    Ok(HttpResponse::Ok()
       .content_type("application/json")
       .body(json_body(&cluster_information)))
}

fn http_api_application() -> App {
    App::new().resource("/", |r| r.method(http::Method::GET).f(cluster_information))
}

fn main() {
    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();

    let host = "0.0.0.0";
    let port = 6500;
    let address: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    let http_api = server::new(http_api_application).bind(address).unwrap();

    http_api.run();
}
