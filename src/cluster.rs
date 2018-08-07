use std::time::SystemTime;

use im::hashmap::HashMap;
use im::hashset::HashSet;

use node::{Node, NodeId, UnknownNode};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Ping {
    pub from: Node,
    pub timestamp: SystemTime,
}

pub type Pong = Ping;

#[derive(Debug)]
pub struct Cluster {
    pub leader: Option<Node>,
    pub name: String,
    pub node: Node,
    pub unknown_peers: HashSet<UnknownNode>,
    pub peers: HashSet<Node>,
    pub noronha_version: String,
    pub pings: HashMap<NodeId, Ping>,
    pub state_version: i64,
}

impl Cluster {
    pub fn make_pong(&self) -> Pong {
        Pong {
            from: self.node.clone(),
            timestamp: SystemTime::now(),
        }
    }

    pub fn identify_peer(&mut self, unknown: &UnknownNode, peer: Node) -> Node {
        self.unknown_peers.remove(unknown);
        self.peers.insert(peer.clone());
        peer
    }

    pub fn register_ping(&mut self, pinger: Node) -> () {
        self.pings.insert(
            pinger.id,
            Ping {
                from: pinger,
                timestamp: SystemTime::now(),
            },
        );
    }

    pub fn nodes(&self) -> HashSet<Node> {
        self.peers.update(self.node.clone())
    }
}
