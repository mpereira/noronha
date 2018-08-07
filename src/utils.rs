use uuid::Uuid;

pub fn make_id() -> Uuid {
    Uuid::new_v4()
}

pub fn make_id_string() -> String {
    make_id().to_string()
}
