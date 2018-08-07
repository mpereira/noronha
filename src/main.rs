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
extern crate uuid;

use std::env;
use std::net::SocketAddr;
use std::thread;

use actix_web::server;
use im::hashmap::HashMap;
use im::hashset::HashSet;

use cluster::Cluster;
use node::{Node, UnknownNode};
use utils::make_id;

use state::CLUSTER;
use state::CONFIGURATION;

mod cluster;
mod configuration;
mod http_transport;
mod http_transport_pinger;
mod http_resources;
mod http_utils;
mod namespace;
mod node;
mod object;
mod state;
mod types;
mod utils;

fn main() {
    let default_configuration_file = "config/Noronha.toml".to_string();

    let configuration_file = match env::var("NORONHA_CONFIGURATION_FILE") {
        Ok(value) => {
            println!("Using configuration file from ENV: {}", value);
            value
        }
        Err(_error) => {
            println!(
                "Using default configuration file: {}",
                default_configuration_file
            );
            default_configuration_file
        }
    };

    configuration::start(&configuration_file);

    let c = CONFIGURATION.read().unwrap();

    let bind_host = c.get_str("bind_host").expect("host");
    let publish_host = c.get_str("publish_host").expect("host");
    let cluster_name = c.get_str("cluster_name").expect("cluster_name");
    let cluster_peers = c.get_array("cluster_peers").expect("cluster_peers");
    let node_name = c.get_str("node_name").expect("node_name");
    let http_resources_port = c
        .get_int("http_resources_port")
        .expect("http_resources_port");
    let http_resources_workers =
        c.get_int("http_resources_workers")
        .expect("http_resources_workers") as usize;
    let http_transport_port =
        c.get_int("http_transport_port").expect("http_transport_port");
    let http_transport_workers = c
        .get_int("http_transport_workers")
        .expect("http_transport_workers") as usize;
    let log_config_file =
        c.get_str("log_config_file").expect("log_config_file");

    drop(c);

    log4rs::init_file(log_config_file, Default::default()).unwrap();

    let http_transport_address: SocketAddr =
        format!("{}:{}", bind_host, http_transport_port).parse().unwrap();
    let http_resources_address: SocketAddr =
        format!("{}:{}", bind_host, http_resources_port).parse().unwrap();

    let node_address = format!("{}:{}", publish_host, http_transport_port);

    {
        let mut cluster = CLUSTER.write().unwrap();

        let node = Node {
            id: make_id(),
            name: node_name,
            address: node_address,
        };

        let unknown_nodes: HashSet<UnknownNode> = cluster_peers
            .iter()
            .map(|ref mut address| UnknownNode {
                address: address.clone().into_str().unwrap()
            })
            .collect();

        *cluster = Some(Cluster {
            leader: None,
            name: cluster_name,
            node: node,
            unknown_peers: unknown_nodes,
            peers: HashSet::new(),
            noronha_version: env!("CARGO_PKG_VERSION").to_string(),
            pings: HashMap::new(),
            state_version: 0,
        });
    }

    let http_resources_server_thread = thread::spawn(move || {
        let http_resources_server = server::new(http_resources::application)
            .workers(http_resources_workers)
            .bind(http_resources_address)
            .unwrap();

        info!("Starting resources server");
        http_resources_server.run();
    });

    let http_transport_server_thread = thread::spawn(move || {
        let http_transport_server = server::new(http_transport::application)
            .workers(http_transport_workers)
            .bind(http_transport_address)
            .unwrap();

        info!("Starting cluster server");
        http_transport_server.run();
    });

    let http_transport_pinger_thread = thread::spawn(move || {
        info!("Starting cluster pinger");
        http_transport_pinger::start();
    });

    http_transport_server_thread.join().unwrap();
    http_resources_server_thread.join().unwrap();
    http_transport_pinger_thread.join().unwrap();
}
