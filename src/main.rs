extern crate actix_web;
extern crate bytes;
extern crate config;
#[macro_use]
extern crate crossbeam_channel;
extern crate either;
extern crate futures;
extern crate im;
#[macro_use]
extern crate lazy_static;
extern crate libc;
#[macro_use]
extern crate log;
extern crate log4rs;
extern crate reqwest;
extern crate serde;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate serde_json;
extern crate signal_hook;
extern crate state;
extern crate uuid;

mod cluster;
mod components;
mod http_utils;
mod namespace;
mod node;
mod object;
mod types;
mod utils;

use components::configuration::Configuration;
use components::logging::Logging;

fn main() {
    let c = Configuration::initialize();

    Logging::initialize(&c.log_config_file);

    info!("{:#?}", c);

    components::cluster::initialize();

    let http_resources_thread = components::http_resources::spawn();
    let http_transport_thread = components::http_transport::spawn();
    let http_transport_pinger_thread =
        components::http_transport_pinger::spawn();
    http_transport_thread.join().unwrap();
    http_resources_thread.join().unwrap();
    http_transport_pinger_thread.join().unwrap();
}
