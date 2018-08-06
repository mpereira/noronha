use uuid::Uuid;

pub fn make_id() -> String {
    Uuid::new_v4().to_string()
}
