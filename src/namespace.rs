use im::hashmap::{
    Entry::{Occupied, Vacant},
    HashMap,
};

use object::Object;
use types::{Bag, Metadata, Outcome::{self, *}};
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

    pub fn create_or_update_object(
        &mut self,
        object: Object,
    ) -> Outcome<Object> {
        let object_id = object.data.get("id").unwrap().to_string();

        match self.objects.entry(object_id) {
            Occupied(mut entry) => {
                entry.insert(object.to_owned());
                Updated(object)
            }
            Vacant(entry) => {
                entry.insert(object.to_owned());
                Created(object)
            }
        }
    }
}
