use std::sync::RwLock;

use storage::Storage;

lazy_static! {
    pub static ref STATE: RwLock<Storage> = RwLock::new(Storage::new());
}
