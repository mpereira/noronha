use im::hashmap::HashMap;

use object::Object;
use types::{Bag, Metadata};
use utils::make_id_string;

#[derive(Clone, Debug, PartialEq)]
pub struct Namespace {
    pub metadata: Metadata,
    pub objects: Bag<Object>,
}

impl Namespace {
    pub fn make(name: &str) -> Self {
        let mut metadata: Metadata = HashMap::new();
        metadata.insert("id".to_string(), make_id_string());
        metadata.insert("name".to_string(), name.to_string());

        Self {
            metadata: metadata,
            objects: HashMap::new(),
        }
    }
}
