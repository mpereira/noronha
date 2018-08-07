use im::hashmap::{
    Entry::{Occupied, Vacant},
    HashMap,
};

use object::Object;
use types::{Bag, Metadata, Operation};
use utils::make_id_string;

#[derive(Clone, Debug)]
pub struct Namespace {
    pub metadata: Metadata,
    pub objects: Bag<Object>,
}

pub fn make(name: &str) -> Namespace {
    let mut metadata: Metadata = HashMap::new();
    metadata.insert("id".to_string(), make_id_string());
    metadata.insert("name".to_string(), name.to_string());

    Namespace {
        metadata: metadata,
        objects: HashMap::new(),
    }
}

pub fn create_or_update_object<'a>(
    Namespace {
        ref mut objects, ..
    }: &'a mut Namespace,
    object: Object,
) -> Operation<Object> {
    let object_id = object.data.get("id").unwrap().to_string();

    match objects.entry(object_id) {
        Occupied(mut entry) => {
            entry.insert(object.clone());
            Operation::Updated(object)
        }
        Vacant(entry) => {
            entry.insert(object.clone());
            Operation::Created(object)
        }
    }
}
