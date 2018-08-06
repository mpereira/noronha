use std::collections::HashMap;

pub type Bag<T> = HashMap<String, T>;

pub type Metadata = Bag<String>;

pub enum Operation<T> {
    Created(T),
    Updated(T),
}
