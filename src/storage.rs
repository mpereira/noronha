// use std::sync::RwLock;
// use std::clone::Clone;

// use im::hashmap::HashMap;

// use cluster::Cluster;
// use configuration::{self, Configuration};
// use namespace::Namespace;
// use types::Bag;

// type Namespaces = Bag<Namespace>;

// type Storage<T> = Bag<T>;

// trait IStorage<T> {
//     fn new() -> Self;
// }

// lazy_static! {
//     pub static ref NAMESPACES: RwLock<Namespaces> = RwLock::new(HashMap::new());
// }

// impl<T> IStorage<T> for Storage<T>
// where T: std::clone::Clone
// {
//     fn new(&self) -> Self {
//     }
// }
