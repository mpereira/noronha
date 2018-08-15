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

use std::net::SocketAddr;
use std::thread;

use actix_web::server;
use im::hashmap::HashMap;
use im::hashset::HashSet;

use cluster::Cluster;
use node::{Node, UnknownNode};
use utils::make_id;

mod cluster;
mod configuration;
mod http_resources;
mod http_transport;
mod http_transport_pinger;
mod http_utils;
mod logging;
mod namespace;
mod node;
mod noronha_state;
mod object;
mod storage;
mod types;
mod utils;

use configuration::Configuration;
use logging::Logging;

use noronha_state::CLUSTER;

fn main() {
    let c = Configuration::start();

    Logging::start(&c.log_config_file);

    info!("{:#?}", c);


    let a = thread::spawn(move || {
        c.cluster_name = "foo".to_string();
        info!("{:#?}", c);
    }).join().unwrap();

    let b = thread::spawn(move || {
        c.cluster_name = "bar".to_string();
        info!("{:#?}", c);
    }).join().unwrap();

    let cc = thread::spawn(move || {
        info!("{:#?}", c);
    }).join().unwrap();

    {
        let node_address =
            format!("{}:{}", &c.publish_host, &c.http_transport_port);
        let mut cluster = CLUSTER.write().unwrap();

        let node = Node {
            id: make_id(),
            name: c.node_name.clone(),
            address: node_address,
        };

        let unknown_nodes: HashSet<UnknownNode> = c
            .cluster_peers
            .iter()
            .map(|ref mut address| UnknownNode {
                address: address.to_string(),
            })
            .collect();

        *cluster = Some(Cluster {
            leader: None,
            name: c.cluster_name.clone(),
            node: node,
            unknown_peers: unknown_nodes,
            peers: HashSet::new(),
            noronha_version: env!("CARGO_PKG_VERSION").to_string(),
            pings: HashMap::new(),
            state_version: 0,
        });
    }

    let http_transport_address: SocketAddr =
        format!("{}:{}", &c.bind_host, &c.http_transport_port)
        .parse()
        .unwrap();

    let http_resources_address: SocketAddr =
        format!("{}:{}", &c.bind_host, &c.http_resources_port)
        .parse()
        .unwrap();

    let http_resources_server_thread = thread::spawn(move || {
        let http_resources_server = server::new(http_resources::application)
            .workers(c.http_resources_workers)
            .bind(http_resources_address)
            .unwrap();

        info!("Starting resources server");
        http_resources_server.run();
    });

    let http_transport_server_thread = thread::spawn(move || {
        let http_transport_server = server::new(http_transport::application)
            .workers(c.http_transport_workers)
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
