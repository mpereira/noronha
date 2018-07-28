extern crate hyper;

#[macro_use]
extern crate log;
extern crate log4rs;

#[macro_use]
extern crate serde_json;

use std::net::SocketAddr;

use hyper::rt::Future;
use hyper::service::service_fn_ok;
use hyper::{Body, Response, Server};

fn main() {
    log4rs::init_file("config/log4rs.yml", Default::default()).unwrap();

    let host = "0.0.0.0";
    let port = 6500;
    let address: SocketAddr = format!("{}:{}", host, port).parse().unwrap();

    let http_api = || {
        service_fn_ok(|_| {
            let data = json!({
                "cluster_name" : "noronha",
                "node_name" : "noronha-0",
                "noronha_version" : "0.1.0"
            });

            let response_body = serde_json::to_string_pretty(&data).unwrap();

            Response::new(Body::from(response_body))
        })
    };

    let server = Server::bind(&address)
        .serve(http_api)
        .map_err(|e| error!("{}", e));

    info!("listening at {}", address);
    info!("started");

    hyper::rt::run(server);
}
