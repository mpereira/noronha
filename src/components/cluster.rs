use std::sync::RwLock;

use im::hashmap::HashMap;
use im::hashset::HashSet;

use cluster::Cluster;
use components::configuration::Configuration;
use node::{Node, UnknownNode};
use utils::make_id;

lazy_static! {
    pub static ref STATE: RwLock<Option<Cluster>> = RwLock::new(None);
}

pub fn initialize() -> () {
    let c = Configuration::read();

    let node_address = format!("{}:{}", c.publish_host, c.http_transport_port);
    let mut cluster = STATE.write().unwrap();

    let node = Node {
        id: make_id(),
        name: c.node_name.to_owned(),
        address: node_address,
    };

    let unknown_nodes: HashSet<UnknownNode> = c
        .cluster_peers
        .iter()
        .map(|ref mut address| UnknownNode {
            address: address.to_owned(),
        })
        .collect();

    *cluster = Some(Cluster {
        leader: None,
        name: c.cluster_name.to_owned(),
        node: node,
        unknown_peers: unknown_nodes,
        peers: HashSet::new(),
        noronha_version: env!("CARGO_PKG_VERSION").to_owned(),
        pings: HashMap::new(),
        state_version: 0,
    });
}
