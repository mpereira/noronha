use im::hashmap::HashMap;

use types::{Bag, Metadata};

pub type ObjectData = Bag<String>;

#[derive(Clone, Debug)]
pub struct Object {
    pub metadata: Metadata,
    pub data: ObjectData,
}

pub fn make_object(mut data: ObjectData, id: &str) -> Object {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("id".to_string(), id.to_string());

    data.insert("id".to_string(), id.to_string());

    Object {
        metadata: metadata,
        data: data,
    }
}
