use im::hashmap::HashMap;

pub type Bag<T> = HashMap<String, T>;

pub type Metadata = Bag<String>;

pub enum Outcome<T> {
    Created(T),
    Updated(T),
}
