use im::hashmap::Entry::{Occupied, Vacant};

use keyspace::Keyspace;
use object::Object;
use types::Bag;

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    CreateOrUpdateKeyspace {
        keyspace: Keyspace,
    },
    CreateOrUpdateKeyspaceObject {
        keyspace_name: String,
        object: Object,
    },
    ReadKeyspace {
        keyspace_name: String,
    },
    ReadKeyspaceObject {
        keyspace_name: String,
        object_id: String,
    },
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Storage {
    log: Vec<Operation>,
    keyspaces: Bag<Keyspace>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    KeyspaceCreated(Keyspace),
    KeyspaceUpdated(Keyspace),
    KeyspaceFound(Keyspace),
    KeyspaceNotFound(String),
    KeyspaceObjectCreated(Object),
    KeyspaceObjectUpdated(Object),
    KeyspaceObjectFound(Object),
    KeyspaceObjectNotFound(String),
}

#[derive(Debug)]
pub struct Error;

use self::Operation::*;
use self::Outcome::*;

impl Keyspace {
    pub fn create_or_update_object(
        &mut self,
        object: Object,
    ) -> Result<Outcome, Error> {
        let object = object.clone();

        match object.data.get("id") {
            Some(object_id) => match self.objects.entry(object_id.to_owned()) {
                Occupied(mut entry) => {
                    entry.insert(object.to_owned());
                    let object = object.clone();
                    Ok(KeyspaceObjectUpdated(object))
                }
                Vacant(entry) => {
                    entry.insert(object.to_owned());
                    let object = object.clone();
                    Ok(KeyspaceObjectCreated(object))
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

    pub fn create_or_update_keyspace(
        &mut self,
        keyspace: Keyspace,
    ) -> Result<Outcome, Error> {
        self.apply(CreateOrUpdateKeyspace { keyspace })
    }

    pub fn read_keyspace(
        &mut self,
        keyspace_name: String,
    ) -> Result<Outcome, Error> {
        self.apply(ReadKeyspace { keyspace_name })
    }

    pub fn create_or_update_keyspace_object(
        &mut self,
        keyspace_name: String,
        object: Object,
    ) -> Result<Outcome, Error> {
        self.apply(CreateOrUpdateKeyspaceObject {
            keyspace_name,
            object,
        })
    }

    pub fn read_keyspace_object(
        &mut self,
        keyspace_name: String,
        object_id: String,
    ) -> Result<Outcome, Error> {
        self.apply(ReadKeyspaceObject {
            keyspace_name,
            object_id,
        })
    }

    fn apply(&mut self, operation: Operation) -> Result<Outcome, Error> {
        self.log.push(operation.clone());

        match operation {
            CreateOrUpdateKeyspace { keyspace } => {
                self._create_or_update_keyspace(keyspace)
            }
            CreateOrUpdateKeyspaceObject {
                keyspace_name,
                object,
            } => {
                self._create_or_update_keyspace_object(keyspace_name, object)
            }
            ReadKeyspace { keyspace_name } => {
                self._read_keyspace(keyspace_name)
            }
            ReadKeyspaceObject {
                keyspace_name,
                object_id,
            } => self._read_keyspace_object(keyspace_name, object_id),
        }
    }

    fn _create_or_update_keyspace(
        &mut self,
        keyspace: Keyspace,
    ) -> Result<Outcome, Error> {
        let keyspace = keyspace.clone();
        let keyspace_name = keyspace.metadata.get("name").unwrap().to_owned();

        match self.keyspaces.entry(keyspace_name) {
            Occupied(mut entry) => {
                entry.insert(keyspace.to_owned());
                Ok(KeyspaceUpdated(keyspace))
            }
            Vacant(entry) => {
                entry.insert(keyspace.clone());
                Ok(KeyspaceCreated(keyspace))
            }
        }
    }

    fn _create_or_update_keyspace_object(
        &mut self,
        keyspace_name: String,
        object: Object,
    ) -> Result<Outcome, Error> {
        match self.keyspaces.get_mut(&keyspace_name) {
            Some(keyspace) => keyspace.create_or_update_object(object),
            None => Ok(KeyspaceNotFound(keyspace_name)),
        }
    }

    fn _read_keyspace(
        &mut self,
        keyspace_name: String,
    ) -> Result<Outcome, Error> {
        match self.keyspaces.get(&keyspace_name) {
            Some(keyspace) => {
                let keyspace = keyspace.to_owned();
                Ok(KeyspaceFound(keyspace))
            }
            None => Ok(KeyspaceNotFound(keyspace_name)),
        }
    }

    fn _read_keyspace_object(
        &mut self,
        keyspace_name: String,
        object_id: String,
    ) -> Result<Outcome, Error> {
        match self.keyspaces.get(&keyspace_name) {
            Some(keyspace) => match keyspace.objects.get(&object_id) {
                Some(object) => {
                    let object = object.to_owned();
                    Ok(KeyspaceObjectFound(object))
                }
                None => Ok(KeyspaceObjectNotFound(object_id)),
            },
            None => Ok(KeyspaceNotFound(keyspace_name)),
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
        assert_eq!(storage.keyspaces, HashMap::new());
    }

    #[test]
    fn test_operations() {
        // Keyspace create:
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. inserts keyspace into keyspaces

        let mut storage = Storage::new();
        let keyspace_name = "people";
        let keyspace = Keyspace::make(keyspace_name);
        let create_or_update_keyspace = CreateOrUpdateKeyspace {
            keyspace: keyspace.to_owned(),
        };

        let expected_outcome = KeyspaceCreated(keyspace.clone());
        let expected_log = vec![create_or_update_keyspace.clone()];
        let mut expected_keyspaces: Bag<Keyspace> = HashMap::new();
        expected_keyspaces
            .insert(keyspace_name.to_owned(), keyspace.to_owned());

        let outcome =
            storage.apply(create_or_update_keyspace.clone()).unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.keyspaces, expected_keyspaces);

        // Keyspace object create:
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. inserts object into keyspace

        let mut keyspace = keyspace.clone();
        let object_id = "1";
        let object = Object::make(object_id, HashMap::new());

        keyspace
            .objects
            .insert(object_id.to_owned(), object.clone());

        let create_or_update_keyspace_object = CreateOrUpdateKeyspaceObject {
            keyspace_name: keyspace_name.to_owned(),
            object: object.clone(),
        };

        let expected_outcome = KeyspaceObjectCreated(object.clone());
        let expected_log = vec![
            create_or_update_keyspace.clone(),
            create_or_update_keyspace_object.clone(),
        ];
        let mut expected_keyspaces: Bag<Keyspace> = HashMap::new();
        expected_keyspaces
            .insert(keyspace_name.to_owned(), keyspace.to_owned());

        let outcome = storage
            .apply(create_or_update_keyspace_object.clone())
            .unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.keyspaces, expected_keyspaces);

        // First keyspace read (through Storage::apply):
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. doesn't change keyspaces

        let read_keyspace = ReadKeyspace {
            keyspace_name: keyspace_name.to_owned(),
        };

        let expected_outcome = KeyspaceFound(keyspace.clone());
        let expected_log = vec![
            create_or_update_keyspace.clone(),
            create_or_update_keyspace_object.clone(),
            read_keyspace.clone(),
        ];

        let outcome = storage.apply(read_keyspace.clone()).unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.keyspaces, expected_keyspaces);

        // Second keyspace read (through Storage::read_keyspace):
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. doesn't change keyspaces

        let expected_log = vec![
            create_or_update_keyspace.clone(),
            create_or_update_keyspace_object.clone(),
            read_keyspace.clone(),
            read_keyspace.clone(),
        ];

        let outcome =
            storage.read_keyspace(keyspace_name.to_owned()).unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.keyspaces, expected_keyspaces);

        // Keyspace object read:
        // 1. has successful outcome
        // 2. appends operation to log
        // 3. doesn't change keyspaces

        let read_keyspace_object = ReadKeyspaceObject {
            keyspace_name: keyspace_name.to_owned(),
            object_id: object_id.to_owned(),
        };

        let expected_outcome = KeyspaceObjectFound(object.clone());
        let expected_log = vec![
            create_or_update_keyspace.clone(),
            create_or_update_keyspace_object.clone(),
            read_keyspace.clone(),
            read_keyspace.clone(),
            read_keyspace_object.clone(),
        ];

        let outcome = storage.apply(read_keyspace_object.clone()).unwrap();

        assert_eq!(outcome, expected_outcome);
        assert_eq!(storage.log, expected_log);
        assert_eq!(storage.keyspaces, expected_keyspaces);
    }
}
