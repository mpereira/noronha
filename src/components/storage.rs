use std::sync::RwLock;

use im::hashmap::HashMap;

use namespace::Namespace;
use types::Bag;

type Namespaces = Bag<Namespace>;

lazy_static! {
    pub static ref STATE: RwLock<Namespaces> = RwLock::new(HashMap::new());
}
