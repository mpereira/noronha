extern crate actix_web;
extern crate bytes;
extern crate futures;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate log4rs;
#[macro_use]
extern crate serde_json;
extern crate uuid;

use actix_web::server;
use std::net::SocketAddr;

mod http_resources;
mod http_utils;
mod namespace;
mod object;
mod types;
mod utils;

fn main() {
    let host = "0.0.0.0";
    let http_resources_port = 6500;
    let http_resources_workers = 10;
    let log_config_file = "config/log4rs.yml";

    log4rs::init_file(log_config_file, Default::default()).unwrap();

    let address: SocketAddr =
        format!("{}:{}", host, http_resources_port).parse().unwrap();

    let http_resources_server = server::new(http_resources::application)
        .workers(http_resources_workers)
        .bind(address)
        .unwrap();

    info!("Starting");
    http_resources_server.run();
}
