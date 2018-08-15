use std::sync::RwLock;

use im::hashmap::HashMap;

use cluster::Cluster;
use namespace::Namespace;
use types::Bag;

type Namespaces = Bag<Namespace>;

lazy_static! {
    pub static ref CLUSTER: RwLock<Option<Cluster>> = RwLock::new(None);
}

lazy_static! {
    pub static ref NAMESPACES: RwLock<Namespaces> = RwLock::new(HashMap::new());
}
