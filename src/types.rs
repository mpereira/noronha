use im::hashmap::HashMap;

pub type Bag<T> = HashMap<String, T>;

pub type Metadata = Bag<String>;

pub enum Operation<T> {
    Created(T),
    Updated(T),
}

// impl<T> Bag<T> {
//     pub fn new<T>() -> Self {
//         HashMap::new::<T>()
//     }
// }
