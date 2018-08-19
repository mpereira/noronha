use im::hashmap::HashMap;

use types::{Bag, Metadata};

pub type ObjectData = Bag<String>;

#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub metadata: Metadata,
    pub data: ObjectData,
}

impl Object {
    pub fn make(id: &str, mut data: ObjectData) -> Self {
        let mut metadata: Metadata = HashMap::new();
        metadata.insert("id".to_owned(), id.to_owned());

        data.insert("id".to_string(), id.to_string());

        Self {
            metadata: metadata,
            data: data,
        }
    }

    // pub fn id(&self) -> String {
    //     self.data.get("id")
    // }
}
