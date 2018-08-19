use im::hashmap::Entry::{Occupied, Vacant};

use namespace::Namespace;
use object::Object;
use types::Bag;

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    CreateOrUpdateNamespace {
        namespace: Namespace,
    },
    CreateOrUpdateNamespaceObject {
        namespace_name: String,
        object: Object,
    },
    ReadNamespace {
        namespace_name: String,
    },
    ReadNamespaceObject {
        namespace_name: String,
        object_id: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Storage {
    log: Vec<Operation>,
    namespaces: Bag<Namespace>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    NamespaceCreated(Namespace),
    NamespaceUpdated(Namespace),
    NamespaceFound(Namespace),
    NamespaceNotFound(String),
    NamespaceObjectCreated(Object),
    NamespaceObjectUpdated(Object),
    NamespaceObjectFound(Object),
    NamespaceObjectNotFound(String),
}

#[derive(Debug)]
pub struct Error;

use self::Operation::*;
use self::Outcome::*;

impl Namespace {
    pub fn create_or_update_object(
        &mut self,
        object: Object,
    ) -> Result<Outcome, Error> {
        let object = object.clone();
        info!("create_or_update_object {:#?}", object);

        match object.data.get("id") {
            Some(object_id) => match self.objects.entry(object_id.to_owned()) {
                Occupied(mut entry) => {
                    entry.insert(object.to_owned());
                    let object = object.clone();
                    Ok(NamespaceObjectUpdated(object))
                }
                Vacant(entry) => {
                    entry.insert(object.to_owned());
                    let object = object.clone();
                    Ok(NamespaceObjectCreated(object))
                }
            },
            None => Err(Error),
        }
    }
}

impl Storage {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn create_or_update_namespace(
        &mut self,
        namespace: Namespace,
    ) -> Result<Outcome, Error> {
        self.apply(CreateOrUpdateNamespace { namespace })
    }

    pub fn read_namespace(
        &mut self,
        namespace_name: String,
    ) -> Result<Outcome, Error> {
        self.apply(ReadNamespace { namespace_name })
    }

    pub fn create_or_update_namespace_object(
        &mut self,
        namespace_name: String,
        object: Object,
    ) -> Result<Outcome, Error> {
        self.apply(CreateOrUpdateNamespaceObject {
            namespace_name,
            object,
        })
    }

    pub fn read_namespace_object(
        &mut self,
        namespace_name: String,
        object_id: String,
    ) -> Result<Outcome, Error> {
        self.apply(ReadNamespaceObject {
            namespace_name,
            object_id,
        })
    }

    fn apply(&mut self, operation: Operation) -> Result<Outcome, Error> {
        self.log.push(operation.clone());

        match operation {
            CreateOrUpdateNamespace { namespace } => {
                self._create_or_update_namespace(namespace)
            }
            CreateOrUpdateNamespaceObject {
                namespace_name,
                object,
            } => {
                self._create_or_update_namespace_object(namespace_name, object)
            }
            ReadNamespace { namespace_name } => {
                self._read_namespace(namespace_name)
            }
            ReadNamespaceObject {
                namespace_name,
                object_id,
            } => self._read_namespace_object(namespace_name, object_id),
        }
    }

    fn _create_or_update_namespace(
        &mut self,
        namespace: Namespace,
    ) -> Result<Outcome, Error> {
        let namespace = namespace.clone();
        let namespace_name = namespace.metadata.get("name").unwrap().to_owned();

        match self.namespaces.entry(namespace_name) {
            Occupied(mut entry) => {
                entry.insert(namespace.to_owned());
                Ok(NamespaceUpdated(namespace))
            }
            Vacant(entry) => {
                entry.insert(namespace.clone());
                Ok(NamespaceCreated(namespace))
            }
        }
    }

    fn _create_or_update_namespace_object(
        &mut self,
        namespace_name: String,
        object: Object,
    ) -> Result<Outcome, Error> {
        match self.namespaces.get_mut(&namespace_name) {
            Some(namespace) => namespace.create_or_update_object(object),
            None => Ok(NamespaceNotFound(namespace_name)),
        }
    }

    fn _read_namespace(
        &mut self,
        namespace_name: String,
    ) -> Result<Outcome, Error> {
        match self.namespaces.get(&namespace_name) {
            Some(namespace) => {
                let namespace = namespace.to_owned();
                Ok(NamespaceFound(namespace))
            }
            None => Ok(NamespaceNotFound(namespace_name)),
        }
    }

    fn _read_namespace_object(
        &mut self,
        namespace_name: String,
        object_id: String,
    ) -> Result<Outcome, Error> {
        match self.namespaces.get(&namespace_name) {
            Some(namespace) => match namespace.objects.get(&object_id) {
                Some(object) => {
                    let object = object.to_owned();
                    Ok(NamespaceObjectFound(object))
                }
                None => Ok(NamespaceObjectNotFound(object_id)),
            },
            None => Ok(NamespaceNotFound(namespace_name)),
        }
    }
}

#[cfg(test)]
mod tests {
    use im::hashmap::HashMap;

    use storage::*;

    #[test]
    fn test_new() {
        let storage = Storage::new();
        assert_eq!(storage.log, Vec::new());
        assert_eq!(storage.namespaces, HashMap::new());
    }

    #[test]
    fn test_operations() {
        // Namespace create:
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. inserts namespace into namespaces

        let mut storage = Storage::new();
        let namespace_name = "people";
        let namespace = Namespace::make(namespace_name);
        let create_or_update_namespace = CreateOrUpdateNamespace {
            namespace: namespace.to_owned(),
        };

        let expected_outcome = NamespaceCreated(namespace.clone());
        let expected_log = vec![create_or_update_namespace.clone()];
        let mut expected_namespaces: Bag<Namespace> = HashMap::new();
        expected_namespaces
            .insert(namespace_name.to_owned(), namespace.to_owned());

        let outcome =
            storage.apply(create_or_update_namespace.clone()).unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.namespaces, expected_namespaces);

        // Namespace object create:
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. inserts object into namespace

        let mut namespace = namespace.clone();
        let object_id = "1";
        let object = Object::make(object_id, HashMap::new());

        namespace
            .objects
            .insert(object_id.to_owned(), object.clone());

        let create_or_update_namespace_object = CreateOrUpdateNamespaceObject {
            namespace_name: namespace_name.to_owned(),
            object: object.clone(),
        };

        let expected_outcome = NamespaceObjectCreated(object.clone());
        let expected_log = vec![
            create_or_update_namespace.clone(),
            create_or_update_namespace_object.clone(),
        ];
        let mut expected_namespaces: Bag<Namespace> = HashMap::new();
        expected_namespaces
            .insert(namespace_name.to_owned(), namespace.to_owned());

        let outcome = storage
            .apply(create_or_update_namespace_object.clone())
            .unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.namespaces, expected_namespaces);

        // First namespace read (through Storage::apply):
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. doesn't change namespaces

        let read_namespace = ReadNamespace {
            namespace_name: namespace_name.to_owned(),
        };

        let expected_outcome = NamespaceFound(namespace.clone());
        let expected_log = vec![
            create_or_update_namespace.clone(),
            create_or_update_namespace_object.clone(),
            read_namespace.clone(),
        ];

        let outcome = storage.apply(read_namespace.clone()).unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.namespaces, expected_namespaces);

        // Second namespace read (through Storage::read_namespace):
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. doesn't change namespaces

        let expected_log = vec![
            create_or_update_namespace.clone(),
            create_or_update_namespace_object.clone(),
            read_namespace.clone(),
            read_namespace.clone(),
        ];

        let outcome =
            storage.read_namespace(namespace_name.to_owned()).unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.namespaces, expected_namespaces);

        // Namespace object read:
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. doesn't change namespaces

        let read_namespace_object = ReadNamespaceObject {
            namespace_name: namespace_name.to_owned(),
            object_id: object_id.to_owned(),
        };

        let expected_outcome = NamespaceObjectFound(object.clone());
        let expected_log = vec![
            create_or_update_namespace.clone(),
            create_or_update_namespace_object.clone(),
            read_namespace.clone(),
            read_namespace.clone(),
            read_namespace_object.clone(),
        ];

        let outcome = storage.apply(read_namespace_object.clone()).unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.namespaces, expected_namespaces);
    }
}
