use im::hashmap::Entry::{Occupied, Vacant};

use namespace::Namespace;
use object::Object;
use types::{
    Bag,
    Outcome::{Created, Updated},
};

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    CreateNamespace {
        namespace: Namespace,
    },
    CreateNamespaceObject {
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
    NamespaceCreated {
        namespace: Namespace,
    },
    NamespaceUpdated {
        namespace: Namespace,
    },
    NamespaceObjectCreated {
        namespace: Namespace,
        object: Object,
    },
    NamespaceObjectUpdated {
        namespace: Namespace,
        object: Object,
    },
    NamespaceFound {
        namespace: Namespace,
    },
    NamespaceNotFound {
        namespace_name: String,
    },
    NamespaceObjectFound {
        namespace: Namespace,
        object: Object,
    },
    NamespaceObjectNotFound {
        namespace_name: String,
        object_id: String,
    },
}

#[derive(Debug)]
struct Error;

use self::Operation::*;
use self::Outcome::*;

impl Storage {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn create_namespace(
        &mut self,
        namespace: Namespace,
    ) -> Result<Outcome, Error> {
        self.apply(CreateNamespace { namespace })
    }

    pub fn get_namespace(
        &mut self,
        namespace_name: String,
    ) -> Result<Outcome, Error> {
        self.apply(ReadNamespace { namespace_name })
    }

    pub fn create_namespace_object(
        &mut self,
        namespace_name: String,
        object: Object,
    ) -> Result<Outcome, Error> {
        self.apply(CreateNamespaceObject {
            namespace_name,
            object,
        })
    }

    fn apply(&mut self, operation: Operation) -> Result<Outcome, Error> {
        self.log.push(operation.clone());

        match operation {
            CreateNamespace { namespace } => self._create_namespace(namespace),
            CreateNamespaceObject {
                namespace_name,
                object,
            } => self._create_namespace_object(namespace_name, object),
            ReadNamespace { namespace_name } => {
                self._read_namespace(namespace_name)
            }
            ReadNamespaceObject {
                namespace_name,
                object_id,
            } => self._read_namespace_object(namespace_name, object_id),
        }
    }

    fn _create_namespace(
        &mut self,
        namespace: Namespace,
    ) -> Result<Outcome, Error> {
        let namespace = namespace.clone();
        let namespace_name = namespace.metadata.get("name").unwrap().to_owned();

        match self.namespaces.entry(namespace_name) {
            Occupied(mut entry) => {
                entry.insert(namespace.to_owned());
                Ok(NamespaceUpdated { namespace })
            }
            Vacant(entry) => {
                entry.insert(namespace.clone());
                Ok(NamespaceCreated { namespace })
            }
        }
    }

    fn _create_namespace_object(
        &mut self,
        namespace_name: String,
        object: Object,
    ) -> Result<Outcome, Error> {
        match self.namespaces.get_mut(&namespace_name) {
            Some(namespace) => {
                match namespace.create_or_update_object(object) {
                    Created(object) => Ok(NamespaceObjectCreated {
                        namespace: namespace.to_owned(),
                        object: object.to_owned(),
                    }),
                    Updated(object) => Ok(NamespaceObjectUpdated {
                        namespace: namespace.to_owned(),
                        object: object.to_owned(),
                    }),
                }
            }
            None => Ok(NamespaceNotFound { namespace_name }),
        }
    }

    fn _read_namespace(
        &mut self,
        namespace_name: String,
    ) -> Result<Outcome, Error> {
        match self.namespaces.get(&namespace_name) {
            Some(namespace) => {
                let namespace = namespace.to_owned();
                Ok(NamespaceFound { namespace })
            }
            None => Ok(NamespaceNotFound { namespace_name }),
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
                    let namespace = namespace.to_owned();
                    let object = object.to_owned();
                    Ok(NamespaceObjectFound { namespace, object })
                }
                None => Ok(NamespaceObjectNotFound {
                    namespace_name,
                    object_id,
                }),
            },
            None => Ok(NamespaceNotFound { namespace_name }),
        }
    }
}

#[cfg(test)]
mod tests {
    use im::hashmap::HashMap;

    use storage::Operation::*;
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
        let create_namespace = CreateNamespace {
            namespace: namespace.to_owned(),
        };

        let expected_outcome = NamespaceCreated {
            namespace: namespace.clone(),
        };
        let expected_log = vec![create_namespace.clone()];
        let mut expected_namespaces: Bag<Namespace> = HashMap::new();
        expected_namespaces
            .insert(namespace_name.to_owned(), namespace.to_owned());

        let outcome = storage.apply(create_namespace.clone()).unwrap();

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

        namespace.create_or_update_object(object.clone());

        let create_namespace_object = CreateNamespaceObject {
            namespace_name: namespace_name.to_owned(),
            object: object.clone(),
        };

        let expected_outcome = NamespaceObjectCreated {
            namespace: namespace.clone(),
            object: object.clone(),
        };
        let expected_log =
            vec![create_namespace.clone(), create_namespace_object.clone()];
        let mut expected_namespaces: Bag<Namespace> = HashMap::new();
        expected_namespaces
            .insert(namespace_name.to_owned(), namespace.to_owned());

        let outcome = storage.apply(create_namespace_object.clone()).unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.namespaces, expected_namespaces);

        // First namespace read (through Storage::apply):
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. doesn't change namespaces

        let get_namespace = ReadNamespace {
            namespace_name: namespace_name.to_owned(),
        };

        let expected_outcome = NamespaceFound {
            namespace: namespace.clone(),
        };
        let expected_log = vec![
            create_namespace.clone(),
            create_namespace_object.clone(),
            get_namespace.clone(),
        ];

        let outcome = storage.apply(get_namespace.clone()).unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.namespaces, expected_namespaces);

        // Second namespace read (through Storage::read_namespace):
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. doesn't change namespaces

        let expected_log = vec![
            create_namespace.clone(),
            create_namespace_object.clone(),
            get_namespace.clone(),
            get_namespace.clone(),
        ];

        let outcome = storage.get_namespace(namespace_name.to_owned()).unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.namespaces, expected_namespaces);

        // Namespace object read:
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. doesn't change namespaces

        let get_namespace_object = ReadNamespaceObject {
            namespace_name: namespace_name.to_owned(),
            object_id: object_id.to_owned(),
        };

        let expected_outcome = NamespaceObjectFound {
            namespace: namespace.clone(),
            object: object.clone(),
        };
        let expected_log = vec![
            create_namespace.clone(),
            create_namespace_object.clone(),
            get_namespace.clone(),
            get_namespace.clone(),
            get_namespace_object.clone(),
        ];

        let outcome = storage.apply(get_namespace_object.clone()).unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.namespaces, expected_namespaces);
    }
}
